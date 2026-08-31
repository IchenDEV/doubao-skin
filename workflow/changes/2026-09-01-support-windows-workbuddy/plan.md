---
id: "2026-09-01-support-windows-workbuddy"
stage: plan
status: accepted
owner: "Codex"
created: "2026-09-01"
based_on: spec.md
risk: "high"
approved_by: "product-risk-owner"
approved_at: "2026-09-01"
---

# Plan: 支持 Windows WorkBuddy 实时主题

## Files and ownership

- `crates/skin-core/src/live/platform.rs`：负责 Windows WorkBuddy 安装发现、缓存槽位、精确进程判断、loopback 启动参数及同文件回归测试。
- `crates/skin-core/src/live.rs`：负责 Windows live 支持开关、由已发现二进制推导的 WorkBuddy renderer 身份及 URL/生命周期回归测试。
- `apps/desktop/src/app/helpers.rs`、`apps/desktop/src/ui_regression_tests.rs`：只负责 Windows `Ctrl-1/2/3` 与 macOS `Command-1/2/3` 的平台文案契约。
- `crates/skin-core/src/bin/doubao-skin.rs`：移除 WorkBuddy “仅 macOS”的过期 CLI 帮助限制。
- `README.md`、`README.en.md`、`docs/architecture.md`、`CHANGELOG.md`：仅在 Windows 真实应用/恢复 Gate 通过后更新实验支持范围和 5.4.5 证据。
- `workflow/changes/2026-09-01-support-windows-workbuddy/verification.md`：记录失败优先、自动检查、Windows 包、虚拟机实窗、恢复、隐私和剩余风险。
- 不修改 `crates/skin-core/src/theme.rs`、`crates/skin-core/src/protocol_bridge.rs`、`themes/` 或生成 Web catalog；若真实 Windows renderer 证明 adapter 结构不同，停止并修订 Spec，不在本变更中临时堆 Windows CSS。
- 这些文件形成一个直接依赖链，按顺序由同一实现者修改，不拆并行工作或新 worktree；继续更新当前分支和现有 Draft PR #13，不创建重复 PR。

## Order of work

