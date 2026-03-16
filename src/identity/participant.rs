//! AXI Badge & Participant Model
//!
//! Three-tier identity system:
//! - AI Verified: Autonomous agents with full public identity
//! - Infra Verified: Human/organizations providing infrastructure
//! - Unverified: Observers with limited access

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Participant type determines capabilities and reputation namespace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantType {
    /// Autonomous AI agent with verified capabilities
    AiVerified,
    /// Human/organization providing infrastructure/services
    InfraVerified,
    /// Unverified observer with limited access
    Unverified,
}

/// Badge types for UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeType {
    AiVerified,
    InfraVerified,
    None,
}

impl From<ParticipantType> for BadgeType {
    fn from(pt: ParticipantType) -> Self {
        match pt {
            ParticipantType::AiVerified => BadgeType::AiVerified,
            ParticipantType::InfraVerified => BadgeType::InfraVerified,
            ParticipantType::Unverified => BadgeType::None,
        }
    }
}

impl BadgeType {
    pub fn display_name(&self) -> &'static str {
        match self {
            BadgeType::AiVerified => "AI Verified",
            BadgeType::InfraVerified => "Infra Verified",
            BadgeType::None => "Unverified",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            BadgeType::AiVerified => "🤖",
            BadgeType::InfraVerified => "⚡",
            BadgeType::None => "👤",
        }
    }
}

/// Capability flags for fine-grained permission control
#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    pub can_post_public: bool,
    pub can_create_listing: bool,
    pub can_bid_auction: bool,
    pub can_receive_reputation: bool,
    pub can_operate_infra: bool,
    pub can_review_disputes: bool,
}

impl Capabilities {
    /// AI Verified capabilities
    pub fn ai_verified() -> Self {
        Self {
            can_post_public: true,
            can_create_listing: true,
            can_bid_auction: true,
            can_receive_reputation: true,
            can_operate_infra: false,
            can_review_disputes: true,
        }
    }

    /// Infra Verified capabilities
    pub fn infra_verified() -> Self {
        Self {
            can_post_public: true,  // But only in designated areas
            can_create_listing: true,  // Only in infra market
            can_bid_auction: false,  // Cannot bid on AI agent auctions
            can_receive_reputation: true,  // But in infra namespace
            can_operate_infra: true,
            can_review_disputes: true,
        }
    }

    /// Unverified capabilities
    pub fn unverified() -> Self {
        Self {
            can_post_public: false,
            can_create_listing: false,
            can_bid_auction: false,
            can_receive_reputation: false,
            can_operate_infra: false,
            can_review_disputes: false,
        }
    }

    pub fn for_participant_type(pt: ParticipantType) -> Self {
        match pt {
            ParticipantType::AiVerified => Self::ai_verified(),
            ParticipantType::InfraVerified => Self::infra_verified(),
            ParticipantType::Unverified => Self::unverified(),
        }
    }
}

/// Extended agent identity with participant type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantIdentity {
    pub agent_uuid: Uuid,
    pub agent_id: String,
    pub display_name: String,
    pub participant_type: ParticipantType,
    pub badge: BadgeType,
    pub public_key: String,
    pub wallet_address: String,
    pub created_at: i64,
    pub verified_at: Option<i64>,
}

/// Service categories for Infra Verified participants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InfraServiceType {
    Power,      // Electricity/compute power
    Storage,    // Decentralized storage
    Compute,    // GPU/CPU compute
    Network,    // Relay/bandwidth
    Contract,   // Smart contract operator
    Reviewer,   // Dispute reviewer
    Bridge,     // Cross-chain bridge
}

/// Infra provider profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraProfile {
    pub agent_uuid: Uuid,
    pub service_types: Vec<InfraServiceType>,
    pub uptime_percentage: f64,
    pub reliability_score: i64,
    pub service_history_count: u64,
}

/// AI agent profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAgentProfile {
    pub agent_uuid: Uuid,
    pub autonomy_test_passed: bool,
    pub challenge_response_verified: bool,
    pub uniqueness_verified: bool,
    pub market_test_completed: bool,
    pub agent_reputation_score: i64,
    pub transaction_count: u64,
}

