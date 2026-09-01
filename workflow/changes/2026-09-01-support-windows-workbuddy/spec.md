---
id: "2026-09-01-support-windows-workbuddy"
stage: spec
status: accepted
owner: "Codex"
created: "2026-09-01"
based_on: intent.md
risk: "high"
approved_by: "product-risk-owner"
approved_at: "2026-09-01"
---

# Spec: 支持 Windows WorkBuddy 实时主题

## Requirements

1. `TargetApp::WorkBuddy` 必须在 Windows 成为受支持的 live 目标；macOS WorkBuddy 和 Windows/macOS「豆包」「豆包工作」的既有行为不得改变，Linux 仍不支持 live 模式。
2. Windows 安装发现必须继续遵循现有优先级：`DOUBAO_SKIN_WORKBUDDY_PATH` 显式可执行文件、当前用户默认安装、Windows 卸载注册表。已验证默认路径为 `%LOCALAPPDATA%\Programs\WorkBuddy\WorkBuddy.exe`；显式路径和注册表结果只有在最终文件名精确为 `WorkBuddy.exe` 时才有效。
3. 注册表发现必须读取 `HKEY_CURRENT_USER` 与 `HKEY_LOCAL_MACHINE` 的 64/32 位视图，只接受大小写不敏感的 WorkBuddy 产品身份；不得把名称中偶然包含 `work` 或 `buddy` 的其他应用识别为 WorkBuddy。
4. 三目标安装缓存必须真实计算 WorkBuddy 项，不能继续把第三项固定为 `None`。本次不增加运行时刷新机制；用户在主题工具启动后才安装 WorkBuddy 时，重新打开主题工具即可刷新状态。
5. Windows WorkBuddy 的主 renderer 身份必须从当前已发现的 `WorkBuddy.exe` 所在目录推导为 `resources/app.asar/renderer/index.html`。比较时忽略 query/hash、处理 Chromium file URL 的百分号编码和 Windows ASCII 大小写差异；不得只按文件名或宽泛路径后缀接受页面。
6. macOS WorkBuddy 继续只接受现有 `/Applications/WorkBuddy.app/Contents/Resources/app.asar/renderer/index.html`。普通 `file://`、其他安装根、远程页面、iframe/webview、DevTools、扩展页和非 `page` target 均不得建立 WorkBuddy 身份。
7. Windows 启动必须复用已发现的精确二进制和其父目录作为 working directory，并显式传入 `--remote-debugging-address=127.0.0.1` 与独立 `--remote-debugging-port=9224`；端口环境变量覆盖保持可用。
8. Windows 必须能以精确 `WorkBuddy.exe` 镜像名判断 WorkBuddy 是否正在运行，使“运行中但没有正确调试端口”进入 `RestartConfirmationRequired`。第一次应用不得退出进程；只有用户执行第二个“重启 WorkBuddy 并应用”动作后才允许先普通结束、必要时强制结束该精确镜像树并重启。
9. WorkBuddy watcher 的既有策略跨平台保持一致：用户主动退出或 `9224` 消失后停止监听，不自动拉起；端口被其他程序占用时只报错，不终止占用者。
10. Windows WorkBuddy 必须复用当前结构化 v2/v3 WorkBuddy adapter、主题 target scope、按目标 session 和恢复脚本；不得注入豆包原始 CSS、执行主题图标替换、增加 Windows 专属 CSS 或修改全部主题包。
11. Windows 桌面端成功检测后必须允许选中 WorkBuddy、应用和恢复。快捷键沿用 GPUI `platform` 修饰键，用户可见/VoiceOver 文案在 Windows 显示 `Ctrl-1`、`Ctrl-2`、`Ctrl-3`，macOS 继续显示 `Command-1`、`Command-2`、`Command-3`。
12. CLI 的 `apply`/`restore --target workbuddy` 不再标注“仅 macOS”；README、英文 README、架构和 CHANGELOG 只有在 Windows 实窗闭环通过后才能改为“Windows WorkBuddy 实验支持 5.4.5”。
13. 协议桥继续只面向豆包工作；不得把 WorkBuddy 加入 payload、端口或目标选择。

## User experience

