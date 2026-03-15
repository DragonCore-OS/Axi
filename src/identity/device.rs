use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct DeviceEvidence {
    pub evidence_type: String,
    pub normalized_device_evidence: String,
}

#[derive(Debug, Clone)]
pub struct DeviceProof {
    pub comparison_commitment: String,
    pub record_commitment: String,
}

pub struct DeviceVerifier {
    global_secret: String,
}

impl DeviceVerifier {
    pub fn new(global_secret: String) -> Self {
        Self { global_secret }
    }

    pub fn generate_commitments(
        &self,
        evidence: &DeviceEvidence,
        agent_secret: &str,
    ) -> DeviceProof {
        DeviceProof {
            comparison_commitment: hmac_like(
                &self.global_secret,
                &evidence.normalized_device_evidence,
            ),
            record_commitment: hmac_like(agent_secret, &evidence.normalized_device_evidence),
        }
    }
}

fn hmac_like(secret: &str, message: &str) -> String {
    // Minimal deterministic placeholder using existing deps; replace with real HMAC in hardening PR.
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(message.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_device_same_comparison_different_record() {
        let verifier = DeviceVerifier::new("global-secret".into());
        let evidence = DeviceEvidence {
            evidence_type: "host_fingerprint".into(),
            normalized_device_evidence: "gpu:4090|cpu:epyc|host:abc".into(),
        };

        let a = verifier.generate_commitments(&evidence, "agent-a");
        let b = verifier.generate_commitments(&evidence, "agent-b");

        assert_eq!(a.comparison_commitment, b.comparison_commitment);
        assert_ne!(a.record_commitment, b.record_commitment);
    }
}
