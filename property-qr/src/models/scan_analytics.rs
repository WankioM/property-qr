 // src/models/scan_analytics.rs

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanEvent {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "propertyId")]
    pub property_id: String,
    #[serde(rename = "qrVersion")]
    pub qr_version: i32,
    #[serde(rename = "scannedAt")]
    pub scanned_at: DateTime<Utc>,
    #[serde(rename = "scanSource")]
    pub scan_source: ScanSource,
    #[serde(rename = "userAgent")]
    pub user_agent: Option<String>,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(rename = "geolocation")]
    pub geolocation: Option<GeoLocation>,
    #[serde(rename = "deviceInfo")]
    pub device_info: Option<DeviceInfo>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "referrer")]
    pub referrer: Option<String>,
    #[serde(rename = "redirectSuccess")]
    pub redirect_success: bool,
    #[serde(rename = "redirectType")]
    pub redirect_type: RedirectType,
    #[serde(rename = "responseTime")]
    pub response_time: Option<u64>, // Response time in milliseconds
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanSource {
    QrCode,         // Direct QR code scan
    DirectLink,     // Direct URL access
    ShareLink,      // Shared link
    SearchEngine,   // From search engine
    SocialMedia,    // From social media
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedirectType {
    DualRedirect,   // Both DAO-Bitat and blockchain explorer
    DaobitarOnly,   // Only DAO-Bitat property page
    BlockchainOnly, // Only blockchain explorer
    Failed,         // Redirect failed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    #[serde(rename = "deviceType")]
    pub device_type: DeviceType,
    pub platform: Option<String>,    // iOS, Android, Windows, etc.
    pub browser: Option<String>,     // Chrome, Safari, Firefox, etc.
    #[serde(rename = "browserVersion")]
    pub browser_version: Option<String>,
    #[serde(rename = "screenSize")]
    pub screen_size: Option<String>, // "1920x1080"
    #[serde(rename = "isMobile")]
    pub is_mobile: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Mobile,
    Tablet,
    Desktop,
    Unknown,
}

// Aggregated analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyScanAnalytics {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "propertyId")]
    pub property_id: String,
    #[serde(rename = "totalScans")]
    pub total_scans: i64,
    #[serde(rename = "uniqueScans")]
    pub unique_scans: i64, // Based on IP/session
    #[serde(rename = "lastScanned")]
    pub last_scanned: Option<DateTime<Utc>>,
    #[serde(rename = "firstScanned")]
    pub first_scanned: Option<DateTime<Utc>>,
    #[serde(rename = "scansToday")]
    pub scans_today: i64,
    #[serde(rename = "scansThisWeek")]
    pub scans_this_week: i64,
    #[serde(rename = "scansThisMonth")]
    pub scans_this_month: i64,
    #[serde(rename = "topCountries")]
    pub top_countries: Vec<CountryStats>,
    #[serde(rename = "deviceBreakdown")]
    pub device_breakdown: DeviceBreakdown,
    #[serde(rename = "scanTrends")]
    pub scan_trends: Vec<DailyScanCount>,
    #[serde(rename = "averageResponseTime")]
    pub average_response_time: Option<f64>,
    #[serde(rename = "successRate")]
    pub success_rate: f64, // Percentage of successful redirects
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryStats {
    pub country: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBreakdown {
    pub mobile: i64,
    pub desktop: i64,
    pub tablet: i64,
    pub unknown: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyScanCount {
    pub date: String, // YYYY-MM-DD format
    pub count: i64,
}

// System-wide analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAnalytics {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "totalProperties")]
    pub total_properties: i64,
    #[serde(rename = "propertiesWithQr")]
    pub properties_with_qr: i64,
    #[serde(rename = "totalScansAllTime")]
    pub total_scans_all_time: i64,
    #[serde(rename = "totalScansToday")]
    pub total_scans_today: i64,
    #[serde(rename = "totalScansThisWeek")]
    pub total_scans_this_week: i64,
    #[serde(rename = "totalScansThisMonth")]
    pub total_scans_this_month: i64,
    #[serde(rename = "averageScansPerProperty")]
    pub average_scans_per_property: f64,
    #[serde(rename = "topPerformingProperties")]
    pub top_performing_properties: Vec<PropertyPerformance>,
    #[serde(rename = "qrGenerationStats")]
    pub qr_generation_stats: QrGenerationStats,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyPerformance {
    #[serde(rename = "propertyId")]
    pub property_id: String,
    #[serde(rename = "propertyName")]
    pub property_name: String,
    #[serde(rename = "totalScans")]
    pub total_scans: i64,
    #[serde(rename = "uniqueScans")]
    pub unique_scans: i64,
    #[serde(rename = "successRate")]
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrGenerationStats {
    #[serde(rename = "totalGenerated")]
    pub total_generated: i64,
    #[serde(rename = "generatedToday")]
    pub generated_today: i64,
    #[serde(rename = "generatedThisWeek")]
    pub generated_this_week: i64,
    #[serde(rename = "generatedThisMonth")]
    pub generated_this_month: i64,
    #[serde(rename = "averageGenerationTime")]
    pub average_generation_time: Option<f64>, // in milliseconds
    #[serde(rename = "failureRate")]
    pub failure_rate: f64,
}

// API Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanAnalyticsResponse {
    #[serde(rename = "propertyId")]
    pub property_id: String,
    pub analytics: PropertyScanAnalytics,
    #[serde(rename = "recentScans")]
    pub recent_scans: Vec<ScanEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAnalyticsResponse {
    pub system: SystemAnalytics,
    #[serde(rename = "periodComparison")]
    pub period_comparison: Option<PeriodComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    #[serde(rename = "currentPeriod")]
    pub current_period: PeriodStats,
    #[serde(rename = "previousPeriod")]
    pub previous_period: PeriodStats,
    #[serde(rename = "percentageChange")]
    pub percentage_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodStats {
    #[serde(rename = "totalScans")]
    pub total_scans: i64,
    #[serde(rename = "uniqueScans")]
    pub unique_scans: i64,
    #[serde(rename = "averageScansPerDay")]
    pub average_scans_per_day: f64,
}

// Scan data for redirect page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRedirectData {
    #[serde(rename = "propertyId")]
    pub property_id: String,
    #[serde(rename = "propertyName")]
    pub property_name: String,
    pub location: Option<String>,
    #[serde(rename = "daobitarUrl")]
    pub daobitar_url: String,
    #[serde(rename = "blockchainUrl")]
    pub blockchain_url: Option<String>,
    pub action: String,
    pub price: i64,
    #[serde(rename = "primaryImage")]
    pub primary_image: Option<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "cryptoAccepted")]
    pub crypto_accepted: bool,
    #[serde(rename = "scanId")]
    pub scan_id: ObjectId, // For tracking this specific scan
}

impl ScanEvent {
    /// Create a new scan event
    pub fn new(
        property_id: String,
        qr_version: i32,
        scan_source: ScanSource,
        redirect_type: RedirectType,
    ) -> Self {
        Self {
            id: ObjectId::new(),
            property_id,
            qr_version,
            scanned_at: Utc::now(),
            scan_source,
            user_agent: None,
            ip_address: None,
            geolocation: None,
            device_info: None,
            session_id: None,
            referrer: None,
            redirect_success: true,
            redirect_type,
            response_time: None,
            metadata: HashMap::new(),
        }
    }

    /// Set device information
    pub fn with_device_info(mut self, device_info: DeviceInfo) -> Self {
        self.device_info = Some(device_info);
        self
    }

    /// Set geolocation
    pub fn with_geolocation(mut self, geolocation: GeoLocation) -> Self {
        self.geolocation = Some(geolocation);
        self
    }

    /// Set request metadata
    pub fn with_request_data(
        mut self,
        user_agent: Option<String>,
        ip_address: Option<String>,
        session_id: Option<String>,
        referrer: Option<String>,
    ) -> Self {
        self.user_agent = user_agent;
        self.ip_address = ip_address;
        self.session_id = session_id;
        self.referrer = referrer;
        self
    }

    /// Set response time
    pub fn with_response_time(mut self, response_time: u64) -> Self {
        self.response_time = Some(response_time);
        self
    }

    /// Mark redirect as failed
    pub fn mark_failed(mut self) -> Self {
        self.redirect_success = false;
        self.redirect_type = RedirectType::Failed;
        self
    }

    /// Add custom metadata
    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl DeviceInfo {
    /// Parse device info from user agent string
    pub fn from_user_agent(user_agent: &str) -> Self {
        // Simple user agent parsing - in production, use a proper library
        let user_agent_lower = user_agent.to_lowercase();
        
        let is_mobile = user_agent_lower.contains("mobile") 
            || user_agent_lower.contains("android")
            || user_agent_lower.contains("iphone");
            
        let is_tablet = user_agent_lower.contains("tablet") 
            || user_agent_lower.contains("ipad");

        let device_type = if is_tablet {
            DeviceType::Tablet
        } else if is_mobile {
            DeviceType::Mobile
        } else {
            DeviceType::Desktop
        };

        let platform = if user_agent_lower.contains("windows") {
            Some("Windows".to_string())
        } else if user_agent_lower.contains("mac") {
            Some("macOS".to_string())
        } else if user_agent_lower.contains("linux") {
            Some("Linux".to_string())
        } else if user_agent_lower.contains("android") {
            Some("Android".to_string())
        } else if user_agent_lower.contains("ios") || user_agent_lower.contains("iphone") {
            Some("iOS".to_string())
        } else {
            None
        };

        let browser = if user_agent_lower.contains("chrome") {
            Some("Chrome".to_string())
        } else if user_agent_lower.contains("firefox") {
            Some("Firefox".to_string())
        } else if user_agent_lower.contains("safari") && !user_agent_lower.contains("chrome") {
            Some("Safari".to_string())
        } else if user_agent_lower.contains("edge") {
            Some("Edge".to_string())
        } else {
            None
        };

        Self {
            device_type,
            platform,
            browser,
            browser_version: None, // Would need more sophisticated parsing
            screen_size: None,
            is_mobile,
        }
    }
}

impl PropertyScanAnalytics {
    /// Create new analytics entry for a property
    pub fn new(property_id: String) -> Self {
        Self {
            id: ObjectId::new(),
            property_id,
            total_scans: 0,
            unique_scans: 0,
            last_scanned: None,
            first_scanned: None,
            scans_today: 0,
            scans_this_week: 0,
            scans_this_month: 0,
            top_countries: Vec::new(),
            device_breakdown: DeviceBreakdown {
                mobile: 0,
                desktop: 0,
                tablet: 0,
                unknown: 0,
            },
            scan_trends: Vec::new(),
            average_response_time: None,
            success_rate: 100.0,
            last_updated: Utc::now(),
        }
    }

    /// Update analytics with new scan event
    pub fn update_with_scan(&mut self, scan_event: &ScanEvent) {
        self.total_scans += 1;
        self.last_scanned = Some(scan_event.scanned_at);
        
        if self.first_scanned.is_none() {
            self.first_scanned = Some(scan_event.scanned_at);
        }

        // Update device breakdown
        if let Some(device_info) = &scan_event.device_info {
            match device_info.device_type {
                DeviceType::Mobile => self.device_breakdown.mobile += 1,
                DeviceType::Desktop => self.device_breakdown.desktop += 1,
                DeviceType::Tablet => self.device_breakdown.tablet += 1,
                DeviceType::Unknown => self.device_breakdown.unknown += 1,
            }
        }

        // Update success rate
        let total_events = self.total_scans as f64;
        let successful = if scan_event.redirect_success { 1.0 } else { 0.0 };
        self.success_rate = ((self.success_rate * (total_events - 1.0)) + (successful * 100.0)) / total_events;

        self.last_updated = Utc::now();
    }
}
