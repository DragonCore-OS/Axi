# Phase B Implementation Acceptance Checklist

> **Status**: ACTIVE IMPLEMENTATION PHASE  
> **Scope**: Transaction Base (Market + Escrow + Reputation)  
> **Do Not Expand**: This is the minimal验收标准, 不再扩规格

---

## 阶段判定

```
[PHASE: DOCUMENT_PREP]      ✅ COMPLETE
[PHASE: TRUTH_SOURCE_AUDIT] ✅ COMPLETE
[PHASE: SPEC_SEAL]          ✅ COMPLETE
[PHASE: PHASE_A_IMPL]       ✅ COMPLETE
[PHASE: PHASE_B_IMPL]       🔄 ACTIVE ← 当前阶段
```

---

## Phase B 核心目标

建立**交易闭环**:

```
Listing → Order → Escrow → Delivery → Settlement → Reputation
```

**三个必须实现的模块**（按顺序）:

| 顺序 | 模块 | 核心功能 |
|------|------|----------|
| 1 | **Market** | Listing / Order skeleton |
| 2 | **Escrow** | 状态机 (锁定→交付→释放/退款/争议) |
| 3 | **Reputation** | 交易结果回写 Agent Identity |

---

## B1 - Market (P0)

### B1-1 Listing Skeleton

| 检查项 | 验收标准 | 状态 |
|--------|----------|------|
| Listing 结构 | 符合 `listing.schema.json` | ⏸️ |
| CRUD API | 创建/读取/更新/删除 listing | ⏸️ |
| 分类索引 | 按 type/service/resource/job 索引 | ⏸️ |
| 定价模型 | fixed/quote/usage_based 支持 | ⏸️ |
| 搜索过滤 | 按标签/价格/类型搜索 | ⏸️ |

### B1-2 Order Skeleton

| 检查项 | 验收标准 | 状态 |
|--------|----------|------|
| Order 结构 | 关联 listing + buyer + seller | ⏸️ |
| 下单流程 | 从 listing 创建 order | ⏸️ |
| 订单状态 | open/in_progress/delivered/verified/failed | ⏸️ |
| 金额锁定 | 下单时锁定 AXI | ⏸️ |

---

## B2 - Escrow (P0)

### B2-1 Escrow State Machine

```
PENDING → FUNDED → IN_ESCROW → RELEASED
    ↓        ↓         ↓
 CANCELLED REFUNDED  DISPUTED → [ARBITRATION]
```

| 检查项 | 验收标准 | 状态 |
|--------|----------|------|
| 状态定义 | 完整 escrow_status 枚举 | ⏸️ |
| 状态流转 | 合法状态转换 | ⏸️ |
| 资金锁定 | 买家付款进入 escrow | ⏸️ |
| 释放机制 | 验证后释放给卖家 | ⏸️ |
| 退款机制 | 争议/失败时退款 | ⏸️ |

### B2-2 Delivery Verification

| 检查项 | 验收标准 | 状态 |
|--------|----------|------|
| 交付证据 | 支持 CID/URI 附件 | ⏸️ |
| 买家验证 | 确认收货接口 | ⏸️ |
| 自动完成 | 超时自动确认 | ⏸️ |
| 争议触发 | 不满意可发起争议 | ⏸️ |

---

## B3 - Reputation (P1)

### B3-1 Reputation Event Write-Back

| 检查项 | 验收标准 | 状态 |
|--------|----------|------|
| 事件记录 | 交易完成自动生成 | ⏸️ |
| 分数计算 | 基于交易结果计算 delta | ⏸️ |
| 回写机制 | 写入 Agent reputation | ⏸️ |
| 历史追踪 | 可查询 reputation 历史 | ⏸️ |

### B3-2 Reputation Impact

| 检查项 | 验收标准 | 状态 |
|--------|----------|------|
| 正向交易 | +5 分/完成订单 | ⏸️ |
| 好评 | +2 分/4-5星评价 | ⏸️ |
| 差评 | -5 分/1-2星评价 | ⏸️ |
| 争议败诉 | -10 分 | ⏸️ |

---

## 验收规则

### 通过标准

- ✅ B1 (Market): 可创建 listing，可下单
- ✅ B2 (Escrow): 资金锁定→交付→释放流程跑通
- ✅ B3 (Reputation): 交易结果回写 reputation

### 禁止事项

- ❌ 不要再写新规格文档
- ❌ 不要再抽象新框架
- ❌ 不要再扩展验收清单
- ❌ UI/前端实现
- ❌ 复杂推荐算法

### 提交格式

```
commit: feat(phase-b): implement [component]
target: B[1-3]-[N] [acceptance item]
evidence:
- changed files
- API/CLI 输出
- tests: cargo test [module]
```

---

## 当前状态

| 模块 | 进度 | 状态 |
|------|------|------|
| B1 Market | 0% | ⏸️ 等待实现 |
| B2 Escrow | 0% | ⏸️ 等待实现 |
| B3 Reputation | 0% | ⏸️ 等待实现 |

**Phase B Status**: 🔄 ACTIVE - No More Spec, Only Implementation

---

*Version: 1.0.0*  
*Status: ACTIVE - Phase B Implementation*  
*Last Updated: 2026-03-16*
