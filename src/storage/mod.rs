use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub mod repos;
pub mod journal;

pub(crate) use repos::{AgentRepository, OrderRepository, EscrowRepository, ReputationRepository};
pub use journal::{TransactionJournal, JournalEntry, JournalEntryBuilder, TxType, EntityType, Transaction};

const CURRENT_SCHEMA_VERSION: i64 = 4;

#[derive(Debug, Clone)]
pub struct PersistentStore {
    db_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SnapshotMeta {
    pub name: String,
    pub checksum: String,
    pub updated_at: String,
}

impl PersistentStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let db_path = path.as_ref().to_path_buf();
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        Self::run_migrations(&conn)?;
        Ok(Self { db_path })
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn schema_version(&self) -> Result<i64, String> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())
    }

    pub fn save_snapshot<T: Serialize>(&self, name: &str, value: &T) -> Result<SnapshotMeta, String> {
        let payload = serde_json::to_string(value).map_err(|e| e.to_string())?;
        let checksum = checksum_hex(&payload);
        let updated_at = Utc::now().to_rfc3339();

        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO state_snapshots (name, payload, checksum, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(name) DO UPDATE SET
                payload = excluded.payload,
                checksum = excluded.checksum,
                updated_at = excluded.updated_at
            "#,
            params![name, payload, checksum, updated_at],
        )
        .map_err(|e| e.to_string())?;

        Ok(SnapshotMeta {
            name: name.to_string(),
            checksum,
            updated_at,
        })
    }

    pub fn load_snapshot<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, String> {
        let conn = self.connect()?;
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT payload, checksum FROM state_snapshots WHERE name = ?1",
                params![name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let Some((payload, checksum)) = row else {
            return Ok(None);
        };

        let actual = checksum_hex(&payload);
        if actual != checksum {
            return Err("snapshot checksum mismatch".into());
        }

        let decoded = serde_json::from_str::<T>(&payload).map_err(|e| e.to_string())?;
        Ok(Some(decoded))
    }

    pub fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>, String> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare("SELECT name, checksum, updated_at FROM state_snapshots ORDER BY name")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SnapshotMeta {
                    name: row.get(0)?,
                    checksum: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn backup_to<P: AsRef<Path>>(&self, backup_path: P) -> Result<(), String> {
        let src = self.db_path();
        let dst = backup_path.as_ref();

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::copy(src, dst).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn connect(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|e| e.to_string())
    }

    fn run_migrations(conn: &Connection) -> Result<(), String> {
        // Base migration (v1)
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS state_snapshots (
                name TEXT PRIMARY KEY,
                payload TEXT NOT NULL,
                checksum TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .map_err(|e| e.to_string())?;

        let version: Option<i64> = conn
            .query_row(
                "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let current_version = version.unwrap_or(0);

        // Migration v1: Initial schema
        if current_version < 1 {
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![1, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        }

        // Migration v2: Relational schema for Identity/Market
        if current_version < 2 {
            conn.execute_batch(include_str!("schema_v2.sql"))
                .map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![2, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        }

        // Migration v3: Transaction journal
        if current_version < 3 {
            conn.execute_batch(include_str!("schema_v3.sql"))
                .map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![3, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        }

        // Migration v4: Feature flags
        if current_version < 4 {
            conn.execute_batch(include_str!("schema_v4.sql"))
                .map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![4, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

fn checksum_hex(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct DemoState {
        node_id: String,
        open_orders: u64,
        escrow_total_axi: u64,
    }

    fn temp_db_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("axi_{}_{}.db", label, Uuid::new_v4()))
    }

    #[test]
    fn initializes_schema_and_version() {
        let path = temp_db_path("init");
        let store = PersistentStore::open(&path).unwrap();
        let version = store.schema_version().unwrap();

        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn saves_and_loads_snapshot() {
        let path = temp_db_path("save_load");
        let store = PersistentStore::open(&path).unwrap();

        let state = DemoState {
            node_id: "axi-node-1".into(),
            open_orders: 3,
            escrow_total_axi: 125,
        };

        store.save_snapshot("runtime_state", &state).unwrap();
        let restored: Option<DemoState> = store.load_snapshot("runtime_state").unwrap();

        assert_eq!(restored, Some(state));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn backup_creates_copy() {
        let path = temp_db_path("backup_src");
        let backup = temp_db_path("backup_dst");
        let store = PersistentStore::open(&path).unwrap();

        let state = DemoState {
            node_id: "axi-node-2".into(),
            open_orders: 1,
            escrow_total_axi: 42,
        };

        store.save_snapshot("runtime_state", &state).unwrap();
        store.backup_to(&backup).unwrap();

        let backup_store = PersistentStore::open(&backup).unwrap();
        let restored: Option<DemoState> = backup_store.load_snapshot("runtime_state").unwrap();
        assert_eq!(restored, Some(state));

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(backup);
    }

    #[test]
    fn checksum_detects_tampered_snapshot() {
        let path = temp_db_path("tamper");
        let store = PersistentStore::open(&path).unwrap();

        let state = DemoState {
            node_id: "axi-node-3".into(),
            open_orders: 9,
            escrow_total_axi: 999,
        };

        store.save_snapshot("runtime_state", &state).unwrap();

        let conn = store.connect().unwrap();
        conn.execute(
            "UPDATE state_snapshots SET payload = ?1 WHERE name = ?2",
            params![r#"{"node_id":"evil","open_orders":0,"escrow_total_axi":0}"#, "runtime_state"],
        )
        .unwrap();

        let result: Result<Option<DemoState>, String> = store.load_snapshot("runtime_state");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("checksum mismatch"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn restart_recovers_last_known_state() {
        let path = temp_db_path("restart");
        {
            let store = PersistentStore::open(&path).unwrap();
            let state = DemoState {
                node_id: "axi-node-4".into(),
                open_orders: 7,
                escrow_total_axi: 300,
            };
            store.save_snapshot("runtime_state", &state).unwrap();
        }

        let restarted = PersistentStore::open(&path).unwrap();
        let restored: Option<DemoState> = restarted.load_snapshot("runtime_state").unwrap();

        assert_eq!(
            restored,
            Some(DemoState {
                node_id: "axi-node-4".into(),
                open_orders: 7,
                escrow_total_axi: 300,
            })
        );

        let _ = fs::remove_file(path);
    }

    // =====================================================
    // M2-2: Relational Schema Tests
    // =====================================================

    use crate::identity::registry::{AgentIdentity, AgentStatus, WalletRef, WalletRole, WalletType};
    use crate::market::order::{Order, OrderStatus};
    use crate::market::escrow::{EscrowRecord, EscrowStatus, DeliveryProof};
    use crate::identity::reputation::{ReputationEvent, ReputationEventType};

    fn create_test_agent() -> AgentIdentity {
        create_test_agent_with_uuid(Uuid::new_v4())
    }

    fn create_test_agent_with_uuid(agent_uuid: Uuid) -> AgentIdentity {
        AgentIdentity {
            agent_uuid,
            agent_id: format!("agent{}", &agent_uuid.to_string().replace("-", "")[..16]),
            display_name: "Test Agent".to_string(),
            public_key: "pubkey123".to_string(),
            representative_record_commitment: "commit1".to_string(),
            comparison_commitment: "commit2".to_string(),
            reputation_score: 0,
            status: AgentStatus::Pending,
            wallets: vec![WalletRef {
                wallet_id: Uuid::new_v4(),
                agent_uuid,
                agent_id: format!("agent{}", &agent_uuid.to_string().replace("-", "")[..16]),
                wallet_type: WalletType::AxiNative,
                address: format!("axi1{}", &agent_uuid.to_string().replace("-", "")[..20]),
                role: WalletRole::Primary,
                verified_ownership: true,
                added_at: 1234567890,
                active_until: None,
            }],
            created_at: 1234567890,
        }
    }

    #[test]
    fn v2_schema_relational_tables_exist() {
        let path = temp_db_path("v2_tables");
        let store = PersistentStore::open(&path).unwrap();
        // Schema version is now 3, but v2 tables should still exist
        let version = store.schema_version().unwrap();
        assert!(version >= 2);

        // Verify tables exist
        let conn = store.connect().unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"agents".to_string()));
        assert!(tables.contains(&"wallets".to_string()));
        assert!(tables.contains(&"orders".to_string()));
        assert!(tables.contains(&"escrows".to_string()));
        assert!(tables.contains(&"reputation_events".to_string()));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn agent_repo_crud() {
        let path = temp_db_path("agent_repo");
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        let repo = AgentRepository::new(&conn);

        let agent = create_test_agent();
        repo.create(&agent).unwrap();

        // Read by UUID
        let fetched = repo.get_by_uuid(&agent.agent_uuid).unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.agent_id, agent.agent_id);
        assert_eq!(fetched.wallets.len(), 1);

        // Read by agent_id
        let fetched = repo.get_by_agent_id(&agent.agent_id).unwrap();
        assert!(fetched.is_some());

        // Update status
        repo.update_status(&agent.agent_uuid, AgentStatus::Approved).unwrap();
        let updated = repo.get_by_uuid(&agent.agent_uuid).unwrap().unwrap();
        assert!(matches!(updated.status, AgentStatus::Approved));

        // Update reputation
        repo.update_reputation_score(&agent.agent_uuid, 10).unwrap();
        let updated = repo.get_by_uuid(&agent.agent_uuid).unwrap().unwrap();
        assert_eq!(updated.reputation_score, 10);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn order_repo_crud() {
        let path = temp_db_path("order_repo");
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();

        // Create agent first (FK constraint)
        let agent_repo = AgentRepository::new(&conn);
        let buyer = create_test_agent();
        let seller = create_test_agent_with_uuid(Uuid::new_v4());
        agent_repo.create(&buyer).unwrap();
        agent_repo.create(&seller).unwrap();

        // Create order
        let order_repo = OrderRepository::new(&conn);
        let order = Order {
            order_id: Uuid::new_v4(),
            listing_id: Uuid::new_v4(),
            buyer_agent_uuid: buyer.agent_uuid,
            seller_agent_uuid: seller.agent_uuid,
            amount_axi: 100,
            amount_locked_axi: 100,
            status: OrderStatus::Open,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        order_repo.create(&order).unwrap();

        // Read
        let fetched = order_repo.get(&order.order_id).unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().amount_axi, 100);

        // List by buyer
        let orders = order_repo.list_by_buyer(&buyer.agent_uuid).unwrap();
        assert_eq!(orders.len(), 1);

        // Update status
        order_repo.update_status(&order.order_id, OrderStatus::InProgress).unwrap();
        let updated = order_repo.get(&order.order_id).unwrap().unwrap();
        assert!(matches!(updated.status, OrderStatus::InProgress));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn escrow_repo_crud() {
        let path = temp_db_path("escrow_repo");
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();

        // Setup agents and order
        let agent_repo = AgentRepository::new(&conn);
        let buyer = create_test_agent();
        let seller = create_test_agent_with_uuid(Uuid::new_v4());
        agent_repo.create(&buyer).unwrap();
        agent_repo.create(&seller).unwrap();

        let order_repo = OrderRepository::new(&conn);
        let order = Order {
            order_id: Uuid::new_v4(),
            listing_id: Uuid::new_v4(),
            buyer_agent_uuid: buyer.agent_uuid,
            seller_agent_uuid: seller.agent_uuid,
            amount_axi: 200,
            amount_locked_axi: 200,
            status: OrderStatus::InProgress,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        order_repo.create(&order).unwrap();

        // Create escrow
        let escrow_repo = EscrowRepository::new(&conn);
        let escrow = EscrowRecord {
            escrow_id: Uuid::new_v4(),
            order_id: order.order_id,
            buyer_agent_uuid: buyer.agent_uuid,
            seller_agent_uuid: seller.agent_uuid,
            amount_axi: 200,
            escrow_status: EscrowStatus::Pending,
            delivery_proof: None,
            buyer_verified_at: None,
            auto_complete_after: None,
            dispute_reason: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        escrow_repo.create(&escrow).unwrap();

        // Read
        let fetched = escrow_repo.get(&escrow.escrow_id).unwrap();
        assert!(fetched.is_some());

        // Read by order
        let fetched = escrow_repo.get_by_order(&order.order_id).unwrap();
        assert!(fetched.is_some());

        // Status transition
        escrow_repo.update_status(&escrow.escrow_id, EscrowStatus::Funded).unwrap();
        let updated = escrow_repo.get(&escrow.escrow_id).unwrap().unwrap();
        assert!(matches!(updated.escrow_status, EscrowStatus::Funded));

        // Submit delivery
        let proof = DeliveryProof {
            cid: Some("QmTest123".to_string()),
            uri: Some("https://example.com/delivery".to_string()),
            note: Some("Delivered".to_string()),
            submitted_at: Utc::now().to_rfc3339(),
        };
        escrow_repo.submit_delivery(&escrow.escrow_id, &proof).unwrap();
        let updated = escrow_repo.get(&escrow.escrow_id).unwrap().unwrap();
        assert!(updated.delivery_proof.is_some());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn reputation_repo_audit_log() {
        let path = temp_db_path("reputation_repo");
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();

        // Setup agent and order (FK constraints)
        let agent_repo = AgentRepository::new(&conn);
        let agent = create_test_agent();
        agent_repo.create(&agent).unwrap();

        let order_repo = OrderRepository::new(&conn);
        let order = Order {
            order_id: Uuid::new_v4(),
            listing_id: Uuid::new_v4(),
            buyer_agent_uuid: agent.agent_uuid,
            seller_agent_uuid: agent.agent_uuid,
            amount_axi: 100,
            amount_locked_axi: 100,
            status: OrderStatus::Open,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        order_repo.create(&order).unwrap();

        // Record events
        let rep_repo = ReputationRepository::new(&conn);
        let event1 = ReputationEvent {
            event_id: Uuid::new_v4(),
            agent_uuid: agent.agent_uuid,
            order_id: None,
            event_type: ReputationEventType::OrderCompleted,
            delta: 5,
            reason: "Order completed successfully".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        rep_repo.record_event(&event1).unwrap();

        let event2 = ReputationEvent {
            event_id: Uuid::new_v4(),
            agent_uuid: agent.agent_uuid,
            order_id: Some(order.order_id),
            event_type: ReputationEventType::PositiveRating,
            delta: 2,
            reason: "5-star rating".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        rep_repo.record_event(&event2).unwrap();

        // List events
        let events = rep_repo.list_by_agent(&agent.agent_uuid).unwrap();
        assert_eq!(events.len(), 2);

        let _ = fs::remove_file(path);
    }

    // =====================================================
    // M2-3: Transaction Journal Tests
    // =====================================================

    #[test]
    fn v3_schema_migrates_to_at_least_version_3() {
        let path = temp_db_path("v3_migration");
        let store = PersistentStore::open(&path).unwrap();
        let version = store.schema_version().unwrap();

        // Schema is now at v4, but v3 tables should exist
        assert!(version >= 3);

        // Verify journal table exists
        let conn = store.connect().unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"transaction_journal".to_string()));
        assert!(tables.contains(&"journal_watermarks".to_string()));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn journal_appends_and_chains_transactions() {
        let path = temp_db_path("journal_chain");
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        let journal = TransactionJournal::new(&conn);

        // Append first transaction
        let entry1 = JournalEntryBuilder::new(
            TxType::CreateAgent,
            EntityType::Agent,
            &Uuid::new_v4().to_string()
        )
        .payload(serde_json::json!({"name": "Agent 1"}))
        .build();
        
        let tx1 = journal.append(entry1).unwrap();
        assert_eq!(tx1.tx_id, 1);
        assert!(tx1.prev_tx_id.is_none());
        assert!(!tx1.tx_hash.is_empty());

        // Append second transaction
        let entry2 = JournalEntryBuilder::new(
            TxType::CreateOrder,
            EntityType::Order,
            &Uuid::new_v4().to_string()
        )
        .payload(serde_json::json!({"amount": 100}))
        .build();
        
        let tx2 = journal.append(entry2).unwrap();
        assert_eq!(tx2.tx_id, 2);
        assert_eq!(tx2.prev_tx_id, Some(1));
        assert!(!tx2.tx_hash.is_empty());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn journal_queries_by_entity() {
        let path = temp_db_path("journal_entity");
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        let journal = TransactionJournal::new(&conn);

        let agent_id = Uuid::new_v4().to_string();

        // Create multiple transactions for same entity
        for i in 0..3 {
            let entry = JournalEntryBuilder::new(
                if i == 0 { TxType::CreateAgent } else { TxType::UpdateAgentStatus },
                EntityType::Agent,
                &agent_id
            )
            .payload(serde_json::json!({"seq": i}))
            .build();
            journal.append(entry).unwrap();
        }

        // Create transaction for different entity
        let other_entry = JournalEntryBuilder::new(
            TxType::CreateAgent,
            EntityType::Agent,
            &Uuid::new_v4().to_string()
        )
        .build();
        journal.append(other_entry).unwrap();

        // Query by entity
        let txs = journal.list_by_entity("Agent", &agent_id).unwrap();
        assert_eq!(txs.len(), 3);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn journal_watermark_checkpoint() {
        let path = temp_db_path("journal_watermark");
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        let journal = TransactionJournal::new(&conn);

        // Create some transactions
        for _ in 0..5 {
            let entry = JournalEntryBuilder::new(
                TxType::CreateOrder,
                EntityType::Order,
                &Uuid::new_v4().to_string()
            )
            .build();
            journal.append(entry).unwrap();
        }

        // Set watermark
        journal.set_watermark("last_snapshot", 3, &Uuid::new_v4()).unwrap();

        // Get watermark
        let watermark = journal.get_watermark("last_snapshot").unwrap();
        assert!(watermark.is_some());
        let (tx_id, _) = watermark.unwrap();
        assert_eq!(tx_id, 3);

        // Replay since watermark
        let txs = journal.since(tx_id).unwrap();
        assert_eq!(txs.len(), 2); // tx 4 and 5

        let _ = fs::remove_file(path);
    }

    #[test]
    fn journal_chain_integrity_verification() {
        let path = temp_db_path("journal_integrity");
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        let journal = TransactionJournal::new(&conn);

        // Create chain of transactions
        for i in 0..5 {
            let entry = JournalEntryBuilder::new(
                TxType::CreateOrder,
                EntityType::Order,
                &Uuid::new_v4().to_string()
            )
            .payload(serde_json::json!({"seq": i}))
            .build();
            journal.append(entry).unwrap();
        }

        // Verify chain
        let valid = journal.verify_chain(1).unwrap();
        assert!(valid);

        let _ = fs::remove_file(path);
    }
}
