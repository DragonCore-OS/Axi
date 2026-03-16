-- M2-3 Transaction Journal
-- Version: 3
-- Append-only audit log for state changes (not full event sourcing)

-- =====================================================
-- TRANSACTION JOURNAL (Append-only, immutable)
-- =====================================================

CREATE TABLE IF NOT EXISTS transaction_journal (
    tx_id INTEGER PRIMARY KEY AUTOINCREMENT,  -- Monotonic sequence
    tx_uuid TEXT UNIQUE NOT NULL,             -- UUID for external reference
    tx_type TEXT NOT NULL,                    -- CreateAgent, UpdateAgentStatus, CreateOrder, etc.
    entity_type TEXT NOT NULL,                -- Agent, Order, Escrow, Wallet, etc.
    entity_id TEXT NOT NULL,                  -- UUID of the affected entity
    payload TEXT NOT NULL,                    -- JSON: before/after diff or full snapshot
    actor_uuid TEXT,                          -- Who initiated (agent UUID or system)
    tx_hash TEXT,                             -- SHA256(prev_hash + payload) for chain integrity
    prev_tx_id INTEGER,                       -- Previous tx_id for chain validation
    created_at TEXT NOT NULL,                 -- RFC3339
    
    -- Optional: foreign key if actor is an agent
    FOREIGN KEY (actor_uuid) REFERENCES agents(agent_uuid) ON DELETE SET NULL
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_tx_journal_type ON transaction_journal(tx_type);
CREATE INDEX IF NOT EXISTS idx_tx_journal_entity ON transaction_journal(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_tx_journal_actor ON transaction_journal(actor_uuid);
CREATE INDEX IF NOT EXISTS idx_tx_journal_created ON transaction_journal(created_at);

-- For chronological replay
CREATE INDEX IF NOT EXISTS idx_tx_journal_sequence ON transaction_journal(tx_id);

-- =====================================================
-- JOURNAL WATERMARK (for checkpoint/replay)
-- =====================================================

CREATE TABLE IF NOT EXISTS journal_watermarks (
    watermark_name TEXT PRIMARY KEY,          -- "last_snapshot", "recovery_point", etc.
    tx_id INTEGER NOT NULL,                   -- Last processed transaction
    tx_uuid TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- =====================================================
-- VIEW: Recent transactions
-- =====================================================

CREATE VIEW IF NOT EXISTS v_recent_transactions AS
SELECT 
    tx_id,
    tx_uuid,
    tx_type,
    entity_type,
    entity_id,
    actor_uuid,
    created_at,
    LENGTH(payload) as payload_size
FROM transaction_journal
ORDER BY tx_id DESC
LIMIT 100;

-- =====================================================
-- VIEW: Transactions by entity
-- =====================================================

CREATE VIEW IF NOT EXISTS v_entity_history AS
SELECT 
    entity_type,
    entity_id,
    tx_id,
    tx_type,
    actor_uuid,
    created_at
FROM transaction_journal
ORDER BY entity_type, entity_id, tx_id;
