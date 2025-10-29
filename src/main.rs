use std::sync::Arc;
use axum::http::Method;
use dotenvy::dotenv;
use tower_http::cors::{CorsLayer, Any};
use novel_api::config::Config;
use novel_api::database::Database;
use novel_api::{routes, AppStateInner};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Starting application...");

    let config = Config::from_env().expect("Failed to load env");

    let port = config.port.clone();

    tracing::info!("Connecting to database...");
    let db = Database::new(&config.database_url, &config.redis_url)
        .await
        .expect("Failed to connect to database");

    if let Err(e) = db.test_connection().await {
        tracing::error!("Database test failed: {}", e);
        tracing::warn!("Please check your DATABASE_URL and ensure the database is running");
    } else {
        tracing::info!("Database connection successful");
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::PATCH, Method::PUT, Method::DELETE, Method::POST])
        .allow_headers(Any);

    let state = Arc::new(AppStateInner {
       db,
        config,
    });

    let app = routes::create_routes(state, cors);

    tracing::info!("Binding to port {}...", port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    tracing::info!("Server running on port {}", port);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
