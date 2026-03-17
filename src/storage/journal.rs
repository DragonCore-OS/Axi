use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Transaction types for the journal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxType {
    CreateAgent,
    UpdateAgentStatus,
    UpdateAgentReputation,
    CreateWallet,
    CreateListing,
    UpdateListingStatus,
    CreateOrder,
    UpdateOrderStatus,
    CreateEscrow,
    UpdateEscrowStatus,
    SubmitDelivery,
    VerifyDelivery,
    OpenDispute,
    ResolveDispute,
    RecordReputation,
}

impl TxType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxType::CreateAgent => "CreateAgent",
            TxType::UpdateAgentStatus => "UpdateAgentStatus",
            TxType::UpdateAgentReputation => "UpdateAgentReputation",
            TxType::CreateWallet => "CreateWallet",
            TxType::CreateListing => "CreateListing",
            TxType::UpdateListingStatus => "UpdateListingStatus",
            TxType::CreateOrder => "CreateOrder",
            TxType::UpdateOrderStatus => "UpdateOrderStatus",
            TxType::CreateEscrow => "CreateEscrow",
            TxType::UpdateEscrowStatus => "UpdateEscrowStatus",
            TxType::SubmitDelivery => "SubmitDelivery",
            TxType::VerifyDelivery => "VerifyDelivery",
            TxType::OpenDispute => "OpenDispute",
            TxType::ResolveDispute => "ResolveDispute",
            TxType::RecordReputation => "RecordReputation",
        }
    }
}

/// Entity types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Agent,
    Wallet,
    Listing,
    Order,
    Escrow,
    ReputationEvent,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Agent => "Agent",
            EntityType::Wallet => "Wallet",
            EntityType::Listing => "Listing",
            EntityType::Order => "Order",
            EntityType::Escrow => "Escrow",
            EntityType::ReputationEvent => "ReputationEvent",
        }
    }
}

/// Transaction record in the journal
#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_id: i64,
    pub tx_uuid: Uuid,
    pub tx_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload: String,
    pub actor_uuid: Option<Uuid>,
    pub tx_hash: String,
    pub prev_tx_id: Option<i64>,
    pub created_at: String,
}

/// Journal entry before persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub tx_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload: serde_json::Value,
    pub actor_uuid: Option<Uuid>,
}

/// Transaction journal repository
pub struct TransactionJournal<'a> {
    conn: &'a Connection,
}

