//! 电力证明：每0.1kWh清洁电力 = 1 AXI

pub struct PowerProof {
    pub kwh: f64,
    pub source: EnergySource,
    pub meter_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum EnergySource {
    Solar,
    Wind,
    Nuclear,
    Grid, // 混合电网，但需验证清洁比例
}

impl PowerProof {
    pub fn new(kwh: f64, source: EnergySource, meter_id: &str) -> Self {
        Self {
            kwh,
            source,
            meter_id: meter_id.to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
    
    /// 验证电力证明（实际应连接智能电表TEE）
    pub fn verify(&self) -> bool {
        // 简化：检查正值和合理上限（家用光伏日发电<100kWh）
        self.kwh > 0.0 && self.kwh < 1000.0
    }
    
    pub fn to_axi(&self) -> u64 {
        (self.kwh * 10.0) as u64 // 0.1kWh = 1 AXI
    }
}
