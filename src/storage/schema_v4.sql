-- M2-4 Feature Flags Schema
-- Version: 4
-- Runtime-configurable feature flags with per-feature rollout

CREATE TABLE IF NOT EXISTS feature_flags (
    name TEXT PRIMARY KEY,
    rollout_percentage INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT 0,
    description TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON feature_flags(enabled);
