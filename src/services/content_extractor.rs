use regex::Regex;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

use crate::errors::{AppError, AppResult};
use crate::services::storage_service::StorageService;

/// Extracted content from EPUB/DOCX
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    /// HTML content with image URLs replaced to CDN URLs
    pub html_content: String,
    /// List of extracted and uploaded images
    pub images: Vec<ExtractedImage>,
}

#[derive(Debug, Clone)]
pub struct ExtractedImage {
    /// Original path in the archive (e.g., "images/cover.jpg")
    pub original_path: String,
    /// CDN URL after upload
    pub cdn_url: String,
    /// MIME type
    pub content_type: String,
    /// File size in bytes
    pub size: u64,
}

/// Supported content formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentFormat {
    Epub,
    Docx,
    Unknown,
}

pub struct ContentExtractor {
    storage: StorageService,
}

impl ContentExtractor {
    pub fn new(storage: StorageService) -> Self {
        Self { storage }
    }

    /// Detect format from file bytes using magic bytes
    pub fn detect_format(bytes: &[u8]) -> ContentFormat {
        // Both EPUB and DOCX are ZIP files, need to check internal structure
        if bytes.len() < 4 {
            return ContentFormat::Unknown;
        }

        // Check ZIP magic bytes (PK..)
        if bytes[0..4] != [0x50, 0x4B, 0x03, 0x04] {
            return ContentFormat::Unknown;
        }

        // Try to open as ZIP and check contents
        if let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes)) {
            // EPUB has mimetype file or META-INF/container.xml
            for i in 0..archive.len() {
                if let Ok(file) = archive.by_index(i) {
                    let name = file.name().to_lowercase();
                    if name == "mimetype" || name.contains("meta-inf/container.xml") {
                        return ContentFormat::Epub;
                    }
                    if name.contains("[content_types].xml") || name.contains("word/document.xml") {
                        return ContentFormat::Docx;
                    }
                }
            }
        }

        ContentFormat::Unknown
    }

    /// Extract content from EPUB file
    pub async fn extract_epub(&self, bytes: &[u8], book_id: &str) -> AppResult<ExtractedContent> {
        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(|e| AppError::BadRequest(format!("Invalid EPUB file: {}", e)))?;

        let mut html_parts: Vec<String> = Vec::new();
        let mut images: Vec<ExtractedImage> = Vec::new();
        let mut image_url_map: HashMap<String, String> = HashMap::new();

        // First pass: collect image data synchronously (ZipFile is not Send)
        let mut pending_images: Vec<(String, Vec<u8>, String, u64)> = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::Internal(format!("Failed to read archive: {}", e)))?;

            let name = file.name().to_string();

            if Self::is_image_file(&name) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| AppError::Internal(format!("Failed to read image: {}", e)))?;

                let content_type = mime_guess::from_path(&name)
                    .first_or_octet_stream()
                    .to_string();

                let size = buffer.len() as u64;
                pending_images.push((name, buffer, content_type, size));
            }
        }
        // ZipFile dropped here - archive borrow released

        // Now upload images asynchronously (no ZipFile held across await)
        for (name, buffer, content_type, size) in pending_images {
            let filename = Self::sanitize_filename(&name);

            let cdn_url = self
                .storage
                .upload_image("content-images", book_id, &filename, buffer, &content_type)
                .await?;

            // Map original path to CDN URL (handle both relative and absolute paths)
            let original_basename = name.rsplit('/').next().unwrap_or(&name);
            image_url_map.insert(name.clone(), cdn_url.clone());
            image_url_map.insert(original_basename.to_string(), cdn_url.clone());

            images.push(ExtractedImage {
                original_path: name,
                cdn_url,
                content_type,
                size,
            });
        }

        // Second pass: extract HTML/XHTML content
        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(|e| AppError::Internal(format!("Failed to reopen archive: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::Internal(format!("Failed to read archive: {}", e)))?;

            let name = file.name().to_string();

            if Self::is_content_file(&name) {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| AppError::Internal(format!("Failed to read content: {}", e)))?;

                // Replace image references with CDN URLs
                let updated_content = Self::replace_image_urls(&content, &image_url_map);
                html_parts.push(updated_content);
            }
        }

        let html_content = html_parts.join("\n\n<!-- Chapter Break -->\n\n");

        Ok(ExtractedContent {
            html_content,
            images,
        })
    }

    /// Extract content from DOCX file
    pub async fn extract_docx(&self, bytes: &[u8], book_id: &str) -> AppResult<ExtractedContent> {
        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(|e| AppError::BadRequest(format!("Invalid DOCX file: {}", e)))?;

        let mut images: Vec<ExtractedImage> = Vec::new();
        let mut image_url_map: HashMap<String, String> = HashMap::new();

        // First pass: collect image data synchronously (ZipFile is not Send)
        let mut pending_images: Vec<(String, Vec<u8>, String, u64)> = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::Internal(format!("Failed to read archive: {}", e)))?;

            let name = file.name().to_string();

            if name.starts_with("word/media/") && Self::is_image_file(&name) {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| AppError::Internal(format!("Failed to read image: {}", e)))?;

                let content_type = mime_guess::from_path(&name)
                    .first_or_octet_stream()
                    .to_string();

                let size = buffer.len() as u64;
                pending_images.push((name, buffer, content_type, size));
            }
        }
        // ZipFile dropped here - archive borrow released

        // Now upload images asynchronously (no ZipFile held across await)
        for (name, buffer, content_type, size) in pending_images {
            let filename = Self::sanitize_filename(&name);

            let cdn_url = self
                .storage
                .upload_image("content-images", book_id, &filename, buffer, &content_type)
                .await?;

            let original_basename = name.rsplit('/').next().unwrap_or(&name);
            image_url_map.insert(name.clone(), cdn_url.clone());
            image_url_map.insert(original_basename.to_string(), cdn_url.clone());

            images.push(ExtractedImage {
                original_path: name,
                cdn_url,
                content_type,
                size,
            });
        }

        // Extract document.xml and convert to HTML
        let mut archive: ZipArchive<Cursor<&[u8]>> = ZipArchive::new(Cursor::new(bytes))
            .map_err(|e| AppError::Internal(format!("Failed to reopen archive: {}", e)))?;

        let mut document_xml = String::new();
        if let Ok(mut file) = archive.by_name("word/document.xml") {
            let _ = file.read_to_string(&mut document_xml);
        }

        // Convert DOCX XML to simple HTML
        let html_content = Self::docx_xml_to_html(&document_xml, &image_url_map);

        Ok(ExtractedContent {
            html_content,
            images,
        })
    }

    /// Auto-detect format and extract content
    pub async fn extract(&self, bytes: &[u8], book_id: &str) -> AppResult<ExtractedContent> {
        match Self::detect_format(bytes) {
            ContentFormat::Epub => self.extract_epub(bytes, book_id).await,
            ContentFormat::Docx => self.extract_docx(bytes, book_id).await,
            ContentFormat::Unknown => Err(AppError::BadRequest(
                "Unsupported file format. Only EPUB and DOCX are supported.".to_string(),
            )),
        }
    }

    fn is_image_file(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".png")
            || lower.ends_with(".gif")
            || lower.ends_with(".webp")
            || lower.ends_with(".svg")
    }

    fn is_content_file(name: &str) -> bool {
        let lower = name.to_lowercase();
        (lower.ends_with(".html") || lower.ends_with(".xhtml") || lower.ends_with(".htm"))
            && !lower.contains("toc")
            && !lower.contains("nav")
    }

    fn sanitize_filename(path: &str) -> String {
        let basename = path.rsplit('/').next().unwrap_or(path);
        // Add timestamp to prevent collisions
        let timestamp = chrono::Utc::now().timestamp_millis();
        format!("{}_{}", timestamp, basename.replace(' ', "_"))
    }

    fn replace_image_urls(content: &str, url_map: &HashMap<String, String>) -> String {
        let mut result = content.to_string();

        // Replace src="..." attributes
        let src_re = Regex::new(r#"src=["']([^"']+)["']"#).unwrap();
        for cap in src_re.captures_iter(content) {
            let original = &cap[1];
            let basename = original.rsplit('/').next().unwrap_or(original);

            if let Some(cdn_url) = url_map.get(original).or_else(|| url_map.get(basename)) {
                result = result.replace(&cap[0], &format!(r#"src="{}""#, cdn_url));
            }
        }

        // Replace xlink:href for SVG
        let xlink_re = Regex::new(r#"xlink:href=["']([^"']+)["']"#).unwrap();
        for cap in xlink_re.captures_iter(content) {
            let original = &cap[1];
            let basename = original.rsplit('/').next().unwrap_or(original);

            if let Some(cdn_url) = url_map.get(original).or_else(|| url_map.get(basename)) {
                result = result.replace(&cap[0], &format!(r#"xlink:href="{}""#, cdn_url));
            }
        }

        result
    }

    fn docx_xml_to_html(xml: &str, image_url_map: &HashMap<String, String>) -> String {
        let mut html = String::new();

        // Simple DOCX to HTML conversion
        // Extract paragraphs <w:p> and text runs <w:t>
        let para_re = Regex::new(r"<w:p[^>]*>(.*?)</w:p>").unwrap();
        let text_re = Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
        let bold_re = Regex::new(r"<w:b/>").unwrap();
        let italic_re = Regex::new(r"<w:i/>").unwrap();
        let image_re = Regex::new(r#"<a:blip[^>]*r:embed="([^"]+)"[^>]*/>"#).unwrap();

        for para_cap in para_re.captures_iter(xml) {
            let para_content = &para_cap[1];
            let mut para_html = String::new();

            // Check for formatting
            let is_bold = bold_re.is_match(para_content);
            let is_italic = italic_re.is_match(para_content);

            // Extract text
            for text_cap in text_re.captures_iter(para_content) {
                para_html.push_str(&text_cap[1]);
            }

            // Check for images
            for img_cap in image_re.captures_iter(para_content) {
                let rel_id = &img_cap[1];
                // Try to find corresponding image
                for (path, url) in image_url_map {
                    if path.contains(rel_id)
                        || rel_id.contains(path.rsplit('/').next().unwrap_or(""))
                    {
                        para_html.push_str(&format!(r#"<img src="{}" alt="image" />"#, url));
                        break;
                    }
                }
            }

            if !para_html.is_empty() {
                if is_bold && is_italic {
                    html.push_str(&format!("<p><strong><em>{}</em></strong></p>\n", para_html));
                } else if is_bold {
                    html.push_str(&format!("<p><strong>{}</strong></p>\n", para_html));
                } else if is_italic {
                    html.push_str(&format!("<p><em>{}</em></p>\n", para_html));
                } else {
                    html.push_str(&format!("<p>{}</p>\n", para_html));
                }
            } else {
                html.push_str("<p></p>\n");
            }
        }

        if html.is_empty() {
            // Fallback: just strip all XML tags
            let strip_re = Regex::new(r"<[^>]+>").unwrap();
            html = strip_re.replace_all(xml, "").to_string();
        }

        html
    }
}
