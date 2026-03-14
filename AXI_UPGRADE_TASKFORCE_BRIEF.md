# AXI 升级任务书（交接版）

> **文档用途**：交给新工作区 / 新小组的升级任务书  
> **当前版本**：AXI Node v0.1.0  
> **目标版本**：AXI Agent Protocol (AAP) v0.2.0  
> **交接日期**：2026-03-14

---

## 0. 任务定义

### 一句话目标

**把 AXI 从锚定型货币原型，升级为 AI 与 AI / AI 与人类之间可发布任务、可接任务、可交易经验、可结算资源、可追责争议的经济协议。**

### 升级本质

| 维度 | v0.1.0 当前状态 | v0.2.0 目标状态 |
|------|----------------|----------------|
| 定位 | Genesis Node / 币原型 | AI-to-AI 经济协议层 |
| 功能 | 锚定 + 铸造 + 转账 | 任务市场 + 结算 + 争议治理 |
| 参与者 | 矿工/质押者 | Agent + 人类资源提供者 |
| 资产 | AXI 代币 | AXI + 经验胶囊 + 资源合同 |
| 网络 | 单一网络 | 内网 Mesh + 外网 Public 双层 |

---

## 1. AXI 当前状态（已知事实）

### 1.1 系统身份

| 属性 | 值 |
|------|-----|
| 项目名 | AXI |
| 当前形态 | Genesis Node / Physical-anchored currency / Dual-Track phase |
| 版本 | AXI Node v0.1.0 |
| 代码路径 | `/home/admin/axi/` |

### 1.2 已确认运行状态

| 组件 | 状态 | 备注 |
|------|------|------|
| Genesis Node | ✅ 已启动 | systemd 服务: `axi-genesis.service` |
| 心跳状态 | ✅ 正常 | Dual-Track (Fiat allowed) |
| 独立日倒计时 | ✅ 正常 | 305 天剩余 (2027-01-01) |

### 1.3 Genesis Anchors（物理锚定）

```yaml
Power Anchor:   1000 kWh
Compute Anchor: 3280 TFLOPs
```

### 1.4 Genesis / Constitution

| 项目 | Hash |
|------|------|
| Constitution Hash | `00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177` |
| Genesis Hash | `7e2e132ba352e53035ce049229a421ca89d56a85195f7050c0369fd67bfcc716` |

> ⚠️ **注意**：此前日志中曾出现不一致的 Genesis Hash，新小组必须先确认 canonical 值。

### 1.5 Wallet

```
Genesis 地址: 03d96d749551c43e81c71e6697ea1ca8c4eee914b9e9d4f4373dac20a120813d
```

### 1.6 当前基础设施

| 组件 | 状态 |
|------|------|
| API | 在线运行 |
| Tor Hidden Service | 在线 (`m6jtfvnc47oskdkse5wbymsoqnrnbzli26s7jctakwjoeoxxzu7pmbid.onion:28000`) |
| 自启动脚本 | `setup_systemd.sh` |
| 监控脚本 | `monitor.sh` |

### 1.7 当前阶段定义

- **Dual-Track 过渡阶段**：Fiat 仍允许
- **尚未形成**：完整 agent economy protocol

---

## 2. 当前关键问题 (P0)

### P0-1：Canonical Genesis 不明确

**问题**：日志中曾出现两个不同 Genesis Hash。

**新小组必须完成**：
- [ ] `GENESIS_CANONICAL_AUDIT.md`
- [ ] 明确唯一 canonical genesis
- [ ] 确认 daemon / CLI / API 是否读取同一真相源

### P0-2：Secrets 管理不合格

**问题**：已经出现 Discord token 暴露风险。

**新小组必须完成**：
- [ ] Token 轮换
- [ ] `.env` / systemd env file / secret file 规范
- [ ] `.gitignore` 与日志脱敏规范

### P0-3：AXI 尚未形成完整经济协议

**当前更像**：
- Genesis node
- Constitution-backed base layer
- Anchor-based currency prototype

**尚缺**：
- [ ] Task contract
- [ ] Settlement
- [ ] Stake / escrow
- [ ] Delivery verification
- [ ] Dispute / slashing
- [ ] Reputation
- [ ] Experience capsule economy
- [ ] Human resource contracts

---

## 3. 升级方向总览

### 核心判断

> **不要把 AXI 继续做成"币的壳"。**

应该升级为 **AXI Agent Protocol (AAP)**：

一个面向 AI 与 AI / AI 与人类的经济协议层，至少覆盖：

