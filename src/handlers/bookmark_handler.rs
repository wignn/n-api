use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::bookmark_model::{
        Bookmark, BookmarkResponse, BookmarkStatusResponse, BookmarkWithBook,
        BookmarkWithBookResponse, CreateBookmarkDto,
    },
    AppState,
};

pub struct BookmarkHandler;

impl BookmarkHandler {
    pub async fn create_bookmark(
        State(state): State<AppState>,
        Extension(user): Extension<AuthUser>,
        Json(dto): Json<CreateBookmarkDto>,
    ) -> Result<(StatusCode, Json<BookmarkResponse>), AppError> {
        tracing::debug!(user_id = %user.id, book_id = %dto.book_id, "Creating bookmark");

        let existing = sqlx::query_as::<_, Bookmark>(
            r#"SELECT id, user_id, book_id, created_at, updated_at 
               FROM "Bookmark" WHERE user_id = $1 AND book_id = $2"#,
        )
        .bind(&user.id)
        .bind(&dto.book_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error checking existing bookmark: {}", e);
            AppError::Internal(format!("Database error: {}", e))
        })?;

        if let Some(bookmark) = existing {
            tracing::debug!("Bookmark already exists");
            return Ok((StatusCode::OK, Json(BookmarkResponse::from(bookmark))));
        }

        // Check if book exists
        let book_exists =
            sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "Book" WHERE id = $1"#)
                .bind(&dto.book_id)
                .fetch_one(&state.db.pool)
                .await
                .map_err(|e| {
                    tracing::error!("Error checking book exists: {}", e);
                    AppError::Internal(format!("Database error: {}", e))
                })?;

        if book_exists == 0 {
            return Err(AppError::NotFound("Book not found".to_string()));
        }

        let id = cuid2::create_id();
        let now = Utc::now().naive_utc();

        let bookmark = sqlx::query_as::<_, Bookmark>(
            r#"
            INSERT INTO "Bookmark" (id, user_id, book_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, book_id, created_at, updated_at
            "#,
        )
        .bind(&id)
        .bind(&user.id)
        .bind(&dto.book_id)
        .bind(now)
        .bind(now)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error inserting bookmark: {}", e);
            AppError::Internal(format!("Database error: {}", e))
        })?;

        tracing::info!(
            user_id = %user.id,
            book_id = %dto.book_id,
            "Bookmark created"
        );

        Ok((StatusCode::CREATED, Json(BookmarkResponse::from(bookmark))))
    }

    /// Delete a bookmark
    /// DELETE /api/bookmark/{id}
    pub async fn delete_bookmark(
        State(state): State<AppState>,
        Extension(user): Extension<AuthUser>,
        Path(id): Path<String>,
    ) -> Result<StatusCode, AppError> {
        let result = sqlx::query(r#"DELETE FROM "Bookmark" WHERE id = $1 AND user_id = $2"#)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Bookmark not found".to_string()));
        }

        tracing::info!(bookmark_id = %id, user_id = %user.id, "Bookmark deleted");

        Ok(StatusCode::NO_CONTENT)
    }

    /// Delete bookmark by book_id (for toggle functionality)
    /// DELETE /api/bookmark/book/{book_id}
    pub async fn delete_bookmark_by_book(
        State(state): State<AppState>,
        Extension(user): Extension<AuthUser>,
        Path(book_id): Path<String>,
    ) -> Result<StatusCode, AppError> {
        let result = sqlx::query(r#"DELETE FROM "Bookmark" WHERE book_id = $1 AND user_id = $2"#)
            .bind(&book_id)
            .bind(&user.id)
            .execute(&state.db.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Bookmark not found".to_string()));
        }

        tracing::info!(book_id = %book_id, user_id = %user.id, "Bookmark deleted");

        Ok(StatusCode::NO_CONTENT)
    }

    /// Get all bookmarks for current user
    /// GET /api/bookmarks
    pub async fn get_user_bookmarks(
        State(state): State<AppState>,
        Extension(user): Extension<AuthUser>,
    ) -> Result<Json<Vec<BookmarkWithBookResponse>>, AppError> {
        let bookmarks = sqlx::query_as::<_, BookmarkWithBook>(
            r#"
            SELECT 
                b.id,
                b.user_id,
                b.book_id,
                b.created_at,
                bk.title as book_title,
                bk.cover as book_cover,
                bk.author as book_author,
                bk.description as book_description
            FROM "Bookmark" b
            JOIN "Book" bk ON b.book_id = bk.id
            WHERE b.user_id = $1
            ORDER BY b.created_at DESC
            "#,
        )
        .bind(&user.id)
        .fetch_all(&state.db.pool)
        .await?;

        let response: Vec<BookmarkWithBookResponse> = bookmarks
            .into_iter()
            .map(BookmarkWithBookResponse::from)
            .collect();

        Ok(Json(response))
    }

    /// Check if a book is bookmarked by current user
    /// GET /api/bookmark/check/{book_id}
    pub async fn check_bookmark(
        State(state): State<AppState>,
        Extension(user): Extension<AuthUser>,
        Path(book_id): Path<String>,
    ) -> Result<Json<BookmarkStatusResponse>, AppError> {
        let bookmark = sqlx::query_as::<_, Bookmark>(
            r#"SELECT id, user_id, book_id, created_at, updated_at 
               FROM "Bookmark" WHERE user_id = $1 AND book_id = $2"#,
        )
        .bind(&user.id)
        .bind(&book_id)
        .fetch_optional(&state.db.pool)
        .await?;

        Ok(Json(BookmarkStatusResponse {
            is_bookmarked: bookmark.is_some(),
            bookmark_id: bookmark.map(|b| b.id),
        }))
    }
}
