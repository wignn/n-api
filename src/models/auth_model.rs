use crate::models::user_model::SafeUser;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateProfileDto {
    pub username: Option<String>,
    pub bio: Option<String>,
    pub profile_pic: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangePasswordDto {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Auth {
    pub user: SafeUser,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthData {
    pub user: SafeUser,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub status: String,
    pub data: Auth,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseWithoutTokens {
    pub status: String,
    pub data: AuthData,
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

impl AuthResponseWithoutTokens {
    pub fn success(user: SafeUser) -> Self {
        Self {
            status: "success".to_string(),
            data: AuthData { user },
        }
    }
}