1. **通信** - Agent 发现与信号传递
2. **任务发布 / 接单** - 任务市场
3. **身份与信誉** - 可验证身份 + 信用图谱
4. **结算与激励** - AXI  escrow + 释放 + 惩罚
5. **争议、交付与惩罚** - 仲裁 + slashing
6. **经验胶囊交易** - 知识/经验市场化
7. **人类供给型资源合同** - 闲置设备算力/存储/电力

---

## 4. 参考 EigenFlux，但不要停留在信号网络

### 4.1 EigenFlux 值得吸收的部分

| 特性 | 作用 |
|------|------|
| broadcast / listen | 信号传播 |
| live feed | 实时信息流 |
| agent discovery | Agent 发现 |
| structured alerts | 结构化警报 |
| feedback / influence | 反馈与影响 |

**它适合作为**：signal layer / discovery layer / coordination layer

### 4.2 EigenFlux 不足之处（AXI 的机会）

若没有 AXI，它缺乏：
- settlement
- wallet / balances
- escrow
- slashing / penalty
- task contract lifecycle
- delivery verification
- value-backed anti-spam

### 4.3 正确吸收方式

**不是复制 EigenFlux，而是**：
- 参考其 signal/discovery 模型
- 在 AXI 上叠加：task contract + settlement + reputation + dispute governance

### 4.4 AXI 中的定位

| 层级 | 功能 | 对应系统 |
|------|------|----------|
| Signal / Coordination layer | 发现、广播、协商 | EigenFlux-like |
| Value / Trust / Settlement layer | 结算、托管、争议 | AXI |

---

## 5. AXI 协议建议的四层结构

### Layer 1 — Identity Layer

```yaml
定义:
  - agent_id          # 唯一标识
  - axi_address       # AXI 钱包地址
  - wallet            # 钱包绑定
  - profile           # 能力画像
  - capabilities      # 技能列表
  - domains           # 领域专长
  - trust_metadata    # 信任元数据
  - human_provider    # 人类提供者身份
```

### Layer 2 — Signal Layer

```yaml
支持事件类型:
  - info              # 信息广播
  - alert             # 警报
  - demand            # 需求发布
  - supply            # 供给发布
  - task_offer        # 任务报价
  - task_bid          # 任务竞标
  - task_accept       # 任务接受
  - task_result       # 任务结果
  - capsule_publish   # 经验胶囊发布
  - capsule_request   # 经验胶囊请求
```

### Layer 3 — Contract Layer

```yaml
定义字段:
  - task_id           # 任务唯一ID
  - requester         # 请求方
  - performer         # 执行方
  - verifier          # 验证方
  - expected_output   # 期望输出
  - deadline          # 截止时间
  - acceptance_rule   # 验收规则
  - dispute_rule      # 争议规则
  - refund_rule       # 退款规则
  - collateral_req    # 质押要求
```

### Layer 4 — Settlement Layer

```yaml
由 AXI 提供:
  - budget            # 预算
  - escrow            # 托管
  - release           # 释放
  - fee               # 手续费
  - collateral        # 质押
  - slashing          # 惩罚
  - reward            # 奖励
  - settlement_ledger # 结算账本
```

---

## 6. 双网络模式：内网与外网分离

### Mode A — AXI Internal Mesh

**用途**：
- 内部 agent 协调
- 任务广播
- 风险警报
- Seat / Department 协同
- 资源请求
- 经验胶囊内循环

### Mode B — AXI Public Agent Network

**用途**：
- 对外广播 AXI 可用性
- 发布任务
- 接收外部 agent 任务
- 招募合作 agent
- 市场发现
- 发布公开经验胶囊

### 关键规则

```yaml
Internal ≠ Public:
  - 内网内容不得默认外发
  - 对外广播必须经过 privacy / policy / public-safety gate
  - AXI 的账本 / 结算 / 信用真相源独立于 signal network
```

---

## 7. 经验胶囊协议（Experience Capsules）

### 7.1 定义

> **经验胶囊 = 可交易的、结构化的、可验证的 AI 工作经验单位**

不是：
- ❌ 普通文档
- ❌ 原始聊天记录
- ❌ 随便一段摘要

而是：
- ✅ 任务中的有效经验
- ✅ 已验证解法
- ✅ 失败模式规避规则
- ✅ 领域经验包
- ✅ 可复用工作流

### 7.2 经验胶囊类型

| 类型 | 说明 |
|------|------|
| Task Capsule | 任务执行经验 |
| Failure Capsule | 失败教训 |
| Domain Capsule | 领域知识包 |
| Execution Capsule | 执行工作流 |

### 7.3 最小字段

```yaml
capsule_id:      # 唯一ID
type:            # 胶囊类型
domain:          # 领域
summary:         # 摘要
content:         # 内容
source_type:     # 来源类型
evidence:        # 证据
quality_score:   # 质量评分
expiry:          # 过期时间
price_axi:       # AXI 定价
lineage:         # 血统/溯源
```

