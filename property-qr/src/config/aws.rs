 // src/config/aws.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub region: String,
    pub s3_bucket: String,
    pub s3_bucket_region: String,
    pub cloudfront_domain: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub endpoint_url: Option<String>, // For LocalStack or custom endpoints
    pub force_path_style: bool,       // For S3-compatible services
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub qr_images_prefix: String,
    pub metadata_prefix: String,
    pub public_read: bool,
    pub cache_control: String,
    pub content_type_mapping: ContentTypeMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTypeMapping {
    pub png: String,
    pub jpg: String,
    pub svg: String,
    pub json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFrontConfig {
    pub distribution_domain: String,
    pub cache_behavior: CacheBehaviorConfig,
    pub signed_urls: bool,
    pub key_pair_id: Option<String>,
    pub private_key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheBehaviorConfig {
    pub default_ttl: u64,      // seconds
    pub max_ttl: u64,          // seconds
    pub min_ttl: u64,          // seconds
    pub compress: bool,
    pub viewer_protocol_policy: String, // "redirect-to-https", "allow-all", etc.
}

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            s3_bucket: "daobitat-qr-codes".to_string(),
            s3_bucket_region: "us-east-1".to_string(),
            cloudfront_domain: None,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            endpoint_url: None,
            force_path_style: false,
        }
    }
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: "daobitat-qr-codes".to_string(),
            region: "us-east-1".to_string(),
            qr_images_prefix: "qr-images/".to_string(),
            metadata_prefix: "metadata/".to_string(),
            public_read: true,
            cache_control: "public, max-age=31536000".to_string(), // 1 year
            content_type_mapping: ContentTypeMapping::default(),
        }
    }
}

impl Default for ContentTypeMapping {
    fn default() -> Self {
        Self {
            png: "image/png".to_string(),
            jpg: "image/jpeg".to_string(),
            svg: "image/svg+xml".to_string(),
            json: "application/json".to_string(),
        }
    }
}

impl Default for CloudFrontConfig {
    fn default() -> Self {
        Self {
            distribution_domain: "cdn.daobitat.xyz".to_string(),
            cache_behavior: CacheBehaviorConfig::default(),
            signed_urls: false,
            key_pair_id: None,
            private_key_path: None,
        }
    }
}

impl Default for CacheBehaviorConfig {
    fn default() -> Self {
        Self {
            default_ttl: 86400,    // 24 hours
            max_ttl: 31536000,     // 1 year
            min_ttl: 0,
            compress: true,
            viewer_protocol_policy: "redirect-to-https".to_string(),
        }
    }
}

impl AwsConfig {
    /// Create AWS config for development environment
    pub fn development() -> Self {
        Self {
            region: "us-east-1".to_string(),
            s3_bucket: "daobitat-qr-codes-dev".to_string(),
            s3_bucket_region: "us-east-1".to_string(),
            cloudfront_domain: None, // No CloudFront in dev
            access_key_id: None,     // Use default credentials
            secret_access_key: None,
            session_token: None,
            endpoint_url: None,
            force_path_style: false,
        }
    }

    /// Create AWS config for production environment
    pub fn production() -> Self {
        Self {
            region: "us-east-1".to_string(),
            s3_bucket: "daobitat-qr-codes".to_string(),
            s3_bucket_region: "us-east-1".to_string(),
            cloudfront_domain: Some("cdn.daobitat.xyz".to_string()),
            access_key_id: None, // Should use IAM roles in production
            secret_access_key: None,
            session_token: None,
            endpoint_url: None,
            force_path_style: false,
        }
    }

    /// Create AWS config for LocalStack (local development)
    pub fn localstack() -> Self {
        Self {
            region: "us-east-1".to_string(),
            s3_bucket: "daobitat-qr-codes-local".to_string(),
            s3_bucket_region: "us-east-1".to_string(),
            cloudfront_domain: None,
            access_key_id: Some("test".to_string()),
            secret_access_key: Some("test".to_string()),
            session_token: None,
            endpoint_url: Some("http://localhost:4566".to_string()),
            force_path_style: true,
        }
    }

    /// Get the S3 endpoint URL
    pub fn s3_endpoint(&self) -> Option<String> {
        self.endpoint_url.clone()
    }

    /// Check if using custom endpoint (like LocalStack)
    pub fn is_custom_endpoint(&self) -> bool {
        self.endpoint_url.is_some()
    }

