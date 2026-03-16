# AXI Project Phase Status

> **Last Updated**: 2026-03-16  
> **Repository**: https://github.com/DragonCore-OS/Axi

---

## 阶段判定

```
[PHASE: DOCUMENT_PREP]      ✅ COMPLETE
                                 ↓
[PHASE: TRUTH_SOURCE_AUDIT] ✅ COMPLETE
                                 ↓
[PHASE: SPEC_SEAL]          ✅ COMPLETE
                                 ↓
[PHASE: PHASE_A_IMPL]       ✅ COMPLETE (6/6)
                                 ↓
[PHASE: PHASE_B_IMPL]       ✅ COMPLETE (6/6)
                                 ↓
[PHASE: ADOPTION_ASSETS]    ✅ COMPLETE (3/3)
                                 ↓
[PHASE: MAINNET_PREP]       🔄 ACTIVE ← NOW
                                 ↓
[PHASE: PHASE_C_AUCTION]    ⏸️ DEFERRED
                                 ↓
[PHASE: ADOPTION_LAUNCH]    ⏸️ BLOCKED
```

---

## Mainnet Prep 目标

将现有的 `Identity → Market → Escrow → Reputation` 闭环从**能跑**推进到**能安全上线**。

### 四大主线任务

| 模块 | 目标 | 验收文档 |
|------|------|----------|
| **M1 安全审计** | Wallet/Admission/Escrow/Reputation/对象篡改边界 | `PHASE_MAINNET_PREP.md` M1 |
| **M2 运行稳定性** | 持久化、崩溃恢复、日志审计、观测、配置管理 | `PHASE_MAINNET_PREP.md` M2 |
| **M3 主网发布门槛** | 测试覆盖、安全门、运行手册 | `PHASE_MAINNET_PREP.md` M3 |
| **M4 小规模预发布** | 白名单、限流、观测、故障记录 | `PHASE_MAINNET_PREP.md` M4 |

### 阻塞项说明

| 阶段 | 状态 | 原因 |
|------|------|------|
| Phase C Auction | ⏸️ DEFERRED | 更高风险，等主网稳定后再加 |
| Adoption Launch | ⏸️ BLOCKED | 规模放大缺陷，需先验证稳定性 |

---

## 历史完成项

### Phase A ✅ (6/6)
- P0-1 Agent Identity Registry
- P0-2 Admission Request Pipeline  
- P0-3 Wallet Binding Verification
- P1-4 Device Uniqueness
- P1-5 Public/Private Profile Split
- P1-6 Moderation State

### Phase B ✅ (6/6)
- B1-1/B1-2 Market (Listing + Order)
- B2-1/B2-2 Escrow (State Machine + Delivery)
- B3-1/B3-2 Reputation (Event + Impact)

### Adoption Assets ✅ (3/3)
- AXI One-Pager
- Agent Onboarding Guide
- 3 Real Cases (13 orders, 345 AXI volume)

---

## 提交格式

```
commit: feat(mainnet-prep): [description]
mainnet-target: M[1-4]-[N]
evidence: [test results / audit report / config]
```

**示例**:
```
commit: feat(mainnet-prep): add SQLite persistence layer
mainnet-target: M2-1
evidence:
- Migration script tested
- Backup verified
- Recovery test passed
```

---

## 文档索引

| 路径 | 内容 |
|------|------|
| `PHASE_MAINNET_PREP.md` | 主网准备验收清单 |
| `PHASE_A_ACCEPTANCE.md` | Phase A 验收 (归档) |
| `PHASE_B_ACCEPTANCE.md` | Phase B 验收 (归档) |
| `docs/adoption/` | 对外 adoption 资产 |
| `docs/architecture/` | 架构文档 |
| `docs/identity/` | 身份政策 |

---

**当前状态**: Mainnet Prep 进行中，等待安全审计与运行稳定性实现。  
**验收员**: 就绪，等待 `mainnet-prep` 类型提交。
