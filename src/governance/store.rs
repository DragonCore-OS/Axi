//! DIBL 事件存储层
//!
//! 使用 JSONL 格式与现有 JSON/CSV 体系兼容。
//! 路径: runtime_state/events/{run_id}.jsonl

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use super::event::GovernanceEvent;

/// 事件存储 trait
pub trait EventStore: Send + Sync {
    /// 追加事件
    fn append_event(&self, event: &GovernanceEvent) -> Result<()>;
    
    /// 加载 run 的所有事件
    fn load_run_events(&self, run_id: &str) -> Result<Vec<GovernanceEvent>>;
    
    /// 获取事件文件路径
    fn event_file_path(&self, run_id: &str) -> PathBuf;
    
    /// 列出所有 run ID
    fn list_run_ids(&self) -> Result<Vec<String>>;
}

/// JSONL 事件存储
/// 
/// 与 DragonCore 现有 runtime_state 结构兼容：
/// - 每个 run 一个 JSONL 文件
/// - 追加写入，支持事件回放
/// - 人类可读，便于调试
pub struct JsonlEventStore {
    base_dir: PathBuf,
}

impl JsonlEventStore {
    /// 创建新的存储实例
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // 确保目录存在
        fs::create_dir_all(&base_dir)
            .with_context(|| format!("Failed to create events directory: {:?}", base_dir))?;
        
        Ok(Self { base_dir })
    }

    /// 使用默认路径创建
    pub fn with_default_path() -> Result<Self> {
        Self::new("runtime_state/events")
    }

    /// 确保 run 的事件文件存在
    fn ensure_run_file(&self, run_id: &str) -> Result<PathBuf> {
        let path = self.event_file_path(run_id);
        
        if !path.exists() {
            // 创建空文件
            File::create(&path)
                .with_context(|| format!("Failed to create event file: {:?}", path))?;
            
            // 写入文件头注释
            let mut file = OpenOptions::new()
                .write(true)
                .open(&path)?;
            
            writeln!(file, "# DragonCore Internal Broadcast Layer")?;
            writeln!(file, "# Run ID: {}", run_id)?;
            writeln!(file, "# Created: {}", Utc::now().to_rfc3339())?;
            writeln!(file, "# Format: JSON Lines (one event per line)")?;
            writeln!(file, "#")?;
        }
        
        Ok(path)
    }

    /// 获取事件文件路径
    pub fn event_file_path(&self, run_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.jsonl", run_id))
    }

    /// 列出所有有事件的 run
    pub fn list_run_ids(&self) -> Result<Vec<String>> {
        let mut run_ids = Vec::new();
        
        if !self.base_dir.exists() {
            return Ok(run_ids);
        }
        
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    run_ids.push(stem.to_string_lossy().to_string());
                }
            }
        }
        
        Ok(run_ids)
    }

    /// 获取 run 的事件数量
    pub fn event_count(&self, run_id: &str) -> Result<usize> {
        let path = self.event_file_path(run_id);
        
        if !path.exists() {
            return Ok(0);
        }
        
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for line in reader.lines() {
            let line = line?;
            // 跳过注释行
            if !line.starts_with('#') && !line.trim().is_empty() {
                count += 1;
            }
        }
        
        Ok(count)
    }
}

impl EventStore for JsonlEventStore {
    fn append_event(&self, event: &GovernanceEvent) -> Result<()> {
        let path = self.ensure_run_file(&event.run_id)?;
        
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open event file: {:?}", path))?;
        
        let jsonl = event.to_jsonl()
            .with_context(|| "Failed to serialize event")?;
        
        writeln!(file, "{}", jsonl)
            .with_context(|| format!("Failed to write event to: {:?}", path))?;
        
        Ok(())
    }

    fn load_run_events(&self, run_id: &str) -> Result<Vec<GovernanceEvent>> {
        let path = self.event_file_path(run_id);
        
        if !path.exists() {
            return Ok(Vec::new());
        }
        
        let file = File::open(&path)
            .with_context(|| format!("Failed to open event file: {:?}", path))?;
        
        let reader = BufReader::new(file);
        let mut events = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line
                .with_context(|| format!("Failed to read line {} from {:?}", line_num + 1, path))?;
            
            // 跳过注释行和空行
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let event = GovernanceEvent::from_jsonl(&line)
                .with_context(|| format!("Failed to parse event at line {}: {}", line_num + 1, line))?;
            
            events.push(event);
        }
        
