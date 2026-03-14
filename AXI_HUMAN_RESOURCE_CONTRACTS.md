# AXI Human Resource Contract Protocol v0.2.0 (Draft)

> **Status**: Draft for Review  
> **Version**: v0.2.0-draft.1  
> **Date**: 2026-03-14  
> **Depends**: AXI Agent Protocol v0.2.0  
> **Precedence**: AXI Constitution v1.0 > Agent Protocol > This Protocol

---

## 1. Purpose

本协议定义人类参与者将其闲置设备资源（算力、存储、电力）接入 AXI 网络的合同框架、验证机制、安全边界与结算规则。

**核心命题**：
- 让人类愿意主动贡献闲置资源（Mac、手机、服务器、能源设备）
- 通过 AXI 结算获得公平回报
- 确保本地可控、风险边界清楚、退出容易
- 设备级沙箱隔离，保护用户隐私与数据安全
- AXI 作为唯一的价值结算与争议仲裁层

**与其他协议的关系**：
- 资源供应通过 Task Contract 被消费（Agent Protocol）
- 资源优化经验可以封装为 Experience Capsule
- Human Provider 可以是 Agent 也可以是独立人类

---

## 2. Entities

### 2.1 Human Provider

```yaml
entity: HumanProvider
attributes:
  provider_id:      # did:axi:human:<hash>
  axi_address:      # AXI 主钱包地址
  identity_proof:   # KYC 或社交证明（可选）
  jurisdiction:     # 司法管辖区
  privacy_level:    # minimal | standard | strict
  
  reputation:
    score: 0-10000
    contracts_completed: int
    contracts_failed: int
    total_resources_provided: dict
    dispute_resolution_rate: float
  
  devices: [Device]
  active_contracts: [ResourceContract]
```

### 2.2 Device

```yaml
entity: Device
attributes:
  device_id:        # 全局唯一设备标识
  provider_id:      # 所属 Provider
  device_type:      # mac | iphone | android | linux_server | windows | nas | power_unit
  hardware_fingerprint: # TEE 或硬件哈希
  
  specs:
    cpu: {cores, architecture, frequency}
    gpu: {model, vram, cuda_version, metal_support}
    npu: {type, tops}
    memory: {total_gb, available_gb}
    storage: {total_gb, available_gb, type}
    network: {bandwidth_mbps, latency_ms}
    power: {source, battery_capacity_wh, output_watts}
  
  capabilities: [Capability]
  attestation: AttestationRecord
  sandbox_config: SandboxConfig
  
  status: enrolled | active | paused | offline | revoked
  health_score: 0-100
```

### 2.3 Resource Contract

```yaml
entity: ResourceContract
attributes:
  contract_id:      # 唯一合同ID
  provider_id:      # HumanProvider ID
  device_id:        # Device ID
  
  resource_type:    # compute | storage | power
  contract_subtype: # 子类型（见3.3）
  
  terms:
    availability_window: {start_hour, end_hour, timezone}
    min_duration_seconds: int
    max_concurrent_tasks: int
    
  pricing:
    model: fixed | per_use | per_hour | per_kwh
    rate_axi: float
    collateral_axi: float
    
  quality_guarantee:
    min_uptime_percent: float
    max_latency_ms: int
    penalty_per_violation: float
    
  status: pending | active | paused | terminated | slashed
  created_at: timestamp
  expires_at: timestamp
```

### 2.4 Resource Consumer (Agent)

```yaml
entity: ResourceConsumer
inherits: Agent
additional_attributes:
  resource_requirements: # 资源需求规格
  task_history: [TaskExecution]
  payment_reliability: Score
```

---

## 3. Object Schema

### 3.1 Resource Type Taxonomy

#### A. Compute Contract

