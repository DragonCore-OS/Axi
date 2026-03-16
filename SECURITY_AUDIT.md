# M1 Security Audit Report

**Scope**: AXI Core System  
**Priority**: Critical path vulnerabilities  
**Date**: 2026-03-14

## Audit Checklist

| Priority | Area | Status | Finding |
|----------|------|--------|---------|
| P0 | Wallet verification bypass | 🔄 Auditing | - |
| P0 | Admission/uniqueness bypass | 🔄 Auditing | - |
| P0 | Escrow illegal transition | 🔄 Auditing | - |
| P1 | Reputation forgery | 🔄 Auditing | - |
| P1 | Object tampering | 🔄 Auditing | - |

---

## P0-1: Wallet Verification Bypass 🚨 CRITICAL

### Target
`src/identity/wallet_verification.rs:23-57`  
`src/identity/registry.rs::attach_wallet`

### Threat Model
Attacker registers wallet without proving ownership.

### Audit Findings

**CRITICAL**: `verify_evm_ownership()` 函数完全未实现实际签名验证。

```rust
// Line 54-56
// MVP: accept valid-looking signatures
// Full implementation would recover public key and verify address
VerificationResult::Valid
```

**漏洞**: 只检查签名格式（65字节、r/s非零、v有效），但不验证签名是否由钱包私钥生成。

**影响**:
- 攻击者可以附加任意钱包地址（包括高声誉或特权钱包）
- 完全绕过所有权验证
- 信任模型基础被破坏

**复现**:
```rust
// 攻击者可声称拥有任何地址
let fake_sig = "0x" + "aa".repeat(64) + "1c"; // 格式正确但无效
verify_evm_ownership("0xVICTIM", "challenge", &fake_sig); 
// => VerificationResult::Valid (错误!)
```

**修复**: 使用 secp256k1 恢复公钥并派生地址，与声称地址比对。

---

## P0-2: Admission Trusts Unverified Input 🚨 HIGH

### Target
`src/identity/admission.rs:22,55-57`

### Threat Model
攻击者绕过钱包验证要求。

### Audit Findings

**HIGH**: `AdmissionRequest.wallet_verified` 是调用者提供的布尔值，无验证。

```rust
// Line 22
pub wallet_verified: bool,  // 用户声称!

// Line 55-57
if !req.wallet_verified {
    state = AdmissionState::Rejected;
}
```

**漏洞**: 攻击者可设置 `wallet_verified: true` 绕过检查，即使未实际验证。

**影响**:
- 绕过钱包验证要求
- 可创建多个未验证钱包的身份
- 与 P0-1 组合可完全匿名参与

**修复**: 查询钱包验证状态，不依赖用户输入。

---

## P0-3: Missing Authorization Checks 🚨 MEDIUM

### Target
`src/market/escrow.rs:91-97, 202-204`  
`fund()`, `move_to_escrow()`, `refund()`

### Threat Model
未授权调用者操作托管。

### Audit Findings

**MEDIUM**: 状态转换函数无调用者身份验证。

```rust
pub fn fund(&mut self, escrow_id: &Uuid) -> Result<(), &'static str> {
    self.transition(escrow_id, EscrowStatus::Funded)  // 谁都能调用!
}

pub fn refund(&mut self, escrow_id: &Uuid) -> Result<(), &'static str> {
    self.transition(escrow_id, EscrowStatus::Refunded)  // 谁都能调用!
}
```

**漏洞**: 任何调用者（非买家/卖家/仲裁者）都可触发状态转换。

**影响**:
- 恶意第三方可 fund/refund 任意托管
- 资金流转控制权旁落
- 商业逻辑可被操纵

**修复**: 添加调用者身份验证，限制 authorized parties。

**注**: `transition()` 状态机本身是正确（lines 216-227），但缺乏授权层。

---

## P1-1: Reputation Event Forgery 🚨 HIGH

### Target
`src/identity/reputation.rs:62-87`  
`record_event()`

