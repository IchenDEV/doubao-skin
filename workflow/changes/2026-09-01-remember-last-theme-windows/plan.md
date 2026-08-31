---
id: "2026-09-01-remember-last-theme-windows"
stage: plan
status: accepted
owner: "codex"
created: "2026-09-01"
based_on: spec.md
risk: "high"
approved_by: "user"
approved_at: "2026-09-01"
---

# Plan: remember last theme windows

## Files and ownership

- `crates/skin-core/src/auto_theme.rs`：把登录会话标识从 macOS `u32 audit_session_id` 泛化为 `u64 login_session_id`，兼容读取旧 marker；保留唯一 supervisor 和原子持久化语义。
- `crates/skin-core/src/theme.rs`、`crates/skin-core/tests/bundled_theme_paths.rs`：证明位于 `helpers/` 的 Windows agent 能发现包顶层 `themes/`；不改变主题格式或 Web catalog。
- `apps/desktop/Cargo.toml`、`Cargo.lock`：仅为 Windows target 增加固定版本的 registry/token/mutex API 依赖与最小 feature；不扩大 macOS 依赖面。
- `apps/desktop/src/app/platform.rs`：实现 HKCU Run 的命令构造、状态、注册、回滚、注销和可选设置入口；保持 macOS `SMAppService` 分支不变并提供纯逻辑测试 seam。
- `apps/desktop/src/bin/doubao-skin-agent.rs`：把现有 macOS helper 循环提为共享实现，增加 Windows GUI subsystem、登录 LUID、命名 mutex、主程序精确路径和句柄清理适配。
- `apps/desktop/src/app/auto_theme.rs`、`apps/desktop/src/app/mod.rs`、`apps/desktop/src/app/types.rs`：把启用/禁用改为可补偿事务，连接 Windows 状态和错误，同时保留主程序 watcher 所有权。
- `apps/desktop/src/ui/widgets.rs`、`apps/desktop/src/i18n.rs`、`apps/desktop/src/ui_regression_tests.rs`：复用现有两个 switch，补 Windows“已注册/未就绪/路径过长”等有限文案和平台状态回归；不增加布局或第三个控件。
- `scripts/package/windows.sh`、`scripts/package/verify-windows-exe.sh`：把同架构 agent 放入 `helpers/`，参数化验证 GUI subsystem/Machine/icon，并检查一个顶层入口和一个 helper。
- `.github/workflows/ci.yml`、`.github/workflows/release.yml`：让 x64/x86/ARM64 原生 job 验证两个 PE、ZIP 布局和测试；不改变发布审批或创建 Release。
- `workflow/changes/2026-09-01-remember-last-theme-windows/verification.md`：记录红绿测试、命令、CI/PE/ZIP、Windows 窗口与登录证据、清理、偏差和 residual risk；最终 verdict 由 fresh-context verifier 或人类填写。
- 所有编辑叠加在当前未提交的 macOS 自动主题实现上。实现期间先检查目标 hunks，不覆盖或重排与本 Windows 纵切无关的用户修改。

## Order of work

