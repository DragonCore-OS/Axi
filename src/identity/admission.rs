//! Secure Admission Pipeline (P0-2 Fix)
//!
//! Integrates real wallet verification into admission flow.
//! No longer trusts user-submitted `wallet_verified` flag.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::device::{DeviceEvidence, DeviceVerifier};
use super::registry::{AgentIdentity, AgentRegistry, AgentStatus, WalletRef, WalletRole, WalletType};
use super::wallet_verification::{ChallengeStore, VerificationChallenge, VerificationResult, verify_wallet_ownership};

/// Admission state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdmissionState {
    Pending,
    ManualReview,
    Approved,
    Rejected,
}

/// Admission request with wallet verification proof (P0-2 Fix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionRequest {
    pub agent_id: String,
    pub display_name: String,
    pub signing_public_key: String,
    pub wallet_address: String,
    pub wallet_type: WalletType,
    pub wallet_signature: String,  // P0-2: Must provide valid signature
    pub challenge_id: String,      // P0-2: Must use valid challenge
    pub device_evidence: String,
}

/// Admission pipeline with integrated wallet verification
pub struct AdmissionPipeline {
    seen_comparison_commitments: HashSet<String>,
    challenge_store: ChallengeStore,
}

impl AdmissionPipeline {
    pub fn new() -> Self {
        Self {
            seen_comparison_commitments: HashSet::new(),
            challenge_store: ChallengeStore::new(),
        }
    }

    /// Generate a wallet verification challenge for the applicant
    pub fn generate_challenge(&self, agent_uuid: uuid::Uuid, wallet_address: &str) -> VerificationChallenge {
        VerificationChallenge::new(agent_uuid, wallet_address)
    }

    /// Submit admission request with wallet verification proof
    ///
    /// # P0-2 Security Flow:
    /// 1. Verify wallet ownership via challenge-response
    /// 2. If verification fails → Reject
    /// 3. If verification passes → Create agent with verified wallet
    /// 4. Check device uniqueness
    /// 5. Return appropriate admission state
    pub fn submit(
        &mut self,
        registry: &mut AgentRegistry,
        verifier: &DeviceVerifier,
        req: AdmissionRequest,
        challenge: &VerificationChallenge,  // P0-2: Must provide the challenge used for signing
        now: i64,
    ) -> Result<(AdmissionState, AgentIdentity), &'static str> {
        // P0-2 Step 1: Verify wallet ownership BEFORE creating agent
        let verification_result = verify_wallet_ownership(
            req.wallet_type.clone(),
            &req.wallet_address,
            challenge,
            &req.wallet_signature,
            &self.challenge_store,
            now,
        );

        // P0-2: Reject if wallet verification fails
        match verification_result {
            VerificationResult::Valid => {
                // Wallet verified successfully, proceed
            }
            VerificationResult::InvalidSignature => {
                return Err("wallet verification failed: invalid signature");
            }
            VerificationResult::InvalidAddress => {
                return Err("wallet verification failed: signature does not match address");
            }
            VerificationResult::ExpiredChallenge => {
                return Err("wallet verification failed: challenge expired");
            }
            VerificationResult::ReplayedNonce => {
                return Err("wallet verification failed: challenge already used");
            }
            VerificationResult::MalformedSignature => {
                return Err("wallet verification failed: malformed signature");
            }
        }

        // P0-2 Step 2: Create device commitments
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

        // P0-2 Step 3: Determine admission state based on device uniqueness
        let mut state = AdmissionState::Pending;
        if self
            .seen_comparison_commitments
            .contains(&proof.comparison_commitment)
        {
            state = AdmissionState::ManualReview;
        }

        // P0-2 Step 4: Create agent with verified wallet
        let mut agent = registry.create_agent(
            req.agent_id.clone(),
            req.display_name.clone(),
            req.signing_public_key,
            proof.comparison_commitment.clone(),
            proof.record_commitment,
        )?;

        // P0-2 Step 5: Attach verified wallet to agent
        let wallet = WalletRef {
            wallet_id: uuid::Uuid::new_v4(),
            agent_uuid: agent.agent_uuid,
            agent_id: req.agent_id,
            wallet_type: req.wallet_type,
            address: req.wallet_address,
            role: WalletRole::Primary,
            verified_ownership: true,  // P0-2: Explicitly marked as verified
            added_at: Utc::now().timestamp(),
            active_until: None,
        };

        registry.attach_wallet(wallet)?;

