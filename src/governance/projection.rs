//! DIBL 投影层
//! 
//! 将事件流投影为 operator 可见的摘要视图。
//! 实现 AXI 的"公开结果，隐藏原始内部判断"原则。

use std::collections::HashMap;

use chrono::{DateTime, Utc, Duration};

use super::event::{GovernanceEvent, EventScope, GovernanceEventType};
use super::store::EventStore;

/// Run 投影
/// 
/// 聚合单个 run 的所有事件，形成当前状态视图。
#[derive(Debug, Clone, Default)]
pub struct RunProjection {
    pub run_id: String,
    pub current_phase: String,
    pub current_seat: Option<String>,
    pub last_event_summary: String,
    pub last_event_time: Option<DateTime<Utc>>,
    pub open_risks: Vec<String>,
    pub veto_count: u32,
    pub is_final_gate_open: bool,
    pub elapsed_time: Duration,
    pub final_outcome: Option<String>,
}

impl RunProjection {
    /// 从事件列表构建投影
    pub fn from_events(run_id: &str, events: &[GovernanceEvent]) -> Self {
        let mut projection = Self {
            run_id: run_id.to_string(),
            current_phase: "initialized".to_string(),
            ..Default::default()
        };

        let first_time = events.first().map(|e| e.created_at);
        
        for event in events {
            projection.apply_event(event, first_time);
        }

        projection
    }

    /// 应用单个事件
    fn apply_event(&mut self, event: &GovernanceEvent, first_time: Option<DateTime<Utc>>) {
        self.last_event_summary = event.summary.clone();
        self.last_event_time = Some(event.created_at);

        match event.event_type {
            GovernanceEventType::RunCreated => {
                self.current_phase = "created".to_string();
            }
            GovernanceEventType::SeatStarted => {
                self.current_phase = "executing".to_string();
                self.current_seat = event.seat_id.clone();
            }
            GovernanceEventType::SeatCompleted => {
                self.current_phase = "reviewing".to_string();
            }
            GovernanceEventType::RiskRaised => {
                self.open_risks.push(event.summary.clone());
                // 风险提出后进入 review 状态
                if self.current_phase == "executing" {
                    self.current_phase = "reviewing".to_string();
                }
            }
            GovernanceEventType::VetoExercised => {
                self.veto_count += 1;
                self.open_risks.push(format!("Veto: {}", event.summary));
            }
            GovernanceEventType::FinalGateOpened => {
                self.is_final_gate_open = true;
                self.current_phase = "final_gate".to_string();
            }
            GovernanceEventType::DecisionCommitted => {
                self.current_phase = "committed".to_string();
                self.final_outcome = Some(event.summary.clone());
            }
            GovernanceEventType::RollbackTriggered => {
                self.current_phase = "rollback".to_string();
                self.open_risks.push("Rollback triggered".to_string());
            }
            GovernanceEventType::ArchiveCompleted => {
                self.current_phase = "archived".to_string();
            }
            GovernanceEventType::TerminateTriggered => {
                self.current_phase = "terminated".to_string();
            }
        }

        // 计算运行时间
        if let Some(first) = first_time {
            self.elapsed_time = event.created_at - first;
        }
    }

    /// 获取风险状态
    pub fn risk_status(&self) -> RiskStatus {
        if self.open_risks.iter().any(|r| r.contains("Veto") || r.contains("Rollback")) {
            RiskStatus::Critical
        } else if !self.open_risks.is_empty() {
            RiskStatus::Warning
        } else {
            RiskStatus::Normal
        }
    }

    /// 获取简短状态摘要（用于 operator 视图）
    pub fn short_summary(&self) -> String {
        let risk_indicator = match self.risk_status() {
            RiskStatus::Normal => "✓",
            RiskStatus::Warning => "⚠",
            RiskStatus::Critical => "✗",
        };

        format!(
            "[{}] {} | Phase: {} | Seat: {} | Risks: {} | Veto: {}",
            risk_indicator,
            self.run_id,
            self.current_phase,
            self.current_seat.as_deref().unwrap_or("-"),
            self.open_risks.len(),
            self.veto_count
        )
    }
}

/// 风险状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskStatus {
    Normal,
    Warning,
    Critical,
}