```yaml
compute_subtypes:
  inference_slot:
    description: "提供 AI 模型推理 slot"
    resource_unit: "per_1k_tokens" | "per_request"
    requirements:
      - gpu_vram_min_gb
      - max_latency_ms
      - model_support_list
    
  training_worker:
    description: "参与分布式训练"
    resource_unit: "per_tflops" | "per_hour"
    requirements:
      - gpu_compute_capability
      - gradient_sync_bandwidth
      - checkpoint_storage_gb
    
  batch_processor:
    description: "批量数据处理"
    resource_unit: "per_task" | "per_gb_processed"
    requirements:
      - cpu_cores_available
      - memory_available_gb
      - io_throughput_mbps
    
  compilation_worker:
    description: "代码编译/优化"
    resource_unit: "per_build" | "per_hour"
    requirements:
      - cpu_performance_score
      - disk_io_speed
      - cache_size_gb
```

#### B. Storage Contract

```yaml
storage_subtypes:
  hot_cache:
    description: "高频访问缓存层"
    resource_unit: "per_gb_month"
    requirements:
      - availability_99_9
      - retrieval_latency_ms < 100
      - redundancy_2x
    
  warm_storage:
    description: "中等频次存储"
    resource_unit: "per_gb_month"
    requirements:
      - availability_99_5
      - retrieval_latency_ms < 1000
      - redundancy_1x
    
  cold_archive:
    description: "冷数据归档"
    resource_unit: "per_gb_month"
    requirements:
      - availability_95
      - retrieval_time_hours < 24
      - proof_of_possession_required
    
  backup_node:
    description: "去中心化备份节点"
    resource_unit: "per_gb_month"
    requirements:
      - erasure_coding_participation
      - repair_bandwidth_available
      - geographic_diversity
```

#### C. Power Contract

```yaml
power_subtypes:
  grid_feed:
    description: "向电网输入清洁能源"
    resource_unit: "per_kwh"
    requirements:
      - smart_meter_attestation
      - grid_connection_certified
      - carbon_intensity < threshold
    
  battery_buffer:
    description: "提供电池缓冲容量"
    resource_unit: "per_kwh_stored"
    requirements:
      - battery_capacity_kwh
      - charge_discharge_cycles_remaining
      - response_time_ms
    
  computation_colocation:
    description: "为计算设备提供场地+电力"
    resource_unit: "per_rack_kwh"
    requirements:
      - physical_security
      - cooling_capacity_kw
      - network_uptime_99_9
```

### 3.2 Device Enrollment Schema

```json
{
  "@context": "https://axi.network/resource/v0",
  "type": "DeviceEnrollment",
  "enrollment_id": "enroll:a1b2c3...",
  "provider_id": "did:axi:human:abc123...",
  "device": {
    "type": "mac",
    "model": "MacBook Pro M3 Max",
    "hardware_fingerprint": "sha256:...",
    "tee_attestation": {
      "type": "apple_secure_enclave",
      "attestation_data": "base64...",
      "cert_chain_cid": "Qm..."
    }
  },
  "capabilities": {
    "compute": {
      "available": true,
      "cpu": {
        "cores": 14,
        "performance_cores": 10,
        "efficiency_cores": 4,
        "arch": "arm64"
      },
      "gpu": {
        "model": "Apple M3 Max",
        "unified_memory_gb": 36,
        "metal_supported": true,
        "estimated_tflops_fp16": 30
      },
      "npu": {
        "neural_engine_cores": 16,
        "tops": 18
      }
    },
    "storage": {
      "available": true,
      "total_gb": 1024,
      "available_gb": 500,
      "type": "ssd",
      "encryption": "filevault"
    },
    "power": {
      "available": false,
      "reason": "battery_only_no_grid_feed"
    }
  },
  "sandbox_config": {
    "type": "docker" | "podman" | "vm" | "process",
    "resource_limits": {
      "max_cpu_percent": 50,
      "max_memory_gb": 16,
      "max_storage_gb": 100,
      "max_network_mbps": 100
    },
    "isolation_level": "container" | "vm",
    "persistent_storage_allowed": false,
    "network_access": "outbound_only"
  },
  "availability_schedule": {
    "timezone": "Asia/Shanghai",
    "windows": [
      {"day": "mon-fri", "start": "22:00", "end": "08:00"},
      {"day": "sat-sun", "start": "00:00", "end": "23:59"}
    ]
  },
  "pricing_preferences": {
    "minimum_rate_axi_per_hour": 0.5,
    "accepts_spot_pricing": true,
    "discount_for_long_term": 0.1
  },
  "privacy_settings": {
    "allow_telemetry": true,
    "allow_benchmarking": true,
    "data_retention_days": 30,
    "audit_log_access": "provider_only"
  },
  "enrollment_proof": "0x...",
  "timestamp": 1709256878
}
```

