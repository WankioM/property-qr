  // src/utils/validation.rs

use mongodb::bson::oid::ObjectId;
use std::str::FromStr;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub error_code: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str, error_code: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            error_code: error_code.to_string(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

pub type ValidationResult<T> = Result<T, ValidationError>;

/// MongoDB ObjectId validation
pub fn validate_object_id(id: &str, field_name: &str) -> ValidationResult<ObjectId> {
    if id.is_empty() {
        return Err(ValidationError::new(
            field_name,
            "ID cannot be empty",
            "EMPTY_ID"
        ));
    }

    ObjectId::from_str(id).map_err(|_| {
        ValidationError::new(
            field_name,
            "Invalid ObjectId format",
            "INVALID_OBJECT_ID"
        )
    })
}

/// Email validation (basic)
pub fn validate_email(email: &str) -> ValidationResult<()> {
    if email.is_empty() {
        return Err(ValidationError::new(
            "email",
            "Email cannot be empty",
            "EMPTY_EMAIL"
        ));
    }

    if !email.contains('@') || !email.contains('.') {
        return Err(ValidationError::new(
            "email",
            "Invalid email format",
            "INVALID_EMAIL_FORMAT"
        ));
    }

    if email.len() > 254 {
        return Err(ValidationError::new(
            "email",
            "Email too long",
            "EMAIL_TOO_LONG"
        ));
    }

    Ok(())
}

/// URL validation
pub fn validate_url(url: &str, field_name: &str) -> ValidationResult<()> {
    if url.is_empty() {
        return Err(ValidationError::new(
            field_name,
            "URL cannot be empty",
            "EMPTY_URL"
        ));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ValidationError::new(
            field_name,
            "URL must start with http:// or https://",
            "INVALID_URL_SCHEME"
        ));
    }

    if url.len() > 2048 {
        return Err(ValidationError::new(
            field_name,
            "URL too long",
            "URL_TOO_LONG"
        ));
    }

    Ok(())
}

/// Property ID validation (MongoDB ObjectId)
pub fn validate_property_id(property_id: &str) -> ValidationResult<ObjectId> {
    validate_object_id(property_id, "property_id")
}

/// User ID validation (MongoDB ObjectId)
pub fn validate_user_id(user_id: &str) -> ValidationResult<ObjectId> {
    validate_object_id(user_id, "user_id")
}

/// Price validation
pub fn validate_price(price: i64, field_name: &str) -> ValidationResult<()> {
    if price < 0 {
        return Err(ValidationError::new(
            field_name,
            "Price cannot be negative",
            "NEGATIVE_PRICE"
        ));
    }

    if price == 0 {
        return Err(ValidationError::new(
            field_name,
            "Price cannot be zero",
            "ZERO_PRICE"
        ));
    }

    if price > 1_000_000_000_000 { // 1 trillion limit
        return Err(ValidationError::new(
            field_name,
            "Price too high",
            "PRICE_TOO_HIGH"
        ));
    }

    Ok(())
}

/// Text length validation
pub fn validate_text_length(
    text: &str, 
    field_name: &str, 
    min_length: usize, 
    max_length: usize
) -> ValidationResult<()> {
    if text.len() < min_length {
        return Err(ValidationError::new(
            field_name,
            &format!("Text too short (minimum {} characters)", min_length),
            "TEXT_TOO_SHORT"
        ));
    }

    if text.len() > max_length {
        return Err(ValidationError::new(
            field_name,
            &format!("Text too long (maximum {} characters)", max_length),
            "TEXT_TOO_LONG"
        ));
    }

    Ok(())
}

/// Property name validation
pub fn validate_property_name(name: &str) -> ValidationResult<()> {
    validate_text_length(name, "property_name", 3, 100)?;
    
    if name.trim().is_empty() {
        return Err(ValidationError::new(
            "property_name",
            "Property name cannot be only whitespace",
            "WHITESPACE_ONLY"
        ));
    }

    Ok(())
}

/// Location validation
pub fn validate_location(location: &str) -> ValidationResult<()> {
    validate_text_length(location, "location", 3, 200)?;
    
    if location.trim().is_empty() {
        return Err(ValidationError::new(
            "location",
            "Location cannot be only whitespace",
            "WHITESPACE_ONLY"
        ));
    }

    Ok(())
}

