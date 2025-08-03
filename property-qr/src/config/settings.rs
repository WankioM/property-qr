 // src/config/settings.rs

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub aws: AwsConfig,
    pub urls: UrlConfig,
    pub qr: QrConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: Environment,
    pub cors_origins: Vec<String>,
    pub request_timeout_seconds: u64,
    pub max_connections: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub mongodb_uri: String,
    pub database_name: String,
    pub connection_timeout_seconds: u64,
    pub max_pool_size: Option<u32>,
    pub min_pool_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub region: String,
    pub s3_bucket: String,
    pub s3_bucket_region: String,
    pub cloudfront_domain: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlConfig {
    pub base_url: String,
    pub daobitat_base_url: String,
    pub blockchain_explorer_base_url: String,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrConfig {
    pub default_size: u32,
    pub default_error_correction: String,
    pub include_logo: bool,
    pub logo_url: Option<String>,
    pub background_color: String,
    pub foreground_color: String,
    pub format: String,
    pub expiry_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub enable_json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Settings {
    /// Load settings from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Settings {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .unwrap_or(3000),
                environment: match env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .to_lowercase()
                    .as_str()
                {
                    "production" => Environment::Production,
                    "staging" => Environment::Staging,
                    _ => Environment::Development,
                },
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000,https://daobitat.xyz".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                request_timeout_seconds: env::var("REQUEST_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                max_connections: env::var("MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok()),
            },
            
            database: DatabaseConfig {
                mongodb_uri: env::var("MONGODB_URI")
                    .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
                database_name: env::var("DATABASE_NAME")
                    .unwrap_or_else(|_| "daobitat_qr".to_string()),
                connection_timeout_seconds: env::var("DB_CONNECTION_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                max_pool_size: env::var("DB_MAX_POOL_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                min_pool_size: env::var("DB_MIN_POOL_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok()),
            },
            
            aws: AwsConfig {
                region: env::var("AWS_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
                s3_bucket: env::var("S3_BUCKET")
                    .unwrap_or_else(|_| "daobitat-qr-codes".to_string()),
                s3_bucket_region: env::var("S3_BUCKET_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
                cloudfront_domain: env::var("CLOUDFRONT_DOMAIN").ok(),
                access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
                secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
                session_token: env::var("AWS_SESSION_TOKEN").ok(),
            },
            
            urls: UrlConfig {
                base_url: env::var("BASE_URL")
                    .unwrap_or_else(|_| "https://qr-service.daobitat.xyz".to_string()),
                daobitat_base_url: env::var("DAOBITAT_BASE_URL")
                    .unwrap_or_else(|_| "https://www.daobitat.xyz".to_string()),
                blockchain_explorer_base_url: env::var("BLOCKCHAIN_EXPLORER_BASE_URL")
                    .unwrap_or_else(|_| "https://basescan.org".to_string()),
                api_version: env::var("API_VERSION")
                    .unwrap_or_else(|_| "v1".to_string()),
            },
            
            qr: QrConfig {
                default_size: env::var("QR_DEFAULT_SIZE")
                    .unwrap_or_else(|_| "256".to_string())
                    .parse()
                    .unwrap_or(256),
                default_error_correction: env::var("QR_ERROR_CORRECTION")
                    .unwrap_or_else(|_| "medium".to_string()),
                include_logo: env::var("QR_INCLUDE_LOGO")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                logo_url: env::var("QR_LOGO_URL").ok(),
                background_color: env::var("QR_BACKGROUND_COLOR")
                    .unwrap_or_else(|_| "#FFFFFF".to_string()),
                foreground_color: env::var("QR_FOREGROUND_COLOR")
                    .unwrap_or_else(|_| "#000000".to_string()),
                format: env::var("QR_FORMAT")
                    .unwrap_or_else(|_| "png".to_string()),
                expiry_days: env::var("QR_EXPIRY_DAYS")
                    .unwrap_or_else(|_| "365".to_string())
                    .parse()
                    .unwrap_or(365),
            },
            
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT")
                    .unwrap_or_else(|_| "pretty".to_string()),
                enable_json: env::var("LOG_JSON")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
            },
        })
    }

    /// Create default settings for development
    pub fn default_dev() -> Self {
        Settings {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                environment: Environment::Development,
                cors_origins: vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:3001".to_string(),
                ],
                request_timeout_seconds: 30,
                max_connections: Some(100),
            },
            
            database: DatabaseConfig {
                mongodb_uri: "mongodb://localhost:27017".to_string(),
                database_name: "daobitat_qr_dev".to_string(),
                connection_timeout_seconds: 10,
                max_pool_size: Some(10),
                min_pool_size: Some(1),
            },
            
            aws: AwsConfig {
                region: "us-east-1".to_string(),
                s3_bucket: "daobitat-qr-codes-dev".to_string(),
                s3_bucket_region: "us-east-1".to_string(),
                cloudfront_domain: None,
                access_key_id: None,
                secret_access_key: None,
                session_token: None,
            },
            
            urls: UrlConfig {
                base_url: "http://localhost:3000".to_string(),
                daobitat_base_url: "http://localhost:3001".to_string(),
                blockchain_explorer_base_url: "https://sepolia.basescan.org".to_string(),
                api_version: "v1".to_string(),
            },
            
            qr: QrConfig {
                default_size: 256,
                default_error_correction: "medium".to_string(),
                include_logo: true,
                logo_url: Some("https://daobitat.xyz/logo.png".to_string()),
                background_color: "#FFFFFF".to_string(),
                foreground_color: "#000000".to_string(),
                format: "png".to_string(),
                expiry_days: 30, // Shorter expiry for dev
            },
            
            logging: LoggingConfig {
                level: "debug".to_string(),
                format: "pretty".to_string(),
                enable_json: false,
            },
        }
    }

    /// Create production settings
    pub fn default_prod() -> Self {
        Settings {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                environment: Environment::Production,
                cors_origins: vec![
                    "https://www.daobitat.xyz".to_string(),
                    "https://app.daobitat.xyz".to_string(),
                ],
                request_timeout_seconds: 30,
                max_connections: Some(1000),
            },
            
            database: DatabaseConfig {
                mongodb_uri: "mongodb://prod-cluster:27017".to_string(),
                database_name: "daobitat_qr".to_string(),
                connection_timeout_seconds: 10,
                max_pool_size: Some(50),
                min_pool_size: Some(5),
            },
            
            aws: AwsConfig {
                region: "us-east-1".to_string(),
                s3_bucket: "daobitat-qr-codes".to_string(),
                s3_bucket_region: "us-east-1".to_string(),
                cloudfront_domain: Some("cdn.daobitat.xyz".to_string()),
                access_key_id: None, // Should come from IAM role or env vars
                secret_access_key: None,
                session_token: None,
            },
            
            urls: UrlConfig {
                base_url: "https://qr-service.daobitat.xyz".to_string(),
                daobitat_base_url: "https://www.daobitat.xyz".to_string(),
                blockchain_explorer_base_url: "https://basescan.org".to_string(),
                api_version: "v1".to_string(),
            },
            
            qr: QrConfig {
                default_size: 512, // Higher quality for production
                default_error_correction: "high".to_string(),
                include_logo: true,
                logo_url: Some("https://cdn.daobitat.xyz/logo.png".to_string()),
                background_color: "#FFFFFF".to_string(),
                foreground_color: "#000000".to_string(),
                format: "png".to_string(),
                expiry_days: 365,
            },
            
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                enable_json: true,
            },
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate server config
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        // Validate database config
        if self.database.mongodb_uri.is_empty() {
            return Err("MongoDB URI cannot be empty".to_string());
        }

        if self.database.database_name.is_empty() {
            return Err("Database name cannot be empty".to_string());
        }

        // Validate AWS config
        if self.aws.s3_bucket.is_empty() {
            return Err("S3 bucket name cannot be empty".to_string());
        }

        // Validate URLs
        if !self.urls.base_url.starts_with("http") {
            return Err("Base URL must start with http or https".to_string());
        }

        if !self.urls.daobitat_base_url.starts_with("http") {
            return Err("DAO-Bitat base URL must start with http or https".to_string());
        }

        // Validate QR config
        if self.qr.default_size < 64 || self.qr.default_size > 2048 {
            return Err("QR size must be between 64 and 2048 pixels".to_string());
        }

        Ok(())
    }

    /// Get the full API base URL
    pub fn api_base_url(&self) -> String {
        format!("{}/api/{}", self.urls.base_url, self.urls.api_version)
    }

    /// Check if running in development mode
    pub fn is_development(&self) -> bool {
        matches!(self.server.environment, Environment::Development)
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        matches!(self.server.environment, Environment::Production)
    }

    /// Get the S3 public URL
    pub fn s3_public_url(&self, key: &str) -> String {
        if let Some(cloudfront_domain) = &self.aws.cloudfront_domain {
            format!("https://{}/{}", cloudfront_domain, key)
        } else {
            format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.aws.s3_bucket, self.aws.s3_bucket_region, key
            )
        }
    }
}