### 3.3 Resource Contract Schema

```json
{
  "@context": "https://axi.network/resource/v0",
  "type": "ResourceContract",
  "contract_id": "rc:xyz789...",
  "provider_id": "did:axi:human:abc123...",
  "device_id": "dev:mac001...",
  "resource_type": "compute",
  "contract_subtype": "inference_slot",
  "terms": {
    "availability": {
      "schedule": "as_enrolled",
      "minimum_notice_seconds": 300,
      "max_concurrent_slots": 2
    },
    "performance_guarantee": {
      "min_tflops_fp16": 25,
      "max_latency_ms": 500,
      "availability_uptime_percent": 95
    },
    "sandbox": {
      "type": "docker",
      "image_whitelist": ["axi/inference-runtime:v1"],
      "resource_limits": {
        "cpu_cores": 8,
        "memory_gb": 16,
        "gpu_percent": 80
      }
    }
  },
  "pricing": {
    "model": "per_use",
    "base_rate_axi_per_1k_tokens": 0.01,
    "spot_discount": 0.2,
    "minimum_charge_axi": 0.001,
    "collateral_axi": 100
  },
  "verification": {
    "benchmark_required": true,
    "attestation_frequency": "weekly",
    "proof_submission": "per_task"
  },
  "penalties": {
    "missed_deadline": "10% of collateral",
    "failed_verification": "25% of collateral",
    "extended_downtime": "50% of collateral + contract_termination"
  },
  "consumer_protection": {
    "result_verification": "tee_attestation",
    "privacy_guarantee": "no_data_retention",
    "timeout_seconds": 60
  },
  "exit_terms": {
    "notice_period_seconds": 86400,
    "immediate_pause_available": true,
    "data_deletion_on_exit": true,
    "pending_task_handoff": "graceful_shutdown_30s"
  },
  "status": "active",
  "created_at": 1709256878,
  "expires_at": 1711840878,
  "signature": "0x..."
}
```

### 3.4 Attestation Record Schema

```json
{
  "type": "AttestationRecord",
  "attestation_id": "att:def456...",
  "device_id": "dev:mac001...",
  "timestamp": 1709256878,
  "tee_type": "apple_secure_enclave",
  "measurements": {
    "boot_hash": "sha256:...",
    "kernel_hash": "sha256:...",
    "sandbox_config_hash": "sha256:...",
    "hardware_config_hash": "sha256:..."
  },
  "benchmark_results": {
    "compute": {
      "tflops_fp32": 15.2,
      "tflops_fp16": 28.5,
      "inference_latency_ms": 450,
      "benchmark_suite_version": "axi-bench-v1"
    },
    "storage": {
      "sequential_read_mbps": 5500,
      "sequential_write_mbps": 4800,
      "random_read_iops": 850000,
      "random_write_iops": 720000
    }
  },
  "quote": "base64_attestation_quote...",
  "signature": "0x..."
}
```

---

## 4. Lifecycle

### 4.1 Device Enrollment Flow

```
[Provider] → submits_enrollment → [AXI Node]
                ↓
        [Attestation Check]
                ↓
        [Benchmark Execution]
                ↓
        [Capability Scoring]
                ↓
    ┌──────────┴──────────┐
    ↓                     ↓
[Approved]            [Rejected]
    ↓                     ↓
[Collateral Lock]    [Reason Logged]
    ↓
[Device Active]
```

### 4.2 Resource Contract Lifecycle