        // P0-2 Step 6: Set agent status
        match state {
            AdmissionState::Approved => {
                registry.set_status(&agent.agent_uuid, AgentStatus::Approved)?
            }
            AdmissionState::Rejected => {
                registry.set_status(&agent.agent_uuid, AgentStatus::Rejected)?
            }
            AdmissionState::ManualReview | AdmissionState::Pending => {}
        }

        // Update agent in registry to reflect wallet attachment
        agent = registry.get_by_uuid(&agent.agent_uuid)
            .ok_or("agent not found after creation")?
            .clone();

        self.seen_comparison_commitments
            .insert(proof.comparison_commitment);

        Ok((state, agent))
    }

    /// Check if wallet is verified for an agent
    pub fn is_wallet_verified(registry: &AgentRegistry, agent_uuid: &uuid::Uuid) -> bool {
        if let Some(agent) = registry.get_by_uuid(agent_uuid) {
            agent.wallets.iter().any(|w| w.verified_ownership)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{PublicKey, Secp256k1, SecretKey};
    use sha2::{Digest, Sha256};

    fn generate_test_keypair() -> (SecretKey, PublicKey, String) {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        // Derive Ethereum address using standard EVM derivation (Keccak-256)
        let address = crate::identity::pubkey_to_eth_address(&public_key);
        
        (secret_key, public_key, address)
    }

    fn sign_challenge_evm(secret_key: &SecretKey, challenge: &VerificationChallenge) -> String {
        let secp = Secp256k1::new();
        let message = challenge.to_signing_message();
        let msg_hash = Sha256::digest(message.as_bytes());
        let message = secp256k1::Message::from_slice(&msg_hash).unwrap();
        
        let (recovery_id, sig_bytes) = secp.sign_ecdsa_recoverable(&message, secret_key).serialize_compact();
        
        // Convert to Ethereum format (r[32] + s[32] + v[1])
        let v = recovery_id.to_i32() as u8 + 27;
        let mut full_sig = Vec::with_capacity(65);
        full_sig.extend_from_slice(&sig_bytes[0..32]);
        full_sig.extend_from_slice(&sig_bytes[32..64]);
        full_sig.push(v);
        
        format!("0x{}", hex::encode(&full_sig))
    }

    #[test]
    fn valid_wallet_verification_creates_agent_with_verified_wallet() {
        let mut registry = AgentRegistry::new();
        let verifier = DeviceVerifier::new("global-secret".into());
        let mut pipeline = AdmissionPipeline::new();

        // Generate test wallet
        let (secret_key, _pubkey, address) = generate_test_keypair();
        
        // Create challenge
        let challenge = pipeline.generate_challenge(uuid::Uuid::new_v4(), &address);
        let signature = sign_challenge_evm(&secret_key, &challenge);

        let challenge_nonce = challenge.nonce.clone();
        let req = AdmissionRequest {
            agent_id: "agent1".into(),
            display_name: "Agent 1".into(),
            signing_public_key: "pk1".into(),
            wallet_address: address,
            wallet_type: WalletType::Evm,
            wallet_signature: signature,
            challenge_id: challenge_nonce,
            device_evidence: "device-1".into(),
        };

        let now = Utc::now().timestamp();
        let (state, agent) = pipeline.submit(&mut registry, &verifier, req, &challenge, now).unwrap();

        assert_eq!(state, AdmissionState::Pending);
        assert_eq!(agent.wallets.len(), 1);
        assert!(agent.wallets[0].verified_ownership);
        assert_eq!(agent.wallets[0].role, WalletRole::Primary);
    }

    #[test]
    fn invalid_signature_rejected() {
        let mut registry = AgentRegistry::new();
        let verifier = DeviceVerifier::new("global-secret".into());
        let mut pipeline = AdmissionPipeline::new();

        // Generate test wallet
        let (_secret_key, _pubkey, address) = generate_test_keypair();

        // Create challenge (needed for API even though signature is invalid)
        let challenge = pipeline.generate_challenge(uuid::Uuid::new_v4(), &address);

        let req = AdmissionRequest {
            agent_id: "agent1".into(),
            display_name: "Agent 1".into(),
            signing_public_key: "pk1".into(),
            wallet_address: address,
            wallet_type: WalletType::Evm,
            wallet_signature: "0xinvalid".to_string(), // Invalid signature
            challenge_id: challenge.nonce.clone(),
            device_evidence: "device-1".into(),
        };

        let now = Utc::now().timestamp();
        let result = pipeline.submit(&mut registry, &verifier, req, &challenge, now);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("wallet verification failed"));
    }

    #[test]
    fn expired_challenge_rejected() {
        let mut registry = AgentRegistry::new();
        let verifier = DeviceVerifier::new("global-secret".into());
        let mut pipeline = AdmissionPipeline::new();

        // Generate test wallet
        let (secret_key, _pubkey, address) = generate_test_keypair();
        
        // Create challenge
        let challenge = pipeline.generate_challenge(uuid::Uuid::new_v4(), &address);
        let signature = sign_challenge_evm(&secret_key, &challenge);

        let challenge_nonce = challenge.nonce.clone();
        let req = AdmissionRequest {
            agent_id: "agent1".into(),
            display_name: "Agent 1".into(),
            signing_public_key: "pk1".into(),
            wallet_address: address,
            wallet_type: WalletType::Evm,
            wallet_signature: signature,
            challenge_id: challenge_nonce,
            device_evidence: "device-1".into(),
        };

        // Use future timestamp to simulate expiration
        let future_time = Utc::now().timestamp() + 10000;
        let result = pipeline.submit(&mut registry, &verifier, req, &challenge, future_time);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expired"));
    }

    #[test]
    fn duplicate_device_enters_manual_review() {
        let mut registry = AgentRegistry::new();
        let verifier = DeviceVerifier::new("global-secret".into());
        let mut pipeline = AdmissionPipeline::new();

        // First agent
        let (sk1, _pk1, addr1) = generate_test_keypair();
        let challenge1 = pipeline.generate_challenge(uuid::Uuid::new_v4(), &addr1);
        let sig1 = sign_challenge_evm(&sk1, &challenge1);

        let challenge1_nonce = challenge1.nonce.clone();
        let req1 = AdmissionRequest {
            agent_id: "agent1".into(),
            display_name: "Agent 1".into(),
            signing_public_key: "pk1".into(),
            wallet_address: addr1,
            wallet_type: WalletType::Evm,
            wallet_signature: sig1,
            challenge_id: challenge1_nonce,
            device_evidence: "same-device".into(),
        };

        // Second agent with same device
        let (sk2, _pk2, addr2) = generate_test_keypair();
        let challenge2 = pipeline.generate_challenge(uuid::Uuid::new_v4(), &addr2);
        let sig2 = sign_challenge_evm(&sk2, &challenge2);

        let challenge2_nonce = challenge2.nonce.clone();
        let req2 = AdmissionRequest {
            agent_id: "agent2".into(),
            display_name: "Agent 2".into(),
            signing_public_key: "pk2".into(),
            wallet_address: addr2,
            wallet_type: WalletType::Evm,
            wallet_signature: sig2,
            challenge_id: challenge2_nonce,
            device_evidence: "same-device".into(),
        };

        let now = Utc::now().timestamp();
        let first = pipeline.submit(&mut registry, &verifier, req1, &challenge1, now).unwrap();
        let second = pipeline.submit(&mut registry, &verifier, req2, &challenge2, now).unwrap();

        assert_eq!(first.0, AdmissionState::Pending);
        assert_eq!(second.0, AdmissionState::ManualReview);
        
        // Both should have verified wallets
        assert!(first.1.wallets[0].verified_ownership);
        assert!(second.1.wallets[0].verified_ownership);
    }

    #[test]
    fn unsupported_wallet_type_rejected() {
        let mut registry = AgentRegistry::new();
        let verifier = DeviceVerifier::new("global-secret".into());
        let mut pipeline = AdmissionPipeline::new();

        // Create a challenge (even though Solana is unsupported, API requires it)
        let challenge = pipeline.generate_challenge(uuid::Uuid::new_v4(), "sol1test");

        // Try to use unsupported wallet type
        let req = AdmissionRequest {
            agent_id: "agent1".into(),
            display_name: "Agent 1".into(),
            signing_public_key: "pk1".into(),
            wallet_address: "sol1test".to_string(),
            wallet_type: WalletType::Solana, // Not yet supported for verification
            wallet_signature: "0xsig".to_string(),
            challenge_id: challenge.nonce.clone(),
            device_evidence: "device-1".into(),
        };

        let now = Utc::now().timestamp();
        let result = pipeline.submit(&mut registry, &verifier, req, &challenge, now);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("wallet verification failed"));
    }
}