        Ok(events)
    }

    fn list_run_ids(&self) -> Result<Vec<String>> {
        use std::collections::HashSet;
        let mut run_ids = HashSet::new();
        
        // 读取事件目录下的所有 .jsonl 文件
        if self.base_dir.exists() {
            for entry in std::fs::read_dir(&self.base_dir)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "jsonl" {
                        if let Some(stem) = path.file_stem() {
                            run_ids.insert(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        
        Ok(run_ids.into_iter().collect())
    }

    fn event_file_path(&self, run_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.jsonl", run_id))
    }
}

/// 内存事件存储（用于测试）
#[derive(Default)]
pub struct InMemoryEventStore {
    events: std::sync::Mutex<Vec<GovernanceEvent>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
    }
}

impl EventStore for InMemoryEventStore {
    fn append_event(&self, event: &GovernanceEvent) -> Result<()> {
        if let Ok(mut events) = self.events.lock() {
            events.push(event.clone());
        }
        Ok(())
    }

    fn load_run_events(&self, run_id: &str) -> Result<Vec<GovernanceEvent>> {
        if let Ok(events) = self.events.lock() {
            Ok(events
                .iter()
                .filter(|e| e.run_id == run_id)
                .cloned()
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn list_run_ids(&self) -> Result<Vec<String>> {
        use std::collections::HashSet;
        if let Ok(events) = self.events.lock() {
            let run_ids: HashSet<String> = events
                .iter()
                .map(|e| e.run_id.clone())
                .collect();
            Ok(run_ids.into_iter().collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn event_file_path(&self, _run_id: &str) -> PathBuf {
        PathBuf::from("memory://events")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::event::{GovernanceEventType, Severity, EventScope};
    use tempfile::TempDir;

    #[test]
    fn jsonl_storage_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let store = JsonlEventStore::new(temp_dir.path()).unwrap();

        let event1 = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run started")
            .with_scope(EventScope::OperatorVisible);
        
        let event2 = GovernanceEvent::new("run-001", GovernanceEventType::SeatStarted, "Seat A started")
            .with_severity(Severity::Info);

        // 追加事件
        store.append_event(&event1).unwrap();
        store.append_event(&event2).unwrap();

        // 读取事件
        let loaded = store.load_run_events("run-001").unwrap();
        
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].event_type, GovernanceEventType::RunCreated);
        assert_eq!(loaded[1].event_type, GovernanceEventType::SeatStarted);
    }

    #[test]
    fn separate_runs() {
        let temp_dir = TempDir::new().unwrap();
        let store = JsonlEventStore::new(temp_dir.path()).unwrap();

        let event1 = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run 1");
        let event2 = GovernanceEvent::new("run-002", GovernanceEventType::RunCreated, "Run 2");

        store.append_event(&event1).unwrap();
        store.append_event(&event2).unwrap();

        let run1_events = store.load_run_events("run-001").unwrap();
        let run2_events = store.load_run_events("run-002").unwrap();

        assert_eq!(run1_events.len(), 1);
        assert_eq!(run2_events.len(), 1);
        assert_eq!(run1_events[0].run_id, "run-001");
        assert_eq!(run2_events[0].run_id, "run-002");
    }

    #[test]
    fn event_count() {
        let temp_dir = TempDir::new().unwrap();
        let store = JsonlEventStore::new(temp_dir.path()).unwrap();

        assert_eq!(store.event_count("run-001").unwrap(), 0);

        store.append_event(&GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run")).unwrap();
        store.append_event(&GovernanceEvent::new("run-001", GovernanceEventType::SeatStarted, "Seat")).unwrap();

        assert_eq!(store.event_count("run-001").unwrap(), 2);
    }

    #[test]
    fn file_contains_comments() {
        let temp_dir = TempDir::new().unwrap();
        let store = JsonlEventStore::new(temp_dir.path()).unwrap();

        let event = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run");
        store.append_event(&event).unwrap();

        let path = store.event_file_path("run-001");
        let content = std::fs::read_to_string(path).unwrap();
        
        // 检查文件头注释
        assert!(content.contains("# DragonCore Internal Broadcast Layer"));
        assert!(content.contains("# Run ID: run-001"));
    }

    #[test]
    fn in_memory_store() {
        let store = InMemoryEventStore::new();

        let event = GovernanceEvent::new("run-001", GovernanceEventType::RunCreated, "Run");
        store.append_event(&event).unwrap();

        let loaded = store.load_run_events("run-001").unwrap();
        assert_eq!(loaded.len(), 1);
    }
}
