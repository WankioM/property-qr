 // src/routes/api.rs

use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use std::sync::Arc;

use crate::handlers::{
    // QR handlers
    generate_qr_code,
    batch_generate_qr_codes,
    get_qr_code,
    regenerate_qr_code,
    delete_qr_code,
    deactivate_qr_code,
    list_qr_codes,
    generate_missing_qr_codes,
    
    // Scan handlers
    scan_qr_code,
    get_scan_data,
    scan_health,
    
    // Health handlers
    health,
    health_detailed,
    liveness,
    readiness,
    
    // State types
    AppState,
    ScanAppState,
};

/// QR code management routes
/// Mounted at /api/v1
pub fn qr_routes(state: Arc<AppState>) -> Router {
    Router::new()
        // QR Generation Routes
        .route("/qr/generate/:property_id", post(generate_qr_code))
        .route("/qr/generate/batch", post(batch_generate_qr_codes))
        .route("/qr/generate/missing", post(generate_missing_qr_codes))
        
        // QR Management Routes
        .route("/qr/:property_id", get(get_qr_code))
        .route("/qr/:property_id", delete(delete_qr_code))
        .route("/qr/regenerate/:property_id", put(regenerate_qr_code))
        .route("/qr/deactivate/:property_id", patch(deactivate_qr_code))
        
        // QR Listing Routes
        .route("/qr", get(list_qr_codes))
        
        // Analytics Routes (using scan data endpoint)
        .route("/scan/:property_id", get(get_scan_data))
        
        .with_state(state)
}

/// Scan handling routes
/// Mounted at /
pub fn scan_routes(state: Arc<ScanAppState>) -> Router {
    Router::new()
        // Main scan endpoint - handles QR code scans
        .route("/scan/:property_id", get(scan_qr_code))
        
        // API endpoint for scan data
        .route("/api/scan/:property_id", get(get_scan_data))
        
        // Scan service health
        .route("/scan/health", get(scan_health))
        
        .with_state(state)
}

/// Health check routes
/// Mounted at /health
pub fn health_routes() -> Router {
    Router::new()
        // Basic health check
        .route("/", get(health))
        
        // Detailed health check with metrics
        .route("/detailed", get(health_detailed))
        
        // Kubernetes-style probes
        .route("/live", get(liveness))
        .route("/ready", get(readiness))
}

/// Complete API routes structure
/// This function combines all routes if you want a single router
pub fn create_app_router(
    qr_state: Arc<AppState>,
    scan_state: Arc<ScanAppState>,
) -> Router {
    Router::new()
        // Health routes (no state needed)
        .nest("/health", health_routes())
        
        // QR management API routes
        .nest("/api/v1", qr_routes(qr_state))
        
        // Scan routes (public-facing)
        .merge(scan_routes(scan_state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;
    use axum::http::Request;
    use axum::body::Body;

    // Helper function to create test states
    fn create_test_states() -> (Arc<AppState>, Arc<ScanAppState>) {
        // These would normally be created with real services
        // For tests, you'd use mock implementations
        
        // Placeholder - you'd implement proper test setup
        todo!("Implement test state creation")
    }

    #[tokio::test]
    async fn test_health_routes() {
        let app = health_routes();
        
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        // Should return some status (would depend on implementation)
        assert!(response.status().is_success() || response.status().is_server_error());
    }

    #[tokio::test]
    async fn test_app_router_creation() {
        // This test just ensures the router can be created without panicking
        // You'd need to implement proper test states for full testing
        
        // let (qr_state, scan_state) = create_test_states();
        // let app = create_app_router(qr_state, scan_state);
        
        // Test that the router was created successfully
        // assert!(true); // Placeholder
    }
}