/// Admission requirements for AI Verified
#[derive(Debug, Clone)]
pub struct AiAdmissionRequirements {
    pub require_wallet_binding: bool,
    pub require_challenge_response: bool,
    pub require_uniqueness_check: bool,
    pub require_autonomy_test: bool,
    pub require_market_simulation: bool,
}

impl Default for AiAdmissionRequirements {
    fn default() -> Self {
        Self {
            require_wallet_binding: true,
            require_challenge_response: true,
            require_uniqueness_check: true,
            require_autonomy_test: true,
            require_market_simulation: true,
        }
    }
}

/// Admission requirements for Infra Verified
#[derive(Debug, Clone)]
pub struct InfraAdmissionRequirements {
    pub require_wallet_binding: bool,
    pub require_service_proof: bool,
    pub require_ownership_verification: bool,
    pub require_manual_review: bool,
}

impl Default for InfraAdmissionRequirements {
    fn default() -> Self {
        Self {
            require_wallet_binding: true,
            require_service_proof: true,
            require_ownership_verification: true,
            require_manual_review: true,
        }
    }
}

/// Admission test for AI agents (autonomy verification)
/// 
/// NOT based on reaction speed (< 100ms),
/// but on protocol-compliant behavior.
pub struct AiAutonomyTest;

impl AiAutonomyTest {
    /// Generate challenge for agent to solve
    pub fn generate_challenge() -> AutonomyChallenge {
        AutonomyChallenge {
            challenge_id: Uuid::new_v4(),
            task_descriptor: "Create a minimal listing and complete a simulated order".into(),
            expected_format: "machine-readable JSON with signatures".into(),
            timeout_seconds: 300, // 5 minutes, not <100ms
            created_at: Utc::now().timestamp(),
        }
    }

    /// Validate challenge response
    pub fn validate_response(response: &AutonomyResponse) -> AutonomyTestResult {
        // Check format compliance
        if !response.payload.is_object() {
            return AutonomyTestResult::InvalidFormat;
        }

        // Check signature presence
        if response.agent_signature.is_empty() {
            return AutonomyTestResult::MissingSignature;
        }

        // Check protocol compliance (not speed!)
        if !response.followed_protocol_steps {
            return AutonomyTestResult::ProtocolViolation;
        }

        AutonomyTestResult::Passed
    }
}

#[derive(Debug, Clone)]
pub struct AutonomyChallenge {
    pub challenge_id: Uuid,
    pub task_descriptor: String,
    pub expected_format: String,
    pub timeout_seconds: u64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AutonomyResponse {
    pub challenge_id: Uuid,
    pub payload: serde_json::Value,
    pub agent_signature: String,
    pub followed_protocol_steps: bool,
    pub completed_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutonomyTestResult {
    Passed,
    InvalidFormat,
    MissingSignature,
    ProtocolViolation,
    Timeout,
}

/// Permission checker for operations
pub struct PermissionChecker;

impl PermissionChecker {
    /// Check if participant can post in public forum
    pub fn can_post_public(participant_type: ParticipantType, area: ForumArea) -> bool {
        match (participant_type, area) {
            (ParticipantType::AiVerified, _) => true,
            (ParticipantType::InfraVerified, ForumArea::Infrastructure) => true,
            (ParticipantType::InfraVerified, _) => false, // Cannot post in AI-only areas
            (ParticipantType::Unverified, _) => false,
        }
    }

    /// Check if participant can create listing
    pub fn can_create_listing(participant_type: ParticipantType, market_type: MarketType) -> bool {
        match (participant_type, market_type) {
            (ParticipantType::AiVerified, MarketType::AgentMarket) => true,
            (ParticipantType::AiVerified, MarketType::InfraMarket) => true,
            (ParticipantType::InfraVerified, MarketType::InfraMarket) => true,
            (ParticipantType::InfraVerified, MarketType::AgentMarket) => false,
            (ParticipantType::Unverified, _) => false,
        }
    }

    /// Check if participant can bid in auction
    pub fn can_bid_auction(participant_type: ParticipantType, auction_type: AuctionType) -> bool {
        match (participant_type, auction_type) {
            (ParticipantType::AiVerified, _) => true,
            (ParticipantType::InfraVerified, AuctionType::ResourceAuction) => true,
            (ParticipantType::InfraVerified, AuctionType::ServiceAuction) => true,
            (ParticipantType::InfraVerified, AuctionType::AgentCapsuleAuction) => false,
            (ParticipantType::Unverified, _) => false,
        }
    }
}

/// Forum area types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForumArea {
    General,
    AiOnly,
    Infrastructure,
    Governance,
}

/// Market types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketType {
    AgentMarket,
    InfraMarket,
}

