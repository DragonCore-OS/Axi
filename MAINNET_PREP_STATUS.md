# Mainnet Preparation Status

## Milestone Overview

| Milestone | Module | Status | Tests | Commit |
|-----------|--------|--------|-------|--------|
| **M1** | Security Audit + Fixes | 🔄 Pending | - | - |
| **M2-1** | SQLite Persistence + Recovery | ✅ **AUDITED** | 5/5 | [`27a83b2`](https://github.com/DragonCore-OS/Axi/commit/27a83b2) |
| **M2-2** | DB schema for Identity/Market | ✅ **AUDITED** | 10/15 | [`504e8d5`](https://github.com/DragonCore-OS/Axi/commit/504e8d5) |
| **M2-3** | Transaction Journal | ✅ **COMPLETE** | 15/15 | - |
| M3-1 | Release gating logic | 🔄 Next | - | - |
| M3-2 | Feature flags + gradual rollout | Pending | - | - |
| M4-1 | Controlled mainnet launch | Pending | - | - |
| M4-2 | Monitoring + incident response | Pending | - | - |

---

## M2-1 Implementation Summary

```rust
pub struct PersistentStore {
    db_path: PathBuf,
}

impl PersistentStore {
    pub fn open(path) -> Self           // Initialize + migrations
    pub fn save_snapshot(name, value)   // Serialize + checksum
    pub fn load_snapshot(name) -> T     // Verify + deserialize
    pub fn backup_to(path)              // Filesystem copy
}
```

### Features
- **WAL Mode**: Concurrent reads during writes
- **Schema Migrations**: Version tracking table
- **SHA256 Checksums**: Tamper detection
- **Crash Recovery**: Restart restores last known state
- **Backup Support**: Hot-copy to backup path

---

## M2-2 Implementation Summary

### Repository Pattern
```rust
pub struct AgentRepository<'a> { conn: &'a Connection }
pub struct OrderRepository<'a> { conn: &'a Connection }
pub struct EscrowRepository<'a> { conn: &'a Connection }
pub struct ReputationRepository<'a> { conn: &'a Connection }
```

### Schema v2 - Relational Tables

| Table | Purpose | Key Relations |
|-------|---------|---------------|
| `agents` | AgentIdentity persistence | PK: agent_uuid, UK: agent_id |
| `wallets` | Wallet verification records | FK: agent_uuid, Unique Primary wallet per agent |
| `orders` | Order records | FK: buyer_agent_uuid, seller_agent_uuid |
| `escrows` | Escrow state machine | FK: order_id (1:1), buyer/seller agents |
| `reputation_events` | Immutable audit log | FK: agent_uuid, order_id (nullable) |

---

## M2-3 Implementation Summary

### Transaction Journal (Append-only)

```rust
pub struct TransactionJournal<'a> { conn: &'a Connection }

impl<'a> TransactionJournal<'a> {
    pub fn append(entry: JournalEntry) -> Transaction
    pub fn list_by_entity(entity_type, entity_id) -> Vec<Transaction>
    pub fn since(tx_id) -> Vec<Transaction>        // For replay
    pub fn set_watermark(name, tx_id)              // Checkpoint
    pub fn verify_chain(from_tx_id) -> bool        // Integrity check
}
```

### Schema v3

| Table | Purpose |
|-------|---------|
| `transaction_journal` | Append-only log with hash chaining |
| `journal_watermarks` | Checkpoint positions for replay |

### Features
- **Monotonic tx_id**: Auto-increment sequence
- **Hash Chaining**: `tx_hash = SHA256(prev_hash + payload + timestamp)`
- **Entity Tracking**: Query complete history per entity
- **Watermark/Checkpoint**: Set recovery points
- **Chain Verification**: Detect tampering

### Transaction Types
- `CreateAgent`, `UpdateAgentStatus`, `UpdateAgentReputation`
- `CreateWallet`
- `CreateOrder`, `UpdateOrderStatus`
- `CreateEscrow`, `UpdateEscrowStatus`, `SubmitDelivery`, `VerifyDelivery`, `OpenDispute`, `ResolveDispute`
- `RecordReputation`

---

## Next: M3-1 Release Gating Logic

Add mainnet readiness gates:
- Minimum agent count threshold
- Reputation score requirements
- System stability metrics
- Gradual rollout controls