/// Operator 视图
/// 
/// 聚合多个 run 的投影，形成操作员可见的整体视图。
pub struct OperatorView {
    store: Box<dyn EventStore>,
    projections: HashMap<String, RunProjection>,
}

impl OperatorView {
    /// 创建新的 operator 视图
    pub fn new(store: Box<dyn EventStore>) -> Self {
        Self {
            store,
            projections: HashMap::new(),
        }
    }

    /// 刷新所有 run 的投影
    pub fn refresh_all(&mut self) -> anyhow::Result<()> {
        let run_ids = self.store.list_run_ids()?;
        
        for run_id in run_ids {
            self.refresh_run(&run_id)?;
        }
        
        Ok(())
    }

    /// 刷新单个 run
    pub fn refresh_run(&mut self, run_id: &str) -> anyhow::Result<()> {
        let events = self.store.load_run_events(run_id)?;
        
        // 只使用 operator_visible 的事件构建视图
        let visible_events: Vec<_> = events
            .into_iter()
            .filter(|e| e.scope.is_operator_visible())
            .collect();
        
        let projection = RunProjection::from_events(run_id, &visible_events);
        self.projections.insert(run_id.to_string(), projection);
        
        Ok(())
    }

    /// 获取 run 投影
    pub fn get_run(&self, run_id: &str) -> Option<&RunProjection> {
        self.projections.get(run_id)
    }

    /// 列出所有 run
    pub fn list_runs(&self) -> Vec<&RunProjection> {
        self.projections.values().collect()
    }

    /// 按风险状态筛选
    pub fn runs_by_risk(&self, status: RiskStatus) -> Vec<&RunProjection> {
        self.projections
            .values()
            .filter(|p| p.risk_status() == status)
            .collect()
    }

    /// 获取需要关注的 run（Critical 或 Warning）
    pub fn runs_needing_attention(&self) -> Vec<&RunProjection> {
        self.projections
            .values()
            .filter(|p| p.risk_status() != RiskStatus::Normal)
            .collect()
    }

    /// 生成 dashboard 摘要
    pub fn dashboard_summary(&self) -> DashboardSummary {
        let total = self.projections.len();
        let critical = self.runs_by_risk(RiskStatus::Critical).len();
        let warning = self.runs_by_risk(RiskStatus::Warning).len();
        let normal = total - critical - warning;

        let active_runs: Vec<_> = self.projections
            .values()
            .filter(|p| !matches!(p.current_phase.as_str(), "archived" | "terminated"))
            .collect();

        DashboardSummary {
            total_runs: total,
            active_runs: active_runs.len(),
            critical,
            warning,
            normal,
            open_final_gates: active_runs.iter().filter(|p| p.is_final_gate_open).count(),
        }
    }

    /// 生成文本报告
    pub fn generate_report(&self) -> String {
        let summary = self.dashboard_summary();
        let mut report = format!(
            "# DragonCore Operator View\n\n## Summary\n- Total: {} runs\n- Active: {}\n- Critical: {}\n- Warning: {}\n- Normal: {}\n- Open Final Gates: {}\n\n## Active Runs\n",
            summary.total_runs,
            summary.active_runs,
            summary.critical,
            summary.warning,
            summary.normal,
            summary.open_final_gates
        );

        for projection in self.list_runs() {
            if projection.current_phase != "archived" && projection.current_phase != "terminated" {
                report.push_str(&format!("- {}\n", projection.short_summary()));
            }
        }

        if summary.critical > 0 {
            report.push_str("\n## Critical Runs (Need Attention)\n");
            for projection in self.runs_by_risk(RiskStatus::Critical) {
                report.push_str(&format!("- {}: {}\n", projection.run_id, projection.last_event_summary));
                for risk in &projection.open_risks {
                    report.push_str(&format!("  - ⚠ {}\n", risk));
                }
            }
        }

        report
    }
}

/// Dashboard 摘要
#[derive(Debug, Clone)]
pub struct DashboardSummary {
    pub total_runs: usize,
    pub active_runs: usize,
    pub critical: usize,
    pub warning: usize,
    pub normal: usize,
    pub open_final_gates: usize,
}

