pub mod handlers;
pub mod models;
pub mod services;
pub mod config;
pub mod routes;
pub mod errors;
pub mod database;
pub mod utils;

use std::sync::Arc;
use database::Database;
use config::Config;

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub db: Database,
    pub config: Config,
}