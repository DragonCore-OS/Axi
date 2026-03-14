# GENESIS_CANONICAL_AUDIT.md

> **Status**: REQUIRED BEFORE ANY PROTOCOL IMPLEMENTATION  
> **Priority**: P0 - BLOCKING  
> **Estimated Time**: 2-4 hours  
> **Sign-off Required**: Genesis Node Operator + Protocol Lead

---

## 1. Purpose

确认 AXI 网络的唯一 canonical genesis 状态，确保所有运行时表面（daemon、CLI、API、systemd service）读取同一真相源。

**为什么这必须在协议实现之前完成**：
- escrow、reputation、slash 都依赖 genesis 作为信任根
- 如果 genesis hash 不一致，经济合约的结算将失去确定性
-  constitution hash 是协议合法性的法律基础

---

## 2. Audit Targets

| 目标 | 路径/命令 | 关键检查项 |
|------|-----------|------------|
| daemon | `axi/src/core/genesis.rs` | genesis 生成逻辑 |
| CLI | `axi/src/main.rs` + `cargo run -- genesis` | CLI 输出 |
| API | runtime API endpoint `/genesis` | HTTP 返回值 |
| systemd service | `systemctl status axi-genesis` + logs | 启动日志 |
| persisted genesis files | `~/.axi/genesis.json` 等 | 持久化文件 |
| constitution hash | `axi/CONSTITUTION.md` | 文件哈希 |
| anchor values | `axi/src/anchor/` | power/compute 锚定值 |

---

## 3. Canonical Values (Expected)

```yaml
canonical_values:
  genesis_hash: "TBD - TO BE DETERMINED BY AUDIT"
  constitution_hash: "00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177"
  power_anchor_kwh: 1000
  compute_anchor_tflops: 3280
  genesis_timestamp: 1709256878  # 2024-03-01T00:00:00Z
  independence_timestamp: 1704067200  # 2027-01-01T00:00:00Z
  
  source_of_truth: "TBD - 代码 / 文件 / 链上状态"
```

**注意**：在 audit 完成前，这些值是候选值，不是最终决定。

---

## 4. Audit Execution Checklist

### 4.1 Code Review

#### 4.1.1 Genesis Generation Logic

```bash
# File to review: axi/src/core/genesis.rs
```

检查点：
- [ ] genesis hash 计算算法是否确定性的？（相同输入 = 相同输出）
- [ ] 是否包含 timestamp？timestamp 来源是什么？
- [ ] 是否包含 constitution hash？
- [ ] 是否包含 anchor values？
- [ ] seed 或随机性来源是否可复现？

代码审查模板：
```rust
// 审查以下函数
genesis_hash = hash(
    constitution_hash,
    power_anchor,
    compute_anchor,
    genesis_timestamp,
    // 还有其他字段吗？
);
```

#### 4.1.2 Constitution Hash Calculation

```bash
# 本地验证 constitution hash
sha256sum axi/CONSTITUTION.md
```

检查点：
- [ ] 文件内容是否与预期一致？
- [ ] 换行符（LF vs CRLF）是否一致？
- [ ] 文件末尾是否有空行？
- [ ] 编码是否为 UTF-8 without BOM？

#### 4.1.3 Anchor Value Sources

```bash
# Files to review:
# - axi/src/anchor/power.rs
# - axi/src/anchor/compute.rs
```

检查点：
- [ ] power_anchor: 1000 kWh 是硬编码还是可配置？
- [ ] compute_anchor: 3280 TFLOPs 是硬编码还是可配置？
- [ ] 这些值在运行时是否可以被覆盖？
- [ ] systemd 环境变量是否会影响这些值？

### 4.2 Runtime Verification

#### 4.2.1 CLI Output

```bash
cd /home/admin/axi
cargo build --release 2>/dev/null
./target/release/axi genesis
```

记录输出：
```
Genesis Hash:        ___________________________
Constitution Hash:   ___________________________
Power Anchor:        ___________ kWh
Compute Anchor:      ___________ TFLOPs
Timestamp:           ___________
```

