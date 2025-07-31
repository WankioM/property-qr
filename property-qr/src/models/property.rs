 // src/models/property.rs

use chrono::{DateTime, Utc};
use mongodb::bson::{oid::ObjectId, doc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub owner: ObjectId,
    #[serde(rename = "propertyName")]
    pub property_name: String,
    pub location: String,
    pub coordinates: Coordinates,
    #[serde(rename = "streetAddress")]
    pub street_address: String,
    #[serde(rename = "googleMapsURL")]
    pub google_maps_url: Option<String>,
    #[serde(rename = "propertyType")]
    pub property_type: PropertyType,
    #[serde(rename = "specificType")]
    pub specific_type: String,
    #[serde(rename = "unitNo")]
    pub unit_no: Option<String>,
    pub status: PropertyStatus,
    pub action: String,
    pub price: i64,
    pub space: i32,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub features: Vec<String>,
    pub security: String,
    pub amenities: Amenities,
    #[serde(rename = "additionalComments")]
    pub additional_comments: Option<String>,
    pub rooms: Option<i32>,
    #[serde(rename = "cryptoAccepted")]
    pub crypto_accepted: bool,
    pub images: Vec<String>,
    pub clicks: i32,
    #[serde(rename = "clickHistory")]
    pub click_history: Vec<ClickHistoryItem>,
    #[serde(rename = "wishlistCount")]
    pub wishlist_count: i32,
    #[serde(rename = "wishlistHistory")]
    pub wishlist_history: Vec<WishlistHistoryItem>,
    #[serde(rename = "popularityScore")]
    pub popularity_score: f64,
    
    // Blockchain and ownership fields
    #[serde(rename = "onchainId")]
    pub onchain_id: Option<String>,
    #[serde(rename = "coOwned")]
    pub co_owned: bool,
    #[serde(rename = "coOwners")]
    pub co_owners: Vec<CoOwner>,
    #[serde(rename = "availableShares")]
    pub available_shares: i32,
    pub blockchain: Option<BlockchainInfo>,
    
    // Activity and management
    pub transactions: Vec<Transaction>,
    pub bookings: Vec<Booking>,
    pub proposals: Vec<Proposal>,
    pub featured: Featured,
    pub reports: Option<Vec<Report>>,
    pub documents: Option<Vec<Document>>,
    
    // Base Names fields
    #[serde(rename = "baseName")]
    pub base_name: Option<String>,
    #[serde(rename = "baseNamePending")]
    pub base_name_pending: Option<bool>,
    #[serde(rename = "baseNameConfirmed")]
    pub base_name_confirmed: Option<bool>,
    #[serde(rename = "baseNameRequestedAt")]
    pub base_name_requested_at: Option<DateTime<Utc>>,
    #[serde(rename = "baseNameConfirmedAt")]
    pub base_name_confirmed_at: Option<DateTime<Utc>>,
    #[serde(rename = "baseNameTransactionHash")]
    pub base_name_transaction_hash: Option<String>,
    
    // Soft deletion and verification
    pub removed: Option<bool>,
    #[serde(rename = "removedBy")]
    pub removed_by: Option<ObjectId>,
    #[serde(rename = "removedAt")]
    pub removed_at: Option<DateTime<Utc>>,
    #[serde(rename = "removedReason")]
    pub removed_reason: Option<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: Option<bool>,
    #[serde(rename = "verifiedBy")]
    pub verified_by: Option<ObjectId>,
    #[serde(rename = "verifiedAt")]
    pub verified_at: Option<DateTime<Utc>>,
    
    // Timestamps
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "__v")]
    pub version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PropertyType {
    Residential,
    Commercial,
    Land,
    #[serde(rename = "Special-purpose")]
    SpecialPurpose,
    #[serde(rename = "Vacation/Short-term rentals")]
    VacationShortTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyStatus {
    pub sold: bool,
    pub occupied: bool,
    #[serde(rename = "listingState")]
    pub listing_state: ListingState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListingState {
    #[serde(rename = "simply listed")]
    SimplyListed,
    #[serde(rename = "requested financing")]
    RequestedFinancing,
    #[serde(rename = "in marketplace waiting for financing")]
    InMarketplaceWaitingForFinancing,
    #[serde(rename = "accepted for collateral")]
    AcceptedForCollateral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amenities {
    pub furnished: bool,
    pub pool: bool,
    pub gym: bool,
    pub garden: bool,
    pub parking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickHistoryItem {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishlistHistoryItem {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoOwner {
    #[serde(rename = "userId")]
    pub user_id: ObjectId,
    #[serde(rename = "percentageOwned")]
    pub percentage_owned: f64,
    #[serde(rename = "investmentDate")]
    pub investment_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub registered: bool,
    #[serde(rename = "registeredAt")]
    pub registered_at: Option<DateTime<Utc>>,
    pub verified: bool,
    #[serde(rename = "verifiedAt")]
    pub verified_at: Option<DateTime<Utc>>,
    #[serde(rename = "verifiedBy")]
    pub verified_by: Option<String>,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<String>,
    #[serde(rename = "sbtId")]
    pub sbt_id: Option<String>,
    #[serde(rename = "ownershipTokenId")]
    pub ownership_token_id: Option<String>,
    #[serde(rename = "zkProof")]
    pub zk_proof: Option<String>,
    #[serde(rename = "lastUpdatedOnChain")]
    pub last_updated_on_chain: Option<DateTime<Utc>>,
    #[serde(rename = "ownerWalletAddress")]
    pub owner_wallet_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "txId")]
    pub tx_id: String,
    #[serde(rename = "buyerId")]
    pub buyer_id: ObjectId,
    pub amount: f64,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "paymentMethod")]
    pub payment_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    #[serde(rename = "renterId")]
    pub renter_id: ObjectId,
    #[serde(rename = "checkIn")]
    pub check_in: DateTime<Utc>,
    #[serde(rename = "checkOut")]
    pub check_out: DateTime<Utc>,
    pub status: BookingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookingStatus {
    Pending,
    Confirmed,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: ProposalStatus,
    pub votes: Vec<Vote>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProposalStatus {
    Proposed,
    Voting,
    Approved,
    Rejected,
    Implemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    #[serde(rename = "userId")]
    pub user_id: ObjectId,
    pub vote: VoteType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoteType {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Featured {
    #[serde(rename = "isFeatured")]
    pub is_featured: bool,
    #[serde(rename = "featuredBy")]
    pub featured_by: Option<ObjectId>,
    #[serde(rename = "featuredAt")]
    pub featured_at: Option<DateTime<Utc>>,
    #[serde(rename = "priorityLevel")]
    pub priority_level: i32, // 1 = Top Featured, 2 = Trending, 3 = Recommended
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "reportedBy")]
    pub reported_by: ObjectId,
    pub reason: String,
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
    #[serde(rename = "resolvedBy")]
    pub resolved_by: Option<ObjectId>,
    #[serde(rename = "resolvedAt")]
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "documentId")]
    pub document_id: String,
    #[serde(rename = "documentType")]
    pub document_type: DocumentType,
    #[serde(rename = "documentName")]
    pub document_name: String,
    #[serde(rename = "documentUrl")]
    pub document_url: String,
    #[serde(rename = "fileType")]
    pub file_type: String,
    #[serde(rename = "uploadDate")]
    pub upload_date: DateTime<Utc>,
    #[serde(rename = "expiryDate")]
    pub expiry_date: Option<DateTime<Utc>>,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "verifiedBy")]
    pub verified_by: Option<ObjectId>,
    #[serde(rename = "verifiedAt")]
    pub verified_at: Option<DateTime<Utc>>,
    #[serde(rename = "verificationNotes")]
    pub verification_notes: Option<String>,
    #[serde(rename = "documentHash")]
    pub document_hash: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentType {
    TitleDeed,
    CertificateOfLease,
    RatesClearance,
    LandRentClearance,
    UtilityBill,
    CourtJudgment,
    Probate,
    ConsentToTransfer,
    SaleAgreement,
    StampDutyReceipt,
    AdversePossession,
    Affidavit,
    Other,
}

// Simplified property for QR generation (only essential fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyQrInfo {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "propertyName")]
    pub property_name: String,
    pub location: String,
    pub action: String,
    pub price: i64,
    #[serde(rename = "onchainId")]
    pub onchain_id: Option<String>,
    #[serde(rename = "cryptoAccepted")]
    pub crypto_accepted: bool,
    pub images: Vec<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: Option<bool>,
    pub removed: Option<bool>,
}

