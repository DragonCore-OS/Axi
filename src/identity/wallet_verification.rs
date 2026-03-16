//! Secure Wallet Ownership Verification (P0-1 Fix)
//! 
//! Implements challenge-response with secp256k1 signature recovery.
//! Prevents replay attacks via challenge expiration and single-use nonces.

use secp256k1::{Message, PublicKey, Secp256k1, Signature, ecdsa::RecoveryId};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Mutex;
use uuid::Uuid;

/// Domain separator for wallet verification
const DOMAIN: &str = "axi.wallet-verification.v1";

/// Challenge nonce store (single-use, expires after use or timeout)
pub struct ChallengeStore {
    used_nonces: Mutex<HashSet<String>>,
}

impl ChallengeStore {
    pub fn new() -> Self {
        Self {
            used_nonces: Mutex::new(HashSet::new()),
        }
    }

    /// Mark a nonce as used
    pub fn mark_used(&self, nonce: &str) -> Result<(), &'static str> {
        let mut store = self.used_nonces.lock().map_err(|_| "lock poisoned")?;
        if !store.insert(nonce.to_string()) {
            return Err("nonce already used");
        }
        Ok(())
    }

    /// Check if nonce was used
    pub fn is_used(&self, nonce: &str) -> bool {
        self.used_nonces
            .lock()
            .map(|store| store.contains(nonce))
            .unwrap_or(true) // fail closed
    }
}

impl Default for ChallengeStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification challenge structure
#[derive(Debug, Clone)]
pub struct VerificationChallenge {
    pub agent_uuid: Uuid,
    pub wallet_address: String,
    pub nonce: String,
    pub issued_at: i64,
    pub expires_at: i64,
    pub domain: String,
}

impl VerificationChallenge {
    /// Create new challenge with 15 minute expiration
    pub fn new(agent_uuid: Uuid, wallet_address: impl Into<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            agent_uuid,
            wallet_address: wallet_address.into(),
            nonce: Uuid::new_v4().to_string(),
            issued_at: now,
            expires_at: now + 900, // 15 minutes
            domain: DOMAIN.to_string(),
        }
    }

    /// Serialize challenge to string for signing
    pub fn to_signing_message(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.domain,
            self.agent_uuid,
            self.wallet_address.to_lowercase(),
            self.nonce,
            self.issued_at,
            self.expires_at
        )
    }

    /// Check if challenge is expired
    pub fn is_expired(&self, now: i64) -> bool {
        now > self.expires_at
    }
}

/// Verification result
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    Valid,
    InvalidSignature,
    InvalidAddress,
    ExpiredChallenge,
    ReplayedNonce,
    MalformedSignature,
}

/// Recover Ethereum address from secp256k1 public key
fn pubkey_to_eth_address(pubkey: &PublicKey) -> String {
    let pubkey_bytes = pubkey.serialize_uncompressed();
    // Skip 0x04 prefix, hash remaining 64 bytes
    let hash = Sha256::digest(&Sha256::digest(&pubkey_bytes[1..]));
    // Take last 20 bytes, prefix with 0x
    format!("0x{}", hex::encode(&hash[12..32]))
}

/// Verify EVM wallet ownership via challenge-response with signature recovery
/// 
/// # Arguments
/// * `wallet_address` - The claimed wallet address (must match recovered address)
/// * `challenge` - The verification challenge
/// * `signature_hex` - The signature (65 bytes: r[32] + s[32] + v[1])
/// * `challenge_store` - Store to prevent replay attacks
/// 
/// # Returns
/// * `VerificationResult::Valid` if signature is valid and addresses match
pub fn verify_evm_ownership(
    wallet_address: &str,
    challenge: &VerificationChallenge,
    signature_hex: &str,
    challenge_store: &ChallengeStore,
    now: i64,
) -> VerificationResult {
    // Check challenge expiration
    if challenge.is_expired(now) {
        return VerificationResult::ExpiredChallenge;
    }

    // Check for replay
    if challenge_store.is_used(&challenge.nonce) {
        return VerificationResult::ReplayedNonce;
    }

    // Parse signature
    let sig_bytes = match hex::decode(signature_hex.trim_start_matches("0x")) {
        Ok(b) => b,
        Err(_) => return VerificationResult::MalformedSignature,
    };

    if sig_bytes.len() != 65 {
        return VerificationResult::MalformedSignature;
    }

    // Extract r, s, v
    let r = &sig_bytes[0..32];
    let s = &sig_bytes[32..64];
    let v = sig_bytes[64];

    // Validate v (27/28 or 0/1 for EIP-155 compatible)
    let rec_id = match v {
        27 | 0 => 0u8,
        28 | 1 => 1u8,
        _ => return VerificationResult::MalformedSignature,
    };

    // Build secp256k1 signature
    let mut sig_bytes_64 = [0u8; 64];
    sig_bytes_64[0..32].copy_from_slice(r);
    sig_bytes_64[32..64].copy_from_slice(s);
    
    let signature = match Signature::from_compact(&sig_bytes_64) {
        Ok(s) => s,
        Err(_) => return VerificationResult::MalformedSignature,
    };

    let recovery_id = match RecoveryId::from_i32(rec_id as i32) {
        Ok(id) => id,
        Err(_) => return VerificationResult::MalformedSignature,
    };

    // Hash the challenge message
    let message = challenge.to_signing_message();
    let msg_hash = Sha256::digest(message.as_bytes());
    let message = match Message::from_slice(&msg_hash) {
        Ok(m) => m,
        Err(_) => return VerificationResult::MalformedSignature,
    };

    // Recover public key
    let secp = Secp256k1::new();
    let pubkey = match secp.recover_ecdsa(&message, &signature, &recovery_id) {
        Ok(pk) => pk,
        Err(_) => return VerificationResult::InvalidSignature,
    };

    // Derive address from recovered public key
    let recovered_address = pubkey_to_eth_address(&pubkey);

    // Compare addresses (case-insensitive)
    if recovered_address.to_lowercase() != wallet_address.to_lowercase() {
        return VerificationResult::InvalidAddress;
    }

    // Mark nonce as used (prevent replay)
    if challenge_store.mark_used(&challenge.nonce).is_err() {
        return VerificationResult::ReplayedNonce;
    }

    VerificationResult::Valid
}

