use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
}

#[derive(Clone, Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
    pub email: String,
    pub password: String,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
    pub email: String,
    pub token: String,
}
impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            token: user.token,
            email: user.email,
            role: user.role,
        }
    }
}
