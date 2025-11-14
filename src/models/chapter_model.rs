use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub content: String,
    pub chapter_num: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterDto {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub content: String,
    pub chapter_num: i32,
}

impl From<Chapter> for ChapterDto {
    fn from(chapter: Chapter) -> Self {
        Self {
            id: chapter.id,
            title: chapter.title,
            book_id: chapter.book_id,
            description: chapter.description,
            created_at: chapter.created_at,
            updated_at: chapter.updated_at,
            content: chapter.content,
            chapter_num: chapter.chapter_num,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateChapterDto {
    pub title: String,
    pub book_id: String,
    pub description: String,
    pub content: String,
    pub chapter_num: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChapterDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub chapter_num: Option<i32>,
}

