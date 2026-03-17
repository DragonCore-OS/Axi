//! Repository traits for Service Layer
//!
//! These traits abstract persistence operations, allowing:
//! - Mock implementations for testing
//! - Swappable storage backends
//! - Controlled access (only Service layer can use repositories)

use std::sync::Arc;
use uuid::Uuid;

use crate::identity::registry::{AgentIdentity, AgentStatus, WalletRef};
use crate::market::order::{Order, OrderStatus};
use crate::market::escrow::{EscrowRecord, EscrowStatus, DeliveryProof};
use crate::market::listing::{Listing, ListingFilter};
use crate::identity::reputation::ReputationEvent;

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Repository-level errors
#[derive(Debug, Clone)]
pub enum RepositoryError {
    NotFound { entity_type: String, id: String },
    Conflict { resource: String, reason: String },
    ConstraintViolation { field: String, reason: String },
    Internal { message: String },
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::NotFound { entity_type, id } => {
                write!(f, "{} not found: {}", entity_type, id)
            }
            RepositoryError::Conflict { resource, reason } => {
                write!(f, "Conflict on {}: {}", resource, reason)
            }
            RepositoryError::ConstraintViolation { field, reason } => {
                write!(f, "Constraint violation on {}: {}", field, reason)
            }
            RepositoryError::Internal { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for RepositoryError {}

/// Agent repository operations
pub trait AgentRepositoryTrait: Send + Sync {
    /// Create a new agent
    fn create(&self, agent: &AgentIdentity) -> RepositoryResult<()>;
    
    /// Get agent by UUID
    fn get_by_uuid(&self, uuid: &Uuid) -> RepositoryResult<Option<AgentIdentity>>;
    
    /// Get agent by agent_id
    fn get_by_agent_id(&self, agent_id: &str) -> RepositoryResult<Option<AgentIdentity>>;
    
    /// List agents with pagination
    fn list(&self, limit: usize, offset: usize) -> RepositoryResult<Vec<AgentIdentity>>;
    
    /// Update agent status
    fn update_status(&self, uuid: &Uuid, status: AgentStatus) -> RepositoryResult<()>;
    
    /// Update reputation score (delta)
    fn update_reputation_score(&self, uuid: &Uuid, delta: i64) -> RepositoryResult<i64>;
    
    /// Add wallet to agent
    fn add_wallet(&self, wallet: &WalletRef) -> RepositoryResult<()>;
}

/// Order repository operations
pub trait OrderRepositoryTrait: Send + Sync {
    /// Create a new order
    fn create(&self, order: &Order) -> RepositoryResult<()>;
    
    /// Get order by ID
    fn get(&self, order_id: &Uuid) -> RepositoryResult<Option<Order>>;
    
    /// Update order status
    fn update_status(&self, order_id: &Uuid, status: OrderStatus) -> RepositoryResult<()>;
    
    /// List orders by buyer
    fn list_by_buyer(&self, buyer_uuid: &Uuid) -> RepositoryResult<Vec<Order>>;
    
    /// List orders by seller
    fn list_by_seller(&self, seller_uuid: &Uuid) -> RepositoryResult<Vec<Order>>;
}

/// Listing repository operations
pub trait ListingRepositoryTrait: Send + Sync {
    /// Create a new listing
    fn create(&self, listing: &Listing) -> RepositoryResult<()>;
    
    /// Get listing by ID
    fn get(&self, listing_id: &Uuid) -> RepositoryResult<Option<Listing>>;
    
    /// Update listing availability
    fn update_availability(&self, listing_id: &Uuid, available: bool) -> RepositoryResult<()>;
    
    /// Delete listing
    fn delete(&self, listing_id: &Uuid) -> RepositoryResult<()>;
    
    /// Search listings with filter
    fn search(&self, filter: &ListingFilter) -> RepositoryResult<Vec<Listing>>;
    
    /// List by seller
    fn list_by_seller(&self, seller_uuid: &Uuid) -> RepositoryResult<Vec<Listing>>;
}

/// Escrow repository operations
pub trait EscrowRepositoryTrait: Send + Sync {
    /// Create a new escrow
    fn create(&self, escrow: &EscrowRecord) -> RepositoryResult<()>;
    
    /// Get escrow by ID
    fn get(&self, escrow_id: &Uuid) -> RepositoryResult<Option<EscrowRecord>>;
    
    /// Get escrow by order ID
    fn get_by_order(&self, order_id: &Uuid) -> RepositoryResult<Option<EscrowRecord>>;
    
    /// Update escrow status
    fn update_status(&self, escrow_id: &Uuid, status: EscrowStatus) -> RepositoryResult<()>;
    
    /// Submit delivery proof
    fn submit_delivery(&self, escrow_id: &Uuid, proof: &DeliveryProof) -> RepositoryResult<()>;
    
    /// Record buyer verification
    fn verify_delivery(&self, escrow_id: &Uuid) -> RepositoryResult<()>;
    
    /// Open dispute
    fn open_dispute(&self, escrow_id: &Uuid, reason: &str) -> RepositoryResult<()>;
    
    /// Resolve dispute
    fn resolve_dispute(&self, escrow_id: &Uuid, resolution: &str) -> RepositoryResult<()>;
}

/// Reputation repository operations
pub trait ReputationRepositoryTrait: Send + Sync {
    /// Record a reputation event
    fn record_event(&self, event: &ReputationEvent) -> RepositoryResult<()>;
    
    /// List events by agent
    fn list_by_agent(&self, agent_uuid: &Uuid) -> RepositoryResult<Vec<ReputationEvent>>;
    
    /// Calculate reputation score from events
    fn calculate_score(&self, agent_uuid: &Uuid) -> RepositoryResult<i64>;
}

/// Container for all repositories
#[derive(Clone)]
pub struct Repositories {
    pub agent: Arc<dyn AgentRepositoryTrait>,
    pub listing: Arc<dyn ListingRepositoryTrait>,
    pub order: Arc<dyn OrderRepositoryTrait>,
    pub escrow: Arc<dyn EscrowRepositoryTrait>,
    pub reputation: Arc<dyn ReputationRepositoryTrait>,
}

impl Repositories {
    /// Create new repository container
    pub fn new(
        agent: Arc<dyn AgentRepositoryTrait>,
        listing: Arc<dyn ListingRepositoryTrait>,
        order: Arc<dyn OrderRepositoryTrait>,
        escrow: Arc<dyn EscrowRepositoryTrait>,
        reputation: Arc<dyn ReputationRepositoryTrait>,
    ) -> Self {
        Self {
            agent,
            listing,
            order,
            escrow,
            reputation,
        }
    }
}

// =====================================================
// In-Memory Implementations for Testing
// =====================================================

use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory agent repository for testing
pub struct InMemoryAgentRepository {
    agents: Mutex<HashMap<Uuid, AgentIdentity>>,
    by_agent_id: Mutex<HashMap<String, Uuid>>,
}

impl InMemoryAgentRepository {
    pub fn new() -> Self {
        Self {
            agents: Mutex::new(HashMap::new()),
            by_agent_id: Mutex::new(HashMap::new()),
        }
    }
}

impl AgentRepositoryTrait for InMemoryAgentRepository {
    fn create(&self, agent: &AgentIdentity) -> RepositoryResult<()> {
        let mut agents = self.agents.lock().unwrap();
        let mut by_id = self.by_agent_id.lock().unwrap();
        
        if agents.contains_key(&agent.agent_uuid) {
            return Err(RepositoryError::Conflict {
                resource: "Agent".to_string(),
                reason: format!("UUID {} already exists", agent.agent_uuid),
            });
        }
        
        if by_id.contains_key(&agent.agent_id) {
            return Err(RepositoryError::Conflict {
                resource: "Agent".to_string(),
                reason: format!("agent_id {} already exists", agent.agent_id),
            });
        }
        
        agents.insert(agent.agent_uuid, agent.clone());
        by_id.insert(agent.agent_id.clone(), agent.agent_uuid);
        Ok(())
    }
    
    fn get_by_uuid(&self, uuid: &Uuid) -> RepositoryResult<Option<AgentIdentity>> {
        let agents = self.agents.lock().unwrap();
        Ok(agents.get(uuid).cloned())
    }
    
    fn get_by_agent_id(&self, agent_id: &str) -> RepositoryResult<Option<AgentIdentity>> {
        let by_id = self.by_agent_id.lock().unwrap();
        let agents = self.agents.lock().unwrap();
        
        Ok(by_id.get(agent_id)
            .and_then(|uuid| agents.get(uuid).cloned()))
    }
    
    fn list(&self, limit: usize, offset: usize) -> RepositoryResult<Vec<AgentIdentity>> {
        let agents = self.agents.lock().unwrap();
        let mut list: Vec<_> = agents.values().cloned().collect();
        list.sort_by_key(|a| a.created_at);
        
        Ok(list.into_iter()
            .skip(offset)
            .take(limit)
            .collect())
    }
    
    fn update_status(&self, uuid: &Uuid, status: AgentStatus) -> RepositoryResult<()> {
        let mut agents = self.agents.lock().unwrap();
        let agent = agents.get_mut(uuid)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Agent".to_string(),
                id: uuid.to_string(),
            })?;
        agent.status = status;
        Ok(())
    }
    