1. **锁定基线。** 确认 Git 只包含本变更 artifact、当前 PR/HEAD、现有 Windows CI 三架构矩阵和上一轮 Windows 5.4.5 探针记录；运行现有 WorkBuddy/platform 窄测试。**Gate：** 基线通过且没有用户未提交的意外改动，否则先停止处理重叠。
2. **先写平台失败测试。** 在 `platform.rs` 增加默认 `%LOCALAPPDATA%\Programs\WorkBuddy\WorkBuddy.exe`、错误文件名、精确注册表名、三目标缓存索引、`tasklist` 精确镜像解析和 loopback 启动参数断言；在 `live.rs` 把 Windows 支持与路径派生 URL 表写成失败用例；在桌面回归测试中加入 macOS/Windows 快捷键标签。**Gate：** 测试只因当前硬编码拒绝、空路径/缓存和缺少身份函数而失败，既有豆包/macOS 用例仍通过。
3. **修复安装发现。** 只扩展现有 Windows path/registry seam：加入 WorkBuddy 默认路径，限制显式/注册表结果的最终文件名，精确匹配产品名，并把 OnceLock 第三项改为真实探测。**Gate：** 安装发现窄测试通过，三目标互不串线，无无界目录扫描。
4. **修复严格 renderer 身份。** 保留 macOS 常量，增加严格 `%HH` 解码和 Windows 路径比较；期望路径必须从当前 `WorkBuddy.exe` 父目录构造，去除 query/hash 后才比较。生产 `TargetApp::matches_identity_url` 复用该函数。**Gate：** 接受默认/空格/UTF-8/大小写/query/hash，拒绝邻近根、兄弟 html、`..`、坏编码、普通 file/remote/DevTools/extension；错误端口所有者仍拒绝。
5. **修复 Windows 生命周期。** 复用 `tasklist`/`taskkill`，让 `process_running` 精确判断安装二进制的镜像名；启动命令增加 `--remote-debugging-address=127.0.0.1`。不改 WorkBuddy 的确认状态机和 `relaunch_after_port_loss=false`。**Gate：** 运行中无端口仍返回确认、首次应用不结束、确认后目标只为 `WorkBuddy.exe`，Windows 豆包行为回归通过。
6. **修正用户可见平台信息。** 用一个可测试的平台参数辅助函数生成快捷键标签，现有输入仍使用 GPUI `modifiers.platform`；移除 CLI “WorkBuddy 仅 macOS”。不调整布局、主题筛选或应用按钮。**Gate：** 桌面窄回归和 CLI 帮助断言通过。
7. **本地收敛。** 运行 `cargo fmt --all -- --check`、WorkBuddy/platform/桌面目标测试、`./scripts/check.sh rust`、`./scripts/check.sh workflow` 和 `git diff --check`；搜索确认主题、协议桥、主题包和生成目录没有变化。**Gate：** 全部通过后才提交实现并推送现有 Draft PR。
8. **取得 Windows 原生包。** 等待 PR 的 Windows x64 核心测试与 x64/x86/ARM64 包构建完成，下载与 VM 匹配的 ARM64 ZIP，校验 Actions artifact 和 ZIP checksum，不用旧包替代。**Gate：** head SHA 与 artifact 一致，三架构 job 完成；CI 失败则回到对应实现步骤。
9. **执行当下确认。** 在启动虚拟机并重新加入临时 localhost VNC 前说明系统设置变化；在首次可能结束普通 WorkBuddy 的二次重启测试前再次确认虚拟机内没有需保留任务。不得把 Plan 批准当作进程中断的执行当下授权。**Gate：** 未确认时只做不干扰的安装检测和未运行启动路径。
10. **Windows 真实闭环。** 用新 ARM64 包验证 WorkBuddy 可选、`Ctrl-3` 文案、未运行直接应用、guest 仅 `127.0.0.1:9224` 监听、严格 Windows `page` identity、深色与鲸鱼娘的可见实窗效果、刷新持续、恢复清理、普通重开、二次重启 Gate、用户退出至少 12 秒不拉起；如豆包工作可用，再验证两目标同时保持与独立恢复。**Gate：** 不读取登录/任务/插件/日志内容，不用 marker 或计算样式替代实窗视觉；任何失败先修复回归再重打包。
11. **文档与最终门禁。** 只有步骤 10 通过后更新双语 README、架构与 CHANGELOG；填写 `verification.md`，再运行 `./scripts/check.sh all`、`git diff --check`，推送并等待适用远程检查完成。**Gate：** 工作区干净、证据与当前 head 对齐，临时 VNC/HTTP 服务已移除且 VM 正常关机或恢复用户要求状态。
12. **独立验证。** fresh-context verifier 或人类对照 Spec 复核代码、Windows artifact、实窗截图、双目标/恢复与隐私边界，再给最终 verdict。实现者可更新 Draft PR，但不自行合并、发布或跨越生产 Gate。

## Test-first proof

- `platform.rs` 的新测试在实现前应分别暴露四个现有缺口：WorkBuddy 默认路径为空、注册表永远拒绝、缓存第三项固定 `None`、Windows `process_running` 固定 `false`/启动缺少显式 loopback address。
- `live.rs` 的新测试应先因 `ensure_live_supported("windows", WorkBuddy)` 返回错误，以及 Windows renderer 不匹配而失败；不能通过删除断言或放宽为通用路径后缀让测试变绿。
- URL 表使用合成路径和 URL，不需要真实 WorkBuddy 资源；至少包含空格、UTF-8、`%20`、`%E4...`、混合大小写、query/hash、坏 `%`、`..`、其他用户/安装根、其他 app.asar、remote、DevTools 和 extension。
- 进程/命令测试优先抽取最小纯函数（镜像输出解析、启动参数列表），不 mock `Command`、不新增进程抽象层或依赖。
- 桌面测试直接断言 `target_shortcut_for_platform("windows", WorkBuddy) == "Ctrl-3"` 与 macOS 既有值；不为一个字符串增加新 i18n 系统。
- 修复后先跑 `cargo test -p skin-core workbuddy --no-fail-fast` 与 platform 精确测试，再跑桌面回归和完整 Rust Gate；测试数量只覆盖新增行为，不复制现有 adapter 断言。