impl<'a> TransactionJournal<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Append a new transaction to the journal
    pub fn append(&self, entry: JournalEntry) -> Result<Transaction, String> {
        let tx_uuid = Uuid::new_v4();
        let created_at = Utc::now().to_rfc3339();
        let payload_json = serde_json::to_string(&entry.payload).map_err(|e| e.to_string())?;

        // Get the previous transaction for hash chaining
        let prev_tx = self.get_last_transaction()?;
        let prev_tx_id = prev_tx.as_ref().map(|t| t.tx_id);

        // Compute transaction hash (chain integrity)
        let tx_hash = self.compute_hash(prev_tx.as_ref(), &tx_uuid, &payload_json, &created_at);

        self.conn.execute(
            r#"
            INSERT INTO transaction_journal (
                tx_uuid, tx_type, entity_type, entity_id, payload,
                actor_uuid, tx_hash, prev_tx_id, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                tx_uuid.to_string(),
                entry.tx_type,
                entry.entity_type,
                entry.entity_id,
                payload_json,
                entry.actor_uuid.map(|u| u.to_string()),
                tx_hash,
                prev_tx_id,
                created_at,
            ],
        ).map_err(|e| e.to_string())?;

        let tx_id = self.conn.last_insert_rowid();

        Ok(Transaction {
            tx_id,
            tx_uuid,
            tx_type: entry.tx_type,
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            payload: payload_json,
            actor_uuid: entry.actor_uuid,
            tx_hash,
            prev_tx_id,
            created_at,
        })
    }

    /// Get transaction by ID
    pub fn get_by_id(&self, tx_id: i64) -> Result<Option<Transaction>, String> {
        self.conn.query_row(
            "SELECT * FROM transaction_journal WHERE tx_id = ?1",
            params![tx_id],
            |row| self.parse_tx_row(row),
        ).optional().map_err(|e| e.to_string())
    }

    /// Get transaction by UUID
    pub fn get_by_uuid(&self, tx_uuid: &Uuid) -> Result<Option<Transaction>, String> {
        self.conn.query_row(
            "SELECT * FROM transaction_journal WHERE tx_uuid = ?1",
            params![tx_uuid.to_string()],
            |row| self.parse_tx_row(row),
        ).optional().map_err(|e| e.to_string())
    }

    /// Get the last transaction (for hash chaining)
    pub fn get_last_transaction(&self) -> Result<Option<Transaction>, String> {
        self.conn.query_row(
            "SELECT * FROM transaction_journal ORDER BY tx_id DESC LIMIT 1",
            [],
            |row| self.parse_tx_row(row),
        ).optional().map_err(|e| e.to_string())
    }

    /// List transactions for a specific entity
    pub fn list_by_entity(&self, entity_type: &str, entity_id: &str) -> Result<Vec<Transaction>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM transaction_journal WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY tx_id"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(
            params![entity_type, entity_id],
            |row| self.parse_tx_row(row),
        ).map_err(|e| e.to_string())?;

        let mut txs = Vec::new();
        for row in rows {
            txs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(txs)
    }

    /// List transactions by type
    pub fn list_by_type(&self, tx_type: &str, limit: usize) -> Result<Vec<Transaction>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM transaction_journal WHERE tx_type = ?1 ORDER BY tx_id DESC LIMIT ?2"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(
            params![tx_type, limit as i64],
            |row| self.parse_tx_row(row),
        ).map_err(|e| e.to_string())?;

        let mut txs = Vec::new();
        for row in rows {
            txs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(txs)
    }

    /// Get transactions since a specific tx_id (for replay)
    pub fn since(&self, since_tx_id: i64) -> Result<Vec<Transaction>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM transaction_journal WHERE tx_id > ?1 ORDER BY tx_id"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(
            params![since_tx_id],
            |row| self.parse_tx_row(row),
        ).map_err(|e| e.to_string())?;

        let mut txs = Vec::new();
        for row in rows {
            txs.push(row.map_err(|e| e.to_string())?);
        }
        Ok(txs)
    }

    /// Set a watermark (checkpoint)
    pub fn set_watermark(&self, name: &str, tx_id: i64, tx_uuid: &Uuid) -> Result<(), String> {
        self.conn.execute(
            r#"
            INSERT INTO journal_watermarks (watermark_name, tx_id, tx_uuid, created_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(watermark_name) DO UPDATE SET
                tx_id = excluded.tx_id,
                tx_uuid = excluded.tx_uuid,
                created_at = excluded.created_at
            "#,
            params![name, tx_id, tx_uuid.to_string(), Utc::now().to_rfc3339()],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Get a watermark
    pub fn get_watermark(&self, name: &str) -> Result<Option<(i64, Uuid)>, String> {
        let row: Option<(i64, String)> = self.conn.query_row(
            "SELECT tx_id, tx_uuid FROM journal_watermarks WHERE watermark_name = ?1",
            params![name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(|e| e.to_string())?;

        match row {
            Some((tx_id, uuid_str)) => {
                let uuid = Uuid::parse_str(&uuid_str).map_err(|e| e.to_string())?;
                Ok(Some((tx_id, uuid)))
            }
            None => Ok(None),
        }
    }

    /// Verify chain integrity from a starting point
    pub fn verify_chain(&self, from_tx_id: i64) -> Result<bool, String> {
        let txs = self.since(from_tx_id - 1)?;
        
        for (i, tx) in txs.iter().enumerate() {
            if i == 0 {
                // First tx in range, just verify it exists
                continue;
            }
            
            let prev_tx = &txs[i - 1];
            if tx.prev_tx_id != Some(prev_tx.tx_id) {
                return Ok(false);
            }
            
            // Recompute hash and verify
            let expected_hash = self.compute_hash(Some(prev_tx), &tx.tx_uuid, &tx.payload, &tx.created_at);
            if tx.tx_hash != expected_hash {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    fn compute_hash(&self, prev_tx: Option<&Transaction>, tx_uuid: &Uuid, payload: &str, created_at: &str) -> String {
        let mut hasher = Sha256::new();
        
        // Include previous hash for chain integrity
        if let Some(prev) = prev_tx {
            hasher.update(prev.tx_hash.as_bytes());
        } else {
            hasher.update(b"genesis");
        }
        
        hasher.update(tx_uuid.to_string().as_bytes());
        hasher.update(payload.as_bytes());
        hasher.update(created_at.as_bytes());
        
        hex::encode(hasher.finalize())
    }

    fn parse_tx_row(&self, row: &Row) -> Result<Transaction, rusqlite::Error> {
        Ok(Transaction {
            tx_id: row.get(0)?,
            tx_uuid: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
            tx_type: row.get(2)?,
            entity_type: row.get(3)?,
            entity_id: row.get(4)?,
            payload: row.get(5)?,
            actor_uuid: row.get::<_, Option<String>>(6)?.map(|s| Uuid::parse_str(&s).unwrap()),
            tx_hash: row.get(7)?,
            prev_tx_id: row.get(8)?,
            created_at: row.get(9)?,
        })
    }
}

/// Helper to create journal entries
pub struct JournalEntryBuilder {
    entry: JournalEntry,
}

impl JournalEntryBuilder {
    pub fn new(tx_type: TxType, entity_type: EntityType, entity_id: &str) -> Self {
        Self {
            entry: JournalEntry {
                tx_type: tx_type.as_str().to_string(),
                entity_type: entity_type.as_str().to_string(),
                entity_id: entity_id.to_string(),
                payload: serde_json::json!({}),
                actor_uuid: None,
            },
        }
    }

    pub fn payload(mut self, payload: serde_json::Value) -> Self {
        self.entry.payload = payload;
        self
    }

    pub fn actor(mut self, actor_uuid: Uuid) -> Self {
        self.entry.actor_uuid = Some(actor_uuid);
        self
    }

    pub fn build(self) -> JournalEntry {
        self.entry
    }
}