1. **固定基线与失败样例。** 记录当前 `git status`、已接受 macOS change 和 Windows 不支持行为；先为旧 marker 兼容、`u64` session 和 `helpers/` 主题路径加入失败测试，不运行或修改系统启动项。
2. **泛化 core 契约。** 最小修改 `auto_theme` 会话 API 与 JSON alias，macOS 调用转换为 `u64`；运行 core 定向测试，证明旧 macOS 用户不会在同一会话重复打开。
3. **实现 Windows service adapter。** 先以纯 backend/seam 测试命令引用、UTF-16 长度边界、状态映射、幂等和失败补偿，再接 `windows-registry` 的 HKCU Run 薄实现。macOS/非 Windows 编译时不触碰真实注册表。
4. **实现跨平台 agent。** 抽取现有 load/start-watcher/supervisor loop，保留 macOS 路径与 bundle fallback；新增 Windows AuthenticationId、Local mutex、包内主程序路径和 GUI subsystem。每一步运行桌面 agent 单元测试，避免重写 supervisor。
5. **收紧 UI 事务。** 保存操作前 settings 快照；启用失败恢复配置并注销新值，禁用失败保持配置关闭并暴露未清理错误。连接 Windows 状态/文案和现有两个 switch，不新建平台 UI。
6. **扩展 Windows 包。** 同 target 构建/copy agent，验证 helper/main 的 PE Machine 与 GUI subsystem、顶层 exe 数量、主题解析和 ZIP 清单；保持现有 x86 SafeSEH 例外只作用于 x86。
7. **扩展原生 CI。** x64 运行 desktop/core Windows 测试，三架构运行构建/打包/解包检查；Release workflow 只验证产物结构，不越过 tag/发布审批。
8. **本地收敛。** 运行格式、core/desktop 定向测试、`./scripts/check.sh rust`、`./scripts/check.sh all` 和 workflow；macOS 包/现有 helper 相关回归在改动触及后重跑。Mac 的 MSVC SDK 缺失只记录，不反复尝试同一无效交叉构建。
9. **Windows 原生验收。** 从最终 ARM64（以及可用的 x64）ZIP 在 Windows 11 的中文/空格路径运行，依次验证父开子关、父子都开、主/helper 交接、重复 agent、手动启动、主动退出、锁屏/重新登录、包移动和外部删除启动值；每个场景前后获取无隐私证据。
10. **清理与审计。** 精确注销 `DoubaoSkinAutoTheme`、确认 helper 退出、恢复测试前主题/配置/窗口并验证官方文件未改；把实际命令、结果、CI/VM 边界和任何偏差写入 `verification.md`，再请求独立 verdict。

## Test-first proof

- **Core red test:** 现代码只能接受 `u32 audit_session_id`，先加入 `u64` Windows LUID、旧字段 alias 和新字段写回测试；初始编译/断言失败后再修改实现。
- **Theme-path red test:** 先断言 `<package>/helpers/doubao-skin-agent.exe` 解析到 `<package>/themes`；现有函数若不满足则保留失败输出，再做最小路径扩展。
- **Registration red tests:** 通过内存 backend 覆盖缺失/精确/陈旧 Run 值、Unicode/空格、引号拒绝、含 NUL 的 260 code-unit 边界、helper 缺失、重复注册、spawn 失败回滚、注销失败和相邻值不变；测试先于真实 registry adapter。
- **Helper red tests:** 给 Windows 包路径推导、主程序精确路径、同会话 mutex 已存在退出、AuthenticationId LUID 组合/零值拒绝和 handle guard 增加可在相应 cfg 下运行的测试。共享 supervisor 继续复用现有父优先/退出不重开测试，不复制状态机。
- **UI transaction red tests:** 模拟 register/unregister/save 的每个失败点，证明启用不虚假成功、启用失败恢复旧配置、禁用失败仍不再自动执行，以及 Windows 不出现 macOS approval 状态。
- 迭代命令以最小目标为主：`cargo test -p skin-core auto_theme`、`cargo test -p skin-core --test bundled_theme_paths`、`cargo test -p doubao-skin-desktop`；方向稳定后才运行完整 gates。
- Windows 原生 job 额外执行隔离 registry Unicode round trip，使用豆皮测试子键并由 guard 精确清理；CI 不写正式 Run 值。正式 HKCU Run 只在受控 VM 产品验收中使用并清理。

## Visual or integration proof

- 在 Windows 11 最终 ZIP 中以正常窗口和窄窗口检查：两个开关可见、父子禁用关系、注册中/已注册/未就绪/错误反馈、透明度和底部操作均可用；键盘 Space/Enter、焦点和 AccessKit 状态与 macOS 一致。
- 用注册表查询确认只存在 `HKCU\\...\\Run\\DoubaoSkinAutoTheme`，值精确引用当前 ZIP 的 quoted helper 路径；保留相邻测试值前后对比，证明未修改其他启动项。
- 用 Task Manager/PowerShell 的可执行路径、PID 和可见窗口信息证明 agent 为 GUI subsystem、无控制台/窗口/托盘，重复启动后仍只有一个有效 supervisor。
- 记录主程序打开时 helper watcher 为零、关闭后 helper 接管、重开主程序后 helper 先让出的进程/端口时间线；再从桌面/开始菜单启动目标，以实际豆包窗口截图确认主题和透明度恢复。
- 父开子关做一次完整注销/登录：无豆皮/豆包窗口，稍后手动打开才恢复。父子都开再做一个全新登录会话：目标最多启动一次。锁屏/解锁不得被误判为新登录。
- 主动退出目标后观察至少 30 秒；包移动和外部删除 Run 值后确认不自动重建。最后关闭父项，确认值删除、agent 在一个轮询周期内退出且当前主题不被清除。
- Windows VM 测试前后比较官方豆包可执行文件签名/哈希；截图只保留空白测试会话并裁掉账号、侧栏、用户名和完整用户路径。

