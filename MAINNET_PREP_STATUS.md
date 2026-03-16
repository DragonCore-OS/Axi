# Mainnet Preparation Status

## Milestone Overview

| Phase | Module | Status | Tests | Commit |
|-------|--------|--------|-------|--------|
| **M1** | Security Audit + Fixes | 🔄 Pending | - | - |
| **M2** | Operational Base | ✅ **COMPLETE** | 15/15 | - |
| ├─ M2-1 | SQLite Persistence + Recovery | ✅ AUDITED | 5/5 | [`27a83b2`](https://github.com/DragonCore-OS/Axi/commit/27a83b2) |
| ├─ M2-2 | DB schema for Identity/Market | ✅ AUDITED | 10/15 | [`504e8d5`](https://github.com/DragonCore-OS/Axi/commit/504e8d5) |
| └─ M2-3 | Transaction Journal | ✅ AUDITED | 15/15 | [`d618d46`](https://github.com/DragonCore-OS/Axi/commit/d618d46) |
| **M3** | Release Gating | ✅ **COMPLETE** | 26/26 | - |
| ├─ M3-1 | Release Gating Logic | ✅ PASSED | 17/17 | [`0747307`](https://github.com/DragonCore-OS/Axi/commit/0747307) |
| └─ M3-2 | Feature Flags + Gradual Rollout | ✅ **PASSED** | 26/26 | - |
| **M4** | Pre-Release | ⏸️ Blocked on M1 | - | - |
| ├─ M4-1 | Controlled Mainnet Launch | ⏸️ | - | - |
| └─ M4-2 | Monitoring + Incident Response | ⏸️ | - | - |

**当前阻塞**: M1 Security Audit 完成后即可进入 M4

---

## M2 Operational Base ✅

### M2-1: SQLite Persistence
- WAL mode, schema migrations, snapshot checksums, backup, recovery

### M2-2: Relational Schema  
- agents, wallets, orders, escrows, reputation_events tables
- Repository pattern

### M2-3: Transaction Journal
- Append-only log with SHA256 hash chaining
- Watermark/checkpoint for replay

---

## M3 Release Gating ✅

### M3-1: Launch Gates
| Gate | Threshold |
|------|-----------|
| Min Agents | 10 |
| Min Test Orders | 50 |
| Max Dispute Rate | 5% |
| Min Uptime | 72h |

### M3-2: Feature Flags
```rust
// Runtime configurable
flags.enable("escrow_auto_release")?;
flags.set_percentage("escrow_auto_release", 50)?;

// Per-agent eligibility (deterministic)
if flags.is_enabled("escrow_auto_release", &agent_uuid) { ... }
```

### Schema v4
- `feature_flags` table - persistent config
- `sync_to_flags()` / `persist_all()` - DB sync

---

## Next: M4 Pre-Release

**解锁条件**: M1 Security Audit ✅

### M4-1: Controlled Mainnet Launch
- Genesis ceremony
- Initial validator set
- Token distribution

### M4-2: Monitoring + Incident Response
- Health metrics
- Alerting
- Runbook
