//! Tests for feature flags

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::storage::PersistentStore;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn feature_state_deterministic_per_agent() {
        let feature = FeatureState {
            name: "test".to_string(),
            rollout_percentage: 50,
            enabled: true,
            description: "test".to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let agent_uuid = Uuid::new_v4();
        
        // Same agent, same result
        let first = feature.is_available_for(&agent_uuid);
        let second = feature.is_available_for(&agent_uuid);
        assert_eq!(first, second);
    }

    #[test]
    fn feature_0_percent_disabled_for_all() {
        let feature = FeatureState::new("test", "test");
        let agent_uuid = Uuid::new_v4();
        
        assert!(!feature.is_available_for(&agent_uuid));
    }

    #[test]
    fn feature_100_percent_enabled_for_all() {
        let mut feature = FeatureState::new("test", "test");
        feature.enable();
        feature.set_percentage(100);
        
        let agent_uuid = Uuid::new_v4();
        assert!(feature.is_available_for(&agent_uuid));
    }

    #[test]
    fn disabled_feature_blocked_regardless_of_percentage() {
        let mut feature = FeatureState::new("test", "test");
        feature.set_percentage(100);
        // enabled remains false
        
        let agent_uuid = Uuid::new_v4();
        assert!(!feature.is_available_for(&agent_uuid));
    }

    #[test]
    fn feature_flags_registry() {
        let flags = FeatureFlags::new();
        
        // Core features should be enabled
        let agent_uuid = Uuid::new_v4();
        assert!(flags.is_enabled("core_identity", &agent_uuid));
        assert!(flags.is_enabled("core_orders", &agent_uuid));
        
        // New features should be disabled
        assert!(!flags.is_enabled("escrow_auto_release", &agent_uuid));
    }

    #[test]
    fn feature_flags_enable_disable() {
        let mut flags = FeatureFlags::new();
        let agent_uuid = Uuid::new_v4();
        
        // Enable a feature
        flags.enable("escrow_auto_release").unwrap();
        flags.set_percentage("escrow_auto_release", 100).unwrap();
        assert!(flags.is_enabled("escrow_auto_release", &agent_uuid));
        
        // Disable it
        flags.disable("escrow_auto_release").unwrap();
        assert!(!flags.is_enabled("escrow_auto_release", &agent_uuid));
    }

    #[test]
    fn feature_flags_percentage_rollout() {
        let mut flags = FeatureFlags::new();
        
        // Enable at 50%
        flags.enable("escrow_auto_release").unwrap();
        flags.set_percentage("escrow_auto_release", 50).unwrap();
        
        // Check distribution roughly 50%
        let mut enabled_count = 0;
        for _ in 0..1000 {
            let agent_uuid = Uuid::new_v4();
            if flags.is_enabled("escrow_auto_release", &agent_uuid) {
                enabled_count += 1;
            }
        }
        
        // Should be roughly 500, allow 10% margin
        assert!(enabled_count > 400 && enabled_count < 600, 
                "Expected ~500, got {}", enabled_count);
    }

    #[test]
    fn feature_flag_store_persist_and_load() {
        let path = std::env::temp_dir().join(format!("test_features_{}.db", Uuid::new_v4()));
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        
        let feature_store = FeatureFlagStore::new(&conn);
        feature_store.init_schema().unwrap();
        
        // Create and save feature
        let mut feature = FeatureState::new("test_feature", "Test description");
        feature.enable();
        feature.set_percentage(75);
        feature_store.save(&feature).unwrap();
        
        // Load back
        let loaded = feature_store.load("test_feature").unwrap().unwrap();
        assert_eq!(loaded.name, "test_feature");
        assert_eq!(loaded.rollout_percentage, 75);
        assert!(loaded.enabled);
        assert_eq!(loaded.description, "Test description");
        
        let _ = fs::remove_file(path);
    }

    #[test]
    fn feature_flag_store_sync() {
        let path = std::env::temp_dir().join(format!("test_sync_{}.db", Uuid::new_v4()));
        let store = PersistentStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        
        let feature_store = FeatureFlagStore::new(&conn);
        feature_store.init_schema().unwrap();
        
        // Save custom feature to DB
        let feature = FeatureState {
            name: "custom_feature".to_string(),
            rollout_percentage: 100,
            enabled: true,
            description: "Custom".to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        feature_store.save(&feature).unwrap();
        
        // Sync to flags
        let mut flags = FeatureFlags::new();
        feature_store.sync_to_flags(&mut flags).unwrap();
        
        // Should have the custom feature
        let agent_uuid = Uuid::new_v4();
        assert!(flags.is_enabled("custom_feature", &agent_uuid));
        
        let _ = fs::remove_file(path);
    }
}