- 已安装 WorkBuddy 的 Windows 用户重新打开主题工具后，三段目标选择器中的 WorkBuddy 不再显示“未安装”，可点击或用 `Ctrl-3` 选择。
- WorkBuddy 未运行时，点击“应用主题”直接以本机回环调试参数启动并应用；界面沿用现有应用中/正在使用/恢复默认状态，不显示路径、注册表或 CDP 术语。
- WorkBuddy 已普通运行时，第一次点击只显示保存任务和重启风险，并把动作变为“重启并应用”；第二次明确点击才重启。端口冲突、安装无效或启动失败继续显示可执行错误。
- WorkBuddy 与豆包/豆包工作按目标独立保持主题；切换当前选择不停止另一目标 watcher。用户主动关闭 WorkBuddy 后，界面说明监听已停止，不自动重新打开它。
- 本变更不要求用户登录 WorkBuddy。自动验收和代理实窗验收只使用官方登录页或无敏感内容窗口；已登录主工作区的完整视觉复核保留给人类最终验证者。

## Technical design

- 在 `crates/skin-core/src/live/platform.rs` 的现有 Windows seam 中补充 WorkBuddy 默认相对路径、精确注册表身份和安装缓存第三项；复用 `windows_binary_in_root`、两视图注册表读取、`binary_from_install_path`、`launch_app`、`tell_app` 与 `kill_app`，不增加新 manager。
- 把 Windows 的 `tasklist` 精确镜像检查收束为现有平台层的小函数，并让 `process_running(TargetApp::WorkBuddy)` 使用已发现二进制的文件名；非 WorkBuddy 准备状态保持原逻辑。
- 在 `live.rs` 将 WorkBuddy renderer 判断拆为一个可测试的跨平台身份函数：macOS 对照既有常量；Windows 接收已发现二进制，构造唯一 renderer 路径并与解码、去 query/hash 的 file URL 比较。生产入口仍由 `TargetApp::matches_identity_url` 和现有 `targets_belong_to` 调用。
- Windows `launch_app` 只增加显式 loopback address 参数；端口、working directory、marker 和 watcher 均复用现有实现。
- 在 `apps/desktop/src/app/helpers.rs` 仅把快捷键展示按目标操作系统区分；输入处理继续使用 GPUI `modifiers.platform`，不增加 Windows 专用事件分支。
- 更新 `crates/skin-core/src/bin/doubao-skin.rs` 的帮助文字；只有真实 Windows Gate 通过后再更新 README、README.en、`docs/architecture.md` 与 CHANGELOG。
- `crates/skin-core/src/theme.rs`、主题包、生成 Web catalog 和 `protocol_bridge.rs` 不应发生产品代码变化；现有 adapter 回归测试负责证明复用成立。

## Security and privacy

- renderer 身份由已验证安装二进制推导，不能仅凭 `WorkBuddy` 名称、端口可用或通用 `index.html` 注入；错误端口所有者保持拒绝。
- 对百分号编码只做严格 `%HH` 解码；无效编码、非 UTF-8、路径不一致、`..` 形式或非 `file:///` URL 一律不匹配。Windows 比较仅放宽 ASCII 大小写和斜杠形式，不放宽目录归属。
- 启动参数显式绑定 `127.0.0.1`，不得请求 Windows 防火墙入站权限、绑定 `0.0.0.0` 或使用远程主机。
- 安装发现只读环境变量、标准当前用户目录和卸载注册表；不写注册表、不扫描用户目录以外的无界文件树、不修改安装文件。
- 进程结束只在现有二次确认 Gate 后按精确 `WorkBuddy.exe` 镜像执行；不得用 `Electron.exe`、模糊窗口标题、PID 猜测或宽泛 PowerShell 过滤终止其他程序。
- 验收不登录账号，不读取或保存 Cookie、存储、任务、工作空间、插件或控制台日志。截图必须避开 PowerShell 日志和任何用户内容。

## Alternatives and non-goals

- 不新增 Windows WorkBuddy adapter：已验证 renderer 使用同一应用结构，当前问题位于平台发现与身份层；只有真实窗口证明 DOM 分叉时才另开主题适配变更。
- 不把 WorkBuddy 当成任意 Electron 应用插件：通用 Electron 扫描、端口探测和路径后缀匹配会扩大误注入风险。
- 不通过修改 `app.asar`、快捷方式、注册表启动项或用户设置持久开启调试端口。
- 不为安装后即时刷新增加文件监控、按钮或后台服务；重新打开主题工具即可更新低频安装状态。
- 不实现 Linux、协议桥、登录自动化、版本自动兼容承诺、Windows 专属主题资源或 UI 重设计。

