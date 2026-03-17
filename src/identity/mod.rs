pub mod registry;
pub mod admission;
pub mod device;
pub mod profile;
pub mod moderation;
pub mod reputation;
pub mod participant;
pub mod wallet_verification;

pub use registry::{AgentIdentity, AgentRegistry, AgentStatus, WalletRef, WalletType, WalletRole};
pub use admission::{AdmissionPipeline, AdmissionRequest, AdmissionState};
pub use device::{DeviceEvidence, DeviceProof, DeviceVerifier};
pub use profile::{PublicProfile, PrivateReviewRecord, ProfileService};
pub use moderation::{ModerationAction, ModerationScope, ModerationStateMachine, ModerationStatus};
pub use reputation::{ReputationDelta, ReputationEvent, ReputationEventType, ReputationService};
pub use participant::{
    ParticipantType, BadgeType, Capabilities,
    ParticipantIdentity, InfraServiceType, InfraProfile, AiAgentProfile,
    AiAdmissionRequirements, InfraAdmissionRequirements,
    AiAutonomyTest, AutonomyChallenge, AutonomyResponse, AutonomyTestResult,
    PermissionChecker, ForumArea, MarketType, AuctionType,
};
pub use wallet_verification::{
    VerificationChallenge, ChallengeStore, VerificationResult,
    verify_evm_ownership, verify_axi_ownership, verify_wallet_ownership,
};

// Internal-only exports for testing
pub(crate) use wallet_verification::pubkey_to_eth_address;
