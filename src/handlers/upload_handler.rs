use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use cuid2;

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::upload_model::{ContentUpload, ContentUploadResponse, ImageInfoDto, UploadedImage},
    services::content_extractor::{ContentExtractor, ContentFormat},
    AppState,
};

pub struct UploadHandler;

impl UploadHandler {
    // Note: Auth is handled by route middleware layer
    pub async fn upload_content(
        State(state): State<AppState>,
        multipart: Multipart,
    ) -> Result<(StatusCode, Json<ContentUploadResponse>), AppError> {
        let mut multipart = multipart;
        let mut file_bytes: Option<Vec<u8>> = None;
        let mut original_filename: Option<String> = None;
        let mut book_id: Option<String> = None;

        // Parse multipart form
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to parse multipart: {}", e)))?
        {
            let name = field.name().unwrap_or("").to_string();

            match name.as_str() {
                "file" => {
                    original_filename = field.file_name().map(|s| s.to_string());
                    file_bytes = Some(
                        field
                            .bytes()
                            .await
                            .map_err(|e| {
                                AppError::BadRequest(format!("Failed to read file: {}", e))
                            })?
                            .to_vec(),
                    );
                }
                "book_id" => {
                    book_id = Some(field.text().await.map_err(|e| {
                        AppError::BadRequest(format!("Failed to read book_id: {}", e))
                    })?);
                }
                _ => {}
            }
        }

        let bytes =
            file_bytes.ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;
        let filename = original_filename.unwrap_or_else(|| "unknown".to_string());

        // Detect format
        let format = ContentExtractor::detect_format(&bytes);
        if format == ContentFormat::Unknown {
            return Err(AppError::BadRequest(
                "Unsupported file format. Only EPUB and DOCX are supported.".to_string(),
            ));
        }

        let format_str = match format {
            ContentFormat::Epub => "epub",
            ContentFormat::Docx => "docx",
            ContentFormat::Unknown => "unknown",
        };

        // Generate upload ID for organizing images
        let upload_id = cuid2::create_id();
        let storage_id = book_id.clone().unwrap_or_else(|| upload_id.clone());

        // Extract content
        let extractor = ContentExtractor::new(state.storage.clone());
        let extracted = extractor.extract(&bytes, &storage_id).await?;

        // Save upload metadata to database
        let now = Utc::now();
        let upload = sqlx::query_as::<_, ContentUpload>(
            r#"
            INSERT INTO "ContentUpload" (
                id, book_id, original_filename, format, html_content, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, book_id, original_filename, format, html_content, created_at, updated_at
            "#,
        )
        .bind(&upload_id)
        .bind(&book_id)
        .bind(&filename)
        .bind(format_str)
        .bind(&extracted.html_content)
        .bind(now)
        .bind(now)
        .fetch_one(&state.db.pool)
        .await?;

        // Save image metadata
        let mut image_dtos: Vec<ImageInfoDto> = Vec::new();
        for img in &extracted.images {
            let image_id = cuid2::create_id();
            sqlx::query(
                r#"
                INSERT INTO "UploadedImage" (
                    id, upload_id, original_path, cdn_url, content_type, size, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(&image_id)
            .bind(&upload_id)
            .bind(&img.original_path)
            .bind(&img.cdn_url)
            .bind(&img.content_type)
            .bind(img.size as i64)
            .bind(now)
            .execute(&state.db.pool)
            .await?;

            image_dtos.push(ImageInfoDto {
                filename: img
                    .original_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&img.original_path)
                    .to_string(),
                url: img.cdn_url.clone(),
                content_type: img.content_type.clone(),
                size: img.size,
            });
        }

        tracing::info!(
            upload_id = %upload_id,
            format = %format_str,
            images_count = extracted.images.len(),
            "Content uploaded successfully"
        );

        Ok((
            StatusCode::CREATED,
            Json(ContentUploadResponse {
                id: upload.id,
                html_content: upload.html_content,
                images: image_dtos,
                format: upload.format,
                created_at: upload.created_at,
            }),
        ))
    }

    /// Get upload by ID
    /// GET /api/upload/{id}
    pub async fn get_upload(
        State(state): State<AppState>,
        axum::extract::Path(id): axum::extract::Path<String>,
    ) -> Result<Json<ContentUploadResponse>, AppError> {
        let upload = sqlx::query_as::<_, ContentUpload>(
            r#"
            SELECT id, book_id, original_filename, format, html_content, created_at, updated_at
            FROM "ContentUpload"
            WHERE id = $1
            "#,
        )
        .bind(&id)
        .fetch_optional(&state.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Upload not found".to_string()))?;

        let images = sqlx::query_as::<_, UploadedImage>(
            r#"
            SELECT id, upload_id, original_path, cdn_url, content_type, size, created_at
            FROM "UploadedImage"
            WHERE upload_id = $1
            "#,
        )
        .bind(&id)
        .fetch_all(&state.db.pool)
        .await?;

        let image_dtos: Vec<ImageInfoDto> = images
            .into_iter()
            .map(|img| ImageInfoDto {
                filename: img
                    .original_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(&img.original_path)
                    .to_string(),
                url: img.cdn_url,
                content_type: img.content_type,
                size: img.size as u64,
            })
            .collect();

        Ok(Json(ContentUploadResponse {
            id: upload.id,
            html_content: upload.html_content,
            images: image_dtos,
            format: upload.format,
            created_at: upload.created_at,
        }))
    }

    /// Delete upload and its images
    /// DELETE /api/upload/{id}
    pub async fn delete_upload(
        State(state): State<AppState>,
        Extension(_user): Extension<AuthUser>,
        axum::extract::Path(id): axum::extract::Path<String>,
    ) -> Result<StatusCode, AppError> {
        // Get images to delete from R2
        let images = sqlx::query_as::<_, UploadedImage>(
            r#"
            SELECT id, upload_id, original_path, cdn_url, content_type, size, created_at
            FROM "UploadedImage"
            WHERE upload_id = $1
            "#,
        )
        .bind(&id)
        .fetch_all(&state.db.pool)
        .await?;

        // Delete images from R2
        for img in &images {
            // Extract key from CDN URL
            if let Some(key) = img.cdn_url.strip_prefix(&state.config.cdn_url) {
                let key = key.trim_start_matches('/');
                if let Err(e) = state.storage.delete_file(key).await {
                    tracing::warn!(error = %e, key = %key, "Failed to delete image from R2");
                }
            }
        }

        // Delete from database
        sqlx::query(r#"DELETE FROM "UploadedImage" WHERE upload_id = $1"#)
            .bind(&id)
            .execute(&state.db.pool)
            .await?;

        sqlx::query(r#"DELETE FROM "ContentUpload" WHERE id = $1"#)
            .bind(&id)
            .execute(&state.db.pool)
            .await?;

        Ok(StatusCode::NO_CONTENT)
    }
}
