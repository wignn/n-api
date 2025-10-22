
use crate::models::user_model::Role;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone ,Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: Role,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Clone,Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_expires_in: i64,
    refresh_expires_in: i64,
}

impl JwtService {
    pub fn new(secret: &str, access_expires_in: i64, refresh_expires_in: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_expires_in,
            refresh_expires_in,
        }
    }

    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        email: &str,
        role: Role,
    ) -> AppResult<String> {
        self.generate_token(user_id, email, role, TokenType::Access, self.access_expires_in)
    }

    pub fn generate_refresh_token(
        &self,
        user_id: Uuid,
        email: &str,
        role: Role,
    ) -> AppResult<String> {
        self.generate_token(user_id, email, role, TokenType::Refresh, self.refresh_expires_in)
    }

    fn generate_token(
        &self,
        user_id: Uuid,
        email: &str,
        role: Role,
        token_type: TokenType,
        expires_in: i64,
    ) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(expires_in);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(AppError::Jwt)
    }

    pub fn verify_token(&self, token: &str) -> AppResult<Claims> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(AppError::Jwt)
    }

    pub fn verify_access_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.verify_token(token)?;
        match claims.token_type {
            TokenType::Access => Ok(claims),
            TokenType::Refresh => Err(AppError::Unauthorized),
        }
    }

    pub fn verify_refresh_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.verify_token(token)?;
        match claims.token_type {
            TokenType::Refresh => Ok(claims),
            TokenType::Access => Err(AppError::Unauthorized),
        }
    }
}