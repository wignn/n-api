use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
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
}