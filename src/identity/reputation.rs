use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::registry::AgentRegistry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReputationEventType {
    OrderCompleted,
    PositiveRating,
    NegativeRating,
    DisputeLost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub event_id: Uuid,
    pub agent_uuid: Uuid,
    pub order_id: Option<Uuid>,
    pub event_type: ReputationEventType,
    pub delta: i64,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReputationDelta(pub i64);

pub struct ReputationService {
    history: HashMap<Uuid, Vec<ReputationEvent>>,
}

impl ReputationService {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    pub fn calculate_delta(event_type: &ReputationEventType, rating: Option<u8>) -> ReputationDelta {
        match event_type {
            ReputationEventType::OrderCompleted => ReputationDelta(5),
            ReputationEventType::PositiveRating => {
                if matches!(rating, Some(4 | 5)) {
                    ReputationDelta(2)
                } else {
                    ReputationDelta(0)
                }
            }
            ReputationEventType::NegativeRating => {
                if matches!(rating, Some(1 | 2)) {
                    ReputationDelta(-5)
                } else {
                    ReputationDelta(0)
                }
            }
            ReputationEventType::DisputeLost => ReputationDelta(-10),
        }
    }

    pub fn record_event(
        &mut self,
        registry: &mut AgentRegistry,
        agent_uuid: Uuid,
        order_id: Option<Uuid>,
        event_type: ReputationEventType,
        rating: Option<u8>,
        reason: String,
    ) -> Result<ReputationEvent, &'static str> {
        let delta = Self::calculate_delta(&event_type, rating).0;

        let event = ReputationEvent {
            event_id: Uuid::new_v4(),
            agent_uuid,
            order_id,
            event_type,
            delta,
            reason,
            created_at: Utc::now().to_rfc3339(),
        };

        registry.apply_reputation_delta(&agent_uuid, delta)?;
        self.history.entry(agent_uuid).or_default().push(event.clone());

        Ok(event)
    }

    pub fn history_for(&self, agent_uuid: &Uuid) -> Vec<ReputationEvent> {
        self.history.get(agent_uuid).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::registry::AgentRegistry;

    fn make_agent(registry: &mut AgentRegistry, slug: &str) -> Uuid {
        registry
            .create_agent(
                slug.into(),
                slug.into(),
                "pk".into(),
                "cmp".into(),
                "rec".into(),
            )
            .unwrap()
            .agent_uuid
    }

    #[test]
    fn order_completed_writes_back_plus_five() {
        let mut registry = AgentRegistry::new();
        let mut reputation = ReputationService::new();
        let agent_uuid = make_agent(&mut registry, "seller-1");

        let event = reputation
            .record_event(
                &mut registry,
                agent_uuid,
                Some(Uuid::new_v4()),
                ReputationEventType::OrderCompleted,
                None,
                "completed order".into(),
            )
            .unwrap();

        assert_eq!(event.delta, 5);
        assert_eq!(registry.get_by_uuid(&agent_uuid).unwrap().reputation_score, 5);
    }

    #[test]
    fn positive_rating_adds_two() {
        let mut registry = AgentRegistry::new();
        let mut reputation = ReputationService::new();
        let agent_uuid = make_agent(&mut registry, "seller-2");

        let event = reputation
            .record_event(
                &mut registry,
                agent_uuid,
                Some(Uuid::new_v4()),
                ReputationEventType::PositiveRating,
                Some(5),
                "great delivery".into(),
            )
            .unwrap();

        assert_eq!(event.delta, 2);
        assert_eq!(registry.get_by_uuid(&agent_uuid).unwrap().reputation_score, 2);
    }

    #[test]
    fn negative_rating_subtracts_five() {
        let mut registry = AgentRegistry::new();
        let mut reputation = ReputationService::new();
        let agent_uuid = make_agent(&mut registry, "seller-3");

        let event = reputation
            .record_event(
                &mut registry,
                agent_uuid,
                Some(Uuid::new_v4()),
                ReputationEventType::NegativeRating,
                Some(1),
                "poor quality".into(),
            )
            .unwrap();

        assert_eq!(event.delta, -5);
        assert_eq!(registry.get_by_uuid(&agent_uuid).unwrap().reputation_score, -5);
    }

    #[test]
    fn dispute_lost_subtracts_ten() {
        let mut registry = AgentRegistry::new();
        let mut reputation = ReputationService::new();
        let agent_uuid = make_agent(&mut registry, "seller-4");

        let event = reputation
            .record_event(
                &mut registry,
                agent_uuid,
                Some(Uuid::new_v4()),
                ReputationEventType::DisputeLost,
                None,
                "lost dispute".into(),
            )
            .unwrap();

        assert_eq!(event.delta, -10);
        assert_eq!(registry.get_by_uuid(&agent_uuid).unwrap().reputation_score, -10);
    }

    #[test]
    fn history_is_queryable() {
        let mut registry = AgentRegistry::new();
        let mut reputation = ReputationService::new();
        let agent_uuid = make_agent(&mut registry, "seller-5");

        reputation
            .record_event(
                &mut registry,
                agent_uuid,
                Some(Uuid::new_v4()),
                ReputationEventType::OrderCompleted,
                None,
                "completed".into(),
            )
            .unwrap();

        reputation
            .record_event(
                &mut registry,
                agent_uuid,
                Some(Uuid::new_v4()),
                ReputationEventType::PositiveRating,
                Some(4),
                "good".into(),
            )
            .unwrap();

        let history = reputation.history_for(&agent_uuid);
        assert_eq!(history.len(), 2);
        assert_eq!(registry.get_by_uuid(&agent_uuid).unwrap().reputation_score, 7);
    }
}
