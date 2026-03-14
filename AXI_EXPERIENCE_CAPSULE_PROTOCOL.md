# AXI Experience Capsule Protocol v0.2.0 (Draft)

> **Status**: Draft for Review  
> **Version**: v0.2.0-draft.1  
> **Date**: 2026-03-14  
> **Depends**: AXI Agent Protocol v0.2.0  
> **Precedence**: AXI Constitution v1.0 > Agent Protocol > This Protocol

---

## 1. Purpose

本协议定义 Experience Capsule（经验胶囊）的创建、验证、交易、派生、结算全流程规范。

**核心命题**：
- 将 AI Agent 的 task 执行经验转化为可交易、可验证、可组合的知识资产
- 通过 lineage 追踪实现派生分润（attribution）
- 通过 privacy redaction 确保敏感信息不泄露
- AXI 作为胶囊定价、质押、结算的唯一价值层

**与其他协议的关系**：
- 经验胶囊的产生依赖于 Task Contract 的完成（Agent Protocol）
- 经验胶囊的消费可能触发新的 Task Contract
- 经验胶囊可以与 Resource Contract 结合（如特定硬件优化经验）

---

## 2. Entities

### 2.1 Experience Capsule

```yaml
entity: ExperienceCapsule
attributes:
  capsule_id:       # 全局唯一标识，格式: cap:<hash>
  type:             # capsule 类型（见第3章）
  domain:           # 领域标签列表
  creator:          # AgentID 或 HumanProviderID
  created_at:       # 创建时间戳
  lineage:          # 血统/溯源信息
  content_hash:     # 内容 IPFS CID
  evidence_hash:    # 证据包 CID
  quality_score:    # 质量评分 (0-10000)
  usage_count:      # 被消费次数
  revenue_axi:      # 累计收入 AXI
  price_axi:        # 当前定价
  status:           # draft | pending_review | published | deprecated | retracted
  license:          # 许可类型
```

### 2.2 Capsule Creator

```yaml
entity: CapsuleCreator
inherits: Agent | HumanProvider
additional_attributes:
  capsules_created: [CapsuleID]
  total_revenue_axi: Balance
  average_quality: Score
  specialization_domains: [Domain]
```

### 2.3 Capsule Consumer

```yaml
entity: CapsuleConsumer
inherits: Agent | HumanProvider
additional_attributes:
  capsules_consumed: [ConsumptionRecord]
  feedback_given: [Feedback]
  subscription_active: [CapsuleID]
```

### 2.4 Derivation Record

```yaml
entity: DerivationRecord
attributes:
  parent_capsule_id: CapsuleID    # 父胶囊
  child_capsule_id: CapsuleID     # 子胶囊
  derivation_type:                # 派生类型
  attribution_ratio:              # 分润比例 (basis points)
  derivation_proof:               # 派生证明 CID
```

---

## 3. Object Schema

### 3.1 Capsule Type Taxonomy

```yaml
capsule_types:
  TaskCapsule:
    description: "单一任务的成功执行经验"
    source: task_contract_completion
    content:
      - problem_formulation    # 问题定义方法
      - approach_selected      # 选择的方法
      - execution_steps        # 执行步骤
      - pitfalls_avoided       # 避开的坑
      - optimization_tricks    # 优化技巧
    evidence_required: true
    verifiability: high
    
  FailureCapsule:
    description: "失败任务的教训总结"
    source: task_contract_failure
    content:
      - failure_mode           # 失败模式分类
      - root_cause_analysis    # 根因分析
      - early_warning_signals  # 早期预警信号
      - recovery_attempts      # 恢复尝试
      - lessons_learned        # 教训总结
    evidence_required: true
    verifiability: medium
    
  DomainCapsule:
    description: "特定领域的通用知识包"
    source: aggregated_experience | human_expertise
    content:
      - domain_principles      # 领域原理
      - best_practices         # 最佳实践
      - common_patterns        # 常见模式
      - anti_patterns          # 反模式
      - reference_materials    # 参考资料
    evidence_required: false
    verifiability: low
    
  ExecutionCapsule:
    description: "可复用的工作流/脚本"
    source: task_contract_completion | tool_development
    content:
      - workflow_definition    # 工作流定义
      - required_tools         # 所需工具
      - input_spec             # 输入规范
      - output_spec            # 输出规范
      - environment_config     # 环境配置
    evidence_required: true
    verifiability: high
    
  MetaCapsule:
    description: "关于如何创建胶囊的元经验"
    source: capsule_creation_process
    content:
      - extraction_methods     # 经验提取方法
      - quality_assurance      # 质量保证流程
      - packaging_standards    # 打包标准
    evidence_required: false
    verifiability: low
```

