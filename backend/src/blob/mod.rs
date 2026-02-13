use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use uuid::Uuid;

use crate::{AppError, Result};

/// Blob storage service for encrypted vault data
pub struct BlobStorage {
    client: Client,
    bucket: String,
}

impl BlobStorage {
    /// Create a new blob storage instance
    pub async fn new() -> Result<Self> {
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

        let bucket =
            std::env::var("S3_BUCKET").unwrap_or_else(|_| "keydrop-vault-blobs".to_string());

        // Check for local S3 endpoint (for development with MinIO/LocalStack)
        let client = if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
            let s3_config = aws_sdk_s3::config::Builder::from(&config)
                .endpoint_url(endpoint)
                .force_path_style(true)
                .build();
            Client::from_conf(s3_config)
        } else {
            Client::new(&config)
        };

        Ok(Self { client, bucket })
    }

    /// Generate a unique blob ID
    pub fn generate_blob_id(user_id: Uuid) -> String {
        format!("{}/{}", user_id, Uuid::new_v4())
    }

    /// Store an encrypted blob
    pub async fn store(&self, blob_id: &str, data: &[u8]) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(blob_id)
            .body(data.to_vec().into())
            .content_type("application/octet-stream")
            .send()
            .await
            .map_err(|e| AppError::BlobStorage(format!("Failed to store blob: {}", e)))?;

        Ok(())
    }

    /// Retrieve an encrypted blob
    pub async fn retrieve(&self, blob_id: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(blob_id)
            .send()
            .await
            .map_err(|e| AppError::BlobStorage(format!("Failed to retrieve blob: {}", e)))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| AppError::BlobStorage(format!("Failed to read blob body: {}", e)))?
            .into_bytes()
            .to_vec();

        Ok(data)
    }

    /// Delete an encrypted blob
    pub async fn delete(&self, blob_id: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(blob_id)
            .send()
            .await
            .map_err(|e| AppError::BlobStorage(format!("Failed to delete blob: {}", e)))?;

        Ok(())
    }

    /// Check if a blob exists
    pub async fn exists(&self, blob_id: &str) -> Result<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(blob_id)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                // Check if it's a "not found" error
                if e.to_string().contains("404") || e.to_string().contains("NoSuchKey") {
                    Ok(false)
                } else {
                    Err(AppError::BlobStorage(format!(
                        "Failed to check blob existence: {}",
                        e
                    )))
                }
            }
        }
    }
}
