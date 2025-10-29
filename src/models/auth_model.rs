use serde::{Deserialize, Serialize};
use crate::models::user_model::{SafeUser};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Auth {
    pub user: SafeUser,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub status: String,
    pub data: Auth,
}

impl Auth {
    pub fn new(user: SafeUser, access_token: String, refresh_token: String) -> Self {
        Self {
            user,
            access_token,
            refresh_token,
        }
    }
}

impl AuthResponse {
    pub fn success(auth: Auth) -> Self {
        Self {
            status: "success".to_string(),
            data: auth,
        }
    }
}