### Threat Model
攻击者伪造声誉事件提升分数。

### Audit Findings

**HIGH**: `record_event()` 无任何验证，任何人可为任何 agent 记录任意事件。

```rust
pub fn record_event(&mut self, registry, agent_uuid, order_id, event_type, ...)
    -> Result<ReputationEvent, &'static str> 
{
    // 直接计算 delta 并应用，无任何验证!
    let delta = Self::calculate_delta(&event_type, rating).0;
    registry.apply_reputation_delta(&agent_uuid, delta)?;  // +5, +2, -5, -10
    // ...
}
```

**漏洞**:
- 无调用者身份验证
- 无订单存在性验证  
- 无订单完成状态验证
- 无重复事件检查（同一订单可多次记录）

**攻击**: 
```rust
// 攻击者循环调用，每秒 +5 声誉
for i in 0..1000 {
    reputation.record_event(registry, victim_uuid, 
        Some(fake_order_id), 
        ReputationEventType::OrderCompleted, 
        None, "completed".into())?;
}
// victim 声誉 +5000
```

**影响**: 声誉系统完全不可信。

**修复**:
1. 验证调用者身份（只有订单相关方可评价）
2. 验证订单存在且已完成
3. 防止重复评价

---

## P1-2: Direct DB Mutation Bypass 🚨 HIGH

### Target
`src/storage/repos.rs:118-132, 211-227`  
`update_status()`, `update_reputation_score()`, `update_escrow_status()`

### Threat Model
攻击者绕过业务逻辑直接修改数据。

### Audit Findings

**HIGH**: Repository 层直接暴露状态修改，无业务逻辑验证。

```rust
// AgentRepository
pub fn update_status(&self, uuid, status) -> Result<(), String> {
    // 直接 SQL UPDATE，无状态机验证
}

pub fn update_reputation_score(&self, uuid, delta) -> Result<(), String> {
    // 直接修改分数，无事件验证
}

// EscrowRepository
pub fn update_status(&self, escrow_id, status) -> Result<(), String> {
    // 直接 UPDATE，无状态转换验证
}
```

**漏洞**:
1. 可绕过 `AgentStatus` 状态机（如 Pending → Banned 直接跳过 Approved）
2. 可直接修改声誉分数，无需 `ReputationEvent`
3. 可绕过 `EscrowStatus` 状态机

**攻击**:
```rust
// 直接给自己 +1000 声誉
agent_repo.update_reputation_score(my_uuid, 1000)?;

// 直接跳过审核进入 Approved
agent_repo.update_status(my_uuid, AgentStatus::Approved)?;

// 直接从 Funded 到 Released 绕过 InEscrow
escrow_repo.update_status(escrow_id, EscrowStatus::Released)?;
```

**影响**: 所有业务规则可被绕过。

**修复**: Repository 层只暴露查询，修改通过 Service 层统一处理。

---

## 审计总结

| 优先级 | 漏洞 | 影响 | 修复紧急度 |
|--------|------|------|------------|
| **P0** | Wallet Verification Bypass | 所有权验证完全失效 | 🔴 立即 |
| **P0** | Admission Trusts Input | 可声称任意钱包已验证 | 🔴 立即 |
| **P0** | Missing Authorization | 未授权状态转换 | 🟠 高 |
| **P1** | Reputation Forgery | 声誉可无限伪造 | 🔴 立即 |
| **P1** | Direct DB Mutation | 业务逻辑全 bypass | 🟠 高 |

### 修复建议

1. **钱包验证**: 实现 secp256k1 签名恢复和地址比对
2. **准入流程**: 不依赖用户输入，直接查询验证状态
3. **授权层**: 所有状态转换验证调用者身份
4. **声誉系统**: 添加订单验证和重复检查
5. **架构调整**: Service 层统一业务逻辑，Repository 只读

**结论**: 当前代码存在多个严重安全漏洞，**不建议主网发布**。
