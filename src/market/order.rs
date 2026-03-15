use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::listing::Listing;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Open,
    InProgress,
    Delivered,
    Verified,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: Uuid,
    pub listing_id: Uuid,
    pub buyer_agent_uuid: Uuid,
    pub seller_agent_uuid: Uuid,
    pub amount_axi: u64,
    pub amount_locked_axi: u64,
    pub status: OrderStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl Order {
    pub fn from_listing(
        listing: &Listing,
        buyer_agent_uuid: Uuid,
        amount_axi: u64,
    ) -> Result<Self, &'static str> {
        if amount_axi == 0 {
            return Err("order amount must be > 0");
        }

        let now = Utc::now().to_rfc3339();

        Ok(Self {
            order_id: Uuid::new_v4(),
            listing_id: listing.listing_id,
            buyer_agent_uuid,
            seller_agent_uuid: listing.seller_agent_uuid,
            amount_axi,
            amount_locked_axi: amount_axi,
            status: OrderStatus::Open,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn transition(&mut self, next: OrderStatus) -> Result<(), &'static str> {
        let valid = matches!(
            (&self.status, &next),
            (OrderStatus::Open, OrderStatus::InProgress)
                | (OrderStatus::Open, OrderStatus::Failed)
                | (OrderStatus::InProgress, OrderStatus::Delivered)
                | (OrderStatus::InProgress, OrderStatus::Failed)
                | (OrderStatus::Delivered, OrderStatus::Verified)
                | (OrderStatus::Delivered, OrderStatus::Failed)
        );

        if !valid {
            return Err("invalid order state transition");
        }

        self.status = next;
        self.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }
}
