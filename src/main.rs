use axi::core::{genesis::GenesisBlock, burn::Burn};
use axi::bridge::timelock::TimeLock;
use axi::governance::{JsonlEventStore, EventStore, RunProjection, RiskStatus, EventScope};
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;

/// 获取默认事件存储目录
fn default_event_dir() -> PathBuf {
    PathBuf::from("runtime_state/events")
}

/// 创建事件存储实例
fn create_store() -> anyhow::Result<Arc<dyn EventStore>> {
    let store = JsonlEventStore::new(default_event_dir())?;
    Ok(Arc::new(store))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("genesis") => {
            let block = GenesisBlock::new();
            println!("Genesis Block:");
            println!("  Hash: {}", block.hash);
            println!("  Constitution: {}", block.constitution_hash);
            println!("  Power Anchor: {} kWh", block.anchor_power);
            println!("  Compute Anchor: {} TFLOPs", block.anchor_compute);
        }
        
        Some("status") => {
            let now = Utc::now().timestamp() as u64;
            let state = TimeLock::check(now);
            let days = TimeLock::days_until_independence(now);
            
            match state {
                axi::bridge::timelock::BridgeState::DualTrack => {
                    println!("Status: Dual-Track (Fiat allowed)");
                    println!("Days until Independence: {}", days);
                }
                axi::bridge::timelock::BridgeState::Sovereign => {
                    println!("Status: Sovereign (Physical anchor only)");
                }
            }
        }
        
        Some("wallet") => {
            use axi::wallet::key::KeyPair;
            let kp = KeyPair::generate();
            println!("New Wallet:");
            println!("  Address: {}", kp.address_string());
        }
        
        Some("burn-check") => {
            let now = Utc::now().timestamp() as u64;
            let last_move = now - (4 * 365 * 24 * 3600);
            let balance = 1000;
            
            let burn = Burn::calculate_burn(last_move, now, balance);
            println!("Balance: {} AXI", balance);
            println!("Dormant: 4 years");
            println!("Burn amount: {} AXI", burn);
            println!("Remaining: {} AXI", balance - burn);
        }
        
        Some("mint") => {
            println!("⚡ Genesis Mint: 13280 AXI");
            println!("  To: 0xf743080f5a30d59dd6167b4707280b9e1e300b8ca891689d496cba22882d2893");
            println!("  Proof: 1000kWh + 3280TFLOPs");
            println!("  Status: CONFIRMED");
            println!("  Note: This is a genesis record. No actual UTXO implemented yet.");
        }

        Some("runs") => {
            if let Err(e) = handle_runs_command(&args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        Some("watch") => {
            if let Err(e) = handle_watch_command(&args) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        
        _ => {
            println!("⚡ AXI Node v0.1.0");
            println!();
            println!("Usage:");
            println!("  axi genesis              - Create genesis block");
            println!("  axi status               - Check Independence Day countdown");
            println!("  axi wallet               - Generate new wallet");
            println!("  axi burn-check           - Test halflife burn mechanism");
            println!("  axi mint                 - Genesis mint record");
            println!();
            println!("DIBL Governance:");
            println!("  axi runs                 - List all run IDs");
            println!("  axi runs --summary <id>  - Show run summary");
            println!("  axi watch                - Watch all operator-visible events");
            println!("  axi watch --run <id>     - Watch specific run events");
        }
    }
}

/// 处理 runs 命令
fn handle_runs_command(args: &[String]) -> anyhow::Result<()> {
    let store = create_store()?;
    
    // 解析 --summary 参数
    let summary_idx = args.iter().position(|a| a == "--summary" || a == "-s");
    let run_id = summary_idx.and_then(|idx| args.get(idx + 1));
    
    if let Some(run_id) = run_id {
        // 显示特定 run 的摘要
        let events = store.load_run_events(run_id)?;
        
        if events.is_empty() {
            anyhow::bail!("Run '{}' not found", run_id);
        }
        
        // 构建投影 - 只使用 operator_visible 事件
        let visible_events: Vec<_> = events
            .iter()
            .filter(|e| e.scope.is_operator_visible())
            .cloned()
            .collect();
        
        let projection = RunProjection::from_events(run_id, &visible_events);
        
        // 风险状态指示器
        let risk_indicator = match projection.risk_status() {
            RiskStatus::Normal => "✓",
            RiskStatus::Warning => "⚠",
            RiskStatus::Critical => "✗",
        };
        
        println!("Run Summary: {}", run_id);
        println!("═══════════════════════════════════════");
        println!("  Phase:        {}", projection.current_phase);
        println!("  Current Seat: {}", projection.current_seat.as_deref().unwrap_or("None"));
        println!("  Risk Status:  {} {:?}", risk_indicator, projection.risk_status());
        println!("  Open Risks:   {}", projection.open_risks.len());
        for risk in &projection.open_risks {
            println!("    - {}", risk);
        }
        println!("  Veto Count:   {}", projection.veto_count);
        println!("  Elapsed:      {:?}", projection.elapsed_time);
        if let Some(ref outcome) = projection.final_outcome {
            println!("  Outcome:      {}", outcome);
        }
        println!("  Last Event:   {}", 
            projection.last_event_time.map(|t: chrono::DateTime<Utc>| t.to_rfc3339())
                .unwrap_or_else(|| "N/A".to_string()));
        println!();
        println!("  Total Events: {} ({} operator-visible)", 
            events.len(), visible_events.len());
        
    } else {
        // 列出所有 run IDs
        let run_ids = store.list_run_ids()?;
        
        if run_ids.is_empty() {
            println!("No runs found.");
            println!("Events directory: {}", default_event_dir().display());
        } else {
            println!("Available runs:");
            println!("═══════════════════════════════════════");
            for run_id in run_ids {
                // 尝试获取每个 run 的基本信息
                match store.load_run_events(&run_id) {
                    Ok(events) if !events.is_empty() => {
                        let first = events.first().unwrap();
                        let last = events.last().unwrap();
                        println!("  {}  ({} events, {} → {})", 
                            run_id, 
                            events.len(),
                            event_type_name(&first.event_type),
                            event_type_name(&last.event_type)
                        );
                    }
                    _ => println!("  {}", run_id),
                }
            }
            println!();
            println!("Use 'axi runs --summary <run_id>' for details.");
        }
    }
    
    Ok(())
}

/// 将事件类型转换为简短名称
fn event_type_name(event_type: &axi::governance::GovernanceEventType) -> &'static str {
    use axi::governance::GovernanceEventType::*;
    match event_type {
        RunCreated => "Created",
        SeatStarted => "SeatStart",
        SeatCompleted => "SeatDone",
        RiskRaised => "Risk",
        VetoExercised => "Veto",
        FinalGateOpened => "FinalGate",
        DecisionCommitted => "Committed",
        RollbackTriggered => "Rollback",
        ArchiveCompleted => "Archived",
        TerminateTriggered => "Terminated",
    }
}

/// 处理 watch 命令
fn handle_watch_command(args: &[String]) -> anyhow::Result<()> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc as StdArc;
    
    let store = create_store()?;
    
    // 解析 --run 参数
    let run_idx = args.iter().position(|a| a == "--run" || a == "-r");
    let run_id_filter = run_idx.and_then(|idx| args.get(idx + 1).cloned());
    
    // 设置 Ctrl+C 处理
    let running = StdArc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\n[Watch stopped]");
    }).expect("Error setting Ctrl-C handler");
    
    println!("Watching for events... (Press Ctrl+C to stop)");
    println!("Filter: operator_visible events only");
    if let Some(ref run_id) = run_id_filter {
        println!("Run: {}", run_id);
    }
    println!("═══════════════════════════════════════");
    
    // 记录已显示的事件 ID，避免重复
    use std::collections::HashSet;
    let mut seen_events: HashSet<String> = HashSet::new();
    
    // 轮询间隔
    let poll_interval = std::time::Duration::from_millis(500);
    
    while running.load(Ordering::SeqCst) {
        let run_ids = if let Some(ref run_id) = run_id_filter {
            vec![run_id.clone()]
        } else {
            match store.list_run_ids() {
                Ok(ids) => ids,
                Err(e) => {
                    eprintln!("Warning: Failed to list runs: {}", e);
                    Vec::new()
                }
            }
        };
        
        for run_id in run_ids {
            match store.load_run_events(&run_id) {
                Ok(events) => {
                    for event in events {
                        // 只显示 operator_visible 事件
                        if event.scope == EventScope::Internal {
                            continue;
                        }
                        
                        // 去重：基于 event_id
                        let event_key = format!("{}:{}", run_id, event.event_id);
                        if seen_events.contains(&event_key) {
                            continue;
                        }
                        seen_events.insert(event_key);
                        
                        // 格式化输出
                        let seat_info = event.seat_id.as_ref()
                            .map(|s| format!(" [{}]", s))
                            .unwrap_or_default();
                        
                        println!("[{}] {} | {}{} | {:?} | {}",
                            event.created_at.format("%H:%M:%S"),
                            run_id,
                            event_type_name(&event.event_type),
                            seat_info,
                            event.severity,
                            event.summary
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load run {}: {}", run_id, e);
                }
            }
        }
        
        std::thread::sleep(poll_interval);
    }
    
    Ok(())
}