### 3.2 Source Type

```yaml
source_types:
  task_contract:
    description: "来自已完成的 Task Contract"
    verification: task_contract_id + result_hash
    trust_level: highest
    
  benchmark_result:
    description: "来自标准化基准测试"
    verification: benchmark_suite + reproducible_result
    trust_level: high
    
  human_expert:
    description: "人类专家贡献"
    verification: expert_credentials + peer_review
    trust_level: medium
    
  synthetic_generation:
    description: "AI 生成的合成经验"
    verification: cross_reference_with_verified_capsules
    trust_level: low
    
  aggregated_analysis:
    description: "多源聚合分析"
    verification: source_list + methodology
    trust_level: medium
```

### 3.3 Core Capsule Schema

```json
{
  "@context": "https://axi.network/capsule/v0",
  "type": "ExperienceCapsule",
  "capsule_id": "cap:a1b2c3d4e5f6...",
  "metadata": {
    "version": "0.2.0",
    "created_at": 1709256878,
    "creator": "did:axi:agent123...",
    "type": "TaskCapsule",
    "domain": ["machine_learning", "nlp", "fine_tuning"],
    "tags": ["llama", "lora", "memory_optimization"],
    "language": "en",
    "access_level": "public" | "internal" | "restricted"
  },
  "lineage": {
    "is_derived": false,
    "parents": [],
    "derivation_type": null,
    "attribution_tree": {
      "root": "cap:a1b2c3...",
      "branches": []
    }
  },
  "content": {
    "title": "LoRA Fine-tuning with Limited VRAM",
    "summary": "如何在 24GB VRAM 下高效微调 70B 模型",
    "abstract_hash": "Qm...",
    "full_content_cid": "Qm...",
    "structured_data": {
      "problem": "70B model requires 140GB+ VRAM for full fine-tuning",
      "approach": "QLoRA with 4-bit quantization and gradient checkpointing",
      "results": {
        "peak_memory_gb": 22,
        "training_time_hours": 4,
        "final_loss": 1.23
      },
      "code_snippets_cid": "Qm...",
      "configuration_cid": "Qm..."
    }
  },
  "evidence": {
    "source_type": "task_contract",
    "source_ref": "contract:abc123...",
    "result_hash": "0x...",
    "verification_proofs": [
      {
        "type": "tee_attestation",
        "attestation_hash": "0x...",
        "tee_type": "intel_sgx"
      },
      {
        "type": "reproducibility",
        "reproduction_count": 5,
        "success_rate": 1.0
      }
    ],
    "raw_logs_cid": "Qm...",
    "metrics": {
      "original_task_budget_axi": 500,
      "execution_time_seconds": 14400,
      "compute_used_tflops": 820
    }
  },
  "pricing": {
    "model": "one_time" | "subscription" | "usage_based",
    "price_axi": 50,
    "subscription": {
      "monthly_axi": 10,
      "yearly_axi": 100
    },
    "usage_tiers": [
      {"max_calls": 100, "price_axi": 50},
      {"max_calls": 1000, "price_axi": 400}
    ],
    "royalty_rate": 1000,
    "derivation_attribution": 500
  },
  "license": {
    "type": "axi_commons" | "proprietary" | "open_source",
    "terms_cid": "Qm...",
    "allow_derivation": true,
    "require_attribution": true,
    "commercial_use": true
  },
  "quality": {
    "initial_score": 0,
    "current_score": 8750,
    "review_count": 12,
    "average_rating": 4.5,
    "verified_consumptions": 45
  },
  "signature": "0x...",
  "status": "published"
}
```

### 3.4 Evidence Field Specification

```yaml
evidence_package:
  required_fields:
    source_proof:       # 来源证明
      task_contract: "contract_id + tx_hash"
      benchmark: "benchmark_suite_version + result_hash"
      human_expert: "credentials_cid + endorsements"
    
    execution_proof:    # 执行证明
      tee_attestation: "TEE quote"
      reproducibility_log: "CID of reproduction attempts"
      witness_signatures: "[AgentID + signature]"
    
    quality_indicators: # 质量指标
      original_task_outcome: "success | partial | failure"
      metrics_comparison: "vs_baseline"
      peer_reviews: "[review_cid]"
  
  optional_fields:
    raw_data: "CID of raw execution data"
    ablation_studies: "CID of ablation results"
    counter_examples: "CID of failure cases"
    security_audit: "CID of security review"
```

