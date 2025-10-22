use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum Language {
    English,
    Korean,
    Japanese
}

#[derive(Clone, Debug, FromRow)]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub cover: String,
    pub status: String,
    pub language: Language,
    pub release_date: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookDto {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub cover: String,
    pub status: String,
    pub language: String,
    pub release_date: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>
}

impl From<Book> for BookDto {
    fn from(book: Book) -> Self {
        Self {
            id: book.id,
            title: book.title,
            author: book.author,
            cover: book.cover,
            status: book.status,
            language: format!("{:?}", book.language),
            release_date: book.release_date,
            created_at: book.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: book.updated_at.map(|dt| dt.to_rfc3339())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBookDto {
    pub title: String,
    pub author: String,
    pub cover: String,
    pub status: String,
    pub language: String,
    pub release_date: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookDto {
    pub title: Option<String>,
    pub author: Option<String>,
    pub cover: Option<String>,
    pub status: Option<String>,
    pub language: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}
