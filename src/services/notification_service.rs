use crate::config::Config;
use crate::database::Database;
use crate::errors::AppResult;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Service account credentials from JSON file
#[derive(Debug, Deserialize)]
struct ServiceAccountCredentials {
    client_email: String,
    private_key: String,
    project_id: Option<String>,
}

/// JWT claims for Google OAuth 2.0
#[derive(Debug, Serialize)]
struct GoogleJwtClaims {
    iss: String,   // Service account email
    scope: String, // FCM scope
    aud: String,   // Token endpoint
    iat: u64,      // Issued at
    exp: u64,      // Expiration
}

/// Google OAuth token response
#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    expires_in: u64,
}

/// Cached access token
struct CachedToken {
    token: String,
    expires_at: u64,
}

pub struct NotificationService {
    db: Database,
    http_client: Client,
    project_id: Option<String>,
    credentials: Option<ServiceAccountCredentials>,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
}

impl NotificationService {
    pub fn new(db: Database, config: &Config) -> Self {
        let (project_id, credentials) = Self::load_credentials(config);

        if project_id.is_none() {
            info!("FCM not configured - push notifications disabled");
        } else {
            info!(
                "FCM V1 API configured for project: {}",
                project_id.as_ref().unwrap()
            );
        }

        Self {
            db,
            http_client: Client::new(),
            project_id,
            credentials,
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    fn load_credentials(config: &Config) -> (Option<String>, Option<ServiceAccountCredentials>) {
        let service_account_path = match &config.fcm_service_account_path {
            Some(path) => path,
            None => return (None, None),
        };

        let json_content = match fs::read_to_string(service_account_path) {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to read service account file: {}", e);
                return (None, None);
            }
        };

        let credentials: ServiceAccountCredentials = match serde_json::from_str(&json_content) {
            Ok(creds) => creds,
            Err(e) => {
                warn!("Failed to parse service account JSON: {}", e);
                return (None, None);
            }
        };

        // Use project_id from config or from credentials file
        let project_id = config
            .fcm_project_id
            .clone()
            .or_else(|| credentials.project_id.clone());

        (project_id, Some(credentials))
    }

    /// Get or refresh OAuth 2.0 access token
    async fn get_access_token(&self) -> Option<String> {
        let credentials = self.credentials.as_ref()?;

        // Check if cached token is still valid (with 5 min buffer)
        {
            let cached = self.cached_token.read().await;
            if let Some(ref token) = *cached {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if token.expires_at > now + 300 {
                    return Some(token.token.clone());
                }
            }
        }

        // Create new JWT for token request
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = GoogleJwtClaims {
            iss: credentials.client_email.clone(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour
        };

        let header = Header::new(Algorithm::RS256);
        let key = match EncodingKey::from_rsa_pem(credentials.private_key.as_bytes()) {
            Ok(k) => k,
            Err(e) => {
                error!("Failed to parse private key: {}", e);
                return None;
            }
        };

        let jwt = match encode(&header, &claims, &key) {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to create JWT: {}", e);
                return None;
            }
        };

        // Exchange JWT for access token
        let response = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                error!("Token request failed: {}", e);
                return None;
            }
        };

        let token_response: GoogleTokenResponse = match response.json().await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to parse token response: {}", e);
                return None;
            }
        };

        // Cache the token
        {
            let mut cached = self.cached_token.write().await;
            *cached = Some(CachedToken {
                token: token_response.access_token.clone(),
                expires_at: now + token_response.expires_in,
            });
        }

        Some(token_response.access_token)
    }

    /// Send push notification to all users who bookmarked a novel
    pub async fn notify_new_chapter(
        &self,
        novel_id: &str,
        novel_title: &str,
        chapter_num: i32,
        chapter_title: &str,
        chapter_id: &str,
    ) -> AppResult<()> {
        let project_id = match &self.project_id {
            Some(id) => id,
            None => {
                info!("FCM not configured, skipping push notification");
                return Ok(());
            }
        };

        let access_token = match self.get_access_token().await {
            Some(token) => token,
            None => {
                error!("Failed to get FCM access token");
                return Ok(());
            }
        };

        // Get all FCM tokens for users who bookmarked this novel
        let tokens = self.get_bookmark_user_tokens(novel_id).await?;

        if tokens.is_empty() {
            info!(
                "No users with FCM tokens have bookmarked novel {}",
                novel_id
            );
            return Ok(());
        }

        info!(
            "Sending push notification to {} users for novel {}",
            tokens.len(),
            novel_id
        );

        let notification_title = format!("📖 {}", novel_title);
        let notification_body = format!(
            "Chapter {} - {} is now available!",
            chapter_num, chapter_title
        );

        for token in tokens {
            if let Err(e) = self
                .send_fcm_v1_notification(
                    project_id,
                    &access_token,
                    &token,
                    &notification_title,
                    &notification_body,
                    novel_id,
                    chapter_id,
                )
                .await
            {
                error!(
                    "Failed to send notification to token {}: {:?}",
                    &token[..20.min(token.len())],
                    e
                );
            }
        }

        Ok(())
    }

    /// Get FCM tokens for all users who bookmarked a specific novel
    async fn get_bookmark_user_tokens(&self, novel_id: &str) -> AppResult<Vec<String>> {
        let tokens = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT u.fcm_token
            FROM "Bookmark" b
            INNER JOIN "User" u ON b.user_id = u.id
            WHERE b.book_id = $1 
            AND u.fcm_token IS NOT NULL 
            AND u.fcm_token != ''
            "#,
        )
        .bind(novel_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(tokens)
    }

    /// Send FCM notification using HTTP V1 API
    async fn send_fcm_v1_notification(
        &self,
        project_id: &str,
        access_token: &str,
        device_token: &str,
        title: &str,
        body: &str,
        novel_id: &str,
        chapter_id: &str,
    ) -> AppResult<()> {
        let payload = serde_json::json!({
            "message": {
                "token": device_token,
                "notification": {
                    "title": title,
                    "body": body
                },
                "data": {
                    "novel_id": novel_id,
                    "chapter_id": chapter_id,
                    "click_action": "FLUTTER_NOTIFICATION_CLICK"
                },
                "android": {
                    "priority": "high",
                    "notification": {
                        "icon": "ic_notification",
                        "color": "#6366F1",
                        "sound": "default"
                    }
                }
            }
        });

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            project_id
        );

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("FCM V1 notification sent successfully");
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("FCM V1 notification failed: {}", error_text);
        }

        Ok(())
    }
}
