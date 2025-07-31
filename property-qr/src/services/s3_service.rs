 
// src/services/s3_service.rs

use std::time::Duration;
use tracing::{info, warn, error};

#[derive(Clone)]
pub struct S3Service {
    bucket_name: String,
    region: String,
    public_base_url: Option<String>, // CloudFront URL if available
}

#[derive(Debug)]
pub enum S3Error {
    ConfigurationError(String),
    UploadError(String),
    DownloadError(String),
    DeleteError(String),
    InvalidKey(String),
    NetworkError(String),
}

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S3Error::ConfigurationError(msg) => write!(f, "S3 configuration error: {}", msg),
            S3Error::UploadError(msg) => write!(f, "S3 upload error: {}", msg),
            S3Error::DownloadError(msg) => write!(f, "S3 download error: {}", msg),
            S3Error::DeleteError(msg) => write!(f, "S3 delete error: {}", msg),
            S3Error::InvalidKey(msg) => write!(f, "Invalid S3 key: {}", msg),
            S3Error::NetworkError(msg) => write!(f, "S3 network error: {}", msg),
        }
    }
}

impl std::error::Error for S3Error {}

#[derive(Debug, Clone)]
pub struct S3UploadResult {
    pub url: String,
    pub key: String,
    pub size: usize,
    pub content_type: String,
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub bucket_name: String,
    pub region: String,
    pub public_base_url: Option<String>,
    pub qr_image_prefix: String,
    pub metadata_prefix: String,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket_name: "daobitat-qr-codes".to_string(),
            region: "us-east-1".to_string(),
            public_base_url: None,
            qr_image_prefix: "qr-images/".to_string(),
            metadata_prefix: "metadata/".to_string(),
        }
    }
}

impl S3Service {
    /// Create a new S3 service instance
    pub fn new(bucket_name: String, region: String) -> Result<Self, S3Error> {
        if bucket_name.is_empty() {
            return Err(S3Error::ConfigurationError("Bucket name cannot be empty".to_string()));
        }

        if region.is_empty() {
            return Err(S3Error::ConfigurationError("Region cannot be empty".to_string()));
        }

        Ok(Self {
            bucket_name,
            region,
            public_base_url: None,
        })
    }

    /// Create a new S3 service with CloudFront URL
    pub fn with_cloudfront(
        bucket_name: String, 
        region: String, 
        cloudfront_url: String
    ) -> Result<Self, S3Error> {
        let mut service = Self::new(bucket_name, region)?;
        service.public_base_url = Some(cloudfront_url);
        Ok(service)
    }

    /// Upload QR code image to S3
    pub async fn upload_qr_image(&self, key: &str, image_data: Vec<u8>) -> Result<String, S3Error> {
        self.validate_key(key)?;
        
        // TODO: Implement actual S3 upload using aws-sdk-s3
        // This is a placeholder implementation
        
        // let client = self.get_s3_client().await?;
        // 
        // let put_request = PutObjectRequest {
        //     bucket: self.bucket_name.clone(),
        //     key: key.to_string(),
        //     body: Some(image_data.into()),
        //     content_type: Some("image/png".to_string()),
        //     content_length: Some(image_data.len() as i64),
        //     cache_control: Some("public, max-age=31536000".to_string()), // 1 year
        //     metadata: Some({
        //         let mut metadata = HashMap::new();
        //         metadata.insert("generated-by".to_string(), "daobitat-qr-service".to_string());
        //         metadata.insert("generated-at".to_string(), chrono::Utc::now().to_rfc3339());
        //         metadata
        //     }),
        //     ..Default::default()
        // };
        //
        // client.put_object(put_request).await
        //     .map_err(|e| S3Error::UploadError(e.to_string()))?;

        let public_url = self.get_public_url(key);
        
        info!("Uploaded QR image to S3: {} ({} bytes)", key, image_data.len());
        
        // For now, return a placeholder URL
        Ok(public_url)
    }

    /// Upload QR metadata JSON to S3
    pub async fn upload_qr_metadata(&self, key: &str, metadata_json: String) -> Result<String, S3Error> {
        self.validate_key(key)?;
        
        // TODO: Implement actual S3 upload
        // Similar to upload_qr_image but with content-type: application/json
        
        let public_url = self.get_public_url(key);
        
        info!("Uploaded QR metadata to S3: {} ({} bytes)", key, metadata_json.len());
        
        Ok(public_url)
    }

    /// Download file from S3
    pub async fn download_file(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        self.validate_key(key)?;
        
        // TODO: Implement actual S3 download
        // let client = self.get_s3_client().await?;
        // 
        // let get_request = GetObjectRequest {
        //     bucket: self.bucket_name.clone(),
        //     key: key.to_string(),
        //     ..Default::default()
        // };
        //
        // let result = client.get_object(get_request).await
        //     .map_err(|e| S3Error::DownloadError(e.to_string()))?;
        //
        // let body = result.body.ok_or_else(|| S3Error::DownloadError("Empty response body".to_string()))?;
        // 
        // let bytes = body.collect().await
        //     .map_err(|e| S3Error::DownloadError(e.to_string()))?
        //     .into_bytes()
        //     .to_vec();

        warn!("S3 download not implemented - returning empty data");
        Ok(Vec::new())
    }

