use crate::handlers::auth_handler::AuthHandler;
use crate::handlers::book_handler::BookHandler;
use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use crate::handlers::genre_handler::GenreHandler;

pub fn create_routes(app_state: AppState, cors: CorsLayer) -> Router {
    Router::new()
        .nest("/api", api_route(app_state.clone()))
        .route("/healthy", get(health_checker_handler))
        .route("/db-health", get(db_health_check))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(app_state)
}

fn api_route(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_route(app_state.clone()))
        .nest("/books", book_route(app_state.clone()))
        .nest("/genre",  genre_route(app_state))

}

fn auth_route(app_state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/me", get(AuthHandler::me))
        .route("/refresh", post(AuthHandler::refresh_token))
        .layer(axum::middleware::from_fn_with_state(
            app_state,
            crate::middleware::auth::auth_middleware
        ));

    Router::new()
        .route("/register", post(AuthHandler::register))
        .route("/login", post(AuthHandler::login))
        .merge(protected)
}

fn book_route(app_state: AppState) -> Router<AppState> {
    // Public 
    let public_routes = Router::new()
        .route("/", get(BookHandler::get_books))
        .route("/{id}", get(BookHandler::get_book))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            crate::middleware::api_key::api_key_middleware
        ));

    // Protected 
    let protected_routes = Router::new()
        .route("/", post(BookHandler::create_book))
        .route(
            "/{id}",
            put(BookHandler::update_book).delete(BookHandler::delete_book),
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state,
            crate::middleware::auth::auth_middleware
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}

fn genre_route(app_state: AppState) -> Router<AppState>  {
    let public_routes =  Router::new()
        .route("/", get(GenreHandler::get_genres))
        .route("/{id}", get(GenreHandler::get_genre))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            crate::middleware::api_key::api_key_middleware
        ));

    let protected_routes = Router::new()
        .route("/", post(GenreHandler::create_genre))
        .route("/{id}", put(GenreHandler::update_genre).delete(GenreHandler::delete_genre))
        .layer(axum::middleware::from_fn_with_state(
            app_state,
            crate::middleware::auth::auth_middleware
        ));
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
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
