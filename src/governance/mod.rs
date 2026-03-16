//! DragonCore Internal Broadcast Layer (DIBL)
//!
//! 基于 AXI 的"双层可见性 + 内部风险广播"原则，
//! 适配 DragonCore 单节点治理运行时的内部事件层。
//!
//! 核心原则：
//! 1. 先持久化，后广播（JSON/ledger 是真相源）
//! 2. 广播层是派生层，不是主控层
//! 3. 内部事件默认 internal，显式降级才可见

pub mod event;
pub mod broadcast;
pub mod store;
pub mod projection;

pub use event::{
    EventScope, EventChannel, GovernanceEventType, Severity,
    GovernanceEvent, CorrelationContext,
};
pub use broadcast::{Broadcaster, EventSink, DiblBroadcaster};
pub use store::{JsonlEventStore, EventStore};
pub use projection::{RunProjection, OperatorView};
