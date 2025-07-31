// src/models/qr_code.rs

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeMetadata {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "propertyId")]
    pub property_id: String, // MongoDB property ID as string
    #[serde(rename = "qrCodeUrl")]
    pub qr_code_url: String, // S3 URL to the QR code image
    #[serde(rename = "qrPattern")]
    pub qr_pattern: String, // The actual QR data/content
    #[serde(rename = "qrCodeHash")]
    pub qr_code_hash: String, // SHA256 hash of QR content for verification
    #[serde(rename = "generatedAt")]
    pub generated_at: DateTime<Utc>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
    #[serde(rename = "scanCount")]
    pub scan_count: i64,
    #[serde(rename = "lastScanned")]
    pub last_scanned: Option<DateTime<Utc>>,
    #[serde(rename = "isActive")]
    pub is_active: bool, // Whether QR code is active or disabled
    #[serde(rename = "qrVersion")]
    pub qr_version: i32, // Version number for QR regeneration tracking
    pub metadata: QrMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrMetadata {
    #[serde(rename = "propertyName")]
    pub property_name: String,
    pub location: String,
    pub action: String, // "for sale" | "for rent"
    pub price: i64,
    #[serde(rename = "onchainId")]
    pub onchain_id: Option<String>,
    #[serde(rename = "cryptoAccepted")]
    pub crypto_accepted: bool,
    #[serde(rename = "primaryImage")]
    pub primary_image: Option<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "generatedBy")]
    pub generated_by: Option<ObjectId>, // User who generated the QR
    #[serde(rename = "generationReason")]
    pub generation_reason: QrGenerationReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QrGenerationReason {
    NewProperty,
    PropertyUpdated,
    ManualRegeneration,
    BatchGeneration,
    ExpiredQr,
}

// QR Code data structure that gets encoded into the QR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeData {
    #[serde(rename = "type")]
    pub qr_type: String, // Always "daobitat_property"
    #[serde(rename = "propertyId")]
    pub property_id: String,
    #[serde(rename = "scanUrl")]
    pub scan_url: String,
    #[serde(rename = "version")]
    pub version: String, // QR format version
    #[serde(rename = "timestamp")]
    pub timestamp: i64, // Unix timestamp when QR was generated
}

