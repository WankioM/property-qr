 
// src/services/qr_generator.rs

use crate::models::{
    QrCodeMetadata, QrCodeData, QrMetadata, QrGenerationSettings, QrGenerationReason,
    QrStatus, QrCodeResponse, BatchQrCodeResponse, QrGenerationError, PropertyQrInfo
};
use crate::services::{PropertyService, S3Service};
use mongodb::{
    bson::{doc, oid::ObjectId}, 
options::FindOptions, Collection, Database};

use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, warn, error};

#[derive(Clone)]
pub struct QrGeneratorService {
    qr_metadata: Collection<QrCodeMetadata>,
    property_service: PropertyService,
    s3_service: S3Service,
    settings: QrGenerationSettings,
    base_url: String,
}

#[derive(Debug)]
pub enum QrGeneratorError {
    PropertyNotFound,
    PropertyNotEligible(String),
    QrGenerationFailed(String),
    S3UploadFailed(String),
    DatabaseError(mongodb::error::Error),
    InvalidPropertyId,
}

// Helper function to convert chrono DateTime to BSON DateTime
fn utc_to_bson(dt: chrono::DateTime<chrono::Utc>) -> mongodb::bson::DateTime {
    mongodb::bson::DateTime::from_millis(dt.timestamp_millis())
}

impl From<mongodb::error::Error> for QrGeneratorError {
    fn from(err: mongodb::error::Error) -> Self {
        QrGeneratorError::DatabaseError(err)
    }
}

impl std::fmt::Display for QrGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QrGeneratorError::PropertyNotFound => write!(f, "Property not found"),
            QrGeneratorError::PropertyNotEligible(reason) => write!(f, "Property not eligible: {}", reason),
            QrGeneratorError::QrGenerationFailed(reason) => write!(f, "QR generation failed: {}", reason),
            QrGeneratorError::S3UploadFailed(reason) => write!(f, "S3 upload failed: {}", reason),
            QrGeneratorError::DatabaseError(e) => write!(f, "Database error: {}", e),
            QrGeneratorError::InvalidPropertyId => write!(f, "Invalid property ID"),
        }
    }
}

impl std::error::Error for QrGeneratorError {}

impl QrGeneratorService {
    /// Create a new QR generator service
    pub fn new(
        db: &Database,
        property_service: PropertyService,
        s3_service: S3Service,
        base_url: String,
    ) -> Self {
        Self {
            qr_metadata: db.collection("qr_metadata"),
            property_service,
            s3_service,
            settings: QrGenerationSettings::default(),
            base_url,
        }
    }

    /// Create a new QR generator service with custom settings
    pub fn with_settings(
        db: &Database,
        property_service: PropertyService,
        s3_service: S3Service,
        base_url: String,
        settings: QrGenerationSettings,
    ) -> Self {
        Self {
            qr_metadata: db.collection("qr_metadata"),
            property_service,
            s3_service,
            settings,
            base_url,
        }
    }

