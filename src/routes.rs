use axum::Json;
use axum::response::IntoResponse;
use axum::routing::{Router, get, post};
use crate::AppState;
use crate::handlers::book_handler::BookHandler;

pub fn create_routes(app_state: AppState) -> Router {
    Router::new()

        .nest("/api", api_route(app_state.clone()))
        .route("healthy", get(health_checker_handler()))
        .with_state(app_state)
}


fn api_route(state:AppState) ->Router<AppState> {
    Router::new()
        .nest("/books", book_route(state.clone()))
}

fn book_route(state: AppState) ->Router<AppState> {
    Router::new()
        .route("/", get(BookHandler::get_books))
        .route("/{id}", get(BookHandler::get_book))
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, Postgres,and Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}