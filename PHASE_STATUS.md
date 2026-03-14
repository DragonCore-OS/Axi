# AXI Project Phase Status

> **Last Updated**: 2026-03-14  
> **Repository**: https://github.com/DragonCore-OS/Axi  
> **Current Phase**: TRUTH_SOURCE_AUDIT (唯一活跃)

---

## 阶段边界标记

```
[PHASE: DOCUMENT_PREP]      ✅ COMPLETE
                             ↓
[PHASE: TRUTH_SOURCE_AUDIT] 🔄 ACTIVE
                             ↓
[PHASE: MINIMAL_POC]        ⏸️ BLOCKED (等待审计签署)
```

---

## 冻结清单（直至 Genesis Audit 完成）

| 任务 | 状态 | 阻塞原因 |
|------|------|----------|
| Task Contract PoC | ⏸️ | escrow 需 genesis 作为信任根 |
| Escrow 实现 | ⏸️ | 需 canonical genesis 绑定 |
| Capsule Market | ⏸️ | 依赖 Task Contract |
| Human Resource | ⏸️ | 需 anchor 值确认 |
| Protocol Consistency | ⏸️ | 需代码基线稳定 |
| Security Queue | ⏸️ | 需审计对象确定 |

---

## 唯一活跃任务

### 📋 GENESIS_CANONICAL_AUDIT.md

**目标**：回答这四个关键问题

| 问题 | 当前状态 |
|------|----------|
| 唯一 canonical genesis hash | **TBD** - 审计中 |
| 唯一 canonical constitution hash | `00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177` |
| 所有 runtime 表面一致 | **TBD** - 待验证 |
| source of truth 路径 | **TBD** - 待确定 |

**完成标准**：第 8 节 Sign-off 已签署

---

## Canonical Values (Candidate)

```yaml
canonical_values:
  genesis_hash: "TBD - TO BE DETERMINED BY AUDIT"
  constitution_hash: "00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177"
  power_anchor_kwh: 1000
  compute_anchor_tflops: 3280
  genesis_timestamp: 1709256878  # 2024-03-01T00:00:00Z
  independence_timestamp: 1704067200  # 2027-01-01T00:00:00Z
  
  source_of_truth: "TBD"
```

---

## 已知问题

### Genesis Hash 不一致历史

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

### 可能原因
- [ ] 不同的 build 时间导致不同的 timestamp
- [ ] 不同的 constitution 文件内容（换行符、编码）
- [ ] 环境变量覆盖了硬编码值
- [ ] 多个 genesis 文件存在，daemon 读取了不同的文件
- [ ] 代码版本不一致
- [ ] 测试网 vs 主网配置混淆

---

## 执行边界

> **从现在起，AXI 项目的唯一进展指标不是"写了多少"，而是"canonical truth source 是否已被确认并签署"。**

这个阶段不用再扩文档，不用再开新分支，不用再谈实现。
先把真相源钉死。

---

## 进展指标

```
[░░░░░░░░░░░░░░░░░░] 0%

当前状态: 审计进行中
下一阶段: MINIMAL_POC (等待审计签署)
```

---

*阶段定义版本: 2026-03-14-v1.0*
