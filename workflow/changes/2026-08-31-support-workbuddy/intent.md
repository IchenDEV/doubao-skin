---
id: "2026-08-31-support-workbuddy"
stage: intent
status: accepted
owner: "codex"
created: "2026-08-31"
source: "user"
risk: "high"
approved_by: "product-owner"
approved_at: "2026-08-31"
---

# Intent: 支持 WorkBuddy 实时主题

## Problem

当前桌面工具只把 macOS 版「豆包」和「豆包工作」建模为可选目标，无法识别或操作 `/Applications/WorkBuddy.app`。本机 WorkBuddy 5.3.14 是 bundle id `com.workbuddy.workbuddy` 的 Electron 应用，主界面位于 `file:///Applications/WorkBuddy.app/Contents/Resources/app.asar/renderer/index.html#`；它可以沿用回环 CDP 的运行时注入安全边界，但页面结构与豆包不同，现有依赖 Semi Design 变量和豆包 DOM 的主题 CSS 不能被视为天然兼容。

## Proposed outcome

让用户在同一个桌面主题工具中明确选择 WorkBuddy，并把所选主题安全地应用到 WorkBuddy 的真实主窗口。首版必须先证明 WorkBuddy 能以独立回环调试端口启动、准确识别目标页、注入最小主题并立即恢复，再接入正式目标选择和主题兼容层；不能只增加名称或 bundle id 后宣称完成。

## Affected users and systems

- 使用 macOS WorkBuddy 的现有用户。
- `crates/skin-core` 中的目标应用身份、进程生命周期、CDP 页面识别、注入与恢复。
- `apps/desktop` 中的目标选择、安装状态、应用/恢复状态和 WorkBuddy 兼容性提示。
- 主题运行时中需要按宿主隔离的 CSS 兼容层，以及相关测试与真实窗口验证。

## Constraints

- 不修改、解包后重打包、重签名或覆盖 `/Applications/WorkBuddy.app`；只允许通过 `127.0.0.1` 上的 CDP 做进程内运行时注入。
- WorkBuddy 必须使用独立于豆包 `9223` 和豆包工作 `9222` 的调试端口，并在注入前以受控 URL/页面特征确认目标身份。
- 用户必须明确选择目标；工具不得自动终止、重启或注入错误应用。
- 只写入主题运行时、自有 DOM 标记和样式；不读取、记录或转发任务正文、账号凭据、Cookie、工作空间、文件内容、插件数据或网络请求。
- WorkBuddy 与豆包的页面体系不同，兼容样式必须按宿主隔离；不得用 WorkBuddy 选择器污染现有豆包主题行为，也不得把“颜色局部生效”表述为完整兼容。
- 退出工具或恢复默认后不留下主题效果；不得破坏 WorkBuddy 自带功能、更新、签名或用户设置。
- 不降低「豆包」和「豆包工作」现有注入、恢复和目标隔离的回归保障。
- 实现前必须由用户依次明确接受本变更的 intent、spec 和 plan。

## Out of scope

- WorkBuddy 的模型替换、协议桥、网络代理、MCP、插件、自动化、账号或任务数据集成。
- 修改 `app.asar`、WorkBuddy 安装包、官方资源或用户配置文件来实现持久换肤。
- Windows/Linux 版 WorkBuddy，或对 WorkBuddy 后续版本作未经验证的兼容承诺。
- 首版替换 WorkBuddy 自有图标、品牌资产或复杂编辑器/文档画布内部样式；除非真实窗口验证证明现有主题资产可以合法、稳定复用。
- 将 WorkBuddy 支持扩展为任意 Electron/VS Code 应用插件系统。

## Success signals

- 桌面工具能准确检测本机 WorkBuddy，并把它显示为第三个明确可选的目标；未安装时显示不可用而不误操作其他应用。
- WorkBuddy 以独立回环端口启动后，工具只识别其主界面目标，不向普通网页、扩展页、登录页、文档内容页或其他应用注入。
- 至少一款基线主题在 WorkBuddy 真实主窗口的侧栏、主内容区、输入区和弹层上呈现一致、可读的颜色与背景；正常和窄窗口均无明显布局破坏。
- 应用主题、页面刷新/导航后的持续注入、恢复默认、退出工具后的还原均在 WorkBuddy 5.3.14 上有可复核证据。
- WorkBuddy 已在无敏感内容的隔离会话中验证，不读取或输出任务正文、凭据、Cookie、工作空间、文件、插件或网络内容。
- Rust 回归测试覆盖 WorkBuddy 元数据、独立端口、目标页过滤、错误端口拒绝和恢复脚本；桌面目标选择测试覆盖三目标状态。
- 现有「豆包」和「豆包工作」相关测试与适用仓库检查继续通过。

## Open questions

无。当前变更只处理 WorkBuddy 的实时主题；“支持”不包含协议桥、插件或数据访问。由于 WorkBuddy 页面结构独立，第一款基线主题验证通过后再决定哪些现有主题可以标记为 WorkBuddy 兼容，未验证主题不得默认宣称支持。

## Decision

待产品负责人明确接受本 Intent 后进入 Spec；当前不修改产品代码，也不重启正在运行的 WorkBuddy。
