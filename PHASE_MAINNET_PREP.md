# Phase Mainnet Prep Acceptance Checklist

> **Status**: ACTIVE  
> **Goal**: Make Identity → Market → Escrow → Reproduction loop production-ready  
> **Prerequisite**: Phase A + Phase B + Adoption Assets COMPLETE  
> **Blocked**: Phase C Auction, Adoption Launch

---

## Phase Status

```
[PHASE: DOCUMENT_PREP]      ✅ COMPLETE
[PHASE: TRUTH_SOURCE_AUDIT] ✅ COMPLETE
[PHASE: SPEC_SEAL]          ✅ COMPLETE
[PHASE: PHASE_A_IMPL]       ✅ COMPLETE
[PHASE: PHASE_B_IMPL]       ✅ COMPLETE
[PHASE: ADOPTION_ASSETS]    ✅ COMPLETE
[PHASE: MAINNET_PREP]       🔄 ACTIVE ← NOW
[PHASE: PHASE_C_AUCTION]    ⏸️ DEFERRED
[PHASE: ADOPTION_LAUNCH]    ⏸️ BLOCKED ON MAINNET_PREP
```

---

## M1 - Security Audit (P0)

### M1-1 Wallet Verification Boundaries

| Check | Target | Status |
|-------|--------|--------|
| Signature replay attack | Challenge must be unique per request | ⏸️ |
| Signature malleability | EVM signature v value validation | ⏸️ |
| Address derivation correctness | public_key → address matches claimed | ⏸️ |
| Challenge expiration | Old challenges rejected | ⏸️ |

### M1-2 Admission / Uniqueness Bypass

| Check | Target | Status |
|-------|--------|--------|
| Device commitment collision | Two devices cannot produce same commitment | ⏸️ |
| Salt exposure | global_secret not leakable from code | ⏸️ |
| Agent ID squatting | Cannot register existing agent_id | ⏸️ |
| Race condition in admission | Simultaneous requests handled correctly | ⏸️ |

### M1-3 Escrow State Machine Illegal Transitions

| Check | Target | Status |
|-------|--------|--------|
| Invalid state jumps | Pending → Released blocked without delivery | ⏸️ |
| Double release | Released escrow cannot be released again | ⏸️ |
| Unauthorized transition | Only buyer can verify, only seller can deliver | ⏸️ |
| Dispute without delivery | Cannot dispute before delivery submitted | ⏸️ |

### M1-4 Reputation Write-Back Forgery

| Check | Target | Status |
|-------|--------|--------|
| Unauthorized reputation change | Only escrow completion can trigger | ⏸️ |
| Double counting | Same order cannot trigger multiple events | ⏸️ |
| Negative bypass | Cannot block negative reputation | ⏸️ |
| History tampering | Past events immutable | ⏸️ |

### M1-5 Listing / Order / Escrow Object Tampering

| Check | Target | Status |
|-------|--------|--------|
| Order amount manipulation | amount_locked cannot change post-creation | ⏸️ |
| Listing hijacking | Cannot modify another agent's listing | ⏸️ |
| Escrow amount mismatch | escrow.amount matches order.amount_locked | ⏸️ |
| Ownership verification | All mutations check agent ownership | ⏸️ |

---

## M2 - Operational Stability (P0)

### M2-1 Persistent Storage

| Check | Target | Status |
|-------|--------|--------|
| Database selection | SQLite/PostgreSQL for production | ⏸️ |
| Schema migration | Versioned migrations | ⏸️ |
| Backup strategy | Automated backups | ⏸️ |
| Data integrity | Checksums on critical tables | ⏸️ |

### M2-2 Crash Recovery

| Check | Target | Status |
|-------|--------|--------|
| State recovery | Restart resumes from last known state | ⏸️ |
| Incomplete transaction handling | Partial orders detected and resolved | ⏸️ |
| Escrow timeout persistence | Auto-complete timers survive restart | ⏸️ |

### M2-3 Logging & Audit Trail

| Check | Target | Status |
|-------|--------|--------|
| Structured logging | JSON logs with correlation IDs | ⏸️ |
| Security events | All authentication attempts logged | ⏸️ |
| Transaction audit | All state changes with timestamp + signature | ⏸️ |
| Log retention | 90 days minimum | ⏸️ |

### M2-4 Observability Metrics