    /// Generate QR code for a single property
    pub async fn generate_qr_code(
        &self,
        property_id: String,
        force_regenerate: bool,
        reason: QrGenerationReason,
    ) -> Result<QrCodeResponse, QrGeneratorError> {
        let start_time = std::time::Instant::now();

        // Check if QR already exists and force_regenerate is false
        if !force_regenerate {
            if let Ok(existing_qr) = self.get_existing_qr(&property_id).await {
                if existing_qr.is_active {
                    return Ok(QrCodeResponse {
                        property_id: property_id.clone(),
                        qr_code_url: existing_qr.qr_code_url,
                        scan_url: format!("{}/scan/{}", self.base_url, property_id),
                        generated_at: existing_qr.generated_at,
                        metadata: existing_qr.metadata,
                        status: QrStatus::Exists,
                    });
                }
            }
        }

        // Get property information
        let property_info = self.property_service
            .get_property_qr_info(&property_id)
            .await
            .map_err(|e| match e {
                crate::services::property_service::PropertyError::NotFound => QrGeneratorError::PropertyNotFound,
                crate::services::property_service::PropertyError::NotEligibleForQr(reason) => QrGeneratorError::PropertyNotEligible(reason),
                crate::services::property_service::PropertyError::InvalidId => QrGeneratorError::InvalidPropertyId,
                crate::services::property_service::PropertyError::DatabaseError(db_err) => QrGeneratorError::DatabaseError(db_err),
            })?;

        // Create QR code data
        let qr_data = QrCodeData::new(property_id.clone(), &self.base_url);
        let qr_json = qr_data.to_json_string()
            .map_err(|e| QrGeneratorError::QrGenerationFailed(e.to_string()))?;

        // Generate QR code image
        let qr_image_data = self.generate_qr_image(&qr_json).await?;

        // Upload to S3
        let s3_key = format!("qr-images/{}.png", property_id);
        let qr_code_url = self.s3_service
            .upload_qr_image(&s3_key, qr_image_data)
            .await
            .map_err(|e| QrGeneratorError::S3UploadFailed(e.to_string()))?;

        // Create metadata
        let metadata = QrMetadata {
            property_name: property_info.property_name,
            location: property_info.location,
            action: property_info.action,
            price: property_info.price,
            onchain_id: property_info.onchain_id,
            crypto_accepted: property_info.crypto_accepted,
            primary_image: property_info.images.first().cloned(),
            is_verified: property_info.is_verified.unwrap_or(false),
            generated_by: None, // TODO: Add user context
            generation_reason: reason.clone(),
        };

        // Create QR metadata record
        let qr_metadata = if force_regenerate {
            // Update existing QR
            let mut existing = self.get_existing_qr(&property_id).await
                .unwrap_or_else(|_| QrCodeMetadata::new(property_id.clone(), qr_json.clone(), qr_code_url.clone(), metadata.clone()));
            existing.regenerate(qr_json, qr_code_url.clone());
            existing.metadata = metadata.clone();
            existing
        } else {
            // Create new QR
            QrCodeMetadata::new(property_id.clone(), qr_json, qr_code_url.clone(), metadata.clone())
        };

        // Save to database
        self.upsert_qr_metadata(&qr_metadata).await?;

        let generation_time = start_time.elapsed();
        info!(
            "Generated QR code for property {} in {:?}", 
            property_id, 
            generation_time
        );

        Ok(QrCodeResponse {
            property_id,
            qr_code_url,
            scan_url: format!("{}/scan/{}", self.base_url, qr_metadata.property_id),
            generated_at: qr_metadata.generated_at,
            metadata,
            status: if force_regenerate { QrStatus::Regenerated } else { QrStatus::Generated },
        })
    }

    /// Generate QR codes for multiple properties
    pub async fn batch_generate_qr_codes(
        &self,
        property_ids: Vec<String>,
        force_regenerate: bool,
        reason: QrGenerationReason,
    ) -> Result<BatchQrCodeResponse, QrGeneratorError> {
        let mut successful = Vec::new();
        let mut failed = Vec::new();
        let total_requested = property_ids.len();

        info!("Starting batch QR generation for {} properties", total_requested);

        // Process each property
        for property_id in property_ids {
            match self.generate_qr_code(property_id.clone(), force_regenerate, reason.clone()).await {
                Ok(qr_response) => {
                    successful.push(qr_response);
                }
                Err(e) => {
                    let error_code = match e {
                        QrGeneratorError::PropertyNotFound => "PROPERTY_NOT_FOUND",
                        QrGeneratorError::PropertyNotEligible(_) => "PROPERTY_NOT_ELIGIBLE",
                        QrGeneratorError::QrGenerationFailed(_) => "QR_GENERATION_FAILED",
                        QrGeneratorError::S3UploadFailed(_) => "S3_UPLOAD_FAILED",
                        QrGeneratorError::DatabaseError(_) => "DATABASE_ERROR",
                        QrGeneratorError::InvalidPropertyId => "INVALID_PROPERTY_ID",
                    };

                    failed.push(QrGenerationError {
                        property_id,
                        error: e.to_string(),
                        error_code: error_code.to_string(),
                    });
                }
            }
        }

        let total_successful = successful.len();
        let total_failed = failed.len();

        info!(
            "Batch QR generation completed: {} successful, {} failed", 
            total_successful, 
            total_failed
        );

        Ok(BatchQrCodeResponse {
            successful,
            failed,
            total_requested,
            total_successful,
            total_failed,
        })
    }

