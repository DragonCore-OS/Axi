use crate::identity::registry::{AgentIdentity, AgentStatus, WalletRef, WalletRole, WalletType};
use crate::identity::reputation::{ReputationEvent, ReputationEventType};
use crate::market::order::{Order, OrderStatus};
use crate::market::escrow::{DeliveryProof, EscrowRecord, EscrowStatus};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

// =====================================================
// AGENT REPOSITORY
// =====================================================

pub struct AgentRepository<'a> {
    conn: &'a Connection,
}

impl<'a> AgentRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, agent: &AgentIdentity) -> Result<(), String> {
        self.conn.execute(
            r#"
            INSERT INTO agents (
                agent_uuid, agent_id, display_name, public_key,
                representative_record_commitment, comparison_commitment,
                reputation_score, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                agent.agent_uuid.to_string(),
                agent.agent_id,
                agent.display_name,
                agent.public_key,
                agent.representative_record_commitment,
                agent.comparison_commitment,
                agent.reputation_score,
                format!("{:?}", agent.status),
                format!("{}", agent.created_at),
                format!("{}", agent.created_at),
            ],
        ).map_err(|e| e.to_string())?;

        // Insert wallets
        for wallet in &agent.wallets {
            self.conn.execute(
                r#"
                INSERT INTO wallets (
                    wallet_id, agent_uuid, wallet_type, address, role,
                    verified_ownership, added_at, active_until
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    wallet.wallet_id.to_string(),
                    wallet.agent_uuid.to_string(),
                    format!("{:?}", wallet.wallet_type),
                    wallet.address,
                    format!("{:?}", wallet.role),
                    wallet.verified_ownership,
                    format!("{}", wallet.added_at),
                    wallet.active_until.map(|t| format!("{}", t)),
                ],
            ).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Result<Option<AgentIdentity>, String> {
        let agent_row = self.conn.query_row(
            "SELECT * FROM agents WHERE agent_uuid = ?1",
            params![uuid.to_string()],
            |row| self.parse_agent_row(row),
        ).optional().map_err(|e| e.to_string())?;

        let Some(mut agent) = agent_row else {
            return Ok(None);
        };

        // Load wallets
        agent.wallets = self.load_wallets(uuid)?;
        Ok(Some(agent))
    }

    pub fn get_by_agent_id(&self, agent_id: &str) -> Result<Option<AgentIdentity>, String> {
        let uuid: Option<String> = self.conn.query_row(
            "SELECT agent_uuid FROM agents WHERE agent_id = ?1",
            params![agent_id],
            |row| row.get(0),
        ).optional().map_err(|e| e.to_string())?;

        match uuid {
            Some(u) => self.get_by_uuid(&Uuid::parse_str(&u).map_err(|e| e.to_string())?),
            None => Ok(None),
        }
    }

    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<AgentIdentity>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM agents ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(
            params![limit as i64, offset as i64],
            |row| self.parse_agent_row(row),
        ).map_err(|e| e.to_string())?;

        let mut agents = Vec::new();
        for row in rows {
            let mut agent = row.map_err(|e| e.to_string())?;
            agent.wallets = self.load_wallets(&agent.agent_uuid)?;
            agents.push(agent);
        }
        Ok(agents)
    }

    pub fn update_status(&self, uuid: &Uuid, status: AgentStatus) -> Result<(), String> {
        self.conn.execute(
            "UPDATE agents SET status = ?1, updated_at = ?2 WHERE agent_uuid = ?3",
            params![format!("{:?}", status), Utc::now().to_rfc3339(), uuid.to_string()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_reputation_score(&self, uuid: &Uuid, delta: i64) -> Result<(), String> {
        self.conn.execute(
            "UPDATE agents SET reputation_score = reputation_score + ?1, updated_at = ?2 WHERE agent_uuid = ?3",
            params![delta, Utc::now().to_rfc3339(), uuid.to_string()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_wallet(&self, wallet: &WalletRef) -> Result<(), String> {
        self.conn.execute(
            r#"
            INSERT INTO wallets (
                wallet_id, agent_uuid, wallet_type, address, role,
                verified_ownership, added_at, active_until
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                wallet.wallet_id.to_string(),
                wallet.agent_uuid.to_string(),
                format!("{:?}", wallet.wallet_type),
                wallet.address,
                format!("{:?}", wallet.role),
                wallet.verified_ownership,
                format!("{}", wallet.added_at),
                wallet.active_until.map(|t| format!("{}", t)),
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn parse_agent_row(&self, row: &Row) -> Result<AgentIdentity, rusqlite::Error> {
        Ok(AgentIdentity {
            agent_uuid: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            agent_id: row.get(1)?,
            display_name: row.get(2)?,
            public_key: row.get(3)?,
            representative_record_commitment: row.get(4)?,
            comparison_commitment: row.get(5)?,
            reputation_score: row.get(6)?,
            status: parse_agent_status(&row.get::<_, String>(7)?),
            wallets: vec![], // Loaded separately
            created_at: row.get::<_, String>(8)?.parse().unwrap_or(0),
        })
    }

    fn load_wallets(&self, agent_uuid: &Uuid) -> Result<Vec<WalletRef>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM wallets WHERE agent_uuid = ?1 AND active_until IS NULL"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(
            params![agent_uuid.to_string()],
            |row| Ok(WalletRef {
                wallet_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                agent_uuid: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                agent_id: String::new(), // Filled by caller if needed
                wallet_type: parse_wallet_type(&row.get::<_, String>(2)?),
                address: row.get(3)?,
                role: parse_wallet_role(&row.get::<_, String>(4)?),
                verified_ownership: row.get(5)?,
                added_at: row.get::<_, String>(6)?.parse().unwrap_or(0),
                active_until: row.get::<_, Option<String>>(7)?.map(|s| s.parse().unwrap_or(0)),
            }),
        ).map_err(|e| e.to_string())?;

        let mut wallets = Vec::new();
        for row in rows {
            wallets.push(row.map_err(|e| e.to_string())?);
        }
        Ok(wallets)
    }
}

// =====================================================
// ORDER REPOSITORY
// =====================================================

pub struct OrderRepository<'a> {
    conn: &'a Connection,
}

impl<'a> OrderRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, order: &Order) -> Result<(), String> {
        self.conn.execute(
            r#"
            INSERT INTO orders (
                order_id, listing_id, buyer_agent_uuid, seller_agent_uuid,
                amount_axi, amount_locked_axi, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                order.order_id.to_string(),
                order.listing_id.to_string(),
                order.buyer_agent_uuid.to_string(),
                order.seller_agent_uuid.to_string(),
                order.amount_axi as i64,
                order.amount_locked_axi as i64,
                format!("{:?}", order.status),
                order.created_at,
                order.updated_at,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get(&self, order_id: &Uuid) -> Result<Option<Order>, String> {
        self.conn.query_row(
            "SELECT * FROM orders WHERE order_id = ?1",
            params![order_id.to_string()],
            |row| self.parse_order_row(row),
        ).optional().map_err(|e| e.to_string())
    }

    pub fn list_by_buyer(&self, buyer_uuid: &Uuid) -> Result<Vec<Order>, String> {
        self.list_by_field("buyer_agent_uuid", buyer_uuid)
    }

    pub fn list_by_seller(&self, seller_uuid: &Uuid) -> Result<Vec<Order>, String> {
        self.list_by_field("seller_agent_uuid", seller_uuid)
    }

    pub fn update_status(&self, order_id: &Uuid, status: OrderStatus) -> Result<(), String> {
        self.conn.execute(
            "UPDATE orders SET status = ?1, updated_at = ?2 WHERE order_id = ?3",
            params![format!("{:?}", status), Utc::now().to_rfc3339(), order_id.to_string()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn list_by_field(&self, field: &str, uuid: &Uuid) -> Result<Vec<Order>, String> {
        let sql = format!(
            "SELECT * FROM orders WHERE {} = ?1 ORDER BY created_at DESC",
            field
        );
        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(
            params![uuid.to_string()],
            |row| self.parse_order_row(row),
        ).map_err(|e| e.to_string())?;

        let mut orders = Vec::new();
        for row in rows {
            orders.push(row.map_err(|e| e.to_string())?);
        }
        Ok(orders)
    }

    fn parse_order_row(&self, row: &Row) -> Result<Order, rusqlite::Error> {
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
    }
}

// =====================================================
// ESCROW REPOSITORY
// =====================================================

pub struct EscrowRepository<'a> {
    conn: &'a Connection,
}

impl<'a> EscrowRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, escrow: &EscrowRecord) -> Result<(), String> {
        self.conn.execute(
            r#"
            INSERT INTO escrows (
                escrow_id, order_id, buyer_agent_uuid, seller_agent_uuid, amount_axi,
                escrow_status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                escrow.escrow_id.to_string(),
                escrow.order_id.to_string(),
                escrow.buyer_agent_uuid.to_string(),
                escrow.seller_agent_uuid.to_string(),
                escrow.amount_axi as i64,
                format!("{:?}", escrow.escrow_status),
                escrow.created_at,
                escrow.updated_at,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get(&self, escrow_id: &Uuid) -> Result<Option<EscrowRecord>, String> {
        self.conn.query_row(
            "SELECT * FROM escrows WHERE escrow_id = ?1",
            params![escrow_id.to_string()],
            |row| self.parse_escrow_row(row),
        ).optional().map_err(|e| e.to_string())
    }

    pub fn get_by_order(&self, order_id: &Uuid) -> Result<Option<EscrowRecord>, String> {
        self.conn.query_row(
            "SELECT * FROM escrows WHERE order_id = ?1",
            params![order_id.to_string()],
            |row| self.parse_escrow_row(row),
        ).optional().map_err(|e| e.to_string())
    }

    pub fn update_status(&self, escrow_id: &Uuid, status: EscrowStatus) -> Result<(), String> {
        self.conn.execute(
            "UPDATE escrows SET escrow_status = ?1, updated_at = ?2 WHERE escrow_id = ?3",
            params![format!("{:?}", status), Utc::now().to_rfc3339(), escrow_id.to_string()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn submit_delivery(
        &self,
        escrow_id: &Uuid,
        proof: &DeliveryProof,
    ) -> Result<(), String> {
        self.conn.execute(
            r#"
            UPDATE escrows SET
                delivery_cid = ?1,
                delivery_uri = ?2,
                delivery_note = ?3,
                delivery_submitted_at = ?4,
                auto_complete_after = ?5,
                updated_at = ?6
            WHERE escrow_id = ?7
            "#,
            params![
                proof.cid,
                proof.uri,
                proof.note,
                proof.submitted_at,
                Utc::now().checked_add_signed(chrono::Duration::hours(24)).map(|t| t.to_rfc3339()),
                Utc::now().to_rfc3339(),
                escrow_id.to_string(),
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn buyer_verify(&self, escrow_id: &Uuid) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE escrows SET buyer_verified_at = ?1, updated_at = ?2 WHERE escrow_id = ?3",
            params![now.clone(), now, escrow_id.to_string()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn open_dispute(&self, escrow_id: &Uuid, reason: &str) -> Result<(), String> {
        self.conn.execute(
            "UPDATE escrows SET dispute_reason = ?1, updated_at = ?2 WHERE escrow_id = ?3",
            params![reason, Utc::now().to_rfc3339(), escrow_id.to_string()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn parse_escrow_row(&self, row: &Row) -> Result<EscrowRecord, rusqlite::Error> {
        Ok(EscrowRecord {
            escrow_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            order_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
            buyer_agent_uuid: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap(),
            seller_agent_uuid: Uuid::parse_str(&row.get::<_, String>(3)?).unwrap(),
            amount_axi: row.get::<_, i64>(4)? as u64,
            escrow_status: parse_escrow_status(&row.get::<_, String>(5)?),
            delivery_proof: match row.get::<_, Option<String>>(6)? {
                Some(_) => Some(DeliveryProof {
                    cid: row.get(6)?,
                    uri: row.get(7)?,
                    note: row.get(8)?,
                    submitted_at: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                }),
                None => None,
            },
            buyer_verified_at: row.get(10)?,
            auto_complete_after: row.get(11)?,
            dispute_reason: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }
}

// =====================================================
// REPUTATION REPOSITORY
// =====================================================

pub struct ReputationRepository<'a> {
    conn: &'a Connection,
}

impl<'a> ReputationRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn record_event(&self, event: &ReputationEvent) -> Result<(), String> {
        self.conn.execute(
            r#"
            INSERT INTO reputation_events (
                event_id, agent_uuid, order_id, event_type, delta, reason, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                event.event_id.to_string(),
                event.agent_uuid.to_string(),
                event.order_id.map(|u| u.to_string()),
                format!("{:?}", event.event_type),
                event.delta,
                event.reason,
                event.created_at,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_by_agent(&self, agent_uuid: &Uuid) -> Result<Vec<ReputationEvent>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM reputation_events WHERE agent_uuid = ?1 ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(
            params![agent_uuid.to_string()],
            |row| self.parse_event_row(row),
        ).map_err(|e| e.to_string())?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| e.to_string())?);
        }
        Ok(events)
    }

    pub fn get_score(&self, agent_uuid: &Uuid) -> Result<i64, String> {
        self.conn.query_row(
            "SELECT reputation_score FROM agents WHERE agent_uuid = ?1",
            params![agent_uuid.to_string()],
            |row| row.get(0),
        ).map_err(|e| e.to_string())
    }

    fn parse_event_row(&self, row: &Row) -> Result<ReputationEvent, rusqlite::Error> {
        Ok(ReputationEvent {
            event_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            agent_uuid: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
            order_id: row.get::<_, Option<String>>(2)?.map(|s| Uuid::parse_str(&s).unwrap()),
            event_type: parse_reputation_event_type(&row.get::<_, String>(3)?),
            delta: row.get(4)?,
            reason: row.get(5)?,
            created_at: row.get(6)?,
        })
    }
}

// =====================================================
// PARSING HELPERS
// =====================================================

fn parse_agent_status(s: &str) -> AgentStatus {
    match s {
        "Approved" => AgentStatus::Approved,
        "Rejected" => AgentStatus::Rejected,
        "Suspended" => AgentStatus::Suspended,
        "Banned" => AgentStatus::Banned,
        _ => AgentStatus::Pending,
    }
}

fn parse_wallet_type(s: &str) -> WalletType {
    match s {
        "AxiNative" => WalletType::AxiNative,
        "Evm" => WalletType::Evm,
        "Btc" => WalletType::Btc,
        "Solana" => WalletType::Solana,
        _ => WalletType::Other,
    }
}

fn parse_wallet_role(s: &str) -> WalletRole {
    match s {
        "Primary" => WalletRole::Primary,
        "LegacyBridge" => WalletRole::LegacyBridge,
        _ => WalletRole::Secondary,
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

fn parse_reputation_event_type(s: &str) -> ReputationEventType {
    match s {
        "OrderCompleted" => ReputationEventType::OrderCompleted,
        "PositiveRating" => ReputationEventType::PositiveRating,
        "NegativeRating" => ReputationEventType::NegativeRating,
        _ => ReputationEventType::DisputeLost,
    }
}
