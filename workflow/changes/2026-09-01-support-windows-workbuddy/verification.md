---
id: "2026-09-01-support-windows-workbuddy"
stage: verification
status: pending
owner: "Codex"
created: "2026-09-01"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: 支持 Windows WorkBuddy 实时主题

## Automated checks

- 基线 `cargo test -p skin-core workbuddy --no-fail-fast` passed：7 项现有 WorkBuddy 核心/adapter 测试和 2 项 v3 WorkBuddy 契约测试通过。
- 基线 `cargo test -p skin-core windows_ --no-fail-fast` passed：5 项 Windows 平台测试及 bundled Windows 路径测试通过。
- `./scripts/check.sh workflow` passed：19 组 artifact、审批策略和 portability 边界通过。
- 失败优先 `cargo test -p skin-core windows_install_detection_checks_all_per_user_targets -- --nocapture` 按预期失败（exit 101）：WorkBuddy 默认当前用户路径返回 `None`。
- 失败优先 `cargo test -p skin-core windows_registry_detection_keeps_the_three_products_isolated -- --nocapture` 按预期失败（exit 101）：精确 `WorkBuddy` 注册表产品仍被拒绝。
- 失败优先 `cargo test -p skin-core live_mode_reports_its_platform_boundary_before_using_app_paths -- --nocapture` 按预期失败（exit 101）：Windows WorkBuddy 仍被 macOS-only 支持开关拒绝。
- 第二组失败优先 `cargo test -p skin-core windows_ --no-fail-fast` 按预期在编译期失败（exit 101）：14 个 `E0425` 精确指向尚不存在的缓存索引、二进制名、loopback 参数、`tasklist` 解析和 Windows renderer 身份函数。
- 第二组失败优先桌面窄测试按预期在编译期失败（exit 101）：`target_shortcut_for_platform` 尚不存在。
- 修复后 `cargo test -p skin-core windows_ --no-fail-fast` passed：11 项 Windows 单元测试和 1 项 bundled Windows 路径测试通过。
- 修复后 `cargo test -p skin-core workbuddy --no-fail-fast` passed：10 项 WorkBuddy 核心/adapter 测试和 2 项 v3 WorkBuddy 契约测试通过。
- 修复后桌面快捷键窄回归 passed；CLI 帮助窄回归 passed。
- 本机 `cargo check --target aarch64-pc-windows-msvc` 无法完成：macOS 主机没有 Windows MSVC C 头文件/`lib.exe`，分别停在既有 `ring`/`psm` 原生构建脚本；此项不作为产品失败，Windows 编译由 PR 原生 runner Gate 验证。
- 首次完整 Rust Gate 的测试全部通过，随后 Clippy 精确发现测试辅助函数被生产模块重导出形成 unused import；将测试直接导入原模块后，`./scripts/check.sh rust` passed（15 desktop、54 core、30 integration/schema tests，Clippy `-D warnings` 通过）。
- `cargo fmt --all -- --check`、`./scripts/check.sh workflow`（19 组）和 `git diff --check` passed。

## Behavioral evidence

- 上一轮 Windows 11 ARM64 实窗探针已确认 WorkBuddy 5.4.5 位于当前用户默认目录、可启动，并在 guest `127.0.0.1:9224` 返回 Windows renderer；本轮实现前没有启动虚拟机或中断应用。

## Visual evidence

Pending：实现与当前 head 的 Windows ARM64 包尚未完成。

## Security and privacy evidence

- 失败优先阶段只使用合成临时目录、合成注册表字段和平台字符串断言；没有读取 Windows 用户内容或修改 WorkBuddy。
- `git diff --name-only` 确认没有修改 `theme.rs`、`protocol_bridge.rs`、`themes/` 或生成 Web catalog。

## Deviations and residual risk

无产品范围偏差。Windows 目标本机交叉检查受 MSVC SDK 缺失阻塞，等待 PR 原生 runner；真实 Windows VM Gate 仍未执行。

## Verdict

Pending。
