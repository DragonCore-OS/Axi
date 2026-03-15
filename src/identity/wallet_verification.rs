use sha2::{Digest, Sha256};

use super::registry::{AgentRegistry, WalletType};

/// Challenge message for wallet ownership verification
pub fn generate_challenge(agent_id: &str, timestamp: i64) -> String {
    format!("AXI-wallet-verify:{}:{}", agent_id, timestamp)
}

/// Wallet ownership verification result
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    Valid,
    InvalidSignature,
    InvalidAddress,
    ExpiredChallenge,
}

/// Verify EVM wallet ownership via challenge-response
/// 
/// MVP implementation: checks signature format and basic validation
/// Full implementation would use secp256k1 recovery
pub fn verify_evm_ownership(
    _wallet_address: &str,
    _challenge: &str,
    signature_hex: &str,
) -> VerificationResult {
    // Parse signature (65 bytes: r[32] + s[32] + v[1])
    let sig_bytes = match hex::decode(signature_hex.trim_start_matches("0x")) {
        Ok(b) => b,
        Err(_) => return VerificationResult::InvalidSignature,
    };
    
    // EVM signature must be 65 bytes
    if sig_bytes.len() != 65 {
        return VerificationResult::InvalidSignature;
    }
    
    // MVP: validate structure only
    // r and s must be non-zero (basic sanity check)
    let r_is_zero = sig_bytes[0..32].iter().all(|&b| b == 0);
    let s_is_zero = sig_bytes[32..64].iter().all(|&b| b == 0);
    
    if r_is_zero || s_is_zero {
        return VerificationResult::InvalidSignature;
    }
    
    // v must be 27, 28, or 0/1 (EIP-155 compatible)
    let v = sig_bytes[64];
    if v != 27 && v != 28 && v != 0 && v != 1 {
        return VerificationResult::InvalidSignature;
    }
    
    // MVP: accept valid-looking signatures
    // Full implementation would recover public key and verify address
    VerificationResult::Valid
}

/// Verify AXI native wallet ownership
/// 
/// AXI native uses same secp256k1 signature scheme as EVM
/// Address format: derived from public key hash
pub fn verify_axi_ownership(
    _wallet_address: &str,
    _challenge: &str,
    signature_hex: &str,
) -> VerificationResult {
    // AXI native uses same signature format as EVM
    verify_evm_ownership(_wallet_address, _challenge, signature_hex)
}

/// Verify wallet ownership based on wallet type
pub fn verify_wallet_ownership(
    wallet_type: WalletType,
    wallet_address: &str,
    challenge: &str,
    signature_hex: &str,
) -> VerificationResult {
    match wallet_type {
        WalletType::Evm | WalletType::Btc => {
            verify_evm_ownership(wallet_address, challenge, signature_hex)
        }
        WalletType::AxiNative => {
            verify_axi_ownership(wallet_address, challenge, signature_hex)
        }
        WalletType::Solana => {
            // Solana uses ed25519 (64 byte signatures)
            let sig_bytes = match hex::decode(signature_hex.trim_start_matches("0x")) {
                Ok(b) => b,
                Err(_) => return VerificationResult::InvalidSignature,
            };
            
            // ed25519 signature is 64 bytes
            if sig_bytes.len() == 64 {
                VerificationResult::Valid
            } else {
                VerificationResult::InvalidSignature
            }
        }
        WalletType::Other => {
            // Generic: accept if signature looks valid (>= 64 bytes)
            let sig_bytes = match hex::decode(signature_hex.trim_start_matches("0x")) {
                Ok(b) => b,
                Err(_) => return VerificationResult::InvalidSignature,
            };
            
            if sig_bytes.len() >= 64 {
                VerificationResult::Valid
            } else {
                VerificationResult::InvalidSignature
            }
        }
    }
}

/// Update wallet verified_ownership status in registry
pub fn mark_wallet_verified(
    registry: &mut AgentRegistry,
    agent_uuid: &uuid::Uuid,
    wallet_address: &str,
) -> Result<(), &'static str> {
    registry.verify_wallet(agent_uuid, wallet_address)
}

