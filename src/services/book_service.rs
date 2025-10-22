use crate::database::Database;
use crate::errors::{AppError, AppResult};
use crate::models::book_model::Book;
use crate::models::paging::{PaginatedResponse, PaginationParams};
use crate::utils::jwt::JwtService;
use sqlx::postgres::PgRow;
use sqlx::Row;

pub struct BookService {
    db: Database,
    jwt_service: JwtService,
}

impl BookService {
    pub fn new(db: Database, jwt_service: JwtService) -> Self {
        Self { db, jwt_service }
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

    fn row_to_book(&self, row: PgRow) -> Result<Book, AppError> {
        Ok(Book {
            id: row.get("id"),
            title: row.get("title"),
            author: row.get("author"),
            cover: row.get("cover"),
            asset: row.get("asset"),
            description: row.get("description"),
            language: row.get("language"),
            status: row.get("status"),
            release_date: row.get("release_date"),
            popular: row.get("popular"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}
