---
id: "2026-08-29-disable-window-resizing"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: spec.md
risk: "low"
approved_by: "engineering-owner"
approved_at: "2026-08-29"
---

# Plan: 固定主题工具窗口尺寸

## Files and ownership

- `apps/desktop/src/main.rs`：集中定义 `1120 × 720` 固定尺寸，构造主窗口参数并设置 `is_resizable = false`；增加尺寸与窗口能力回归测试。保持现有界面渲染、主题状态和应用逻辑不变。
- `workflow/changes/2026-08-29-disable-window-resizing/verification.md`：记录自动检查、真实窗口尺寸、拖拽/缩放尝试、截图和剩余风险。
- 不修改 `crates/skin-core`、`themes/**`、`apps/web/**`、官方应用或生成目录。

## Order of work

1. 修改前核对当前 `main.rs` 的窗口创建片段和并行工作树状态，只在目标行做局部变更。
2. 先增加会失败的桌面 UI 回归测试，要求主窗口参数为 `1120 × 720`、`is_resizable = false` 且最小尺寸一致。
3. 将现有默认尺寸提取为本文件内的明确常量/小函数，复用到居中边界和最小尺寸，不新增窗口管理模块。
4. 在 `WindowOptions` 中显式设置 `is_resizable = false`，保留默认的可移动、可最小化和可关闭行为。
5. 运行格式化、定向 UI 测试、桌面编译和 workflow 检查，修复本变更引入的问题。
6. 构建 macOS 应用包，从打包产物启动应用，记录初始窗口尺寸并逐项尝试边缘、角落、绿色按钮、标题栏双击和全屏快捷键。
7. 将命令、实际尺寸、交互结果、截图和低分辨率限制写入 `verification.md`；实现会话不自行给出 fresh-context 最终通过结论。

## Test-first proof

- 新测试通过可独立构造的窗口参数直接断言固定宽高、不可缩放和最小尺寸。
- 在实现前运行该测试并确认它因当前窗口仍可缩放而失败；实现后运行 `cargo test -p doubao-skin-desktop ui_regression_tests --locked` 并确认通过。
- 同一测试保留 `is_movable` 与 `is_minimizable` 的期望，避免固定尺寸时误伤标准窗口能力。

## Visual or integration proof

- 启动最终打包的「豆包主题」，使用 macOS 辅助功能树读取窗口边界，确认内容尺寸为 `1120 × 720`。
- 依次尝试拖拽上、下、左、右四条边及四个角，操作前后读取的宽高必须完全一致。
- 点击/长按绿色按钮、双击标题栏并发送 `Control-Command-F`，窗口不得进入缩放、平铺或全屏状态。
- 移动窗口、最小化、从 Dock 还原和关闭仍可用；保留一张无敏感内容的固定窗口截图。

## Risks and mitigations

- GPUI/macOS 对不可缩放窗口的绿色按钮表现可能随系统版本变化：以真实 macOS 交互为验收，不只依赖字段测试。
- 固定窗口在小于 `1120 × 720` 可用区域的显示器上可能被裁切：按已确认产品约束记录，不增加未批准的降级尺寸。
- 并行工作可能同时修改 `main.rs`：应用局部补丁并在构建前重新核对目标片段和 mtime，不回退他人改动。
- 打包产物可能陈旧：构建前后核对二进制时间，验证从本次产物启动的进程路径。

## Rollback

- 只移除 `main.rs` 中新增的固定尺寸常量/窗口参数函数和 `is_resizable = false`，恢复原窗口创建参数。
- 回滚不使用 `git reset --hard` 或 `git checkout --`，不触碰主题、用户偏好、官方客户端和并行改动。
- 从回滚后的源代码重新构建 `.app`，不手工编辑已打包二进制。

## Deviations

无。若真实平台仍允许改变尺寸并需要 AppKit 私有窗口操作，先更新规格和计划并重新确认，不在实现中临时绕过。

## Decision

等待工程负责人确认本计划后开始修改产品代码与测试。
