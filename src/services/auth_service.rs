use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::models::auth_model::{Auth, LoginDto, RegisterDto};
use crate::models::user_model::User;
use crate::utils;
use crate::utils::jwt::JwtService;
use chrono::Utc;

pub struct AuthService {
    db: Database,
    jwt_service: JwtService,
}

impl AuthService {
    pub fn new(db: Database, jwt_service: JwtService) -> Self {
        Self { db, jwt_service }
    }

    pub async fn register(&self, request: RegisterDto) -> AppResult<Auth> {
        if self.email_exists(&request.email).await? {
            return Err(AppError::BadRequest("Email already exists".to_string()));
        }

        let hashed_password = utils::password::PasswordService::hash_password(&request.password).map_err(|e| AppError::PasswordHash(e.to_string()))?;
        let user_id = cuid2::create_id();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO "User" (id, username, email, password, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, username, email, password, created_at, updated_at
            "#,
        )
        .bind(&user_id)
        .bind(&request.name)
        .bind(&request.email)
        .bind(&hashed_password)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db.pool)
        .await?;

        let access_token = self.jwt_service.generate_access_token(&user.id, &user.email, user.role.clone())?;
        let refresh_token = self.jwt_service.generate_refresh_token(&user.id, &user.email, user.role.clone())?;

        Ok(Auth::new(user, access_token, refresh_token))
    }

    pub async fn login(&self, request: LoginDto) -> AppResult<Auth> {
        let user = self.get_user_by_email(&request.email).await?;

        if !utils::password::PasswordService::verify_password(&request.password, &user.password)
            .map_err(|_| AppError::Unauthorized)? {
            return Err(AppError::Unauthorized);
        }

        let access_token = self.jwt_service.generate_access_token(&user.id, &user.email, user.role.clone())?;
        let refresh_token = self.jwt_service.generate_refresh_token(&user.id, &user.email, user.role.clone())?;

        Ok(Auth::new(user, access_token, refresh_token))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<Auth> {
        let claims = self.jwt_service.verify_refresh_token(refresh_token)?;

        let user = self.get_user_by_id(&claims.sub).await?;

        let new_access_token = self.jwt_service.generate_access_token(&user.id, &user.email, user.role.clone())?;
        let new_refresh_token = self.jwt_service.generate_refresh_token(&user.id, &user.email, user.role.clone())?;

        Ok(Auth::new(user, new_access_token, new_refresh_token))
    }



    async fn email_exists(&self, email: &str) -> AppResult<bool> {
        let result: Option<(bool,)> = sqlx::query_as(
            r#"SELECT EXISTS(SELECT 1 FROM "User" WHERE email = $1)"#,
        )
        .bind(email)
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(result.map(|(exists,)| exists).unwrap_or(false))
    }

    async fn get_user_by_email(&self, email: &str) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password, created_at, updated_at
            FROM "User"
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_one(&self.db.pool)
        .await
        .map_err(|_| AppError::Unauthorized)?;

        Ok(user)
    }

    async fn get_user_by_id(&self, id: &str) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password, created_at, updated_at
            FROM "User"
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.db.pool)
        .await
        .map_err(|_| AppError::NotFound("User not found".to_string()))?;

        Ok(user)
    }
}