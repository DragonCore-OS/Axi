//! DIBL Governance Runtime (簡化演示)
//!
//! 展示如何在 AXI 側接入 DIBL event emission。
//! 非完整 19-seat 實現，僅供 emission 模式參考。

use std::sync::Arc;
use anyhow::Result;

use super::{
    DiblBroadcaster, GovernanceEvent, GovernanceEventType, 
    CorrelationContext, EventScope
};

/// 簡化版治理運行時
pub struct GovernanceRuntime {
    broadcaster: Arc<DiblBroadcaster>,
}

impl GovernanceRuntime {
    /// 創建新的運行時實例
    pub fn new(broadcaster: Arc<DiblBroadcaster>) -> Self {
        Self { broadcaster }
    }

    /// 使用默認存儲創建
    pub fn with_default_store() -> Result<Self> {
        let broadcaster = Arc::new(DiblBroadcaster::with_default_store()?);
        Ok(Self::new(broadcaster))
    }

    // ========== 8個 Event Emission 點 ==========

    /// 1. RunCreated - 創建新的治理 run
    pub fn init_run(&self, run_id: &str, context: &str) -> Result<()> {
        // 先落盤（這裡簡化為記錄，真實實現會保存到 state store）
        // 真實實現：先落盤到 state store

        // 後廣播
        let event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::RunCreated,
            format!("Run created: {}", context)
        ).with_correlation(CorrelationContext {
            correlation_id: Some(format!("corr-{}", run_id)),
            parent_event_id: None,
            actor: "operator".to_string(),
            trigger_context: Some(context.to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    /// 2. SeatStarted - Seat 開始執行
    pub fn start_seat(&self, run_id: &str, seat_id: &str, task: &str) -> Result<()> {
        let event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::SeatStarted,
            format!("Seat {} started: {}", seat_id, task)
        )
        .with_seat(seat_id)
        .with_scope(EventScope::Internal)
        .with_correlation(CorrelationContext {
            correlation_id: Some(format!("corr-{}", run_id)),
            parent_event_id: None, // 真實實現應關聯 RunCreated
            actor: seat_id.to_string(),
            trigger_context: Some(task.to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    /// 3. SeatCompleted - Seat 完成執行
    pub fn complete_seat(&self, run_id: &str, seat_id: &str, result: &str) -> Result<()> {
        let event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::SeatCompleted,
            format!("Seat {} completed: {}", seat_id, result)
        )
        .with_seat(seat_id)
        .with_scope(EventScope::Internal)
        .with_correlation(CorrelationContext {
            correlation_id: Some(format!("corr-{}", run_id)),
            parent_event_id: None,
            actor: seat_id.to_string(),
            trigger_context: Some(result.to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    /// 4. RiskRaised - 風險提出
    pub fn raise_risk(&self, run_id: &str, seat_id: Option<&str>, risk_desc: &str) -> Result<()> {
        let mut event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::RiskRaised,
            risk_desc.to_string()
        );
        
        if let Some(sid) = seat_id {
            event = event.with_seat(sid);
        }
        
        event = event.with_correlation(CorrelationContext {
            correlation_id: Some(format!("risk-{}", uuid::Uuid::new_v4())),
            parent_event_id: None,
            actor: seat_id.unwrap_or("system").to_string(),
            trigger_context: Some("automated scan".to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    /// 5. VetoExercised - 行使否決權
    pub fn exercise_veto(&self, run_id: &str, seat_id: &str, reason: &str) -> Result<()> {
        let event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::VetoExercised,
            format!("Veto by {}: {}", seat_id, reason)
        )
        .with_correlation(CorrelationContext {
            correlation_id: Some(format!("veto-{}", uuid::Uuid::new_v4())),
            parent_event_id: None,
            actor: seat_id.to_string(),
            trigger_context: Some(reason.to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    /// 6. FinalGateOpened - 最終閘門開啟
    pub fn open_final_gate(&self, run_id: &str) -> Result<()> {
        let event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::FinalGateOpened,
            "All seats completed, final gate opened"
        )
        .with_correlation(CorrelationContext {
            correlation_id: Some(format!("corr-{}", run_id)),
            parent_event_id: None,
            actor: "system".to_string(),
            trigger_context: Some("auto progression".to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    /// 7. DecisionCommitted - 決策提交
    pub fn commit_decision(&self, run_id: &str, decision: &str) -> Result<()> {
        let event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::DecisionCommitted,
            format!("Decision committed: {}", decision)
        )
        .with_correlation(CorrelationContext {
            correlation_id: Some(format!("corr-{}", run_id)),
            parent_event_id: None,
            actor: "system".to_string(),
            trigger_context: Some(decision.to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    /// 8a. ArchiveCompleted - 歸檔完成
    pub fn archive_run(&self, run_id: &str, archive_ref: &str) -> Result<()> {
        let event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::ArchiveCompleted,
            format!("Run archived to {}", archive_ref)
        )
        .with_correlation(CorrelationContext {
            correlation_id: Some(format!("corr-{}", run_id)),
            parent_event_id: None,
            actor: "system".to_string(),
            trigger_context: Some(archive_ref.to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    /// 8b. TerminateTriggered - 終止觸發
    pub fn terminate_run(&self, run_id: &str, reason: &str, triggered_by: &str) -> Result<()> {
        let event = GovernanceEvent::new(
            run_id,
            GovernanceEventType::TerminateTriggered,
            format!("Run terminated: {}", reason)
        )
        .with_correlation(CorrelationContext {
            correlation_id: Some(format!("term-{}", uuid::Uuid::new_v4())),
            parent_event_id: None,
            actor: triggered_by.to_string(),
            trigger_context: Some(reason.to_string()),
        });

        self.emit_event(event)?;
        Ok(())
    }

    // ========== 內部輔助方法 ==========

    /// 發射事件（遵循 DragonCore 模式）
    fn emit_event(&self, event: GovernanceEvent) -> Result<()> {
        if let Err(e) = self.broadcaster.publish(event) {
            eprintln!("[DIBL] Event emission failed: {}", e);
            // 不返回錯誤，不阻塞主操作
        }
        Ok(())
    }

    /// 獲取 broadcaster 引用（供外部訂閱使用）
    pub fn broadcaster(&self) -> Arc<DiblBroadcaster> {
        self.broadcaster.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{InMemoryEventStore, EventStore};

    #[test]
    fn runtime_emission_8_points() {
        let store = Arc::new(InMemoryEventStore::new());
        let broadcaster = Arc::new(DiblBroadcaster::new(store.clone()));
        let runtime = GovernanceRuntime::new(broadcaster);

        // 執行 8 個 emission 點
        runtime.init_run("test-run-001", "Test feature implementation").unwrap();
        runtime.start_seat("test-run-001", "Tianshu", "analysis").unwrap();
        runtime.complete_seat("test-run-001", "Tianshu", "analysis complete").unwrap();
        runtime.raise_risk("test-run-001", Some("Yuheng"), "Potential deadlock").unwrap();
        runtime.exercise_veto("test-run-001", "operator", "Risk unacceptable").unwrap();
        runtime.open_final_gate("test-run-001").unwrap();
        runtime.commit_decision("test-run-001", "Proceed with caution").unwrap();
        runtime.archive_run("test-run-001", "/archive/test-run-001.json").unwrap();

        // 驗證事件已存儲
        let events = store.load_run_events("test-run-001").unwrap();
        assert_eq!(events.len(), 8);

        // 驗證事件類型順序
        let types: Vec<_> = events.iter().map(|e| e.event_type).collect();
        assert!(matches!(types[0], GovernanceEventType::RunCreated));
        assert!(matches!(types[1], GovernanceEventType::SeatStarted));
        assert!(matches!(types[7], GovernanceEventType::ArchiveCompleted));
    }

    #[test]
    fn emission_failure_not_blocking() {
        // 使用會失敗的存儲（這個測試驗證 emit 失敗不阻塞）
        let store = Arc::new(InMemoryEventStore::new());
        let broadcaster = Arc::new(DiblBroadcaster::new(store));
        let runtime = GovernanceRuntime::new(broadcaster);

        // 即使內部失敗，也不應該返回錯誤
        let result = runtime.init_run("test", "context");
        assert!(result.is_ok());
    }
}
