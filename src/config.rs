use serde::{Deserialize, Serialize};
use std::env;
use crate::errors::ConfigError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret_key: String,
    pub jwt_refresh_token: i64,
    pub jwt_expire_in: i64,
    pub api_key: String,
    pub email: String,
    pub password: String,
    pub cloudflare_api_token: String,
    pub cloudflare_secret: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub cdn_url: String,
    pub port: String
}



impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: Self::get_env("DATABASE_URL")?,
            jwt_secret_key: Self::get_env("JWT_SECRET_KEY")?,
            jwt_refresh_token: Self::get_env_i64("JWT_REFRESH_TOKEN")?,
            jwt_expire_in: Self::get_env_i64("JWT_ACCESS_EXPIRES_IN")?,
            api_key: Self::get_env("API_KEY")?,
            email: Self::get_env("EMAIL")?,
            password: Self::get_env("PASSWORD")?,
            cloudflare_api_token: Self::get_env("CLOUDFLARE_API_TOKEN")?,
            cloudflare_secret: Self::get_env("CLOUDFLARE_SECRET")?,
            s3_endpoint: Self::get_env("S3_ENDPOINT")?,
            s3_bucket: Self::get_env("S3_BUCKET")?,
            cdn_url: Self::get_env("CDN_URL")?,
            port: Self::get_env("PORT")?
        })

    }

    fn get_env(key: &str) -> Result<String, ConfigError> {
        env::var(key).map_err(|_| ConfigError::MissingVar(key.to_string()))
    }

    fn get_env_i64(key: &str) -> Result<i64, ConfigError> {
        let val = env::var(key).map_err(|_| ConfigError::MissingVar(key.to_string()))?;
        val.parse::<i64>()
            .map_err(|e| ConfigError::ParseError(key.to_string(), e))
    }
}