impl Property {
    /// Check if property is available for QR generation
    pub fn is_qr_eligible(&self) -> bool {
        // Property should not be removed
        if self.removed.unwrap_or(false) {
            return false;
        }
        
        // Property should have at least one image
        if self.images.is_empty() {
            return false;
        }
        
        // Property should have a valid price
        if self.price <= 0 {
            return false;
        }
        
        true
    }
    
    /// Get essential info for QR code generation
    pub fn to_qr_info(&self) -> PropertyQrInfo {
        PropertyQrInfo {
            id: self.id,
            property_name: self.property_name.clone(),
            location: self.location.clone(),
            action: self.action.clone(),
            price: self.price,
            onchain_id: self.onchain_id.clone(),
            crypto_accepted: self.crypto_accepted,
            images: self.images.clone(),
            is_verified: self.is_verified,
            removed: self.removed,
        }
    }
    
    /// Get the primary property image URL
    pub fn get_primary_image(&self) -> Option<&String> {
        self.images.first()
    }
    
    /// Check if property has blockchain registration
    pub fn has_blockchain_info(&self) -> bool {
        self.onchain_id.is_some() || 
        (self.blockchain.is_some() && self.blockchain.as_ref().unwrap().registered)
    }
    
    /// Get formatted price string
    pub fn get_formatted_price(&self) -> String {
        if self.crypto_accepted {
            format!("KES {} (Crypto Accepted)", self.price)
        } else {
            format!("KES {}", self.price)
        }
    }
}

