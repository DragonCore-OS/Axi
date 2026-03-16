# Mainnet Preparation Status

## Milestone Overview

| Phase | Module | Status | Tests | Commit |
|-------|--------|--------|-------|--------|
| **M1** | Security Audit + Fixes | 🔄 Pending | - | - |
| **M2** | Operational Base | ✅ **COMPLETE** | 15/15 | - |
| ├─ M2-1 | SQLite Persistence + Recovery | ✅ AUDITED | 5/5 | [`27a83b2`](https://github.com/DragonCore-OS/Axi/commit/27a83b2) |
| ├─ M2-2 | DB schema for Identity/Market | ✅ AUDITED | 10/15 | [`504e8d5`](https://github.com/DragonCore-OS/Axi/commit/504e8d5) |
| └─ M2-3 | Transaction Journal | ✅ AUDITED | 15/15 | [`d618d46`](https://github.com/DragonCore-OS/Axi/commit/d618d46) |
| **M3** | Release Gating | 🔄 **ACTIVE** | 17/17 | - |
| ├─ M3-1 | Release Gating Logic | ✅ **COMPLETE** | 17/17 | - |
| └─ M3-2 | Feature Flags + Gradual Rollout | 🔄 Next | - | - |
| **M4** | Pre-Release | ⏸️ Blocked | - | - |
| ├─ M4-1 | Controlled Mainnet Launch | ⏸️ | - | - |
| └─ M4-2 | Monitoring + Incident Response | ⏸️ | - | - |

**M4 解锁条件**: M1 ✅ + M3 ✅

---

## M2 Operational Base - Summary

### M2-1: SQLite Persistence
- WAL mode, schema migrations, snapshot checksums, backup, recovery

### M2-2: Relational Schema  
- agents, wallets, orders, escrows, reputation_events tables
- Repository pattern (Agent, Order, Escrow, Reputation)

### M2-3: Transaction Journal
- Append-only log with SHA256 hash chaining
- Watermark/checkpoint for replay
- Entity-based history query

---

## M3-1: Release Gating Logic ✅

### API
```rust
// Gate definitions
let gates = ReleaseGates::mainnet_minimum();  // or ::testnet()

// Check readiness
let result = gates.check(&metrics);
assert!(result.is_ready());

// Launch controller
let mut controller = LaunchController::new(gates);
controller.tick(metrics);  // PreLaunch -> Gradual
controller.advance_rollout(50)?;  // 50% rollout
controller.emergency_stop();  // Halt immediately
```

### Mainnet Minimum Gates
| Gate | Threshold |
|------|-----------|
| Min Agents | 10 |
| Min Avg Reputation | 0 |
| Min Test Orders | 50 |
| Max Dispute Rate | 5% |
| Min Uptime | 72 hours |
| Min Test Period | 7 days |

### Rollout States
```
PreLaunch → Gradual(0%) → Gradual(N%) → Live
                ↓
          EmergencyStop
```

### Tests: 17/17 ✅
- Mainnet/testnet gate validation
- Blocker detection (agents, dispute rate, uptime)
- Launch controller state transitions
- Gradual rollout advancement
- Emergency stop/resume
- Feature availability (deterministic % based)

---

## Next: M3-2 Feature Flags + Gradual Rollout

- Per-feature availability controls
- Agent-hash based deterministic rollout
- Runtime configuration updates