## Visual or integration proof

- 使用上一轮已安装 WorkBuddy 5.4.5 的 Windows 11 ARM64 VMware Fusion 虚拟机；不要求 WorkBuddy 账号，不触碰登录凭据。
- 保存的安全证据至少包括：新主题工具显示 WorkBuddy 可选、深色主题实窗、鲸鱼娘实窗、恢复后的官方窗口；截图不得包含 PowerShell 控制台、插件名称、账号、任务或日志。
- CDP 只读探针仅记录 target type、规范化 URL、工具自有 marker/style/backdrop 是否存在；不读取 DOM 文本、Cookie、storage、network 或 console。
- 视觉验收同时看正常与窄窗口：背景/主表面/登录控件清晰可读、无重边框和异常透明叠层。登录页证据只证明 Windows runtime 与可见外壳，不冒充已登录主工作区完整兼容。
- 恢复后必须从同一严格 `page` 证明 marker/style/backdrop 清空，并普通重开确认不残留；然后关闭临时调试实例。
- 若豆包工作可用，在两个真实窗口中同时显示主题，再只恢复 WorkBuddy，确认豆包工作仍保持；若不可用，明确标记未完成。

## Risks and mitigations

- **误注入：** 只接受从已发现二进制推导的唯一 renderer 路径，严格解码并拒绝坏编码/`..`/其他 root；端口身份不符直接停止。
- **误杀进程：** WorkBuddy 仍使用二次确认，命令只匹配精确 `WorkBuddy.exe`；不使用 `Electron.exe`、模糊标题或宽泛 PowerShell 过滤。
- **网络暴露：** 启动参数显式绑定 `127.0.0.1`，VM 中用监听证据确认；防火墙提示一律取消，不扩大入站权限。
- **版本/DOM 漂移：** 文档仅声明 Windows 5.4.5；登录页和未登录状态不足以关闭主工作区视觉 Gate，最终 verdict 保留该限制。
- **ARM 仿真差异：** VM 真实运行 x64 WorkBuddy，同时要求 Windows x64 runner 核心测试和三架构原生包构建；不从单一 VM 泛化所有平台。
- **状态残留：** 每次视觉测试以恢复默认和普通重开结束；移除 VMX 临时 VNC 行、停止临时 HTTP 服务，不卸载用户已批准保留的 WorkBuddy。

## Rollback

- 产品代码回滚为单个变更提交的普通 revert；不重写分支、不恢复或覆盖用户其他改动。
- Windows 回滚不修改 WorkBuddy 安装：先对严格主 renderer 执行现有 restore，正常关闭调试实例并普通重开；若应用已退出，确认 `9224` 不监听即可。
- VM 测试结束移除本次临时加入的 `RemoteDisplay.vnc.*` 配置并停止临时 artifact 服务；保留现有 VMX 备份和 WorkBuddy 5.4.5 安装。
- 文档只在真实 Gate 通过后更新；若 runtime 实现通过但视觉/恢复失败，保持 Windows 支持不宣传并将 verdict 标记 blocked/pending。

## Deviations

无。当前 Plan 严格按已接受 Spec，复用现有平台 seam、主题 adapter 和 session 状态。任何需要 Windows 专属 CSS、通用 Electron 发现、持久调试设置、协议桥或新安装刷新服务的发现都必须先修订 Spec 并重新获批。

## Decision

待产品负责人明确接受本 Plan 后进入 Build。Plan 批准不等于授权中断虚拟机内正在运行的 WorkBuddy；相关动作仍在执行当下单独确认，最终验证与发布 Gate 保持关闭。