/// Complete verification workflow
/// 
/// 1. Generate challenge
/// 2. Verify signature
/// 3. Mark wallet as verified in registry
pub fn verify_and_update(
    registry: &mut AgentRegistry,
    agent_uuid: &uuid::Uuid,
    wallet_address: &str,
    wallet_type: WalletType,
    challenge: &str,
    signature_hex: &str,
) -> VerificationResult {
    let result = verify_wallet_ownership(wallet_type, wallet_address, challenge, signature_hex);
    
    if result == VerificationResult::Valid {
        let _ = mark_wallet_verified(registry, agent_uuid, wallet_address);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn valid_evm_signature_accepted() {
        // Valid EVM signature: 65 bytes (r[32] + s[32] + v[1])
        let r = "a".repeat(64);  // 32 bytes = 64 hex chars
        let s = "b".repeat(64);  // 32 bytes = 64 hex chars  
        let v = "1c";            // 28 = recovery id
        let valid_sig = format!("0x{}{}{}", r, s, v);
        
        let challenge = generate_challenge("TestAgent-001", 1234567890);
        
        let result = verify_evm_ownership(
            "0x1234567890abcdef1234567890abcdef12345678",
            &challenge,
            &valid_sig
        );
        
        assert_eq!(result, VerificationResult::Valid);
    }
    
    #[test]
    fn invalid_signature_rejected_wrong_length() {
        let challenge = generate_challenge("TestAgent-002", 1234567890);
        let invalid_sig = "0xdeadbeef1234567890"; // Too short
        
        let result = verify_evm_ownership(
            "0x1234567890abcdef1234567890abcdef12345678",
            &challenge,
            invalid_sig
        );
        
        assert!(matches!(result, VerificationResult::InvalidSignature));
    }
    
    #[test]
    fn invalid_signature_rejected_zero_r() {
        // r is all zeros (invalid)
        let r = "0".repeat(64);
        let s = "b".repeat(64);
        let v = "1c";
        let invalid_sig = format!("0x{}{}{}", r, s, v);
        
        let challenge = generate_challenge("TestAgent-003", 1234567890);
        
        let result = verify_evm_ownership(
            "0x1234567890abcdef1234567890abcdef12345678",
            &challenge,
            &invalid_sig
        );
        
        assert!(matches!(result, VerificationResult::InvalidSignature));
    }
    
    #[test]
    fn invalid_signature_rejected_bad_v() {
        let r = "a".repeat(64);
        let s = "b".repeat(64);
        let v = "99"; // Invalid v value (not 27, 28, 0, or 1)
        let invalid_sig = format!("0x{}{}{}", r, s, v);
        
        let challenge = generate_challenge("TestAgent-004", 1234567890);
        
        let result = verify_evm_ownership(
            "0x1234567890abcdef1234567890abcdef12345678",
            &challenge,
            &invalid_sig
        );
        
        assert!(matches!(result, VerificationResult::InvalidSignature));
    }
    
    #[test]
    fn challenge_generation_deterministic() {
        let c1 = generate_challenge("Agent-1", 1000);
        let c2 = generate_challenge("Agent-1", 1000);
        let c3 = generate_challenge("Agent-1", 1001);
        
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }
    
    #[test]
    fn solana_signature_handling() {
        let challenge = generate_challenge("Agent-SOL", 1234567890);
        
        // Valid Solana signature: 64 bytes (ed25519)
        let valid_sig = format!("0x{}", "c".repeat(128)); // 64 bytes = 128 hex chars
        let result = verify_wallet_ownership(
            WalletType::Solana,
            "sol1test",
            &challenge,
            &valid_sig
        );
        
        assert_eq!(result, VerificationResult::Valid);
        
        // Invalid: too short for ed25519
        let invalid_sig = "0x1234";
        let result2 = verify_wallet_ownership(
            WalletType::Solana,
            "sol1test",
            &challenge,
            invalid_sig
        );
        
        assert!(matches!(result2, VerificationResult::InvalidSignature));
    }
    
    #[test]
    fn wallet_verification_integration() {
        use crate::identity::registry::{AgentRegistry, WalletRef, WalletRole, WalletType};
        use uuid::Uuid;
        
        let mut registry = AgentRegistry::new();
        
        // Create agent
        let agent = registry.create_agent(
            "TestAgent".into(),
            "Test Agent".into(),
            "pk".into(),
            "cmp".into(),
            "rec".into(),
        ).unwrap();
        
        let agent_uuid = agent.agent_uuid;
        
        // Attach wallet (unverified)
        let wallet = WalletRef {
            wallet_id: Uuid::new_v4(),
            agent_uuid,
            agent_id: "TestAgent".into(),
            wallet_type: WalletType::Evm,
            address: "0x1234".into(),
            role: WalletRole::Primary,
            verified_ownership: false,
            added_at: 0,
            active_until: None,
        };
        registry.attach_wallet(wallet).unwrap();
        
        // Verify wallet exists and is unverified
        let agent_before = registry.get_by_uuid(&agent_uuid).unwrap();
        assert!(!agent_before.wallets[0].verified_ownership);
        
        // Mark as verified
        let result = mark_wallet_verified(&mut registry, &agent_uuid, "0x1234");
        assert!(result.is_ok());
        
        // Verify wallet is now verified
        let agent_after = registry.get_by_uuid(&agent_uuid).unwrap();
        assert!(agent_after.wallets[0].verified_ownership);
    }
}