    /// Get existing QR code for a property
    pub async fn get_qr_code(&self, property_id: &str) -> Result<QrCodeMetadata, QrGeneratorError> {
        self.get_existing_qr(property_id).await
    }

    /// Delete QR code for a property
    pub async fn delete_qr_code(&self, property_id: &str) -> Result<bool, QrGeneratorError> {
        // Get existing QR to get S3 key
        if let Ok(existing_qr) = self.get_existing_qr(property_id).await {
            // Delete from S3
            let s3_key = existing_qr.get_s3_key();
            if let Err(e) = self.s3_service.delete_qr_image(&s3_key).await {
                warn!("Failed to delete QR image from S3: {}", e);
            }
        }

        // Delete from database
        let result = self.qr_metadata
        .delete_one(doc! { "propertyId": property_id })
            .await?;

        Ok(result.deleted_count > 0)
    }

    /// Deactivate QR code (soft delete)
    pub async fn deactivate_qr_code(&self, property_id: &str) -> Result<bool, QrGeneratorError> {
        let update = doc! {
            "$set": {
                "isActive": false,
                "lastUpdated": utc_to_bson(Utc::now())
            }
        };

        let result = self.qr_metadata
        .update_one(doc! { "propertyId": property_id }, update)
            .await?;

        Ok(result.modified_count > 0)
    }

    /// Get all QR codes with pagination
    pub async fn get_all_qr_codes(
        &self,
        limit: Option<i64>,
        skip: Option<u64>,
    ) -> Result<Vec<QrCodeMetadata>, QrGeneratorError> {
        let options = if let Some(limit) = limit {  
    mongodb::options::FindOptions::builder()
        .limit(limit)
        .skip(skip)
        .sort(doc! { "generatedAt": -1 })
        .build()
} else {
    mongodb::options::FindOptions::builder()
        .skip(skip)
        .sort(doc! { "generatedAt": -1 })
        .build()
};

        let mut cursor = self.qr_metadata.find(doc! {}).with_options(options).await?;
        let mut qr_codes = Vec::new();

        while cursor.advance().await? {
            let qr_code = cursor.deserialize_current()?;
            qr_codes.push(qr_code);
        }

        Ok(qr_codes)
    }

    /// Get QR codes that need regeneration (expired or outdated)
    pub async fn get_qr_codes_needing_regeneration(&self, expiry_days: i64) -> Result<Vec<String>, QrGeneratorError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(expiry_days);
        
        let filter = doc! {
            "generatedAt": { "$lt": utc_to_bson(cutoff_date) },
            "isActive": true
        };

        let mut cursor = self.qr_metadata.find(filter).await?;
        let mut property_ids = Vec::new();

        while cursor.advance().await? {
            let qr_code: QrCodeMetadata = cursor.deserialize_current()?;
            property_ids.push(qr_code.property_id);
        }

        Ok(property_ids)
    }

    /// Update QR generation settings
    pub fn update_settings(&mut self, new_settings: QrGenerationSettings) {
        self.settings = new_settings;
    }

    /// Generate QR codes for all eligible properties that don't have them
    pub async fn generate_missing_qr_codes(&self) -> Result<BatchQrCodeResponse, QrGeneratorError> {
        // Get all existing QR property IDs
        let existing_qr_property_ids = self.get_all_qr_property_ids().await?;

        // Get properties that need QR codes
        let properties_needing_qr = self.property_service
            .get_properties_needing_qr(existing_qr_property_ids)
            .await
            .map_err(|e| QrGeneratorError::DatabaseError(mongodb::error::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))))?;

        let property_ids: Vec<String> = properties_needing_qr
            .into_iter()
            .map(|p| p.id.to_hex())
            .collect();

        if property_ids.is_empty() {
            return Ok(BatchQrCodeResponse {
                successful: Vec::new(),
                failed: Vec::new(),
                total_requested: 0,
                total_successful: 0,
                total_failed: 0,
            });
        }

        self.batch_generate_qr_codes(property_ids, false, QrGenerationReason::BatchGeneration).await
    }

    /// Private helper methods
    async fn get_existing_qr(&self, property_id: &str) -> Result<QrCodeMetadata, QrGeneratorError> {
        self.qr_metadata
        .find_one(doc! { "propertyId": property_id })
            .await?
            .ok_or(QrGeneratorError::PropertyNotFound)
    }

    async fn upsert_qr_metadata(&self, qr_metadata: &QrCodeMetadata) -> Result<(), QrGeneratorError> {
        let filter = doc! { "propertyId": &qr_metadata.property_id };
        let options = mongodb::options::ReplaceOptions::builder()
            .upsert(true)
            .build();

        self.qr_metadata
        .replace_one(filter, qr_metadata).with_options(options)
        .await?;

    Ok(())
}