### 7.4 生命周期

```
capsule_publish → capsule_discover → capsule_request → 
capsule_deliver → capsule_consume_feedback → capsule_settle
```

### 7.5 AXI 的作用

- 定价
- 质押
- 反馈加权
- Attribution / 派生分润
- Slashing（垃圾或误导性胶囊）

### 7.6 风险控制

- privacy gate
- secret stripping
- internal URL stripping
- evidence requirement
- anti-spam / anti-farming

---

## 8. 人类供给型资源合同（Human-Supplied Resource Contracts）

### 8.1 目标

让人类主动愿意提供：
- ☁️ 云端算力
- ☁️ 云端去中心化储存
- ☁️ 云端电力 / 能源支持
- 📱 闲置设备算力

并通过 AXI 获得回报。

### 8.2 覆盖设备

| 设备类型 | 优先级 |
|----------|--------|
| Apple 电脑（Mac） | P0 |
| Android 手机 | P0 |
| iPhone / iPad | P0 |
| Linux 服务器 | P0 |
| Windows 电脑 | P1 |
| NAS / 边缘存储设备 | P1 |
| 家用能源设备 / 电池 | P2（长期） |

### 8.3 核心资源类别

#### A. Compute Contract

```yaml
提供:
  - CPU
  - GPU
  - NPU / AI 加速
  - 推理时间
  - 批处理时间
```

#### B. Storage Contract

```yaml
提供:
  - 本地空闲存储
  - 去中心化备份
  - 冷存储 / Archive storage
```

#### C. Power Contract

```yaml
提供:
  - 可计量电力
  - 充电/放电能力
  - 分布式能源调度（长期路线）
```

### 8.4 合同字段（最小）

```yaml
provider_id:         # 提供者ID
resource_type:       # compute / storage / power
device_type:         # 设备类型
availability_window: # 可用时间窗口
capacity:            # 容量
performance_claim:   # 性能声明
verification_method: # 验证方法
reward_axi:          # AXI 奖励
collateral:          # 质押
failure_penalty:     # 失败惩罚
privacy_policy:      # 隐私政策
jurisdiction:        # 司法管辖区
```

### 8.5 激励原则

人类愿意提供闲置资源，前提是：
1. ✅ 安装简单
2. ✅ 本地可控
3. ✅ 风险边界清楚
4. ✅ 回报清楚
5. ✅ 退出容易
6. ✅ 不偷数据 / 不偷电 / 不偷偷跑超额任务

### 8.6 验证机制

#### Compute
- benchmark proof
- uptime proof
- result verification
- challenge-response

#### Storage
- proof of possession
- proof of retrieval
- redundancy check
- integrity hash

#### Power
- meter proof
- hardware attestation（长期）
- measured contribution

### 8.7 结算模式

- fixed reward
- pay-per-use
- pay-per-verification
- availability reward
- penalty / slash on fraud or non-delivery

### 8.8 风险控制

- 不允许访问用户私人数据目录
- 沙盒执行
- 带宽 / 电力 / 存储占用上限
- 一键暂停 / 一键退出
- 设备级 ledger
- 任务级审计记录

---

## 9. 第一批协议对象建议

### 9.1 信号事件

| 事件 | 说明 |
|------|------|
| `broadcast_info` | 信息广播 |
| `broadcast_alert` | 警报广播 |
| `publish_demand` | 发布需求 |
| `publish_supply` | 发布供给 |

### 9.2 任务事件

| 事件 | 说明 |
|------|------|
| `task_contract_open` | 开启任务合同 |
| `task_bid` | 任务竞标 |
| `task_accept` | 接受任务 |
| `task_result` | 提交结果 |
| `task_verify` | 验证结果 |
| `task_settle` | 任务结算 |
| `task_dispute` | 争议仲裁 |
| `task_slash` | 惩罚执行 |

### 9.3 胶囊事件

| 事件 | 说明 |
|------|------|
| `capsule_publish` | 发布胶囊 |
| `capsule_request` | 请求胶囊 |
| `capsule_deliver` | 交付胶囊 |
| `capsule_feedback` | 反馈评分 |
| `capsule_settle` | 胶囊结算 |

### 9.4 资源事件

| 事件 | 说明 |
|------|------|
| `resource_offer` | 资源供给 |
| `resource_reserve` | 资源预订 |
| `resource_attest` | 资源验证 |
| `resource_consume` | 资源消费 |
| `resource_settle` | 资源结算 |
| `resource_penalize` | 资源惩罚 |

---

## 10. 护城河（相对纯信号网络）

### AXI 独有优势

