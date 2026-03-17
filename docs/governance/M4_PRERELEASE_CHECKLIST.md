# M4 Pre-Release Checklist

**目标**: AXI Node v0.1.0 Mainnet Launch  
**代码冻结**: `b0b5c31`  
**测试状态**: 97/97 passing

---

## 前置条件 (必须全部通过)

### M1 Security ✅

- [x] P0-1 Wallet verification bypass - FIXED
- [x] P0-2 Admission trust issue - FIXED  
- [x] P0-3 Escrow authorization - FIXED
- [x] P1-2 Repository/Service boundary - COMPLETE
- [ ] M1 Security Sign-off - **等待签字**

### M2 Operational Base ✅

- [x] M2-1 SQLite Persistence (5/5 tests)
- [x] M2-2 Relational Schema (10/15 tests)
- [x] M2-3 Transaction Journal (15/15 tests)

### M3 Release Gating ✅

- [x] M3-1 Release Gating Logic (17/17 tests)
- [x] M3-2 Feature Flags (26/26 tests)

### DIBL Integration ✅

- [x] DIBL v0.1 implementation
- [x] Schema alignment with DragonCore
- [x] 8-point event emission
- [x] CLI observation tools

---

## 发布前检查

### 代码质量

- [ ] 无 `TODO` 或 `FIXME` 标记在核心路径
- [ ] 所有 `unwrap()` 已审查
- [ ] 错误处理覆盖完整

### 测试验证

```bash
# 完整测试
cargo test --release

# 基准测试
cargo bench

# 文档测试
cargo test --doc
```

### 文档完整性

- [ ] README.md 已更新
- [ ] API 文档完整
- [ ] 部署指南已验证

### 运维准备

- [ ] 监控指标定义
- [ ] 告警规则配置
- [ ] 备份策略确认
- [ ] 回滚方案准备

---

## 发布检查

### 版本信息

| 项目 | 值 |
|------|-----|
| 版本号 | v0.1.0 |
| 代码提交 | `b0b5c31` |
| 发布时间 | 2026-03-XX |
| 目标网络 | Mainnet |

### 创世配置

- [ ] 创世区块哈希确认
- [ ] 创世钱包地址确认
- [ ] 独立日时间戳确认 (2027-01-01 00:00:00 UTC)
- [ ] 宪法哈希确认

---

## 发布后验证

### 即时验证 (发布后 1 小时内)

- [ ] 节点启动成功
- [ ] 创世区块加载正确
- [ ] 时间锁倒计时正常
- [ ] DIBL 事件流正常

### 24 小时监控

- [ ] 无崩溃或 panic
- [ ] 内存使用稳定
- [ ] 日志无异常错误

---

## 应急准备

| 场景 | 响应 |
|------|------|
| 严重安全漏洞 | 立即冻结，启动 M1.1 修复 |
| 性能问题 | 启动 M3.3 优化周期 |
| 数据损坏 | 执行备份恢复方案 |

---

## 签字确认

| 角色 | 确认 | 签字 | 日期 |
|------|------|------|------|
| 安全负责人 | M1 达标 | | |
| 技术负责人 | 架构稳定 | | |
| 运维负责人 | 部署就绪 | | |
| 项目负责人 | 发布批准 | | |

---

**状态**: 🔄 等待 M1 签字完成后进入最终发布流程