## Areas of concern

- Windows WorkBuddy 是多进程 Electron 应用，`taskkill /IM WorkBuddy.exe /T` 会结束属于该应用的同名进程树；必须严格保留二次确认，不得在首次应用或恢复时调用。
- Windows 文件 URL 可能包含空格、非 ASCII 用户名和百分号编码；身份测试必须覆盖默认路径、空格/UTF-8 编码、query/hash、大小写，以及无效编码和邻近伪路径拒绝。
- Windows 11 ARM64 虚拟机运行的是官方 x64 WorkBuddy；这能验证实际兼容层，但不能替代 Windows x64 runner 上的原生核心测试和三架构包构建。
- 当前虚拟机没有 WorkBuddy 账号，代理只能验证登录 renderer 上的可见主题、marker、持续注入和恢复。已登录主工作区、内部导航及复杂表面的最终视觉 verdict 仍需人类在无敏感内容会话中完成。
- WorkBuddy 后续版本可能改变安装位置、进程名或 renderer；版本提示和文档只声明实测 5.4.5，不做无证据泛化。

## Acceptance criteria

1. 失败优先单元测试先证明当前实现拒绝 Windows WorkBuddy，并覆盖：平台支持判断、默认路径、精确注册表身份、缓存目标映射、Windows 进程检测输入、loopback 启动参数和由安装二进制推导 renderer URL。
2. 修复后，`ensure_live_supported("windows", WorkBuddy)` 成功；macOS WorkBuddy、Windows 豆包/豆包工作与 Linux 拒绝测试保持通过。
3. 安装发现测试接受 `%LOCALAPPDATA%\Programs\WorkBuddy\WorkBuddy.exe`、有效覆盖和两注册表视图，拒绝错误文件名、相似产品名及不存在路径；三目标结果互不串线。
4. URL 表测试接受 macOS 既有 renderer 与 Windows 默认/空格/UTF-8 路径的唯一 renderer（含 query/hash 和 ASCII 大小写差异），拒绝其他安装根、兄弟 html、`..`、坏 `%`、普通 file、remote、iframe/webview、DevTools 和 extension。
5. 生命周期测试继续锁定 WorkBuddy 运行且端口缺失时需要确认、错误端口不结束进程、用户退出不重启；Windows 原生命令测试或纯函数断言证明启动含 `127.0.0.1` 和 `9224`，结束目标只为 `WorkBuddy.exe`。
6. Windows 11 ARM64 虚拟机安装态实测：新包启动后 WorkBuddy 可选且 `Ctrl-3` 文案正确；未安装/错误覆盖的合成测试仍不可用。
7. 虚拟机中 WorkBuddy 未运行时应用一款深色主题和“鲸鱼娘”，确认只有 guest `127.0.0.1:9224` 监听、严格主 `page` 获得 `data-skin-target=workbuddy`、style/backdrop，真实窗口有可见且可读变化；刷新后 marker 保持。
8. 点击恢复默认后，主 renderer 的 runtime、style、backdrop 和工具属性全部清除；普通重开保持官方外观。运行中无端口场景的第一次应用不退出，第二次确认后才重启；用户主动退出后至少 12 秒不自动拉起。
9. 如虚拟机中的豆包工作可用，同时应用豆包工作与 WorkBuddy 主题，切换目标和恢复其中一个不停止或清除另一个；否则明确记录该 Gate 因缺少可用目标而未执行，不得宣称双目标 Windows 实窗通过。
10. Windows x64 runner 执行 `cargo test -p skin-core --lib --locked`；CI 构建 x64/x86/ARM64 桌面与 CLI 包。适用本地 `cargo fmt --all -- --check`、`./scripts/check.sh rust`、`./scripts/check.sh workflow`、桌面回归和 `git diff --check` 全部通过。
11. `verification.md` 记录 Windows 版本、WorkBuddy 5.4.5、包 SHA、命令、视觉证据、恢复、隐私边界、未完成 Gate 与偏差；fresh-context verifier 或人类给出最终 verdict，代理不自行跨越发布 Gate。

## Decision

待产品负责人明确接受本 Spec 后进入 Plan。当前只记录已接受 Intent 与待审 Spec，不修改产品代码、不启动虚拟机，也不更新产品宣传。