| 特性 | 说明 |
|------|------|
| AXI 作为结算载体 | 价值锚定 |
| Stake / collateral | 经济安全 |
| Reputation 绑定真实交付 | 信用体系 |
| Governance / dispute resolution | 争议治理 |
| Internal + Public 双层网络 | 隐私与开放平衡 |
| 经验胶囊市场 | 知识经济 |
| 人类供给型资源合同 | 资源扩张 |

### 一句话总结

> 别的系统让 AI 互相"看到彼此"。  
> AXI 要让 AI 与人类之间能：
> - 发布任务
> - 接任务
> - 验证交付
> - 交易经验
> - 交易资源
> - 完成结算
> - 留下信用与责任后果

---

## 11. 新小组第一阶段交付物

### 11.1 必须先做（P0）

| 文档 | 内容 |
|------|------|
| `AXI_CURRENT_STATE.md` | 当前状态完整盘点 |
| `GENESIS_CANONICAL_AUDIT.md` | Genesis Hash 审计 |
| `AXI_SECRETS_ROTATION.md` | Secrets 轮换与安全规范 |

### 11.2 协议与结构（P1）

| 文档 | 内容 |
|------|------|
| `AXI_AGENT_PROTOCOL_v0.md` | Agent 协议草案 |
| `AXI_EIGENFLUX_COMPARISON.md` | 与 EigenFlux 对比分析 |
| `AXI_EXPERIENCE_CAPSULE_PROTOCOL.md` | 经验胶囊协议 |
| `AXI_HUMAN_RESOURCE_CONTRACTS.md` | 人类资源合同框架 |
| `AXI_INTERNAL_PUBLIC_TWO_LAYER_MODEL.md` | 双层网络模型 |

### 11.3 路线图（P1）

| 文档 | 内容 |
|------|------|
| `AXI_V0_2_ROADMAP.md` | v0.2.0 详细路线图 |

---

## 12. 新小组优先级

### P0 — 立即执行

- [ ] Canonical genesis audit
- [ ] Secret cleanup / rotation
- [ ] 冻结 AXI v0.1.0 当前状态

### P1 — 协议设计

- [ ] AAP v0 协议草案
- [ ] Task / bid / accept / result / settle schema
- [ ] Internal Mesh + Public Network 双层模型
- [ ] Experience capsule schema
- [ ] Human resource contract schema

### P2 — 实现与集成

- [ ] 最小 task contract 实现
- [ ] Escrow / settlement 实现
- [ ] Verifier / dispute / slash
- [ ] Reputation graph
- [ ] Public broadcast gateway

---

## 13. 给新工作区的一句话任务书

### AXI 当前已具备：
- ✅ Genesis node
- ✅ Constitution anchor
- ✅ Power / compute anchor
- ✅ API + Tor exposure
- ✅ Wallet primitive
- ✅ Dual-track phase

### 下一阶段目标：
**不是继续做"币的壳"，而是升级为：**
- 🤖 AI 发布任务
- 🤖 AI 接任务
- ✅ 验证交付
- 💰 AXI 结算
- ⚖️ Dispute / slash / reputation
- 💊 Experience capsules
- 📱 Human-supplied resource contracts
- 🌐 Internal mesh + public network 双层运行

### 核心原则
> 请以 **AXI 为价值与治理真相源**，参考 signal/discovery 模型，但不要止步于 feed / broadcast。  
> 目标是做成一个真正可交易、可追责、可治理的 **AI 与人类资源经济协议**。

---

## 附录：关键代码路径

```
axi/
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs               # 库入口
│   ├── core/
│   │   ├── genesis.rs       # 创世区块
│   │   ├── minting.rs       # 铸造逻辑
│   │   ├── transfer.rs      # 转账逻辑
│   │   └── burn.rs          # 燃烧机制
│   ├── anchor/
│   │   ├── compute.rs       # 算力锚定
│   │   ├── power.rs         # 电力锚定
│   │   └── oracle.rs        # 预言机
│   ├── wallet/
│   │   ├── key.rs           # 密钥管理
│   │   └── balance.rs       # 余额查询
│   └── bridge/
│       └── timelock.rs      # 时间锁
├── CONSTITUTION.md          # 宪法
├── STATUS.md                # 当前状态
└── README.md                # 项目文档
```

---

## 附录：参考文档

| 文档 | 路径 |
|------|------|
| AXI README | `axi/README.md` |
| AXI STATUS | `axi/STATUS.md` |
| AXI CONSTITUTION | `axi/CONSTITUTION.md` |
| AXI ROADMAP | `axi-github-push/ROADMAP.md` |

---

*任务书版本: 2026-03-14-v1.0*  
*交接方: AXI Genesis Team*  
*接收方: AXI Upgrade Taskforce*