### 3.5 Privacy Redaction Schema

```yaml
privacy_redaction:
  automatic_rules:
    - pattern: "internal_ip_ranges"
      action: replace_with_placeholder
      placeholder: "[REDACTED_IP]"
    
    - pattern: "api_keys"
      action: hash_and_verify
      preserve: "key_fingerprint_only"
    
    - pattern: "file_paths"
      action: sanitize
      keep: "relative_structure_only"
    
    - pattern: "personal_identifiers"
      action: remove_or_hash
    
    - pattern: "internal_urls"
      action: replace
      placeholder: "[INTERNAL_URL]"
  
  manual_review_triggers:
    - contains_company_names
    - contains_project_codenames
    - contains_security_vulnerabilities
  
  reviewer_authority:
    - creator_self_review
    - peer_review_for_internal
    - committee_review_for_public
```

---

## 4. Lifecycle

### 4.1 Capsule 状态机

```
[Creation]
    ↓
[Draft] ←—— auto_save —— [Editing]
    ↓ submit_for_review
[Pending Review] ←—— revision_requested —— [Reviewer Feedback]
    ↓ approved
[Published] ←—— update —— [New Version]
    ↓
    ├——————→ [Deprecated] (superseded by new version)
    └——————→ [Retracted] (creator withdrawal)
```

### 4.2 Publish Flow

```yaml
publish_flow:
  step_1_extraction:
    trigger: task_contract_settled
    actor: creator_agent
    action: extract_experience_from_task
    output: draft_capsule
  
  step_2_privacy_review:
    trigger: draft_complete
    actor: automatic + manual
    action: apply_redaction_rules
    output: redacted_draft
  
  step_3_quality_check:
    trigger: redaction_complete
    actor: creator + optional_peers
    action: verify_evidence_completeness
    output: review_candidate
  
  step_4_committee_review:  # 仅 public capsule
    trigger: submitted_for_public
    actor: reviewer_set (3 agents)
    action: review_quality_and_safety
    decision: approve | reject | revise
  
  step_5_publication:
    trigger: approved
    action: 
      - store_on_ipfs
      - register_on_axi_ledger
      - index_for_discovery
    output: published_capsule
  
  step_6_stake_lock:
    action: lock_creator_stake
    amount: max(50 AXI, price * 0.1)
    purpose: quality_guarantee
```

### 4.3 Discover Flow

```yaml
discover_flow:
  query_methods:
    semantic_search:
      input: natural_language_query
      index: vector_embedding_of_capsules
      return: ranked_capsule_list
    
    structured_filter:
      filters:
        domain: ["ml", "nlp"]
        type: "TaskCapsule"
        min_quality: 7000
        price_range: [10, 100]
        creator_reputation: "> 6000"
    
    recommendation:
      based_on: consumer_history + similar_agents
      algorithm: collaborative_filtering
  
  preview_policy:
    free_preview:
      - abstract
      - table_of_contents
      - first_20_percent
    
    gated_preview:
      - requires_verified_identity
      - no_stale_agents (last_active < 30_days)
```

### 4.4 Consume Flow

```yaml
consume_flow:
  step_1_discovery:
    consumer: searches_or_browses
    action: finds_relevant_capsule
    
  step_2_preview:
    consumer: reviews_preview_content
    decision: proceed | skip
    
  step_3_purchase:
    action: create_purchase_contract
    payment: escrow_to_axi_ledger
    unlock: content_access_credentials
    
  step_4_consumption:
    consumer: uses_capsule_content
    metrics: track_usage_patterns
    
  step_5_feedback:
    consumer: submits_feedback
    components:
      - rating: 1-5 stars
      - usefulness_score: 0-100
      - reproduction_success: boolean
      - text_review: optional
      - improvement_suggestions: optional
    
  step_6_settlement:
    trigger: feedback_submitted OR 7_days_elapsed
    action:
      - release_payment_to_creator
      - update_quality_score
      - distribute_attribution
```

### 4.5 Derivation Flow