#### 4.2.2 Daemon State

```bash
# 如果 daemon 正在运行
curl -s http://localhost:28000/genesis 2>/dev/null || echo "API not available"

# 或者通过 systemd
sudo systemctl status axi-genesis
sudo journalctl -u axi-genesis --since "2026-03-01" | grep -i genesis
```

记录输出：
```
API Genesis Hash:    ___________________________
Log Genesis Hash:    ___________________________
```

#### 4.2.3 Persisted Files

```bash
# 检查所有可能的 genesis 文件位置
ls -la ~/.axi/ 2>/dev/null || echo "No ~/.axi/"
ls -la /var/lib/axi/ 2>/dev/null || echo "No /var/lib/axi/"
ls -la /etc/axi/ 2>/dev/null || echo "No /etc/axi/"

# 如果存在 genesis.json，检查内容
cat ~/.axi/genesis.json 2>/dev/null | jq .
```

记录输出：
```
File Location:       ___________________________
File Genesis Hash:   ___________________________
File Constitution:   ___________________________
```

#### 4.2.4 Environment Variables

```bash
# 检查可能影响 genesis 的环境变量
env | grep -i axi
env | grep -i genesis
cat /etc/systemd/system/axi-genesis.service 2>/dev/null | grep -i env
```

记录：
```
AXI_GENESIS_HASH:    ___________________________
AXI_CONSTITUTION:    ___________________________
Other relevant vars: ___________________________
```

### 4.3 Cross-Reference Matrix

| Surface | Genesis Hash | Constitution Hash | Power Anchor | Compute Anchor | Matches Canonical? |
|---------|--------------|-------------------|--------------|----------------|-------------------|
| Code (genesis.rs) | | | | | |
| CLI (`axi genesis`) | | | | | |
| API (`/genesis`) | | | | | |
| Systemd Logs | | | | | |
| `~/.axi/genesis.json` | | | | | |
| `axi/STATUS.md` | | | | | |
| `axi-github-push/` (if different) | | | | | |

---

## 5. Known Mismatch History

### 5.1 观察到的异常值

```yaml
observed_anomalies:
  - location: "logs from 2026-03-02"
    genesis_hash_1: "9f5085f9238d5da9417352032ed1096875e0f17d946290394febfda442483316"
    genesis_hash_2: "793868fe37446d5a3c8543fc3bd603b1fd025809296ecf96065ba8f61685790d"
    note: "Two different genesis hashes observed in logs"
    
  - location: "axi/STATUS.md"
    genesis_hash: "7e2e132ba352e53035ce049229a421ca89d56a85195f7050c0369fd67bfcc716"
    note: "Current documented value"
```

### 5.2 可能的根本原因

- [ ] 不同的 build 时间导致不同的 timestamp
- [ ] 不同的 constitution 文件内容（换行符、编码）
- [ ] 环境变量覆盖了硬编码值
- [ ] 多个 genesis 文件存在，daemon 读取了不同的文件
- [ ] 代码版本不一致（不同目录的 axi/ vs axi-github-push/）
- [ ] 测试网 vs 主网配置混淆

---

## 6. Resolution Process

### 6.1 确定 Canonical Source

决策树：
```
1. 是否存在多个物理 genesis 文件？
   ├─ Yes → 选择最新修改的 / 选择 systemd 指向的 / 选择代码生成的
   └─ No → 继续

2. 代码生成逻辑是否确定性的？
   ├─ No → 修复代码，使其确定性
   └─ Yes → 继续

3. 环境变量是否影响 genesis？
   ├─ Yes → 决定：禁止覆盖 / 允许但记录 / 其他
   └─ No → 继续

4. 选择 canonical source:
   ├─ Option A: 代码生成值 (genesis.rs)
   ├─ Option B: 持久化文件 (~/.axi/genesis.json)
   ├─ Option C: systemd 环境变量
   └─ Option D: 手动指定的固定值
```

