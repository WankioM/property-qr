 
// src/handlers/qr_handler.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error};

use crate::models::{
    GenerateQrRequest, BatchGenerateQrRequest, QrCodeResponse, BatchQrCodeResponse,
    QrGenerationReason, QrStatus, QrCodeMetadata
};
use crate::services::QrGeneratorService;

// Application state that will be passed to handlers
#[derive(Clone)]
pub struct AppState {
    pub qr_generator: QrGeneratorService,
}

// Query parameters for pagination and filtering
#[derive(Debug, Deserialize)]
pub struct QrListQuery {
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub property_id: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RegenerateQuery {
    pub reason: Option<QrGenerationReason>,
}

// Error response structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: String,
    pub path: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            path: None,
        }
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }
}

// Success response wrapper
#[derive(Debug, Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
    pub timestamp: String,
}

impl<T> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Generate QR code for a single property
/// POST /generate/{property_id}
pub async fn generate_qr_code(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<String>,
    Json(request): Json<GenerateQrRequest>,
) -> Result<ResponseJson<SuccessResponse<QrCodeResponse>>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("Generating QR code for property: {}", property_id);

    // Validate that the property_id in the path matches the request
    if request.property_id != property_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "validation_error",
                "Property ID in path does not match request body"
            ))
        ));
    }

    let force_regenerate = request.force_regenerate.unwrap_or(false);
    let reason = request.reason.unwrap_or(QrGenerationReason::NewProperty);

    match state.qr_generator.generate_qr_code(property_id.clone(), force_regenerate, reason).await {
        Ok(qr_response) => {
            info!("Successfully generated QR code for property: {}", property_id);
            Ok(Json(SuccessResponse::new(qr_response)))
        }
        Err(e) => {
            error!("Failed to generate QR code for property {}: {}", property_id, e);
            let (status_code, error_type) = match e {
                crate::services::qr_generator::QrGeneratorError::PropertyNotFound => {
                    (StatusCode::NOT_FOUND, "property_not_found")
                }
                crate::services::qr_generator::QrGeneratorError::PropertyNotEligible(_) => {
                    (StatusCode::BAD_REQUEST, "property_not_eligible")
                }
                crate::services::qr_generator::QrGeneratorError::InvalidPropertyId => {
                    (StatusCode::BAD_REQUEST, "invalid_property_id")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "generation_failed")
            };

            Err((
                status_code,
                Json(ErrorResponse::new(error_type, &e.to_string()))
            ))
        }
    }
}

/// Generate QR codes for multiple properties
/// POST /generate/batch
pub async fn batch_generate_qr_codes(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BatchGenerateQrRequest>,
) -> Result<ResponseJson<SuccessResponse<BatchQrCodeResponse>>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("Batch generating QR codes for {} properties", request.property_ids.len());

    if request.property_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "validation_error",
                "Property IDs list cannot be empty"
            ))
        ));
    }

    if request.property_ids.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "validation_error",
                "Cannot process more than 100 properties at once"
            ))
        ));
    }

    let force_regenerate = request.force_regenerate.unwrap_or(false);
    let reason = request.reason.unwrap_or(QrGenerationReason::BatchGeneration);

    match state.qr_generator.batch_generate_qr_codes(request.property_ids, force_regenerate, reason).await {
        Ok(batch_response) => {
            info!(
                "Batch QR generation completed: {} successful, {} failed",
                batch_response.total_successful,
                batch_response.total_failed
            );
            Ok(Json(SuccessResponse::new(batch_response)))
        }
        Err(e) => {
            error!("Batch QR generation failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("batch_generation_failed", &e.to_string()))
            ))
        }
    }
}

/// Get existing QR code for a property
/// GET /qr/{property_id}
pub async fn get_qr_code(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<String>,
) -> Result<ResponseJson<SuccessResponse<QrCodeMetadata>>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("Getting QR code for property: {}", property_id);

    match state.qr_generator.get_qr_code(&property_id).await {
        Ok(qr_metadata) => {
            Ok(Json(SuccessResponse::new(qr_metadata)))
        }
        Err(e) => {
            warn!("QR code not found for property {}: {}", property_id, e);
            let (status_code, error_type) = match e {
                crate::services::qr_generator::QrGeneratorError::PropertyNotFound => {
                    (StatusCode::NOT_FOUND, "qr_not_found")
                }
                crate::services::qr_generator::QrGeneratorError::InvalidPropertyId => {
                    (StatusCode::BAD_REQUEST, "invalid_property_id")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "retrieval_failed")
            };

            Err((
                status_code,
                Json(ErrorResponse::new(error_type, &e.to_string()))
            ))
        }
    }
}

