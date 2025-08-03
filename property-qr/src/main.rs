// src/main.rs

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tracing::{info, error};
use std::error::Error;

// Import your modules
mod config;
mod models;
mod services;
mod utils;
mod handlers;
mod errors;
mod routes;

// Import configuration and services
use config::Settings;
use services::{AnalyticsService, PropertyService, QrGeneratorService, S3Service};
use handlers::{AppState, ScanAppState};
use routes::{qr_routes, scan_routes, health_routes};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting DAO-Bitat QR Service...");
    
    // Load configuration
    let settings = Settings::from_env()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;
    
    // Validate configuration
    settings.validate()
        .map_err(|e| format!("Invalid configuration: {}", e))?;
    
    info!("Configuration loaded successfully");
    info!("Environment: {:?}", settings.server.environment);
    info!("Server will listen on {}:{}", settings.server.host, settings.server.port);
    
    // Connect to MongoDB
    let client = mongodb::Client::with_uri_str(&settings.database.mongodb_uri).await
        .map_err(|e| format!("Failed to connect to MongoDB: {}", e))?;
    
    let database = client.database(&settings.database.database_name);
    
    // Test MongoDB connection
    database.run_command(mongodb::bson::doc! {"ping": 1}, None).await
        .map_err(|e| format!("MongoDB connection test failed: {}", e))?;
    
    info!("MongoDB connection verified");
    
    // Initialize services
    let property_service = PropertyService::new(&database);
    let s3_service = S3Service::new(
        settings.aws.s3_bucket.clone(),
        settings.aws.region.clone(),
    ).map_err(|e| format!("Failed to create S3 service: {}", e))?;
    
    let analytics_service = AnalyticsService::new(&database);
    let qr_generator_service = QrGeneratorService::new(
        &database,
        property_service.clone(),
        s3_service.clone(),
        settings.urls.base_url.clone(),
    );
    
    info!("Services initialized successfully");
    
    // Create application states
    let app_state = Arc::new(AppState {
        qr_generator: qr_generator_service,
    });
    
    let scan_state = Arc::new(ScanAppState {
        qr_generator: app_state.qr_generator.clone(),
        property_service,
        analytics_service,
        daobitar_base_url: settings.urls.daobitat_base_url.clone(),
        blockchain_explorer_base_url: settings.urls.blockchain_explorer_base_url.clone(),
    });
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(
            settings.server.cors_origins
                .iter()
                .map(|origin| origin.parse::<HeaderValue>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Invalid CORS origin: {}", e))?,
        )
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH]);
    
    // Build the application router
    let app = Router::new()
        // Health routes
        .nest("/health", health_routes())
        
        // QR management API routes
        .nest("/api/v1", qr_routes(app_state))
        
        // Scan routes (public-facing)
        .merge(scan_routes(scan_state))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(settings.server.request_timeout_seconds)))
                .layer(cors)
        );
    
    // Create server address
    let addr = SocketAddr::new(
        settings.server.host.parse()
            .map_err(|e| format!("Invalid host address: {}", e))?,
        settings.server.port,
    );
    
    info!("Server starting on {}", addr);
    
    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| format!("Failed to bind to address {}: {}", addr, e))?;
    
    info!("✅ DAO-Bitat QR Service is running on http://{}", addr);
    info!("🔍 Health check: http://{}/health", addr);
    info!("📱 QR API: http://{}/api/v1/qr", addr);
    info!("🔗 Scan endpoint: http://{}/scan/{{property_id}}", addr);
    
    // Start the server
    axum::serve(listener, app).await
        .map_err(|e| format!("Server error: {}", e))?;
    
    Ok(())
}