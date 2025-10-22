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
        println!("Testing database connection...");

        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&self.pool)
            .await?;

        println!("✓ Database connection test successful! Result: {}", result.0);

        // Test if Book table exists
        let table_exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS (
                SELECT FROM information_schema.tables
                WHERE table_schema = 'public'
                AND table_name = 'Book'
            )"#
        )
        .fetch_one(&self.pool)
        .await?;

        if table_exists {
            println!("✓ Table 'Book' exists in database");

            // Count books in table
            let book_count: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM "Book""#)
                .fetch_one(&self.pool)
                .await?;
            println!("✓ Found {} books in database", book_count);
        } else {
            println!("⚠ Warning: Table 'Book' does not exist in database");
        }

        Ok(())
    }
}

pub async fn get_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}
