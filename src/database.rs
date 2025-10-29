use sqlx::{postgres::PgPoolOptions, PgPool};
use anyhow::Result;
use crate::redis::RedisClient;


#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
    pub redis: RedisClient,
}

impl Database {
    pub async fn new(database_url: &str, redis_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(100)
            .min_connections(10)
            .connect(database_url)
            .await?;

        let redis = RedisClient::new(redis_url).await?;

        Ok(Self { pool, redis })
    }

    pub async fn test_connection(&self) -> Result<()> {
        tracing::info!("Testing database connection...");
        
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        
        tracing::info!("Database connection test successful");
        
        self.redis.test_connection().await?;
        
        Ok(())
    }
}

pub async fn get_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}
