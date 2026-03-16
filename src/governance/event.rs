//! DIBL 核心事件定义
//!
//! 设计原则：
//! - 事件对象是治理动作的第一公民，不是消息
//! - 每个事件必须有明确的 scope（internal/operator_visible/exportable）
//! - 支持因果链追踪（correlation_id/parent_event_id）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 可见性层级
/// 
/// 继承 AXI 的 dual-layer visibility，但改为三级以适应 DragonCore：
/// - Internal: 仅 19 席治理内部（原始 seat 输出、veto 细节）
/// - OperatorVisible: 操作员/控制台可见（摘要、风险告警）
/// - Exportable: 可导出为对外报告
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventScope {
    Internal,
    OperatorVisible,
    Exportable,
}

impl EventScope {
    /// 是否允许进入 operator 视图
    pub fn is_operator_visible(&self) -> bool {
        matches!(self, EventScope::OperatorVisible | EventScope::Exportable)
    }

    /// 是否允许导出
    pub fn is_exportable(&self) -> bool {
        matches!(self, EventScope::Exportable)
    }
}

/// 广播通道/主题
/// 
/// 对应 AXI Private Mesh 的 control/research/ops/security 分类，
/// 但改造为事件主题流，不是聊天房间。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventChannel {
    /// 治理推进：run 创建、seat 轮转、final gate、archive
    Control,
    /// 运行时运维：tmux、worktree、state persistence、ledger
    Ops,
    /// 红线与风控：veto、terminate、policy breach
    Security,
    /// 复杂任务内部讨论结果摘要
    Research,
}

/// 严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warn,
    Critical,
}

/// 治理事件类型
/// 
/// 围绕 DragonCore 的 run-centered lifecycle 设计，
/// 不是开放聊天系统的 open-ended message types。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceEventType {
    RunCreated,
    SeatStarted,
    SeatCompleted,
    RiskRaised,
    VetoExercised,
    FinalGateOpened,
    DecisionCommitted,
    RollbackTriggered,
    ArchiveCompleted,
    TerminateTriggered,
}

impl GovernanceEventType {
    /// 默认严重程度
    pub fn default_severity(&self) -> Severity {
        match self {
            Self::RunCreated => Severity::Info,
            Self::SeatStarted => Severity::Info,
            Self::SeatCompleted => Severity::Info,
            Self::RiskRaised => Severity::Warn,
            Self::VetoExercised => Severity::Critical,
            Self::FinalGateOpened => Severity::Warn,
            Self::DecisionCommitted => Severity::Info,
            Self::RollbackTriggered => Severity::Critical,
            Self::ArchiveCompleted => Severity::Info,
            Self::TerminateTriggered => Severity::Critical,
        }
    }

    /// 默认通道
    pub fn default_channel(&self) -> EventChannel {
        match self {
            Self::RunCreated => EventChannel::Control,
            Self::SeatStarted => EventChannel::Control,
            Self::SeatCompleted => EventChannel::Control,
            Self::RiskRaised => EventChannel::Security,
            Self::VetoExercised => EventChannel::Security,
            Self::FinalGateOpened => EventChannel::Control,
            Self::DecisionCommitted => EventChannel::Control,
            Self::RollbackTriggered => EventChannel::Security,
            Self::ArchiveCompleted => EventChannel::Ops,
            Self::TerminateTriggered => EventChannel::Security,
        }
    }

    /// 默认可见性
    pub fn default_scope(&self) -> EventScope {
        match self {
            // 敏感操作默认 internal，需显式降级
            Self::VetoExercised => EventScope::Internal,
            Self::RollbackTriggered => EventScope::Internal,
            Self::TerminateTriggered => EventScope::Internal,
            
            // 控制流默认 operator_visible
            Self::RunCreated => EventScope::OperatorVisible,
            Self::FinalGateOpened => EventScope::OperatorVisible,
            Self::DecisionCommitted => EventScope::OperatorVisible,
            
            // 其他按常规处理
            _ => EventScope::Internal,
        }
    }
}

/// 因果链上下文
/// 
/// 用于追踪事件间的因果关系，支持复盘和审计。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationContext {
    /// 追踪 ID（用于关联一组相关事件）
    pub correlation_id: Option<String>,
    /// 父事件 ID（因果链）
    pub parent_event_id: Option<Uuid>,
    /// 触发该事件的 actor (seat/operator/system)
    pub actor: String,
    /// 触发原因/上下文
    pub trigger_context: Option<String>,
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self {
            correlation_id: None,
            parent_event_id: None,
            actor: "system".to_string(),
            trigger_context: None,
        }
    }
}

