-- Create ContentUpload table for storing extracted content metadata
CREATE TABLE "ContentUpload" (
    id TEXT PRIMARY KEY,
    book_id TEXT,
    original_filename TEXT NOT NULL,
    format TEXT NOT NULL,
    html_content TEXT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL,
    FOREIGN KEY (book_id) REFERENCES "Book"(id) ON DELETE SET NULL ON UPDATE CASCADE
);

-- Create UploadedImage table for tracking images extracted from uploads
CREATE TABLE "UploadedImage" (
    id TEXT PRIMARY KEY,
    upload_id TEXT NOT NULL,
    original_path TEXT NOT NULL,
    cdn_url TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size BIGINT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (upload_id) REFERENCES "ContentUpload"(id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- Create indexes for better query performance
CREATE INDEX idx_content_upload_book_id ON "ContentUpload"(book_id);
CREATE INDEX idx_uploaded_image_upload_id ON "UploadedImage"(upload_id);
