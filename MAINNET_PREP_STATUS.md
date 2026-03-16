# Mainnet Preparation Status

## Milestone Overview

| Milestone | Module | Status | Tests | Commit |
|-----------|--------|--------|-------|--------|
| **M1** | Security Audit + Fixes | 🔄 Pending | - | - |
| **M2-1** | SQLite Persistence + Recovery | ✅ **AUDITED** | 5/5 | [`27a83b2`](https://github.com/DragonCore-OS/Axi/commit/27a83b2) |
| M2-2 | DB schema for Identity/Market | 🔄 Next | - | - |
| M2-3 | Transaction journal | Pending | - |
| M3-1 | Release gating logic | Pending | - |
| M3-2 | Feature flags + gradual rollout | Pending | - |
| M4-1 | Controlled mainnet launch | Pending | - |
| M4-2 | Monitoring + incident response | Pending | - |

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

## Next: M2-2 Database Schema Migration

Extend storage with relational tables:
- `agents` - AgentIdentity persistence
- `wallets` - Wallet verification records
- `orders` - Order records
- `escrows` - Escrow state machine
- `reputation_events` - Reputation audit log
