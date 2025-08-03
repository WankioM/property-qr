use std::collections::HashMap;

/// URL builder utility for constructing application URLs
#[derive(Debug, Clone)]
pub struct UrlBuilder {
    base_url: String,
    daobitat_base_url: String,
    blockchain_explorer_base_url: String,
}

impl UrlBuilder {
    /// Create a new URL builder
    pub fn new(
        base_url: String,
        daobitat_base_url: String,
        blockchain_explorer_base_url: String,
    ) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            daobitat_base_url: daobitat_base_url.trim_end_matches('/').to_string(),
            blockchain_explorer_base_url: blockchain_explorer_base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Create URL builder with default settings
    pub fn default_config() -> Self {
        Self::new(
            "https://qr-service.daobitat.xyz".to_string(),
            "https://www.daobitat.xyz".to_string(),
            "https://basescan.org".to_string(),
        )
    }

    /// Build QR scan URL
    pub fn build_scan_url(&self, property_id: &str) -> String {
        format!("{}/scan/{}", self.base_url, property_id)
    }

    /// Build DAO-Bitat property details URL
    pub fn build_daobitat_property_url(&self, property_id: &str) -> String {
        format!("{}/property-details/{}", self.daobitat_base_url, property_id)
    }

    /// Build blockchain explorer URL for onchain ID
    pub fn build_blockchain_explorer_url(&self, onchain_id: &str) -> String {
        format!("{}/address/{}", self.blockchain_explorer_base_url, onchain_id)
    }

    /// Build API endpoint URL
    pub fn build_api_url(&self, endpoint: &str) -> String {
        let endpoint = endpoint.trim_start_matches('/');
        format!("{}/api/v1/{}", self.base_url, endpoint)
    }

    /// Build QR generation API URL
    pub fn build_qr_generate_url(&self, property_id: &str) -> String {
        self.build_api_url(&format!("qr/generate/{}", property_id))
    }

    /// Build QR analytics API URL
    pub fn build_qr_analytics_url(&self, property_id: &str) -> String {
        self.build_api_url(&format!("qr/{}/analytics", property_id))
    }

    /// Build batch QR generation URL
    pub fn build_batch_qr_generate_url(&self) -> String {
        self.build_api_url("qr/batch-generate")
    }

    /// Build health check URL
    pub fn build_health_url(&self) -> String {
        format!("{}/health", self.base_url)
    }

    /// Build webhook URL for property updates
    pub fn build_property_webhook_url(&self, property_id: &str) -> String {
        self.build_api_url(&format!("webhooks/property/{}", property_id))
    }

