use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "Language", rename_all = "PascalCase")]
pub enum Language {
    English,
    Japanese,
    Korean,
}

impl Default for Language {
    fn default() -> Self {
        Language::Korean
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "Status", rename_all = "PascalCase")]
pub enum Status {
    Ongoing,
    Completed,
    Drop,
}

impl Default for Status {
    fn default() -> Self {
        Status::Ongoing
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub cover: String,
    pub description: String,
    pub asset: Option<String>,
    pub status: Status,
    pub language: Language,
    pub release_date: Option<i32>,
    pub popular: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookDto {
    pub id: String,
    pub title: String,
    pub author: String,
    pub cover: String,
    pub description: String,
    pub asset: Option<String>,
    pub status: Status,
    pub language: Language,
    pub release_date: Option<i32>,
    pub popular: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Book> for BookDto {
    fn from(book: Book) -> Self {
        Self {
            id: book.id,
            title: book.title,
            author: book.author,
            cover: book.cover,
            description: book.description,
            asset: book.asset,
            status: book.status,
            language: book.language,
            release_date: book.release_date,
            popular: book.popular,
            created_at: book.created_at,
            updated_at: book.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBookDto {
    pub title: String,
    pub author: String,
    pub cover: String,
    pub description: String,
    pub asset: Option<String>,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub language: Language,
    pub release_date: Option<i32>,
    #[serde(default)]
    pub popular: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookDto {
    pub title: Option<String>,
    pub author: Option<String>,
    pub cover: Option<String>,
    pub description: Option<String>,
    pub asset: Option<String>,
    pub status: Option<Status>,
    pub language: Option<Language>,
    pub release_date: Option<i32>,
    pub popular: Option<bool>,
}





#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bookmark {
    pub id: String,
    pub user_id: String,
    pub book_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookmarkDto {
    pub id: String,
    pub user_id: String,
    pub book_id: String,
    pub created_at: NaiveDateTime,
}

impl From<Bookmark> for BookmarkDto {
    fn from(bookmark: Bookmark) -> Self {
        Self {
            id: bookmark.id,
            user_id: bookmark.user_id,
            book_id: bookmark.book_id,
            created_at: bookmark.created_at,
        }
    }
}