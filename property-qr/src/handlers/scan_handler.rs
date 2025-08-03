 
// src/handlers/scan_handler.rs

use axum::{
    extract::{Path, Query, State, ConnectInfo},
    http::{StatusCode, HeaderMap},
    response::{Html, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, net::SocketAddr};
use tracing::{info, warn, error};
use axum::response::IntoResponse;

use crate::models::{
    ScanSource, RedirectType, ScanRedirectData, PropertyQrInfo
};
use crate::services::{QrGeneratorService, PropertyService, AnalyticsService};

// Application state for scan handlers
#[derive(Clone)]
pub struct ScanAppState {
    pub qr_generator: QrGeneratorService,
    pub property_service: PropertyService,
    pub analytics_service: AnalyticsService,
    pub daobitar_base_url: String,
    pub blockchain_explorer_base_url: String,
}

// Query parameters for scan redirects
#[derive(Debug, Deserialize)]
pub struct ScanQuery {
    pub source: Option<String>,        // "qr", "direct", "share", etc.
    pub redirect: Option<String>,      // "dual", "property", "blockchain"
    pub ref_: Option<String>,          // Referrer information
    pub utm_source: Option<String>,    // UTM tracking
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub success: bool,
    pub property_id: String,
    pub redirect_type: String,
    pub urls: RedirectUrls,
    pub scan_id: String,
}

#[derive(Debug, Serialize)]
pub struct RedirectUrls {
    pub property_url: String,
    pub blockchain_url: Option<String>,
    pub redirect_page_url: String,
}

/// Handle QR code scan with property ID
/// GET /scan/{property_id}
pub async fn scan_qr_code(
    State(state): State<Arc<ScanAppState>>,
    Path(property_id): Path<String>,
    Query(query): Query<ScanQuery>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Response, StatusCode> {
    info!("QR code scan for property: {}", property_id);

    // Extract request information for analytics
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    let ip_address = addr.ip().to_string();
    let referrer = headers.get("referer")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Determine scan source
    let scan_source = match query.source.as_deref() {
        Some("qr") => ScanSource::QrCode,
        Some("direct") => ScanSource::DirectLink,
        Some("share") => ScanSource::ShareLink,
        Some("search") => ScanSource::SearchEngine,
        Some("social") => ScanSource::SocialMedia,
        _ => ScanSource::QrCode, // Default assumption
    };

    // Get property information
    let property_info = match state.property_service.get_property_qr_info(&property_id).await {
        Ok(info) => info,
        Err(_) => {
            warn!("Property not found for scan: {}", property_id);
            // Record failed scan
            let _ = state.analytics_service.record_failed_scan(
                property_id.clone(),
                1, // Default QR version
                "Property not found".to_string(),
                user_agent,
                Some(ip_address),
            ).await;
            
            return Ok(Html(create_error_page("Property not found", &property_id)).into_response());
        }
    };

    // Determine redirect type
    let redirect_type = match query.redirect.as_deref() {
        Some("property") => RedirectType::DaobitarOnly,
        Some("blockchain") => RedirectType::BlockchainOnly,
        Some("dual") | _ => {
            if property_info.onchain_id.is_some() {
                RedirectType::DualRedirect
            } else {
                RedirectType::DaobitarOnly
            }
        }
    };

    // Record scan analytics
    let scan_id = match state.analytics_service.record_scan(
        property_id.clone(),
        1, // QR version - would get from QR metadata
        scan_source,
        redirect_type.clone(),
        user_agent,
        Some(ip_address),
        None, // session_id
        referrer,
    ).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to record scan analytics: {}", e);
            mongodb::bson::oid::ObjectId::new() // Fallback
        }
    };

    // Update property click count
    let _ = state.property_service.increment_property_clicks(&property_id).await;

    // Generate URLs
    let property_url = format!("{}/property/{}", state.daobitar_base_url, property_id);
    let blockchain_url = property_info.onchain_id.as_ref().map(|onchain_id| {
        format!("{}/token/{}", state.blockchain_explorer_base_url, onchain_id)
    });

    // Handle different redirect types
    match redirect_type {
        RedirectType::DaobitarOnly => {
            info!("Redirecting to DAO-Bitat property page: {}", property_id);
            Ok(Redirect::permanent(&property_url).into_response())
        }
        RedirectType::BlockchainOnly => {
            if let Some(blockchain_url) = blockchain_url {
                info!("Redirecting to blockchain explorer: {}", property_id);
                Ok(Redirect::permanent(&blockchain_url).into_response())
            } else {
                warn!("Blockchain redirect requested but no onchain_id: {}", property_id);
                Ok(Redirect::permanent(&property_url).into_response())
            }
        }
        RedirectType::DualRedirect => {
            info!("Showing dual redirect page for property: {}", property_id);
            let redirect_data = ScanRedirectData {
                property_id: property_id.clone(),
                property_name: property_info.property_name.clone(),
                location: Some(property_info.location.clone()),
                daobitar_url: property_url,
                blockchain_url,
                action: property_info.action,
                price: property_info.price,
                primary_image: property_info.images.first().cloned(),
                is_verified: property_info.is_verified.unwrap_or(false),
                crypto_accepted: property_info.crypto_accepted,
                scan_id,
            };
            
            let html_page = create_redirect_page(&redirect_data);
            Ok(Html(html_page).into_response())
        }
        RedirectType::Failed => {
            error!("Scan failed for property: {}", property_id);
            Ok(Html(create_error_page("Scan failed", &property_id)).into_response())
        }
    }
}