    /// Build S3 object URL
    pub fn build_s3_url(&self, bucket: &str, key: &str, region: &str) -> String {
        format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, key)
    }

    /// Build CloudFront URL
    pub fn build_cloudfront_url(&self, distribution_domain: &str, key: &str) -> String {
        format!("https://{}/{}", distribution_domain.trim_end_matches('/'), key)
    }

    /// Build URL with query parameters
    pub fn build_url_with_params(&self, base_path: &str, params: HashMap<String, String>) -> String {
        let base = if base_path.starts_with("http") {
            base_path.to_string()
        } else {
            format!("{}/{}", self.base_url, base_path.trim_start_matches('/'))
        };

        if params.is_empty() {
            return base;
        }

        let query_string: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();

        format!("{}?{}", base, query_string.join("&"))
    }

    /// Build pagination URL
    pub fn build_paginated_url(&self, base_path: &str, page: u32, limit: u32) -> String {
        let mut params = HashMap::new();
        params.insert("page".to_string(), page.to_string());
        params.insert("limit".to_string(), limit.to_string());
        self.build_url_with_params(base_path, params)
    }

    /// Build search URL with query parameters
    pub fn build_search_url(&self, endpoint: &str, query: &str, filters: Option<HashMap<String, String>>) -> String {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(filters) = filters {
            params.extend(filters);
        }

        self.build_url_with_params(&self.build_api_url(endpoint), params)
    }

    /// Build property search URL
    pub fn build_property_search_url(&self, query: &str, filters: Option<PropertySearchFilters>) -> String {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(filters) = filters {
            if let Some(location) = filters.location {
                params.insert("location".to_string(), location);
            }
            if let Some(min_price) = filters.min_price {
                params.insert("min_price".to_string(), min_price.to_string());
            }
            if let Some(max_price) = filters.max_price {
                params.insert("max_price".to_string(), max_price.to_string());
            }
            if let Some(property_type) = filters.property_type {
                params.insert("property_type".to_string(), property_type);
            }
            if filters.verified_only {
                params.insert("verified_only".to_string(), "true".to_string());
            }
            if filters.crypto_accepted {
                params.insert("crypto_accepted".to_string(), "true".to_string());
            }
        }

        self.build_url_with_params("api/v1/properties/search", params)
    }

    /// Build redirect HTML for dual redirect functionality
    pub fn build_dual_redirect_html(&self, property_id: &str, property_name: &str, onchain_id: Option<&str>) -> String {
        let daobitat_url = self.build_daobitat_property_url(property_id);
        let blockchain_url = onchain_id.map(|id| self.build_blockchain_explorer_url(id));

        let mut html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Redirecting to {}</title>
    <style>
        body {{ 
            font-family: Arial, sans-serif; 
            text-align: center; 
            padding: 50px; 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }}
        .container {{
            max-width: 600px;
            margin: 0 auto;
            background: rgba(255,255,255,0.1);
            padding: 30px;
            border-radius: 15px;
            backdrop-filter: blur(10px);
        }}
        .logo {{
            font-size: 2em;
            font-weight: bold;
            margin-bottom: 20px;
        }}
        .property-name {{
            font-size: 1.5em;
            margin-bottom: 20px;
        }}
        .redirect-info {{
            margin-bottom: 30px;
            line-height: 1.6;
        }}
        .links {{
            display: flex;
            gap: 20px;
            justify-content: center;
            flex-wrap: wrap;
        }}
        .link-button {{
            background: rgba(255,255,255,0.2);
            color: white;
            text-decoration: none;
            padding: 12px 24px;
            border-radius: 8px;
            border: 1px solid rgba(255,255,255,0.3);
            transition: background 0.3s;
        }}
        .link-button:hover {{
            background: rgba(255,255,255,0.3);
        }}
        .countdown {{
            margin-top: 20px;
            font-size: 0.9em;
            opacity: 0.8;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">🏡 DAO-Bitat</div>
        <div class="property-name">{}</div>
        <div class="redirect-info">
            <p>You're being redirected to view this property...</p>
            <p>If the redirect doesn't work, use the links below:</p>
        </div>
        <div class="links">
            <a href="{}" class="link-button" target="_blank">View Property Details</a>"#,
            property_name, property_name, daobitat_url
        );

        if let Some(blockchain_url) = blockchain_url {
            html.push_str(&format!(
                r#"<a href="{}" class="link-button" target="_blank">View on Blockchain</a>"#,
                blockchain_url
            ));
        }

        html.push_str(&format!(
            r#"
        </div>
        <div class="countdown" id="countdown">Redirecting in 3 seconds...</div>
    </div>
    <script>
        // Immediate redirect to main property page
        setTimeout(function() {{
            window.location.href = '{}';
        }}, 3000);

        // Open blockchain explorer in new tab if available
        {}

        // Countdown timer
        let countdown = 3;
        const countdownElement = document.getElementById('countdown');
        const timer = setInterval(function() {{
            countdown--;
            if (countdown > 0) {{
                countdownElement.textContent = `Redirecting in ${{countdown}} seconds...`;
            }} else {{
                countdownElement.textContent = 'Redirecting now...';
                clearInterval(timer);
            }}
        }}, 1000);
    </script>
</body>
</html>"#,
            daobitat_url,
            if blockchain_url.is_some() {
                format!(
                    "setTimeout(function() {{ window.open('{}', '_blank'); }}, 1000);",
                    blockchain_url.unwrap()
                )
            } else {
                String::new()
            }
        ));

        html
    }

    /// Get base URL
    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    /// Get DAO-Bitat base URL
    pub fn get_daobitat_base_url(&self) -> &str {
        &self.daobitat_base_url
    }

    /// Get blockchain explorer base URL
    pub fn get_blockchain_explorer_base_url(&self) -> &str {
        &self.blockchain_explorer_base_url
    }

    /// Update base URLs
    pub fn update_urls(&mut self, base_url: Option<String>, daobitat_url: Option<String>, blockchain_url: Option<String>) {
        if let Some(url) = base_url {
            self.base_url = url.trim_end_matches('/').to_string();
        }
        if let Some(url) = daobitat_url {
            self.daobitat_base_url = url.trim_end_matches('/').to_string();
        }
        if let Some(url) = blockchain_url {
            self.blockchain_explorer_base_url = url.trim_end_matches('/').to_string();
        }
    }
}

/// Property search filters for URL building
#[derive(Debug, Clone, Default)]
pub struct PropertySearchFilters {
    pub location: Option<String>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub property_type: Option<String>,
    pub verified_only: bool,
    pub crypto_accepted: bool,
}

