//! Secure Reputation System (P1-1 Fix)
//!
//! Validates all reputation events against order state.
//! Prevents forgery via DB unique constraints and service-layer validation.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::registry::AgentRegistry;
use crate::market::order::{Order, OrderStatus};
use crate::market::escrow::EscrowStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReputationEventType {
    OrderCompleted,
    PositiveRating,
    NegativeRating,
    DisputeLost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub event_id: Uuid,
    pub agent_uuid: Uuid,
    pub order_id: Option<Uuid>,
    pub event_type: ReputationEventType,
    pub delta: i64,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReputationDelta(pub i64);

/// Order provider trait for dependency injection
pub trait OrderProvider {
    fn get_order(&self, order_id: &Uuid) -> Result<Option<Order>, String>;
    fn get_order_escrow_status(&self, order_id: &Uuid) -> Result<Option<EscrowStatus>, String>;
}

/// Secure reputation service with validation
pub struct ReputationService<P: OrderProvider> {
    history: HashMap<Uuid, Vec<ReputationEvent>>,
    order_provider: P,
}

impl<P: OrderProvider> ReputationService<P> {
    pub fn new(order_provider: P) -> Self {
        Self {
            history: HashMap::new(),
            order_provider,
        }
    }

    /// Calculate delta based on event type and rating
    pub fn calculate_delta(event_type: &ReputationEventType, rating: Option<u8>) -> ReputationDelta {
        match event_type {
            ReputationEventType::OrderCompleted => ReputationDelta(5),
            ReputationEventType::PositiveRating => {
                if matches!(rating, Some(4 | 5)) {
                    ReputationDelta(2)
                } else {
                    ReputationDelta(0)
                }
            }
            ReputationEventType::NegativeRating => {
                if matches!(rating, Some(1 | 2)) {
                    ReputationDelta(-5)
                } else {
                    ReputationDelta(0)
                }
            }
            ReputationEventType::DisputeLost => ReputationDelta(-10),
        }
    }

    /// Record reputation event with full validation
    /// 
    /// # Validation Rules (P1-1 Fix)
    /// 1. Order must exist
    /// 2. Agent must be buyer or seller in the order
    /// 3. Order must be in Verified state for OrderCompleted
    /// 4. Escrow must be Released for OrderCompleted
    /// 5. No duplicate events for same (agent, order, type)
    pub fn record_event(
        &mut self,
        registry: &mut AgentRegistry,
        agent_uuid: Uuid,
        order_id: Option<Uuid>,
        event_type: ReputationEventType,
        rating: Option<u8>,
        reason: String,
    ) -> Result<ReputationEvent, String> {
        // Non-order events (rare, but possible)
        let order_id = match order_id {
            Some(id) => id,
            None => {
                // Only certain event types can be order-less
                if !matches!(event_type, ReputationEventType::DisputeLost) {
                    return Err("order_id required for this event type".into());
                }
                // For order-less events, use system-only ID
                let event = self.create_event(agent_uuid, None, event_type, rating, reason)?;
                self.apply_event(registry, agent_uuid, event.delta, &event)?;
                return Ok(event);
            }
        };

        // P1-1 Fix: Validate order exists
        let order = self.order_provider.get_order(&order_id)?
            .ok_or_else(|| format!("order {} not found", order_id))?;

        // P1-1 Fix: Validate agent is participant in order
        if order.buyer_agent_uuid != agent_uuid && order.seller_agent_uuid != agent_uuid {
            return Err(format!(
                "agent {} is not a participant in order {}",
                agent_uuid, order_id
            ));
        }

        // P1-1 Fix: Validate order state for OrderCompleted
        if matches!(event_type, ReputationEventType::OrderCompleted) {
            if order.status != OrderStatus::Verified {
                return Err(format!(
                    "order {} not in Verified state (current: {:?})",
                    order_id, order.status
                ));
            }

            // Validate escrow is released
            let escrow_status = self.order_provider.get_order_escrow_status(&order_id)?;
            match escrow_status {
                Some(EscrowStatus::Released) => {}
                Some(other) => {
                    return Err(format!(
                        "escrow not released for order {} (current: {:?})",
                        order_id, other
                    ));
                }
                None => {
                    return Err(format!("no escrow found for order {}", order_id));
                }
            }
        }

        // P1-1 Fix: Check for duplicates (in-memory check, DB has unique constraint)
        if let Some(events) = self.history.get(&agent_uuid) {
            if events.iter().any(|e| {
                e.order_id == Some(order_id) && e.event_type == event_type
            }) {
                return Err(format!(
                    "duplicate event: agent {} already has {:?} for order {}",
                    agent_uuid, event_type, order_id
                ));
            }
        }

        let event = self.create_event(agent_uuid, Some(order_id), event_type, rating, reason)?;
        self.apply_event(registry, agent_uuid, event.delta, &event)?;
        
        Ok(event)
    }

    fn create_event(
        &self,
        agent_uuid: Uuid,
        order_id: Option<Uuid>,
        event_type: ReputationEventType,
        rating: Option<u8>,
        reason: String,
    ) -> Result<ReputationEvent, String> {
        let delta = Self::calculate_delta(&event_type, rating).0;

        Ok(ReputationEvent {
            event_id: Uuid::new_v4(),
            agent_uuid,
            order_id,
            event_type,
            delta,
            reason,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    fn apply_event(
        &mut self,
        registry: &mut AgentRegistry,
        agent_uuid: Uuid,
        delta: i64,
        event: &ReputationEvent,
    ) -> Result<(), String> {
        registry.apply_reputation_delta(&agent_uuid, delta)?;
        self.history.entry(agent_uuid).or_default().push(event.clone());
        Ok(())
    }

    pub fn history_for(&self, agent_uuid: &Uuid) -> Vec<ReputationEvent> {
        self.history.get(agent_uuid).cloned().unwrap_or_default()
    }
}

/// Database-backed order provider
pub struct DbOrderProvider<'a> {
    conn: &'a Connection,
}

impl<'a> DbOrderProvider<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> OrderProvider for DbOrderProvider<'a> {
    fn get_order(&self, order_id: &Uuid) -> Result<Option<Order>, String> {
        self.conn.query_row(
            "SELECT order_id, listing_id, buyer_agent_uuid, seller_agent_uuid, 
                    amount_axi, amount_locked_axi, status, created_at, updated_at 
             FROM orders WHERE order_id = ?1",
            params![order_id.to_string()],
            |row| {
                Ok(Order {
                    order_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    listing_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    buyer_agent_uuid: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap(),
                    seller_agent_uuid: Uuid::parse_str(&row.get::<_, String>(3)?).unwrap(),
                    amount_axi: row.get::<_, i64>(4)? as u64,
                    amount_locked_axi: row.get::<_, i64>(5)? as u64,
                    status: parse_order_status(&row.get::<_, String>(6)?),
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        ).optional().map_err(|e| e.to_string())
    }

    fn get_order_escrow_status(&self, order_id: &Uuid) -> Result<Option<EscrowStatus>, String> {
        self.conn.query_row(
            "SELECT escrow_status FROM escrows WHERE order_id = ?1",
            params![order_id.to_string()],
            |row| {
                let status_str: String = row.get(0)?;
                Ok(parse_escrow_status(&status_str))
            },
        ).optional().map_err(|e| e.to_string())
    }
}

fn parse_order_status(s: &str) -> OrderStatus {
    match s {
        "InProgress" => OrderStatus::InProgress,
        "Delivered" => OrderStatus::Delivered,
        "Verified" => OrderStatus::Verified,
        "Failed" => OrderStatus::Failed,
        _ => OrderStatus::Open,
    }
}

fn parse_escrow_status(s: &str) -> EscrowStatus {
    use crate::market::escrow::EscrowStatus;
    match s {
        "Funded" => EscrowStatus::Funded,
        "InEscrow" => EscrowStatus::InEscrow,
        "Released" => EscrowStatus::Released,
        "Cancelled" => EscrowStatus::Cancelled,
        "Refunded" => EscrowStatus::Refunded,
        "Disputed" => EscrowStatus::Disputed,
        _ => EscrowStatus::Pending,
    }
}

/// Mock order provider for testing
#[cfg(test)]
pub struct MockOrderProvider {
    orders: HashMap<Uuid, Order>,
    escrow_statuses: HashMap<Uuid, EscrowStatus>,
}

#[cfg(test)]
impl MockOrderProvider {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
            escrow_statuses: HashMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        self.orders.insert(order.order_id, order);
    }

    pub fn set_escrow_status(&mut self, order_id: Uuid, status: EscrowStatus) {
        self.escrow_statuses.insert(order_id, status);
    }
}

#[cfg(test)]
impl OrderProvider for MockOrderProvider {
    fn get_order(&self, order_id: &Uuid) -> Result<Option<Order>, String> {
        Ok(self.orders.get(order_id).cloned())
    }

    fn get_order_escrow_status(&self, order_id: &Uuid) -> Result<Option<EscrowStatus>, String> {
        Ok(self.escrow_statuses.get(order_id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::registry::AgentRegistry;

    fn make_agent(registry: &mut AgentRegistry, slug: &str) -> Uuid {
        registry
            .create_agent(
                slug.into(),
                slug.into(),
                "pk".into(),
                "cmp".into(),
                "rec".into(),
            )
            .unwrap()
            .agent_uuid
    }

    fn create_verified_order(buyer: Uuid, seller: Uuid) -> Order {
        Order {
            order_id: Uuid::new_v4(),
            listing_id: Uuid::new_v4(),
            buyer_agent_uuid: buyer,
            seller_agent_uuid: seller,
            amount_axi: 100,
            amount_locked_axi: 100,
            status: OrderStatus::Verified,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn order_completed_requires_verified_state() {
        let mut registry = AgentRegistry::new();
        let seller = make_agent(&mut registry, "seller-1");
        let buyer = make_agent(&mut registry, "buyer-1");

        let order_id = Uuid::new_v4();
        let mut mock_provider = MockOrderProvider::new();
        
        // Order in Open state (not Verified)
        let order = Order {
            order_id,
            listing_id: Uuid::new_v4(),
            buyer_agent_uuid: buyer,
            seller_agent_uuid: seller,
            amount_axi: 100,
            amount_locked_axi: 100,
            status: OrderStatus::Open, // Wrong state!
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        mock_provider.add_order(order);
        mock_provider.set_escrow_status(order_id, EscrowStatus::Released);

        let mut reputation = ReputationService::new(mock_provider);

        let result = reputation.record_event(
            &mut registry,
            seller,
            Some(order_id),
            ReputationEventType::OrderCompleted,
            None,
            "completed order".into(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in Verified state"));
    }

    #[test]
    fn order_completed_requires_released_escrow() {
        let mut registry = AgentRegistry::new();
        let seller = make_agent(&mut registry, "seller-2");
        let buyer = make_agent(&mut registry, "buyer-2");

        let mut mock_provider = MockOrderProvider::new();
        
        let order = create_verified_order(buyer, seller);
        let order_id = order.order_id;
        mock_provider.add_order(order.clone());
        // Escrow still in InEscrow (not Released)
        mock_provider.set_escrow_status(order_id, EscrowStatus::InEscrow);

        let mut reputation = ReputationService::new(mock_provider);

        let result = reputation.record_event(
            &mut registry,
            seller,
            Some(order_id),
            ReputationEventType::OrderCompleted,
            None,
            "completed order".into(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("escrow not released"));
    }

    #[test]
    fn non_participant_cannot_record_reputation() {
        let mut registry = AgentRegistry::new();
        let seller = make_agent(&mut registry, "seller-3");
        let buyer = make_agent(&mut registry, "buyer-3");
        let outsider = make_agent(&mut registry, "outsider");

        let order = create_verified_order(buyer, seller);
        let order_id = order.order_id;
        let mut mock_provider = MockOrderProvider::new();
        mock_provider.add_order(order);
        mock_provider.set_escrow_status(order_id, EscrowStatus::Released);

        let mut reputation = ReputationService::new(mock_provider);

        // Outsider tries to record reputation
        let result = reputation.record_event(
            &mut registry,
            outsider,
            Some(order_id),
            ReputationEventType::OrderCompleted,
            None,
            "completed order".into(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a participant"));
    }

    #[test]
    fn fake_order_id_rejected() {
        let mut registry = AgentRegistry::new();
        let seller = make_agent(&mut registry, "seller-4");

        let fake_order_id = Uuid::new_v4();
        let mock_provider = MockOrderProvider::new();
        // No order added - order not found

        let mut reputation = ReputationService::new(mock_provider);

        let result = reputation.record_event(
            &mut registry,
            seller,
            Some(fake_order_id),
            ReputationEventType::OrderCompleted,
            None,
            "completed order".into(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn duplicate_event_rejected() {
        let mut registry = AgentRegistry::new();
        let seller = make_agent(&mut registry, "seller-5");
        let buyer = make_agent(&mut registry, "buyer-5");

        let order = create_verified_order(buyer, seller);
        let order_id = order.order_id;
        let mut mock_provider = MockOrderProvider::new();
        mock_provider.add_order(order);
        mock_provider.set_escrow_status(order_id, EscrowStatus::Released);

        let mut reputation = ReputationService::new(mock_provider);

        // First event succeeds
        let result1 = reputation.record_event(
            &mut registry,
            seller,
            Some(order_id),
            ReputationEventType::OrderCompleted,
            None,
            "completed order".into(),
        );
        assert!(result1.is_ok());

        // Duplicate event fails
        let result2 = reputation.record_event(
            &mut registry,
            seller,
            Some(order_id),
            ReputationEventType::OrderCompleted,
            None,
            "completed order again".into(),
        );
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("duplicate"));
    }

    #[test]
    fn valid_order_completed_succeeds() {
        let mut registry = AgentRegistry::new();
        let seller = make_agent(&mut registry, "seller-6");
        let buyer = make_agent(&mut registry, "buyer-6");

        let order = create_verified_order(buyer, seller);
        let order_id = order.order_id;
        let mut mock_provider = MockOrderProvider::new();
        mock_provider.add_order(order);
        mock_provider.set_escrow_status(order_id, EscrowStatus::Released);

        let mut reputation = ReputationService::new(mock_provider);

        let event = reputation.record_event(
            &mut registry,
            seller,
            Some(order_id),
            ReputationEventType::OrderCompleted,
            None,
            "completed order".into(),
        ).unwrap();

        assert_eq!(event.delta, 5);
        assert_eq!(registry.get_by_uuid(&seller).unwrap().reputation_score, 5);
    }

    #[test]
    fn buyer_can_also_record_reputation() {
        let mut registry = AgentRegistry::new();
        let seller = make_agent(&mut registry, "seller-7");
        let buyer = make_agent(&mut registry, "buyer-7");

        let order = create_verified_order(buyer, seller);
        let order_id = order.order_id;
        let mut mock_provider = MockOrderProvider::new();
        mock_provider.add_order(order);
        mock_provider.set_escrow_status(order_id, EscrowStatus::Released);

        let mut reputation = ReputationService::new(mock_provider);

        // Buyer records positive rating for seller
        let event = reputation.record_event(
            &mut registry,
            seller, // Recording for seller
            Some(order_id),
            ReputationEventType::PositiveRating,
            Some(5),
            "great service".into(),
        ).unwrap();

        assert_eq!(event.delta, 2);
    }
}
