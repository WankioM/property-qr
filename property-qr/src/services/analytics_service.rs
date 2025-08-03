// src/services/analytics_service.rs

use crate::models::{
    ScanEvent, PropertyScanAnalytics, SystemAnalytics, ScanSource, RedirectType, 
    DeviceInfo, GeoLocation, CountryStats, DailyScanCount, PropertyPerformance,
    QrGenerationStats, PeriodStats, PeriodComparison,
    ScanAnalyticsResponse, SystemAnalyticsResponse
};
use futures_util::stream::TryStreamExt;
use chrono::{DateTime, Utc, Duration, Datelike};
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
    Collection, Database, options::{ReplaceOptions, FindOptions},
};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn, error};

#[derive(Clone)]
pub struct AnalyticsService {
    scan_events: Collection<ScanEvent>,
    property_analytics: Collection<PropertyScanAnalytics>,
    system_analytics: Collection<SystemAnalytics>,
}

// Helper function to convert chrono DateTime to BSON DateTime
fn utc_to_bson(dt: chrono::DateTime<Utc>) -> BsonDateTime {
    BsonDateTime::from_millis(dt.timestamp_millis())
}

// Helper function to convert BSON DateTime to chrono DateTime
fn bson_to_utc(dt: BsonDateTime) -> chrono::DateTime<Utc> {
    chrono::DateTime::from_timestamp_millis(dt.timestamp_millis())
        .unwrap_or_else(|| Utc::now())
}

impl AnalyticsService {
    /// Create a new analytics service
    pub fn new(db: &Database) -> Self {
        Self {
            scan_events: db.collection("scan_events"),
            property_analytics: db.collection("property_analytics"),
            system_analytics: db.collection("system_analytics"),
        }
    }

    /// Record a new scan event
    pub async fn record_scan(
        &self,
        property_id: String,
        qr_version: i32,
        scan_source: ScanSource,
        redirect_type: RedirectType,
        user_agent: Option<String>,
        ip_address: Option<String>,
        session_id: Option<String>,
        referrer: Option<String>,
    ) -> Result<ObjectId, mongodb::error::Error> {
        let start_time = std::time::Instant::now();

        // Parse device info from user agent
        let device_info = user_agent.as_ref()
            .map(|ua| DeviceInfo::from_user_agent(ua));

        // TODO: Get geolocation from IP address (would use external service)
        let geolocation = self.get_geolocation_from_ip(ip_address.as_deref()).await;

        // Create scan event
        let mut scan_event = ScanEvent::new(
            property_id.clone(),
            qr_version,
            scan_source,
            redirect_type,
        )
        .with_request_data(user_agent, ip_address, session_id, referrer);

        if let Some(device_info) = device_info {
            scan_event = scan_event.with_device_info(device_info);
        }

        if let Some(geolocation) = geolocation {
            scan_event = scan_event.with_geolocation(geolocation);
        }

        let response_time = start_time.elapsed().as_millis() as u64;
        scan_event = scan_event.with_response_time(response_time);

        // Insert scan event
        let result = self.scan_events.insert_one(&scan_event).await?;
        let scan_id = result.inserted_id.as_object_id().unwrap();

        // Update property analytics asynchronously
        let analytics_service = self.clone();
        let property_id_clone = property_id.clone();
        let scan_event_clone = scan_event.clone();
        
        tokio::spawn(async move {
            if let Err(e) = analytics_service.update_property_analytics(&property_id_clone, &scan_event_clone).await {
                error!("Failed to update property analytics: {}", e);
            }
        });

        // Update system analytics asynchronously
        let analytics_service = self.clone();
        tokio::spawn(async move {
            if let Err(e) = analytics_service.update_system_analytics().await {
                error!("Failed to update system analytics: {}", e);
            }
        });

        info!("Recorded scan for property {} with ID {}", property_id, scan_id);
        Ok(scan_id)
    }

