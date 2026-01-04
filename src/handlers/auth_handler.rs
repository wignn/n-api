use crate::middleware::auth::AuthUser;
use crate::models::auth_model::{AuthResponseWithoutTokens, LoginDto, RegisterDto};
use crate::models::response_model::ApiResponse;
use crate::services::auth_service::AuthService;
use crate::utils::jwt::JwtService;
use crate::{errors::AppError, AppState};
use axum::Extension;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use time::Duration;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info, instrument, warn};

pub struct AuthHandler;

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

impl AuthHandler {
    fn create_service(state: &AppState) -> AuthService {
        let jwt_service = JwtService::new(
            &state.config.jwt_secret_key,
            state.config.jwt_expire_in,
            state.config.jwt_refresh_expire_in,
        );
        AuthService::new(state.db.clone(), jwt_service, state.storage.clone())
    }

    #[instrument(skip(state, cookies, request), fields(
        username = %request.username,
        email = %request.email
    ))]
    pub async fn register(
        State(state): State<AppState>,
        cookies: Cookies,
        Json(request): Json<RegisterDto>,
    ) -> Result<impl IntoResponse, AppError> {
        info!("Attempting user registration");

        let service = Self::create_service(&state);

        match service.register(request).await {
            Ok(auth) => {
                info!(
                    user_id = %auth.user.id,
                    username = %auth.user.username,
                    "User registered successfully"
                );

                // Set HTTP-only cookies
                let mut access_cookie = Cookie::new("access_token", auth.access_token.clone());
                access_cookie.set_http_only(true);
                access_cookie.set_secure(true);
                access_cookie.set_path("/");
                access_cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
                access_cookie.set_max_age(Duration::minutes(state.config.jwt_expire_in as i64));

                let mut refresh_cookie = Cookie::new("refresh_token", auth.refresh_token.clone());
                refresh_cookie.set_http_only(true);
                refresh_cookie.set_secure(true);
                refresh_cookie.set_path("/");
                refresh_cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
                refresh_cookie
                    .set_max_age(Duration::minutes(state.config.jwt_refresh_expire_in as i64));

                cookies.add(access_cookie);
                cookies.add(refresh_cookie);

                info!("Cookies set successfully for new user");

                // Return full auth response with tokens (same as login)
                Ok((
                    StatusCode::CREATED,
                    Json(crate::models::auth_model::AuthResponse::success(auth)),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to register user");
                Err(e)
            }
        }
    }

    #[instrument(skip(state, cookies, request), fields(
        email = %request.email
    ))]
    pub async fn login(
        State(state): State<AppState>,
        cookies: Cookies,
        Json(request): Json<LoginDto>,
    ) -> Result<Json<crate::models::auth_model::AuthResponse>, AppError> {
        info!("Attempting user login");

        let service = Self::create_service(&state);

        match service.login(request).await {
            Ok(auth) => {
                info!(
                    user_id = %auth.user.id,
                    username = %auth.user.username,
                    "User logged in successfully"
                );

                // Set HTTP-only cookies
                let mut access_cookie = Cookie::new("access_token", auth.access_token.clone());
                access_cookie.set_http_only(true);
                access_cookie.set_secure(true);
                access_cookie.set_path("/");
                access_cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
                access_cookie.set_max_age(Duration::minutes(state.config.jwt_expire_in as i64));

                let mut refresh_cookie = Cookie::new("refresh_token", auth.refresh_token.clone());
                refresh_cookie.set_http_only(true);
                refresh_cookie.set_secure(true);
                refresh_cookie.set_path("/");
                refresh_cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
                refresh_cookie
                    .set_max_age(Duration::minutes(state.config.jwt_refresh_expire_in as i64));

                cookies.add(access_cookie);
                cookies.add(refresh_cookie);

                info!("Cookies set successfully for login");

                Ok(Json(crate::models::auth_model::AuthResponse::success(auth)))
            }
            Err(e) => {
                warn!(error = ?e, "Login attempt failed");
                Err(e)
            }
        }
    }

    #[instrument(skip(state, cookies, body))]
    pub async fn refresh_token(
        State(state): State<AppState>,
        cookies: Cookies,
        body: Option<Json<RefreshTokenRequest>>,
    ) -> Result<Json<crate::models::auth_model::AuthResponse>, AppError> {
        info!("Attempting to refresh token");

        // Try to get refresh token from body first (for mobile), then from cookie (for web)
        let refresh_token = if let Some(Json(req)) = body {
            req.refresh_token
        } else {
            cookies
                .get("refresh_token")
                .ok_or_else(|| {
                    warn!("Refresh token not found in body or cookies");
                    AppError::Unauthorized
                })?
                .value()
                .to_string()
        };

        let service = Self::create_service(&state);

        match service.refresh_token(&refresh_token).await {
            Ok(auth) => {
                info!(
                    user_id = %auth.user.id,
                    username = %auth.user.username,
                    "Token refreshed successfully"
                );

                // Set new HTTP-only cookies (for web clients)
                let mut access_cookie = Cookie::new("access_token", auth.access_token.clone());
                access_cookie.set_http_only(true);
                access_cookie.set_secure(true);
                access_cookie.set_path("/");
                access_cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
                access_cookie.set_max_age(Duration::minutes(state.config.jwt_expire_in as i64));

                let mut refresh_cookie = Cookie::new("refresh_token", auth.refresh_token.clone());
                refresh_cookie.set_http_only(true);
                refresh_cookie.set_secure(true);
                refresh_cookie.set_path("/");
                refresh_cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
                refresh_cookie
                    .set_max_age(Duration::minutes(state.config.jwt_refresh_expire_in as i64));

                cookies.add(access_cookie);
                cookies.add(refresh_cookie);

                info!("New cookies set successfully");

                // Return full auth response with tokens (for mobile clients)
                Ok(Json(crate::models::auth_model::AuthResponse::success(auth)))
            }
            Err(e) => {
                error!(error = ?e, "Failed to refresh token");
                Err(e)
            }
        }
    }

    #[instrument(skip(state), fields(
        user_id = %auth_user.id
    ))]
    pub async fn me(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
    ) -> Result<Json<ApiResponse<crate::models::user_model::SafeUser>>, AppError> {
        info!("Fetching current user profile");

        let service = Self::create_service(&state);

        match service.get_user_by_id(auth_user.id.as_str()).await {
            Ok(user) => {
                info!("User profile fetched successfully");
                Ok(Json(ApiResponse::success(user)))
            }
            Err(e) => {
                error!(error = ?e, "Failed to fetch user profile");
                Err(e)
            }
        }
    }

    #[instrument(skip(cookies))]
    pub async fn logout(cookies: Cookies) -> Result<Json<ApiResponse<String>>, AppError> {
        info!("User logging out");

        // Remove cookies
        let mut access_cookie = Cookie::new("access_token", "");
        access_cookie.set_http_only(true);
        access_cookie.set_path("/");
        access_cookie.set_max_age(Duration::seconds(0));

        let mut refresh_cookie = Cookie::new("refresh_token", "");
        refresh_cookie.set_http_only(true);
        refresh_cookie.set_path("/");
        refresh_cookie.set_max_age(Duration::seconds(0));

        cookies.add(access_cookie);
        cookies.add(refresh_cookie);

        info!("User logged out successfully, cookies cleared");

        Ok(Json(ApiResponse::success(
            "Logged out successfully".to_string(),
        )))
    }

    #[instrument(skip(state, request), fields(user_id = %auth_user.id))]
    pub async fn update_profile(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<crate::models::auth_model::UpdateProfileDto>,
    ) -> Result<Json<ApiResponse<crate::models::user_model::SafeUser>>, AppError> {
        info!("Updating user profile");

        let service = Self::create_service(&state);

        match service.update_profile(&auth_user.id, request).await {
            Ok(user) => {
                info!("Profile updated successfully");
                Ok(Json(ApiResponse::success(user)))
            }
            Err(e) => {
                error!(error = ?e, "Failed to update profile");
                Err(e)
            }
        }
    }

    #[instrument(skip(state, request), fields(user_id = %auth_user.id))]
    pub async fn change_password(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<crate::models::auth_model::ChangePasswordDto>,
    ) -> Result<Json<ApiResponse<String>>, AppError> {
        info!("Changing user password");

        let service = Self::create_service(&state);

        match service
            .change_password(
                &auth_user.id,
                &request.current_password,
                &request.new_password,
            )
            .await
        {
            Ok(_) => {
                info!("Password changed successfully");
                Ok(Json(ApiResponse::success(
                    "Password changed successfully".to_string(),
                )))
            }
            Err(e) => {
                error!(error = ?e, "Failed to change password");
                Err(e)
            }
        }
    }

    #[instrument(skip(state, multipart), fields(user_id = %auth_user.id))]
    pub async fn upload_avatar(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        mut multipart: axum::extract::Multipart,
    ) -> Result<Json<ApiResponse<String>>, AppError> {
        info!("Uploading avatar");

        let service = Self::create_service(&state);

        // Extract file from multipart
        let mut file_bytes: Option<Vec<u8>> = None;
        let mut content_type: Option<String> = None;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read multipart: {}", e)))?
        {
            let name = field.name().unwrap_or("").to_string();
            if name == "avatar" || name == "file" {
                content_type = field.content_type().map(|s| s.to_string());
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("Failed to read file: {}", e)))?
                        .to_vec(),
                );
                break;
            }
        }

        let bytes =
            file_bytes.ok_or_else(|| AppError::BadRequest("No file uploaded".to_string()))?;
        let ct = content_type.unwrap_or_else(|| "image/jpeg".to_string());

        // Validate content type
        if !ct.starts_with("image/") {
            return Err(AppError::BadRequest(
                "Only image files are allowed".to_string(),
            ));
        }

        // Limit file size (5MB)
        if bytes.len() > 5 * 1024 * 1024 {
            return Err(AppError::BadRequest(
                "File size must be less than 5MB".to_string(),
            ));
        }

        match service.upload_avatar(&auth_user.id, bytes, &ct).await {
            Ok(url) => {
                info!("Avatar uploaded successfully");
                Ok(Json(ApiResponse::success(url)))
            }
            Err(e) => {
                error!(error = ?e, "Failed to upload avatar");
                Err(e)
            }
        }
    }
}
