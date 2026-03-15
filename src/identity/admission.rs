use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::device::{DeviceEvidence, DeviceVerifier};
use super::registry::{AgentIdentity, AgentRegistry, AgentStatus};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdmissionState {
    Pending,
    ManualReview,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionRequest {
    pub agent_id: String,
    pub display_name: String,
    pub signing_public_key: String,
    pub wallet_address: String,
    pub wallet_verified: bool,
    pub device_evidence: String,
}

#[derive(Default)]
pub struct AdmissionPipeline {
    seen_comparison_commitments: HashSet<String>,
}

impl AdmissionPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(
        &mut self,
        registry: &mut AgentRegistry,
        verifier: &DeviceVerifier,
        req: AdmissionRequest,
    ) -> Result<(AdmissionState, AgentIdentity), &'static str> {
        let evidence = DeviceEvidence {
            evidence_type: "host_fingerprint".into(),
            normalized_device_evidence: req.device_evidence,
        };

        let agent_secret = format!(
            "{}:{}",
            req.agent_id,
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        let proof = verifier.generate_commitments(&evidence, &agent_secret);

        let mut state = AdmissionState::Pending;
        if !req.wallet_verified {
            state = AdmissionState::Rejected;
        } else if self
            .seen_comparison_commitments
            .contains(&proof.comparison_commitment)
        {
            state = AdmissionState::ManualReview;
        }

        let agent = registry.create_agent(
            req.agent_id,
            req.display_name,
            req.signing_public_key,
            proof.comparison_commitment.clone(),
            proof.record_commitment,
        )?;

        match state {
            AdmissionState::Approved => {
                registry.set_status(&agent.agent_uuid, AgentStatus::Approved)?
            }
            AdmissionState::Rejected => {
                registry.set_status(&agent.agent_uuid, AgentStatus::Rejected)?
            }
            AdmissionState::ManualReview | AdmissionState::Pending => {}
        }

        self.seen_comparison_commitments
            .insert(proof.comparison_commitment);

        Ok((state, agent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_device_enters_manual_review() {
        let mut registry = AgentRegistry::new();
        let verifier = DeviceVerifier::new("global-secret".into());
        let mut pipeline = AdmissionPipeline::new();

        let req1 = AdmissionRequest {
            agent_id: "agent1".into(),
            display_name: "Agent 1".into(),
            signing_public_key: "pk1".into(),
            wallet_address: "0x1".into(),
            wallet_verified: true,
            device_evidence: "same-device".into(),
        };

        let req2 = AdmissionRequest {
            agent_id: "agent2".into(),
            display_name: "Agent 2".into(),
            signing_public_key: "pk2".into(),
            wallet_address: "0x2".into(),
            wallet_verified: true,
            device_evidence: "same-device".into(),
        };

        let first = pipeline.submit(&mut registry, &verifier, req1).unwrap();
        let second = pipeline.submit(&mut registry, &verifier, req2).unwrap();

        assert_eq!(first.0, AdmissionState::Pending);
        assert_eq!(second.0, AdmissionState::ManualReview);
    }
}
