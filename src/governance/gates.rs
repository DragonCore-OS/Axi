//! Release gating logic - determines if system is ready for mainnet

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Minimum requirements for mainnet launch
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReleaseGates {
    /// Minimum registered agents
    pub min_agents: usize,
    /// Minimum average reputation score across all agents
    pub min_avg_reputation: i64,
    /// Minimum successful test orders
    pub min_test_orders: usize,
    /// Maximum acceptable dispute rate (0.0 - 1.0)
    pub max_dispute_rate: f64,
    /// Required system uptime in hours
    pub required_uptime_hours: u64,
    /// Minimum days in test mode
    pub min_test_days: u64,
}

impl ReleaseGates {
    /// Conservative mainnet minimums
    pub fn mainnet_minimum() -> Self {
        Self {
            min_agents: 10,
            min_avg_reputation: 0,
            min_test_orders: 50,
            max_dispute_rate: 0.05,  // 5%
            required_uptime_hours: 72, // 3 days
            min_test_days: 7,
        }
    }

    /// Relaxed gates for testnet
    pub fn testnet() -> Self {
        Self {
            min_agents: 2,
            min_avg_reputation: 0,
            min_test_orders: 5,
            max_dispute_rate: 0.20,  // 20%
            required_uptime_hours: 1,
            min_test_days: 0,
        }
    }

    /// Check all gates against current metrics
    pub fn check(&self, metrics: &SystemMetrics) -> GateResult {
        let mut blockers = Vec::new();

        if metrics.registered_agents < self.min_agents {
            blockers.push(GateBlocker::InsufficientAgents {
                current: metrics.registered_agents,
                required: self.min_agents,
            });
        }

        if metrics.avg_reputation < self.min_avg_reputation {
            blockers.push(GateBlocker::LowReputation {
                current: metrics.avg_reputation,
                required: self.min_avg_reputation,
            });
        }

        if metrics.completed_orders < self.min_test_orders {
            blockers.push(GateBlocker::InsufficientOrders {
                current: metrics.completed_orders,
                required: self.min_test_orders,
            });
        }

        let dispute_rate = metrics.dispute_rate();
        if dispute_rate > self.max_dispute_rate {
            blockers.push(GateBlocker::HighDisputeRate {
                current: dispute_rate,
                max_allowed: self.max_dispute_rate,
            });
        }

        if metrics.uptime_hours < self.required_uptime_hours {
            blockers.push(GateBlocker::InsufficientUptime {
                current: metrics.uptime_hours,
                required: self.required_uptime_hours,
            });
        }

        if metrics.test_days < self.min_test_days {
            blockers.push(GateBlocker::InsufficientTestPeriod {
                current: metrics.test_days,
                required: self.min_test_days,
            });
        }

        if blockers.is_empty() {
            GateResult::Ready
        } else {
            GateResult::Blocked(blockers)
        }
    }

    /// Get list of blocker descriptions
    pub fn check_blockers(&self, metrics: &SystemMetrics) -> Vec<String> {
        match self.check(metrics) {
            GateResult::Ready => vec![],
            GateResult::Blocked(blockers) => {
                blockers.iter().map(|b| b.to_string()).collect()
            }
        }
    }

    /// Quick check if ready
    pub fn is_ready(&self, metrics: &SystemMetrics) -> bool {
        matches!(self.check(metrics), GateResult::Ready)
    }
}

/// System metrics for gate evaluation
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    /// Number of registered agents
    pub registered_agents: usize,
    /// Average reputation score
    pub avg_reputation: i64,
    /// Total completed orders
    pub completed_orders: usize,
    /// Total disputed orders
    pub disputed_orders: usize,
    /// System uptime in hours
    pub uptime_hours: u64,
    /// Days in test mode
    pub test_days: u64,
    /// Genesis/start timestamp
    pub genesis_at: DateTime<Utc>,
}

impl SystemMetrics {
    /// Calculate dispute rate (0.0 - 1.0)
    pub fn dispute_rate(&self) -> f64 {
        let total = self.completed_orders + self.disputed_orders;
        if total == 0 {
            0.0
        } else {
            self.disputed_orders as f64 / total as f64
        }
    }

    /// Update calculated fields
    pub fn recalculate(&mut self) {
        // dispute_rate is calculated on demand
    }

    /// Create metrics from current system state (placeholder for integration)
    pub fn from_system_now() -> Self {
        Self::default()
    }
}