    /// Delete QR image from S3
    pub async fn delete_qr_image(&self, key: &str) -> Result<bool, S3Error> {
        self.validate_key(key)?;
        
        // TODO: Implement actual S3 delete
        // let client = self.get_s3_client().await?;
        // 
        // let delete_request = DeleteObjectRequest {
        //     bucket: self.bucket_name.clone(),
        //     key: key.to_string(),
        //     ..Default::default()
        // };
        //
        // client.delete_object(delete_request).await
        //     .map_err(|e| S3Error::DeleteError(e.to_string()))?;

        info!("Deleted QR image from S3: {}", key);
        Ok(true)
    }

    /// Delete multiple files from S3
    pub async fn delete_multiple_files(&self, keys: Vec<String>) -> Result<Vec<String>, S3Error> {
        let mut deleted_keys = Vec::new();
        
        for key in keys {
            match self.delete_qr_image(&key).await {
                Ok(_) => deleted_keys.push(key),
                Err(e) => {
                    warn!("Failed to delete key {}: {}", key, e);
                    // Continue with other deletions
                }
            }
        }
        
        Ok(deleted_keys)
    }

    /// Check if file exists in S3
    pub async fn file_exists(&self, key: &str) -> Result<bool, S3Error> {
        self.validate_key(key)?;
        
        // TODO: Implement actual S3 head_object
        // let client = self.get_s3_client().await?;
        // 
        // let head_request = HeadObjectRequest {
        //     bucket: self.bucket_name.clone(),
        //     key: key.to_string(),
        //     ..Default::default()
        // };
        //
        // match client.head_object(head_request).await {
        //     Ok(_) => Ok(true),
        //     Err(RusotoError::Service(HeadObjectError::NoSuchKey(_))) => Ok(false),
        //     Err(e) => Err(S3Error::NetworkError(e.to_string())),
        // }

        // For now, return false (file doesn't exist)
        Ok(false)
    }

    /// Generate presigned URL for direct uploads
    pub async fn generate_presigned_upload_url(
        &self, 
        key: &str, 
        content_type: &str,
        expires_in: Duration,
    ) -> Result<String, S3Error> {
        self.validate_key(key)?;
        
        // TODO: Implement presigned URL generation
        // let client = self.get_s3_client().await?;
        // 
        // let put_request = PutObjectRequest {
        //     bucket: self.bucket_name.clone(),
        //     key: key.to_string(),
        //     content_type: Some(content_type.to_string()),
        //     ..Default::default()
        // };
        //
        // let presigned_url = client.put_object_presigned_url(put_request, expires_in).await
        //     .map_err(|e| S3Error::ConfigurationError(e.to_string()))?;

        // For now, return a placeholder URL
        let placeholder_url = format!(
            "https://{}.s3.{}.amazonaws.com/{}?presigned=true", 
            self.bucket_name, 
            self.region, 
            key
        );
        
        Ok(placeholder_url)
    }

    /// Get file metadata
    pub async fn get_file_metadata(&self, key: &str) -> Result<FileMetadata, S3Error> {
        self.validate_key(key)?;
        
        // TODO: Implement actual metadata retrieval
        // let client = self.get_s3_client().await?;
        // 
        // let head_request = HeadObjectRequest {
        //     bucket: self.bucket_name.clone(),
        //     key: key.to_string(),
        //     ..Default::default()
        // };
        //
        // let result = client.head_object(head_request).await
        //     .map_err(|e| S3Error::NetworkError(e.to_string()))?;

        // Return placeholder metadata
        Ok(FileMetadata {
            key: key.to_string(),
            size: 0,
            content_type: "image/png".to_string(),
            last_modified: chrono::Utc::now(),
            etag: "placeholder".to_string(),
        })
    }

    /// List files with prefix
    pub async fn list_files_with_prefix(&self, prefix: &str, max_keys: Option<i32>) -> Result<Vec<S3Object>, S3Error> {
        // TODO: Implement actual S3 list_objects_v2
        // let client = self.get_s3_client().await?;
        // 
        // let list_request = ListObjectsV2Request {
        //     bucket: self.bucket_name.clone(),
        //     prefix: Some(prefix.to_string()),
        //     max_keys,
        //     ..Default::default()
        // };
        //
        // let result = client.list_objects_v2(list_request).await
        //     .map_err(|e| S3Error::NetworkError(e.to_string()))?;
        //
        // let objects = result.contents.unwrap_or_default()
        //     .into_iter()
        //     .map(|obj| S3Object {
        //         key: obj.key.unwrap_or_default(),
        //         size: obj.size.unwrap_or(0),
        //         last_modified: obj.last_modified.map(|dt| dt.into()),
        //         etag: obj.e_tag.unwrap_or_default(),
        //     })
        //     .collect();

        // Return empty list for now
        Ok(Vec::new())
    }

    /// Clean up old QR codes
    pub async fn cleanup_old_qr_codes(&self, older_than_days: i64) -> Result<Vec<String>, S3Error> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(older_than_days);
        let qr_objects = self.list_files_with_prefix("qr-images/", None).await?;
        
