# QR Code Payment Generator Microservice

A Rust Axum microservice that generates property QR codes, stores them in AWS S3, and provides scanning endpoints that redirect to both DAO-Bitat property pages and Base testnet blockchain explorers.

## Core Files Structure
qr-payment-service/
├── src/
│   ├── main.rs                    # Entry point & server setup
│   ├── config/
│   │   ├── mod.rs
│   │   ├── settings.rs            # App config (AWS, MongoDB, URLs)
│   │   └── aws.rs                 # AWS S3 client setup

***Models***

📁 models/mod.rs

Module declarations and re-exports for convenience

📁 models/property.rs

Complete Property struct matching your TypeScript MongoDB schema
All fields mapped with proper serde annotations (camelCase ↔ snake_case conversion)
PropertyQrInfo - simplified struct for QR generation
Helper methods like is_qr_eligible(), to_qr_info(), has_blockchain_info()
All nested structs: Coordinates, Amenities, BlockchainInfo, Document, etc.
Proper enum handling for PropertyType, ListingState, DocumentType, etc.

📁 models/qr_code.rs

QrCodeMetadata - main struct for storing QR codes in MongoDB
QrCodeData - the actual data structure encoded into QR codes
Request/Response DTOs for API endpoints
S3 configuration and generation settings structs
Methods for QR lifecycle: new(), record_scan(), regenerate(), is_expired()
Batch processing support with error handling

📁 models/scan_analytics.rs

ScanEvent - individual scan tracking with geolocation, device info, etc.
PropertyScanAnalytics - aggregated analytics per property
SystemAnalytics - system-wide metrics
Device detection from user agent strings
Geographic and temporal analytics support
Performance tracking (response times, success rates)

***Services***

📁 services/mod.rs

Module declarations and re-exports for all services

📁 services/property_service.rs
Core Property Management Service:

✅ Property retrieval by ID with validation
✅ QR eligibility checking with detailed reasons
✅ Batch property operations for multiple IDs
✅ Property statistics and analytics
✅ Search functionality with filters (location, price, type, verification)
✅ Owner-based queries and recent properties
✅ Click tracking for analytics integration
✅ Comprehensive error handling with custom error types

📁 services/qr_generator.rs
QR Code Generation Engine:

✅ Single QR generation with force regeneration support
✅ Batch QR generation for multiple properties
✅ QR code lifecycle management (create, update, deactivate, delete)
✅ Expiration handling and regeneration detection
✅ Comprehensive metadata storage with property info
✅ Integration ready for actual QR libraries (qrcode crate)
✅ S3 upload integration for QR images
✅ Error handling with detailed error types and codes

📁 services/s3_service.rs
AWS S3 Storage Management:

✅ QR image upload/download with proper content types
✅ Metadata file management for JSON data
✅ CloudFront integration for CDN support
✅ Presigned URL generation for direct uploads
✅ File operations (exists, delete, metadata retrieval)
✅ Batch operations and cleanup utilities
✅ Bucket statistics and monitoring
✅ Key validation and URL construction
✅ Ready for aws-sdk-s3 integration (placeholder implementations)

📁 services/analytics_service.rs   # Scan tracking service

Usage example:


```
// Record a scan
let scan_id = analytics_service.record_scan(
    property_id,
    qr_version,
    ScanSource::QrCode,
    RedirectType::DualRedirect,
    Some(user_agent),
    Some(ip_address),
    Some(session_id),
    referrer,
).await?;

// Get property analytics
let analytics = analytics_service
    .get_property_analytics(&property_id, true)
    .await?;
```

***Handlers***

📁 handlers/mod.rs

Module declarations and re-exports
Clean organization of handler modules

📁 handlers/health.rs

Basic health check: /health - Simple service status
Detailed health check: /health/detailed - Full system metrics
Liveness probe: /liveness - Kubernetes-style "I'm alive" check
Readiness probe: /readiness - "Ready to handle requests" check
Includes system info, memory usage, and service dependency checks

📁 handlers/qr_handler.rs

