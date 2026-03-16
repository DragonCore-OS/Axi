# DIBL Schema v0.1 (AXI-DragonCore Aligned)

## 共享 JSONL 格式

每行一个 JSON 对象，代表一个治理事件。

### GovernanceEvent 结构

```json
{
  "event_id": "uuid-v4",
  "run_id": "string",
  "seat_id": "string|null",
  "channel": "Control|Ops|Security|Research",
  "event_type": "RunCreated|SeatStarted|SeatCompleted|RiskRaised|VetoExercised|FinalGateOpened|DecisionCommitted|RollbackTriggered|ArchiveCompleted|TerminateTriggered",
  "scope": "Internal|OperatorVisible|Exportable",
  "severity": "Info|Warn|Critical",
  "summary": "string",
  "details_ref": "string|null",
  "artifact_refs": ["string"],
  "created_at": "ISO8601-timestamp",
  "correlation_id": "string|null",
  "parent_event_id": "uuid|null",
  "actor": "string",
  "trigger_context": "string|null"
}
```

### CorrelationContext (Flattened)

| 字段 | 类型 | 说明 |
|------|------|------|
| `correlation_id` | `string|null` | 追踪 ID，关联一组相关事件 |
| `parent_event_id` | `uuid|null` | 父事件 ID，形成因果链 |
| `actor` | `string` | 触发者：seat名称/operator/system |
| `trigger_context` | `string|null` | 触发上下文/原因 |

### 枚举值

#### EventScope
- `Internal`: 仅内部治理（19-seat细节）
- `OperatorVisible`: 操作员/控制台可见
- `Exportable`: 可导出为报告

#### EventChannel
- `Control`: 治理推进（run创建、seat轮转、final gate）
- `Ops`: 运行时运维（tmux、worktree、state persistence）
- `Security`: 红线与风控（veto、terminate、rollback）
- `Research`: 复杂任务内部讨论结果

#### GovernanceEventType
| 类型 | 默认 Channel | 默认 Scope | 默认 Severity |
|------|-------------|------------|---------------|
| `RunCreated` | Control | OperatorVisible | Info |
| `SeatStarted` | Control | Internal | Info |
| `SeatCompleted` | Research | Internal | Info |
| `RiskRaised` | Security | OperatorVisible | Warn |
| `VetoExercised` | Security | Internal | Critical |
| `FinalGateOpened` | Control | OperatorVisible | Info |
| `DecisionCommitted` | Control | Exportable | Info |
| `RollbackTriggered` | Security | OperatorVisible | Critical |
| `ArchiveCompleted` | Ops | OperatorVisible | Info |
| `TerminateTriggered` | Security | OperatorVisible | Critical |

### 文件存储

```
runtime_state/events/{run_id}.jsonl
```

- 每个 run 一个文件
- 追加写入，支持事件回放
- 文件名 = run_id

## AXI ↔️ DragonCore 对齐状态

| 项目 | AXI | DragonCore | 状态 |
|------|-----|------------|------|
| `actor` (原 `triggered_by`) | ✅ | 添加中 | 命名统一 |
| `CorrelationContext` | ✅ | 添加中 | 结构一致 |
| JSONL 路径 | `runtime_state/events/` | 相同 | ✅ |
| 事件发射模式 | CLI观测层 | 8点已实现 | AXI待接入 |

## 测试向量

见 `test_vectors/axi_sample.jsonl` - AXI 生成的样本文件，供 DragonCore 验证互解析。

## Event Emission 模式 (DragonCore 参考)

```rust
// 1. 先持久化 JSON/ledger (source of truth)
self.store.save_run(&run)?;
ledger.record_veto(run_id, seat)?;

// 2. 然后 emit 事件（不阻塞操作）
let event = GovernanceEvent::new(run_id, GovernanceEventType::VetoExercised, "...")
    .with_seat(seat_id);
    
if let Err(e) = self.dibl.emit(event) {
    tracing::error!("Event emission failed: {}", e);  // 只记录，不失败
}
```

### DragonCore 8个接入点

| 操作 | 文件 | 函数 |
|------|------|------|
| RunCreated | `src/runtime/mod.rs` | `init_run()` |
| SeatStarted | `src/runtime/mod.rs` | `execute_seat()` |
| SeatCompleted | `src/runtime/mod.rs` | `execute_seat()` |
| VetoExercised | `src/runtime/mod.rs` | `exercise_veto()` |
| FinalGateOpened | `src/runtime/mod.rs` | `final_gate()` |
| DecisionCommitted | `src/runtime/mod.rs` | `final_gate()` |
| ArchiveCompleted | `src/runtime/mod.rs` | `archive_run()` |
| TerminateTriggered | `src/runtime/mod.rs` | `terminate_run()` |
