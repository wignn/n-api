pub mod config;
pub mod database;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;
pub mod middleware;
pub mod redis;

use std::sync::Arc;
use config::Config;
use database::Database;


pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub db: Database,
    pub config: Config,
}