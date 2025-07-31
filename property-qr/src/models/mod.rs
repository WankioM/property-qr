 // src/models/mod.rs

pub mod property;
pub mod qr_code;
pub mod scan_analytics;

// Re-export commonly used types for convenience
pub use property::*;
pub use qr_code::*;
pub use scan_analytics::*;