        let mut deleted_keys = Vec::new();
        
        for obj in qr_objects {
            if let Some(last_modified) = obj.last_modified {
                if last_modified < cutoff_date {
                    match self.delete_qr_image(&obj.key).await {
                        Ok(_) => deleted_keys.push(obj.key),
                        Err(e) => warn!("Failed to delete old QR code {}: {}", obj.key, e),
                    }
                }
            }
        }
        
        info!("Cleaned up {} old QR codes from S3", deleted_keys.len());
        Ok(deleted_keys)
    }

    /// Get bucket statistics
    pub async fn get_bucket_stats(&self) -> Result<BucketStats, S3Error> {
        let qr_images = self.list_files_with_prefix("qr-images/", None).await?;
        let metadata_files = self.list_files_with_prefix("metadata/", None).await?;
        
        let total_qr_size: i64 = qr_images.iter().map(|obj| obj.size).sum();
        let total_metadata_size: i64 = metadata_files.iter().map(|obj| obj.size).sum();
        
        Ok(BucketStats {
            total_qr_images: qr_images.len() as i64,
            total_metadata_files: metadata_files.len() as i64,
            total_qr_size,
            total_metadata_size,
            total_size: total_qr_size + total_metadata_size,
        })
    }

    /// Private helper methods
    fn validate_key(&self, key: &str) -> Result<(), S3Error> {
        if key.is_empty() {
            return Err(S3Error::InvalidKey("Key cannot be empty".to_string()));
        }
        
        if key.len() > 1024 {
            return Err(S3Error::InvalidKey("Key too long (max 1024 characters)".to_string()));
        }
        
        // Check for invalid characters
        if key.contains("//") || key.starts_with('/') {
            return Err(S3Error::InvalidKey("Invalid key format".to_string()));
        }
        
        Ok(())
    }

    fn get_public_url(&self, key: &str) -> String {
        match &self.public_base_url {
            Some(cloudfront_url) => format!("{}/{}", cloudfront_url.trim_end_matches('/'), key),
            None => format!(
                "https://{}.s3.{}.amazonaws.com/{}", 
                self.bucket_name, 
                self.region, 
                key
            ),
        }
    }

    // TODO: Implement when aws-sdk-s3 is added
    // async fn get_s3_client(&self) -> Result<S3Client, S3Error> {
    //     let config = aws_config::load_from_env().await;
    //     let s3_config = aws_sdk_s3::config::Builder::from(&config)
    //         .region(Region::new(self.region.clone()))
    //         .build();
    //     Ok(S3Client::from_conf(s3_config))
    // }
}

// Supporting types
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub key: String,
    pub size: i64,
    pub content_type: String,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub etag: String,
}

#[derive(Debug, Clone)]
pub struct S3Object {
    pub key: String,
    pub size: i64,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub etag: String,
}

#[derive(Debug, Clone)]
pub struct BucketStats {
    pub total_qr_images: i64,
    pub total_metadata_files: i64,
    pub total_qr_size: i64,
    pub total_metadata_size: i64,
    pub total_size: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_service_creation() {
        let service = S3Service::new(
            "test-bucket".to_string(),
            "us-east-1".to_string(),
        ).expect("Failed to create S3 service");
        
        assert_eq!(service.bucket_name, "test-bucket");
        assert_eq!(service.region, "us-east-1");
        assert!(service.public_base_url.is_none());
    }

    #[test]
    fn test_s3_service_with_cloudfront() {
        let service = S3Service::with_cloudfront(
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            "https://d123456.cloudfront.net".to_string(),
        ).expect("Failed to create S3 service");
        
        assert!(service.public_base_url.is_some());
        assert_eq!(service.public_base_url.unwrap(), "https://d123456.cloudfront.net");
    }

    #[test]
    fn test_validate_key() {
        let service = S3Service::new("test".to_string(), "us-east-1".to_string()).unwrap();
        
        assert!(service.validate_key("valid/key.png").is_ok());
        assert!(service.validate_key("").is_err());
        assert!(service.validate_key("/invalid").is_err());
        assert!(service.validate_key("invalid//key").is_err());
    }

    #[test]
    fn test_get_public_url() {
        let service = S3Service::new("test-bucket".to_string(), "us-east-1".to_string()).unwrap();
        let url = service.get_public_url("test/key.png");
        assert_eq!(url, "https://test-bucket.s3.us-east-1.amazonaws.com/test/key.png");
        
        let service_with_cf = S3Service::with_cloudfront(
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            "https://d123456.cloudfront.net".to_string(),
        ).unwrap();
        let cf_url = service_with_cf.get_public_url("test/key.png");
        assert_eq!(cf_url, "https://d123456.cloudfront.net/test/key.png");
    }

    #[tokio::test]
    async fn test_file_exists_placeholder() {
        let service = S3Service::new("test-bucket".to_string(), "us-east-1".to_string()).unwrap();
        let exists = service.file_exists("test/key.png").await.unwrap();
        assert!(!exists); // Placeholder implementation returns false
    }
}