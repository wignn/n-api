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

    let config = Config::from_env().expect("Failed to load env");

    let port = config.port.clone();

    let db = Database::new(&config.database_url)
        .await
        .expect("Failed to connect to database");

    if let Err(e) = db.test_connection().await {
        eprintln!("Database test failed: {}", e);
        eprintln!("Please check your DATABASE_URL and ensure the database is running");
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

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    println!("Server running on http://0.0.0.0:{}", port);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
