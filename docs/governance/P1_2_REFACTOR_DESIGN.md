# P1-2 Repository/Service Refactor Design

## Problem Statement

Current architecture has business logic scattered:
- Repositories expose direct write interfaces (`update_status`, `add_wallet`)
- No unified transaction boundary for writes
- DIBL emission not integrated into business operations
- Inconsistent patterns across identity/market/escrow modules

## Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Service Layer                         │
│  (IdentityService, MarketService, EscrowService, etc.)      │
│                                                              │
│  Responsibility:                                             │
│  - Authorization checks                                      │
│  - Invariant validation                                      │
│  - Domain state mutation                                     │
│  - Orchestrate: persist → journal → dibl emit               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Repository Layer                        │
│  (AgentRepository, OrderRepository, EscrowRepository, etc.) │
│                                                              │
│  Responsibility:                                             │
│  - Pure persistence (CRUD)                                   │
│  - Query operations                                          │
│  - NO business logic                                         │
│  - NO direct exposure to external callers                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Infrastructure Layer                     │
│  (Storage, Journal, DIBL Broadcaster)                       │
└─────────────────────────────────────────────────────────────┘
```

## Fixed Write Sequence

Every business write operation MUST follow this sequence:

```rust
pub fn business_operation(&self, params) -> Result<Output, Error> {
    // 1. VALIDATE
    self.check_authorization(caller)?;
    self.validate_invariants(&params)?;
    
    // 2. MUTATE DOMAIN STATE
    let mut entity = self.repository.load(id)?;
    entity.transition(params)?;
    
    // 3. PERSIST
    self.repository.save(&entity)?;
    
    // 4. JOURNAL (audit log)
    self.journal.append(JournalEntry {
        entity_type: "...",
        entity_id: id,
        operation: "...",
        before: ...,
        after: ...,
    })?;
    
    // 5. DIBL EMIT (non-blocking)
    let event = GovernanceEvent::new(...);
    if let Err(e) = self.dibl.emit(event) {
        tracing::error!("DIBL emission failed: {}", e);
    }
    
    Ok(Output { ... })
}
```

## Module Structure

```
src/
├── service/                    # NEW: Unified service layer
│   ├── mod.rs                  # Service exports
│   ├── identity_service.rs     # Agent/wallet operations
│   ├── market_service.rs       # Listing/order operations (refactored)
│   ├── escrow_service.rs       # Escrow lifecycle
│   └── types.rs                # Service-level types
│
├── storage/
│   └── repos.rs                # Repository layer (simplified)
│
└── governance/
    └── ...                     # DIBL (already completed)
```

## Refactor Plan

### Phase 1: Create Service Layer Foundation
1. Create `src/service/mod.rs` with shared infrastructure
2. Define `ServiceContext` for dependency injection
3. Move existing `MarketService` from `market/` to `service/`

### Phase 2: Identity Service
1. Create `IdentityService` wrapping `AgentRepository`
2. Migrate business logic from repository to service:
   - Agent registration with validation
   - Wallet addition with ownership verification
   - Reputation updates
3. Repository retains only CRUD

### Phase 3: Escrow Service
1. Create `EscrowService` coordinating escrow lifecycle
2. Integrate DIBL emission for key events:
   - Escrow created
   - Delivery submitted
   - Delivery verified
   - Dispute opened/resolved

### Phase 4: Integration & Cleanup
1. Update all call sites to use Service layer
2. Remove dangerous direct repository exports
3. Add comprehensive integration tests

## Key Principles

1. **Repository is an implementation detail** - Only Service exposes public API
2. **Fail fast on validation** - Don't mutate if validation fails
3. **Journal is mandatory** - Every state change must be auditable
4. **DIBL is best-effort** - Never block business logic on event emission
5. **Transactions at Service level** - Repository operations are atomic, composition at Service

## DIBL Integration Points

| Service | Operation | DIBL Event |
|---------|-----------|------------|
| Identity | Agent registered | `AgentRegistered` |
| Identity | Wallet added | `WalletLinked` |
| Identity | Reputation changed | `ReputationUpdated` |
| Market | Listing created | `ListingCreated` |
| Market | Order placed | `OrderCreated` |
| Market | Order state changed | `OrderStateChanged` |
| Escrow | Escrow funded | `EscrowFunded` |
| Escrow | Delivery submitted | `DeliverySubmitted` |
| Escrow | Delivery verified | `DeliveryVerified` |
| Escrow | Dispute opened | `DisputeOpened` |

## Success Criteria

- [ ] No direct Repository usage outside Service layer
- [ ] All business writes follow 5-step sequence
- [ ] DIBL events emitted for all state changes
- [ ] Journal entries for all mutations
- [ ] 100% test coverage for Service layer
- [ ] Zero regression in existing functionality