// Request/Response DTOs for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateQrRequest {
    #[serde(rename = "propertyId")]
    pub property_id: String,
    #[serde(rename = "forceRegenerate")]
    pub force_regenerate: Option<bool>, // Force regeneration even if QR exists
    pub reason: Option<QrGenerationReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGenerateQrRequest {
    #[serde(rename = "propertyIds")]
    pub property_ids: Vec<String>,
    #[serde(rename = "forceRegenerate")]
    pub force_regenerate: Option<bool>,
    pub reason: Option<QrGenerationReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeResponse {
    #[serde(rename = "propertyId")]
    pub property_id: String,
    #[serde(rename = "qrCodeUrl")]
    pub qr_code_url: String,
    #[serde(rename = "scanUrl")]
    pub scan_url: String,
    #[serde(rename = "generatedAt")]
    pub generated_at: DateTime<Utc>,
    pub metadata: QrMetadata,
    pub status: QrStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQrCodeResponse {
    pub successful: Vec<QrCodeResponse>,
    pub failed: Vec<QrGenerationError>,
    #[serde(rename = "totalRequested")]
    pub total_requested: usize,
    #[serde(rename = "totalSuccessful")]
    pub total_successful: usize,
    #[serde(rename = "totalFailed")]
    pub total_failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrGenerationError {
    #[serde(rename = "propertyId")]
    pub property_id: String,
    pub error: String,
    #[serde(rename = "errorCode")]
    pub error_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QrStatus {
    Generated,
    Regenerated,
    Failed,
    Exists,
}

// S3 storage configuration for QR codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrS3Config {
    pub bucket: String,
    pub region: String,
    #[serde(rename = "qrImagePrefix")]
    pub qr_image_prefix: String, // "qr-images/"
    #[serde(rename = "metadataPrefix")]
    pub metadata_prefix: String, // "metadata/"
    #[serde(rename = "publicBaseUrl")]
    pub public_base_url: String, // CloudFront URL if used
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum QrErrorCorrection {
    Low,    // ~7%
    Medium, // ~15%
    Quartile, // ~25%
    High,   // ~30%
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum QrImageFormat {
    Png,
    Jpeg,
    Svg,
}
// QR Code generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrGenerationSettings {
    pub size: u32,           // QR code size in pixels (e.g., 256)
    #[serde(rename = "errorCorrection")]
    pub error_correction: QrErrorCorrection,
    #[serde(rename = "includelogo")]
    pub include_logo: bool,  // Whether to include DAO-Bitat logo
    #[serde(rename = "logoUrl")]
    pub logo_url: Option<String>, // URL to logo image
    #[serde(rename = "backgroundColor")]
    pub background_color: String, // Hex color code
    #[serde(rename = "foregroundColor")]
    pub foreground_color: String, // Hex color code
    pub format: QrImageFormat,
}



impl QrCodeMetadata {
    /// Create a new QR code metadata entry
    pub fn new(
        property_id: String,
        qr_pattern: String,
        qr_code_url: String,
        metadata: QrMetadata,
    ) -> Self {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(&qr_pattern);
        let qr_code_hash = format!("{:x}", hasher.finalize());

        Self {
            id: ObjectId::new(),
            property_id,
            qr_code_url,
            qr_pattern,
            qr_code_hash,
            generated_at: Utc::now(),
            last_updated: Utc::now(),
            scan_count: 0,
            last_scanned: None,
            is_active: true,
            qr_version: 1,
            metadata,
        }
    }

    /// Increment scan count and update last scanned timestamp
    pub fn record_scan(&mut self) {
        self.scan_count += 1;
        self.last_scanned = Some(Utc::now());
        self.last_updated = Utc::now();
    }

    /// Deactivate the QR code
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.last_updated = Utc::now();
    }

    /// Regenerate QR with new version
    pub fn regenerate(&mut self, new_pattern: String, new_url: String) {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(&new_pattern);
        
        self.qr_pattern = new_pattern;
        self.qr_code_url = new_url;
        self.qr_code_hash = format!("{:x}", hasher.finalize());
        self.qr_version += 1;
        self.last_updated = Utc::now();
        self.is_active = true;
    }

    /// Check if QR code is expired (older than X days)
    pub fn is_expired(&self, expiry_days: i64) -> bool {
        let expiry_date = self.generated_at + chrono::Duration::days(expiry_days);
        Utc::now() > expiry_date
    }

    /// Get S3 key for the QR image
    pub fn get_s3_key(&self) -> String {
        format!("qr-images/{}.png", self.property_id)
    }

    /// Get S3 key for metadata
    pub fn get_metadata_s3_key(&self) -> String {
        format!("metadata/{}.json", self.property_id)
    }
}

impl QrCodeData {
    /// Create new QR code data
    pub fn new(property_id: String, scan_base_url: &str) -> Self {
        Self {
            qr_type: "daobitat_property".to_string(),
            property_id: property_id.clone(),
            scan_url: format!("{}/scan/{}", scan_base_url, property_id),
            version: "1.0".to_string(),
            timestamp: Utc::now().timestamp(),
        }
    }

    /// Convert to JSON string for QR encoding
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Create from JSON string
    pub fn from_json_string(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Validate QR data structure
    pub fn is_valid(&self) -> bool {
        !self.property_id.is_empty() 
            && !self.scan_url.is_empty() 
            && self.qr_type == "daobitat_property"
            && !self.version.is_empty()
    }
}

impl Default for QrGenerationSettings {
    fn default() -> Self {
        Self {
            size: 256,
            error_correction: QrErrorCorrection::Medium,
            include_logo: true,
            logo_url: Some("https://daobitat.xyz/logo.png".to_string()),
            background_color: "#FFFFFF".to_string(),
            foreground_color: "#000000".to_string(),
            format: QrImageFormat::Png,
        }
    }
}