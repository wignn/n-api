use crate::middleware::auth::AuthUser;
use crate::models::auth_model::{AuthResponse, LoginDto, RegisterDto};
use crate::models::response_model::ApiResponse;

use crate::services::auth_service::AuthService;
use crate::utils::jwt::JwtService;
use crate::{errors::AppError, AppState};
use axum::Extension;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use serde::Deserialize;

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

    pub async fn register(
        State(state): State<AppState>,
        Json(request): Json<RegisterDto>,
    ) -> Result<impl IntoResponse, AppError> {
        let service = Self::create_service(&state);
        let auth = service.register(request).await?;

        Ok((StatusCode::CREATED, Json(AuthResponse::success(auth))))
    }

    pub async fn login(
        State(state): State<AppState>,
        Json(request): Json<LoginDto>,
    ) -> Result<Json<AuthResponse>, AppError> {
        let service = Self::create_service(&state);
        let auth = service.login(request).await?;

        Ok(Json(AuthResponse::success(auth)))
    }

    pub async fn refresh_token(
        State(state): State<AppState>,
        Json(request): Json<RefreshTokenRequest>,
    ) -> Result<Json<AuthResponse>, AppError> {
        let service = Self::create_service(&state);
        let auth = service.refresh_token(&request.refresh_token).await?;

        Ok(Json(AuthResponse::success(auth)))
    }

    pub async fn me(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
    ) -> Result<Json<ApiResponse<crate::models::user_model::SafeUser>>, AppError> {
        
        let service = Self::create_service(&state);
        let user = service.get_user_by_id(auth_user.id.as_str()).await?;
       
        Ok(Json(ApiResponse::success(user)))
    }
}
