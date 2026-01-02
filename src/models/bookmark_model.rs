use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub user_id: String,
    pub book_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Bookmark with joined Book data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BookmarkWithBook {
    pub id: String,
    pub user_id: String,
    pub book_id: String,
    pub created_at: NaiveDateTime,
    // Book fields
    pub book_title: String,
    pub book_cover: String,
    pub book_author: String,
    pub book_description: String,
}

/// DTO for creating a bookmark
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBookmarkDto {
    pub book_id: String,
}

/// Response DTO for bookmark
#[derive(Debug, Clone, Serialize)]
pub struct BookmarkResponse {
    pub id: String,
    pub book_id: String,
    pub created_at: NaiveDateTime,
}

/// Response for bookmark with book details
#[derive(Debug, Clone, Serialize)]
pub struct BookmarkWithBookResponse {
    pub id: String,
    pub book_id: String,
    pub created_at: NaiveDateTime,
    pub book: BookSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct BookSummary {
    pub id: String,
    pub title: String,
    pub cover: String,
    pub author: String,
    pub description: String,
}

/// Response for check bookmark status
#[derive(Debug, Clone, Serialize)]
pub struct BookmarkStatusResponse {
    pub is_bookmarked: bool,
    pub bookmark_id: Option<String>,
}

impl From<Bookmark> for BookmarkResponse {
    fn from(bookmark: Bookmark) -> Self {
        Self {
            id: bookmark.id,
            book_id: bookmark.book_id,
            created_at: bookmark.created_at,
        }
    }
}

impl From<BookmarkWithBook> for BookmarkWithBookResponse {
    fn from(b: BookmarkWithBook) -> Self {
        Self {
            id: b.id,
            book_id: b.book_id.clone(),
            created_at: b.created_at,
            book: BookSummary {
                id: b.book_id,
                title: b.book_title,
                cover: b.book_cover,
                author: b.book_author,
                description: b.book_description,
            },
        }
    }
}