async fn generate_qr_image(&self, _qr_data: &str) -> Result<Vec<u8>, QrGeneratorError> {
    // TODO: Implement actual QR code generation using qrcode crate
    // For now, return a placeholder
    // 
    // This would typically use:
    // use qrcode::{QrCode, Version, EcLevel};
    // use image::{ImageBuffer, Luma};
    
    // let code = QrCode::with_error_correction_level(qr_data, EcLevel::M)
    //     .map_err(|e| QrGeneratorError::QrGenerationFailed(e.to_string()))?;
    
    // let image = code.render::<Luma<u8>>()
    //     .min_dimensions(self.settings.size, self.settings.size)
    //     .build();
    
    // Convert image to PNG bytes
    // let mut png_data = Vec::new();
    // image.write_to(&mut Cursor::new(&mut png_data), image::ImageOutputFormat::Png)
    //     .map_err(|e| QrGeneratorError::QrGenerationFailed(e.to_string()))?;
    
    // For now, return empty vec as placeholder
    warn!("QR image generation not implemented - returning placeholder");
    Ok(vec![0x89, 0x50, 0x4E, 0x47]) // PNG header as placeholder
}

async fn get_all_qr_property_ids(&self) -> Result<Vec<String>, QrGeneratorError> {
    let projection = doc! { "propertyId": 1, "_id": 0 };
    let options = mongodb::options::FindOptions::builder()
        .projection(projection)
        .build();

    let mut cursor = self.qr_metadata.find(doc! {}).with_options(options).await?;
    let mut property_ids = Vec::new();

    while cursor.advance().await? {
        if let Ok(property_id) = cursor.current().get_str("propertyId") {
            property_ids.push(property_id.to_string());
        }
    }

    Ok(property_ids)
}
}

#[cfg(test)]
mod tests {
use super::*;
use mongodb::Client;
use crate::services::{PropertyService, S3Service};

async fn get_test_service() -> QrGeneratorService {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("Failed to connect to MongoDB");
    let db = client.database("test_qr_generator");
    
    let property_service = PropertyService::new(&db);
    let s3_service = S3Service::new("test-bucket".to_string(), "us-east-1".to_string())
        .expect("Failed to create S3 service");
    
    QrGeneratorService::new(
        &db,
        property_service,
        s3_service,
        "https://qr-service.daobitat.xyz".to_string(),
    )
}

#[tokio::test]
async fn test_qr_generator_creation() {
    let service = get_test_service().await;
    assert_eq!(service.base_url, "https://qr-service.daobitat.xyz");
}

#[tokio::test]
async fn test_get_all_qr_codes_empty() {
    let service = get_test_service().await;
    let qr_codes = service.get_all_qr_codes(Some(10), None).await
        .expect("Failed to get QR codes");
    
    // Should not fail, may return empty vec
    assert!(qr_codes.len() >= 0);
}

#[tokio::test]
async fn test_batch_generate_empty_list() {
    let service = get_test_service().await;
    let result = service.batch_generate_qr_codes(
        vec![], 
        false, 
        QrGenerationReason::BatchGeneration
    ).await.expect("Failed batch generation");
    
    assert_eq!(result.total_requested, 0);
    assert_eq!(result.total_successful, 0);
    assert_eq!(result.total_failed, 0);
}
}