### 6.2 修复不一致

对于每个不一致的表面：

| Surface | 修复操作 | 执行人 | 完成时间 |
|---------|----------|--------|----------|
| Code | | | |
| CLI | | | |
| API | | | |
| Systemd | | | |
| Persisted Files | | | |

### 6.3 防止未来分歧

- [ ] 在代码中添加 genesis hash 自检（启动时验证）
- [ ] 在 API 中添加 `/health/genesis` 端点
- [ ] 在 CLI 中添加 `axi genesis --verify` 命令
- [ ] 在 systemd 启动脚本中添加 pre-flight check
- [ ] 文档化：任何 genesis 变更必须经过共识流程

---

## 7. Final Decision

**此部分在 audit 完成后填写**

```yaml
final_decision:
  canonical_genesis_hash: ""
  canonical_constitution_hash: "00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177"
  canonical_power_anchor_kwh: 1000
  canonical_compute_anchor_tflops: 3280
  
  source_of_truth: ""  # 例如: "Code in axi/src/core/genesis.rs"
  source_file_path: ""  # 例如: "/home/admin/axi/src/core/genesis.rs"
  
  rationale: ""  # 为什么选择这个作为 canonical
  
  approved_by:
    - name: ""
      signature: ""
      date: ""
    - name: ""
      signature: ""
      date: ""
  
  effective_date: "2026-03-XX"
  
  migration_required: true | false
  migration_plan: ""  # 如果需要迁移，描述步骤
```

---

## 8. Sign-off

| 角色 | 姓名 | 签名/确认 | 日期 |
|------|------|-----------|------|
| Genesis Node Operator | | | |
| Protocol Lead | | | |
| Security Reviewer | | | |

**声明**：
> 以上签名者确认，AXI 网络的 canonical genesis 状态已确定，所有运行时表面已同步到此状态。未来的协议实现（escrow、reputation、slash）将基于此状态作为信任根。

---

## Appendix A: Quick Verification Commands

```bash
#!/bin/bash
# save as: verify_genesis.sh
# usage: ./verify_genesis.sh

echo "=== AXI Genesis Verification ==="
echo ""

echo "1. Code Source:"
cd /home/admin/axi
grep -n "genesis_hash" src/core/genesis.rs 2>/dev/null | head -5
echo ""

echo "2. CLI Output:"
./target/release/axi genesis 2>/dev/null || echo "CLI not available"
echo ""

echo "3. Constitution Hash:"
sha256sum CONSTITUTION.md
echo ""

echo "4. Persisted Files:"
ls -la ~/.axi/ 2>/dev/null
cat ~/.axi/genesis.json 2>/dev/null | jq '.hash' 2>/dev/null
echo ""

echo "5. Systemd Service:"
systemctl is-active axi-genesis 2>/dev/null || echo "Service not active"
systemctl cat axi-genesis 2>/dev/null | grep -i genesis | head -3
echo ""

echo "=== End Verification ==="
```

---

## Appendix B: Genesis State Schema

```yaml
# 建议的 canonical genesis 结构
genesis_state:
  version: "0.1.0"
  
  block:
    hash: "sha256_of_below"
    timestamp: 1709256878
    index: 0
    
  anchors:
    power_kwh: 1000
    compute_tflops: 3280
    
  constitution:
    hash: "00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177"
    uri: "ipfs://Qm..."
    
  parameters:
    independence_timestamp: 1704067200
    halflife_years: 5
    anti_whale_threshold: 0.30
    
  treasury:
    initial_supply_axi: 13280
    genesis_address: "03d96d749551c43e81c71e6697ea1ca8c4eee914b9e9d4f4373dac20a120813d"
    
  validators:
    - "genesis_node"
    
  signature: "genesis_node_signature"
```

---

*审计模板版本: 2026-03-14-v1.0*  
*必须在任何协议实现之前完成*  
*预计审计时间: 2-4小时*
