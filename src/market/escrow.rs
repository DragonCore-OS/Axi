use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::order::{Order, OrderStatus};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EscrowStatus {
    Pending,
    Funded,
    InEscrow,
    Released,
    Cancelled,
    Refunded,
    Disputed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryProof {
    pub cid: Option<String>,
    pub uri: Option<String>,
    pub note: Option<String>,
    pub submitted_at: String,
}

impl DeliveryProof {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.cid.is_none() && self.uri.is_none() {
            return Err("delivery proof requires cid or uri");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowRecord {
    pub escrow_id: Uuid,
    pub order_id: Uuid,
    pub buyer_agent_uuid: Uuid,
    pub seller_agent_uuid: Uuid,
    pub amount_axi: u64,
    pub escrow_status: EscrowStatus,
    pub delivery_proof: Option<DeliveryProof>,
    pub buyer_verified_at: Option<String>,
    pub auto_complete_after: Option<String>,
    pub dispute_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Default)]
pub struct EscrowService {
    escrows: HashMap<Uuid, EscrowRecord>,
}

impl EscrowService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_for_order(&mut self, order: &Order) -> Result<EscrowRecord, &'static str> {
        if order.amount_locked_axi == 0 {
            return Err("cannot create escrow for zero amount");
        }

        let now = Utc::now().to_rfc3339();
        let escrow = EscrowRecord {
            escrow_id: Uuid::new_v4(),
            order_id: order.order_id,
            buyer_agent_uuid: order.buyer_agent_uuid,
            seller_agent_uuid: order.seller_agent_uuid,
            amount_axi: order.amount_locked_axi,
            escrow_status: EscrowStatus::Pending,
            delivery_proof: None,
            buyer_verified_at: None,
            auto_complete_after: None,
            dispute_reason: None,
            created_at: now.clone(),
            updated_at: now,
        };

        self.escrows.insert(escrow.escrow_id, escrow.clone());
        Ok(escrow)
    }

    pub fn get(&self, escrow_id: &Uuid) -> Option<&EscrowRecord> {
        self.escrows.get(escrow_id)
    }

    pub fn fund(&mut self, escrow_id: &Uuid) -> Result<(), &'static str> {
        self.transition(escrow_id, EscrowStatus::Funded)
    }

    pub fn move_to_escrow(&mut self, escrow_id: &Uuid) -> Result<(), &'static str> {
        self.transition(escrow_id, EscrowStatus::InEscrow)
    }

    pub fn submit_delivery(
        &mut self,
        escrow_id: &Uuid,
        proof: DeliveryProof,
        order: &mut Order,
    ) -> Result<(), &'static str> {
        proof.validate()?;

        let escrow = self
            .escrows
            .get_mut(escrow_id)
            .ok_or("escrow not found")?;

        if escrow.escrow_status != EscrowStatus::InEscrow {
            return Err("delivery can only be submitted while in escrow");
        }

        escrow.delivery_proof = Some(proof);
        escrow.auto_complete_after = Some((Utc::now() + Duration::hours(24)).to_rfc3339());
        escrow.updated_at = Utc::now().to_rfc3339();

        order.transition(OrderStatus::Delivered)?;
        Ok(())
    }

    pub fn buyer_verify(
        &mut self,
        escrow_id: &Uuid,
        order: &mut Order,
    ) -> Result<(), &'static str> {
        let escrow = self
            .escrows
            .get_mut(escrow_id)
            .ok_or("escrow not found")?;

        if escrow.delivery_proof.is_none() {
            return Err("cannot verify before delivery proof exists");
        }
        if escrow.escrow_status != EscrowStatus::InEscrow {
            return Err("buyer verification requires in-escrow status");
        }

        escrow.escrow_status = EscrowStatus::Released;
        escrow.buyer_verified_at = Some(Utc::now().to_rfc3339());
        escrow.updated_at = Utc::now().to_rfc3339();

        order.transition(OrderStatus::Verified)?;
        Ok(())
    }

    pub fn auto_complete_if_due(
        &mut self,
        escrow_id: &Uuid,
        now_rfc3339: &str,
        order: &mut Order,
    ) -> Result<bool, &'static str> {
        let escrow = self
            .escrows
            .get_mut(escrow_id)
            .ok_or("escrow not found")?;

        if escrow.escrow_status != EscrowStatus::InEscrow {
            return Ok(false);
        }

        let due = match &escrow.auto_complete_after {
            Some(v) => v,
            None => return Ok(false),
        };

        if now_rfc3339 >= due {
            escrow.escrow_status = EscrowStatus::Released;
            escrow.updated_at = Utc::now().to_rfc3339();
            order.transition(OrderStatus::Verified)?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn open_dispute(
        &mut self,
        escrow_id: &Uuid,
        reason: String,
    ) -> Result<(), &'static str> {
        let escrow = self
            .escrows
            .get_mut(escrow_id)
            .ok_or("escrow not found")?;

        if escrow.delivery_proof.is_none() {
            return Err("cannot dispute before delivery proof exists");
        }
        if escrow.escrow_status != EscrowStatus::InEscrow {
            return Err("dispute requires in-escrow status");
        }

        escrow.escrow_status = EscrowStatus::Disputed;
        escrow.dispute_reason = Some(reason);
        escrow.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn refund(&mut self, escrow_id: &Uuid) -> Result<(), &'static str> {
        self.transition(escrow_id, EscrowStatus::Refunded)
    }

    fn transition(
        &mut self,
        escrow_id: &Uuid,
        next: EscrowStatus,
    ) -> Result<(), &'static str> {
        let escrow = self
            .escrows
            .get_mut(escrow_id)
            .ok_or("escrow not found")?;

        let valid = matches!(
            (&escrow.escrow_status, &next),
            (EscrowStatus::Pending, EscrowStatus::Funded)
                | (EscrowStatus::Pending, EscrowStatus::Cancelled)
                | (EscrowStatus::Funded, EscrowStatus::InEscrow)
                | (EscrowStatus::Funded, EscrowStatus::Refunded)
                | (EscrowStatus::InEscrow, EscrowStatus::Released)
                | (EscrowStatus::InEscrow, EscrowStatus::Refunded)
                | (EscrowStatus::InEscrow, EscrowStatus::Disputed)
                | (EscrowStatus::Disputed, EscrowStatus::Refunded)
                | (EscrowStatus::Disputed, EscrowStatus::Released)
        );

        if !valid {
            return Err("invalid escrow state transition");
        }

        escrow.escrow_status = next;
        escrow.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{
        ListingType, MarketService, PricingModel, SettlementMode,
    };

    fn setup_order() -> (Order, Uuid) {
        let mut market = MarketService::new();
        let seller = Uuid::new_v4();
        let buyer = Uuid::new_v4();

        let listing = market
            .create_listing(
                ListingType::Service,
                seller,
                "Inference".into(),
                "Inference API".into(),
                vec!["inference".into()],
                PricingModel::Fixed,
                Some(100),
                None,
                SettlementMode::Escrow,
            )
            .unwrap();

        let order = market
            .create_order_from_listing(&listing.listing_id, buyer, 100)
            .unwrap();

        (order, buyer)
    }

    #[test]
    fn escrow_state_machine_runs_to_release() {
        let (mut order, _) = setup_order();
        let mut service = EscrowService::new();

        let escrow = service.create_for_order(&order).unwrap();
        service.fund(&escrow.escrow_id).unwrap();
        service.move_to_escrow(&escrow.escrow_id).unwrap();

        // Move order to InProgress before delivery
        order.transition(OrderStatus::InProgress).unwrap();

        let proof = DeliveryProof {
            cid: Some("bafy123".into()),
            uri: None,
            note: Some("delivered".into()),
            submitted_at: Utc::now().to_rfc3339(),
        };

        service
            .submit_delivery(&escrow.escrow_id, proof, &mut order)
            .unwrap();
        service.buyer_verify(&escrow.escrow_id, &mut order).unwrap();

        assert_eq!(
            service.get(&escrow.escrow_id).unwrap().escrow_status,
            EscrowStatus::Released
        );
        assert_eq!(order.status, OrderStatus::Verified);
    }

    #[test]
    fn delivery_requires_cid_or_uri() {
        let (mut order, _) = setup_order();
        let mut service = EscrowService::new();

        let escrow = service.create_for_order(&order).unwrap();
        service.fund(&escrow.escrow_id).unwrap();
        service.move_to_escrow(&escrow.escrow_id).unwrap();

        let proof = DeliveryProof {
            cid: None,
            uri: None,
            note: None,
            submitted_at: Utc::now().to_rfc3339(),
        };

        let err = service
            .submit_delivery(&escrow.escrow_id, proof, &mut order)
            .unwrap_err();

        assert_eq!(err, "delivery proof requires cid or uri");
    }

    #[test]
    fn dispute_moves_escrow_to_disputed() {
        let (mut order, _) = setup_order();
        let mut service = EscrowService::new();

        let escrow = service.create_for_order(&order).unwrap();
        service.fund(&escrow.escrow_id).unwrap();
        service.move_to_escrow(&escrow.escrow_id).unwrap();

        // Move order to InProgress before delivery
        order.transition(OrderStatus::InProgress).unwrap();

        let proof = DeliveryProof {
            cid: Some("bafy123".into()),
            uri: None,
            note: None,
            submitted_at: Utc::now().to_rfc3339(),
        };

        service
            .submit_delivery(&escrow.escrow_id, proof, &mut order)
            .unwrap();
        service
            .open_dispute(&escrow.escrow_id, "bad output".into())
            .unwrap();

        assert_eq!(
            service.get(&escrow.escrow_id).unwrap().escrow_status,
            EscrowStatus::Disputed
        );
    }

    #[test]
    fn auto_complete_releases_when_due() {
        let (mut order, _) = setup_order();
        let mut service = EscrowService::new();

        let escrow = service.create_for_order(&order).unwrap();
        service.fund(&escrow.escrow_id).unwrap();
        service.move_to_escrow(&escrow.escrow_id).unwrap();

        // Move order to InProgress before delivery
        order.transition(OrderStatus::InProgress).unwrap();

        let proof = DeliveryProof {
            cid: None,
            uri: Some("https://example.com/output".into()),
            note: None,
            submitted_at: Utc::now().to_rfc3339(),
        };

        service
            .submit_delivery(&escrow.escrow_id, proof, &mut order)
            .unwrap();

        let completed = service
            .auto_complete_if_due(&escrow.escrow_id, "9999-12-31T23:59:59+00:00", &mut order)
            .unwrap();

        assert!(completed);
        assert_eq!(
            service.get(&escrow.escrow_id).unwrap().escrow_status,
            EscrowStatus::Released
        );
        assert_eq!(order.status, OrderStatus::Verified);
    }
}