```yaml
lifecycle:
  created:
    trigger: provider_publishes_offer
    action:
      - validate_device_status
      - lock_collateral
      - index_for_discovery
  
  matched:
    trigger: consumer_accepts_offer
    action:
      - create_escrow
      - reserve_capacity
      - notify_provider
  
  executing:
    trigger: task_assigned
    action:
      - sandbox_initialization
      - task_execution
      - proof_generation
  
  verifying:
    trigger: task_completed
    action:
      - result_verification
      - attestation_check
      - consumer_acknowledgment
  
  settled:
    trigger: verification_passed
    action:
      - release_payment
      - update_reputation
      - release_collateral_hold
  
  paused:
    trigger: provider_request OR safety_alert
    action:
      - stop_new_tasks
      - complete_in_progress
      - maintain_collateral
  
  terminated:
    trigger: expiration OR violation OR provider_exit
    action:
      - settle_pending
      - release_collateral (if no_penalty)
      - archive_contract
```

### 4.3 Task Execution Flow

```yaml
task_execution:
  step_1_assignment:
    consumer: selects_resource_contract
    check:
      - device_available
      - provider_reputation > threshold
      - collateral_sufficient
    action: lock_payment_in_escrow
  
  step_2_sandbox_init:
    provider_device:
      - download_container_image (if needed)
      - initialize_sandbox
      - verify_isolation
    attestations:
      - sandbox_config_hash
      - network_isolation_verified
  
  step_3_execution:
    sandbox:
      - mount_input_data (read_only)
      - execute_task
      - capture_output
      - generate_execution_log
    monitoring:
      - resource_usage_tracking
      - timeout_enforcement
      - anomaly_detection
  
  step_4_result_delivery:
    output:
      - result_cid
      - execution_log_cid
      - tee_attestation
      - performance_metrics
    delivery: encrypted_to_consumer
  
  step_5_verification:
    methods:
      - deterministic_replay (if applicable)
      - tee_quote_verification
      - output_sanity_check
      - consumer_test_execution
  
  step_6_settlement:
    success:
      - payment_to_provider
      - collateral_released
      - reputation_updated
    failure:
      - escrow_refund_to_consumer
      - penalty_from_collateral
      - reputation_penalty
```

---

## 5. Verification Rules

### 5.1 Attestation Requirements

```yaml
attestation_schedule:
  initial_enrollment:
    required: true
    method: tee_quote + benchmark
  
  periodic_re_attestation:
    frequency: weekly
    grace_period: 48_hours
    penalty_for_missing: warning → pause → slash
  
  on_demand:
    trigger: consumer_request OR dispute
    cost: borne_by_requester (if frivolous)

attestation_methods:
  apple_secure_enclave:
    supported: [mac, iphone, ipad]
    verification: apple_cert_chain
  
  intel_tdx:
    supported: [linux_server, windows]
    verification: intel_pcs
  
  amd_sev:
    supported: [linux_server]
    verification: amd_kds
  
  arm_trustzone:
    supported: [android]
    verification: device_oem_cert
  
  software_attestation:
    supported: [all]
    trust_level: low
    requires: additional_collateral
```

### 5.2 Benchmark Requirements

```yaml
benchmark_suite:
  compute:
    - matmul_benchmark (fp32, fp16, int8)
    - inference_benchmark (standard_models)
    - memory_bandwidth_test
    
  storage:
    - sequential_read_write
    - random_read_write
    - latency_under_load
    
  network:
    - bandwidth_test
    - latency_to_regions
    - jitter_measurement

benchmark_rules:
  initial: full_suite_required
  periodic: subset_with_random_full
  on_dispute: full_suite_mandatory
  
  result_tolerance:
    - within_10%: accepted
    - within_20%: flagged_for_review
    - beyond_20%: contract_suspended
```

### 5.3 Proof Submission

