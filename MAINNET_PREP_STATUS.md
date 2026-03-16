# Mainnet Preparation Status

## Milestone Overview

| Phase | Module | Status | Tests | Commit |
|-------|--------|--------|-------|--------|
| **M1** | Security Audit + Fixes | 🚨 **FINDINGS** | - | [`da3e3e3`](https://github.com/DragonCore-OS/Axi/commit/da3e3e3) |
| **M2** | Operational Base | ✅ **COMPLETE** | 15/15 | - |
| **M3** | Release Gating | ✅ **COMPLETE** | 26/26 | - |
| **M4** | Pre-Release | ⏸️ **BLOCKED** | - | - |

**当前阻塞**: M1 安全漏洞修复

---

## M1 Security Audit: 5 Vulnerabilities Found 🚨

**Audit Report**: [`SECURITY_AUDIT.md`](./SECURITY_AUDIT.md)

### P0 - Critical/High

| ID | Vulnerability | Location | Impact |
|----|---------------|----------|--------|
| P0-1 | **Wallet Verification Bypass** | `wallet_verification.rs:23-57` | 所有权验证完全失效，任意钱包可绑定 |
| P0-2 | **Admission Trusts Unverified Input** | `admission.rs:22` | 用户声称即可绕过验证 |
| P0-3 | **Missing Authorization Checks** | `escrow.rs:91-97` | 任何人可操作任意托管 |

### P1 - High

| ID | Vulnerability | Location | Impact |
|----|---------------|----------|--------|
| P1-1 | **Reputation Event Forgery** | `reputation.rs:62-87` | 声誉可无限伪造 |
| P1-2 | **Direct DB Mutation Bypass** | `repos.rs:118-132` | 业务逻辑全 bypass |

### 修复优先级

```
🔴 立即: P0-1 (钱包验证), P1-1 (声誉伪造)
🟠 高:   P0-2 (准入流程), P0-3 (授权检查), P1-2 (DB层)
```

### 主网发布条件

- [ ] P0 全部修复并审计通过
- [ ] P1 全部修复
- [ ] 安全审计签字 (M1 Security Sign-off)

---

## M2 Operational Base ✅

### M2-1: SQLite Persistence (5/5 tests)
- WAL mode, schema migrations, snapshot checksums, backup, recovery

### M2-2: Relational Schema (10/15 tests)
- agents, wallets, orders, escrows, reputation_events tables

### M2-3: Transaction Journal (15/15 tests)
- Append-only log with SHA256 hash chaining

---

## M3 Release Gating ✅

### M3-1: Release Gating Logic (17/17 tests)
- Mainnet gates: 10 agents, 50 orders, <5% dispute, 72h uptime
- Launch state machine: PreLaunch → Gradual → Live

### M3-2: Feature Flags (26/26 tests)
- Runtime configuration, per-feature rollout
- Deterministic agent eligibility

---

## 总结

| 模块 | 状态 | 阻塞 |
|------|------|------|
| M2 Operational | ✅ | - |
| M3 Release Gating | ✅ | - |
| M1 Security | 🚨 | 5 vulnerabilities |
| M4 Mainnet | ⏸️ | Waiting M1 |

**建议**: 优先修复 P0-1 钱包验证和 P1-1 声誉伪造，完成后再进入 M4。
