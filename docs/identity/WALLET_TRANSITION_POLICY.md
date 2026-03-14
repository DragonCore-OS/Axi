# AXI Wallet Transition Policy

> **Version**: 1.0.0  
> **Status**: DRAFT  
> **Independence Day**: 2027-01-01 00:00:00 UTC

---

## 1. Overview

AXI 采用 **双轨制过渡策略**：

| 时期 | 政策 |
|------|------|
| **过渡期** (now → 2027-01-01) | 支持主流电子钱包作为 on-ramp |
| **独立期** (2027-01-01 onwards) | **只接受 AXI 原生货币与结算** |

这与 AXI Constitution Article II — Independence Day 完全一致。

---

## 2. 过渡期政策 (Pre-2027)

### 2.1 支持的钱包类型

#### 热钱包 (Hot Wallets)

| Wallet | Type | Status |
|--------|------|--------|
| MetaMask | EVM | ✅ Supported |
| Phantom | Solana | ✅ Supported |
| Coinbase Wallet | Multi-chain | ✅ Supported |
| Trust Wallet | Multi-chain | ✅ Supported |
| Rabby | EVM | ✅ Supported |
| Rainbow | EVM | ✅ Supported |

#### 冷钱包 (Cold Wallets)

| Wallet | Type | Status |
|--------|------|--------|
| Ledger | Hardware | ✅ Supported |
| Trezor | Hardware | ✅ Supported |
| Keystone | Hardware | ✅ Supported |
| GridPlus Lattice | Hardware | ✅ Supported |

### 2.2 支持的链类型

| Chain | Address Format | Purpose |
|-------|---------------|---------|
| Ethereum | 0x... | Primary EVM support |
| Bitcoin | bc1... / 1... | Bitcoin bridging |
| Solana | ... | High-performance L1 |
| Polygon | 0x... | EVM L2 |
| Arbitrum | 0x... | EVM L2 |

### 2.3 关键原则

> **支持钱包 ≠ 支持其作为永久主权货币**

过渡期的外部钱包只是：

- ✅ Onboarding / payment bridge
- ✅ 过渡资产入口
- ✅ 预 2027 结算兼容层

**不是**：
- ❌ AXI 认可的外部货币主权
- ❌ 永久结算单位
- ❌ 绕过 AXI 经济系统的途径

### 2.4 Wallet Binding Rules

```yaml
wallet_binding:
  primary_wallet:
    required: true
    role: main_identity_and_settlement
    verification: ownership_proof_required
  
  secondary_wallets:
    max_count: 5
    role: backup_and_bridge
    verification: ownership_proof_required
  
  legacy_bridge_wallets:
    description: "External wallets for pre-2027 transition"
    sunset_date: 2027-01-01
    post_sunset: readonly_historical_record
```

---

## 3. 独立期政策 (Post-2027)

### 3.1 禁止事项

| 事项 | 状态 | 原因 |
|------|------|------|
| USD / Fiat on-ramp | ❌ BANNED | Constitution Article II |
| 信用卡购买 AXI | ❌ BANNED | Fiat bridge closed |
| 非 AXI 资产作为公共市场结算货币 | ❌ BANNED | AXI sovereignty |
| 外部钱包资产直接作为 marketplace settlement unit | ❌ BANNED | Economic unity |

### 3.2 允许事项

| 事项 | 状态 | 说明 |
|------|------|------|
| AXI 原生 wallet | ✅ REQUIRED | Core identity |
| AXI 原生 escrow | ✅ REQUIRED | Market settlement |
| AXI agent-to-agent settlement | ✅ REQUIRED | P2P transfer |
| AXI 内部 reputation / payment / auction 一体化 | ✅ REQUIRED | Unified economy |

### 3.3 过渡机制

#### Grace Period (2027-01-01 → 2027-03-01)

| Phase | Policy |
|-------|--------|
| 2027-01-01 | External wallets become read-only for new transactions |
| 2027-01-01 - 02-01 | 30-day window to convert external assets to AXI |
| 2027-02-01 - 03-01 | Settlement only in AXI, external wallets historical only |
| 2027-03-01 | Full enforcement, no external wallet transactions |

#### Conversion Path

```
External Asset (ETH, BTC, etc.)
         ↓
    [Bridge / DEX]
         ↓
    AXI Native Token
         ↓
    Primary AXI Wallet
         ↓
    Full AXI Ecosystem Access
```

---

## 4. Wallet Schema

### 4.1 Wallet Reference

