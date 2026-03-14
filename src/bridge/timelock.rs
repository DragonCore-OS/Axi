//! 2027年独立日：法币桥接自动熔断

pub const INDEPENDENCE_DAY: u64 = 1798761600; // 2027-01-01 00:00:00 UTC

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BridgeState {
    DualTrack,  // 2026年：允许法币兑换
    Sovereign,  // 2027年后：纯物理锚定
}

pub struct TimeLock;

impl TimeLock {
    pub fn check(current_timestamp: u64) -> BridgeState {
        if current_timestamp >= INDEPENDENCE_DAY {
            BridgeState::Sovereign
        } else {
            BridgeState::DualTrack
        }
    }
    
    pub fn is_fiat_allowed(current_timestamp: u64) -> bool {
        Self::check(current_timestamp) == BridgeState::DualTrack
    }
    
    pub fn days_until_independence(current_timestamp: u64) -> i64 {
        let seconds = INDEPENDENCE_DAY as i64 - current_timestamp as i64;
        seconds / 86400
    }
}
