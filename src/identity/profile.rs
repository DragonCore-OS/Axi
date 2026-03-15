use serde::{Deserialize, Serialize};

use super::registry::AgentIdentity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicProfile {
    pub agent_id: String,
    pub display_name: String,
    pub primary_wallet: Option<String>,
    pub reputation_score: u64,
    pub admission_status: String,
    pub uniqueness_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateReviewRecord {
    pub agent_uuid: String,
    pub comparison_commitment: String,
    pub representative_record_commitment: String,
    pub internal_notes: Option<String>,
}

pub struct ProfileService;

impl ProfileService {
    pub fn public_profile(agent: &AgentIdentity) -> PublicProfile {
        let primary_wallet = agent
            .wallets
            .iter()
            .find(|w| matches!(w.role, super::registry::WalletRole::Primary))
            .map(|w| w.address.clone());

        PublicProfile {
            agent_id: agent.agent_id.clone(),
            display_name: agent.display_name.clone(),
            primary_wallet,
            reputation_score: 0,
            admission_status: format!("{:?}", agent.status),
            uniqueness_status: "verified_or_pending".into(),
        }
    }

    pub fn private_record(agent: &AgentIdentity) -> PrivateReviewRecord {
        PrivateReviewRecord {
            agent_uuid: agent.agent_uuid.to_string(),
            comparison_commitment: agent.comparison_commitment.clone(),
            representative_record_commitment: agent.representative_record_commitment.clone(),
            internal_notes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::registry::{AgentStatus, WalletRef, WalletRole, WalletType};
    use uuid::Uuid;

    #[test]
    fn public_profile_hides_private_commitments() {
        let agent_uuid = Uuid::new_v4();
        let agent = AgentIdentity {
            agent_uuid,
            agent_id: "Agent-1".into(),
            display_name: "Agent 1".into(),
            public_key: "pk".into(),
            representative_record_commitment: "record-secret".into(),
            comparison_commitment: "comparison-secret".into(),
            status: AgentStatus::Pending,
            wallets: vec![WalletRef {
                wallet_id: Uuid::new_v4(),
                agent_uuid,
                agent_id: "Agent-1".into(),
                wallet_type: WalletType::AxiNative,
                address: "axi1abc".into(),
                role: WalletRole::Primary,
                verified_ownership: true,
                added_at: 0,
                active_until: None,
            }],
            created_at: 0,
        };

        let public = ProfileService::public_profile(&agent);
        let serialized = serde_json::to_string(&public).unwrap();

        assert!(!serialized.contains("comparison-secret"));
        assert!(!serialized.contains("record-secret"));
    }
}
