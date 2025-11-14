use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use http::StatusCode;
use crate::AppState;

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, Postgres, and Axum";

    Json(serde_json::json!({
        "status": "success",
        "message": MESSAGE
    }))
}

pub async fn db_health_check(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.test_connection().await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "message": "Database connection is healthy",
                "database": "connected"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Database connection failed: {}", e),
                "database": "disconnected"
            })),
        ),
    }
}