    fn update_reputation_score(&self, uuid: &Uuid, delta: i64) -> RepositoryResult<i64> {
        let mut agents = self.agents.lock().unwrap();
        let agent = agents.get_mut(uuid)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Agent".to_string(),
                id: uuid.to_string(),
            })?;
        agent.reputation_score += delta;
        Ok(agent.reputation_score)
    }
    
    fn add_wallet(&self, wallet: &WalletRef) -> RepositoryResult<()> {
        let mut agents = self.agents.lock().unwrap();
        let agent = agents.get_mut(&wallet.agent_uuid)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Agent".to_string(),
                id: wallet.agent_uuid.to_string(),
            })?;
        agent.wallets.push(wallet.clone());
        Ok(())
    }
}

/// In-memory listing repository for testing
pub struct InMemoryListingRepository {
    listings: Mutex<HashMap<Uuid, Listing>>,
}

impl InMemoryListingRepository {
    pub fn new() -> Self {
        Self {
            listings: Mutex::new(HashMap::new()),
        }
    }
}

impl ListingRepositoryTrait for InMemoryListingRepository {
    fn create(&self, listing: &Listing) -> RepositoryResult<()> {
        let mut listings = self.listings.lock().unwrap();
        if listings.contains_key(&listing.listing_id) {
            return Err(RepositoryError::Conflict {
                resource: "Listing".to_string(),
                reason: format!("Listing {} already exists", listing.listing_id),
            });
        }
        listings.insert(listing.listing_id, listing.clone());
        Ok(())
    }
    