    /// Record a failed scan attempt
    pub async fn record_failed_scan(
        &self,
        property_id: String,
        qr_version: i32,
        error_reason: String,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<ObjectId, mongodb::error::Error> {
        let device_info = user_agent.as_ref()
            .map(|ua| DeviceInfo::from_user_agent(ua));

        let mut scan_event = ScanEvent::new(
            property_id.clone(),
            qr_version,
            ScanSource::QrCode,
            RedirectType::Failed,
        )
        .with_request_data(user_agent, ip_address, None, None)
        .mark_failed()
        .add_metadata("error_reason".to_string(), Value::String(error_reason));

        if let Some(device_info) = device_info {
            scan_event = scan_event.with_device_info(device_info);
        }

        let result = self.scan_events.insert_one(&scan_event).await?;
        let scan_id = result.inserted_id.as_object_id().unwrap();

        warn!("Recorded failed scan for property {} with ID {}", property_id, scan_id);
        Ok(scan_id)
    }

    /// Get analytics for a specific property
    pub async fn get_property_analytics(
        &self,
        property_id: &str,
        include_recent_scans: bool,
    ) -> Result<ScanAnalyticsResponse, mongodb::error::Error> {
        // Get or create property analytics
        let analytics = match self.property_analytics
            .find_one(doc! { "propertyId": property_id })
            .await?
        {
            Some(analytics) => analytics,
            None => {
                // Create new analytics entry
                let new_analytics = PropertyScanAnalytics::new(property_id.to_string());
                self.property_analytics.insert_one(&new_analytics).await?;
                new_analytics
            }
        };

        // Get recent scans if requested
        let recent_scans = if include_recent_scans {
            self.scan_events
                .find(doc! { 
                    "propertyId": property_id,
                    "scannedAt": { 
                        "$gte": utc_to_bson(Utc::now() - Duration::days(7)) 
                    }
                })
                .await?
                .try_collect()
                .await?
        } else {
            Vec::new()
        };

        Ok(ScanAnalyticsResponse {
            property_id: property_id.to_string(),
            analytics,
            recent_scans,
        })
    }

    /// Get system-wide analytics
    pub async fn get_system_analytics(
        &self,
        include_comparison: bool,
    ) -> Result<SystemAnalyticsResponse, mongodb::error::Error> {
        let system_analytics = self.get_or_create_system_analytics().await?;

        let period_comparison = if include_comparison {
            Some(self.calculate_period_comparison().await?)
        } else {
            None
        };

        Ok(SystemAnalyticsResponse {
            system: system_analytics,
            period_comparison,
        })
    }

    /// Get top performing properties
    pub async fn get_top_performing_properties(
        &self,
        limit: i64,
        days: i64,
    ) -> Result<Vec<PropertyPerformance>, mongodb::error::Error> {
        let since_date = utc_to_bson(Utc::now() - Duration::days(days));

        // Aggregate top properties by scan count
        let pipeline = vec![
            doc! {
                "$match": {
                    "scannedAt": { "$gte": since_date },
                    "redirectSuccess": true
                }
            },
            doc! {
                "$group": {
                    "_id": "$propertyId",
                    "totalScans": { "$sum": 1 },
                    "uniqueScans": { "$addToSet": "$ipAddress" },
                    "successfulScans": {
                        "$sum": { "$cond": ["$redirectSuccess", 1, 0] }
                    }
                }
            },
            doc! {
                "$project": {
                    "propertyId": "$_id",
                    "totalScans": 1,
                    "uniqueScans": { "$size": "$uniqueScans" },
                    "successRate": {
                        "$multiply": [
                            { "$divide": ["$successfulScans", "$totalScans"] },
                            100
                        ]
                    }
                }
            },
            doc! { "$sort": { "totalScans": -1 } },
            doc! { "$limit": limit }
        ];

        let mut cursor = self.scan_events.aggregate(pipeline).await?;
        let mut performances = Vec::new();

        while cursor.advance().await? {
            let doc = cursor.current();
            let performance = PropertyPerformance {
                property_id: doc.get_str("propertyId").unwrap_or("").to_string(),
                property_name: format!("Property {}", doc.get_str("propertyId").unwrap_or("")), // TODO: Get from property service
                total_scans: doc.get_i64("totalScans").unwrap_or(0),
                unique_scans: doc.get_i64("uniqueScans").unwrap_or(0),
                success_rate: doc.get_f64("successRate").unwrap_or(0.0),
            };
            performances.push(performance);
        }

        Ok(performances)
    }

    /// Get scan trends for a property
    pub async fn get_property_scan_trends(
        &self,
        property_id: &str,
        days: i64,
    ) -> Result<Vec<DailyScanCount>, mongodb::error::Error> {
        let since_date = utc_to_bson(Utc::now() - Duration::days(days));

        let pipeline = vec![
            doc! {
                "$match": {
                    "propertyId": property_id,
                    "scannedAt": { "$gte": since_date }
                }
            },
            doc! {
                "$group": {
                    "_id": {
                        "$dateToString": {
                            "format": "%Y-%m-%d",
                            "date": "$scannedAt"
                        }
                    },
                    "count": { "$sum": 1 }
                }
            },
            doc! { "$sort": { "_id": 1 } }
        ];

        let mut cursor = self.scan_events.aggregate(pipeline).await?;
        let mut trends = Vec::new();

        while cursor.advance().await? {
            let doc = cursor.current();
            let trend = DailyScanCount {
                date: doc.get_str("_id").unwrap_or("").to_string(),
                count: doc.get_i64("count").unwrap_or(0),
            };
            trends.push(trend);
        }

        Ok(trends)
    }

    /// Get geographic distribution of scans
    pub async fn get_geographic_distribution(
        &self,
        property_id: Option<&str>,
        days: i64,
    ) -> Result<Vec<CountryStats>, mongodb::error::Error> {
        let since_date = utc_to_bson(Utc::now() - Duration::days(days));
        
        let mut match_doc = doc! {
            "scannedAt": { "$gte": since_date },
            "geolocation.country": { "$exists": true, "$ne": null }
        };

        if let Some(property_id) = property_id {
            match_doc.insert("propertyId", property_id);
        }

        let pipeline = vec![
            doc! { "$match": match_doc },
            doc! {
                "$group": {
                    "_id": "$geolocation.country",
                    "count": { "$sum": 1 }
                }
            },
            doc! { "$sort": { "count": -1 } },
            doc! { "$limit": 20 }
        ];

        let mut cursor = self.scan_events.aggregate(pipeline).await?;
        let mut countries = Vec::new();
        let mut total_count = 0i64;

        // First pass: collect data and calculate total
        let mut temp_countries = Vec::new();
        while cursor.advance().await? {
            let doc = cursor.current();
            let count = doc.get_i64("count").unwrap_or(0);
            total_count += count;
            temp_countries.push((
                doc.get_str("_id").unwrap_or("Unknown").to_string(),
                count,
            ));
        }

        // Second pass: calculate percentages
        for (country, count) in temp_countries {
            let percentage = if total_count > 0 {
                (count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            };

            countries.push(CountryStats {
                country,
                count,
                percentage,
            });
        }

        Ok(countries)
    }

    /// Delete old scan events (data retention)
    pub async fn cleanup_old_events(&self, retention_days: i64) -> Result<u64, mongodb::error::Error> {
        let cutoff_date = utc_to_bson(Utc::now() - Duration::days(retention_days));
        
        let result = self.scan_events
            .delete_many(doc! { "scannedAt": { "$lt": cutoff_date } })
            .await?;

        info!("Cleaned up {} old scan events", result.deleted_count);
        Ok(result.deleted_count)
    }

    /// Update property analytics with new scan event
    async fn update_property_analytics(
        &self,
        property_id: &str,
        scan_event: &ScanEvent,
    ) -> Result<(), mongodb::error::Error> {
        // Try to find existing analytics
        let mut analytics = match self.property_analytics
            .find_one(doc! { "propertyId": property_id })
            .await?
        {
            Some(analytics) => analytics,
            None => PropertyScanAnalytics::new(property_id.to_string()),
        };

        // Update analytics with new scan
        analytics.update_with_scan(scan_event);

        // Update time-based counters
        let now = Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();
        let week_start = today_start - Duration::days(now.weekday().num_days_from_monday() as i64);
        let month_start = now.date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();

        if scan_event.scanned_at >= today_start {
            analytics.scans_today += 1;
        }
        if scan_event.scanned_at >= week_start {
            analytics.scans_this_week += 1;
        }
        if scan_event.scanned_at >= month_start {
            analytics.scans_this_month += 1;
        }

        // Upsert the analytics
        let filter = doc! { "propertyId": property_id };
        let options = ReplaceOptions::builder().upsert(true).build();
        self.property_analytics
            .replace_one(filter, &analytics)
            .with_options(options)
            .await?;

        Ok(())
    }

    /// Update system-wide analytics
    async fn update_system_analytics(&self) -> Result<(), mongodb::error::Error> {
        let mut system_analytics = self.get_or_create_system_analytics().await?;

        // Update counters (this is a simplified version - in production you'd want more efficient aggregations)
        let total_scans = self.scan_events
            .count_documents(doc! {})
            .await? as i64;

        let today_start = utc_to_bson(Utc::now().date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap());

        let scans_today = self.scan_events
            .count_documents(doc! { "scannedAt": { "$gte": today_start } })
            .await? as i64;

        system_analytics.total_scans_all_time = total_scans;
        system_analytics.total_scans_today = scans_today;
        system_analytics.last_updated = Utc::now();

        // Update top performing properties
        system_analytics.top_performing_properties = self.get_top_performing_properties(10, 30).await?;

        // Upsert system analytics
        let filter = doc! {};
        let options = ReplaceOptions::builder().upsert(true).build();
        self.system_analytics
            .replace_one(filter, &system_analytics)
            .with_options(options)
            .await?;

        Ok(())
    }

    /// Get or create system analytics
    async fn get_or_create_system_analytics(&self) -> Result<SystemAnalytics, mongodb::error::Error> {
        match self.system_analytics.find_one(doc! {}).await? {
            Some(analytics) => Ok(analytics),
            None => {
                let new_analytics = SystemAnalytics {
                    id: ObjectId::new(),
                    total_properties: 0,
                    properties_with_qr: 0,
                    total_scans_all_time: 0,
                    total_scans_today: 0,
                    total_scans_this_week: 0,
                    total_scans_this_month: 0,
                    average_scans_per_property: 0.0,
                    top_performing_properties: Vec::new(),
                    qr_generation_stats: QrGenerationStats {
                        total_generated: 0,
                        generated_today: 0,
                        generated_this_week: 0,
                        generated_this_month: 0,
                        average_generation_time: None,
                        failure_rate: 0.0,
                    },
                    last_updated: Utc::now(),
                };
                
                self.system_analytics.insert_one(&new_analytics).await?;
                Ok(new_analytics)
            }
        }
    }

    /// Calculate period comparison (current vs previous period)
    async fn calculate_period_comparison(&self) -> Result<PeriodComparison, mongodb::error::Error> {
        let now = Utc::now();
        let thirty_days_ago = now - Duration::days(30);
        let sixty_days_ago = now - Duration::days(60);

        // Current period (last 30 days)
        let current_scans = self.scan_events
            .count_documents(doc! { "scannedAt": { "$gte": utc_to_bson(thirty_days_ago) } })
            .await? as i64;

        // Previous period (30-60 days ago)
        let previous_scans = self.scan_events
            .count_documents(doc! { 
                "scannedAt": { 
                    "$gte": utc_to_bson(sixty_days_ago),
                    "$lt": utc_to_bson(thirty_days_ago)
                } 
            })
            .await? as i64;

        let percentage_change = if previous_scans > 0 {
            ((current_scans - previous_scans) as f64 / previous_scans as f64) * 100.0
        } else {
            0.0
        };

        Ok(PeriodComparison {
            current_period: PeriodStats {
                total_scans: current_scans,
                unique_scans: current_scans, // Simplified
                average_scans_per_day: current_scans as f64 / 30.0,
            },
            previous_period: PeriodStats {
                total_scans: previous_scans,
                unique_scans: previous_scans, // Simplified
                average_scans_per_day: previous_scans as f64 / 30.0,
            },
            percentage_change,
        })
    }

    /// Get geolocation from IP address (placeholder - would use external service)
    async fn get_geolocation_from_ip(&self, _ip_address: Option<&str>) -> Option<GeoLocation> {
        // TODO: Implement with external geolocation service like MaxMind or ipapi
        // For now, return None
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::Client;

    async fn get_test_service() -> AnalyticsService {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to connect to MongoDB");
        let db = client.database("test_qr_analytics");
        AnalyticsService::new(&db)
    }

    #[tokio::test]
    async fn test_record_scan() {
        let service = get_test_service().await;
        
        let scan_id = service.record_scan(
            "test_property_123".to_string(),
            1,
            ScanSource::QrCode,
            RedirectType::DualRedirect,
            Some("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)".to_string()),
            Some("192.168.1.1".to_string()),
            Some("session_123".to_string()),
            None,
        ).await.expect("Failed to record scan");

        assert!(scan_id.to_hex().len() > 0);
    }

    #[tokio::test]
    async fn test_get_property_analytics() {
        let service = get_test_service().await;
        
        // Record a scan first
        service.record_scan(
            "test_property_456".to_string(),
            1,
            ScanSource::QrCode,
            RedirectType::DualRedirect,
            None,
            None,
            None,
            None,
        ).await.expect("Failed to record scan");

        // Get analytics
        let analytics = service.get_property_analytics("test_property_456", true)
            .await
            .expect("Failed to get analytics");

        assert_eq!(analytics.property_id, "test_property_456");
    }
}