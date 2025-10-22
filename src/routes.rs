use crate::handlers::book_handler::BookHandler;
use crate::AppState;
use axum::{
    routing::{delete, get, post, put},
    response::IntoResponse,
    extract::State,
    http::StatusCode,
    Json, Router,
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub fn create_routes(app_state: AppState, cors: CorsLayer) -> Router {
    Router::new()
        .nest("/api", api_route())
        .route("/healthy", get(health_checker_handler))
        .route("/db-health", get(db_health_check))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(app_state)
}

fn api_route() -> Router<AppState> {
    Router::new().nest("/books", book_route())
}

fn book_route() -> Router<AppState> {
    Router::new().route("/", get(BookHandler::get_books))
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, Postgres, and Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn db_health_check(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.test_connection().await {
        Ok(_) => {
            let json_response = serde_json::json!({
                "status": "success",
                "message": "Database connection is healthy",
                "database": "connected"
            });
            (StatusCode::OK, Json(json_response))
        }
        Err(e) => {
            let json_response = serde_json::json!({
                "status": "error",
                "message": format!("Database connection failed: {}", e),
                "database": "disconnected"
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_response))
        }
    }
}
