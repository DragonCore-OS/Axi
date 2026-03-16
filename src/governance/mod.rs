//! M3-1: Release Gating Logic
//! 
//! Minimal mainnet readiness gates with gradual rollout support.
//! Not a full feature flag platform - just blockers and percentage rollout.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Re-export serde for gates module
pub use serde;

pub mod gates;
pub mod features;
pub mod features_tests;

pub use gates::{ReleaseGates, SystemMetrics, GateResult, GateBlocker, RolloutState};
pub use features::{FeatureState, FeatureFlags, FeatureFlagStore};

/// Governance configuration for mainnet readiness
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    pub release_gates: ReleaseGates,
    pub rollout: RolloutConfig,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            release_gates: ReleaseGates::mainnet_minimum(),
            rollout: RolloutConfig::default(),
        }
    }
}

/// Rollout configuration (gradual release)
#[derive(Debug, Clone)]
pub struct RolloutConfig {
    /// Current rollout percentage (0-100)
    pub current_percentage: u8,
    /// Auto-increment step per successful period
    pub increment_step: u8,
    /// Hours between auto-increments
    pub increment_interval_hours: u64,
    /// Last increment timestamp
    pub last_increment_at: Option<DateTime<Utc>>,
    /// Target percentage for this phase
    pub target_percentage: u8,
    /// Emergency stop flag
    pub emergency_stop: bool,
}

impl Default for RolloutConfig {
    fn default() -> Self {
        Self {
            current_percentage: 0,
            increment_step: 10,
            increment_interval_hours: 24,
            last_increment_at: None,
            target_percentage: 100,
            emergency_stop: false,
        }
    }
}

impl RolloutConfig {
    /// Create rollout starting at 0%, targeting 100%
    pub fn gradual() -> Self {
        Self::default()
    }

    /// Create immediate 100% rollout
    pub fn immediate() -> Self {
        Self {
            current_percentage: 100,
            target_percentage: 100,
            ..Default::default()
        }
    }

    /// Check if rollout can auto-increment
    pub fn can_increment(&self, now: DateTime<Utc>) -> bool {
        if self.emergency_stop {
            return false;
        }
        if self.current_percentage >= self.target_percentage {
            return false;
        }
        match self.last_increment_at {
            None => true,
            Some(last) => {
                let hours_since = (now - last).num_hours() as u64;
                hours_since >= self.increment_interval_hours
            }
        }
    }

    /// Perform auto-increment if due
    pub fn try_increment(&mut self, now: DateTime<Utc>) -> bool {
        if !self.can_increment(now) {
            return false;
        }
        self.current_percentage = 
            (self.current_percentage + self.increment_step).min(self.target_percentage);
        self.last_increment_at = Some(now);
        true
    }

    /// Trigger emergency stop
    pub fn emergency_stop(&mut self) {
        self.emergency_stop = true;
    }

    /// Resume from emergency stop
    pub fn resume(&mut self) {
        self.emergency_stop = false;
    }
}

/// Feature availability based on rollout percentage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureAvailability {
    /// Feature fully disabled
    Disabled,
    /// Feature in gradual rollout (percentage based)
    Rollout { percentage: u8 },
    /// Feature fully enabled
    Enabled,
}

impl FeatureAvailability {
    /// Check if feature is available for a given agent (based on agent hash)
    pub fn is_available_for(&self, agent_uuid: &uuid::Uuid) -> bool {
        match self {
            FeatureAvailability::Disabled => false,
            FeatureAvailability::Enabled => true,
            FeatureAvailability::Rollout { percentage } => {
                // Deterministic based on agent UUID
                let hash = agent_uuid.to_u128_le();
                let bucket = (hash % 100) as u8;
                bucket < *percentage
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollout_config_starts_at_zero() {
        let config = RolloutConfig::gradual();
        assert_eq!(config.current_percentage, 0);
        assert_eq!(config.target_percentage, 100);
    }

    #[test]
    fn rollout_config_immediate_is_100() {
        let config = RolloutConfig::immediate();
        assert_eq!(config.current_percentage, 100);
    }

    #[test]
    fn rollout_auto_increments_when_due() {
        let mut config = RolloutConfig::gradual();
        let now = Utc::now();
        
        assert!(config.can_increment(now));
        assert!(config.try_increment(now));
        assert_eq!(config.current_percentage, 10);
    }

    #[test]
    fn rollout_respects_interval() {
        let mut config = RolloutConfig::gradual();
        let now = Utc::now();
        
        config.try_increment(now);
        
        // Cannot increment immediately
        assert!(!config.can_increment(now));
        assert!(!config.try_increment(now));
    }

    #[test]
    fn emergency_stop_prevents_increment() {
        let mut config = RolloutConfig::gradual();
        let now = Utc::now();
        
        config.emergency_stop();
        assert!(!config.can_increment(now));
        assert!(!config.try_increment(now));
    }

    #[test]
    fn feature_availability_rollout_deterministic() {
        let uuid = uuid::Uuid::new_v4();
        let rollout = FeatureAvailability::Rollout { percentage: 50 };
        
        // Same UUID always gives same result
        let first = rollout.is_available_for(&uuid);
        let second = rollout.is_available_for(&uuid);
        assert_eq!(first, second);
    }

    #[test]
    fn feature_availability_0_percent_disabled() {
        let uuid = uuid::Uuid::new_v4();
        let rollout = FeatureAvailability::Rollout { percentage: 0 };
        assert!(!rollout.is_available_for(&uuid));
    }

    #[test]
    fn feature_availability_100_percent_enabled() {
        let uuid = uuid::Uuid::new_v4();
        let rollout = FeatureAvailability::Rollout { percentage: 100 };
        assert!(rollout.is_available_for(&uuid));
    }
}
