use crate::database::Database;
use crate::errors::AppResult;
use crate::models::book_model::{Book, CreateBookDto, UpdateBookDto};
use crate::models::paging::{PaginatedResponse, PaginationParams};
use cuid2;
use chrono::Utc;
use sqlx::QueryBuilder;

pub struct BookService {
    db: Database,
}

impl BookService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_book(&self, request: CreateBookDto) -> AppResult<Book> {
        let id = cuid2::create_id();

        let book = sqlx::query_as::<_, Book>(
            r#"
            INSERT INTO "Book" (
                id, title, author, cover, description, asset,
                status, language, release_date, popular,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, title, author, cover, description, asset,
                      status, language, release_date, popular,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.title)
        .bind(&request.author)
        .bind(&request.cover)
        .bind(&request.description)
        .bind(&request.asset)
        .bind(request.status)
        .bind(&request.language)
        .bind(&request.release_date)
        .bind(request.popular)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db.pool)
        .await?;

        Ok(book)
    }

    pub async fn get_books(&self, params: PaginationParams) -> AppResult<PaginatedResponse<Book>> {
        let offset = (params.page - 1) * params.page_size;

        let total_items: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM "Book""#)
            .fetch_one(&self.db.pool)
            .await?;

        let books: Vec<Book> = sqlx::query_as::<_, Book>(
            r#"
                SELECT id, title, author, cover, description, asset,
                       status, language, release_date, popular,
                       created_at, updated_at
                FROM "Book"
                ORDER BY "created_at" DESC
                LIMIT $1 OFFSET $2
            "#,
        )
        .bind(params.page_size)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        let total_pages = (total_items as f64 / params.page_size as f64).ceil() as i64;

        Ok(PaginatedResponse {
            data: books,
            page: params.page,
            page_size: params.page_size,
            total_items,
            total_pages,
        })
    }

    pub async fn get_book(&self, id: String) -> AppResult<Book> {
        let book = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, title, author, cover, description, asset, status, language, release_date, popular,
                   created_at, updated_at
            FROM "Book" WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.db.pool)
        .await?;
        Ok(book)
    }

    pub async fn update_book(&self, id: String, request: UpdateBookDto) -> AppResult<Book> {
        let mut builder = QueryBuilder::new(r#"UPDATE "Book" SET "#);
        let mut separated = builder.separated(", ");
        let mut has_updates = false;

        if let Some(ref title) = request.title {
            separated.push("title = ").push_bind_unseparated(title);
            has_updates = true;
        }
        if let Some(ref author) = request.author {
            separated.push("author = ").push_bind_unseparated(author);
            has_updates = true;
        }
        if let Some(ref cover) = request.cover {
            separated.push("cover = ").push_bind_unseparated(cover);
            has_updates = true;
        }
        if let Some(ref description) = request.description {
            separated.push("description = ").push_bind_unseparated(description);
            has_updates = true;
        }
        if let Some(ref asset) = request.asset {
            separated.push("asset = ").push_bind_unseparated(asset);
            has_updates = true;
        }
        if let Some(ref status) = request.status {
            separated.push("status = ").push_bind_unseparated(status);
            has_updates = true;
        }
        if let Some(ref language) = request.language {
            separated.push("language = ").push_bind_unseparated(language);
            has_updates = true;
        }
        if let Some(ref release_date) = request.release_date {
            separated.push("release_date = ").push_bind_unseparated(release_date);
            has_updates = true;
        }
        if let Some(ref popular) = request.popular {
            separated.push("popular = ").push_bind_unseparated(popular);
            has_updates = true;
        }

        if !has_updates {
            return self.get_book(id).await;
        }

        separated.push("updated_at = ").push_bind_unseparated(Utc::now());
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" RETURNING *");

        let updated_book = builder
            .build_query_as::<Book>()
            .fetch_one(&self.db.pool)
            .await?;

        Ok(updated_book)
    }

    pub async fn delete_book(&self, id: String) -> AppResult<Book> {
        let book = self.get_book(id.clone()).await?;

        sqlx::query(r#"DELETE FROM "Book" WHERE id = $1"#)
            .bind(id)
            .execute(&self.db.pool)
            .await?;

        Ok(book)
    }
}
