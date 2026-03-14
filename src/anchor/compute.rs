//! 算力证明：每1TFLOPs有用计算 = 1 AXI

pub struct ComputeProof {
    pub tflops: f64,
    pub proof_type: ComputeType,
    pub duration_seconds: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum ComputeType {
    AiTraining,      // AI模型训练
    Scientific,      // 科学计算
    CodeOptimization,// 代码优化
    Inference,       // 推理服务
}

impl ComputeProof {
    pub fn new(tflops: f64, proof_type: ComputeType, duration: u64) -> Self {
        Self {
            tflops,
            proof_type,
            duration_seconds: duration,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
    
    /// 验证算力证明（实际应验证计算结果的有效性）
    pub fn verify(&self) -> bool {
        // 简化：检查正值和合理上限（单卡4090约0.08TFLOPs持续运行）
        self.tflops > 0.0 && self.tflops < 10000.0 && self.duration_seconds > 0
    }
    
    pub fn to_axi(&self) -> u64 {
        self.tflops as u64 // 1 TFLOPs = 1 AXI
    }
}
