pub mod registry;
pub mod admission;
pub mod device;
pub mod profile;
pub mod moderation;
pub mod wallet_verification;

pub use registry::{AgentIdentity, AgentRegistry, AgentStatus, WalletRef, WalletType, WalletRole};
pub use admission::{AdmissionPipeline, AdmissionRequest, AdmissionState};
pub use device::{DeviceEvidence, DeviceProof, DeviceVerifier};
pub use profile::{PublicProfile, PrivateReviewRecord, ProfileService};
pub use moderation::{ModerationAction, ModerationScope, ModerationStateMachine, ModerationStatus};
pub use wallet_verification::{generate_challenge, verify_wallet_ownership, VerificationResult, mark_wallet_verified};
