use sqlx::{postgres::PgPoolOptions, PgPool};
use anyhow::Result;


#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn test_connection(&self) -> Result<()> {
        tracing::info!("Testing database connection...");
        
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        
        tracing::info!("Database connection test successful");
        Ok(())
    }
}

pub async fn get_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}