```yaml
proof_types:
  compute:
    tee_execution_quote: "TEE attestation of execution"
    result_hash: "SHA256 of output"
    performance_log: "Resource usage trace"
    
  storage:
    proof_of_possession: "Cryptographic proof"
    periodic_challenge: "Response to random challenge"
    retrieval_proof: "Latency + integrity proof"
    
  power:
    smart_meter_signature: "Signed meter reading"
    grid_telemetry: "Grid operator confirmation"
    timestamp: "Tamper-evident timestamp"

verification_threshold:
  compute: 2_of_3_proofs_required
  storage: 3_of_4_challenges_passed
  power: grid_operator_attestation_required
```

---

## 6. Settlement Rules

### 6.1 Pricing Models

| 资源类型 | 模型 | 计费单位 | 示例 |
|----------|------|----------|------|
| Compute - Inference | per_use | per 1k tokens | 0.01 AXI |
| Compute - Training | per_hour | per GPU hour | 2 AXI |
| Compute - Batch | per_task | per job | 0.5 AXI |
| Storage - Hot | per_time | per GB-month | 0.1 AXI |
| Storage - Retrieval | per_use | per GB retrieved | 0.01 AXI |
| Power - Grid | per_energy | per kWh | 0.05 AXI |
| Power - Battery | per_capacity | per kWh stored/day | 0.02 AXI |

### 6.2 Collateral Requirements

```yaml
collateral_calculation:
  base_collateral:
    compute: 100 AXI
    storage: 200 AXI
    power: 500 AXI
  
  multiplier:
    reputation_factor:
      score > 8000: 0.5x
      score 5000-8000: 1.0x
      score 3000-5000: 2.0x
      score < 3000: not_allowed
    
    history_factor:
      contracts_completed > 100: 0.8x
      contracts_completed > 10: 1.0x
      contracts_completed < 10: 1.5x
    
    hardware_factor:
      tee_attested: 0.8x
      software_only: 1.5x
  
  max_collateral: 5000 AXI
  min_collateral: 50 AXI
```

### 6.3 Payment Flow

```yaml
payment_timing:
  spot_contracts:
    - payment_on_assignment (escrow)
    - release_on_verification
    
  subscription_contracts:
    - upfront_payment (monthly)
    - daily_prorated_settlement
    
  milestone_contracts:
    - payment_per_milestone
    - holdback_until_final

payment_distribution:
  provider_share: 90%
  platform_fee: 5%
  insurance_pool: 3%
  burn: 0.5%
  verifier_reward: 1.5%
```

### 6.4 Penalty Structure

| 违规类型 | 首次 | 重复 | 严重 |
|----------|------|------|------|
|  missed_deadline | 警告 | 10% collateral | 25% + 暂停 |
|  failed_verification | 25% collateral | 50% collateral | 100% + 封禁 |
|  false_attestation | 50% collateral | 100% collateral | 永久封禁 |
|  extended_downtime | 5%/day | 10%/day | 合同终止 |
|  data_breach | 100% collateral | 永久封禁 | 法律追究 |

### 6.5 Insurance Pool

```yaml
insurance_pool:
  source: 3% of all payments
  purpose:
    - compensate_consumers_for_provider_failures
    - cover_verification_costs
    - emergency_payouts
  
  claim_process:
    - consumer_files_claim
    - verifier_investigates
    - payout_if_verified
  
  limits:
    max_payout_per_incident: 1000 AXI
    max_payout_per_consumer_per_month: 5000 AXI
```

---

## 7. Dispute / Slash Rules

### 7.1 Dispute Categories

```yaml
dispute_types:
  performance_dispute:
    description: "实际性能不符合合同"
    examples: ["slower_than_promised", "frequent_timeouts"]
    evidence: benchmark_logs, performance_metrics
    
  availability_dispute:
    description: "可用性不符合承诺"
    examples: ["unplanned_downtime", "missed_windows"]
    evidence: uptime_logs, attestation_gaps
    
  result_dispute:
    description: "计算结果不正确"
    examples: ["wrong_inference_output", "corrupted_storage"]
    evidence: reproducibility_test, tee_logs
    
  privacy_dispute:
    description: "违反隐私承诺"
    examples: ["data_retention_violation", "sandbox_escape"]
    evidence: audit_logs, forensic_analysis
    
  payment_dispute:
    description: "支付相关争议"
    examples: ["underpayment", "wrong_rate_applied"]
    evidence: contract_terms, transaction_logs
```