/// Coordinates validation
pub fn validate_coordinates(lat: f64, lng: f64) -> ValidationResult<()> {
    if lat < -90.0 || lat > 90.0 {
        return Err(ValidationError::new(
            "latitude",
            "Latitude must be between -90 and 90",
            "INVALID_LATITUDE"
        ));
    }

    if lng < -180.0 || lng > 180.0 {
        return Err(ValidationError::new(
            "longitude",
            "Longitude must be between -180 and 180",
            "INVALID_LONGITUDE"
        ));
    }

    Ok(())
}

/// Image URL validation
pub fn validate_image_url(url: &str) -> ValidationResult<()> {
    validate_url(url, "image_url")?;
    
    let url_lower = url.to_lowercase();
    if !url_lower.ends_with(".jpg") 
        && !url_lower.ends_with(".jpeg") 
        && !url_lower.ends_with(".png") 
        && !url_lower.ends_with(".webp") {
        
        warn!("Image URL doesn't have common image extension: {}", url);
        // Don't fail validation, just warn
    }

    Ok(())
}

/// QR scan URL validation
pub fn validate_scan_url(url: &str) -> ValidationResult<()> {
    validate_url(url, "scan_url")?;
    
    if !url.contains("/scan/") {
        return Err(ValidationError::new(
            "scan_url",
            "Invalid scan URL format - must contain /scan/",
            "INVALID_SCAN_URL_FORMAT"
        ));
    }

    Ok(())
}

/// Phone number validation (basic international format)
pub fn validate_phone_number(phone: &str) -> ValidationResult<()> {
    if phone.is_empty() {
        return Err(ValidationError::new(
            "phone",
            "Phone number cannot be empty",
            "EMPTY_PHONE"
        ));
    }

    // Remove common separators for validation
    let cleaned = phone.replace([' ', '-', '(', ')', '+'], "");
    
    if cleaned.len() < 7 || cleaned.len() > 15 {
        return Err(ValidationError::new(
            "phone",
            "Phone number must be between 7 and 15 digits",
            "INVALID_PHONE_LENGTH"
        ));
    }

    if !cleaned.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new(
            "phone",
            "Phone number can only contain digits and separators",
            "INVALID_PHONE_CHARACTERS"
        ));
    }

    Ok(())
}

/// Password validation
pub fn validate_password(password: &str) -> ValidationResult<()> {
    if password.len() < 8 {
        return Err(ValidationError::new(
            "password",
            "Password must be at least 8 characters long",
            "PASSWORD_TOO_SHORT"
        ));
    }

    if password.len() > 128 {
        return Err(ValidationError::new(
            "password",
            "Password too long",
            "PASSWORD_TOO_LONG"
        ));
    }

    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_lowercase || !has_uppercase || !has_digit {
        return Err(ValidationError::new(
            "password",
            "Password must contain at least one lowercase letter, one uppercase letter, and one digit",
            "PASSWORD_WEAK"
        ));
    }

    Ok(())
}

/// Pagination parameters validation
pub fn validate_pagination(page: Option<u32>, limit: Option<u32>) -> ValidationResult<(u32, u32)> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);

    if page == 0 {
        return Err(ValidationError::new(
            "page",
            "Page number must be greater than 0",
            "INVALID_PAGE"
        ));
    }

    if limit == 0 {
        return Err(ValidationError::new(
            "limit",
            "Limit must be greater than 0",
            "INVALID_LIMIT"
        ));
    }

    if limit > 100 {
        return Err(ValidationError::new(
            "limit",
            "Limit cannot exceed 100",
            "LIMIT_TOO_HIGH"
        ));
    }

    Ok((page, limit))
}

/// Search query validation
pub fn validate_search_query(query: &str) -> ValidationResult<()> {
    if query.is_empty() {
        return Err(ValidationError::new(
            "query",
            "Search query cannot be empty",
            "EMPTY_QUERY"
        ));
    }

    if query.len() < 2 {
        return Err(ValidationError::new(
            "query",
            "Search query must be at least 2 characters",
            "QUERY_TOO_SHORT"
        ));
    }

    if query.len() > 100 {
        return Err(ValidationError::new(
            "query",
            "Search query too long",
            "QUERY_TOO_LONG"
        ));
    }

    Ok(())
}