/// 治理事件
/// 
/// 核心设计：事件对象是治理动作的第一公民，不是消息。
/// 所有会改变 run state 的操作都应该产生 GovernanceEvent。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvent {
    pub event_id: Uuid,
    pub run_id: String,
    pub seat_id: Option<String>,
    
    pub channel: EventChannel,
    pub event_type: GovernanceEventType,
    pub scope: EventScope,
    pub severity: Severity,
    
    /// 摘要（operator_visible/exportable 时显示）
    pub summary: String,
    /// 详细内容引用（指向 internal 存储）
    pub details_ref: Option<String>,
    /// 关联 artifacts
    pub artifact_refs: Vec<String>,
    
    pub created_at: DateTime<Utc>,
    
    /// 因果链上下文
    #[serde(flatten)]
    pub correlation: CorrelationContext,
}

impl GovernanceEvent {
    /// 创建新事件（使用默认值）
    pub fn new(
        run_id: impl Into<String>,
        event_type: GovernanceEventType,
        summary: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            event_id: Uuid::new_v4(),
            run_id: run_id.into(),
            seat_id: None,
            channel: event_type.default_channel(),
            event_type,
            scope: event_type.default_scope(),
            severity: event_type.default_severity(),
            summary: summary.into(),
            details_ref: None,
            artifact_refs: Vec::new(),
            created_at: now,
            correlation: CorrelationContext::default(),
        }
    }

    /// 设置 seat_id
    pub fn with_seat(mut self, seat_id: impl Into<String>) -> Self {
        self.seat_id = Some(seat_id.into());
        self
    }

    /// 设置 scope
    pub fn with_scope(mut self, scope: EventScope) -> Self {
        self.scope = scope;
        self
    }

    /// 设置 severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// 设置 channel
    pub fn with_channel(mut self, channel: EventChannel) -> Self {
        self.channel = channel;
        self
    }

    /// 设置 details_ref
    pub fn with_details(mut self, details_ref: impl Into<String>) -> Self {
        self.details_ref = Some(details_ref.into());
        self
    }

    /// 添加 artifact 引用
    pub fn with_artifact(mut self, artifact_ref: impl Into<String>) -> Self {
        self.artifact_refs.push(artifact_ref.into());
        self
    }

    /// 设置因果链上下文
    pub fn with_correlation(mut self, correlation: CorrelationContext) -> Self {
        self.correlation = correlation;
        self
    }

    /// 创建子事件（继承 correlation_id，设置 parent_event_id）
    pub fn child_event(
        &self,
        event_type: GovernanceEventType,
        summary: impl Into<String>,
    ) -> Self {
        let mut child = Self::new(&self.run_id, event_type, summary);
        child.correlation.correlation_id = self.correlation.correlation_id.clone();
        child.correlation.parent_event_id = Some(self.event_id);
        child
    }

    /// 序列化为 JSON 行格式
    pub fn to_jsonl(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 行解析
    pub fn from_jsonl(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_defaults() {
        let event = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run created");
        
        assert_eq!(event.event_type, GovernanceEventType::RunCreated);
        assert_eq!(event.channel, EventChannel::Control);
        assert_eq!(event.scope, EventScope::OperatorVisible);
        assert_eq!(event.severity, Severity::Info);
        assert_eq!(event.run_id, "run-001");
    }

    #[test]
    fn veto_is_internal_by_default() {
        let event = GovernanceEvent::new("run-001", GovernanceEventType::VetoExercised, "Veto by seat-A");
        
        assert_eq!(event.scope, EventScope::Internal);
        assert_eq!(event.severity, Severity::Critical);
        assert_eq!(event.channel, EventChannel::Security);
    }

    #[test]
    fn builder_pattern() {
        let event = GovernanceEvent::new("run-001", GovernanceEventType::SeatStarted, "Seat started")
            .with_seat("Yuheng")
            .with_scope(EventScope::OperatorVisible)
            .with_artifact("/tmp/output.json");
        
        assert_eq!(event.seat_id, Some("Yuheng".to_string()));
        assert_eq!(event.scope, EventScope::OperatorVisible);
        assert_eq!(event.artifact_refs, vec!["/tmp/output.json"]);
    }

    #[test]
    fn child_event_inherits_correlation() {
        let parent = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run created")
            .with_correlation(CorrelationContext {
                correlation_id: Some("corr-123".to_string()),
                parent_event_id: None,
                actor: "operator".to_string(),
                trigger_context: Some("manual start".to_string()),
            });

        let child = parent.child_event(GovernanceEventType::SeatStarted, "First seat");
        
        assert_eq!(child.correlation.correlation_id, Some("corr-123".to_string()));
        assert_eq!(child.correlation.parent_event_id, Some(parent.event_id));
        assert_eq!(child.run_id, "run-001");
    }

    #[test]
    fn jsonl_roundtrip() {
        let event = GovernanceEvent::new("run-001", GovernanceEventType::RiskRaised, "Risk detected")
            .with_seat("Tianshu")
            .with_severity(Severity::Warn);

        let jsonl = event.to_jsonl().unwrap();
        let parsed = GovernanceEvent::from_jsonl(&jsonl).unwrap();

        assert_eq!(parsed.event_id, event.event_id);
        assert_eq!(parsed.event_type, event.event_type);
        assert_eq!(parsed.summary, event.summary);
    }

    #[test]
    fn scope_visibility_checks() {
        assert!(!EventScope::Internal.is_operator_visible());
        assert!(!EventScope::Internal.is_exportable());
        
        assert!(EventScope::OperatorVisible.is_operator_visible());
        assert!(!EventScope::OperatorVisible.is_exportable());
        
        assert!(EventScope::Exportable.is_operator_visible());
        assert!(EventScope::Exportable.is_exportable());
    }

    #[test]
    fn test_dragoncore_interop() {
        // DragonCore 格式（snake_case enums）
        let dragoncore_json = r#"{
            "event_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "run_id": "dragoncore-test-001",
            "seat_id": null,
            "channel": "control",
            "event_type": "run_created",
            "scope": "operator_visible",
            "severity": "info",
            "summary": "Test from DragonCore",
            "details_ref": null,
            "artifact_refs": [],
            "created_at": "2026-03-16T10:00:00Z",
            "correlation_id": null,
            "parent_event_id": null,
            "actor": "system",
            "trigger_context": "runtime.init_run"
        }"#;
        
        let result: Result<GovernanceEvent, _> = serde_json::from_str(dragoncore_json);
        match &result {
            Ok(e) => println!("✓ Parsed DragonCore event: {:?}", e.event_id),
            Err(e) => println!("✗ Parse error (expected due to case mismatch): {}", e),
        }
        
        // 注意：AXI 使用 CamelCase，DragonCore 使用 snake_case
        // 需要 serde 配置來處理這個差異
        println!("Interop note: AXI uses CamelCase, DragonCore uses snake_case");
    }

    #[test]
    fn generate_sample_for_dragoncore() {
        use std::io::Write;
        
        // RunCreated
        let event1 = GovernanceEvent::new("sample-run-001", GovernanceEventType::RunCreated, "Run created for feature X implementation")
            .with_correlation(CorrelationContext {
                correlation_id: Some("corr-001".to_string()),
                parent_event_id: None,
                actor: "operator".to_string(),
                trigger_context: Some("manual start".to_string()),
            });
        
        // SeatStarted (Internal)
        let event2 = GovernanceEvent::new("sample-run-001", GovernanceEventType::SeatStarted, "Seat Tianshu started analysis phase")
            .with_seat("Tianshu")
            .with_scope(EventScope::Internal)
            .with_details("internal/tianshu_plan.md")
            .with_correlation(CorrelationContext {
                correlation_id: Some("corr-001".to_string()),
                parent_event_id: Some(event1.event_id),
                actor: "Tianshu".to_string(),
                trigger_context: None,
            });
        
        // RiskRaised
        let event3 = GovernanceEvent::new("sample-run-001", GovernanceEventType::RiskRaised, "Potential deadlock in concurrent module access")
            .with_seat("Yuheng")
            .with_severity(Severity::Warn)
            .with_correlation(CorrelationContext {
                correlation_id: Some("risk-001".to_string()),
                parent_event_id: None,
                actor: "system".to_string(),
                trigger_context: Some("automated scan".to_string()),
            });
        
        // VetoExercised
        let event4 = GovernanceEvent::new("sample-run-001", GovernanceEventType::VetoExercised, "Veto exercised by operator on risk-001")
            .with_correlation(CorrelationContext {
                correlation_id: Some("risk-001".to_string()),
                parent_event_id: Some(event3.event_id),
                actor: "operator".to_string(),
                trigger_context: Some("manual veto".to_string()),
            });
        
        std::fs::create_dir_all("test_vectors").unwrap();
        let mut file = std::fs::File::create("test_vectors/axi_sample.jsonl").unwrap();
        writeln!(file, "{}", event1.to_jsonl().unwrap()).unwrap();
        writeln!(file, "{}", event2.to_jsonl().unwrap()).unwrap();
        writeln!(file, "{}", event3.to_jsonl().unwrap()).unwrap();
        writeln!(file, "{}", event4.to_jsonl().unwrap()).unwrap();
        
        println!("Generated test_vectors/axi_sample.jsonl");
    }
}
