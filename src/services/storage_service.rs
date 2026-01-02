use crate::config::Config;
use crate::errors::AppResult;
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct StorageService {
    client: Client,
    bucket: String,
    cdn_url: String,
}

impl StorageService {
    pub fn new(config: &Config) -> Self {
        let credentials = Credentials::new(
            &config.cloudflare_api_token,
            &config.cloudflare_secret,
            None,
            None,
            "cloudflare-r2",
        );

        let s3_config = S3ConfigBuilder::new()
            .endpoint_url(&config.s3_endpoint)
            .region(Region::new("auto"))
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        Self {
            client,
            bucket: config.s3_bucket.clone(),
            cdn_url: config.cdn_url.clone(),
        }
    }

    /// Upload bytes to R2 and return the CDN URL
    pub async fn upload_bytes(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> AppResult<String> {
        let body = ByteStream::from(bytes);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| crate::errors::AppError::Internal(format!("R2 upload failed: {}", e)))?;

        Ok(self.get_public_url(key))
    }

    /// Upload an image with structured path: {folder}/{book_id}/{filename}
    pub async fn upload_image(
        &self,
        folder: &str,
        book_id: &str,
        filename: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> AppResult<String> {
        let key = format!("{}/{}/{}", folder, book_id, filename);
        self.upload_bytes(&key, bytes, content_type).await
    }

    /// Generate CDN URL for a key
    pub fn get_public_url(&self, key: &str) -> String {
        format!("{}/{}", self.cdn_url.trim_end_matches('/'), key)
    }

    /// Delete a file from R2
    pub async fn delete_file(&self, key: &str) -> AppResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| crate::errors::AppError::Internal(format!("R2 delete failed: {}", e)))?;

        Ok(())
    }
}