/// Verify AXI native wallet ownership (same as EVM)
pub fn verify_axi_ownership(
    wallet_address: &str,
    challenge: &VerificationChallenge,
    signature_hex: &str,
    challenge_store: &ChallengeStore,
    now: i64,
) -> VerificationResult {
    verify_evm_ownership(wallet_address, challenge, signature_hex, challenge_store, now)
}

/// Verify wallet ownership based on wallet type
pub fn verify_wallet_ownership(
    wallet_type: super::registry::WalletType,
    wallet_address: &str,
    challenge: &VerificationChallenge,
    signature_hex: &str,
    challenge_store: &ChallengeStore,
    now: i64,
) -> VerificationResult {
    use super::registry::WalletType;
    match wallet_type {
        WalletType::Evm | WalletType::AxiNative => {
            verify_evm_ownership(wallet_address, challenge, signature_hex, challenge_store, now)
        }
        WalletType::Btc | WalletType::Solana | WalletType::Other => {
            // For non-EVM wallets, return invalid until properly implemented
            // This prevents bypass by using unsupported wallet types
            VerificationResult::InvalidSignature
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::SecretKey;

    fn generate_test_keypair() -> (SecretKey, PublicKey) {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        (secret_key, public_key)
    }

    fn sign_challenge(secret_key: &SecretKey, challenge: &VerificationChallenge) -> String {
        let secp = Secp256k1::new();
        let message = challenge.to_signing_message();
        let msg_hash = Sha256::digest(message.as_bytes());
        let message = Message::from_slice(&msg_hash).unwrap();
        
        let sig = secp.sign_ecdsa_recoverable(&message, secret_key);
        let (recovery_id, sig_bytes) = sig.serialize_compact();
        
        // Convert to Ethereum format (r[32] + s[32] + v[1])
        let v = recovery_id.to_i32() as u8 + 27;
        let mut full_sig = Vec::with_capacity(65);
        full_sig.extend_from_slice(&sig_bytes[0..32]);
        full_sig.extend_from_slice(&sig_bytes[32..64]);
        full_sig.push(v);
        
        format!("0x{}", hex::encode(&full_sig))
    }

    #[test]
    fn valid_signature_passes_verification() {
        let (secret_key, pubkey) = generate_test_keypair();
        let wallet_address = pubkey_to_eth_address(&pubkey);
        let agent_uuid = Uuid::new_v4();
        let challenge = VerificationChallenge::new(agent_uuid, &wallet_address);
        let signature = sign_challenge(&secret_key, &challenge);
        let store = ChallengeStore::new();
        let now = chrono::Utc::now().timestamp();

        let result = verify_evm_ownership(
            &wallet_address,
            &challenge,
            &signature,
            &store,
            now,
        );

        assert_eq!(result, VerificationResult::Valid);
    }

    #[test]
    fn wrong_address_fails_verification() {
        let (secret_key, _pubkey) = generate_test_keypair();
        let wrong_address = "0x1234567890123456789012345678901234567890";
        let agent_uuid = Uuid::new_v4();
        let challenge = VerificationChallenge::new(agent_uuid, wrong_address);
        let signature = sign_challenge(&secret_key, &challenge);
        let store = ChallengeStore::new();
        let now = chrono::Utc::now().timestamp();

        let result = verify_evm_ownership(
            wrong_address,
            &challenge,
            &signature,
            &store,
            now,
        );

        assert_eq!(result, VerificationResult::InvalidAddress);
    }

    #[test]
    fn expired_challenge_rejected() {
        let (secret_key, pubkey) = generate_test_keypair();
        let wallet_address = pubkey_to_eth_address(&pubkey);
        let agent_uuid = Uuid::new_v4();
        let challenge = VerificationChallenge::new(agent_uuid, &wallet_address);
        let signature = sign_challenge(&secret_key, &challenge);
        let store = ChallengeStore::new();
        
        // 1 hour in the future (challenge expired)
        let future_time = challenge.expires_at + 3600;

        let result = verify_evm_ownership(
            &wallet_address,
            &challenge,
            &signature,
            &store,
            future_time,
        );

        assert_eq!(result, VerificationResult::ExpiredChallenge);
    }

    #[test]
    fn replayed_nonce_rejected() {
        let (secret_key, pubkey) = generate_test_keypair();
        let wallet_address = pubkey_to_eth_address(&pubkey);
        let agent_uuid = Uuid::new_v4();
        let challenge = VerificationChallenge::new(agent_uuid, &wallet_address);
        let signature = sign_challenge(&secret_key, &challenge);
        let store = ChallengeStore::new();
        let now = chrono::Utc::now().timestamp();

        // First verification succeeds
        let result1 = verify_evm_ownership(
            &wallet_address,
            &challenge,
            &signature,
            &store,
            now,
        );
        assert_eq!(result1, VerificationResult::Valid);

        // Second verification with same nonce fails (replay)
        let result2 = verify_evm_ownership(
            &wallet_address,
            &challenge,
            &signature,
            &store,
            now,
        );
        assert_eq!(result2, VerificationResult::ReplayedNonce);
    }

    #[test]
    fn malformed_signature_rejected() {
        let wallet_address = "0x1234567890123456789012345678901234567890";
        let agent_uuid = Uuid::new_v4();
        let challenge = VerificationChallenge::new(agent_uuid, wallet_address);
        let store = ChallengeStore::new();
        let now = chrono::Utc::now().timestamp();

        // Too short
        let result = verify_evm_ownership(
            wallet_address,
            &challenge,
            "0x1234",
            &store,
            now,
        );
        assert_eq!(result, VerificationResult::MalformedSignature);
    }

    #[test]
    fn invalid_signature_rejected() {
        let wallet_address = "0x1234567890123456789012345678901234567890";
        let agent_uuid = Uuid::new_v4();
        let challenge = VerificationChallenge::new(agent_uuid, wallet_address);
        let store = ChallengeStore::new();
        let now = chrono::Utc::now().timestamp();

        // Valid format but random bytes
        let fake_sig = format!("0x{}", "a".repeat(130)); // 65 bytes of 'a'
        
        let result = verify_evm_ownership(
            wallet_address,
            &challenge,
            &fake_sig,
            &store,
            now,
        );
        // Could be MalformedSignature or InvalidSignature depending on parsing
        assert!(matches!(result, 
            VerificationResult::MalformedSignature | 
            VerificationResult::InvalidSignature
        ));
    }

    #[test]
    fn challenge_message_format() {
        let agent_uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let challenge = VerificationChallenge {
            agent_uuid,
            wallet_address: "0xAbCdEf1234567890".to_string(),
            nonce: "test-nonce-123".to_string(),
            issued_at: 1000,
            expires_at: 1900,
            domain: "axi.wallet-verification.v1".to_string(),
        };

        let message = challenge.to_signing_message();
        assert!(message.contains("axi.wallet-verification.v1"));
        assert!(message.contains(&agent_uuid.to_string()));
        assert!(message.contains("0xabcdef1234567890")); // lowercase
        assert!(message.contains("test-nonce-123"));
    }

    #[test]
    fn unsupported_wallet_types_rejected() {
        use super::super::registry::WalletType;
        
        let agent_uuid = Uuid::new_v4();
        let challenge = VerificationChallenge::new(agent_uuid, "0x1234");
        let store = ChallengeStore::new();
        let now = chrono::Utc::now().timestamp();

        // Solana wallets should be rejected until properly implemented
        let result = verify_wallet_ownership(
            WalletType::Solana,
            "0x1234",
            &challenge,
            "0xaaaa",
            &store,
            now,
        );
        assert_eq!(result, VerificationResult::InvalidSignature);

        // BTC wallets should be rejected
        let result = verify_wallet_ownership(
            WalletType::Btc,
            "0x1234",
            &challenge,
            "0xaaaa",
            &store,
            now,
        );
        assert_eq!(result, VerificationResult::InvalidSignature);
    }
}
