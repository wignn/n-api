use crate::errors::AppError;
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

const API_KEY_HEADER: &str = "x-api-key";

pub async fn api_key_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_key = extract_api_key_from_header(&headers)?;
    
    // Verify API key
    if api_key != state.config.api_key {
        return Err(AppError::Unauthorized);
    }

    Ok(next.run(request).await)
}

fn extract_api_key_from_header(headers: &HeaderMap) -> Result<String, AppError> {
    let api_key = headers
        .get(API_KEY_HEADER)
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::Unauthorized)?;

    Ok(api_key.to_string())
}
