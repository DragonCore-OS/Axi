//! Escrow Service - Business logic for escrow lifecycle
//!
//! Coordinates escrow operations with proper authorization and DIBL emission

use std::sync::Arc;
use uuid::Uuid;

use crate::market::escrow::{EscrowRecord, EscrowStatus, DeliveryProof};
use crate::storage::journal::{TxType, EntityType};

use super::{ServiceContext, ServiceResult, ServiceError};
use super::types::{OperationContext, Permission};

/// Escrow operations
pub struct EscrowService {
    ctx: Arc<ServiceContext>,
}

impl EscrowService {
    /// Create new escrow service
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// Create and fund an escrow for an order
    /// 
    /// Sequence: validate → mutate → persist → journal → dibl emit
    pub fn fund_escrow(
        &self,
        ctx: &OperationContext,
        order_id: Uuid,
        buyer_uuid: Uuid,
        seller_uuid: Uuid,
        amount_axi: u64,
    ) -> ServiceResult<Uuid> {
        // 1. VALIDATE
        if !ctx.caller.has_permission(Permission::PlaceOrder) {
            return Err(ServiceError::Unauthorized {
                reason: "Cannot fund escrows".to_string(),
            });
        }

        if amount_axi == 0 {
            return Err(ServiceError::InvalidInput {
                field: "amount_axi".to_string(),
                reason: "Amount must be greater than 0".to_string(),
            });
        }

        // 2. MUTATE: Create escrow record
        let now = chrono::Utc::now().to_rfc3339();
        let escrow = EscrowRecord {
            escrow_id: Uuid::new_v4(),
            order_id,
            buyer_agent_uuid: buyer_uuid,
            seller_agent_uuid: seller_uuid,
            amount_axi,
            escrow_status: EscrowStatus::Funded,
            delivery_proof: None,
            buyer_verified_at: None,
            auto_complete_after: None,
            dispute_reason: None,
            created_at: now.clone(),
            updated_at: now,
        };

        // 3. PERSIST
        self.ctx.repos.escrow.create(&escrow)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to create escrow: {}", e),
            })?;

        // 4. JOURNAL
        let journal_entry = crate::storage::journal::JournalEntry {
            tx_type: TxType::CreateEscrow.as_str().to_string(),
            entity_type: EntityType::Escrow.as_str().to_string(),
            entity_id: escrow.escrow_id.to_string(),
            payload: serde_json::json!({
                "order_id": order_id.to_string(),
                "amount_axi": amount_axi,
            }),
            actor_uuid: Some(ctx.caller.agent_uuid),
        };
        
        if let Err(e) = self.ctx.journal.append(journal_entry) {
            eprintln!("[JOURNAL] Failed to append: {}", e);
        }

        // 5. DIBL EMIT
        let event = crate::governance::GovernanceEvent::new(
            &format!("escrow-{}", escrow.escrow_id),
            crate::governance::GovernanceEventType::RunCreated,
            format!("Escrow funded for order {}", order_id),
        )
        .with_correlation(crate::governance::CorrelationContext {
            correlation_id: Some(ctx.correlation_id.clone()),
            parent_event_id: None,
            actor: ctx.caller.agent_id.clone(),
            trigger_context: Some("escrow funding".to_string()),
        });

        if let Err(e) = self.ctx.dibl.publish(event) {
            eprintln!("[DIBL] Failed to emit escrow funded event: {}", e);
        }

        Ok(escrow.escrow_id)
    }

    /// Submit delivery proof (seller)
    /// 
    /// Sequence: validate → mutate → persist → journal → dibl emit
    pub fn submit_delivery(
        &self,
        ctx: &OperationContext,
        escrow_id: Uuid,
        proof_cid: String,
    ) -> ServiceResult<()> {
        // 1. VALIDATE
        if !ctx.caller.has_permission(Permission::SubmitDelivery) {
            return Err(ServiceError::Unauthorized {
                reason: "Only sellers can submit delivery".to_string(),
            });
        }

        // Verify caller is the seller for this escrow
        let escrow = self.ctx.repos.escrow.get(&escrow_id)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to get escrow: {}", e),
            })?
            .ok_or_else(|| ServiceError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;

        if escrow.seller_agent_uuid != ctx.caller.agent_uuid {
            return Err(ServiceError::Unauthorized {
                reason: "Only the seller for this escrow can submit delivery".to_string(),
            });
        }

        if !matches!(escrow.escrow_status, EscrowStatus::Funded | EscrowStatus::InEscrow) {
            return Err(ServiceError::InvalidTransition {
                from: format!("{:?}", escrow.escrow_status),
                to: "DeliverySubmitted".to_string(),
            });
        }

        // 2. MUTATE: Create delivery proof
        let proof = DeliveryProof {
            cid: Some(proof_cid.clone()),
            uri: None,
            note: Some("Delivery submitted".to_string()),
            submitted_at: chrono::Utc::now().to_rfc3339(),
        };

        // 3. PERSIST
        self.ctx.repos.escrow.submit_delivery(&escrow_id, &proof)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to submit delivery: {}", e),
            })?;

        // 4. JOURNAL
        let journal_entry = crate::storage::journal::JournalEntry {
            tx_type: TxType::SubmitDelivery.as_str().to_string(),
            entity_type: EntityType::Escrow.as_str().to_string(),
            entity_id: escrow_id.to_string(),
            payload: serde_json::json!({
                "proof_cid": proof_cid,
            }),
            actor_uuid: Some(ctx.caller.agent_uuid),
        };
        
        if let Err(e) = self.ctx.journal.append(journal_entry) {
            eprintln!("[JOURNAL] Failed to append: {}", e);
        }

        // 5. DIBL EMIT
        let event = crate::governance::GovernanceEvent::new(
            &format!("escrow-{}", escrow_id),
            crate::governance::GovernanceEventType::SeatCompleted,
            format!("Delivery submitted for escrow {}", escrow_id),
        )
        .with_correlation(crate::governance::CorrelationContext {
            correlation_id: Some(ctx.correlation_id.clone()),
            parent_event_id: None,
            actor: ctx.caller.agent_id.clone(),
            trigger_context: Some("delivery submission".to_string()),
        });

        if let Err(e) = self.ctx.dibl.publish(event) {
            eprintln!("[DIBL] Failed to emit delivery submitted event: {}", e);
        }

        Ok(())
    }

    /// Verify delivery (buyer)
    /// 
    /// Sequence: validate → mutate → persist → journal → dibl emit
    pub fn verify_delivery(
        &self,
        ctx: &OperationContext,
        escrow_id: Uuid,
    ) -> ServiceResult<()> {
        // 1. VALIDATE
        if !ctx.caller.has_permission(Permission::VerifyDelivery) {
            return Err(ServiceError::Unauthorized {
                reason: "Only buyers can verify delivery".to_string(),
            });
        }

        // Verify caller is the buyer for this escrow
        let escrow = self.ctx.repos.escrow.get(&escrow_id)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to get escrow: {}", e),
            })?
            .ok_or_else(|| ServiceError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;

        if escrow.buyer_agent_uuid != ctx.caller.agent_uuid {
            return Err(ServiceError::Unauthorized {
                reason: "Only the buyer for this escrow can verify delivery".to_string(),
            });
        }

        if escrow.delivery_proof.is_none() {
            return Err(ServiceError::InvalidInput {
                field: "escrow".to_string(),
                reason: "No delivery has been submitted yet".to_string(),
            });
        }

        // 2. MUTATE: Update status to Released
        // 3. PERSIST
        self.ctx.repos.escrow.update_status(&escrow_id, EscrowStatus::Released)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to update escrow status: {}", e),
            })?;

        self.ctx.repos.escrow.verify_delivery(&escrow_id)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to record verification: {}", e),
            })?;

        // 4. JOURNAL
        let journal_entry = crate::storage::journal::JournalEntry {
            tx_type: TxType::VerifyDelivery.as_str().to_string(),
            entity_type: EntityType::Escrow.as_str().to_string(),
            entity_id: escrow_id.to_string(),
            payload: serde_json::json!({}),
            actor_uuid: Some(ctx.caller.agent_uuid),
        };
        
        if let Err(e) = self.ctx.journal.append(journal_entry) {
            eprintln!("[JOURNAL] Failed to append: {}", e);
        }

        // 5. DIBL EMIT
        let event = crate::governance::GovernanceEvent::new(
            &format!("escrow-{}", escrow_id),
            crate::governance::GovernanceEventType::DecisionCommitted,
            format!("Delivery verified, escrow {} released", escrow_id),
        )
        .with_correlation(crate::governance::CorrelationContext {
            correlation_id: Some(ctx.correlation_id.clone()),
            parent_event_id: None,
            actor: ctx.caller.agent_id.clone(),
            trigger_context: Some("delivery verification".to_string()),
        });

        if let Err(e) = self.ctx.dibl.publish(event) {
            eprintln!("[DIBL] Failed to emit delivery verified event: {}", e);
        }

        Ok(())
    }

    /// Open dispute
    /// 
    /// Can be called by buyer or seller
    pub fn open_dispute(
        &self,
        ctx: &OperationContext,
        escrow_id: Uuid,
        reason: String,
    ) -> ServiceResult<Uuid> {
        // 1. VALIDATE
        let escrow = self.ctx.repos.escrow.get(&escrow_id)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to get escrow: {}", e),
            })?
            .ok_or_else(|| ServiceError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;

        // Must be buyer or seller
        let is_buyer = escrow.buyer_agent_uuid == ctx.caller.agent_uuid;
        let is_seller = escrow.seller_agent_uuid == ctx.caller.agent_uuid;
        
        if !is_buyer && !is_seller {
            return Err(ServiceError::Unauthorized {
                reason: "Only buyer or seller can open dispute".to_string(),
            });
        }

        // Can only dispute funded or in-escrow states
        if !matches!(escrow.escrow_status, EscrowStatus::Funded | EscrowStatus::InEscrow) {
            return Err(ServiceError::InvalidTransition {
                from: format!("{:?}", escrow.escrow_status),
                to: "Disputed".to_string(),
            });
        }

        // 2. MUTATE: Open dispute
        // 3. PERSIST
        self.ctx.repos.escrow.open_dispute(&escrow_id, &reason)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to open dispute: {}", e),
            })?;

        // 4. JOURNAL
        let journal_entry = crate::storage::journal::JournalEntry {
            tx_type: TxType::OpenDispute.as_str().to_string(),
            entity_type: EntityType::Escrow.as_str().to_string(),
            entity_id: escrow_id.to_string(),
            payload: serde_json::json!({
                "reason": reason,
            }),
            actor_uuid: Some(ctx.caller.agent_uuid),
        };
        
        if let Err(e) = self.ctx.journal.append(journal_entry) {
            eprintln!("[JOURNAL] Failed to append: {}", e);
        }

        // 5. DIBL EMIT
        let event = crate::governance::GovernanceEvent::new(
            &format!("escrow-{}", escrow_id),
            crate::governance::GovernanceEventType::RiskRaised,
            format!("Dispute opened for escrow {}: {}", escrow_id, reason),
        )
        .with_correlation(crate::governance::CorrelationContext {
            correlation_id: Some(ctx.correlation_id.clone()),
            parent_event_id: None,
            actor: ctx.caller.agent_id.clone(),
            trigger_context: Some("dispute opened".to_string()),
        });

        if let Err(e) = self.ctx.dibl.publish(event) {
            eprintln!("[DIBL] Failed to emit dispute opened event: {}", e);
        }

        Ok(escrow_id)
    }

    /// Resolve dispute (arbitration)
    /// 
    /// Only authorized arbitrators
    pub fn resolve_dispute(
        &self,
        ctx: &OperationContext,
        escrow_id: Uuid,
        resolution: DisputeResolution,
    ) -> ServiceResult<()> {
        // 1. VALIDATE: Caller must be authorized arbitrator
        if !ctx.caller.has_permission(Permission::ResolveDispute) {
            return Err(ServiceError::Unauthorized {
                reason: "Only arbitrators can resolve disputes".to_string(),
            });
        }

        let escrow = self.ctx.repos.escrow.get(&escrow_id)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to get escrow: {}", e),
            })?
            .ok_or_else(|| ServiceError::NotFound {
                entity_type: "Escrow".to_string(),
                id: escrow_id.to_string(),
            })?;

        if !matches!(escrow.escrow_status, EscrowStatus::Disputed) {
            return Err(ServiceError::InvalidTransition {
                from: format!("{:?}", escrow.escrow_status),
                to: "Resolved".to_string(),
            });
        }

        // 2. MUTATE: Apply resolution
        let resolution_str = match &resolution {
            DisputeResolution::RefundBuyer => "refund_buyer",
            DisputeResolution::ReleaseToSeller => "release_seller",
            DisputeResolution::Split { .. } => "split",
        };

        // 3. PERSIST
        self.ctx.repos.escrow.resolve_dispute(&escrow_id, resolution_str)
            .map_err(|e| ServiceError::Internal {
                message: format!("Failed to resolve dispute: {}", e),
            })?;

        // 4. JOURNAL
        let journal_entry = crate::storage::journal::JournalEntry {
            tx_type: TxType::ResolveDispute.as_str().to_string(),
            entity_type: EntityType::Escrow.as_str().to_string(),
            entity_id: escrow_id.to_string(),
            payload: serde_json::json!({
                "resolution": resolution_str,
            }),
            actor_uuid: Some(ctx.caller.agent_uuid),
        };
        
        if let Err(e) = self.ctx.journal.append(journal_entry) {
            eprintln!("[JOURNAL] Failed to append: {}", e);
        }

        // 5. DIBL EMIT
        let event = crate::governance::GovernanceEvent::new(
            &format!("escrow-{}", escrow_id),
            crate::governance::GovernanceEventType::DecisionCommitted,
            format!("Dispute resolved for escrow {}: {}", escrow_id, resolution_str),
        )
        .with_correlation(crate::governance::CorrelationContext {
            correlation_id: Some(ctx.correlation_id.clone()),
            parent_event_id: None,
            actor: ctx.caller.agent_id.clone(),
            trigger_context: Some("dispute resolved".to_string()),
        });

        if let Err(e) = self.ctx.dibl.publish(event) {
            eprintln!("[DIBL] Failed to emit dispute resolved event: {}", e);
        }

        Ok(())
    }
}

/// Dispute resolution options
#[derive(Debug, Clone)]
pub enum DisputeResolution {
    /// Full refund to buyer
    RefundBuyer,
    /// Full release to seller
    ReleaseToSeller,
    /// Split amount (buyer gets refund_amount)
    Split { refund_amount: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fund_escrow_creates_id() {
        let ctx = Arc::new(ServiceContext::new_test());
        let service = EscrowService::new(ctx);
        let op_ctx = OperationContext::new(super::super::types::Caller::system());

        let escrow_id = service.fund_escrow(
            &op_ctx, 
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            1000
        ).unwrap();
        assert!(!escrow_id.is_nil());
    }
}