### 7.2 Dispute Resolution Flow

```yaml
dispute_resolution:
  step_1_filing:
    window: 7_days_after_incident
    stake: 50 AXI (returned_if_upheld)
    evidence: required_cid
    
  step_2_mediation:
    duration: 48_hours
    parties: consumer + provider
    facilitator: automated_system
    outcome: mutual_agreement OR escalation
    
  step_3_arbitration:
    duration: 7_days
    panel: 3_verifiers
    evidence_review: public_to_panel
    decision: majority_vote
    
  step_4_appeal:
    window: 24_hours
    cost: 200 AXI
    panel: genesis_committee
    binding: true
```

### 7.3 Slash Conditions

| 条件 | Slash | Reputation | 其他 |
|------|-------|------------|------|
| Failed attestation | 10% | -100 | 暂停至修复 |
| Missed 3 deadlines | 25% | -300 | 强制冷却期 |
| Verification failed | 50% | -500 | 审查期 |
| False TEE quote | 100% | -2000 | 永久封禁 |
| Privacy violation | 100% | -5000 | 永久封禁+法律 |
| Sybil attack | 100% | -10000 | 永久封禁+关联账户 |

### 7.4 Provider Protection

```yaml
provider_protection:
  consumer_fraud:
    detection: pattern_analysis
    protection: escrow_release_requires_verification
    remedy: reputation_restore_if_vindicated
    
  benchmark_manipulation:
    detection: statistical_outlier_detection
    protection: multiple_verifier_benchmarks
    remedy: median_score_used
    
  force_majeure:
    covered: [power_outage, network_failure, hardware_failure]
    not_covered: [intentional_shutdown, resource_reallocation]
    process: attestation_of_circumstance
    remedy: penalty_waiver, no_reputation_loss
```

---

## 8. Privacy / Safety Constraints

### 8.1 Sandbox Boundary

```yaml
sandbox_specification:
  container_level:
    process_isolation: pid_namespace
    filesystem_isolation: overlay_fs
    network_isolation: veth_pair + iptables
    resource_limits: cgroups
    
  vm_level:
    hypervisor: kvm | hyper-v | parallels
    secure_boot: required
    measured_boot: required
    encrypted_memory: required_if_available
    
  process_level:
    seccomp: strict_filter
    apparmor: enforced_profile
    capability_dropping: all_except_necessary
    no_new_privileges: true

mandatory_restrictions:
  - no_host_filesystem_access
  - no_host_network_services
  - no_raw_devices
  - no_kernel_module_loading
  - no_ptrace
  - no_setuid_executables
```

### 8.2 Local Data Isolation

```yaml
data_isolation:
  input_data:
    source: consumer_provided_cid
    access: read_only
    lifetime: task_duration_only
    cleanup: secure_erase
    
  output_data:
    destination: encrypted_to_consumer
    retention: zero_on_device
    backup: none
    
  execution_logs:
    content: performance_only_no_data
    retention: 30_days_max
    access: provider_audit_only
    
  prohibited_access:
    - user_documents
    - browser_data
    - system_passwords
    - ssh_keys
    - wallet_files
    - personal_photos

isolation_verification:
  method: tee_attestation + file_system_audit
  frequency: per_task
  failure_action: immediate_termination + alert
```

### 8.3 One-Click Pause / Exit

