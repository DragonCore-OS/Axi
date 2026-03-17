# DIBL v0.1 Release Notes

**DragonCore Internal Broadcast Layer**  
**Release Date:** 2026-03-16  
**Status:** ✅ Ready for controlled production pilot

---

## 1. What is DIBL?

DIBL (DragonCore Internal Broadcast Layer) is AXI's governance event system, adapted from the "dual-layer visibility + internal risk broadcast" principles of the AXI Private Mesh.

### Core Design Principles

1. **Persist First, Broadcast Second**  
   JSON/ledger is the source of truth; broadcast is derived.

2. **Three-Tier Visibility**  
   - `internal`: 19-seat governance internals only
   - `operator_visible`: Console/dashboard visible
   - `exportable`: External reports

3. **Run-Centered Lifecycle**  
   Events track governance runs, not open-ended chat messages.

---

## 2. What's Completed

### Module Status

| Module | File | Status | Description |
|--------|------|--------|-------------|
| Event Core | `src/governance/event.rs` | ✅ | Event definitions, CorrelationContext, serde |
| Storage | `src/governance/store.rs` | ✅ | JSONL persistence, EventStore trait |
| Broadcast | `src/governance/broadcast.rs` | ✅ | DiblBroadcaster, pub/sub |
| Projection | `src/governance/projection.rs` | ✅ | RunProjection, OperatorView |
| Runtime | `src/governance/runtime.rs` | ✅ | **8-point event emission** |
| CLI Tools | `src/main.rs` | ✅ | Observation commands |

### 8-Point Event Emission

All governance lifecycle events are now instrumented:

| # | Event | Method | Default Scope |
|---|-------|--------|---------------|
| 1 | `RunCreated` | `init_run()` | OperatorVisible |
| 2 | `SeatStarted` | `start_seat()` | Internal |
| 3 | `SeatCompleted` | `complete_seat()` | Internal |
| 4 | `RiskRaised` | `raise_risk()` | OperatorVisible |
| 5 | `VetoExercised` | `exercise_veto()` | Internal |
| 6 | `FinalGateOpened` | `open_final_gate()` | OperatorVisible |
| 7 | `DecisionCommitted` | `commit_decision()` | Exportable |
| 8 | `ArchiveCompleted` | `archive_run()` | OperatorVisible |
| 8b | `TerminateTriggered` | `terminate_run()` | OperatorVisible |

### Test Coverage

```
✅ 87 tests passing
✅ 0 tests failing
```

Key tests:
- `runtime_emission_8_points` - All emission points functional
- `emission_failure_not_blocking` - Fault tolerance verified
- `test_dragoncore_interop` - Cross-implementation compatibility
- `jsonl_roundtrip` - Serialization integrity

---

## 3. CLI Usage

### Installation

```bash
cargo build --release
```

### Commands

#### List all runs
```bash
$ axi runs
Available runs:
═══════════════════════════════════════
  sample-run-001  (4 events, run_created → veto_exercised)

Use 'axi runs --summary <run_id>' for details.
```

#### View run summary
```bash
$ axi runs --summary sample-run-001
Run Summary: sample-run-001
═══════════════════════════════════════
  Phase:        reviewing
  Current Seat: Yuheng
  Risk Status:  ⚠ Warning
  Open Risks:   1
    - Potential deadlock detected
  Veto Count:   0
  Elapsed:      10m
```

#### Watch events (real-time)
```bash
$ axi watch
Watching for events... (Press Ctrl+C to stop)
Filter: operator_visible events only
═══════════════════════════════════════
[10:00:00] sample-run-001 | run_created | info | Run created
[10:05:00] sample-run-001 | seat_started [Tianshu] | info | Seat started
[10:35:00] sample-run-001 | risk_raised [Yuheng] | warn | Potential deadlock
```

#### Watch specific run
```bash
$ axi watch --run sample-run-001
```

---

## 4. Schema Alignment with DragonCore

### Shared JSONL Format

Each line is a JSON object representing a governance event.

```json
{
  "event_id": "uuid-v4",
  "run_id": "string",
  "seat_id": "string|null",
  "channel": "control|ops|security|research",
  "event_type": "run_created|seat_started|...",
  "scope": "internal|operator_visible|exportable",
  "severity": "info|warn|critical",
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

### Alignment Points

| Aspect | AXI | DragonCore | Status |
|--------|-----|------------|--------|
| **Storage path** | `runtime_state/events/` | Same | ✅ |
| **File naming** | `{run_id}.jsonl` | Same | ✅ |
| **Serialization** | snake_case | snake_case | ✅ |
| **Event fields** | 15 fields | 15 fields | ✅ |
| **Correlation** | `actor` field | `actor` field | ✅ |

### Interoperability

AXI can parse DragonCore events and vice versa. Test vectors provided:
- `test_vectors/axi_sample.jsonl` - AXI-generated sample
- DragonCore provides equivalent for cross-validation

---

## 5. What's Not Done (Future Work)

### DIBL v0.2 Candidates

| Feature | Status | Notes |
|---------|--------|-------|
| `--channel` filter in CLI | ⏳ | Planned for next iteration |
| `--severity` filter in CLI | ⏳ | Minimum severity threshold |
| Real-time WebSocket feed | ⏳ | For external dashboards |
| Event replay from checkpoint | ⏳ | Resume from specific event |
| Cross-run aggregation | ⏳ | Multi-run operator views |

### Integration Points

- **P1-2 Repository/Service Refactor**: Will integrate DIBL emission into refactored service layer
- **19-seat runtime**: DIBL ready for full governance runtime integration
- **Ledger binding**: Events already reference ledger commits via `artifact_refs`

---

## 6. Files & References

### Source Files
```
src/governance/
├── mod.rs           # Module exports
├── event.rs         # Event definitions
├── store.rs         # JSONL persistence
├── broadcast.rs     # Pub/sub mechanism
├── projection.rs    # State aggregation
└── runtime.rs       # 8-point emission
```

### Documentation
```
docs/governance/
└── DIBL_v0_1_RELEASE.md    # This file

DIBL_SCHEMA.md               # Complete field reference
```

### Test Vectors
```
test_vectors/
└── axi_sample.jsonl         # Interop validation sample
```

---

## 7. Changelog

### v0.1.0 (2026-03-16)

**Added:**
- Initial DIBL implementation with 4-layer architecture
- 8-point event emission for governance lifecycle
- CLI observation tools (`runs`, `watch`)
- JSONL persistence compatible with runtime_state
- Snake_case serialization for DragonCore interop
- CorrelationContext with `actor` field
- 87 comprehensive tests

**Fixed:**
- Renamed `triggered_by` → `actor` for clarity
- Added `#[serde(rename_all = "snake_case")]` for compatibility

---

## 8. Verification

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
# 87 tests passing
```

### CLI Demo
```bash
mkdir -p runtime_state/events
cp test_vectors/axi_sample.jsonl runtime_state/events/
./target/release/axi runs --summary sample-run-001
```

---

**DIBL v0.1 is ready for controlled production use within verified boundaries.**  
All 6 commits pushed to `origin/main`.
