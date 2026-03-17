//! P1-2: Unified Service Layer
//!
//! Business logic consolidation with fixed write sequence:
//! validate → mutate → persist → journal → dibl emit

use std::sync::Arc;
use crate::governance::DiblBroadcaster;
// Repositories imported via pub use below
// Journal trait placeholder for audit logging
pub trait AuditJournal: Send + Sync {
    fn append(&self, entry: crate::storage::journal::JournalEntry) -> Result<(), String>;
}

// In-memory implementation for testing
pub struct InMemoryJournal;
impl InMemoryJournal {
    pub fn new() -> Self { Self }
}
impl AuditJournal for InMemoryJournal {
    fn append(&self, _entry: crate::storage::journal::JournalEntry) -> Result<(), String> {
        Ok(())
    }
}

pub mod identity_service;
pub mod market_service;
pub mod escrow_service;
pub mod types;
pub mod repositories;
pub use repositories::{
    Repositories,
    RepositoryError, RepositoryResult,
};

pub(crate) use repositories::{
    AgentRepositoryTrait, ListingRepositoryTrait, OrderRepositoryTrait, 
    EscrowRepositoryTrait, ReputationRepositoryTrait,
    InMemoryAgentRepository, InMemoryListingRepository, InMemoryOrderRepository,
    InMemoryEscrowRepository, InMemoryReputationRepository,
};

use identity_service::IdentityService;
use market_service::MarketService;
use escrow_service::EscrowService;

/// Shared service context for dependency injection
pub struct ServiceContext {
    /// DIBL broadcaster for governance events
    pub dibl: Arc<DiblBroadcaster>,
    /// Audit journal for all mutations
    pub journal: Arc<dyn AuditJournal>,
    /// Repositories for persistence
    pub repos: Repositories,
}

impl ServiceContext {
    /// Create new context with dependencies
    pub fn new(
        dibl: Arc<DiblBroadcaster>,
        journal: Arc<dyn AuditJournal>,
        repos: Repositories,
    ) -> Self {
        Self { dibl, journal, repos }
    }

    /// Create context for testing with in-memory repositories
    #[cfg(test)]
    pub fn new_test() -> Self {
        use crate::governance::InMemoryEventStore;
        use repositories::{InMemoryAgentRepository, InMemoryListingRepository};
        
        let store = Arc::new(InMemoryEventStore::new());
        let dibl = Arc::new(DiblBroadcaster::new(store));
        let journal: Arc<dyn AuditJournal> = Arc::new(InMemoryJournal);
        
        let repos = Repositories {
            agent: Arc::new(InMemoryAgentRepository::new()),
            listing: Arc::new(InMemoryListingRepository::new()),
            order: Arc::new(InMemoryOrderRepository::new()),
            escrow: Arc::new(InMemoryEscrowRepository::new()),
            reputation: Arc::new(InMemoryReputationRepository::new()),
        };
        
        Self { dibl, journal, repos }
    }
}

/// Service facade providing unified access to all business operations
pub struct Services {
    pub identity: IdentityService,
    pub market: MarketService,
    pub escrow: EscrowService,
}

impl Services {
    /// Initialize all services with shared context
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self {
            identity: IdentityService::new(ctx.clone()),
            market: MarketService::new(ctx.clone()),
            escrow: EscrowService::new(ctx.clone()),
        }
    }
}

/// Trait for service implementations
/// 
/// Ensures consistent error handling and logging across all services
pub trait Service {
    /// Service name for logging/telemetry
    fn name(&self) -> &'static str;
}

/// Result type for service operations
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Service-level errors
#[derive(Debug, Clone)]
pub enum ServiceError {
    /// Authorization failure
    Unauthorized { reason: String },
    /// Validation failure
    InvalidInput { field: String, reason: String },
    /// State transition not allowed
    InvalidTransition { from: String, to: String },
    /// Entity not found
    NotFound { entity_type: String, id: String },
    /// Conflict (e.g., duplicate)
    Conflict { resource: String, reason: String },
    /// Internal error (persistence, etc.)
    Internal { message: String },
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::Unauthorized { reason } => {
                write!(f, "Unauthorized: {}", reason)
            }
            ServiceError::InvalidInput { field, reason } => {
                write!(f, "Invalid input for {}: {}", field, reason)
            }
            ServiceError::InvalidTransition { from, to } => {
                write!(f, "Invalid state transition from {} to {}", from, to)
            }
            ServiceError::NotFound { entity_type, id } => {
                write!(f, "{} not found: {}", entity_type, id)
            }
            ServiceError::Conflict { resource, reason } => {
                write!(f, "Conflict on {}: {}", resource, reason)
            }
            ServiceError::Internal { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for ServiceError {}

/// Helper macro for service method boilerplate
/// 
/// Usage:
/// ```rust
/// service_method! {
///     fn register_agent(&self, params) -> ServiceResult<Agent> {
///         // validation
///         // mutation
///         // persist
///         // journal
///         // dibl emit
///         Ok(agent)
///     }
/// }
/// ```
#[macro_export]
macro_rules! service_method {
    (
        fn $name:ident(&self $(, $param:ident: $type:ty)* $(,)?) -> ServiceResult<$ret:ty> $body:block
    ) => {
        fn $name(&self $(, $param: $type)*) -> ServiceResult<$ret> {
            let _span = tracing::info_span!(stringify!($name)).entered();
            tracing::info!("Service method called");
            
            let result = (|| -> ServiceResult<$ret> $body)();
            
            match &result {
                Ok(_) => tracing::info!("Service method succeeded"),
                Err(e) => tracing::error!("Service method failed: {}", e),
            }
            
            result
        }
    };
}
