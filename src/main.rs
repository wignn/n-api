use axum::http::{header, HeaderValue, Method};
use dotenvy::dotenv;
use novel_api::config::Config;
use novel_api::database::Database;
use novel_api::{routes, AppStateInner};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_level(true)
                .with_thread_names(true),
        )
        .with(tracing_subscriber::EnvFilter::from_default_env())
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

    // CORS configuration - support credentials with specific origins
    let allowed_origins = [
        "http://localhost:5173",
        "http://localhost:3000",
        "https://fenrir-realm.vercel.app",
    ];

    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .map(|origin| origin.parse::<HeaderValue>().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::POST,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::HeaderName::from_static("x-api-key"),
        ])
        .allow_credentials(true);

    let state = Arc::new(AppStateInner { db, config });

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
