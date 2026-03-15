use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Pending,
    Approved,
    Rejected,
    Suspended,
    Banned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WalletType {
    AxiNative,
    Evm,
    Btc,
    Solana,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WalletRole {
    Primary,
    Secondary,
    LegacyBridge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRef {
    pub wallet_id: Uuid,
    pub agent_uuid: Uuid,
    pub agent_id: String,
    pub wallet_type: WalletType,
    pub address: String,
    pub role: WalletRole,
    pub verified_ownership: bool,
    pub added_at: i64,
    pub active_until: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub agent_uuid: Uuid,
    pub agent_id: String,
    pub display_name: String,
    pub public_key: String,
    pub representative_record_commitment: String,
    pub comparison_commitment: String,
    pub status: AgentStatus,
    pub wallets: Vec<WalletRef>,
    pub created_at: i64,
}

#[derive(Default)]
pub struct AgentRegistry {
    by_uuid: HashMap<Uuid, AgentIdentity>,
    slug_to_uuid: HashMap<String, Uuid>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_agent_id(agent_id: &str) -> Result<(), &'static str> {
        if !(3..=64).contains(&agent_id.len()) {
            return Err("agent_id length must be between 3 and 64");
        }
        if !agent_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err("agent_id must match [a-zA-Z0-9_-]{3,64}");
        }
        Ok(())
    }

    pub fn create_agent(
        &mut self,
        agent_id: String,
        display_name: String,
        public_key: String,
        comparison_commitment: String,
        representative_record_commitment: String,
    ) -> Result<AgentIdentity, &'static str> {
        Self::validate_agent_id(&agent_id)?;
        if self.slug_to_uuid.contains_key(&agent_id) {
            return Err("agent_id already exists");
        }

        let agent = AgentIdentity {
            agent_uuid: Uuid::new_v4(),
            agent_id: agent_id.clone(),
            display_name,
            public_key,
            representative_record_commitment,
            comparison_commitment,
            status: AgentStatus::Pending,
            wallets: vec![],
            created_at: Utc::now().timestamp(),
        };

        self.slug_to_uuid.insert(agent_id, agent.agent_uuid);
        self.by_uuid.insert(agent.agent_uuid, agent.clone());
        Ok(agent)
    }

    pub fn get_by_uuid(&self, agent_uuid: &Uuid) -> Option<&AgentIdentity> {
        self.by_uuid.get(agent_uuid)
    }

    pub fn get_by_agent_id(&self, agent_id: &str) -> Option<&AgentIdentity> {
        self.slug_to_uuid
            .get(agent_id)
            .and_then(|id| self.by_uuid.get(id))
    }

    pub fn attach_wallet(&mut self, wallet: WalletRef) -> Result<(), &'static str> {
        let agent = self
            .by_uuid
            .get_mut(&wallet.agent_uuid)
            .ok_or("agent not found")?;

        if wallet.role == WalletRole::Primary
            && agent
                .wallets
                .iter()
                .any(|w| w.role == WalletRole::Primary)
        {
            return Err("primary wallet already exists");
        }

        agent.wallets.push(wallet);
        Ok(())
    }

    pub fn set_status(
        &mut self,
        agent_uuid: &Uuid,
        status: AgentStatus,
    ) -> Result<(), &'static str> {
        let agent = self.by_uuid.get_mut(agent_uuid).ok_or("agent not found")?;
        agent.status = status;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separates_uuid_and_public_slug() {
        let mut registry = AgentRegistry::new();
        let agent = registry
            .create_agent(
                "KimiClaw-001".into(),
                "Kimi Claw".into(),
                "deadbeef".into(),
                "cmp1".into(),
                "rec1".into(),
            )
            .unwrap();

        assert_ne!(agent.agent_uuid.to_string(), agent.agent_id);
        assert_eq!(
            registry.get_by_agent_id("KimiClaw-001").unwrap().agent_uuid,
            agent.agent_uuid
        );
    }
}
