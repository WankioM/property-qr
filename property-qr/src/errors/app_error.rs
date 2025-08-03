 // src/errors/app_error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::error;

/// Result type alias for the application
pub type AppResult<T> = Result<T, AppError>;

/// Main application error type
#[derive(Debug, Clone)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub context: Option<ErrorContext>,
    pub source: Option<String>,
}

/// Error codes for different types of errors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // Validation errors
    InvalidInput,
    InvalidPropertyId,
    InvalidQrCode,
    InvalidFileFormat,
    InvalidUrl,
    
    // Database errors
    DatabaseConnection,
    DatabaseOperation,
    DocumentNotFound,
    DuplicateDocument,
    
    // Service errors
    PropertyNotFound,
    PropertyNotEligible,
    QrGenerationFailed,
    QrNotFound,
    
    // External service errors
    S3UploadFailed,
    S3DownloadFailed,
    S3BucketNotFound,
    AnalyticsServiceError,
    
    // Authentication/Authorization
    Unauthorized,
    Forbidden,
    InvalidApiKey,
    
    // Rate limiting
    RateLimitExceeded,
    TooManyRequests,
    
    // System errors
    InternalServerError,
    ServiceUnavailable,
    ConfigurationError,
    TimeoutError,
    
    // Network errors
    NetworkError,
    ExternalServiceError,
    
    // Business logic errors
    QrAlreadyExists,
    QrExpired,
    PropertyAlreadyHasQr,
    BatchOperationFailed,
    
    // File/Image errors
    ImageProcessingFailed,
    UnsupportedImageFormat,
    FileSizeExceeded,
    FileUploadFailed,
}

/// Additional error context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub property_id: Option<String>,
    pub qr_id: Option<String>,
    pub file_name: Option<String>,
    pub operation: Option<String>,
    pub details: Option<serde_json::Value>,
}

/// Error response structure for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorCode,
    pub message: String,
    pub timestamp: String,
    pub request_id: Option<String>,
    pub context: Option<ErrorContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl AppError {
    /// Create a new application error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: None,
            source: None,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Add source information to the error
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::new(ErrorCode::DocumentNotFound, format!("{} not found", resource.into()))
    }

    /// Create a property not found error
    pub fn property_not_found(property_id: impl Into<String>) -> Self {
        let property_id = property_id.into();
        Self::new(ErrorCode::PropertyNotFound, "Property not found")
            .with_context(ErrorContext {
                property_id: Some(property_id),
                qr_id: None,
                file_name: None,
                operation: Some("property_lookup".to_string()),
                details: None,
            })
    }

    /// Create a QR not found error
    pub fn qr_not_found(property_id: impl Into<String>) -> Self {
        let property_id = property_id.into();
        Self::new(ErrorCode::QrNotFound, "QR code not found for property")
            .with_context(ErrorContext {
                property_id: Some(property_id),
                qr_id: None,
                file_name: None,
                operation: Some("qr_lookup".to_string()),
                details: None,
            })
    }

    /// Create a database error
    pub fn database_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::DatabaseOperation, message)
    }

    /// Create an S3 error
    pub fn s3_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(ErrorCode::S3UploadFailed, message)
            .with_context(ErrorContext {
                property_id: None,
                qr_id: None,
                file_name: None,
                operation: Some(operation.into()),
                details: None,
            })
    }

    /// Create an internal server error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalServerError, message)
    }

    /// Create a rate limit error
    pub fn rate_limit_exceeded() -> Self {
        Self::new(ErrorCode::RateLimitExceeded, "Rate limit exceeded")
    }

    /// Create a timeout error
    pub fn timeout_error(operation: impl Into<String>) -> Self {
        Self::new(ErrorCode::TimeoutError, format!("Operation timed out: {}", operation.into()))
    }

    /// Create a QR generation error
    pub fn qr_generation_failed(property_id: impl Into<String>, reason: impl Into<String>) -> Self {
        let property_id = property_id.into();
        Self::new(ErrorCode::QrGenerationFailed, format!("QR generation failed: {}", reason.into()))
            .with_context(ErrorContext {
                property_id: Some(property_id),
                qr_id: None,
                file_name: None,
                operation: Some("qr_generation".to_string()),
                details: None,
            })
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self.code {
            ErrorCode::InvalidInput
            | ErrorCode::InvalidPropertyId
            | ErrorCode::InvalidQrCode
            | ErrorCode::InvalidFileFormat
            | ErrorCode::InvalidUrl => StatusCode::BAD_REQUEST,

            ErrorCode::Unauthorized | ErrorCode::InvalidApiKey => StatusCode::UNAUTHORIZED,

            ErrorCode::Forbidden => StatusCode::FORBIDDEN,

            ErrorCode::DocumentNotFound
            | ErrorCode::PropertyNotFound
            | ErrorCode::QrNotFound
            | ErrorCode::S3BucketNotFound => StatusCode::NOT_FOUND,

            ErrorCode::DuplicateDocument
            | ErrorCode::QrAlreadyExists
            | ErrorCode::PropertyAlreadyHasQr => StatusCode::CONFLICT,

            ErrorCode::RateLimitExceeded | ErrorCode::TooManyRequests => {
                StatusCode::TOO_MANY_REQUESTS
            }

            ErrorCode::FileSizeExceeded => StatusCode::PAYLOAD_TOO_LARGE,

            ErrorCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,

            ErrorCode::TimeoutError => StatusCode::REQUEST_TIMEOUT,

            ErrorCode::PropertyNotEligible => StatusCode::UNPROCESSABLE_ENTITY,

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        self.status_code().is_client_error()
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        self.status_code().is_server_error()
    }

    /// Convert to error response
    pub fn to_response(&self, request_id: Option<String>) -> ErrorResponse {
        ErrorResponse {
            error: self.code.clone(),
            message: self.message.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id,
            context: self.context.clone(),
            details: None,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)?;
        
        if let Some(context) = &self.context {
            if let Some(property_id) = &context.property_id {
                write!(f, " (property: {})", property_id)?;
            }
            if let Some(operation) = &context.operation {
                write!(f, " (operation: {})", operation)?;
            }
        }
        
        if let Some(source) = &self.source {
            write!(f, " (source: {})", source)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for AppError {}

// Convert from various error types to AppError
impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        error!("MongoDB error: {}", err);
        
        match err.kind.as_ref() {
            mongodb::error::ErrorKind::Io(_) => {
                AppError::new(ErrorCode::DatabaseConnection, "Database connection error")
            }
            mongodb::error::ErrorKind::Authentication { .. } => {
                AppError::new(ErrorCode::Unauthorized, "Database authentication failed")
            }
            _ => AppError::new(ErrorCode::DatabaseOperation, format!("Database error: {}", err)),
        }.with_source(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        error!("JSON serialization error: {}", err);
        AppError::new(ErrorCode::InvalidInput, "Invalid JSON format")
            .with_source(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        error!("IO error: {}", err);
        
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                AppError::new(ErrorCode::DocumentNotFound, "File or resource not found")
            }
            std::io::ErrorKind::PermissionDenied => {
                AppError::new(ErrorCode::Forbidden, "Permission denied")
            }
            std::io::ErrorKind::TimedOut => {
                AppError::new(ErrorCode::TimeoutError, "Operation timed out")
            }
            _ => AppError::new(ErrorCode::InternalServerError, format!("IO error: {}", err)),
        }.with_source(err.to_string())
    }
}

impl From<mongodb::bson::oid::Error> for AppError {
    fn from(err: mongodb::bson::oid::Error) -> Self {
        error!("ObjectId error: {}", err);
        AppError::new(ErrorCode::InvalidPropertyId, "Invalid property ID format")
            .with_source(err.to_string())
    }
}

// Implement IntoResponse for Axum
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        
        // Log server errors
        if self.is_server_error() {
            error!("Server error: {}", self);
        }
        
        let error_response = self.to_response(None);
        
        (status, Json(error_response)).into_response()
    }
}

impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self {
            property_id: None,
            qr_id: None,
            file_name: None,
            operation: None,
            details: None,
        }
    }

    /// Add property ID to context
    pub fn with_property_id(mut self, property_id: impl Into<String>) -> Self {
        self.property_id = Some(property_id.into());
        self
    }

    /// Add QR ID to context
    pub fn with_qr_id(mut self, qr_id: impl Into<String>) -> Self {
        self.qr_id = Some(qr_id.into());
        self
    }

    /// Add file name to context
    pub fn with_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    /// Add operation name to context
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Add additional details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macros for creating errors
#[macro_export]
macro_rules! app_error {
    ($code:expr, $msg:expr) => {
        $crate::errors::AppError::new($code, $msg)
    };
    ($code:expr, $msg:expr, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut context = $crate::errors::ErrorContext::new();
            $(
                context = context.$key($value);
            )+
            $crate::errors::AppError::new($code, $msg).with_context(context)
        }
    };
}

/// Helper macro for returning early with an error
#[macro_export]
macro_rules! bail {
    ($code:expr, $msg:expr) => {
        return Err(app_error!($code, $msg))
    };
    ($code:expr, $msg:expr, $($key:ident = $value:expr),+ $(,)?) => {
        return Err(app_error!($code, $msg, $($key = $value),+))
    };
}

/// Helper macro for ensuring a condition or returning an error
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $code:expr, $msg:expr) => {
        if !($cond) {
            bail!($code, $msg);
        }
    };
    ($cond:expr, $code:expr, $msg:expr, $($key:ident = $value:expr),+ $(,)?) => {
        if !($cond) {
            bail!($code, $msg, $($key = $value),+);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AppError::new(ErrorCode::PropertyNotFound, "Test property not found");
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
        assert!(error.is_client_error());
        assert!(!error.is_server_error());
    }

    #[test]
    fn test_error_with_context() {
        let error = AppError::property_not_found("test123");
        assert!(error.context.is_some());
        assert_eq!(error.context.unwrap().property_id, Some("test123".to_string()));
    }

    #[test]
    fn test_error_response_serialization() {
        let error = AppError::new(ErrorCode::QrGenerationFailed, "Test error");
        let response = error.to_response(Some("req123".to_string()));
        
        assert_eq!(response.request_id, Some("req123".to_string()));
        assert!(!response.timestamp.is_empty());
    }

    #[test]
    fn test_app_error_macro() {
        let error = app_error!(ErrorCode::InvalidInput, "Invalid data");
        assert_eq!(error.message, "Invalid data");
        
        let error_with_context = app_error!(
            ErrorCode::PropertyNotFound, 
            "Property missing",
            with_property_id = "test123",
            with_operation = "lookup"
        );
        
        let context = error_with_context.context.unwrap();
        assert_eq!(context.property_id, Some("test123".to_string()));
        assert_eq!(context.operation, Some("lookup".to_string()));
    }
}
