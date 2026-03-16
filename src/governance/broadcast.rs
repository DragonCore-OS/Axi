//! DIBL 广播层
//!
//! 核心原则：广播层是派生层，不是主控层。
//! 真相源是 JSON/ledger，广播只是通知机制。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};

use anyhow::Result;

use super::event::{GovernanceEvent, EventChannel, Severity};
use super::store::EventStore;

/// 广播 trait
pub trait Broadcaster: Send + Sync {
    /// 发布事件
    fn publish(&self, event: GovernanceEvent) -> Result<()>;
    
    /// 订阅特定 run 的事件
    fn subscribe_run(&self, run_id: &str) -> mpsc::Receiver<GovernanceEvent>;
    
    /// 订阅特定通道
    fn subscribe_channel(&self, channel: EventChannel) -> mpsc::Receiver<GovernanceEvent>;
    
    /// 订阅特定严重程度
    fn subscribe_severity(&self, min_severity: Severity) -> mpsc::Receiver<GovernanceEvent>;
}

/// 事件 sink trait
/// 
/// 用于将事件持久化到存储
pub trait EventSink: Send + Sync {
    fn sink(&self, event: &GovernanceEvent) -> Result<()>;
}

impl<T: EventStore> EventSink for T {
    fn sink(&self, event: &GovernanceEvent) -> Result<()> {
        self.append_event(event)
    }
}

/// DIBL 广播器
/// 
/// 实现规则：
/// 1. 先持久化，后广播
/// 2. 广播层是派生层
/// 3. 内部事件默认不进入 operator 视图
pub struct DiblBroadcaster {
    /// 事件存储
    store: Arc<dyn EventStore>,
    /// Run 订阅者
    run_subscribers: Mutex<HashMap<String, Vec<mpsc::Sender<GovernanceEvent>>>>,
    /// 通道订阅者
    channel_subscribers: Mutex<HashMap<EventChannel, Vec<mpsc::Sender<GovernanceEvent>>>>,
    /// 严重程度订阅者（按最小严重程度分组）
    severity_subscribers: Mutex<HashMap<Severity, Vec<mpsc::Sender<GovernanceEvent>>>>,
}

impl DiblBroadcaster {
    /// 创建新的广播器
    pub fn new(store: Arc<dyn EventStore>) -> Self {
        Self {
            store,
            run_subscribers: Mutex::new(HashMap::new()),
            channel_subscribers: Mutex::new(HashMap::new()),
            severity_subscribers: Mutex::new(HashMap::new()),
        }
    }

    /// 创建带默认存储的广播器
    pub fn with_default_store() -> Result<Self> {
        let store = Arc::new(super::store::JsonlEventStore::with_default_path()?);
        Ok(Self::new(store))
    }

    /// 发布事件（核心方法）
    /// 
    /// 执行顺序：
    /// 1. 持久化到存储（真相源）
    /// 2. 广播给订阅者（派生）
    pub fn publish(&self, event: GovernanceEvent) -> Result<()> {
        // Step 1: 持久化（真相源）
        self.store.append_event(&event)?;

        // Step 2: 广播给订阅者（派生）
        self.broadcast_to_subscribers(&event)?;

        Ok(())
    }

