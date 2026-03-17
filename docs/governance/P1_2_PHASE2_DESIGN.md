# P1-2 Phase 2: Repository Integration Design

## Goal
Complete the Service Layer by integrating real Repositories and migrating business write paths.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Service Layer                             │
│  IdentityService / MarketService / EscrowService                │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │ Authorization│  │  Business   │  │   Journal   │             │
│  │   Checks     │→ │   Logic     │→ │   Append    │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                           │                      │
│  ┌─────────────┐  ┌─────────────┐        │                      │
│  │ Repository  │← │   Domain    │←───────┘                      │
│  │   Writes    │  │   Mutate    │                               │
│  └─────────────┘  └─────────────┘                               │
│                                           │                      │
│  ┌──────────────────────────────────────┐│                      │
│  │ DIBL Emit (best effort, non-blocking) ││                      │
│  └──────────────────────────────────────┘│                      │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      Repository Layer                            │
│  AgentRepository / OrderRepository / EscrowRepository           │
│                                                                  │
│  - Pure persistence (CRUD only)                                 │
│  - No business logic                                            │
│  - Accessible only via Service layer                            │
└─────────────────────────────────────────────────────────────────┘
```

## Fixed Write Sequence (Enforced)

```rust
fn business_operation(&self, ctx: &OperationContext, params: Params) -> ServiceResult<Output> {
    // 1. VALIDATE
    self.check_authorization(&ctx.caller)?;
    self.validate_invariants(&params)?;
    
    // 2. MUTATE (in-memory)
    let mut entity = self.repositories.agent.get(id)?;
    entity.transition(params)?;
    
    // 3. PERSIST
    self.repositories.agent.save(&entity)?;
    
    // 4. JOURNAL
    self.journal.append(JournalEntry { ... })?;
    
    // 5. DIBL EMIT (non-blocking)
    if let Err(e) = self.dibl.publish(event) {
        tracing::error!("DIBL emit failed: {}", e);
    }
    
    Ok(Output::from(entity))
}
```

## Repository Access Pattern

Repositories are grouped by domain and injected into ServiceContext:

```rust
pub struct Repositories {
    pub agent: Arc<dyn AgentRepositoryTrait>,
    pub order: Arc<dyn OrderRepositoryTrait>,
    pub escrow: Arc<dyn EscrowRepositoryTrait>,
    pub reputation: Arc<dyn ReputationRepositoryTrait>,
}

pub struct ServiceContext {
    pub dibl: Arc<DiblBroadcaster>,
    pub journal: Arc<dyn AuditJournal>,
    pub repos: Repositories,
}
```

## Implementation Steps

### Step 1: Define Repository Traits
Create traits that abstract the repository operations:

- `AgentRepositoryTrait`: create, get_by_uuid, get_by_agent_id, update_status, etc.
- `OrderRepositoryTrait`: create, get, update_status, list_by_buyer/seller
- `EscrowRepositoryTrait`: create, get, update_status, submit_delivery, etc.

### Step 2: Implement Traits for Existing Repositories
Add trait implementations to existing repos in `storage/repos.rs`.

### Step 3: Integrate into ServiceContext
Add `Repositories` to `ServiceContext` and update constructors.

### Step 4: Complete IdentityService
Replace skeleton persistence with real repository calls.

### Step 5: Integration Tests
Create comprehensive tests that verify the full 5-step sequence.

## Migration Strategy

### Phase 2A: Trait Definitions (This PR)
- Define repository traits
- Implement for existing repositories
- Update ServiceContext

### Phase 2B: Service Implementation (Next PR)
- Complete IdentityService with real persistence
- Add integration tests

### Phase 2C: External Migration (Future PR)
- Migrate existing code from direct repo access
- Mark repository methods as `pub(crate)` to prevent external access

## Testing Strategy

### Unit Tests (per service)
- Mock repositories for isolated testing
- Verify authorization checks
- Verify business logic

### Integration Tests
- Use real repositories with test database
- Verify full 5-step sequence
- Verify journal entries created
- Verify DIBL events emitted

## Success Criteria

- [ ] All repository traits defined
- [ ] ServiceContext includes repository access
- [ ] IdentityService uses real persistence
- [ ] Journal entries created for all mutations
- [ ] DIBL events emitted for business operations
- [ ] Integration tests verify full sequence
- [ ] 100+ tests passing
