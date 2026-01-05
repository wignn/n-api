use crate::errors::ConfigError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret_key: String,
    pub jwt_expire_in: i64,
    pub jwt_refresh_expire_in: i64,
    pub api_key: String,
    pub email: String,
    pub password: String,
    pub cloudflare_api_token: String,
    pub cloudflare_secret: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub cdn_url: String,
    pub port: String,
    // FCM V1 API (optional)
    pub fcm_project_id: Option<String>,
    pub fcm_service_account_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: Self::get_env("DATABASE_URL")?,
            redis_url: Self::get_env("REDIS_URL")?,
            jwt_secret_key: Self::get_env("JWT_SECRET_KEY")?,
            jwt_expire_in: Self::get_env_i64("JWT_ACCESS_EXPIRES_IN")?,
            jwt_refresh_expire_in: Self::get_env_i64("JWT_REFRESH_EXPIRES_IN")?,
            api_key: Self::get_env("API_KEY")?,
            email: Self::get_env("EMAIL")?,
            password: Self::get_env("PASSWORD")?,
            cloudflare_api_token: Self::get_env("AWS_ACCESS_KEY_ID")?,
            cloudflare_secret: Self::get_env("AWS_SECRET_ACCESS_KEY")?,
            s3_endpoint: Self::get_env("AWS_ENDPOINT")?,
            s3_bucket: Self::get_env("AWS_BUCKET")?,
            cdn_url: Self::get_env("AWS_URL")?,
            port: Self::get_env("PORT")?,
            // FCM V1 API (optional - app still works without these)
            fcm_project_id: Self::get_env_optional("FCM_PROJECT_ID"),
            fcm_service_account_path: Self::get_env_optional("GOOGLE_APPLICATION_CREDENTIALS"),
        })
    }

    fn get_env(key: &str) -> Result<String, ConfigError> {
        env::var(key).map_err(|_| ConfigError::MissingVar(key.to_string()))
    }

    fn get_env_i64(key: &str) -> Result<i64, ConfigError> {
        let val = env::var(key).map_err(|_| ConfigError::MissingVar(key.to_string()))?;
        val.parse::<i64>()
            .map_err(|e| ConfigError::ParseError(key.to_string(), e))
    }

    fn get_env_optional(key: &str) -> Option<String> {
        env::var(key).ok().filter(|v| !v.is_empty())
    }

    pub async fn test_database_connection(&self) -> Result<(), String> {
        println!("Testing database connection...");

        match PgPool::connect(&self.database_url).await {
            Ok(pool) => {
                println!("✓ Database connection successful!");

                match sqlx::query_scalar::<_, i64>("SELECT 1")
                    .fetch_one(&pool)
                    .await
                {
                    Ok(_) => {
                        println!("✓ Test query successful!");
                        pool.close().await;
                        Ok(())
                    }
                    Err(e) => {
                        pool.close().await;
                        Err(format!("Test query failed: {}", e))
                    }
                }
            }
            Err(e) => Err(format!("Database connection failed: {}", e)),
        }
    }
}