impl PropertySearchFilters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn price_range(mut self, min: Option<i64>, max: Option<i64>) -> Self {
        self.min_price = min;
        self.max_price = max;
        self
    }

    pub fn property_type(mut self, property_type: String) -> Self {
        self.property_type = Some(property_type);
        self
    }

    pub fn verified_only(mut self) -> Self {
        self.verified_only = true;
        self
    }

    pub fn crypto_accepted(mut self) -> Self {
        self.crypto_accepted = true;
        self
    }
}

/// URL validation utilities
pub struct UrlValidator;

impl UrlValidator {
    /// Validate if URL is well-formed
    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Validate if URL is secure (HTTPS)
    pub fn is_secure_url(url: &str) -> bool {
        url.starts_with("https://")
    }

    /// Extract domain from URL
    pub fn extract_domain(url: &str) -> Option<String> {
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                Some(after_protocol[..end].to_string())
            } else {
                Some(after_protocol.to_string())
            }
        } else {
            None
        }
    }

    /// Check if URL belongs to allowed domains
    pub fn is_allowed_domain(url: &str, allowed_domains: &[&str]) -> bool {
        if let Some(domain) = Self::extract_domain(url) {
            allowed_domains.iter().any(|&allowed| domain.ends_with(allowed))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_builder_creation() {
        let builder = UrlBuilder::new(
            "https://qr.example.com".to_string(),
            "https://app.example.com".to_string(),
            "https://explorer.example.com".to_string(),
        );

        assert_eq!(builder.get_base_url(), "https://qr.example.com");
        assert_eq!(builder.get_daobitat_base_url(), "https://app.example.com");
        assert_eq!(builder.get_blockchain_explorer_base_url(), "https://explorer.example.com");
    }

    #[test]
    fn test_scan_url_building() {
        let builder = UrlBuilder::default_config();
        let scan_url = builder.build_scan_url("12345");
        assert_eq!(scan_url, "https://qr-service.daobitat.xyz/scan/12345");
    }

    #[test]
    fn test_property_url_building() {
        let builder = UrlBuilder::default_config();
        let property_url = builder.build_daobitat_property_url("67890");
        assert_eq!(property_url, "https://www.daobitat.xyz/property-details/67890");
    }

    #[test]
    fn test_blockchain_url_building() {
        let builder = UrlBuilder::default_config();
        let blockchain_url = builder.build_blockchain_explorer_url("0x1234567890abcdef");
        assert_eq!(blockchain_url, "https://basescan.org/address/0x1234567890abcdef");
    }

    #[test]
    fn test_url_with_params() {
        let builder = UrlBuilder::default_config();
        let mut params = HashMap::new();
        params.insert("page".to_string(), "2".to_string());
        params.insert("limit".to_string(), "20".to_string());

        let url = builder.build_url_with_params("api/v1/properties", params);
        assert!(url.contains("page=2"));
        assert!(url.contains("limit=20"));
        assert!(url.starts_with("https://qr-service.daobitat.xyz/api/v1/properties?"));
    }

    #[test]
    fn test_property_search_url() {
        let builder = UrlBuilder::default_config();
        let filters = PropertySearchFilters::new()
            .location("Nairobi".to_string())
            .price_range(Some(100000), Some(500000))
            .verified_only();

        let search_url = builder.build_property_search_url("apartment", Some(filters));
        assert!(search_url.contains("q=apartment"));
        assert!(search_url.contains("location=Nairobi"));
        assert!(search_url.contains("min_price=100000"));
        assert!(search_url.contains("verified_only=true"));
    }

    #[test]
    fn test_url_validator() {
        assert!(UrlValidator::is_valid_url("https://example.com"));
        assert!(UrlValidator::is_valid_url("http://example.com"));
        assert!(!UrlValidator::is_valid_url("example.com"));

        assert!(UrlValidator::is_secure_url("https://example.com"));
        assert!(!UrlValidator::is_secure_url("http://example.com"));

        assert_eq!(
            UrlValidator::extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );

        let allowed_domains = &["daobitat.xyz", "basescan.org"];
        assert!(UrlValidator::is_allowed_domain("https://qr-service.daobitat.xyz/scan/123", allowed_domains));
        assert!(!UrlValidator::is_allowed_domain("https://malicious.com/scan/123", allowed_domains));
    }

    #[test]
    fn test_dual_redirect_html() {
        let builder = UrlBuilder::default_config();
        let html = builder.build_dual_redirect_html(
            "test123",
            "Test Property",
            Some("0x1234567890abcdef")
        );

        assert!(html.contains("Test Property"));
        assert!(html.contains("property-details/test123"));
        assert!(html.contains("address/0x1234567890abcdef"));
        assert!(html.contains("<!DOCTYPE html>"));
    }
}