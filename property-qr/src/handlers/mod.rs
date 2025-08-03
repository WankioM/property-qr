 // src/handlers/mod.rs

pub mod health;
pub mod qr_handler;
pub mod scan_handler;

// Re-export handler functions for convenience
pub use health::*;
pub use qr_handler::*;
pub use scan_handler::*;
