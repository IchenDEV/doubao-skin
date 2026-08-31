---
id: "2026-09-01-support-windows-workbuddy"
stage: intent
status: accepted
owner: "Codex"
created: "2026-09-01"
source: "user"
risk: "high"
approved_by: "product-owner"
approved_at: "2026-09-01"
---

# Intent: 支持 Windows WorkBuddy 实时主题

## Problem

当前 Windows 版主题工具可以识别并操作「豆包」和「豆包工作」，但把 WorkBuddy 明确硬编码为不支持：安装路径和注册表匹配不会返回 WorkBuddy，已安装二进制缓存固定为空，实时主题入口也拒绝 Windows WorkBuddy。因此即使 Windows 11 虚拟机已经安装并运行 WorkBuddy 5.4.5，界面仍显示“WorkBuddy · 未安装”，用户无法应用或恢复主题。

实机探针已经证明这不是上游应用的技术阻塞：官方当前用户安装位于 `C:\Users\<user>\AppData\Local\Programs\WorkBuddy\WorkBuddy.exe`，以 `--remote-debugging-address=127.0.0.1 --remote-debugging-port=9224` 启动后会在 guest 回环地址提供主 `page`，renderer URL 为同一安装目录下的 `resources/app.asar/renderer/index.html`。当前缺口集中在现有平台适配层，而不是主题包格式或 WorkBuddy CSS adapter。

## Proposed outcome

让现有 Windows 主题工具准确检测受支持的 WorkBuddy 安装，并沿用已有 WorkBuddy 生命周期、回环 CDP watcher 和结构化主题 adapter，在 Windows WorkBuddy 5.4.5 上完成选择、应用、持续注入和恢复默认。Windows 与 macOS 只共享目标语义和主题 adapter；安装发现、进程控制和 renderer 身份由现有平台层按操作系统处理。

## Affected users and systems

- 使用 Windows 11 x64，或在 Windows 11 ARM64 上运行官方 x64 WorkBuddy 的主题工具用户。
- `crates/skin-core` 中 WorkBuddy 的平台安装发现、严格 renderer 身份、启动/退出和实时主题支持判断。
- Windows 桌面工具中的 WorkBuddy 安装状态、目标选择、重启确认、应用/恢复状态；不重做界面和主题包。
- 现有 macOS WorkBuddy、Windows/macOS 豆包与豆包工作行为必须保持不变。

## Constraints

- 复用现有 `TargetApp`、Windows 安装发现、CDP watcher、按目标 session 状态和 WorkBuddy adapter；不增加第二套 manager、插件系统、持久服务或 Windows 专属主题格式。
- CDP 只绑定 `127.0.0.1`，WorkBuddy 继续使用独立端口 `9224`；注入前必须把页面身份绑定到已发现/显式配置的 WorkBuddy 安装，不接受任意 `file://`、普通网页、DevTools、webview 或其他 Electron 应用。
- 不修改 `app.asar`、安装目录、注册表、用户设置、登录状态或 Windows 防火墙；不读取或提交账号、Cookie、任务、插件、工作空间或日志内容。
- WorkBuddy 正在无调试端口运行时，第一次应用仍只提示风险；只有用户明确执行“重启 WorkBuddy 并应用”后才能结束精确 WorkBuddy 进程并带回环参数重启。用户主动退出后不得自动拉起。
- 支持专用路径覆盖，并至少识别已验证的当前用户默认安装与相应卸载注册表项；不得通过无界目录扫描或仅凭模糊名称误认其他程序。
- 先补失败优先回归测试，再实现修复；最终必须在 Windows 11 ARM64 虚拟机的真实主题工具和真实 WorkBuddy 窗口中验证应用与恢复。

## Out of scope

- Linux WorkBuddy、Windows 旧版本或 5.4.5 之外版本的无证据兼容承诺。
- WorkBuddy 登录、账号配置、任务执行、协议桥、模型路由、插件、MCP、自动化或数据访问。
- 修改主题包 schema、迁移主题、增加 Windows 专属 CSS、替换 WorkBuddy 图标/品牌或重新设计桌面 UI。
- 修改或重新分发 WorkBuddy 安装包、绕过应用安全策略，或开放局域网/公网调试端口。

## Success signals

- 在已安装 WorkBuddy 5.4.5 的 Windows 11 ARM64 虚拟机中，当前 Windows 测试包不再显示“WorkBuddy · 未安装”，可选中 WorkBuddy；未安装或无效覆盖路径仍保持不可用。
- 正常应用路径只连接 `127.0.0.1:9224`，严格接受已验证 Windows renderer，拒绝合成的其他 `file://`、远程页、DevTools、webview 和错误端口所有者。
- 至少一款深色主题和“鲸鱼娘”在真实 Windows WorkBuddy 窗口中出现可见、可读的主题效果；刷新/内部导航后持续存在，恢复默认后工具 marker、style 和 backdrop 全部清除。
- Windows WorkBuddy 已运行但端口缺失时仍执行二次重启确认；用户主动退出后不自动重启；豆包、豆包工作和 macOS WorkBuddy 回归行为不变。
- Windows 原生构建/测试、Rust/workflow 门禁和 `git diff --check` 通过，验证记录不包含用户或上游专有内容。

## Open questions

无阻塞产品问题。5.4.5 默认路径和 renderer 已实测；更宽版本覆盖只在获得新的真实窗口证据后扩展。

## Decision

待产品负责人明确接受本 Intent 后进入 Spec。当前只建立 Windows WorkBuddy 独立变更，不修改产品代码，也不重新启动虚拟机。
