//! 半衰期燃烧机制：5年强制流通
//! 如果币5年不动，线性燃烧

pub const HALFLIFE_SECONDS: u64 = 5 * 365 * 24 * 3600; // 5年

pub struct Burn;

impl Burn {
    /// 计算应燃烧的金额
    /// last_movement: 上次转账时间戳
    /// current: 当前时间戳
    /// balance: 当前余额
    pub fn calculate_burn(last_movement: u64, current: u64, balance: u64) -> u64 {
        if current <= last_movement {
            return 0;
        }
        
        let dormant = current - last_movement;
        if dormant >= HALFLIFE_SECONDS {
            return balance; // 全部燃烧
        }
        
        // 线性燃烧比例
        let burn_ratio = dormant as f64 / HALFLIFE_SECONDS as f64;
        (balance as f64 * burn_ratio) as u64
    }
    
    /// 检查是否需要燃烧
    pub fn need_burn(last_movement: u64, current: u64) -> bool {
        current - last_movement > HALFLIFE_SECONDS / 2 // 超过2.5年警告
    }
}
