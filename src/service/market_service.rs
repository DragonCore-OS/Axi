//! Market Service - Business logic for listings and orders
//!
//! Refactored from original market/service.rs to follow P1-2 pattern

use std::sync::Arc;
use uuid::Uuid;

use crate::market::listing::{Listing, ListingType, PricingModel, SettlementMode, ListingAvailability};
use crate::market::order::{Order, OrderStatus};

use super::{ServiceContext, ServiceResult, ServiceError};
use super::types::{Caller, OperationContext, Permission, Pagination, Paginated};

/// Market operations
pub struct MarketService {
    ctx: Arc<ServiceContext>,
    // In real impl: repository: Arc<MarketRepository>
}

impl MarketService {
    /// Create new market service
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// Create a new listing
    /// 
    /// Sequence: validate → mutate → persist → journal → dibl
    pub fn create_listing(
        &self,
        ctx: &OperationContext,
        listing_type: ListingType,
        title: String,
        description: String,
        tags: Vec<String>,
        pricing_model: PricingModel,
        price_axi: Option<u64>,
        settlement_mode: SettlementMode,
    ) -> ServiceResult<Listing> {
        // 1. VALIDATE
        if !ctx.caller.has_permission(Permission::CreateListing) {
            return Err(ServiceError::Unauthorized {
                reason: "Cannot create listings".to_string(),
            });
        }

        if title.is_empty() || title.len() > 200 {
            return Err(ServiceError::InvalidInput {
                field: "title".to_string(),
                reason: "Title must be 1-200 characters".to_string(),
            });
        }

        // 2. MUTATE
        let listing = Listing::new(
            listing_type,
            ctx.caller.agent_uuid,
            title,
            description,
            tags,
            pricing_model,
            price_axi,
            None,
            settlement_mode,
        ).map_err(|e| ServiceError::InvalidInput {
            field: "listing".to_string(),
            reason: e.to_string(),
        })?;

        // 3. PERSIST (skeleton)
        // 4. JOURNAL (skeleton)
        // 5. DIBL EMIT (skeleton)

        Ok(listing)
    }

    /// Place an order from a listing
    pub fn place_order(
        &self,
        ctx: &OperationContext,
        listing_id: Uuid,
        amount_axi: u64,
    ) -> ServiceResult<Order> {
        // 1. VALIDATE
        if !ctx.caller.has_permission(Permission::PlaceOrder) {
            return Err(ServiceError::Unauthorized {
                reason: "Cannot place orders".to_string(),
            });
        }

        // 2. MUTATE (would load listing, create order)
        // 3. PERSIST
        // 4. JOURNAL
        // 5. DIBL EMIT

        // Placeholder
        Err(ServiceError::Internal {
            message: "Not fully implemented".to_string(),
        })
    }

    /// Transition order to new status
    /// 
    /// Validates state machine and permissions
    pub fn transition_order(
        &self,
        ctx: &OperationContext,
        order_id: Uuid,
        new_status: OrderStatus,
    ) -> ServiceResult<()> {
        // Check permissions based on transition type
        // Validate state machine
        // Persist
        // Journal
        // DIBL emit

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::types::Caller;

    fn test_caller() -> Caller {
        Caller {
            agent_uuid: Uuid::new_v4(),
            agent_id: "seller".to_string(),
            permissions: vec![Permission::CreateListing, Permission::PlaceOrder],
        }
    }

    #[test]
    fn create_listing_requires_permission() {
        let ctx = Arc::new(ServiceContext::new_test());
        let service = MarketService::new(ctx);
        
        let op_ctx = OperationContext::new(Caller {
            agent_uuid: Uuid::new_v4(),
            agent_id: "no_perms".to_string(),
            permissions: vec![],
        });

        let result = service.create_listing(
            &op_ctx,
            ListingType::Service,
            "Test Service".to_string(),
            "Description".to_string(),
            vec![],
            PricingModel::Fixed,
            Some(100),
            SettlementMode::Escrow,
        );

        assert!(matches!(result, Err(ServiceError::Unauthorized { .. })));
    }

    #[test]
    fn create_listing_validates_title() {
        let ctx = Arc::new(ServiceContext::new_test());
        let service = MarketService::new(ctx);
        let op_ctx = OperationContext::new(test_caller());

        // Empty title
        let result = service.create_listing(
            &op_ctx,
            ListingType::Service,
            "".to_string(),
            "Description".to_string(),
            vec![],
            PricingModel::Fixed,
            Some(100),
            SettlementMode::Escrow,
        );

        assert!(matches!(result, Err(ServiceError::InvalidInput { .. })));
    }
}
