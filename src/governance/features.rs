//! M3-2: Feature Flags + Gradual Rollout
//!
//! Runtime-configurable feature flags with per-feature rollout percentages.
//! Deterministic agent eligibility based on UUID hash.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Feature flag state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureState {
    /// Feature name (unique identifier)
    pub name: String,
    /// Rollout percentage (0-100)
    pub rollout_percentage: u8,
    /// Master enable switch (can disable regardless of percentage)
    pub enabled: bool,
    /// Description for documentation
    pub description: String,
    /// Last updated timestamp
    pub updated_at: String,
}

impl FeatureState {
    /// Create new feature state at 0% rollout, disabled
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rollout_percentage: 0,
            enabled: false,
            description: description.into(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create feature fully enabled (100%)
    pub fn fully_enabled(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rollout_percentage: 100,
            enabled: true,
            description: description.into(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Check if this feature is available for a specific agent
    pub fn is_available_for(&self, agent_uuid: &Uuid) -> bool {
        if !self.enabled {
            return false;
        }
        if self.rollout_percentage == 0 {
            return false;
        }
        if self.rollout_percentage >= 100 {
            return true;
        }
        // Deterministic based on agent UUID
        let hash = agent_uuid.to_u128_le();
        let bucket = (hash % 100) as u8;
        bucket < self.rollout_percentage
    }

    /// Set rollout percentage (clamped to 0-100)
    pub fn set_percentage(&mut self, percentage: u8) {
        self.rollout_percentage = percentage.min(100);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Enable the feature (master switch)
    pub fn enable(&mut self) {
        self.enabled = true;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Disable the feature (master switch)
    pub fn disable(&mut self) {
        self.enabled = false;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

/// Feature flags registry
#[derive(Debug, Clone)]
pub struct FeatureFlags {
    features: HashMap<String, FeatureState>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        let mut flags = Self {
            features: HashMap::new(),
        };
        flags.register_defaults();
        flags
    }
}

impl FeatureFlags {
    /// Create with default features
    pub fn new() -> Self {
        Self::default()
    }

    /// Register default features
    fn register_defaults(&mut self) {
        // Core features (always on)
        self.register(FeatureState::fully_enabled(
            "core_identity",
            "Core identity management",
        ));
        self.register(FeatureState::fully_enabled(
            "core_orders",
            "Core order management",
        ));
        self.register(FeatureState::fully_enabled(
            "core_escrow",
            "Core escrow functionality",
        ));

        // Gradual rollout features
        self.register(FeatureState::new(
            "escrow_auto_release",
            "Automatic escrow release after delivery timeout",
        ));
        self.register(FeatureState::new(
            "reputation_bonuses",
            "Reputation score bonuses for streaks",
        ));
        self.register(FeatureState::new(
            "advanced_dispute",
            "Advanced dispute resolution with arbitration",
        ));
        self.register(FeatureState::new(
            "auction_preview",
            "Preview auction functionality (Phase C prep)",
        ));
    }

    /// Register a feature
    pub fn register(&mut self, feature: FeatureState) {
        self.features.insert(feature.name.clone(), feature);
    }

    /// Get feature state (returns disabled if not found)
    pub fn get(&self, name: &str) -> FeatureState {
        self.features.get(name).cloned().unwrap_or_else(|| {
            FeatureState::new(name, "Unknown feature")
        })
    }

    /// Check if feature is enabled for a specific agent
    pub fn is_enabled(&self, name: &str, agent_uuid: &Uuid) -> bool {
        self.get(name).is_available_for(agent_uuid)
    }

    /// Set rollout percentage for a feature
    pub fn set_percentage(&mut self, name: &str, percentage: u8) -> Result<(), &'static str> {
        let feature = self.features.get_mut(name)
            .ok_or("feature not found")?;
        feature.set_percentage(percentage);
        Ok(())
    }

    /// Enable a feature
    pub fn enable(&mut self, name: &str) -> Result<(), &'static str> {
        let feature = self.features.get_mut(name)
            .ok_or("feature not found")?;
        feature.enable();
        Ok(())
    }

    /// Disable a feature
    pub fn disable(&mut self, name: &str) -> Result<(), &'static str> {
        let feature = self.features.get_mut(name)
            .ok_or("feature not found")?;
        feature.disable();
        Ok(())
    }

    /// List all registered features
    pub fn list(&self) -> Vec<&FeatureState> {
        self.features.values().collect()
    }

    /// List features enabled for a specific agent
    pub fn list_enabled_for(&self, agent_uuid: &Uuid) -> Vec<&FeatureState> {
        self.features.values()
            .filter(|f| f.is_available_for(agent_uuid))
            .collect()
    }
}

/// Persistent feature flag storage
pub struct FeatureFlagStore<'a> {
    conn: &'a Connection,
}

impl<'a> FeatureFlagStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Initialize database schema for feature flags
    pub fn init_schema(&self) -> Result<(), String> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS feature_flags (
                name TEXT PRIMARY KEY,
                rollout_percentage INTEGER NOT NULL DEFAULT 0,
                enabled BOOLEAN NOT NULL DEFAULT 0,
                description TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL
            )",
            [],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Save feature state to database
    pub fn save(&self, feature: &FeatureState) -> Result<(), String> {
        self.conn.execute(
            "INSERT INTO feature_flags (name, rollout_percentage, enabled, description, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(name) DO UPDATE SET
                rollout_percentage = excluded.rollout_percentage,
                enabled = excluded.enabled,
                description = excluded.description,
                updated_at = excluded.updated_at",
            params![
                feature.name,
                feature.rollout_percentage,
                feature.enabled,
                feature.description,
                feature.updated_at,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Load feature state from database
    pub fn load(&self, name: &str) -> Result<Option<FeatureState>, String> {
        self.conn.query_row(
            "SELECT name, rollout_percentage, enabled, description, updated_at 
             FROM feature_flags WHERE name = ?1",
            params![name],
            |row| Ok(FeatureState {
                name: row.get(0)?,
                rollout_percentage: row.get(1)?,
                enabled: row.get(2)?,
                description: row.get(3)?,
                updated_at: row.get(4)?,
            }),
        ).optional().map_err(|e| e.to_string())
    }

    /// Load all features from database
    pub fn load_all(&self) -> Result<Vec<FeatureState>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT name, rollout_percentage, enabled, description, updated_at 
             FROM feature_flags ORDER BY name"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([], |row| {
            Ok(FeatureState {
                name: row.get(0)?,
                rollout_percentage: row.get(1)?,
                enabled: row.get(2)?,
                description: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut features = Vec::new();
        for row in rows {
            features.push(row.map_err(|e| e.to_string())?);
        }
        Ok(features)
    }

    /// Sync in-memory flags with database (load from DB, merge with defaults)
    pub fn sync_to_flags(&self, flags: &mut FeatureFlags) -> Result<(), String> {
        let db_features = self.load_all()?;
        for feature in db_features {
            flags.register(feature);
        }
        Ok(())
    }

    /// Persist all flags to database
    pub fn persist_all(&self, flags: &FeatureFlags) -> Result<(), String> {
        for feature in flags.list() {
            self.save(feature)?;
        }
        Ok(())
    }
}
