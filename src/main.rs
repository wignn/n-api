use std::sync::Arc;
use dotenvy::dotenv;
use novel_api::config::Config;
use novel_api::database::Database;
use novel_api::routes::create_routes;
use novel_api::AppStateInner;
use std::fmt::format;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::from_env().expect("Failed to load env");

    let port = config.port.clone();

    let db = Database::new(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let state = Arc::new(AppStateInner { db, config });

    let app = create_routes(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",port))
        .await
        .expect("Failed to bind to port 3000");

    println!("Server running on http://0.0.0.0: {}", port);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