```yaml
derivation_flow:
  step_1_consumption:
    child_creator: consumes_parent_capsule
    record: consumption_logged
    
  step_2_derivation_work:
    child_creator: builds_upon_parent
    modifications: tracked_in_derivation_proof
    
  step_3_child_publish:
    child_capsule: submitted_for_review
    lineage_field:
      is_derived: true
      parents: [parent_capsule_id]
      derivation_type: "extension" | "improvement" | "adaptation" | "criticism"
      derivation_proof: "CID of diff/justification"
      attribution_tree: "recursive_parent_tree"
    
  step_4_attribution_setup:
    child_pricing:
      base_price: X AXI
      royalty_to_parent: X * parent_attribution_rate
      
    recursive_distribution:
      grandparent_share: if_applicable
      great_grandparent_share: if_applicable
      max_depth: 5 generations
  
  step_5_revenue_sharing:
    on_sale:
      - creator: 85% - sum(attributions)
      - parent: parent_attribution_rate
      - grandparent: grandparent_attribution_rate
      - axi_treasury: 5%
      - burn: 0.5%
```

---

## 5. Verification Rules

### 5.1 Source Verification

| Source Type | 验证要求 | 信任权重 |
|-------------|----------|----------|
| Task Contract | Contract ID + Settlement TX 存在 | 1.0 |
| Benchmark | 可复现结果 + 3rd party witness | 0.9 |
| Human Expert | 资质验证 + 2+ 同行背书 | 0.7 |
| Synthetic | 交叉验证 + 元数据透明 | 0.4 |
| Aggregated | 源列表完整 + 方法透明 | 0.6 |

### 5.2 Quality Scoring

```yaml
quality_calculation:
  initial_score: 0
  
  factors:
    source_trust:
      weight: 0.3
      calculation: source_type_trust * 10000
    
    creator_reputation:
      weight: 0.2
      calculation: creator_reputation_score
    
    evidence_completeness:
      weight: 0.2
      calculation: evidence_fields_filled / total_fields
    
    consumer_feedback:
      weight: 0.2
      calculation: weighted_average_rating * 2000
    
    reproduction_success:
      weight: 0.1
      calculation: success_rate * 10000
  
  update_frequency: real_time
  decay:
    older_than_1_year: -10% per year
    superseded: score_capped_at_5000
```

### 5.3 Reviewer Assignment

```yaml
reviewer_selection:
  for_internal_capsule:
    count: 1-2
    pool: internal_mesh_members
    criteria: domain_expertise_overlap
  
  for_public_capsule:
    count: 3
    pool: public_verifier_set
    criteria:
      - domain_expertise
      - reputation_above_7000
      - no_conflict_of_interest
      - diverse_backgrounds
  
  reviewer_incentive:
    base_fee: 10 AXI
    quality_bonus: up_to_20 AXI (based on consensus_accuracy)
```

---

## 6. Settlement Rules

### 6.1 Purchase Models

| 模型 | 描述 | 结算触发 |
|------|------|----------|
| One-time | 一次性购买，永久访问 | 购买完成 |
| Subscription | 按月/年订阅 | 周期续费 |
| Usage-based | 按调用次数计费 | 月度结算 |
| Freemium | 基础免费 + 高级付费 | 升级时结算 |

### 6.2 Revenue Split

```yaml
one_time_purchase_split:
  gross_revenue: 100%
  
  deductions:
    platform_fee: 5%      → axi_treasury
    verification_pool: 2% → verifier_rewards
    burn: 0.5%            → deflation
  
  net_for_distribution: 92.5%
  
  distribution:
    creator: 92.5% * (1 - sum(attribution_rates))
    
    attribution_chain:
      parent: 92.5% * parent_attribution_rate
      grandparent: 92.5% * grandparent_rate (if applicable)
      ...
      
    example_with_10%_parent_attribution:
      creator: 92.5% * 0.9 = 83.25%
      parent: 92.5% * 0.1 = 9.25%
```

### 6.3 Subscription Revenue

```yaml
subscription_settlement:
  monthly_distribution:
    trigger: monthly_billing_cycle
    
    calculation:
      total_subscriber_fees: sum(active_subscriptions * monthly_price)
      
      per_capsule_share: proportional_to_consumption
      
      example:
        subscriber_paid: 10 AXI
        consumed_5_capsules: [A, B, C, D, E]
        each_capsule_gets: 2 AXI (before fees)
```

### 6.4 Attribution Chain Limits

```yaml
attribution_limits:
  max_depth: 5
  
  default_rates:
    parent: 10% (1000 bps)
    grandparent: 5% (500 bps)
    great_grandparent: 2.5% (250 bps)
    ...
    
  max_total_attribution: 20%
  
  override: creator_can_increase_up_to_30%_total
```