/// Regenerate QR code for a property
/// PUT /regenerate/{property_id}
pub async fn regenerate_qr_code(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<String>,
    Query(query): Query<RegenerateQuery>,
) -> Result<ResponseJson<SuccessResponse<QrCodeResponse>>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("Regenerating QR code for property: {}", property_id);

    let reason = query.reason.unwrap_or(QrGenerationReason::ManualRegeneration);

    match state.qr_generator.generate_qr_code(property_id.clone(), true, reason).await {
        Ok(qr_response) => {
            info!("Successfully regenerated QR code for property: {}", property_id);
            Ok(Json(SuccessResponse::new(qr_response)))
        }
        Err(e) => {
            error!("Failed to regenerate QR code for property {}: {}", property_id, e);
            let (status_code, error_type) = match e {
                crate::services::qr_generator::QrGeneratorError::PropertyNotFound => {
                    (StatusCode::NOT_FOUND, "property_not_found")
                }
                crate::services::qr_generator::QrGeneratorError::PropertyNotEligible(_) => {
                    (StatusCode::BAD_REQUEST, "property_not_eligible")
                }
                crate::services::qr_generator::QrGeneratorError::InvalidPropertyId => {
                    (StatusCode::BAD_REQUEST, "invalid_property_id")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "regeneration_failed")
            };

            Err((
                status_code,
                Json(ErrorResponse::new(error_type, &e.to_string()))
            ))
        }
    }
}

/// Delete QR code for a property
/// DELETE /qr/{property_id}
pub async fn delete_qr_code(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<String>,
) -> Result<ResponseJson<SuccessResponse<serde_json::Value>>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("Deleting QR code for property: {}", property_id);

    match state.qr_generator.delete_qr_code(&property_id).await {
        Ok(deleted) => {
            if deleted {
                info!("Successfully deleted QR code for property: {}", property_id);
                Ok(Json(SuccessResponse::new(serde_json::json!({
                    "deleted": true,
                    "property_id": property_id
                }))))
            } else {
                warn!("QR code not found for deletion: {}", property_id);
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse::new("qr_not_found", "QR code not found for deletion"))
                ))
            }
        }
        Err(e) => {
            error!("Failed to delete QR code for property {}: {}", property_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("deletion_failed", &e.to_string()))
            ))
        }
    }
}

/// Deactivate QR code for a property (soft delete)
/// PATCH /deactivate/{property_id}
pub async fn deactivate_qr_code(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<String>,
) -> Result<ResponseJson<SuccessResponse<serde_json::Value>>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("Deactivating QR code for property: {}", property_id);

    match state.qr_generator.deactivate_qr_code(&property_id).await {
        Ok(deactivated) => {
            if deactivated {
                info!("Successfully deactivated QR code for property: {}", property_id);
                Ok(Json(SuccessResponse::new(serde_json::json!({
                    "deactivated": true,
                    "property_id": property_id
                }))))
            } else {
                warn!("QR code not found for deactivation: {}", property_id);
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse::new("qr_not_found", "QR code not found for deactivation"))
                ))
            }
        }
        Err(e) => {
            error!("Failed to deactivate QR code for property {}: {}", property_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("deactivation_failed", &e.to_string()))
            ))
        }
    }
}

/// List all QR codes with pagination
/// GET /qr
pub async fn list_qr_codes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<QrListQuery>,
) -> Result<ResponseJson<SuccessResponse<Vec<QrCodeMetadata>>>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("Listing QR codes with query: {:?}", query);

    let limit = query.limit.unwrap_or(50).min(100); // Cap at 100
    let skip = query.skip.unwrap_or(0);

    match state.qr_generator.get_all_qr_codes(Some(limit), Some(skip)).await {
        Ok(qr_codes) => {
            let filtered_codes = if let Some(property_id) = query.property_id {
                qr_codes.into_iter()
                    .filter(|qr| qr.property_id == property_id)
                    .collect()
            } else if query.active_only.unwrap_or(false) {
                qr_codes.into_iter()
                    .filter(|qr| qr.is_active)
                    .collect()
            } else {
                qr_codes
            };

            info!("Retrieved {} QR codes", filtered_codes.len());
            Ok(Json(SuccessResponse::new(filtered_codes)))
        }
        Err(e) => {
            error!("Failed to list QR codes: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("list_failed", &e.to_string()))
            ))
        }
    }
}

/// Generate QR codes for all properties that don't have them
/// POST /generate/missing
pub async fn generate_missing_qr_codes(
    State(state): State<Arc<AppState>>,
) -> Result<ResponseJson<SuccessResponse<BatchQrCodeResponse>>, (StatusCode, ResponseJson<ErrorResponse>)> {
    info!("Generating QR codes for properties that don't have them");

    match state.qr_generator.generate_missing_qr_codes().await {
        Ok(batch_response) => {
            info!(
                "Missing QR generation completed: {} successful, {} failed",
                batch_response.total_successful,
                batch_response.total_failed
            );
            Ok(Json(SuccessResponse::new(batch_response)))
        }
        Err(e) => {
            error!("Failed to generate missing QR codes: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("missing_generation_failed", &e.to_string()))
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::new("test_error", "Test message");
        assert_eq!(error.error, "test_error");
        assert_eq!(error.message, "Test message");
        assert!(error.path.is_none());
    }

    #[test]
    fn test_error_response_with_path() {
        let error = ErrorResponse::new("test_error", "Test message")
            .with_path("/test/path");
        assert_eq!(error.path, Some("/test/path".to_string()));
    }

    #[test]
    fn test_success_response_creation() {
        let data = "test data";
        let response = SuccessResponse::new(data);
        assert_eq!(response.success, true);
        assert_eq!(response.data, "test data");
    }
}