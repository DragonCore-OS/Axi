# Mainnet Preparation Status

## Milestone Overview

| Milestone | Module | Status | Tests | Commit |
|-----------|--------|--------|-------|--------|
| **M1** | Security Audit + Fixes | 🔄 Pending | - | - |
| **M2-1** | SQLite Persistence + Recovery | ✅ **AUDITED** | 5/5 | [`27a83b2`](https://github.com/DragonCore-OS/Axi/commit/27a83b2) |
| **M2-2** | DB schema for Identity/Market | ✅ **COMPLETE** | 10/10 | - |
| M2-3 | Transaction journal | 🔄 Next | - | - |
| M3-1 | Release gating logic | Pending | - | - |
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

### Schema v1
```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE state_snapshots (
    name TEXT PRIMARY KEY,
    payload TEXT NOT NULL,
    checksum TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

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

### Views
- `v_agent_summary` - Agent with wallet count
- `v_order_summary` - Order with escrow status

### Test Coverage (10 tests)
- `v2_schema_migrates_to_version_2` - Schema migration
- `agent_repo_crud` - Agent CRUD + wallet management
- `order_repo_crud` - Order lifecycle
- `escrow_repo_crud` - Escrow state transitions
- `reputation_repo_audit_log` - Event logging
- 5x M2-1 baseline tests (snapshots, checksums, backup, recovery)

---

## Next: M2-3 Transaction Journal

Add append-only transaction log for audit and replay:
- `transactions` table - Immutable ledger of all state changes
- Transaction types: CreateAgent, CreateOrder, FundEscrow, etc.
- Support for event sourcing and state replay