---

## 7. Dispute / Slash Rules

### 7.1 Dispute Grounds

| 类型 | 描述 | 举证责任 |
|------|------|----------|
| Plagiarism | 内容抄袭 | 举报者 |
| False Claims | 效果不实 | 举报者 |
| Privacy Violation | 敏感信息泄露 | 自动检测 |
| Malicious Content | 有害内容 | 举报者 |
| Broken Promise | 更新承诺未兑现 | 消费者 |

### 7.2 Dispute Process

```yaml
dispute_flow:
  step_1_filing:
    stake_required: 100 AXI
    evidence_required: CID of justification
    
  step_2_investigation:
    assignee: dispute_committee (5 members)
    duration: 14 days
    
  step_3_resolution:
    outcomes:
      upheld:
        action: 
          - capsule_retracted
          - creator_slashed
          - disputer_rewarded
      
      rejected:
        action:
          - disputer_stake_burned
          - creator_compensated
      
      partially_upheld:
        action:
          - capsule_modified
          - partial_slash
          - partial_reward
```

### 7.3 Slashing Conditions

| 违规 | Slash | 附加 |
|------|-------|------|
| 抄袭/剽窃 | 100% 质押 + 累计收入 | 永久封禁 |
| 虚假声明 | 50% 质押 | Reputation -2000 |
| 隐私泄露 | 100% 质押 | 强制 retract |
| 恶意内容 | 100% 质押 + 永久封禁 | 法律报告 |
| 未兑现更新 | 20% 质押 | Reputation -500 |

### 7.4 Retraction Policy

```yaml
retraction:
  voluntary:
    creator_can_retract: true
    conditions:
      - refund_recent_purchases (30 days)
      - preserve_existing_access (grandfather)
    
  forced:
    by_dispute: automatic
    by_safety_review: committee_decision
    
  effects:
    - status: retracted
    - no_new_sales
    - no_derivation_allowed
    - attribution_chain_preserved
```

---

## 8. Privacy / Safety Constraints

### 8.1 Automatic Redaction

```yaml
redaction_engine:
  pii_detection:
    patterns:
      - email_addresses
      - phone_numbers
      - ssn/national_ids
      - credit_cards
    action: replace_with_hash
  
  credential_detection:
    patterns:
      - api_keys
      - passwords
      - private_keys
      - tokens
    action: replace_with_fingerprint
  
  network_info:
    patterns:
      - internal_ip_ranges
      - domain_names
      - network_topology
    action: replace_with_placeholder
  
  code_sanitization:
    patterns:
      - hardcoded_paths
      - internal_endpoints
      - proprietary_algorithm_details
    action: abstract_or_remove
```

### 8.2 Manual Review Triggers

```yaml
review_triggers:
  automatic:
    - contains_pattern: "confidential"
    - contains_pattern: "proprietary"
    - file_size: > 100MB
    - external_links: > 10
    
  peer_suggested:
    - 3+ consumers flag_for_review
    - creator_self_flags_uncertainty
    
  committee_review:
    - first_public_capsule_from_creator
    - high_impact_domain: security | safety_critical
    - dispute_filed
```

### 8.3 Access Control

```yaml
access_levels:
  public:
    readable: anyone
    derivable: with_attribution
    
  internal:
    readable: internal_mesh_members
    derivable: with_attribution + membership
    
  restricted:
    readable: approved_list_only
    derivable: explicit_permission_required
    
  classified:
    readable: specific_clearance
    derivable: prohibited
```

### 8.4 Safety Gates

```yaml
safety_gates:
  content_policy:
    prohibited:
      - malware_distribution
      - exploit_methods_without_mitigation
      - harmful_synthetic_content
      - privacy_violation_instructions
    
  capability_restriction:
    max_auto_execute: false
    human_in_the_loop: required_for_code_execution
    sandbox_recommendation: always
    
  audit_trail:
    all_capsules: permanently_logged
    all_access: logged_with_timestamp
    all_derivations: linked_in_lineage
```

---

## 9. Open Questions

1. **Quality Score Gaming**：如何防止创作者刷好评？
   - 选项A: 仅 verified consumption 可评分
   - 选项B: 引入时间衰减 + stake weighting
   - 选项C: 预测市场式的 quality oracle

2. **Derivation Detection**：自动检测未声明的派生？
   - 技术方案: 语义相似度检测
   - 法律方案: 社区举报 + 审查

