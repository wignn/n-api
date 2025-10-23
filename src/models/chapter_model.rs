use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub description: String,
    pub content: String,
    pub chapter_num: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub chapter_num: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Chapter> for ChapterDto {
    fn from(chapter: Chapter) -> Self {
        Self {
            id: chapter.id,
            title: chapter.title,
            description: chapter.description,
            content: chapter.content,
            chapter_num: chapter.chapter_num,
            created_at: chapter.created_at,
            updated_at: chapter.updated_at,
        }
    }
}