/// API endpoint to get scan redirect data as JSON
/// GET /api/scan/{property_id}
pub async fn get_scan_data(
    State(state): State<Arc<ScanAppState>>,
    Path(property_id): Path<String>,
    Query(query): Query<ScanQuery>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<ScanResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Getting scan data for property: {}", property_id);

    // Extract request information
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    let ip_address = addr.ip().to_string();

    // Get property information
    let property_info = match state.property_service.get_property_qr_info(&property_id).await {
        Ok(info) => info,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "property_not_found",
                    "message": "Property not found"
                }))
            ));
        }
    };

    // Determine scan source and redirect type
    let scan_source = match query.source.as_deref() {
        Some("qr") => ScanSource::QrCode,
        Some("direct") => ScanSource::DirectLink,
        _ => ScanSource::QrCode,
    };

    let redirect_type = if property_info.onchain_id.is_some() {
        RedirectType::DualRedirect
    } else {
        RedirectType::DaobitarOnly
    };

    // Record scan
    let scan_id = state.analytics_service.record_scan(
        property_id.clone(),
        1,
        scan_source,
        redirect_type.clone(),
        user_agent,
        Some(ip_address),
        None,
        None,
    ).await.unwrap_or_else(|_| mongodb::bson::oid::ObjectId::new());

    // Generate URLs
    let property_url = format!("{}/property/{}", state.daobitar_base_url, property_id);
    let blockchain_url = property_info.onchain_id.as_ref().map(|onchain_id| {
        format!("{}/token/{}", state.blockchain_explorer_base_url, onchain_id)
    });
    let redirect_page_url = format!("{}/scan/{}", state.daobitar_base_url, property_id);

    let response = ScanResponse {
        success: true,
        property_id,
        redirect_type: match redirect_type {
            RedirectType::DualRedirect => "dual".to_string(),
            RedirectType::DaobitarOnly => "property".to_string(),
            RedirectType::BlockchainOnly => "blockchain".to_string(),
            RedirectType::Failed => "failed".to_string(),
        },
        urls: RedirectUrls {
            property_url,
            blockchain_url,
            redirect_page_url,
        },
        scan_id: scan_id.to_hex(),
    };

    Ok(Json(response))
}