/// 脱敏过滤器
/// 
/// 将 internal 事件降级为 operator_visible 时脱敏。
pub fn sanitize_for_operator(event: &GovernanceEvent) -> Option<GovernanceEvent> {
    if !event.scope.is_operator_visible() {
        return None;
    }

    let mut sanitized = event.clone();
    
    // 如果是 internal 详细信息，只保留摘要
    if event.scope == EventScope::Internal {
        sanitized.details_ref = None;
        sanitized.artifact_refs.clear();
    }
    
    Some(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::store::InMemoryEventStore;

    #[test]
    fn run_projection_from_events() {
        let events = vec![
            GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run created")
                .with_scope(EventScope::OperatorVisible),
            GovernanceEvent::new("run-001", GovernanceEventType::SeatStarted, "Seat Yuheng started")
                .with_seat("Yuheng")
                .with_scope(EventScope::OperatorVisible),
            GovernanceEvent::new("run-001", GovernanceEventType::RiskRaised, "Risk detected")
                .with_scope(EventScope::OperatorVisible),
        ];

        let projection = RunProjection::from_events("run-001", &events);

        assert_eq!(projection.run_id, "run-001");
        assert_eq!(projection.current_phase, "reviewing");
        assert_eq!(projection.current_seat, Some("Yuheng".to_string()));
        assert_eq!(projection.open_risks.len(), 1);
        assert_eq!(projection.risk_status(), RiskStatus::Warning);
    }

    #[test]
    fn veto_makes_critical() {
        let events = vec![
            GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run"),
            GovernanceEvent::new("run-001", GovernanceEventType::VetoExercised, "Veto by Tianshu"),
        ];

        let projection = RunProjection::from_events("run-001", &events);
        assert_eq!(projection.risk_status(), RiskStatus::Critical);
        assert_eq!(projection.veto_count, 1);
    }

    #[test]
    fn operator_view_filters_internal() {
        let store = Box::new(InMemoryEventStore::new());
        
        // 添加 operator_visible 事件
        let visible = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run")
            .with_scope(EventScope::OperatorVisible);
        store.append_event(&visible).unwrap();
        
        // 添加 internal 事件
        let internal = GovernanceEvent::new("run-001", GovernanceEventType::VetoExercised, "Veto")
            .with_scope(EventScope::Internal);
        store.append_event(&internal).unwrap();

        let mut view = OperatorView::new(store);
        view.refresh_run("run-001").unwrap();

        // Veto 是 internal，不应该在 operator 视图中
        let projection = view.get_run("run-001").unwrap();
        assert_eq!(projection.veto_count, 0);  // 过滤掉了
    }

    #[test]
    fn dashboard_summary() {
        let store = Box::new(InMemoryEventStore::new());
        
        // Run 1: Normal
        store.append_event(&GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run")
            .with_scope(EventScope::OperatorVisible)).unwrap();
        
        // Run 2: Critical (veto)
        store.append_event(&GovernanceEvent::new("run-002", GovernanceEventType::RunCreated, "Run")
            .with_scope(EventScope::OperatorVisible)).unwrap();
        store.append_event(&GovernanceEvent::new("run-002", GovernanceEventType::VetoExercised, "Veto")
            .with_scope(EventScope::OperatorVisible)).unwrap();
        
        // Run 3: Warning (risk)
        store.append_event(&GovernanceEvent::new("run-003", GovernanceEventType::RunCreated, "Run")
            .with_scope(EventScope::OperatorVisible)).unwrap();
        store.append_event(&GovernanceEvent::new("run-003", GovernanceEventType::RiskRaised, "Risk")
            .with_scope(EventScope::OperatorVisible)).unwrap();

        let mut view = OperatorView::new(store);
        view.refresh_all().unwrap();

        let summary = view.dashboard_summary();
        assert_eq!(summary.total_runs, 3);
        assert_eq!(summary.critical, 1);
        assert_eq!(summary.warning, 1);
        assert_eq!(summary.normal, 1);
    }

    #[test]
    fn sanitize_internal_event() {
        let event = GovernanceEvent::new("run-001", GovernanceEventType::VetoExercised, "Veto")
            .with_scope(EventScope::Internal)
            .with_details("/secret/path.json")
            .with_artifact("/secret/output.json");

        // Internal 事件不应该被 operator 看到
        assert!(sanitize_for_operator(&event).is_none());
        
        // OperatorVisible 事件应该可见
        let visible = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run")
            .with_scope(EventScope::OperatorVisible);
        assert!(sanitize_for_operator(&visible).is_some());
    }
}
