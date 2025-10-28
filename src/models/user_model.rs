use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(sqlx::Type, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
}


#[derive(Clone, Debug, FromRow, Serialize)]
pub struct User {
    pub id: String,
    profile_pic: Option<String>,
    bio: Option<String>,
    pub username: String,
    pub role: Role,
    pub email: String,
    pub password:String,
}


#[derive(Clone, Debug, FromRow, Serialize)]
pub struct SafeUser {
    pub id: String,
    pub username: String,
    pub profile_pic: Option<String>,
    pub bio: Option<String>,
    pub role: Role,
    pub email: String,
}

impl From<User> for SafeUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            profile_pic: user.profile_pic,
            bio: user.bio,
            username: user.username,
            role: user.role,
            email: user.email,
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub role: Role,
    pub email: String,
}
impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
        }
    }
}
