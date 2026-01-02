-- Drop indexes
DROP INDEX IF EXISTS idx_uploaded_image_upload_id;
DROP INDEX IF EXISTS idx_content_upload_book_id;

-- Drop tables
DROP TABLE IF EXISTS "UploadedImage";
DROP TABLE IF EXISTS "ContentUpload";
