-- M2-2 Relational Schema Migration
-- Version: 2
-- Adds relational tables for Identity, Market, Orders, Escrows, Reputation

-- =====================================================
-- AGENTS & IDENTITY
-- =====================================================

CREATE TABLE IF NOT EXISTS agents (
    agent_uuid TEXT PRIMARY KEY,           -- UUID v4
    agent_id TEXT UNIQUE NOT NULL,         -- Human-readable slug [a-zA-Z0-9_-]{3,64}
    display_name TEXT NOT NULL,
    public_key TEXT NOT NULL,
    representative_record_commitment TEXT NOT NULL,
    comparison_commitment TEXT NOT NULL,
    reputation_score INTEGER DEFAULT 0,
    status TEXT DEFAULT 'Pending',         -- Pending, Approved, Rejected, Suspended, Banned
    created_at TEXT NOT NULL,              -- RFC3339
    updated_at TEXT NOT NULL               -- RFC3339
);

CREATE INDEX IF NOT EXISTS idx_agents_agent_id ON agents(agent_id);
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);

-- =====================================================
-- WALLETS (1:N with agents)
-- =====================================================

CREATE TABLE IF NOT EXISTS wallets (
    wallet_id TEXT PRIMARY KEY,            -- UUID v4
    agent_uuid TEXT NOT NULL,
    wallet_type TEXT NOT NULL,             -- AxiNative, Evm, Btc, Solana, Other
    address TEXT NOT NULL,
    role TEXT DEFAULT 'Secondary',         -- Primary, Secondary, LegacyBridge
    verified_ownership BOOLEAN DEFAULT 0,
    added_at TEXT NOT NULL,
    active_until TEXT,                     -- NULL = active
    FOREIGN KEY (agent_uuid) REFERENCES agents(agent_uuid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_wallets_agent ON wallets(agent_uuid);
CREATE INDEX IF NOT EXISTS idx_wallets_address ON wallets(address);
CREATE UNIQUE INDEX IF NOT EXISTS idx_wallets_primary 
    ON wallets(agent_uuid) WHERE role = 'Primary';

-- =====================================================
-- ORDERS
-- =====================================================

CREATE TABLE IF NOT EXISTS orders (
    order_id TEXT PRIMARY KEY,             -- UUID v4
    listing_id TEXT NOT NULL,              -- UUID v4
    buyer_agent_uuid TEXT NOT NULL,
    seller_agent_uuid TEXT NOT NULL,
    amount_axi INTEGER NOT NULL CHECK (amount_axi > 0),
    amount_locked_axi INTEGER NOT NULL,
    status TEXT DEFAULT 'Open',            -- Open, InProgress, Delivered, Verified, Failed
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (buyer_agent_uuid) REFERENCES agents(agent_uuid),
    FOREIGN KEY (seller_agent_uuid) REFERENCES agents(agent_uuid)
);

CREATE INDEX IF NOT EXISTS idx_orders_buyer ON orders(buyer_agent_uuid);
CREATE INDEX IF NOT EXISTS idx_orders_seller ON orders(seller_agent_uuid);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_orders_listing ON orders(listing_id);

-- =====================================================
-- ESCROWS
-- =====================================================

CREATE TABLE IF NOT EXISTS escrows (
    escrow_id TEXT PRIMARY KEY,            -- UUID v4
    order_id TEXT UNIQUE NOT NULL,         -- 1:1 with orders
    buyer_agent_uuid TEXT NOT NULL,
    seller_agent_uuid TEXT NOT NULL,
    amount_axi INTEGER NOT NULL CHECK (amount_axi > 0),
    escrow_status TEXT DEFAULT 'Pending',  -- Pending, Funded, InEscrow, Released, Cancelled, Refunded, Disputed
    delivery_cid TEXT,                     -- IPFS CID
    delivery_uri TEXT,                     -- HTTPS URL
    delivery_note TEXT,
    delivery_submitted_at TEXT,
    buyer_verified_at TEXT,
    auto_complete_after TEXT,              -- 24h deadline
    dispute_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (order_id) REFERENCES orders(order_id),
    FOREIGN KEY (buyer_agent_uuid) REFERENCES agents(agent_uuid),
    FOREIGN KEY (seller_agent_uuid) REFERENCES agents(agent_uuid)
);

CREATE INDEX IF NOT EXISTS idx_escrows_order ON escrows(order_id);
CREATE INDEX IF NOT EXISTS idx_escrows_buyer ON escrows(buyer_agent_uuid);
CREATE INDEX IF NOT EXISTS idx_escrows_seller ON escrows(seller_agent_uuid);
CREATE INDEX IF NOT EXISTS idx_escrows_status ON escrows(escrow_status);

-- =====================================================
-- REPUTATION EVENTS (Audit Log - Immutable)
-- =====================================================

CREATE TABLE IF NOT EXISTS reputation_events (
    event_id TEXT PRIMARY KEY,             -- UUID v4
    agent_uuid TEXT NOT NULL,
    order_id TEXT,                         -- Nullable - some events not order-related
    event_type TEXT NOT NULL,              -- OrderCompleted, PositiveRating, NegativeRating, DisputeLost
    delta INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (agent_uuid) REFERENCES agents(agent_uuid),
    FOREIGN KEY (order_id) REFERENCES orders(order_id)
);

CREATE INDEX IF NOT EXISTS idx_rep_events_agent ON reputation_events(agent_uuid);
CREATE INDEX IF NOT EXISTS idx_rep_events_order ON reputation_events(order_id);
CREATE INDEX IF NOT EXISTS idx_rep_events_created ON reputation_events(created_at);

-- P1-1 Fix: Prevent duplicate reputation events for same order
CREATE UNIQUE INDEX IF NOT EXISTS idx_rep_events_unique 
    ON reputation_events(agent_uuid, order_id, event_type) 
    WHERE order_id IS NOT NULL;

-- =====================================================
-- VIEWS FOR COMMON QUERIES
-- =====================================================

-- Agent summary with wallet count
CREATE VIEW IF NOT EXISTS v_agent_summary AS
SELECT 
    a.agent_uuid,
    a.agent_id,
    a.display_name,
    a.reputation_score,
    a.status,
    COUNT(w.wallet_id) as wallet_count,
    a.created_at
FROM agents a
LEFT JOIN wallets w ON a.agent_uuid = w.agent_uuid AND w.active_until IS NULL
GROUP BY a.agent_uuid;

-- Order summary with escrow status
CREATE VIEW IF NOT EXISTS v_order_summary AS
SELECT 
    o.order_id,
    o.listing_id,
    o.buyer_agent_uuid,
    o.seller_agent_uuid,
    o.amount_axi,
    o.status as order_status,
    e.escrow_status,
    o.created_at,
    o.updated_at
FROM orders o
LEFT JOIN escrows e ON o.order_id = e.order_id;