/// Property action validation (for sale/rent)
pub fn validate_property_action(action: &str) -> ValidationResult<()> {
    let valid_actions = ["for sale", "for rent", "for lease"];
    
    if !valid_actions.contains(&action.to_lowercase().as_str()) {
        return Err(ValidationError::new(
            "action",
            "Invalid property action. Must be 'for sale', 'for rent', or 'for lease'",
            "INVALID_ACTION"
        ));
    }

    Ok(())
}

/// Blockchain address validation (basic)
pub fn validate_blockchain_address(address: &str, field_name: &str) -> ValidationResult<()> {
    if address.is_empty() {
        return Err(ValidationError::new(
            field_name,
            "Blockchain address cannot be empty",
            "EMPTY_ADDRESS"
        ));
    }

    // Basic Ethereum address validation (0x + 40 hex chars)
    if address.starts_with("0x") && address.len() == 42 {
        let hex_part = &address[2..];
        if hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(());
        }
    }

    // Could add more blockchain address formats here
    return Err(ValidationError::new(
        field_name,
        "Invalid blockchain address format",
        "INVALID_ADDRESS_FORMAT"
    ));
}

/// Validation helper for multiple fields
pub struct ValidationBuilder {
    errors: Vec<ValidationError>,
}

impl ValidationBuilder {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn validate<T, F>(&mut self, validation_fn: F) -> &mut Self 
    where 
        F: FnOnce() -> ValidationResult<T>
    {
        if let Err(error) = validation_fn() {
            self.errors.push(error);
        }
        self
    }

    pub fn build(self) -> ValidationResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            // Return the first error, or could be modified to return all errors
            Err(self.errors.into_iter().next().unwrap())
        }
    }

    pub fn build_all_errors(self) -> Result<(), Vec<ValidationError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}

impl Default for ValidationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_object_id() {
        // Valid ObjectId
        let valid_id = "507f1f77bcf86cd799439011";
        assert!(validate_object_id(valid_id, "test_id").is_ok());

        // Invalid ObjectId
        assert!(validate_object_id("invalid", "test_id").is_err());
        assert!(validate_object_id("", "test_id").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("").is_err());
        assert!(validate_email("test@").is_err());
    }

    #[test]
    fn test_validate_price() {
        assert!(validate_price(100, "price").is_ok());
        assert!(validate_price(0, "price").is_err());
        assert!(validate_price(-50, "price").is_err());
        assert!(validate_price(1_000_000_000_001, "price").is_err());
    }

    #[test]
    fn test_validate_coordinates() {
        assert!(validate_coordinates(0.0, 0.0).is_ok());
        assert!(validate_coordinates(-90.0, -180.0).is_ok());
        assert!(validate_coordinates(90.0, 180.0).is_ok());
        assert!(validate_coordinates(-91.0, 0.0).is_err());
        assert!(validate_coordinates(0.0, 181.0).is_err());
    }

    #[test]
    fn test_validate_property_action() {
        assert!(validate_property_action("for sale").is_ok());
        assert!(validate_property_action("for rent").is_ok());
        assert!(validate_property_action("For Sale").is_ok()); // Case insensitive
        assert!(validate_property_action("invalid").is_err());
    }

    #[test]
    fn test_validate_blockchain_address() {
        let valid_eth_address = "0x1234567890123456789012345678901234567890";
        assert!(validate_blockchain_address(valid_eth_address, "address").is_ok());
        
        assert!(validate_blockchain_address("invalid", "address").is_err());
        assert!(validate_blockchain_address("", "address").is_err());
    }

    #[test]
    fn test_validation_builder() {
        let mut builder = ValidationBuilder::new();
        let result = builder
            .validate(|| validate_price(100, "price"))
            .validate(|| validate_email("test@example.com"))
            .build();
        
        assert!(result.is_ok());

        let mut builder = ValidationBuilder::new();
        let result = builder
            .validate(|| validate_price(-100, "price"))
            .validate(|| validate_email("invalid"))
            .build_all_errors();
        
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }
}

