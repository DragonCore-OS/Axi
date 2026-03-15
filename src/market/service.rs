use std::collections::HashMap;
use uuid::Uuid;

use super::listing::{
    Listing, ListingAvailability, ListingFilter, ListingType, PricingModel, SettlementMode,
};
use super::order::{Order, OrderStatus};

#[derive(Default)]
pub struct MarketService {
    listings: HashMap<Uuid, Listing>,
    orders: HashMap<Uuid, Order>,
    by_type: HashMap<String, Vec<Uuid>>,
}

impl MarketService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_listing(
        &mut self,
        listing_type: ListingType,
        seller_agent_uuid: Uuid,
        title: String,
        description: String,
        tags: Vec<String>,
        pricing_model: PricingModel,
        price_axi: Option<u64>,
        price_per_unit_axi: Option<u64>,
        settlement_mode: SettlementMode,
    ) -> Result<Listing, &'static str> {
        let listing = Listing::new(
            listing_type.clone(),
            seller_agent_uuid,
            title,
            description,
            tags,
            pricing_model,
            price_axi,
            price_per_unit_axi,
            settlement_mode,
        )?;

        self.by_type
            .entry(type_key(&listing_type))
            .or_default()
            .push(listing.listing_id);
        self.listings.insert(listing.listing_id, listing.clone());

        Ok(listing)
    }

    pub fn get_listing(&self, listing_id: &Uuid) -> Option<&Listing> {
        self.listings.get(listing_id)
    }

    pub fn update_listing_availability(
        &mut self,
        listing_id: &Uuid,
        availability: ListingAvailability,
    ) -> Result<(), &'static str> {
        let listing = self
            .listings
            .get_mut(listing_id)
            .ok_or("listing not found")?;
        listing.availability = availability;
        listing.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn delete_listing(&mut self, listing_id: &Uuid) -> Result<(), &'static str> {
        let listing = self.listings.remove(listing_id).ok_or("listing not found")?;
        if let Some(ids) = self.by_type.get_mut(&type_key(&listing.listing_type)) {
            ids.retain(|id| id != listing_id);
        }
        Ok(())
    }

    pub fn search_listings(&self, filter: ListingFilter) -> Vec<Listing> {
        self.listings
            .values()
            .filter(|listing| {
                if let Some(ref t) = filter.listing_type {
                    if &listing.listing_type != t {
                        return false;
                    }
                }

                if let Some(max_price) = filter.max_price_axi {
                    match listing.pricing_model {
                        PricingModel::Fixed => {
                            if listing.price_axi.unwrap_or(u64::MAX) > max_price {
                                return false;
                            }
                        }
                        PricingModel::UsageBased => {
                            if listing.price_per_unit_axi.unwrap_or(u64::MAX) > max_price {
                                return false;
                            }
                        }
                        PricingModel::Quote => {}
                    }
                }

                if let Some(ref tag) = filter.tag {
                    if !listing.tags.iter().any(|t| t == tag) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    pub fn create_order_from_listing(
        &mut self,
        listing_id: &Uuid,
        buyer_agent_uuid: Uuid,
        amount_axi: u64,
    ) -> Result<Order, &'static str> {
        let listing = self
            .listings
            .get(listing_id)
            .ok_or("listing not found")?;

        if listing.availability != ListingAvailability::Available {
            return Err("listing is not available");
        }

        let order = Order::from_listing(listing, buyer_agent_uuid, amount_axi)?;
        self.orders.insert(order.order_id, order.clone());
        Ok(order)
    }

    pub fn get_order(&self, order_id: &Uuid) -> Option<&Order> {
        self.orders.get(order_id)
    }

    pub fn transition_order(
        &mut self,
        order_id: &Uuid,
        next: OrderStatus,
    ) -> Result<(), &'static str> {
        let order = self.orders.get_mut(order_id).ok_or("order not found")?;
        order.transition(next)
    }
}

fn type_key(t: &ListingType) -> String {
    match t {
        ListingType::Service => "service".into(),
        ListingType::Resource => "resource".into(),
        ListingType::Job => "job".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_read_listing() {
        let mut market = MarketService::new();
        let seller = Uuid::new_v4();

        let listing = market
            .create_listing(
                ListingType::Service,
                seller,
                "Inference API".into(),
                "Fast inference service".into(),
                vec!["inference".into(), "gpu".into()],
                PricingModel::Fixed,
                Some(100),
                None,
                SettlementMode::Escrow,
            )
            .unwrap();

        let fetched = market.get_listing(&listing.listing_id).unwrap();
        assert_eq!(fetched.title, "Inference API");
        assert_eq!(fetched.visibility, "public");
    }

    #[test]
    fn search_filters_by_tag_price_and_type() {
        let mut market = MarketService::new();
        let seller = Uuid::new_v4();

        market
            .create_listing(
                ListingType::Service,
                seller,
                "Inference API".into(),
                "Fast inference".into(),
                vec!["inference".into()],
                PricingModel::Fixed,
                Some(100),
                None,
                SettlementMode::Escrow,
            )
            .unwrap();

        market
            .create_listing(
                ListingType::Resource,
                seller,
                "GPU Minutes".into(),
                "Rent a GPU".into(),
                vec!["gpu".into()],
                PricingModel::UsageBased,
                None,
                Some(5),
                SettlementMode::Escrow,
            )
            .unwrap();

        let results = market.search_listings(ListingFilter {
            tag: Some("gpu".into()),
            max_price_axi: Some(10),
            listing_type: Some(ListingType::Resource),
        });

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "GPU Minutes");
    }

    #[test]
    fn create_order_from_listing_locks_amount_and_sets_open() {
        let mut market = MarketService::new();
        let seller = Uuid::new_v4();
        let buyer = Uuid::new_v4();

        let listing = market
            .create_listing(
                ListingType::Service,
                seller,
                "Code Review".into(),
                "Review Rust code".into(),
                vec!["code".into()],
                PricingModel::Fixed,
                Some(250),
                None,
                SettlementMode::Escrow,
            )
            .unwrap();

        let order = market
            .create_order_from_listing(&listing.listing_id, buyer, 250)
            .unwrap();

        assert_eq!(order.seller_agent_uuid, seller);
        assert_eq!(order.buyer_agent_uuid, buyer);
        assert_eq!(order.amount_axi, 250);
        assert_eq!(order.amount_locked_axi, 250);
        assert_eq!(order.status, OrderStatus::Open);
    }

    #[test]
    fn order_state_machine_runs_open_to_verified() {
        let mut market = MarketService::new();
        let seller = Uuid::new_v4();
        let buyer = Uuid::new_v4();

        let listing = market
            .create_listing(
                ListingType::Job,
                seller,
                "Dataset cleanup".into(),
                "Clean records".into(),
                vec!["data".into()],
                PricingModel::Fixed,
                Some(300),
                None,
                SettlementMode::Escrow,
            )
            .unwrap();

        let order = market
            .create_order_from_listing(&listing.listing_id, buyer, 300)
            .unwrap();

        market
            .transition_order(&order.order_id, OrderStatus::InProgress)
            .unwrap();
        market
            .transition_order(&order.order_id, OrderStatus::Delivered)
            .unwrap();
        market
            .transition_order(&order.order_id, OrderStatus::Verified)
            .unwrap();

        assert_eq!(
            market.get_order(&order.order_id).unwrap().status,
            OrderStatus::Verified
        );
    }
}