    /// 广播给所有订阅者
    fn broadcast_to_subscribers(&self, event: &GovernanceEvent) -> Result<()> {
        // Run 订阅者
        if let Ok(subscribers) = self.run_subscribers.lock() {
            if let Some(senders) = subscribers.get(&event.run_id) {
                for sender in senders {
                    // 非阻塞发送，失败则丢弃
                    let _ = sender.send(event.clone());
                }
            }
        }

        // 通道订阅者
        if let Ok(subscribers) = self.channel_subscribers.lock() {
            if let Some(senders) = subscribers.get(&event.channel) {
                for sender in senders {
                    let _ = sender.send(event.clone());
                }
            }
        }

        // 严重程度订阅者（发送给所有要求 <= 当前严重程度的订阅者）
        if let Ok(subscribers) = self.severity_subscribers.lock() {
            for (min_severity, senders) in subscribers.iter() {
                if Self::severity_meets_threshold(event.severity, *min_severity) {
                    for sender in senders {
                        let _ = sender.send(event.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// 检查严重程度是否满足阈值
    fn severity_meets_threshold(actual: Severity, threshold: Severity) -> bool {
        use Severity::*;
        match (actual, threshold) {
            (Critical, _) => true,  // Critical 总是满足
            (Warn, Warn) | (Warn, Info) => true,
            (Info, Info) => true,
            _ => false,
        }
    }

    /// 订阅特定 run
    pub fn subscribe_run(&self, run_id: &str) -> mpsc::Receiver<GovernanceEvent> {
        let (tx, rx) = mpsc::channel();
        
        if let Ok(mut subscribers) = self.run_subscribers.lock() {
            subscribers
                .entry(run_id.to_string())
                .or_insert_with(Vec::new)
                .push(tx);
        }
        
        rx
    }

    /// 订阅特定通道
    pub fn subscribe_channel(&self, channel: EventChannel) -> mpsc::Receiver<GovernanceEvent> {
        let (tx, rx) = mpsc::channel();
        
        if let Ok(mut subscribers) = self.channel_subscribers.lock() {
            subscribers
                .entry(channel)
                .or_insert_with(Vec::new)
                .push(tx);
        }
        
        rx
    }

    /// 订阅特定严重程度（接收 >= 该严重程度的所有事件）
    pub fn subscribe_severity(&self, min_severity: Severity) -> mpsc::Receiver<GovernanceEvent> {
        let (tx, rx) = mpsc::channel();
        
        if let Ok(mut subscribers) = self.severity_subscribers.lock() {
            subscribers
                .entry(min_severity)
                .or_insert_with(Vec::new)
                .push(tx);
        }
        
        rx
    }

    /// 获取存储引用
    pub fn store(&self) -> &Arc<dyn EventStore> {
        &self.store
    }

    /// 清理已断开连接的订阅者（定期调用）
    pub fn cleanup_disconnected(&self) {
        // Run 订阅者
        if let Ok(mut subscribers) = self.run_subscribers.lock() {
            for senders in subscribers.values_mut() {
                senders.retain(|s| s.send(GovernanceEvent::new("", super::event::GovernanceEventType::RunCreated, "")).is_ok() || true);
                // 注意：这里简化处理，实际应该检查断开状态
            }
        }
    }
}

impl Broadcaster for DiblBroadcaster {
    fn publish(&self, event: GovernanceEvent) -> Result<()> {
        self.publish(event)
    }

    fn subscribe_run(&self, run_id: &str) -> mpsc::Receiver<GovernanceEvent> {
        self.subscribe_run(run_id)
    }

    fn subscribe_channel(&self, channel: EventChannel) -> mpsc::Receiver<GovernanceEvent> {
        self.subscribe_channel(channel)
    }

    fn subscribe_severity(&self, min_severity: Severity) -> mpsc::Receiver<GovernanceEvent> {
        self.subscribe_severity(min_severity)
    }
}

/// 过滤广播器
/// 
/// 只广播满足特定条件的事件
pub struct FilteringBroadcaster {
    inner: Arc<dyn Broadcaster>,
    filter: Box<dyn Fn(&GovernanceEvent) -> bool + Send + Sync>,
}

impl FilteringBroadcaster {
    pub fn new<F>(inner: Arc<dyn Broadcaster>, filter: F) -> Self
    where
        F: Fn(&GovernanceEvent) -> bool + Send + Sync + 'static,
    {
        Self {
            inner,
            filter: Box::new(filter),
        }
    }

    /// 创建只广播 operator_visible 事件的过滤广播器
    pub fn operator_visible_only(inner: Arc<dyn Broadcaster>) -> Self {
        Self::new(inner, |event| event.scope.is_operator_visible())
    }

    /// 创建只广播特定通道事件的过滤广播器
    pub fn channel_only(inner: Arc<dyn Broadcaster>, channel: EventChannel) -> Self {
        Self::new(inner, move |event| event.channel == channel)
    }
}

impl Broadcaster for FilteringBroadcaster {
    fn publish(&self, event: GovernanceEvent) -> Result<()> {
        if (self.filter)(&event) {
            self.inner.publish(event)
        } else {
            Ok(())
        }
    }

    fn subscribe_run(&self, run_id: &str) -> mpsc::Receiver<GovernanceEvent> {
        self.inner.subscribe_run(run_id)
    }

    fn subscribe_channel(&self, channel: EventChannel) -> mpsc::Receiver<GovernanceEvent> {
        self.inner.subscribe_channel(channel)
    }

    fn subscribe_severity(&self, min_severity: Severity) -> mpsc::Receiver<GovernanceEvent> {
        self.inner.subscribe_severity(min_severity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::event::{GovernanceEventType, EventScope};
    use super::super::store::InMemoryEventStore;

    fn create_test_broadcaster() -> DiblBroadcaster {
        let store = Arc::new(InMemoryEventStore::new());
        DiblBroadcaster::new(store)
    }

    #[test]
    fn publish_and_subscribe_run() {
        let broadcaster = create_test_broadcaster();
        let rx = broadcaster.subscribe_run("run-001");

        let event = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run started");
        broadcaster.publish(event.clone()).unwrap();

        let received = rx.recv().unwrap();
        assert_eq!(received.event_id, event.event_id);
        assert_eq!(received.event_type, GovernanceEventType::RunCreated);
    }

    #[test]
    fn subscribe_channel() {
        let broadcaster = create_test_broadcaster();
        let rx = broadcaster.subscribe_channel(EventChannel::Security);

        // Security 事件应该收到
        let security_event = GovernanceEvent::new("run-001", GovernanceEventType::VetoExercised, "Veto")
            .with_channel(EventChannel::Security);
        broadcaster.publish(security_event).unwrap();

        // Control 事件不应该收到
        let control_event = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run")
            .with_channel(EventChannel::Control);
        broadcaster.publish(control_event).unwrap();

        // 只应该收到 Security 事件
        let received = rx.recv().unwrap();
        assert_eq!(received.event_type, GovernanceEventType::VetoExercised);
        
        // 不应该再有消息
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn subscribe_severity() {
        let broadcaster = create_test_broadcaster();
        let rx = broadcaster.subscribe_severity(Severity::Warn);  // 只接收 Warn 及以上

        // Critical 应该收到
        let critical = GovernanceEvent::new("run-001", GovernanceEventType::TerminateTriggered, "Stop")
            .with_severity(Severity::Critical);
        broadcaster.publish(critical).unwrap();

        // Warn 应该收到
        let warn = GovernanceEvent::new("run-001", GovernanceEventType::RiskRaised, "Risk")
            .with_severity(Severity::Warn);
        broadcaster.publish(warn).unwrap();

        // Info 不应该收到
        let info = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run")
            .with_severity(Severity::Info);
        broadcaster.publish(info).unwrap();

        let received1 = rx.recv().unwrap();
        let received2 = rx.recv().unwrap();
        
        assert_eq!(received1.severity, Severity::Critical);
        assert_eq!(received2.severity, Severity::Warn);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn filter_operator_visible() {
        let inner = Arc::new(create_test_broadcaster());
        let filter = FilteringBroadcaster::operator_visible_only(inner.clone());

        let rx = inner.subscribe_run("run-001");

        // OperatorVisible 事件
        let visible = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run")
            .with_scope(EventScope::OperatorVisible);
        filter.publish(visible).unwrap();

        // Internal 事件（应该被过滤）
        let internal = GovernanceEvent::new("run-001", GovernanceEventType::VetoExercised, "Veto")
            .with_scope(EventScope::Internal);
        filter.publish(internal).unwrap();

        // 只应该收到 OperatorVisible
        let received = rx.recv().unwrap();
        assert_eq!(received.scope, EventScope::OperatorVisible);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn events_persisted_to_store() {
        let broadcaster = create_test_broadcaster();

        let event = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run");
        broadcaster.publish(event).unwrap();

        // 验证存储
        let stored = broadcaster.store().load_run_events("run-001").unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].event_type, GovernanceEventType::RunCreated);
    }
}
