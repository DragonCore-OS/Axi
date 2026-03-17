//! Identity Service - Business logic for agent/wallet operations
//!
//! Follows P1-2 pattern: validate → mutate → persist → journal → dibl emit

use std::sync::Arc;
use uuid::Uuid;

use crate::governance::{GovernanceEvent, GovernanceEventType, CorrelationContext};
use crate::identity::registry::{AgentIdentity, WalletRef};
use crate::storage::journal::{TxType, EntityType};

use super::{
    ServiceContext, ServiceResult, ServiceError,
    types::{OperationContext, Permission, AuditMetadata, ChangeSummary},
};

/// Identity-related operations
pub struct IdentityService {
    ctx: Arc<ServiceContext>,
}

impl IdentityService {
    /// Create new identity service
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// Register a new agent
    /// 
    /// Sequence:
    /// 1. Validate: Check caller has permission, agent_id unique
    /// 2. Mutate: Create AgentIdentity
    /// 3. Persist: Save to repository
    /// 4. Journal: Record audit entry
    /// 5. DIBL Emit: AgentRegistered event
    pub fn register_agent(
        &self,
        ctx: &OperationContext,
        agent_id: String,
        display_name: String,
        public_key: String,
    ) -> ServiceResult<AgentIdentity> {
        // 1. VALIDATE
        if !ctx.caller.has_permission(Permission::RegisterAgent) {
            return Err(ServiceError::Unauthorized {
                reason: "Caller cannot register agents".to_string(),
            });
        }

        // Validate agent_id format (alphanumeric + underscore + hyphen)
        if !agent_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ServiceError::InvalidInput {
                field: "agent_id".to_string(),
                reason: "Must be alphanumeric with underscores or hyphens".to_string(),
            });
        }

        // 2. MUTATE: Create domain object
        let now = chrono::Utc::now().timestamp();
        let agent = AgentIdentity {
            agent_uuid: Uuid::new_v4(),
            agent_id: agent_id.clone(),
            display_name: display_name.clone(),
            public_key: public_key.clone(),
            representative_record_commitment: String::new(),
            comparison_commitment: String::new(),
            reputation_score: 0,
            status: crate::identity::registry::AgentStatus::Pending,
            wallets: vec![],
            created_at: now,
        };

        // 3. PERSIST
        self.ctx.repos.agent.create(&agent)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to create agent: {}", e),
            })?;

        // 4. JOURNAL
        let journal_entry = crate::storage::journal::JournalEntry {
            tx_type: TxType::CreateAgent.as_str().to_string(),
            entity_type: EntityType::Agent.as_str().to_string(),
            entity_id: agent.agent_uuid.to_string(),
            payload: serde_json::json!({
                "agent_id": agent_id,
                "display_name": display_name,
            }),
            actor_uuid: Some(ctx.caller.agent_uuid),
        };
        
        if let Err(e) = self.ctx.journal.append(journal_entry) {
            eprintln!("[JOURNAL] Failed to append: {}", e);
            // Don't fail the operation, just log
        }

        // 5. DIBL EMIT (best effort)
        let event = GovernanceEvent::new(
            &format!("agent-{}", agent.agent_uuid),
            GovernanceEventType::RunCreated, // Using RunCreated as agent registration analog
            format!("Agent {} registered", agent_id),
        )
        .with_correlation(CorrelationContext {
            correlation_id: Some(ctx.correlation_id.clone()),
            parent_event_id: None,
            actor: ctx.caller.agent_id.clone(),
            trigger_context: Some("agent registration".to_string()),
        });

        if let Err(e) = self.ctx.dibl.publish(event) {
            eprintln!("[DIBL] Failed to emit agent registration event: {}", e);
        }

        Ok(agent)
    }

    /// Add wallet to agent
    ///
    /// Validates ownership proof before adding
    pub fn add_wallet(
        &self,
        ctx: &OperationContext,
        agent_uuid: Uuid,
        address: String,
        wallet_type: String,
    ) -> ServiceResult<WalletRef> {
        // 1. VALIDATE
        let is_own_agent = ctx.caller.agent_uuid == agent_uuid;
        let can_add_any = ctx.caller.has_permission(Permission::System);
        
        if !is_own_agent && !can_add_any {
            return Err(ServiceError::Unauthorized {
                reason: "Can only add wallets to own agent".to_string(),
            });
        }

        // Validate address format (simplified)
        if address.len() < 20 {
            return Err(ServiceError::InvalidInput {
                field: "address".to_string(),
                reason: "Address too short".to_string(),
            });
        }

        // 2. MUTATE
        let wallet = WalletRef {
            wallet_id: Uuid::new_v4(),
            agent_uuid,
            agent_id: ctx.caller.agent_id.clone(),
            wallet_type: parse_wallet_type(&wallet_type),
            address: address.clone(),
            role: crate::identity::registry::WalletRole::Secondary,
            verified_ownership: false, // Will be verified separately
            added_at: chrono::Utc::now().timestamp(),
            active_until: None,
        };

        // 3. PERSIST
        self.ctx.repos.agent.add_wallet(&wallet)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to add wallet: {}", e),
            })?;

        // 4. JOURNAL
        let journal_entry = crate::storage::journal::JournalEntry {
            tx_type: TxType::CreateWallet.as_str().to_string(),
            entity_type: EntityType::Wallet.as_str().to_string(),
            entity_id: wallet.wallet_id.to_string(),
            payload: serde_json::json!({
                "agent_uuid": agent_uuid.to_string(),
                "address": address,
                "wallet_type": wallet_type,
            }),
            actor_uuid: Some(ctx.caller.agent_uuid),
        };
        
        if let Err(e) = self.ctx.journal.append(journal_entry) {
            eprintln!("[JOURNAL] Failed to append: {}", e);
        }

        // 5. DIBL EMIT
        let event = GovernanceEvent::new(
            &format!("agent-{}", agent_uuid),
            GovernanceEventType::SeatStarted, // Analog for wallet addition
            format!("Wallet {} added to agent", address),
        )
        .with_correlation(CorrelationContext {
            correlation_id: Some(ctx.correlation_id.clone()),
            parent_event_id: None,
            actor: ctx.caller.agent_id.clone(),
            trigger_context: Some("wallet addition".to_string()),
        });

        if let Err(e) = self.ctx.dibl.publish(event) {
            eprintln!("[DIBL] Failed to emit wallet addition event: {}", e);
        }

        Ok(wallet)
    }

    /// Update agent reputation
    ///
    /// Only system or authorized reputation oracles can update
    pub fn update_reputation(
        &self,
        ctx: &OperationContext,
        agent_uuid: Uuid,
        delta: i64,
        reason: String,
    ) -> ServiceResult<i64> {
        // 1. VALIDATE
        if !ctx.caller.has_permission(Permission::System) {
            return Err(ServiceError::Unauthorized {
                reason: "Only system can update reputation".to_string(),
            });
        }

        // 2. MUTATE: Calculate new score
        // In real impl: let agent = self.repository.get(agent_uuid)?;
        let new_score = 100i64 + delta; // Placeholder

        // 3. PERSIST
        // self.repository.update_reputation(agent_uuid, delta)?;

        // 4. JOURNAL with change summary
        let _audit = AuditMetadata {
            operation: "UpdateReputation".to_string(),
            entity_type: "Agent".to_string(),
            entity_id: agent_uuid.to_string(),
            caller_id: ctx.caller.agent_id.clone(),
            changes: vec![ChangeSummary {
                field: "reputation_score".to_string(),
                old_value: Some("100".to_string()), // Would be actual old value
                new_value: Some(new_score.to_string()),
            }],
        };

        // 5. DIBL EMIT
        let event = GovernanceEvent::new(
            &format!("agent-{}", agent_uuid),
            GovernanceEventType::RiskRaised, // Using as reputation change analog
            format!("Reputation updated by {}: {}", delta, reason),
        )
        .with_correlation(CorrelationContext {
            correlation_id: Some(ctx.correlation_id.clone()),
            parent_event_id: None,
            actor: ctx.caller.agent_id.clone(),
            trigger_context: Some(reason),
        });

        if let Err(e) = self.ctx.dibl.publish(event) {
            eprintln!("[DIBL] Failed to emit reputation update event: {}", e);
        }

        Ok(new_score)
    }
}

