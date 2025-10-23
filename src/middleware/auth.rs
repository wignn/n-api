use crate::errors::AppError;
use crate::models::user_model::Role;
use crate::utils::jwt::{Claims, JwtService};
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
}

impl AuthUser {
    pub fn from_claims(claims: Claims) -> Result<Self, AppError> {
        let id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(Self {
            id,
            email: claims.email,
            role: claims.role,
        })
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {

    
    let jwt_service = JwtService::new(
        &state.config.jwt_secret_key,
        state.config.jwt_expire_in,
        state.config.jwt_refresh_expire_in,
    );

    let token = extract_token_from_header(&headers)?;
    let claims = jwt_service.verify_access_token(&token)?;
    let auth_user = AuthUser::from_claims(claims)?;

    // Insert auth user into request extensions
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

fn extract_token_from_header(headers: &HeaderMap) -> Result<String, AppError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized);
    }

    Ok(auth_header[7..].to_string())
}

#[macro_export]
macro_rules! require_role {
    ($auth_user:expr, $required_role:expr) => {
        match ($auth_user.role.clone(), $required_role) {
            (crate::models::user::UserRole::Admin, _) => Ok(()),
            (role, required) if role == required => Ok(()),
            _ => Err(crate::errors::AppError::Forbidden),
        }
    };
}
