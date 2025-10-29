use redis::{aio::ConnectionManager, Client};
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct RedisClient {
    pub connection: ConnectionManager,
}

impl std::fmt::Debug for RedisClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisClient")
            .field("connection", &"ConnectionManager")
            .finish()
    }
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;
        
        Ok(Self { connection })
    }

    /// Test
    pub async fn test_connection(&self) -> Result<()> {
        tracing::info!("Testing Redis connection...");
        
        let mut conn = self.connection.clone();
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await?;
        
        tracing::info!("Redis connection test successful");
        Ok(())
    }

    /// Set string value dengan expiration (dalam detik)
    pub async fn set_ex(&self, key: &str, value: &str, ttl: i64) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl)
            .arg(value)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Set value tanpa expiration
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Get string value
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.connection.clone();
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Delete key
    pub async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("DEL")
            .arg(key)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Set JSON value dengan expiration
    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T, ttl: i64) -> Result<()> {
        let json = serde_json::to_string(value)?;
        self.set_ex(key, &json, ttl).await
    }

    /// Get JSON value
    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        if let Some(json_str) = self.get(key).await? {
            let value: T = serde_json::from_str(&json_str)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.connection.clone();
        let result: i32 = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result > 0)
    }

    /// Set expiration pada key yang sudah ada
    pub async fn expire(&self, key: &str, ttl: i64) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("EXPIRE")
            .arg(key)
            .arg(ttl)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Get TTL dari key
    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let mut conn = self.connection.clone();
        let result: i64 = redis::cmd("TTL")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Increment value
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.connection.clone();
        let result: i64 = redis::cmd("INCR")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Decrement value
    pub async fn decr(&self, key: &str) -> Result<i64> {
        let mut conn = self.connection.clone();
        let result: i64 = redis::cmd("DECR")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Get multiple keys
    pub async fn mget(&self, keys: Vec<&str>) -> Result<Vec<Option<String>>> {
        let mut conn = self.connection.clone();
        let result: Vec<Option<String>> = redis::cmd("MGET")
            .arg(keys)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Set multiple key-value pairs
    pub async fn mset(&self, pairs: Vec<(&str, &str)>) -> Result<()> {
        let mut conn = self.connection.clone();
        let mut cmd = redis::cmd("MSET");
        for (key, value) in pairs {
            cmd.arg(key).arg(value);
        }
        cmd.query_async::<()>(&mut conn).await?;
        Ok(())
    }

    /// Add to set
    pub async fn sadd(&self, key: &str, member: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("SADD")
            .arg(key)
            .arg(member)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Check if member exists in set
    pub async fn sismember(&self, key: &str, member: &str) -> Result<bool> {
        let mut conn = self.connection.clone();
        let result: i32 = redis::cmd("SISMEMBER")
            .arg(key)
            .arg(member)
            .query_async(&mut conn)
            .await?;
        Ok(result > 0)
    }

    /// Get all members of set
    pub async fn smembers(&self, key: &str) -> Result<Vec<String>> {
        let mut conn = self.connection.clone();
        let result: Vec<String> = redis::cmd("SMEMBERS")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Remove from set
    pub async fn srem(&self, key: &str, member: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("SREM")
            .arg(key)
            .arg(member)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Push to list (left)
    pub async fn lpush(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("LPUSH")
            .arg(key)
            .arg(value)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Push to list (right)
    pub async fn rpush(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("RPUSH")
            .arg(key)
            .arg(value)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Get list range
    pub async fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>> {
        let mut conn = self.connection.clone();
        let result: Vec<String> = redis::cmd("LRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Hash set field
    pub async fn hset(&self, key: &str, field: &str, value: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("HSET")
            .arg(key)
            .arg(field)
            .arg(value)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Hash get field
    pub async fn hget(&self, key: &str, field: &str) -> Result<Option<String>> {
        let mut conn = self.connection.clone();
        let result: Option<String> = redis::cmd("HGET")
            .arg(key)
            .arg(field)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Hash get all fields
    pub async fn hgetall(&self, key: &str) -> Result<Vec<(String, String)>> {
        let mut conn = self.connection.clone();
        let result: Vec<(String, String)> = redis::cmd("HGETALL")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Flush database (clear all keys) - gunakan dengan hati-hati!
    pub async fn flush_db(&self) -> Result<()> {
        let mut conn = self.connection.clone();
        redis::cmd("FLUSHDB")
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Delete with prefix
     pub async fn del_prefix(&self, prefix: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        let mut cursor: u64 = 0;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(format!("{}*", prefix))
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;

            if !keys.is_empty() {
                let _: () = redis::cmd("DEL")
                    .arg(keys)
                    .query_async(&mut conn)
                    .await?;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }
        
        Ok(())
    }
}
