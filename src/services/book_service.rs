use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::models::book_model::{Book, BookDto, CreateBookDto, UpdateBookDto};
use crate::models::paging::{PaginatedResponse, PaginationMeta, PaginationParams};

pub struct BookService {
    db: Database,
}

impl BookService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn get_books(
        &self,
        params: PaginationParams,
    ) -> AppResult<PaginatedResponse<BookDto>> {
        let _ = params.validate().map_err(|e| AppError::ValidationError);

        let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books")
            .fetch_one(&self.db.pool)
            .await?;

        let books = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, title, author, cover, status, language, release_date, created_at, updated_at
            FROM books
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(params.take())
        .bind(params.skip())
        .fetch_all(&self.db.pool)
        .await?;

        let book_dtos: Vec<BookDto> = books.into_iter().map(|book| book.into()).collect();

        let pagination = PaginationMeta::new(params.page, params.limit, total_count.0);

        Ok(PaginatedResponse::new(book_dtos, pagination))
    }

    pub async fn get_book_by_id(&self, id: &str) -> AppResult<BookDto> {
        let book = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, title, author, cover, status, language, release_date, created_at, updated_at
            FROM books
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(book.into())
    }

    pub async fn create_book(&self, create_dto: CreateBookDto) -> AppResult<BookDto> {
        let id = uuid::Uuid::new_v4().to_string();

        let book = sqlx::query_as::<_, Book>(
            r#"
            INSERT INTO books (id, title, author, cover, status, language, release_date, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING id, title, author, cover, status, language, release_date, created_at, updated_at
            "#
        )
        .bind(&id)
        .bind(&create_dto.title)
        .bind(&create_dto.author)
        .bind(&create_dto.cover)
        .bind(&create_dto.status)
        .bind(&create_dto.language)
        .bind(&create_dto.release_date)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(book.into())
    }

    pub async fn update_book(&self, id: &str, update_dto: UpdateBookDto) -> AppResult<BookDto> {
        // Fetch existing book first
        let mut book = sqlx::query_as::<_, Book>(
            "SELECT id, title, author, cover, status, language, release_date, created_at, updated_at FROM books WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.db.pool)
        .await?;

        // Update fields if provided
        if let Some(title) = update_dto.title {
            book.title = title;
        }
        if let Some(author) = update_dto.author {
            book.author = author;
        }
        if let Some(cover) = update_dto.cover {
            book.cover = cover;
        }
        if let Some(status) = update_dto.status {
            book.status = status;
        }
        if let Some(release_date) = update_dto.release_date {
            book.release_date = release_date;
        }

        // Save to database
        let updated_book = sqlx::query_as::<_, Book>(
            r#"
            UPDATE books
            SET title = $2, author = $3, cover = $4, status = $5, release_date = $6, updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, author, cover, status, language, release_date, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.cover)
        .bind(&book.status)
        .bind(&book.release_date)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(updated_book.into())
    }

    pub async fn delete_book(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM books WHERE id = $1")
            .bind(id)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }
}
