pub mod config;
pub mod database;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod redis;
pub mod routes;
pub mod services;
pub mod utils;

use config::Config;
use database::Database;
use services::storage_service::StorageService;
use std::sync::Arc;

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub db: Database,
    pub config: Config,
    pub storage: StorageService,
}