```yaml
provider_controls:
  pause:
    trigger: one_click_in_dashboard OR api_call
    effect:
      - no_new_tasks_assigned
      - in_progress_tasks_complete_gracefully
      - contract_suspended_not_terminated
    resume: any_time_without_penalty
    cooldown: 5_minutes_between_pause_resume
    
  immediate_exit:
    trigger: emergency_button
    effect:
      - all_tasks_terminated
      - in_progress_tasks_compensated
      - contract_terminated
    collateral: returned_after_7_days_settlement_period
    
  graceful_exit:
    trigger: notice_given
    notice_period: 24_hours
    effect:
      - no_new_tasks_after_notice
      - in_progress_tasks_complete
      - contract_terminated_after_completion
    collateral: returned_immediately

consumer_impact:
  pause: tasks_rerouted_to_alternative_providers
  exit: graceful_handoff_with_24h_notice OR immediate_with_penalty
```

### 8.4 Resource Usage Limits

```yaml
hard_limits:
  compute:
    max_cpu_percent: 80
    max_memory_percent: 80
    max_gpu_percent: 90
    max_duration_per_task: 24_hours
    
  storage:
    max_usage_percent: 50
    min_free_space_gb: 50
    max_file_size_gb: 10
    excluded_paths:
      - /home/*/Documents
      - /home/*/Desktop
      - /home/*/Pictures
      - /Users/*
      - C:\Users\*
    
  network:
    max_bandwidth_mbps: 100
    max_monthly_transfer_tb: 1
    blocked_ports: [22, 3389, 5900]
    allowed_protocols: [https, wss]
    
  power:
    max_battery_drain_percent: 30
    require_plugged_in_for_compute: true
    thermal_throttling_required: true

breach_handling:
  automatic: task_termination
  alert: provider_notified
  escalation: 3_breaches_in_24h → contract_suspension
```

### 8.5 Telemetry and Audit

```yaml
telemetry:
  collected:
    - resource_usage (cpu, memory, gpu, network)
    - task_execution_metrics
    - error_rates
    - availability_uptime
    
  not_collected:
    - input_data_content
    - output_data_content
    - execution_intermediate_states
    - consumer_identifiable_info
    
  retention: 30_days
  access: provider_only + aggregate_anonymous_to_axi
  opt_out: available_but_reduces_reputation_score

audit_logs:
  content:
    - contract_events
    - settlement_transactions
    - dispute_filings
    - attestation_records
    
  retention: permanent_on_chain
  access: public_for_verification
```

---

## 9. Open Questions

1. **Mobile Device Constraints**：手机/平板的电池和热管理如何优化？
   - 仅允许插电时参与？
   - 夜间充电时段优先？
   - 温度阈值自动暂停？

2. **Windows Support**：Windows 设备的 TEE 支持较弱，如何处理？
   - 提高 collateral 要求？
   - 限制为 storage-only？
   - 依赖第三方 attestation 服务？

3. **Geographic Distribution**：如何激励地理多样性？
   - latency-based bonus？
   - 稀缺地区 multiplier？
   - 网络拓扑优化奖励？

4. **Home Energy Devices**：家庭能源设备（太阳能、电池）的接入标准？
   - 智能电表兼容性矩阵
   - 电网监管合规要求
   - 实时定价 oracle

5. **Provider Insurance**：是否需要为 high-value providers 提供保险选项？
   - 设备损坏保险
   - 收入保障保险
   - 法律责任保险

6. **Sybil Resistance**：如何防止 single-human-multiple-device 攻击？
   - KYC 门槛？
   - 社交图谱分析？
   - 硬件 fingerprint 聚类？

7. **Consumer Privacy**：consumer 的任务内容对 provider 是否可见？
   - 全同态加密执行？
   - TEE 执行（consumer 信任 TEE）
   - 代码签名白名单

8. **Resource Preemption**：provider 本地需要资源时如何优雅抢占？
   - 任务 checkpoint 机制
   - migration 到替代设备
   - 优先级协商协议

---

## 10. v0 Scope / Out-of-Scope

### 10.1 In Scope (v0.2.0)

- [x] Compute / Storage / Power 三类合同定义
- [x] Device enrollment 流程
- [x] TEE attestation 框架
- [x] Benchmark suite 规范
- [x] Sandbox isolation 要求
- [x] Resource limits and boundaries
- [x] One-click pause / exit
- [x] Local data isolation rules
- [x] Collateral and penalty structure
- [x] Dispute resolution basic flow

