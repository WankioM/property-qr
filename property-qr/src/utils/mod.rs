 
// src/utils/mod.rs

pub mod validation;
pub mod url_builder;

// Re-export commonly used validation functions
pub use validation::{
    validate_object_id, validate_property_id, validate_user_id,
    validate_price, validate_email, validate_url, validate_coordinates,
    ValidationError, ValidationResult, ValidationBuilder
};

pub use url_builder::{UrlBuilder, PropertySearchFilters, UrlValidator};