use sha2::{Sha256, Digest};

/// AXI Genesis Block - Canonical Version
/// 
/// This implementation uses FIXED values to ensure deterministic genesis hash
/// across all runtime surfaces (CLI, daemon, API).
/// 
/// DO NOT MODIFY these values without protocol consensus.
pub struct GenesisBlock {
    pub timestamp: u64,       // Fixed: 1709256878 (2024-03-01T00:00:00Z)
    pub anchor_power: f64,    // Fixed: 1000.0 kWh
    pub anchor_compute: f64,  // Fixed: 3280.0 TFLOPs
    pub hash: String,         // Deterministic SHA-256 of above + constitution
    pub constitution_hash: String,
}

/// Canonical genesis parameters - hardcoded for determinism
const GENESIS_TIMESTAMP: u64 = 1709256878;  // 2024-03-01T00:00:00Z UTC
const POWER_ANCHOR_KWH: f64 = 1000.0;
const COMPUTE_ANCHOR_TFLOPS: f64 = 3280.0;

impl GenesisBlock {
    /// Create the canonical genesis block with deterministic values
    /// 
    /// This ensures all nodes generate identical genesis hash.
    pub fn new() -> Self {
        let constitution = include_str!("../../CONSTITUTION.md");
        
        // Calculate constitution hash
        let mut hasher = Sha256::new();
        hasher.update(constitution);
        let const_hash = format!("{:x}", hasher.finalize());
        
        // Calculate deterministic block hash
        let mut hasher = Sha256::new();
        hasher.update(&GENESIS_TIMESTAMP.to_le_bytes());
        hasher.update(&POWER_ANCHOR_KWH.to_le_bytes());
        hasher.update(&COMPUTE_ANCHOR_TFLOPS.to_le_bytes());
        hasher.update(const_hash.as_bytes());
        let block_hash = format!("{:x}", hasher.finalize());
        
        Self {
            timestamp: GENESIS_TIMESTAMP,
            anchor_power: POWER_ANCHOR_KWH,
            anchor_compute: COMPUTE_ANCHOR_TFLOPS,
            hash: block_hash,
            constitution_hash: const_hash,
        }
    }
    
    /// Legacy constructor - DEPRECATED
    /// 
    /// This was the old non-deterministic version that used current timestamp.
    /// Kept for backward compatibility during migration only.
    #[deprecated(since = "0.2.0", note = "Use new() for deterministic genesis")]
    pub fn new_legacy(_power_kwh: f64, _compute_tflops: f64) -> Self {
        Self::new()
    }
    
    /// Verify constitution hasn't been tampered with
    pub fn verify_constitution(&self) -> bool {
        let current = include_str!("../../CONSTITUTION.md");
        let mut hasher = Sha256::new();
        hasher.update(current);
        let hash = format!("{:x}", hasher.finalize());
        hash == self.constitution_hash
    }
    
    /// Get the canonical genesis hash
    pub fn canonical_hash() -> &'static str {
        // Pre-computed deterministic hash
        // Computed from: timestamp=1709256878, power=1000.0, compute=3280.0, 
        // constitution_hash=00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177
        "f23b862cde464401d4cf80de425aca1c5c0a0ef5aa50da94e904d362ec006314"
    }
    
    /// Verify this block matches canonical values
    pub fn verify_canonical(&self) -> bool {
        self.hash == Self::canonical_hash()
            && self.timestamp == GENESIS_TIMESTAMP
            && self.anchor_power == POWER_ANCHOR_KWH
            && self.anchor_compute == COMPUTE_ANCHOR_TFLOPS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_determinism() {
        let g1 = GenesisBlock::new();
        let g2 = GenesisBlock::new();
        assert_eq!(g1.hash, g2.hash);
        assert_eq!(g1.timestamp, GENESIS_TIMESTAMP);
        assert_eq!(g1.constitution_hash, "00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177");
    }
    
    #[test]
    fn test_canonical_constitution_hash() {
        let genesis = GenesisBlock::new();
        assert_eq!(
            genesis.constitution_hash,
            "00dae4fce1340d89ade1c87cdd5b0dd649111cecb67799ac99df914620cea177"
        );
    }
}