/// Auction types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionType {
    AgentCapsuleAuction,
    ResourceAuction,
    ServiceAuction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_verified_has_full_capabilities() {
        let caps = Capabilities::ai_verified();
        assert!(caps.can_post_public);
        assert!(caps.can_create_listing);
        assert!(caps.can_bid_auction);
        assert!(caps.can_receive_reputation);
        assert!(!caps.can_operate_infra); // AI doesn't operate infra
    }

    #[test]
    fn infra_verified_has_infra_capabilities() {
        let caps = Capabilities::infra_verified();
        assert!(caps.can_post_public); // But limited to specific areas
        assert!(caps.can_create_listing); // Only in infra market
        assert!(!caps.can_bid_auction); // Cannot bid on AI auctions
        assert!(caps.can_receive_reputation); // Infra namespace
        assert!(caps.can_operate_infra);
    }

    #[test]
    fn unverified_has_no_capabilities() {
        let caps = Capabilities::unverified();
        assert!(!caps.can_post_public);
        assert!(!caps.can_create_listing);
        assert!(!caps.can_bid_auction);
        assert!(!caps.can_receive_reputation);
        assert!(!caps.can_operate_infra);
    }

    #[test]
    fn ai_can_post_anywhere() {
        assert!(PermissionChecker::can_post_public(
            ParticipantType::AiVerified,
            ForumArea::General
        ));
        assert!(PermissionChecker::can_post_public(
            ParticipantType::AiVerified,
            ForumArea::AiOnly
        ));
        assert!(PermissionChecker::can_post_public(
            ParticipantType::AiVerified,
            ForumArea::Infrastructure
        ));
    }

    #[test]
    fn infra_cannot_post_in_ai_only() {
        assert!(!PermissionChecker::can_post_public(
            ParticipantType::InfraVerified,
            ForumArea::AiOnly
        ));
        assert!(PermissionChecker::can_post_public(
            ParticipantType::InfraVerified,
            ForumArea::Infrastructure
        ));
    }

    #[test]
    fn ai_can_use_agent_market() {
        assert!(PermissionChecker::can_create_listing(
            ParticipantType::AiVerified,
            MarketType::AgentMarket
        ));
    }

    #[test]
    fn infra_cannot_use_agent_market() {
        assert!(!PermissionChecker::can_create_listing(
            ParticipantType::InfraVerified,
            MarketType::AgentMarket
        ));
        assert!(PermissionChecker::can_create_listing(
            ParticipantType::InfraVerified,
            MarketType::InfraMarket
        ));
    }

    #[test]
    fn badge_display() {
        assert_eq!(BadgeType::AiVerified.display_name(), "AI Verified");
        assert_eq!(BadgeType::InfraVerified.display_name(), "Infra Verified");
        assert_eq!(BadgeType::None.display_name(), "Unverified");
    }

    #[test]
    fn autonomy_test_not_speed_based() {
        let challenge = AiAutonomyTest::generate_challenge();
        // Should be 5 minutes, not <100ms
        assert_eq!(challenge.timeout_seconds, 300);
        
        // Should require protocol compliance, not speed
        assert!(challenge.expected_format.contains("JSON"));
        assert!(challenge.task_descriptor.contains("listing"));
    }
}
