use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::models::auth_model::{Auth, LoginDto, RegisterDto};
use crate::models::user_model::Role;
use crate::models::user_model::{SafeUser, User};
use crate::services::storage_service::StorageService;
use crate::utils;
use crate::utils::jwt::JwtService;
use chrono::Utc;

pub struct AuthService {
    db: Database,
    jwt_service: JwtService,
    storage: StorageService,
}

impl AuthService {
    pub fn new(db: Database, jwt_service: JwtService, storage: StorageService) -> Self {
        Self {
            db,
            jwt_service,
            storage,
        }
    }

    pub async fn register(&self, request: RegisterDto) -> AppResult<Auth> {
        if self.email_exists(&request.email).await? {
            return Err(AppError::BadRequest("Email already exists".to_string()));
        }

        if self.username_exists(&request.username).await? {
            return Err(AppError::BadRequest("Username already exists".to_string()));
        }

        let hashed_password = utils::password::PasswordService::hash_password(&request.password)
            .map_err(|e| AppError::PasswordHash(e.to_string()))?;
        let user_id = cuid2::create_id();

        let user = sqlx::query_as::<_, SafeUser>(
            r#"
    INSERT INTO "User" (id, username, email, password, created_at, updated_at, role)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING id, username, email, role, bio, profile_pic
    "#,
        )
        .bind(&user_id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(&hashed_password)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(Role::User)
        .fetch_one(&self.db.pool)
        .await?;

        let access_token =
            self.jwt_service
                .generate_access_token(&user.id, &user.email, user.role.clone())?;
        let refresh_token =
            self.jwt_service
                .generate_refresh_token(&user.id, &user.email, user.role.clone())?;

        Ok(Auth::new(user, access_token, refresh_token))
    }

    pub async fn login(&self, request: LoginDto) -> AppResult<Auth> {
        let user = self.get_user_by_email(&request.email).await?;

        if !utils::password::PasswordService::verify_password(&request.password, &user.password)
            .map_err(|_| AppError::Unauthorized)?
        {
            return Err(AppError::Unauthorized);
        }

        let access_token =
            self.jwt_service
                .generate_access_token(&user.id, &user.email, user.role.clone())?;
        let refresh_token =
            self.jwt_service
                .generate_refresh_token(&user.id, &user.email, user.role.clone())?;

        Ok(Auth::new(user.into(), access_token, refresh_token))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<Auth> {
        let claims = self.jwt_service.verify_refresh_token(refresh_token)?;

        let user = self.get_user_by_id(&claims.sub).await?;

        let new_access_token =
            self.jwt_service
                .generate_access_token(&user.id, &user.email, user.role.clone())?;
        let new_refresh_token =
            self.jwt_service
                .generate_refresh_token(&user.id, &user.email, user.role.clone())?;

        Ok(Auth::new(user.into(), new_access_token, new_refresh_token))
    }

    async fn email_exists(&self, email: &str) -> AppResult<bool> {
        let result: Option<(bool,)> =
            sqlx::query_as(r#"SELECT EXISTS(SELECT 1 FROM "User" WHERE email = $1)"#)
                .bind(email)
                .fetch_optional(&self.db.pool)
                .await?;

        Ok(result.map(|(exists,)| exists).unwrap_or(false))
    }

    async fn username_exists(&self, username: &str) -> AppResult<bool> {
        let redis = &self.db.redis;
        let cache_key = format!("user:{username}");

        if let Ok(Some(cached_book)) = redis.get_json::<bool>(&cache_key).await {
            return Ok(cached_book);
        }

        let result: Option<(bool,)> =
            sqlx::query_as(r#"SELECT EXISTS(SELECT 1 FROM "User" WHERE username = $1)"#)
                .bind(username)
                .fetch_optional(&self.db.pool)
                .await?;

        let _ = redis.set_json(&cache_key, &result, 600).await;
        Ok(result.map(|(exists,)| exists).unwrap_or(false))
    }

    async fn get_user_by_email(&self, email: &str) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password, role, bio, profile_pic
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

    pub async fn get_user_by_id(&self, id: &str) -> AppResult<SafeUser> {
        let redis = &self.db.redis;
        let cache_key = format!("user:{id}");

        if let Ok(Some(cached_book)) = redis.get_json::<SafeUser>(&cache_key).await {
            return Ok(cached_book);
        }

        let user = sqlx::query_as::<_, SafeUser>(
            r#"
            SELECT id, username, email, bio, profile_pic, role
            FROM "User"
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.db.pool)
        .await
        .map_err(|e| AppError::NotFound(e.to_string()))?;

        let _ = redis.set_json(&cache_key, &user, 600).await;
        Ok(user)
    }

    pub async fn update_profile(
        &self,
        user_id: &str,
        request: crate::models::auth_model::UpdateProfileDto,
    ) -> AppResult<SafeUser> {
        let redis = &self.db.redis;

        // Check username uniqueness if being changed
        if let Some(ref new_username) = request.username {
            let existing = sqlx::query_scalar::<_, String>(
                r#"SELECT id FROM "User" WHERE username = $1 AND id != $2"#,
            )
            .bind(new_username)
            .bind(user_id)
            .fetch_optional(&self.db.pool)
            .await?;

            if existing.is_some() {
                return Err(AppError::BadRequest("Username already taken".to_string()));
            }
        }

        let user = sqlx::query_as::<_, SafeUser>(
            r#"
            UPDATE "User" 
            SET username = COALESCE($2, username),
                bio = COALESCE($3, bio),
                profile_pic = COALESCE($4, profile_pic),
                updated_at = $5
            WHERE id = $1
            RETURNING id, username, email, bio, profile_pic, role
            "#,
        )
        .bind(user_id)
        .bind(&request.username)
        .bind(&request.bio)
        .bind(&request.profile_pic)
        .bind(Utc::now())
        .fetch_one(&self.db.pool)
        .await?;

        // Clear cache
        let _ = redis.del(&format!("user:{user_id}")).await;

        Ok(user)
    }

    pub async fn change_password(
        &self,
        user_id: &str,
        current_password: &str,
        new_password: &str,
    ) -> AppResult<()> {
        let redis = &self.db.redis;

        // Get user with password
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, username, email, password, role, bio, profile_pic FROM "User" WHERE id = $1"#
        )
        .bind(user_id)
        .fetch_one(&self.db.pool)
        .await
        .map_err(|_| AppError::NotFound("User not found".to_string()))?;

        // Verify current password
        if !utils::password::PasswordService::verify_password(current_password, &user.password)
            .map_err(|_| AppError::BadRequest("Invalid current password".to_string()))?
        {
            return Err(AppError::BadRequest("Invalid current password".to_string()));
        }

        // Hash new password
        let hashed_password = utils::password::PasswordService::hash_password(new_password)
            .map_err(|e| AppError::PasswordHash(e.to_string()))?;

        // Update password
        sqlx::query(r#"UPDATE "User" SET password = $2, updated_at = $3 WHERE id = $1"#)
            .bind(user_id)
            .bind(&hashed_password)
            .bind(Utc::now())
            .execute(&self.db.pool)
            .await?;

        // Clear cache
        let _ = redis.del(&format!("user:{user_id}")).await;

        Ok(())
    }

    pub async fn upload_avatar(
        &self,
        user_id: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> AppResult<String> {
        let redis = &self.db.redis;

        // Generate unique filename
        let extension = match content_type {
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "jpg",
        };
        let filename = format!("avatars/{}/{}.{}", user_id, cuid2::create_id(), extension);

        // Upload to R2 via storage service
        let url = self
            .storage
            .upload_bytes(&filename, bytes, content_type)
            .await?;

        // Update user profile_pic in database
        sqlx::query(r#"UPDATE "User" SET profile_pic = $2, updated_at = $3 WHERE id = $1"#)
            .bind(user_id)
            .bind(&url)
            .bind(Utc::now())
            .execute(&self.db.pool)
            .await?;

        // Clear cache
        let _ = redis.del(&format!("user:{user_id}")).await;

        Ok(url)
    }
}
