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
        AuthService::new(state.db.clone(), jwt_service)
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

    #[instrument(skip(state, cookies))]
    pub async fn refresh_token(
        State(state): State<AppState>,
        cookies: Cookies,
    ) -> Result<Json<AuthResponseWithoutTokens>, AppError> {
        info!("Attempting to refresh token");

        // Get refresh token from cookie
        let refresh_token = cookies
            .get("refresh_token")
            .ok_or_else(|| {
                warn!("Refresh token not found in cookies");
                AppError::Unauthorized
            })?
            .value()
            .to_string();

        let service = Self::create_service(&state);

        match service.refresh_token(&refresh_token).await {
            Ok(auth) => {
                info!(
                    user_id = %auth.user.id,
                    username = %auth.user.username,
                    "Token refreshed successfully"
                );

                // Set new HTTP-only cookies
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

                Ok(Json(AuthResponseWithoutTokens::success(auth.user)))
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
}