/// Health check endpoint for scan service
/// GET /scan/health
pub async fn scan_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "scan_handler",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Create HTML page for dual redirect
fn create_redirect_page(data: &ScanRedirectData) -> String {
    let blockchain_section = if let Some(blockchain_url) = &data.blockchain_url {
        format!(
            r#"
            <div class="redirect-option blockchain">
                <h3>🔗 View on Blockchain</h3>
                <p>See this property's on-chain verification and ownership details</p>
                <a href="{}" class="redirect-btn blockchain-btn" target="_blank">
                    View on Base Explorer
                </a>
            </div>
            "#,
            blockchain_url
        )
    } else {
        String::new()
    };

    let verified_badge = if data.is_verified {
        r#"<span class="verified-badge">✓ Verified</span>"#
    } else {
        ""
    };

    let crypto_badge = if data.crypto_accepted {
        r#"<span class="crypto-badge">₿ Crypto Accepted</span>"#
    } else {
        ""
    };

    let image_section = if let Some(image_url) = &data.primary_image {
        format!(r#"<img src="{}" alt="Property Image" class="property-image">"#, image_url)
    } else {
        r#"<div class="property-image-placeholder">🏠</div>"#.to_string()
    };

    format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>{} - DAO-Bitat Property</title>
            <style>
                body {{
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    margin: 0;
                    padding: 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }}
                .container {{
                    background: white;
                    border-radius: 20px;
                    padding: 30px;
                    max-width: 600px;
                    width: 100%;
                    box-shadow: 0 20px 40px rgba(0,0,0,0.1);
                    text-align: center;
                }}
                .property-header {{
                    margin-bottom: 30px;
                }}
                .property-image, .property-image-placeholder {{
                    width: 200px;
                    height: 150px;
                    object-fit: cover;
                    border-radius: 10px;
                    margin: 0 auto 20px;
                    display: block;
                    background: #f0f0f0;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 48px;
                }}
                .property-title {{
                    font-size: 24px;
                    font-weight: bold;
                    margin: 10px 0;
                    color: #333;
                }}
                .property-details {{
                    color: #666;
                    margin-bottom: 20px;
                }}
                .property-price {{
                    font-size: 20px;
                    font-weight: bold;
                    color: #2563eb;
                    margin: 10px 0;
                }}
                .badges {{
                    margin: 15px 0;
                }}
                .verified-badge, .crypto-badge {{
                    display: inline-block;
                    background: #10b981;
                    color: white;
                    padding: 5px 12px;
                    border-radius: 20px;
                    font-size: 12px;
                    font-weight: bold;
                    margin: 0 5px;
                }}
                .crypto-badge {{
                    background: #f59e0b;
                }}
                .redirect-options {{
                    display: grid;
                    gap: 20px;
                    margin-top: 30px;
                }}
                .redirect-option {{
                    border: 2px solid #e5e7eb;
                    border-radius: 15px;
                    padding: 20px;
                    transition: all 0.3s ease;
                }}
                .redirect-option:hover {{
                    border-color: #3b82f6;
                    transform: translateY(-2px);
                    box-shadow: 0 10px 20px rgba(0,0,0,0.1);
                }}
                .redirect-option h3 {{
                    margin: 0 0 10px 0;
                    font-size: 18px;
                    color: #333;
                }}
                .redirect-option p {{
                    margin: 0 0 15px 0;
                    color: #666;
                    font-size: 14px;
                }}
                .redirect-btn {{
                    display: inline-block;
                    padding: 12px 24px;
                    background: #3b82f6;
                    color: white;
                    text-decoration: none;
                    border-radius: 8px;
                    font-weight: bold;
                    transition: background 0.3s ease;
                }}
                .redirect-btn:hover {{
                    background: #2563eb;
                }}
                .blockchain-btn {{
                    background: #8b5cf6;
                }}
                .blockchain-btn:hover {{
                    background: #7c3aed;
                }}
                .footer {{
                    margin-top: 30px;
                    padding-top: 20px;
                    border-top: 1px solid #e5e7eb;
                    color: #9ca3af;
                    font-size: 12px;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="property-header">
                    {}
                    <h1 class="property-title">{}</h1>
                   <div class="property-details">
    📍 {} • {}
</div>
                    <div class="property-price">KES {}</div>
                    <div class="badges">
                        {}
                        {}
                    </div>
                </div>

                <div class="redirect-options">
                    <div class="redirect-option property">
                        <h3>🏠 View Property Details</h3>
                        <p>See full property information, photos, and contact the owner</p>
                        <a href="{}" class="redirect-btn">
                            View on DAO-Bitat
                        </a>
                    </div>

                    {}
                </div>

                <div class="footer">
                    <p>Powered by DAO-Bitat • Secure Property Transactions</p>
                    <p>Scan ID: {}</p>
                </div>
            </div>

            <script>
                // Auto-redirect after 10 seconds to property page
                setTimeout(() => {{
                    window.location.href = '{}';
                }}, 10000);
            </script>
        </body>
        </html>
        "#,
        data.property_name,
        image_section,
        data.property_name,
        data.location.as_deref().unwrap_or("Location not specified"), 
        data.action,
        data.price,
        verified_badge,
        crypto_badge,
        data.daobitar_url,
        blockchain_section,
        data.scan_id.to_hex(),
        data.daobitar_url
    )
}

/// Create error page HTML
fn create_error_page(error_message: &str, property_id: &str) -> String {
    format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Error - DAO-Bitat</title>
            <style>
                body {{
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    margin: 0;
                    padding: 20px;
                    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    color: white;
                }}
                .container {{
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 20px;
                    padding: 40px;
                    max-width: 500px;
                    width: 100%;
                    text-align: center;
                    backdrop-filter: blur(10px);
                }}
                .error-icon {{
                    font-size: 64px;
                    margin-bottom: 20px;
                }}
                h1 {{
                    margin: 0 0 10px 0;
                    font-size: 24px;
                }}
                p {{
                    margin: 0 0 30px 0;
                    opacity: 0.9;
                }}
                .home-btn {{
                    display: inline-block;
                    padding: 12px 24px;
                    background: white;
                    color: #dc2626;
                    text-decoration: none;
                    border-radius: 8px;
                    font-weight: bold;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="error-icon">⚠️</div>
                <h1>{}</h1>
                <p>Property ID: {}</p>
                <p>The property you're looking for could not be found or is no longer available.</p>
                <a href="https://daobitat.xyz" class="home-btn">
                    Go to DAO-Bitat
                </a>
            </div>
        </body>
        </html>
        "#,
        error_message, property_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_error_page() {
        let html = create_error_page("Test Error", "test123");
        assert!(html.contains("Test Error"));
        assert!(html.contains("test123"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_scan_response_creation() {
        let response = ScanResponse {
            success: true,
            property_id: "test123".to_string(),
            redirect_type: "dual".to_string(),
            urls: RedirectUrls {
                property_url: "https://daobitat.xyz/property/test123".to_string(),
                blockchain_url: Some("https://explorer.base.org/token/test123".to_string()),
                redirect_page_url: "https://qr.daobitat.xyz/scan/test123".to_string(),
            },
            scan_id: "scan123".to_string(),
        };

        assert_eq!(response.success, true);
        assert_eq!(response.property_id, "test123");
    }
}