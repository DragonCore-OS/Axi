# Mainnet Preparation Status

## Milestone Overview

| Phase | Module | Status | Tests | Commit |
|-------|--------|--------|-------|--------|
| **M1** | Security Audit + Fixes | 🔄 **IN PROGRESS** | 64/64 | [`95ebbde`](https://github.com/DragonCore-OS/Axi/commit/95ebbde) |
| **M2** | Operational Base | ✅ **COMPLETE** | 15/15 | - |
| **M3** | Release Gating | ✅ **COMPLETE** | 26/26 | - |
| **M4** | Pre-Release | ⏸️ **BLOCKED** | - | - |

---

## M1 Security Status

### ✅ FIXED

| ID | Vulnerability | Fix Commit | Tests |
|----|---------------|------------|-------|
| **P0-1** | Wallet Verification Bypass | `95ebbde` | ✅ 12 new tests |
| **P1-1** | Reputation Event Forgery | `95ebbde` | ✅ 7 new tests |

### 🔄 PENDING FIX

| ID | Vulnerability | Status |
|----|---------------|--------|
| **P0-2** | Admission Trusts Unverified Input | Partial (field removed) |
| **P0-3** | Missing Authorization Checks | Pending |
| **P1-2** | Direct DB Mutation Bypass | Pending |

### P0-1 Fix Details

**Before (Vulnerable)**:
```rust
// Only checked signature format, not ownership
pub fn verify_evm_ownership(...) -> VerificationResult {
    // Check 65 bytes, r/s non-zero, v valid...
    VerificationResult::Valid  // Always returned Valid!
}
```

**After (Secure)**:
```rust
pub fn verify_evm_ownership(
    wallet_address: &str,
    challenge: &VerificationChallenge,
    signature_hex: &str,
    challenge_store: &ChallengeStore,
    now: i64,
) -> VerificationResult {
    // 1. Check expiration
    if challenge.is_expired(now) { return ExpiredChallenge; }
    
    // 2. Check replay
    if challenge_store.is_used(&challenge.nonce) { return ReplayedNonce; }
    
    // 3. Recover public key from signature
    let pubkey = secp.recover_ecdsa(&message, &signature, &recovery_id)?;
    
    // 4. Derive address
    let recovered_address = pubkey_to_eth_address(&pubkey);
    
    // 5. Verify match
    if recovered_address != wallet_address { return InvalidAddress; }
    
    // 6. Mark nonce used
    challenge_store.mark_used(&challenge.nonce)?;
    
    VerificationResult::Valid
}
```

### P1-1 Fix Details

**Validation Rules**:
1. Order must exist
2. Agent must be buyer or seller
3. Order must be in `Verified` state for `OrderCompleted`
4. Escrow must be `Released` for `OrderCompleted`
5. No duplicate events: unique constraint on `(agent_uuid, order_id, event_type)`

---

## M2 Operational Base ✅

- M2-1: SQLite Persistence (5/5 tests)
- M2-2: Relational Schema (10/15 tests)  
- M2-3: Transaction Journal (15/15 tests)

---

## M3 Release Gating ✅

- M3-1: Release Gating Logic (17/17 tests)
- M3-2: Feature Flags (26/26 tests)

---

## Next Steps

1. **P0-2**: Integrate wallet verification into admission flow
2. **P0-3**: Add authorization checks to escrow operations
3. **P1-2**: Repository layer read-only, Service layer for writes
4. **Final Review**: Security audit sign-off

**Current Blockers for Mainnet**: P0-2, P0-3, P1-2
