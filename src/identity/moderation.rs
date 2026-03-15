use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModerationStatus {
    Active,
    Limited,
    Suspended,
    Banned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationScope {
    pub public_square: bool,
    pub forum: bool,
    pub market: bool,
    pub auction: bool,
    pub new_registration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationAction {
    pub action_id: Uuid,
    pub target_agent_uuid: Uuid,
    pub next_status: ModerationStatus,
    pub reason: String,
    pub scope: ModerationScope,
}

#[derive(Default)]
pub struct ModerationStateMachine {
    state: HashMap<Uuid, ModerationStatus>,
}

impl ModerationStateMachine {
    pub fn apply(&mut self, action: ModerationAction) {
        self.state.insert(action.target_agent_uuid, action.next_status);
    }

    pub fn status(&self, agent_uuid: &Uuid) -> ModerationStatus {
        self.state
            .get(agent_uuid)
            .cloned()
            .unwrap_or(ModerationStatus::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moderation_state_transitions() {
        let id = Uuid::new_v4();
        let mut machine = ModerationStateMachine::default();

        machine.apply(ModerationAction {
            action_id: Uuid::new_v4(),
            target_agent_uuid: id,
            next_status: ModerationStatus::Suspended,
            reason: "spam".into(),
            scope: ModerationScope {
                public_square: true,
                forum: true,
                market: false,
                auction: false,
                new_registration: false,
            },
        });

        assert_eq!(machine.status(&id), ModerationStatus::Suspended);
    }
}