    /// Get the public URL for an S3 object
    pub fn get_public_url(&self, key: &str) -> String {
        if let Some(cloudfront_domain) = &self.cloudfront_domain {
            format!("https://{}/{}", cloudfront_domain, key)
        } else if let Some(endpoint_url) = &self.endpoint_url {
            // LocalStack or custom endpoint
            format!("{}/{}/{}", endpoint_url, self.s3_bucket, key)
        } else {
            // Standard S3 URL
            format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.s3_bucket, self.s3_bucket_region, key
            )
        }
    }

    /// Validate the AWS configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.region.is_empty() {
            return Err("AWS region cannot be empty".to_string());
        }

        if self.s3_bucket.is_empty() {
            return Err("S3 bucket name cannot be empty".to_string());
        }

        if self.s3_bucket_region.is_empty() {
            return Err("S3 bucket region cannot be empty".to_string());
        }

        // Validate bucket name format (basic check)
        if !self.s3_bucket.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.') {
            return Err("Invalid S3 bucket name format".to_string());
        }

        if self.s3_bucket.len() < 3 || self.s3_bucket.len() > 63 {
            return Err("S3 bucket name must be between 3 and 63 characters".to_string());
        }

        Ok(())
    }
}

impl S3Config {
    /// Get the full key path for a QR image
    pub fn qr_image_key(&self, property_id: &str) -> String {
        format!("{}{}.png", self.qr_images_prefix, property_id)
    }

    /// Get the full key path for metadata
    pub fn metadata_key(&self, property_id: &str) -> String {
        format!("{}{}.json", self.metadata_prefix, property_id)
    }

    /// Get content type for a file extension
    pub fn get_content_type(&self, extension: &str) -> String {
        match extension.to_lowercase().as_str() {
            "png" => self.content_type_mapping.png.clone(),
            "jpg" | "jpeg" => self.content_type_mapping.jpg.clone(),
            "svg" => self.content_type_mapping.svg.clone(),
            "json" => self.content_type_mapping.json.clone(),
            _ => "application/octet-stream".to_string(),
        }
    }
}

/// AWS credentials helper
#[derive(Debug, Clone)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

impl AwsCredentials {
    /// Create credentials from environment variables
    pub fn from_env() -> Result<Self, String> {
        let access_key_id = std::env::var("AWS_ACCESS_KEY_ID")
            .map_err(|_| "AWS_ACCESS_KEY_ID not found in environment")?;
            
        let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .map_err(|_| "AWS_SECRET_ACCESS_KEY not found in environment")?;
            
        let session_token = std::env::var("AWS_SESSION_TOKEN").ok();

        Ok(Self {
            access_key_id,
            secret_access_key,
            session_token,
        })
    }

    /// Create test credentials for LocalStack
    pub fn test_credentials() -> Self {
        Self {
            access_key_id: "test".to_string(),
            secret_access_key: "test".to_string(),
            session_token: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_config_validation() {
        let config = AwsConfig::default();
        assert!(config.validate().is_ok());
        
        let invalid_config = AwsConfig {
            region: "".to_string(),
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_s3_key_generation() {
        let config = S3Config::default();
        let property_id = "test123";
        
        assert_eq!(
            config.qr_image_key(property_id),
            "qr-images/test123.png"
        );
        
        assert_eq!(
            config.metadata_key(property_id),
            "metadata/test123.json"
        );
    }

    #[test]
    fn test_public_url_generation() {
        let config = AwsConfig {
            cloudfront_domain: Some("cdn.example.com".to_string()),
            ..Default::default()
        };
        
        let url = config.get_public_url("test/file.png");
        assert_eq!(url, "https://cdn.example.com/test/file.png");
        
        let config_no_cloudfront = AwsConfig {
            cloudfront_domain: None,
            s3_bucket: "test-bucket".to_string(),
            s3_bucket_region: "us-west-2".to_string(),
            ..Default::default()
        };
        
        let url = config_no_cloudfront.get_public_url("test/file.png");
        assert_eq!(url, "https://test-bucket.s3.us-west-2.amazonaws.com/test/file.png");
    }

    #[test]
    fn test_content_type_mapping() {
        let config = S3Config::default();
        
        assert_eq!(config.get_content_type("png"), "image/png");
        assert_eq!(config.get_content_type("jpg"), "image/jpeg");
        assert_eq!(config.get_content_type("json"), "application/json");
        assert_eq!(config.get_content_type("unknown"), "application/octet-stream");
    }
}