Generate QR: POST /generate/{property_id} - Single QR generation
Batch generate: POST /generate/batch - Multiple QR generation
Get QR: GET /qr/{property_id} - Retrieve existing QR
Regenerate: PUT /regenerate/{property_id} - Force regeneration
Delete QR: DELETE /qr/{property_id} - Hard delete
Deactivate: PATCH /deactivate/{property_id} - Soft delete
List QRs: GET /qr - Paginated QR list
Generate missing: POST /generate/missing - Auto-generate for properties without QR

📁 handlers/scan_handler.rs

QR Scan: GET /scan/{property_id} - Handle QR code scans with smart redirect
Scan API: GET /api/scan/{property_id} - JSON response for scan data
Dual redirect page - Beautiful HTML page for properties with blockchain presence
Error pages - User-friendly error handling
Analytics tracking - Records scan events for analytics
Auto-redirect - 10-second timer to property page

│   ├── routes/
│   │   ├── mod.rs
│   │   └── api.rs                 # Route definitions
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── validation.rs          # Input validation
│   │   └── url_builder.rs         # URL construction helpers
│   └── errors/
│       ├── mod.rs
│       └── app_error.rs           # Custom error types
├── Dockerfile
├── docker-compose.yml
└── deploy/
    ├── cloudformation/                 # Infrastructure as Code
    │   ├── main.tf
    │   ├── s3.tf
    │   └── ecs.tf
    └── github-actions/
        └── deploy.yml             # CI/CD pipeline
        buildspec.yml
2. Key Data Structures
Property Struct (MongoDB mapping)
rust#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub owner: ObjectId,
    pub property_name: String,
    pub location: String,
    pub onchain_id: Option<String>,
    pub price: i32,
    pub action: String, // "for sale" | "for rent"
    pub crypto_accepted: bool,
    pub created_at: DateTime<Utc>,
    // ... other fields as needed
}
QR Code Metadata
rust#[derive(Debug, Serialize, Deserialize)]
pub struct QrCodeMetadata {
    pub id: ObjectId,
    pub property_id: String,
    pub qr_code_url: String,        // S3 URL
    pub qr_pattern: String,         // The actual QR data
    pub generated_at: DateTime<Utc>,
    pub scan_count: i32,
    pub last_scanned: Option<DateTime<Utc>>,
}
QR Scan Data
rust#[derive(Debug, Serialize, Deserialize)]
pub struct QrScanData {
    pub property_id: String,
    pub daobitat_url: String,
    pub blockchain_url: String,
    pub property_name: String,
    pub price: i32,
    pub action: String,
}
3. Core API Endpoints
QR Generation Endpoints
POST /api/v1/qr/generate/{property_id}     # Generate QR for single property
POST /api/v1/qr/batch-generate             # Generate QRs for multiple properties
GET  /api/v1/qr/{property_id}              # Get existing QR code
QR Scan Endpoints
GET /scan/{property_id}                    # Redirect endpoint (returns HTML with dual redirect)
GET /api/v1/scan/{property_id}             # API endpoint returning scan data
GET /api/v1/qr/{property_id}/analytics     # Get scan analytics
Management Endpoints
GET  /api/v1/qr/list                       # List all QR codes
DELETE /api/v1/qr/{property_id}            # Delete QR code
PUT  /api/v1/qr/{property_id}/regenerate   # Regenerate QR code
4. QR Code Implementation Strategy
QR Data Format
json{
  "type": "daobitat_property",
  "property_id": "67ec31596333aba221e84df8",
  "scan_url": "https://qr-service.daobitat.xyz/scan/67ec31596333aba221e84df8"
}
Dual Redirect Strategy
When QR is scanned → hits /scan/{property_id} → returns HTML page that:

Immediately redirects to DAO-Bitat property page
Opens second tab/window to Base testnet explorer
Tracks the scan event

## 5. AWS Infrastructure Requirements (Serverless Architecture)

