 // src/routes/mod.rs

pub mod api;

// Re-export route functions
pub use api::{qr_routes, scan_routes, health_routes};
