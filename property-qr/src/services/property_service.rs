// src/services/property_service.rs

use crate::models::{Property, PropertyQrInfo};
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
    Collection, Database, options::FindOptions,
};
use std::str::FromStr;
use tracing::info;

#[derive(Clone)]
pub struct PropertyService {
    properties: Collection<Property>,
}

#[derive(Debug)]
pub enum PropertyError {
    NotFound,
    InvalidId,
    DatabaseError(mongodb::error::Error),
    NotEligibleForQr(String),
}

impl From<mongodb::error::Error> for PropertyError {
    fn from(err: mongodb::error::Error) -> Self {
        PropertyError::DatabaseError(err)
    }
}

impl std::fmt::Display for PropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyError::NotFound => write!(f, "Property not found"),
            PropertyError::InvalidId => write!(f, "Invalid property ID format"),
            PropertyError::DatabaseError(e) => write!(f, "Database error: {}", e),
            PropertyError::NotEligibleForQr(reason) => write!(f, "Property not eligible for QR: {}", reason),
        }
    }
}

impl std::error::Error for PropertyError {}

// Helper function to convert chrono DateTime to BSON DateTime
fn utc_to_bson(dt: chrono::DateTime<chrono::Utc>) -> BsonDateTime {
    BsonDateTime::from_millis(dt.timestamp_millis())
}

impl PropertyService {
    /// Create a new property service
    pub fn new(db: &Database) -> Self {
        Self {
            properties: db.collection("properties"),
        }
    }

    /// Get a property by its MongoDB ID
    pub async fn get_property_by_id(&self, property_id: &str) -> Result<Property, PropertyError> {
        let object_id = ObjectId::from_str(property_id)
            .map_err(|_| PropertyError::InvalidId)?;

        let property = self.properties
            .find_one(doc! { "_id": object_id })
            .await?
            .ok_or(PropertyError::NotFound)?;

        Ok(property)
    }

    /// Get property info suitable for QR generation
    pub async fn get_property_qr_info(&self, property_id: &str) -> Result<PropertyQrInfo, PropertyError> {
        let property = self.get_property_by_id(property_id).await?;
        
        // Check if property is eligible for QR generation
        if !property.is_qr_eligible() {
            let reason = self.get_ineligibility_reason(&property);
            return Err(PropertyError::NotEligibleForQr(reason));
        }

        Ok(property.to_qr_info())
    }

    /// Get multiple properties by their IDs
    pub async fn get_properties_by_ids(&self, property_ids: Vec<String>) -> Result<Vec<Property>, PropertyError> {
        let mut object_ids = Vec::new();
        
        for id in property_ids {
            let object_id = ObjectId::from_str(&id)
                .map_err(|_| PropertyError::InvalidId)?;
            object_ids.push(object_id);
        }

        let filter = doc! { "_id": { "$in": object_ids } };
        let mut cursor = self.properties.find(filter).await?;
        let mut properties = Vec::new();

        while cursor.advance().await? {
            let property = cursor.deserialize_current()?;
            properties.push(property);
        }

        Ok(properties)
    }

    /// Get properties eligible for QR generation
    pub async fn get_qr_eligible_properties(&self, limit: Option<i64>) -> Result<Vec<PropertyQrInfo>, PropertyError> {
        let filter = doc! {
            "removed": { "$ne": true },
            "images.0": { "$exists": true },
            "price": { "$gt": 0 }
        };

        let options = if let Some(limit) = limit {
            FindOptions::builder()
                .limit(limit)
                .sort(doc! { "createdAt": -1 })
                .build()
        } else {
            FindOptions::builder()
                .sort(doc! { "createdAt": -1 })
                .build()
        };
       

        let mut cursor = self.properties.find(filter).with_options(options).await?;
        let mut qr_infos = Vec::new();

        while cursor.advance().await? {
            let property: Property = cursor.deserialize_current()?;
            if property.is_qr_eligible() {
                qr_infos.push(property.to_qr_info());
            }
        }

        info!("Found {} QR-eligible properties", qr_infos.len());
        Ok(qr_infos)
    }