### S3 Bucket Structure
```
daobitat-qr-codes/
├── qr-images/
│   ├── 67ec31596333aba221e84df8.png
│   └── ...
└── metadata/
    ├── 67ec31596333aba221e84df8.json
    └── ...
```

### Required AWS Services (Cost-Optimized)

**Core Services:**
- **AWS Lambda**: Rust runtime for QR generation logic
- **API Gateway**: REST API endpoints (`/generate`, `/scan/{id}`)
- **S3**: QR code image storage with public read access
- **CloudFront**: CDN for QR images (optional for cost savings)

**Supporting Services:**
- **IAM**: Lambda execution roles and S3 permissions
- **Route 53**: DNS management (if custom domain needed)
- **CloudWatch**: Logging and monitoring



## 6. CI/CD Deployment Strategy (AWS CodePipeline)

### CodePipeline Architecture
```
GitHub → CodeBuild → Lambda Deployment
    ↓
CloudFormation (Infrastructure)
    ↓
S3 + API Gateway + Lambda
```

### buildspec.yml (CodeBuild Configuration)
```yaml
version: 0.2
phases:
  install:
    runtime-versions:
      python: 3.9
    commands:
      - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
      - source ~/.cargo/env
      - rustup target add x86_64-unknown-linux-musl
      
  build:
    commands:
      - cargo build --release --target x86_64-unknown-linux-musl
      - cp target/x86_64-unknown-linux-musl/release/property-qr bootstrap
      - zip lambda-deployment.zip bootstrap
      
artifacts:
  files:
    - lambda-deployment.zip
    - deploy/cloudformation/*.yml
```

### CloudFormation Infrastructure

**Core Stack (`infrastructure.yml`):**
- **Lambda Function**: Rust binary deployment
- **API Gateway**: REST API with `/generate` and `/scan/{id}` endpoints
- **S3 Bucket**: QR storage with public read policy
- **IAM Roles**: Lambda execution with S3 write permissions
- **CloudWatch Logs**: Function logging

**Pipeline Stack (`pipeline.yml`):**
- **CodePipeline**: GitHub → CodeBuild → CloudFormation
- **CodeBuild Project**: Rust compilation and packaging
- **S3 Artifact Store**: Pipeline artifacts storage

### Deployment Flow
1. **Source Stage**: GitHub webhook triggers pipeline
2. **Build Stage**: CodeBuild compiles Rust → Lambda zip
3. **Deploy Stage**: CloudFormation updates Lambda function
4. **Test Stage**: Automated health checks via API Gateway



### Environment Configuration
```yaml
# Parameters for different environments
Development:
  - Lambda Memory: 128MB 
  - S3 Storage Class: Standard
  - API Gateway: Regional

Production:
  - Lambda Memory: 256MB 
  - S3 Storage Class: Intelligent Tiering
  - API Gateway: Edge Optimized + CloudFront
```



### Monitoring & Alerts
- **CloudWatch Dashboards**: Lambda performance metrics
- **Cost Alerts**: Billing notifications if over budget
- **Error Tracking**: Failed QR generations and scan redirects

8. Integration Points
MongoDB Integration

Read-only access to main DAO-Bitat property collection
Separate collection for QR metadata and analytics
Connection pooling for performance

URL Construction
rust// DAO-Bitat URL
let daobitat_url = format!("https://www.daobitat.xyz/property-details/{}", property_id);

// Base testnet explorer URL (using onchain_id)
let blockchain_url = format!("https://basescan.org/address/{}", onchain_id);
QR Code Content

Encode the scan URL: https://qr-service.daobitat.xyz/scan/{property_id}
256x256 PNG format
Error correction level: Medium
Include DAO-Bitat branding/logo overlay

9. Monitoring & Analytics
Metrics to Track

QR generation success/failure rates
Scan frequency per property
Geographic scan distribution
Redirect success rates
S3 upload/download performance

Health Checks

MongoDB connectivity
S3 bucket access
QR generation capability
Scan endpoint responsiveness

This architecture provides a scalable, maintainable solution for QR code generation and property scanning with proper separation of concerns and cloud-native deployment.