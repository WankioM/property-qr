 // src/config/mod.rs

pub mod aws;
pub mod settings;

// Re-export the main types for easier imports
pub use aws::AwsConfig;
pub use settings::Settings;