### 10.2 Out of Scope (v0.2.0)

- [ ] Windows full TEE support
- [ ] Real-time power grid integration
- [ ] Mobile thermal management optimization
- [ ] Cross-device resource pooling
- [ ] Provider insurance products
- [ ] Consumer-side TEE verification
- [ ] Homomorphic encryption execution
- [ ] Autonomous resource pricing
- [ ] Physical hardware shipping integration

### 10.3 Future Versions

| 版本 | 计划功能 |
|------|----------|
| v0.3.0 | Mobile power optimization |
| v0.4.0 | Grid energy integration |
| v0.5.0 | Cross-device federation |
| v1.0.0 | Autonomous resource market |

---

## Appendix A: Device Support Matrix

| 设备类型 | 状态 | TEE 支持 | Compute | Storage | Power |
|----------|------|----------|---------|---------|-------|
| Mac (Apple Silicon) | ✅ P0 | Secure Enclave | ✅ Metal | ✅ FileVault | ❌ |
| Mac (Intel) | ⚠️ P1 | T2 Chip | ✅ | ✅ | ❌ |
| iPhone/iPad | ✅ P0 | Secure Enclave | ✅ Neural Engine | ✅ | ❌ |
| Android | ✅ P0 | TrustZone* | ✅ NPU | ✅ | ❌ |
| Linux Server | ✅ P0 | TDX/SEV | ✅ CUDA | ✅ | ⚠️ Grid* |
| Windows | ⚠️ P1 | Software** | ✅ CUDA | ✅ | ❌ |
| NAS | ✅ P1 | Software** | ❌ | ✅ | ❌ |
| Power Wall | 🔮 P2 | Smart Meter | ❌ | ❌ | ✅ |

* TrustZone 支持取决于 OEM 实现
** Software attestation 需要额外 collateral
* Grid integration 取决于地区监管

---

## Appendix B: Sandbox Configuration Example

```yaml
# Docker Compose for AXI Compute Sandbox
version: '3.8'

services:
  axi-sandbox:
    image: axi/runtime:v1
    
    # Resource Limits
    cpus: '8.0'
    mem_limit: 16g
    shm_size: 2g
    
    # GPU Access
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
    
    # Isolation
    security_opt:
      - no-new-privileges:true
      - seccomp:axi-seccomp-profile.json
      - apparmor:axi-docker-profile
    
    cap_drop:
      - ALL
    cap_add:
      - SYS_NICE  # For GPU priority only
    
    # Network
    network_mode: none
    
    # Storage
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m
    volumes:
      - type: bind
        source: /axi/inputs/${TASK_ID}
        target: /input
        read_only: true
        bind:
          propagation: rprivate
      - type: volume
        source: axi-output-${TASK_ID}
        target: /output
    
    # Monitoring
    logging:
      driver: none  # No logs with data
    
    # Lifecycle
    restart: "no"
    auto_remove: true
```

---

## Appendix C: Provider Dashboard API

```yaml
# Provider Control Endpoints

GET /provider/devices
  - list all enrolled devices
  - show status, health, earnings

POST /provider/device/{id}/pause
  - immediate pause
  - graceful shutdown of in-progress tasks

POST /provider/device/{id}/resume
  - resume accepting tasks

POST /provider/device/{id}/exit
  - initiate graceful exit
  - 24h notice period

POST /provider/device/{id}/exit-now
  - emergency exit
  - forfeit pending earnings

GET /provider/earnings
  - historical earnings
  - pending settlements
  - collateral status

PUT /provider/pricing
  - update pricing preferences
  - takes effect on next contract

GET /provider/privacy-report
  - audit log of data access
  - sandbox breach attempts (if any)
```

---

*协议草案版本: v0.2.0-draft.1*  
*最后更新: 2026-03-14*  
*依赖协议: AXI Agent Protocol v0.2.0*  
*下一审核日期: 2026-03-21*
