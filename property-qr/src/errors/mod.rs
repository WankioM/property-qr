 
// src/errors/mod.rs

pub mod app_error;

// Re-export the main error types for easier imports
pub use app_error::{AppError, AppResult, ErrorCode, ErrorContext};