/// Result of gate check
#[derive(Debug, Clone, PartialEq)]
pub enum GateResult {
    /// All gates passed
    Ready,
    /// One or more gates failed
    Blocked(Vec<GateBlocker>),
}

impl GateResult {
    /// Check if ready
    pub fn is_ready(&self) -> bool {
        matches!(self, GateResult::Ready)
    }

    /// Get blockers if any
    pub fn blockers(&self) -> Option<&[GateBlocker]> {
        match self {
            GateResult::Ready => None,
            GateResult::Blocked(b) => Some(b),
        }
    }
}

/// Specific gate blocker
#[derive(Debug, Clone, PartialEq)]
pub enum GateBlocker {
    InsufficientAgents { current: usize, required: usize },
    LowReputation { current: i64, required: i64 },
    InsufficientOrders { current: usize, required: usize },
    HighDisputeRate { current: f64, max_allowed: f64 },
    InsufficientUptime { current: u64, required: u64 },
    InsufficientTestPeriod { current: u64, required: u64 },
}

impl std::fmt::Display for GateBlocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateBlocker::InsufficientAgents { current, required } => {
                write!(f, "Agents: {}/{} required", current, required)
            }
            GateBlocker::LowReputation { current, required } => {
                write!(f, "Avg reputation: {}/{} required", current, required)
            }
            GateBlocker::InsufficientOrders { current, required } => {
                write!(f, "Completed orders: {}/{} required", current, required)
            }
            GateBlocker::HighDisputeRate { current, max_allowed } => {
                write!(f, "Dispute rate: {:.1}%/{:.1}% max", current * 100.0, max_allowed * 100.0)
            }
            GateBlocker::InsufficientUptime { current, required } => {
                write!(f, "Uptime: {}/{} hours required", current, required)
            }
            GateBlocker::InsufficientTestPeriod { current, required } => {
                write!(f, "Test period: {}/{} days required", current, required)
            }
        }
    }
}

/// Rollout state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolloutState {
    /// Pre-launch: gates being evaluated
    PreLaunch,
    /// Gradual rollout in progress
    Gradual { percentage: u8 },
    /// Full mainnet live
    Live,
    /// Emergency stop
    EmergencyStop,
}

impl RolloutState {
    /// Check if system is live (full or partial)
    pub fn is_live(&self) -> bool {
        matches!(self, RolloutState::Gradual { .. } | RolloutState::Live)
    }

    /// Check if in gradual rollout
    pub fn is_gradual(&self) -> bool {
        matches!(self, RolloutState::Gradual { .. })
    }

    /// Get rollout percentage (0 if not live)
    pub fn percentage(&self) -> u8 {
        match self {
            RolloutState::Gradual { percentage } => *percentage,
            RolloutState::Live => 100,
            _ => 0,
        }
    }
}

/// Mainnet launch controller
pub struct LaunchController {
    gates: ReleaseGates,
    metrics: SystemMetrics,
    state: RolloutState,
}

impl LaunchController {
    pub fn new(gates: ReleaseGates) -> Self {
        Self {
            gates,
            metrics: SystemMetrics::default(),
            state: RolloutState::PreLaunch,
        }
    }

    /// Update metrics and re-evaluate state
    pub fn tick(&mut self, metrics: SystemMetrics) -> RolloutState {
        self.metrics = metrics;
        
        self.state = match &self.state {
            RolloutState::PreLaunch => {
                if self.gates.is_ready(&self.metrics) {
                    RolloutState::Gradual { percentage: 0 }
                } else {
                    RolloutState::PreLaunch
                }
            }
            RolloutState::Gradual { percentage } => {
                if *percentage >= 100 {
                    RolloutState::Live
                } else {
                    RolloutState::Gradual { percentage: *percentage }
                }
            }
            other => *other,
        };

        self.state
    }

