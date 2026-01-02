use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database model for content uploads
#[derive(Debug, Clone, FromRow)]
pub struct ContentUpload {
    pub id: String,
    pub book_id: Option<String>,
    pub original_filename: String,
    pub format: String,
    pub html_content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Database model for uploaded images
#[derive(Debug, Clone, FromRow)]
pub struct UploadedImage {
    pub id: String,
    pub upload_id: String,
    pub original_path: String,
    pub cdn_url: String,
    pub content_type: String,
    pub size: i64,
    pub created_at: NaiveDateTime,
}

/// Response DTO for content upload
#[derive(Debug, Serialize)]
pub struct ContentUploadResponse {
    pub id: String,
    pub html_content: String,
    pub images: Vec<ImageInfoDto>,
    pub format: String,
    pub created_at: NaiveDateTime,
}

/// Image info DTO
#[derive(Debug, Serialize, Clone)]
pub struct ImageInfoDto {
    pub filename: String,
    pub url: String,
    pub content_type: String,
    pub size: u64,
}

/// Request for creating a chapter from uploaded content
#[derive(Debug, Deserialize)]
pub struct CreateChapterFromUploadDto {
    pub upload_id: String,
    pub title: String,
    pub book_id: String,
    pub description: String,
    pub chapter_num: i32,
}

impl From<ContentUpload> for ContentUploadResponse {
    fn from(upload: ContentUpload) -> Self {
        Self {
            id: upload.id,
            html_content: upload.html_content,
            images: vec![], // Images loaded separately
            format: upload.format,
            created_at: upload.created_at,
        }
    }
}
