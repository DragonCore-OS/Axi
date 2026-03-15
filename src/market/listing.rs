use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ListingType {
    Service,
    Resource,
    Job,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PricingModel {
    Fixed,
    Quote,
    UsageBased,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SettlementMode {
    Direct,
    Escrow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ListingAvailability {
    Available,
    Busy,
    Paused,
    Delisted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub listing_id: Uuid,
    pub listing_type: ListingType,
    pub seller_agent_uuid: Uuid,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub pricing_model: PricingModel,
    pub price_axi: Option<u64>,
    pub price_per_unit_axi: Option<u64>,
    pub visibility: String,
    pub settlement_mode: SettlementMode,
    pub availability: ListingAvailability,
    pub created_at: String,
    pub updated_at: String,
}

impl Listing {
    pub fn new(
        listing_type: ListingType,
        seller_agent_uuid: Uuid,
        title: String,
        description: String,
        tags: Vec<String>,
        pricing_model: PricingModel,
        price_axi: Option<u64>,
        price_per_unit_axi: Option<u64>,
        settlement_mode: SettlementMode,
    ) -> Result<Self, &'static str> {
        validate_pricing(&pricing_model, price_axi, price_per_unit_axi)?;

        let now = Utc::now().to_rfc3339();

        Ok(Self {
            listing_id: Uuid::new_v4(),
            listing_type,
            seller_agent_uuid,
            title,
            description,
            tags,
            pricing_model,
            price_axi,
            price_per_unit_axi,
            visibility: "public".into(),
            settlement_mode,
            availability: ListingAvailability::Available,
            created_at: now.clone(),
            updated_at: now,
        })
    }
}

fn validate_pricing(
    pricing_model: &PricingModel,
    price_axi: Option<u64>,
    price_per_unit_axi: Option<u64>,
) -> Result<(), &'static str> {
    match pricing_model {
        PricingModel::Fixed => {
            if price_axi.is_none() {
                return Err("fixed pricing requires price_axi");
            }
        }
        PricingModel::Quote => {
            if price_axi.is_some() || price_per_unit_axi.is_some() {
                return Err("quote pricing must not set fixed or per-unit price");
            }
        }
        PricingModel::UsageBased => {
            if price_per_unit_axi.is_none() {
                return Err("usage_based pricing requires price_per_unit_axi");
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct ListingFilter {
    pub tag: Option<String>,
    pub max_price_axi: Option<u64>,
    pub listing_type: Option<ListingType>,
}