## Risks and mitigations

- **公共 Run API 无法观测系统外部禁用：** 状态只叫“已注册”，不读 `StartupApproved`、不自动修复；真实登录行为单独验收并记录平台边界。
- **便携路径过长或移动：** 写入前按 UTF-16 + 引号 + NUL 检查 260 上限；移动后显示 `NotFound`，只在用户明确关/开后重写绝对路径。
- **文件/注册表/进程无法原子提交：** 用操作前 settings 快照和精确值名做补偿；逐个失败点测试。禁用失败优先保持行为关闭，不为了表面一致重新启用 helper。
- **重复 helper 或双 watcher：** Windows Local mutex 作为最终竞态门，精确主程序路径和共享 supervisor 管理所有 watcher；VM 用 PID/端口时间线证明单所有者。
- **登录 ID 或 Win32 handle 错误：** 只使用 token AuthenticationId；零值/API 失败时禁止主动打开。token/mutex 都由小 RAII guard 在所有 return path 关闭，Windows 原生测试覆盖。
- **无控制台只在 Windows 才可证实：** 同时检查源码 subsystem 属性、PE header 和真实启动窗口；任一失败都不交付。
- **三架构打包混用：** verifier 接收明确 expected Machine，GUI 与 agent 从同 target 输出复制；x64/x86/ARM64 ZIP 分别解包检查。
- **macOS 回归：** helper 抽共享循环但平台 API 保持薄分支；先跑现有 macOS agent/core 测试，再跑完整 Rust/all gates。若共享化扩大风险，退回同文件内共享函数，不引入通用后台框架。
- **远端 CI/VM 可用性：** 本地不能伪造原生通过。若需要推送分支触发 CI 或操作登录会话，将在到达该步骤时按现有授权边界执行；未取得的证据明确标为阻塞，不用 Mac 交叉失败替代。

## Rollback

- 运行时首先在当前测试包关闭“自动保持上次主题”，确认配置关闭、固定 Run 值消失和 agent 退出；当前页面主题保持不变，只有用户选择“恢复默认”才清页面样式。
- 若 GUI 无法注销，先只读确认精确键/值和路径，再删除 `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run` 下单个 `DoubaoSkinAutoTheme` 值；不得删除整个 Run key、其他值、StartupApproved 或系统策略。
- 若 agent 未自行退出，只在验证 PID 的可执行路径等于测试包 `helpers\\doubao-skin-agent.exe` 后结束该精确进程；不得按 `doubao*` 模糊批量结束。
- 代码回滚用 `apply_patch` 撤销 Windows registry/token/mutex 分支、helper 打包、CI 清单和 UI 文案，恢复非 macOS `Unsupported`；保留已接受且已验证的 macOS 自动主题实现。不得用 `git reset --hard`、`git checkout --` 覆盖用户工作树。
- `auto-theme-session.json` 的新字段与旧字段兼容，回滚代码读取不到新字段时可能安全报错。因此如确需代码回滚，先关闭自动化；配置文件默认保留，不强删用户数据。只有用户明确要求时才移动精确 marker 到废纸篓。
- 回滚后重新运行 core/desktop 定向测试、`./scripts/check.sh rust`、workflow 和现有 macOS package 检查；Windows ZIP 恢复原先一个 GUI 的结构，未发布的测试产物不上传或创建 Release。

## Deviations

- 当前无偏差。实现若需要 Startup shortcut、Task Scheduler、服务、额外 UI、第二状态机、自动修复外部禁用或修改官方豆包文件，必须停止并回到 Spec 重新批准，不得记成普通实现细节。

## Decision

等待用户明确批准本 Plan 后才开始红测试、产品代码、Windows 包和受控系统验收。Plan 批准不授权发布、合并、创建 Release 或跨越生产审批门。