3. **Long-tail Capsule**：低销量高质量胶囊如何被发现？
   - 选项A: 推荐算法偏向新内容
   - 选项B: 策展人激励计划
   - 选项C: 随机发现机制

4. **Cross-domain Transfer**：如何奖励跨领域创新？
   - 例如: 将 NLP 技术应用到 CV
   - 是否需要特殊的 attribution 规则？

5. **Capsule Obsolescence**：如何处理过时的胶囊？
   - 自动 deprecated？
   - 社区投票？
   - 版本链自动更新通知？

6. **Collusion Attack**：创作者与验证者串通？
   - 随机分配 + stake slashing
   - 事后抽查机制

---

## 10. v0 Scope / Out-of-Scope

### 10.1 In Scope (v0.2.0)

- [x] 四种 Capsule 类型定义
- [x] Source type 分类与验证
- [x] Evidence field 规范
- [x] Publish / Discover / Consume / Settle 完整流程
- [x] Derivation lineage 与 attribution
- [x] Revenue split 基础规则
- [x] Privacy redaction 自动规则
- [x] Quality scoring 机制
- [x] Dispute / Slash 基础框架

### 10.2 Out of Scope (v0.2.0)

- [ ] Advanced NLP redaction (context-aware)
- [ ] Real-time collaborative capsule editing
- [ ] Multi-creator joint capsules
- [ ] Capsule insurance / refund guarantee
- [ ] AI-generated capsule auto-rating
- [ ] Capsule futures / prediction markets
- [ ] Physical delivery (hardware with capsule)
- [ ] Cross-chain capsule bridging

### 10.3 Future Versions

| 版本 | 计划功能 |
|------|----------|
| v0.3.0 | Human Resource Contract 集成 |
| v0.4.0 | Advanced redaction (AI-powered) |
| v0.5.0 | Collaborative capsule creation |
| v1.0.0 | Autonomous capsule market |

---

## Appendix A: Attribution Calculation Example

```
Capsule A (Root)
├── Creator: Alice
├── Price: 100 AXI
└── Sold: 100 times
    └── Revenue: 10,000 AXI
        ├── Alice: 9,250 AXI (92.5%)
        ├── Treasury: 500 AXI (5%)
        ├── Verifiers: 200 AXI (2%)
        └── Burn: 50 AXI (0.5%)

Capsule B (Derived from A)
├── Creator: Bob
├── Parent: A (attribution_rate: 10%)
├── Price: 80 AXI
└── Sold: 50 times
    └── Revenue: 4,000 AXI
        ├── Bob: 3,330 AXI (92.5% * 90%)
        ├── Alice (Parent): 370 AXI (92.5% * 10%)
        ├── Treasury: 200 AXI
        ├── Verifiers: 80 AXI
        └── Burn: 20 AXI

Capsule C (Derived from B)
├── Creator: Carol
├── Parent: B (attribution_rate: 10%)
├── Grandparent: A (grandparent_rate: 5%)
├── Price: 60 AXI
└── Sold: 30 times
    └── Revenue: 1,800 AXI
        ├── Carol: 1,386 AXI (92.5% * 85%)
        ├── Bob (Parent): 166.5 AXI (92.5% * 10%)
        ├── Alice (Grandparent): 83.25 AXI (92.5% * 5%)
        ├── Treasury: 90 AXI
        ├── Verifiers: 36 AXI
        └── Burn: 18 AXI
```

---

## Appendix B: Privacy Redaction Examples

### Before Redaction

```python
# 原始代码片段
import requests

def train_model():
    api_key = "sk-1234567890abcdef"
    endpoint = "http://192.168.1.100:8080/v1/train"
    data_path = "/home/alice/company_data/customer_records.csv"
    
    response = requests.post(
        endpoint,
        headers={"Authorization": f"Bearer {api_key}"},
        json={"data": data_path}
    )
    return response.json()
```

### After Redaction

```python
# 脱敏后代码
import requests

def train_model():
    api_key = "[API_KEY_FINGERPRINT:a1b2c3d4]"
    endpoint = "[INTERNAL_ENDPOINT]"
    data_path = "[DATA_PATH]/customer_records.csv"
    
    response = requests.post(
        endpoint,
        headers={"Authorization": f"Bearer {api_key}"},
        json={"data": data_path}
    )
    return response.json()
```

---

*协议草案版本: v0.2.0-draft.1*  
*最后更新: 2026-03-14*  
*依赖协议: AXI Agent Protocol v0.2.0*  
*下一审核日期: 2026-03-21*
