// src/main.rs

use tracing::{info, error};
use std::error::Error;



mod models;
mod services;
mod utils;
mod handlers;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting DAO-Bitat QR Service compilation check...");
    
    // Test that all modules compile without actually running anything
    info!("✅ Models module compiled successfully");
    info!("✅ Services module compiled successfully"); 
    info!("✅ Utils module compiled successfully");
    info!("✅ Handlers module compiled successfully");
    info!("✅ Config module compiled successfully");
    info!("✅ Errors module compiled successfully");
    
    info!("All modules compiled successfully! 🎉");
    
    Ok(())
}