impl Default for Property {
    fn default() -> Self {
        Self {
            id: ObjectId::new(),
            owner: ObjectId::new(),
            property_name: String::new(),
            location: String::new(),
            coordinates: Coordinates { lat: 0.0, lng: 0.0 },
            street_address: String::new(),
            google_maps_url: None,
            property_type: PropertyType::Residential,
            specific_type: String::new(),
            unit_no: None,
            status: PropertyStatus {
                sold: false,
                occupied: false,
                listing_state: ListingState::SimplyListed,
            },
            action: String::new(),
            price: 0,
            space: 0,
            bedrooms: None,
            bathrooms: None,
            features: Vec::new(),
            security: String::new(),
            amenities: Amenities {
                furnished: false,
                pool: false,
                gym: false,
                garden: false,
                parking: false,
            },
            additional_comments: None,
            rooms: None,
            crypto_accepted: false,
            images: Vec::new(),
            clicks: 0,
            click_history: Vec::new(),
            wishlist_count: 0,
            wishlist_history: Vec::new(),
            popularity_score: 0.0,
            onchain_id: None,
            co_owned: false,
            co_owners: Vec::new(),
            available_shares: 0,
            blockchain: None,
            transactions: Vec::new(),
            bookings: Vec::new(),
            proposals: Vec::new(),
            featured: Featured {
                is_featured: false,
                featured_by: None,
                featured_at: None,
                priority_level: 3,
            },
            reports: None,
            documents: None,
            base_name: None,
            base_name_pending: None,
            base_name_confirmed: None,
            base_name_requested_at: None,
            base_name_confirmed_at: None,
            base_name_transaction_hash: None,
            removed: None,
            removed_by: None,
            removed_at: None,
            removed_reason: None,
            is_verified: None,
            verified_by: None,
            verified_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: None,
        }
    }
}