    fn get(&self, listing_id: &Uuid) -> RepositoryResult<Option<Listing>> {
        let listings = self.listings.lock().unwrap();
        Ok(listings.get(listing_id).cloned())
    }
    
    fn update_availability(&self, listing_id: &Uuid, available: bool) -> RepositoryResult<()> {
        let mut listings = self.listings.lock().unwrap();
        let listing = listings.get_mut(listing_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Listing".to_string(),
                id: listing_id.to_string(),
            })?;
        listing.availability = if available {
            crate::market::listing::ListingAvailability::Available
        } else {
            crate::market::listing::ListingAvailability::Paused
        };
        listing.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
    
    fn delete(&self, listing_id: &Uuid) -> RepositoryResult<()> {
        let mut listings = self.listings.lock().unwrap();
        listings.remove(listing_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Listing".to_string(),
                id: listing_id.to_string(),
            })?;
        Ok(())
    }
    
    fn search(&self, filter: &ListingFilter) -> RepositoryResult<Vec<Listing>> {
        let listings = self.listings.lock().unwrap();
        let mut results: Vec<_> = listings.values()
            .filter(|l| {
                if let Some(ref t) = filter.listing_type {
                    if &l.listing_type != t {
                        return false;
                    }
                }
                if let Some(max_price) = filter.max_price_axi {
                    match l.pricing_model {
                        crate::market::listing::PricingModel::Fixed => {
                            if l.price_axi.unwrap_or(u64::MAX) > max_price {
                                return false;
                            }
                        }
                        crate::market::listing::PricingModel::UsageBased => {
                            if l.price_per_unit_axi.unwrap_or(u64::MAX) > max_price {
                                return false;
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(ref tag) = filter.tag {
                    if !l.tags.iter().any(|t| t == tag) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(results)
    }
    
    fn list_by_seller(&self, seller_uuid: &Uuid) -> RepositoryResult<Vec<Listing>> {
        let listings = self.listings.lock().unwrap();
        let mut results: Vec<_> = listings.values()
            .filter(|l| l.seller_agent_uuid == *seller_uuid)
            .cloned()
            .collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(results)
    }
}

/// In-memory order repository for testing
pub struct InMemoryOrderRepository {
    orders: Mutex<HashMap<Uuid, Order>>,
}

impl InMemoryOrderRepository {
    pub fn new() -> Self {
        Self {
            orders: Mutex::new(HashMap::new()),
        }
    }
}

impl OrderRepositoryTrait for InMemoryOrderRepository {
    fn create(&self, order: &Order) -> RepositoryResult<()> {
        let mut orders = self.orders.lock().unwrap();
        if orders.contains_key(&order.order_id) {
            return Err(RepositoryError::Conflict {
                resource: "Order".to_string(),
                reason: format!("Order {} already exists", order.order_id),
            });
        }
        orders.insert(order.order_id, order.clone());
        Ok(())
    }
    
    fn get(&self, order_id: &Uuid) -> RepositoryResult<Option<Order>> {
        let orders = self.orders.lock().unwrap();
        Ok(orders.get(order_id).cloned())
    }
    
    fn update_status(&self, order_id: &Uuid, status: OrderStatus) -> RepositoryResult<()> {
        let mut orders = self.orders.lock().unwrap();
        let order = orders.get_mut(order_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Order".to_string(),
                id: order_id.to_string(),
            })?;
        order.status = status;
        order.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
    
    fn list_by_buyer(&self, buyer_uuid: &Uuid) -> RepositoryResult<Vec<Order>> {
        let orders = self.orders.lock().unwrap();
        let mut list: Vec<_> = orders.values()
            .filter(|o| o.buyer_agent_uuid == *buyer_uuid)
            .cloned()
            .collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }
    
    fn list_by_seller(&self, seller_uuid: &Uuid) -> RepositoryResult<Vec<Order>> {
        let orders = self.orders.lock().unwrap();
        let mut list: Vec<_> = orders.values()
            .filter(|o| o.seller_agent_uuid == *seller_uuid)
            .cloned()
            .collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }
}

/// In-memory escrow repository for testing
pub struct InMemoryEscrowRepository {
    escrows: Mutex<HashMap<Uuid, EscrowRecord>>,
}

impl InMemoryEscrowRepository {
    pub fn new() -> Self {
        Self {
            escrows: Mutex::new(HashMap::new()),
        }
    }
}

impl EscrowRepositoryTrait for InMemoryEscrowRepository {
    fn create(&self, escrow: &EscrowRecord) -> RepositoryResult<()> {
        let mut escrows = self.escrows.lock().unwrap();
        if escrows.contains_key(&escrow.escrow_id) {
            return Err(RepositoryError::Conflict {
                resource: "Escrow".to_string(),
                reason: format!("Escrow {} already exists", escrow.escrow_id),
            });
        }
        escrows.insert(escrow.escrow_id, escrow.clone());
        Ok(())
    }
    
    fn get(&self, escrow_id: &Uuid) -> RepositoryResult<Option<EscrowRecord>> {
        let escrows = self.escrows.lock().unwrap();
        Ok(escrows.get(escrow_id).cloned())
    }
    
    fn get_by_order(&self, order_id: &Uuid) -> RepositoryResult<Option<EscrowRecord>> {
        let escrows = self.escrows.lock().unwrap();
        Ok(escrows.values()
            .find(|e| e.order_id == *order_id)
            .cloned())
    }
    
    fn update_status(&self, escrow_id: &Uuid, status: EscrowStatus) -> RepositoryResult<()> {
        let mut escrows = self.escrows.lock().unwrap();
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;
        escrow.escrow_status = status;
        escrow.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
    
    fn submit_delivery(&self, escrow_id: &Uuid, proof: &DeliveryProof) -> RepositoryResult<()> {
        let mut escrows = self.escrows.lock().unwrap();
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;
        escrow.delivery_proof = Some(proof.clone());
        escrow.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
    
    fn verify_delivery(&self, escrow_id: &Uuid) -> RepositoryResult<()> {
        let mut escrows = self.escrows.lock().unwrap();
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;
        escrow.buyer_verified_at = Some(chrono::Utc::now().to_rfc3339());
        escrow.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
    
    fn open_dispute(&self, escrow_id: &Uuid, reason: &str) -> RepositoryResult<()> {
        let mut escrows = self.escrows.lock().unwrap();
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;
        escrow.escrow_status = EscrowStatus::Disputed;
        escrow.dispute_reason = Some(reason.to_string());
        escrow.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
    
    fn resolve_dispute(&self, escrow_id: &Uuid, _resolution: &str) -> RepositoryResult<()> {
        let mut escrows = self.escrows.lock().unwrap();
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;
        escrow.escrow_status = EscrowStatus::Released;
        escrow.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
}

/// In-memory reputation repository for testing
pub struct InMemoryReputationRepository {
    events: Mutex<Vec<ReputationEvent>>,
}

impl InMemoryReputationRepository {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }
}

impl ReputationRepositoryTrait for InMemoryReputationRepository {
    fn record_event(&self, event: &ReputationEvent) -> RepositoryResult<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        Ok(())
    }
    
    fn list_by_agent(&self, agent_uuid: &Uuid) -> RepositoryResult<Vec<ReputationEvent>> {
        let events = self.events.lock().unwrap();
        let list: Vec<_> = events.iter()
            .filter(|e| e.agent_uuid == *agent_uuid)
            .cloned()
            .collect();
        Ok(list)
    }
    
    fn calculate_score(&self, agent_uuid: &Uuid) -> RepositoryResult<i64> {
        let events = self.events.lock().unwrap();
        let score: i64 = events.iter()
            .filter(|e| e.agent_uuid == *agent_uuid)
            .map(|e| e.delta)
            .sum();
        Ok(score)
    }
}