    /// Get properties that need QR code generation
    pub async fn get_properties_needing_qr(&self, qr_property_ids: Vec<String>) -> Result<Vec<PropertyQrInfo>, PropertyError> {
        // Find properties that don't have QR codes yet
        let filter = doc! {
            "removed": { "$ne": true },
            "images.0": { "$exists": true },
            "price": { "$gt": 0 },
            "_id": { "$nin": qr_property_ids }
        };

        let mut cursor = self.properties.find(filter).await?;
        let mut properties_needing_qr = Vec::new();

        while cursor.advance().await? {
            let property: Property = cursor.deserialize_current()?;
            if property.is_qr_eligible() {
                properties_needing_qr.push(property.to_qr_info());
            }
        }

        info!("Found {} properties needing QR codes", properties_needing_qr.len());
        Ok(properties_needing_qr)
    }

    /// Check if property exists and is valid
    pub async fn validate_property(&self, property_id: &str) -> Result<bool, PropertyError> {
        match self.get_property_by_id(property_id).await {
            Ok(property) => Ok(!property.removed.unwrap_or(false)),
            Err(PropertyError::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get property statistics
    pub async fn get_property_stats(&self) -> Result<PropertyStats, PropertyError> {
        let total_properties = self.properties
            .count_documents(doc! {})
            .await? as i64;

        let active_properties = self.properties
            .count_documents(doc! { "removed": { "$ne": true } })
            .await? as i64;

        let verified_properties = self.properties
            .count_documents(doc! { 
                "removed": { "$ne": true },
                "isVerified": true 
            })
            .await? as i64;

        let properties_with_images = self.properties
            .count_documents(doc! { 
                "removed": { "$ne": true },
                "images.0": { "$exists": true }
            })
            .await? as i64;

        let qr_eligible_properties = self.properties
            .count_documents(doc! {
                "removed": { "$ne": true },
                "images.0": { "$exists": true },
                "price": { "$gt": 0 }
            })
            .await? as i64;

        Ok(PropertyStats {
            total_properties,
            active_properties,
            verified_properties,
            properties_with_images,
            qr_eligible_properties,
        })
    }

    /// Search properties by criteria
    pub async fn search_properties(&self, criteria: PropertySearchCriteria) -> Result<Vec<Property>, PropertyError> {
        let mut filter = doc! { "removed": { "$ne": true } };

        // Add location filter
        if let Some(location) = criteria.location {
            filter.insert("location", doc! { "$regex": location, "$options": "i" });
        }

        // Add price range filter
      // Add price range filter
let mut price_conditions = doc! {};
if let Some(min_price) = criteria.min_price {
    price_conditions.insert("$gte", min_price);
}
if let Some(max_price) = criteria.max_price {
    price_conditions.insert("$lte", max_price);
}
if !price_conditions.is_empty() {
    filter.insert("price", price_conditions);
}
        // Add property type filter
        if let Some(property_type) = criteria.property_type {
            filter.insert("propertyType", property_type);
        }

        // Add verification filter
        if criteria.verified_only {
            filter.insert("isVerified", true);
        }

        // Add blockchain filter
        if criteria.blockchain_only {
            filter.insert("onchainId", doc! { "$exists": true, "$ne": null });
        }

        let options = if let Some(limit) = criteria.limit {
            FindOptions::builder()
                .limit(limit)
                .sort(doc! { "createdAt": -1 })
                .build()
        } else {
            FindOptions::builder()
                .sort(doc! { "createdAt": -1 })
                .build()
        };

        let mut cursor = self.properties.find(filter).with_options(options).await?;
        let mut properties = Vec::new();

        while cursor.advance().await? {
            let property = cursor.deserialize_current()?;
            properties.push(property);
        }

        Ok(properties)
    }

    /// Update property click count (for analytics)
    pub async fn increment_property_clicks(&self, property_id: &str) -> Result<(), PropertyError> {
        let object_id = ObjectId::from_str(property_id)
            .map_err(|_| PropertyError::InvalidId)?;

        let update = doc! {
            "$inc": { "clicks": 1 },
            "$push": { 
                "clickHistory": { 
                    "timestamp": utc_to_bson(chrono::Utc::now()),
                    "_id": ObjectId::new()
                }
            }
        };

        self.properties
            .update_one(doc! { "_id": object_id }, update)
            .await?;

        Ok(())
    }

    /// Get properties by owner
    pub async fn get_properties_by_owner(&self, owner_id: &str) -> Result<Vec<Property>, PropertyError> {
        let owner_object_id = ObjectId::from_str(owner_id)
            .map_err(|_| PropertyError::InvalidId)?;

        let filter = doc! { 
            "owner": owner_object_id,
            "removed": { "$ne": true }
        };

        let mut cursor = self.properties.find(filter).await?;
        let mut properties = Vec::new();

        while cursor.advance().await? {
            let property = cursor.deserialize_current()?;
            properties.push(property);
        }

        Ok(properties)
    }

    /// Get recently added properties
    pub async fn get_recent_properties(&self, limit: i64) -> Result<Vec<PropertyQrInfo>, PropertyError> {
        let filter = doc! { "removed": { "$ne": true } };
        let options = FindOptions::builder()
            .limit(limit)
            .sort(doc! { "createdAt": -1 })
            .build();

        let mut cursor = self.properties.find(filter).with_options(options).await?;
        let mut recent_properties = Vec::new();

        while cursor.advance().await? {
            let property: Property = cursor.deserialize_current()?;
            if property.is_qr_eligible() {
                recent_properties.push(property.to_qr_info());
            }
        }

        Ok(recent_properties)
    }

    /// Helper method to determine why a property is not eligible for QR generation
    fn get_ineligibility_reason(&self, property: &Property) -> String {
        if property.removed.unwrap_or(false) {
            return "Property has been removed".to_string();
        }
        
        if property.images.is_empty() {
            return "Property has no images".to_string();
        }
        
        if property.price <= 0 {
            return "Property has invalid price".to_string();
        }
        
        "Unknown reason".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct PropertyStats {
    pub total_properties: i64,
    pub active_properties: i64,
    pub verified_properties: i64,
    pub properties_with_images: i64,
    pub qr_eligible_properties: i64,
}

#[derive(Debug, Clone)]
pub struct PropertySearchCriteria {
    pub location: Option<String>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub property_type: Option<String>,
    pub verified_only: bool,
    pub blockchain_only: bool,
    pub limit: Option<i64>,
}

impl Default for PropertySearchCriteria {
    fn default() -> Self {
        Self {
            location: None,
            min_price: None,
            max_price: None,
            property_type: None,
            verified_only: false,
            blockchain_only: false,
            limit: Some(50),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::Client;

    async fn get_test_service() -> PropertyService {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to connect to MongoDB");
        let db = client.database("test_qr_properties");
        PropertyService::new(&db)
    }

    #[tokio::test]
    async fn test_validate_invalid_property_id() {
        let service = get_test_service().await;
        
        let result = service.validate_property("invalid_id").await;
        assert!(matches!(result, Err(PropertyError::InvalidId)));
    }

    #[tokio::test]
    async fn test_get_property_stats() {
        let service = get_test_service().await;
        
        let stats = service.get_property_stats().await
            .expect("Failed to get property stats");
        
        assert!(stats.total_properties >= 0);
        assert!(stats.active_properties >= 0);
    }

    #[tokio::test]
    async fn test_search_properties_default() {
        let service = get_test_service().await;
        
        let criteria = PropertySearchCriteria::default();
        let properties = service.search_properties(criteria).await
            .expect("Failed to search properties");
        
        // Should not fail, may return empty vec
        assert!(properties.len() >= 0);
    }
}