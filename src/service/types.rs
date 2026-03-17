//! Service-level shared types
//!
//! These types represent the domain model at the service boundary,
//! decoupled from storage representation.

use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Caller identity for authorization
#[derive(Debug, Clone)]
pub struct Caller {
    pub agent_uuid: Uuid,
    pub agent_id: String,
    pub permissions: Vec<Permission>,
}

impl Caller {
    /// Create system caller (for automated operations)
    pub fn system() -> Self {
        Self {
            agent_uuid: Uuid::nil(),
            agent_id: "system".to_string(),
            permissions: vec![Permission::System],
        }
    }

    /// Check if caller has specific permission
    pub fn has_permission(&self, perm: Permission) -> bool {
        self.permissions.contains(&perm) || self.permissions.contains(&Permission::System)
    }
}

/// Permissions for authorization checks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    /// System-level access
    System,
    /// Register new agents
    RegisterAgent,
    /// Update own agent profile
    UpdateOwnProfile,
    /// Update any agent profile (admin)
    UpdateAnyProfile,
    /// Create listings
    CreateListing,
    /// Place orders
    PlaceOrder,
    /// Manage own orders
    ManageOwnOrder,
    /// Manage any order (arbitration)
    ManageAnyOrder,
    /// Submit delivery (seller)
    SubmitDelivery,
    /// Verify delivery (buyer)
    VerifyDelivery,
    /// Open disputes
    OpenDispute,
    /// Resolve disputes (arbitration)
    ResolveDispute,
}

/// Metadata attached to every service operation
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub caller: Caller,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: String,
    pub client_info: Option<String>,
}

impl OperationContext {
    /// Create new operation context
    pub fn new(caller: Caller) -> Self {
        Self {
            caller,
            timestamp: Utc::now(),
            correlation_id: Uuid::new_v4().to_string(),
            client_info: None,
        }
    }

    /// With client info
    pub fn with_client_info(mut self, info: impl Into<String>) -> Self {
        self.client_info = Some(info.into());
        self
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub limit: usize,
    pub offset: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

impl Pagination {
    /// Create with specific limit
    pub fn with_limit(limit: usize) -> Self {
        Self { limit, ..Default::default() }
    }

    /// Move to next page
    pub fn next_page(&self) -> Self {
        Self {
            limit: self.limit,
            offset: self.offset + self.limit,
        }
    }
}

/// Paginated result
#[derive(Debug, Clone)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub pagination: Pagination,
}

impl<T> Paginated<T> {
    /// Check if there are more pages
    pub fn has_more(&self) -> bool {
        self.pagination.offset + self.items.len() < self.total
    }
}

/// Change summary for journal entries
#[derive(Debug, Clone)]
pub struct ChangeSummary {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// Audit metadata for operations
#[derive(Debug, Clone)]
pub struct AuditMetadata {
    pub operation: String,
    pub entity_type: String,
    pub entity_id: String,
    pub caller_id: String,
    pub changes: Vec<ChangeSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_system_has_all_permissions() {
        let system = Caller::system();
        assert!(system.has_permission(Permission::RegisterAgent));
        assert!(system.has_permission(Permission::ManageAnyOrder));
    }

    #[test]
    fn pagination_calculations() {
        let p = Pagination::default();
        assert_eq!(p.limit, 20);
        assert_eq!(p.offset, 0);

        let next = p.next_page();
        assert_eq!(next.offset, 20);
    }
}
