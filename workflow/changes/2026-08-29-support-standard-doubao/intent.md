---
id: "2026-08-29-support-standard-doubao"
stage: intent
status: accepted
owner: "codex"
created: "2026-08-29"
source: "user"
risk: "high"
approved_by: "product-owner"
approved_at: "2026-08-29"
---

# Intent: 同时支持「豆包」和「豆包工作」

## Problem

当前主题工具在运行时、应用路径和用户文案中都硬编码指向「豆包工作」。本机同时安装了 macOS 标准版「豆包」 `/Applications/Doubao.app`（bundle id `com.bot.pc.doubao`）和「豆包工作」 `/Applications/DoubaoWork.app`（bundle id `com.work.pc.doubao`），但现有产品只会对后者应用主题，无法覆盖用户的两种真实使用场景。

## Proposed outcome

让用户在同一个桌面主题工具中明确选择「豆包」或「豆包工作」作为当前目标，再将所选主题实时应用到对应的官方 macOS 应用真实窗口。退出主题工具或恢复默认后，两款官方安装包都不应留下改动。应用名称、主界面、官网和使用文档应清晰说明同时支持两款应用。

## Affected users and systems

- 使用 macOS 「豆包」或「豆包工作」的主题工具用户。
- `crates/skin-core` 中两款应用的身份、路径、CDP 启动、进程生命周期、目标页识别和主题注入。
- `apps/desktop` 中的目标选择、应用、恢复和用户可见文案。
- `apps/web`、README 及打包元数据中的产品名称和兼容性说明。

## Constraints

- 不得修改 `/Applications/Doubao.app` 或 `/Applications/DoubaoWork.app`；实时模式只能通过本机回环 CDP 注入。
- 用户必须看得出当前要操作哪款应用；不可因自动猜测而退出、重启或注入错误的应用。
- 两款应用的进程、启动标识和调试端口必须隔离，不得将一款应用的 CDP 端口误判为另一款。
- 不读取、记录或转发会话内容、凭据、Cookie、工作区数据或附件。
- 主题差异留在 manifest 和 CSS 中，不为单个主题新增桌面端分支。
- 保存当前工作区中与本需求无关的未提交改动。
- 采用本机已安装的两款应用做兼容性基线，并把各自版本假设记录在验证工件中。
- 新增「豆包」支持不得破坏「豆包工作」已有的主题注入、窗口恢复和退出还原行为。

## Out of scope

- 豆包网页版、Windows 版和移动端。
- 豆包模型替换、协议桥、网络代理或任何会话转发能力。
- 单次操作同时向「豆包」和「豆包工作」两个运行中应用注入主题；用户一次明确选择一个目标。
- 修改官方应用安装包、重签名官方应用或绕过平台安全限制。
- 将实验性协议桥扩展到标准版「豆包」；其现有的「豆包工作」边界保持原样。

## Success signals

- 桌面工具能分别识别 `/Applications/Doubao.app` 和 `/Applications/DoubaoWork.app`，并让用户明确选择当前目标。
- 当需要以回环调试端口重启应用时，只退出和重启用户选定的那一款应用。
- 选定主题分别在两款应用的真实主对话窗口中生效，导航或打开新页后仍能持续生效。
- “恢复默认”或退出主题工具后，重新打开相应官方应用即为官方外观。
- 对两款应用都在正常和窄窗口尺寸下完成真实窗口截图验证，无可见的布局破坏或文字不可读。
- 核心回归测试覆盖两款应用的标识、路径、调试端口和目标 URL 过滤，相关 Rust、Web 和 workflow 检查通过。

## Open questions

无。“同时支持”指同一个产品提供两个明确可选的应用目标；不指单次点击同时操作两个运行中应用。

## Decision

产品负责人已确认：同一个主题工具同时支持「豆包」和「豆包工作」，由用户一次明确选择一个目标应用。