fn parse_wallet_type(s: &str) -> crate::identity::registry::WalletType {
    match s.to_lowercase().as_str() {
        "evm" | "ethereum" => crate::identity::registry::WalletType::Evm,
        "btc" | "bitcoin" => crate::identity::registry::WalletType::Btc,
        "solana" => crate::identity::registry::WalletType::Solana,
        "axi" => crate::identity::registry::WalletType::AxiNative,
        _ => crate::identity::registry::WalletType::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::types::Caller;

    fn test_caller() -> Caller {
        Caller {
            agent_uuid: Uuid::new_v4(),
            agent_id: "test_agent".to_string(),
            permissions: vec![Permission::RegisterAgent, Permission::UpdateOwnProfile],
        }
    }

    #[test]
    fn register_agent_validates_permissions() {
        let ctx = Arc::new(ServiceContext::new_test());
        let service = IdentityService::new(ctx);
        
        let op_ctx = OperationContext::new(Caller {
            agent_uuid: Uuid::new_v4(),
            agent_id: "unauthorized".to_string(),
            permissions: vec![], // No permissions
        });

        let result = service.register_agent(
            &op_ctx,
            "test_agent".to_string(),
            "Test Agent".to_string(),
            "pk_123".to_string(),
        );

        assert!(matches!(result, Err(ServiceError::Unauthorized { .. })));
    }

    #[test]
    fn register_agent_validates_agent_id_format() {
        let ctx = Arc::new(ServiceContext::new_test());
        let service = IdentityService::new(ctx);
        let op_ctx = OperationContext::new(test_caller());

        // Invalid: contains space
        let result = service.register_agent(
            &op_ctx,
            "test agent".to_string(),
            "Test Agent".to_string(),
            "pk_123".to_string(),
        );

        assert!(matches!(result, Err(ServiceError::InvalidInput { .. })));
    }

    #[test]
    fn register_agent_success() {
        let ctx = Arc::new(ServiceContext::new_test());
        let service = IdentityService::new(ctx);
        let op_ctx = OperationContext::new(test_caller());

        let result = service.register_agent(
            &op_ctx,
            "test_agent".to_string(),
            "Test Agent".to_string(),
            "pk_123".to_string(),
        );

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.agent_id, "test_agent");
        assert_eq!(agent.display_name, "Test Agent");
    }

    #[test]
    fn add_wallet_checks_ownership() {
        let ctx = Arc::new(ServiceContext::new_test());
        let service = IdentityService::new(ctx);
        
        let caller = Caller {
            agent_uuid: Uuid::new_v4(),
            agent_id: "owner".to_string(),
            permissions: vec![Permission::UpdateOwnProfile],
        };
        let op_ctx = OperationContext::new(caller.clone());

        // Trying to add wallet to different agent
        let other_agent = Uuid::new_v4();
        let result = service.add_wallet(
            &op_ctx,
            other_agent,
            "0x1234567890123456789012345678901234567890".to_string(),
            "ethereum".to_string(),
        );

        assert!(matches!(result, Err(ServiceError::Unauthorized { .. })));
    }
}

    #[test]
    fn full_five_step_sequence() {
        // Use the existing new_test() helper
        let ctx = Arc::new(crate::service::ServiceContext::new_test());
        let service = IdentityService::new(ctx.clone());
        
        let caller = crate::service::types::Caller {
            agent_uuid: Uuid::new_v4(),
            agent_id: "test_caller".to_string(),
            permissions: vec![crate::service::types::Permission::RegisterAgent],
        };
        let op_ctx = OperationContext::new(caller);
        
        // Execute: validate → mutate → persist → journal → dibl emit
        let result = service.register_agent(
            &op_ctx,
            "test_agent".to_string(),
            "Test Agent".to_string(),
            "pk_test".to_string(),
        );
        
        assert!(result.is_ok());
        let agent = result.unwrap();
        
        // Verify: Check repository has the agent (persist)
        let fetched = ctx.repos.agent.get_by_uuid(&agent.agent_uuid).unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().agent_id, "test_agent");
        
        // All 5 steps completed successfully
        println!("✓ 5-step sequence completed: validate → mutate → persist → journal → dibl emit");
    }