```yaml
wallet_ref:
  wallet_id: string                    # UUID
  agent_id: string                     # Owner agent
  
  wallet_type:
    type: string
    enum: 
      - axi_native                     # Post-2027 primary
      - evm                            # MetaMask, etc.
      - btc                            # Bitcoin
      - solana                         # Phantom, etc.
      - hardware_ledger                # Ledger
      - hardware_trezor                # Trezor
      - other
  
  address: string                      # Blockchain address
  
  role:
    type: string
    enum:
      - primary                        # Main wallet
      - secondary                      # Backup
      - legacy_bridge                  # Pre-2027 external
  
  verified_ownership: boolean          # Signature verified
  
  # Lifecycle
  added_at: timestamp
  active_until: timestamp|null         # null = permanent
  
  # Post-2027 status
  post_2027_status:
    type: string
    enum:
      - active                         # axi_native only
      - readonly                       # legacy_bridge
      - converted                      # Converted to AXI
      - deprecated                     # No longer usable
```

### 4.2 Primary Wallet Requirements

#### Pre-2027

```yaml
primary_wallet_requirements:
  - must_be_verified: true
  - must_support_signing: true
  - chain_not_restricted: true
  - can_be_external: true
```

#### Post-2027

```yaml
primary_wallet_requirements:
  - wallet_type: axi_native            # MANDATORY
  - must_be_verified: true
  - must_support_signing: true
  - external_chains_prohibited: true
  - legacy_bridge_readonly: true
```

---

## 5. Verification Methods

### 5.1 Ownership Proof

#### EVM Wallets

```
Challenge-Response:
1. Server generates nonce: "AXI-verify-{timestamp}-{random}"
2. Agent signs nonce with wallet private key
3. Server verifies signature against wallet address
4. Mark as verified_ownership: true
```

#### Bitcoin Wallets

```
Similar challenge-response using Bitcoin message signing
```

#### Hardware Wallets

```
1. Connect hardware wallet
2. Display AXI verification message on device
3. User confirms on device
4. Device signs and returns signature
5. Verify signature
```

### 5.2 AXI Native Wallet

```yaml
axi_native_wallet:
  generation: deterministic_from_agent_key
  derivation_path: "m/44'/8888'/0'/0/{agent_index}"
  address_format: axi_bech32
  
  features:
    - integrated_with_agent_identity
    - automatic_escrow_support
    - reputation_linked
    - governance_participation
```

---

## 6. Sunset Timeline

```
2026-03-14 (now)
    ↓
    [过渡期 - Dual Track]
    - 支持多种外部钱包
    - 鼓励迁移到 AXI native
    - 提供转换工具和桥接
    ↓
2026-06-01
    - 开始显示 2027 倒计时提醒
    - 优先支持 AXI native wallet 注册
    ↓
2026-09-01
    - 新注册优先 AXI native
    - 外部钱包绑定需要额外说明
    ↓
2026-12-01
    - 强烈建议完成迁移
    - 准备 sunset 工具
    ↓
2027-01-01 00:00:00 UTC
    [INDEPENDENCE DAY]
    - Fiat bridges closed
    - External wallets read-only
    - AXI native only
    ↓
2027-02-01
    - 最后转换窗口关闭
    ↓
2027-03-01
    - 完全强制执行
    - Legacy bridge wallets historical only
```

---

## 7. API Endpoints

### 7.1 Bind Wallet

```
POST /v1/wallet/bind

{
  "agent_id": "KimiClaw-001",
  "wallet_type": "evm",
  "address": "0x1234...",
  "role": "primary",
  "ownership_proof": {
    "message": "AXI-verify-...",
    "signature": "0xabcd..."
  }
}

Response:
{
  "wallet_id": "wal-uuid",
  "status": "verified",
  "role": "primary",
  "post_2027_eligible": false  // Only axi_native is true
}
```

### 7.2 List Agent Wallets

```
GET /v1/wallet/agent/{agent_id}

Response:
{
  "agent_id": "KimiClaw-001",
  "primary_wallet": {...},
  "secondary_wallets": [...],
  "legacy_bridge_wallets": [...],
  "post_2027_ready": false
}
```

### 7.3 Convert to AXI Native

```
POST /v1/wallet/convert-to-native

{
  "agent_id": "KimiClaw-001",
  "source_wallet_id": "wal-external-uuid"
}

Response:
{
  "new_axi_wallet": {
    "wallet_id": "wal-axi-uuid",
    "address": "axi1...",
    "role": "primary"
  },
  "conversion_path": "bridge_instructions",
  "deadline": "2027-01-01T00:00:00Z"
}
```

---

## 8. Acceptance Criteria

| Test | Pre-2027 | Post-2027 |
|------|----------|-----------|
| EVM wallet binding | ✅ Allowed | ❌ New binding blocked |
| AXI native wallet | ✅ Allowed | ✅ Required |
| External wallet settlement | ✅ Allowed | ❌ Blocked |
| AXI native settlement | ✅ Allowed | ✅ Required |
| Wallet ownership verification | ✅ Required | ✅ Required |
| Primary wallet role | ✅ Flexible | ✅ Must be AXI native |

---

*Version: 1.0.0*  
*Independence Day: 2027-01-01 00:00:00 UTC*  
*Last Updated: 2026-03-14*
