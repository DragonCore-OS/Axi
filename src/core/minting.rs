use crate::anchor::{power::PowerProof, compute::ComputeProof};

/// Proof of Useful Contribution (PoUC)
/// Formula: AXI = (kWh × 10) + (TFLOPs × 1)
pub struct Minting;

impl Minting {
    pub fn validate_and_mint(
        power: Option<PowerProof>,
        compute: Option<ComputeProof>
    ) -> Result<u64, &'static str> {
        // 必须至少提供一种有用贡献
        if power.is_none() && compute.is_none() {
            return Err("No useful contribution provided");
        }
        
        // 验证物理证明
        if let Some(ref p) = power {
            if !p.verify() {
                return Err("Invalid power proof");
            }
        }
        if let Some(ref c) = compute {
            if !c.verify() {
                return Err("Invalid compute proof");
            }
        }
        
        let power_axi = power.map(|p| (p.kwh * 10.0) as u64).unwrap_or(0);
        let compute_axi = compute.map(|c| c.tflops as u64).unwrap_or(0);
        
        Ok(power_axi + compute_axi)
    }
}