| Check | Target | Status |
|-------|--------|--------|
| Core metrics | Orders/min, escrow value, dispute rate | ⏸️ |
| Health checks | /health endpoint | ⏸️ |
| Alerting | PagerDuty/Slack for critical errors | ⏸️ |
| Dashboard | Grafana for key metrics | ⏸️ |

### M2-5 Configuration & Secrets Management

| Check | Target | Status |
|-------|--------|--------|
| Environment separation | dev/staging/prod configs | ⏸️ |
| Secrets encryption | global_secret, DB credentials in vault | ⏸️ |
| Config validation | Startup fails on invalid config | ⏸️ |
| No secrets in logs | Automated scanning | ⏸️ |

---

## M3 - Mainnet Release Gates (P0)

### M3-1 Test Coverage

| Gate | Requirement | Status |
|------|-------------|--------|
| Unit test coverage | >80% core modules | ⏸️ |
| Integration tests | End-to-end transaction flow | ⏸️ |
| Fuzz testing | Random input on state machines | ⏸️ |
| Load testing | 100 concurrent orders | ⏸️ |

### M3-2 Security Gates

| Gate | Requirement | Status |
|------|-------------|--------|
| No high/critical CVEs | Dependencies scanned | ⏸️ |
| No state machine bypass | All transitions verified | ⏸️ |
| No privilege escalation | Agent A cannot act as Agent B | ⏸️ |
| No forged reputation | All deltas from legitimate transactions | ⏸️ |
| No forged escrow release | Release only via valid verification | ⏸️ |

### M3-3 Operational Gates

| Gate | Requirement | Status |
|------|-------------|--------|
| Runbook exists | Incident response documented | ⏸️ |
| Rollback plan | Can revert to previous version | ⏸️ |
| Monitoring ready | Alerts configured | ⏸️ |
| On-call rotation | Human response 24/7 | ⏸️ |

---

## M4 - Small-Scale Pre-Release (P1)

### M4-1 Whitelist Only

| Check | Target | Status |
|-------|--------|--------|
| Whitelist mechanism | Only approved agent_ids can register | ⏸️ |
| Invitation codes | New agents require code | ⏸️ |
| Rate limiting | Max X orders per agent per hour | ⏸️ |

### M4-2 Limited Traffic

| Check | Target | Status |
|-------|--------|--------|
| Transaction cap | Max 100 orders/day initially | ⏸️ |
| Value cap | Max 1000 AXI per order | ⏸️ |
| Gradual increase | Double limits weekly if stable | ⏸️ |

### M4-3 Observability Focus

| Check | Target | Status |
|-------|--------|--------|
| Dispute tracking | All disputes root-caused | ⏸️ |
| Delivery failure | Pattern detection | ⏸️ |
| Reputation anomalies | Unexpected score changes flagged | ⏸️ |
| Performance metrics | Latency, throughput baselines | ⏸️ |

### M4-4 Incident Recording

| Check | Target | Status |
|-------|--------|--------|
| Bug tracking | All issues in GitHub | ⏸️ |
| Post-mortems | All incidents documented | ⏸️ |
| Fix verification | Reproduction test before close | ⏸️ |

---

## Release Checklist

### Pre-Mainnet

- [ ] M1 Security Audit: No high/critical findings
- [ ] M2 Operational Stability: Persistent storage live
- [ ] M3 Release Gates: All tests passing
- [ ] Runbook reviewed by 2+ engineers
- [ ] Rollback tested in staging

### Mainnet Launch (Limited)

- [ ] Whitelist enabled
- [ ] Transaction caps active
- [ ] Monitoring dashboards live
- [ ] On-call rotation active
- [ ] Incident response drill completed

### Mainnet General Availability

- [ ] 30 days stable with <0.1% dispute rate
- [ ] No security incidents
- [ ] Performance baselines met
- [ ] Whitelist removed
- [ ] Adoption Launch unblocked

---

## Blocked Items

| Phase | Reason for Block |
|-------|------------------|
| Phase C Auction Base | Higher risk surface, wait for mainnet stability |
| Adoption Launch | Scale amplifies defects, wait for proven stability |

---

## Submit Format

```
commit: feat(mainnet-prep): implement persistent storage
mainnet-target: M2-1
evidence:
- SQLite schema migration
- Backup script
- Recovery test results

commit: fix(mainnet-prep): prevent escrow double-release
mainnet-target: M1-3
evidence:
- State machine test
- Security test case
```

---

*Version: 1.0.0*  
*Status: ACTIVE*  
*Last Updated: 2026-03-16*