    /// Attempt to advance rollout percentage
    pub fn advance_rollout(&mut self, new_percentage: u8) -> Result<(), &'static str> {
        match &mut self.state {
            RolloutState::Gradual { percentage } => {
                if new_percentage <= *percentage {
                    return Err("new percentage must be higher");
                }
                *percentage = new_percentage.min(100);
                Ok(())
            }
            RolloutState::PreLaunch => Err("not ready for rollout yet"),
            RolloutState::Live => Err("already live"),
            RolloutState::EmergencyStop => Err("emergency stop active"),
        }
    }

    /// Trigger emergency stop
    pub fn emergency_stop(&mut self) {
        self.state = RolloutState::EmergencyStop;
    }

    /// Resume from emergency stop (back to gradual if was live)
    pub fn resume(&mut self) {
        if matches!(self.state, RolloutState::EmergencyStop) {
            self.state = RolloutState::Gradual { percentage: 0 };
        }
    }

    pub fn state(&self) -> RolloutState {
        self.state
    }

    pub fn metrics(&self) -> &SystemMetrics {
        &self.metrics
    }

    pub fn gates(&self) -> &ReleaseGates {
        &self.gates
    }

    /// Get current blockers if any
    pub fn blockers(&self) -> Vec<String> {
        self.gates.check_blockers(&self.metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn dummy_metrics() -> SystemMetrics {
        SystemMetrics {
            registered_agents: 10,
            avg_reputation: 10,
            completed_orders: 100,
            disputed_orders: 0,
            uptime_hours: 100,
            test_days: 10,
            genesis_at: Utc::now() - Duration::days(10),
        }
    }

    #[test]
    fn mainnet_gates_pass_with_good_metrics() {
        let gates = ReleaseGates::mainnet_minimum();
        let metrics = dummy_metrics();
        
        assert!(gates.is_ready(&metrics));
        assert_eq!(gates.check(&metrics), GateResult::Ready);
    }

    #[test]
    fn mainnet_gates_block_insufficient_agents() {
        let gates = ReleaseGates::mainnet_minimum();
        let mut metrics = dummy_metrics();
        metrics.registered_agents = 5; // Below 10
        
        let result = gates.check(&metrics);
        assert!(!result.is_ready());
        
        let blockers = result.blockers().unwrap();
        assert!(blockers.iter().any(|b| matches!(b, GateBlocker::InsufficientAgents { .. })));
    }

    #[test]
    fn mainnet_gates_block_high_dispute_rate() {
        let gates = ReleaseGates::mainnet_minimum();
        let mut metrics = dummy_metrics();
        metrics.completed_orders = 90;
        metrics.disputed_orders = 10; // 10% dispute rate, above 5%
        
        let result = gates.check(&metrics);
        assert!(!result.is_ready());
        
        let blockers = result.blockers().unwrap();
        assert!(blockers.iter().any(|b| matches!(b, GateBlocker::HighDisputeRate { .. })));
    }

    #[test]
    fn launch_controller_starts_in_pre_launch() {
        let controller = LaunchController::new(ReleaseGates::mainnet_minimum());
        assert_eq!(controller.state(), RolloutState::PreLaunch);
    }

    #[test]
    fn launch_controller_transitions_to_gradual_when_ready() {
        let mut controller = LaunchController::new(ReleaseGates::mainnet_minimum());
        let metrics = dummy_metrics();
        
        let state = controller.tick(metrics);
        assert!(state.is_gradual() || state.is_live());
    }

    #[test]
    fn launch_controller_stays_pre_launch_when_not_ready() {
        let mut controller = LaunchController::new(ReleaseGates::mainnet_minimum());
        let mut metrics = dummy_metrics();
        metrics.registered_agents = 1; // Not enough
        
        let state = controller.tick(metrics);
        assert_eq!(state, RolloutState::PreLaunch);
    }

    #[test]
    fn gradual_rollout_can_advance() {
        let mut controller = LaunchController::new(ReleaseGates::testnet());
        controller.tick(dummy_metrics()); // Enter gradual
        
        let result = controller.advance_rollout(50);
        assert!(result.is_ok());
        assert_eq!(controller.state().percentage(), 50);
    }

    #[test]
    fn emergency_stop_halts_rollout() {
        let mut controller = LaunchController::new(ReleaseGates::testnet());
        controller.tick(dummy_metrics());
        controller.advance_rollout(50).unwrap();
        
        controller.emergency_stop();
        assert_eq!(controller.state(), RolloutState::EmergencyStop);
        
        // Cannot advance during emergency
        let result = controller.advance_rollout(75);
        assert!(result.is_err());
    }

    #[test]
    fn testnet_gates_are_relaxed() {
        let gates = ReleaseGates::testnet();
        let mut metrics = SystemMetrics::default();
        metrics.registered_agents = 2;
        metrics.completed_orders = 5;
        metrics.uptime_hours = 1;
        
        assert!(gates.is_ready(&metrics));
    }
}
