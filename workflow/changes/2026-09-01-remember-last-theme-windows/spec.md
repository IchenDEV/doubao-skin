---
id: "2026-09-01-remember-last-theme-windows"
stage: spec
status: accepted
owner: "codex"
created: "2026-09-01"
based_on: intent.md
risk: "high"
approved_by: "user"
approved_at: "2026-09-01"
---

# Spec: remember last theme windows

## Requirements

1. Windows 10/11 的桌面应用必须显示并支持现有两个开关：“自动保持上次主题”和“登录时打开豆包”。不得增加 Windows 专用设置页、第三个开关、托盘图标或后台窗口；macOS 两开关和既有语义保持不变。
2. “登录时打开豆包”继续依赖“自动保持上次主题”。父项关闭时子项必须同时持久化为关闭并不可操作；父项开启、子项关闭时，Windows 登录只启动无窗口 helper，不主动打开豆包或豆包工作。
3. 首次成功应用主题、父子开关、恢复默认、失败不覆盖和主程序 watcher 行为继续以已接受的 `2026-08-31-remember-last-theme` 规格为准。Windows 实现不得新增第二份配置、主题加载器或 supervisor 状态机。
4. 开启父项必须为当前 Windows 用户注册一个精确、可撤销的登录启动值并立即启动随包 helper。只有启动值与当前包内 helper 的预期命令完全一致、helper 文件存在且配置已成功写入时，UI 才可显示已注册；不请求管理员权限。
5. Windows helper 必须使用 GUI PE subsystem，运行时不得出现控制台、窗口、任务栏项或托盘图标。每个 Windows 登录会话只允许一个 helper supervisor；重复启动必须立即、无副作用退出。
6. helper 必须在豆皮主程序运行时让出 watcher，主程序退出后接管。后续从桌面、开始菜单或官方入口手动启动已保存目标时，最多执行一次受控重启并恢复最后成功主题；用户主动退出目标后至少等待下一次新的启动转换，不自行重开。
7. 子项开启时，每个 Windows 登录会话最多主动启动一次保存目标。豆皮刚注册并立即启动 helper 的当前会话必须标记为已消费，不因打开开关马上弹出豆包；helper 崩溃/重启、主程序开关和官方自身启动项竞态不得重复启动。
8. 关闭父项或成功恢复默认必须先写入禁用配置，再精确删除豆皮拥有的当前用户启动值，并让 helper 自行退出；不得删除相邻注册表值、结束模糊同名进程、移除当前页面主题或修改官方豆包启动设置。
9. Windows ZIP 移动、路径含空格或中文、重复开启/关闭、目标/主题缺失、配置损坏、注册表值缺失或陈旧、helper 崩溃均必须安全失败或幂等恢复。不得产生启动循环、双 watcher、半写注册表或虚假成功状态。
10. x64、x86、ARM64 的 Windows 原生 CI 必须构建对应 GUI、CLI 和 helper；真实 Windows 11 VM 必须完成注册、无窗口、主/helper 交接、手动启动、登录去重、退出保持和清理验收。macOS 上缺少 MSVC SDK 的交叉编译结果不能替代原生证据。

## User experience

- Windows 复用 macOS 已实现的同一个紧凑设置组、同一标题/说明、父子层级、键盘焦点和 AccessKit `Switch` 语义，不新增平台分支布局。正常与窄窗口仍必须看到主题操作、透明度和两个开关。
- 没有最后成功主题时，父项仍不可开启并提示先应用主题。父项成功开启后，反馈表述为“已注册后台启动”；它表示豆皮已写入当前用户登录启动项，不声称能够读取 Windows 设置中未公开的外部禁用状态。
- 父开子关时，关闭豆皮窗口不会改变当前主题；下次 Windows 登录不显示豆皮或豆包窗口。用户稍后直接点官方豆包入口，允许短暂出现默认外观，随后最多一次受控重启并恢复主题。
- 父子都开时，下一次新的 Windows 登录会话由 helper 最多主动打开保存目标一次。打开开关的当前会话不立即打开；关闭子项不注销 helper，关闭父项才注销。
- 注册表值被外部删除或包被移动后，UI 显示“后台启动项未就绪”，不静默重建。用户在当前新位置明确关闭再开启父项后才写入新路径；这也尊重用户在系统外部作出的禁用选择。
- Windows 不显示 macOS 的 `requiresApproval` 文案。必要的诊断入口可以打开 Windows“启动应用”设置页，但它不是第三个开关，也不能被当作可查询的批准状态。
- 错误使用产品语言说明“无法注册后台启动”“安装路径过长”或“当前包缺少后台服务”；不得暴露注册表原始值、用户完整路径、Windows token、LUID、句柄或系统错误栈。

## Technical design

### Current-user startup registration

- Windows adapter 使用 `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run`，固定值名 `DoubaoSkinAutoTheme`，数据是仅含绝对路径且带双引号的命令：`"<package>\\helpers\\doubao-skin-agent.exe"`。不带 shell、参数、环境变量或工作目录依赖。
- 选择 HKCU `Run` 是因为它是 Windows 公开的当前用户每次登录启动机制，无需管理员权限。Microsoft 说明该值在用户登录时运行，但运行顺序不保证、系统可能延迟，命令行上限为 260 个字符：[Run and RunOnce registry keys](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)。
- 注册前必须验证 helper 路径为绝对现存文件、没有嵌入引号，并确保包含外围双引号和结尾 NUL 后不超过 260 个 UTF-16 code units。超限时不写注册表、不改成功状态，提示用户把便携包移到更短路径。
- `status()` 的 Windows 映射为：值缺失是 `NotRegistered`；helper 缺失或值与当前绝对命令不同是 `NotFound`；两者完全匹配是 `Enabled`；`RequiresApproval` 不在 Windows 产生。`Enabled` 只表示公开 Run 注册契约成立。
- `register()` 幂等写入豆皮拥有的固定值，并立即启动当前 helper。若同会话 helper 已存在，精确路径检查或 helper 单实例门使重复实例无副作用退出。写入后启动失败必须删除该固定值并返回失败，不留下新启动项。
- `unregister()` 只删除 HKCU Run 下固定值名 `DoubaoSkinAutoTheme`；值不存在视为成功，不枚举或修改其他启动项。它不强制结束进程，helper 从禁用配置退出；若删除失败，配置保持禁用并显示“后台启动项未移除”，下次显式关闭动作可重试。
- 可选“打开设置”动作使用 `explorer.exe ms-settings:startupapps` 的直接进程参数，不使用 `cmd.exe`/PowerShell。实现和 UI 不读取、写入或推断未文档化的 `StartupApproved` 注册表数据，也不因外部删除而自动修复。

### Shared helper and Windows platform adapter

- 将 `doubao-skin-agent` 从 macOS-only 模块拆为一个共享 supervisor 循环和两个很薄的平台适配器。共享层继续调用 `skin_core::auto_theme`、`theme::list_installed`、`live::run_with_policy(... PortLossPolicy::Stop ...)`，不复制目标启动、身份校验或主题注入逻辑。
- 源文件对 Windows 设置 `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]`。Windows helper 从自己的 `<package>\\helpers` 位置严格推导同包顶层 `doubao-skin.exe`，只用精确规范化可执行路径判断主程序是否运行；不按窗口标题、模糊进程名或安装目录扫描。
- Windows helper 用当前登录会话内的命名 mutex（`Local\\dev.ichen.doubao-skin.agent`）保证单 supervisor。创建时若返回 `ERROR_ALREADY_EXISTS`，新进程立即成功退出；持有实例退出时关闭句柄。不得使用跨用户 `Global\\` mutex、系统服务或锁住可移动包文件。
- 父项注册后立即启动 helper；helper 看到主豆皮正在运行时不创建 watcher，并把当前登录会话的子项主动打开机会标记为已消费。父进程退出后 helper 重新读取最终配置并按既有 supervisor 接管。
- 关闭父项时 UI 先原子保存 `keep_requested=false`（现有 setter 同时关闭子项），再注销 Run 值。helper 最多在下一次一秒轮询退出。helper 意外退出不会自动由主程序反复拉起；后续登录由 Run 值启动，用户显式重新开启时才立即再启动。

### Login-session identity and persisted compatibility

- Windows 登录会话标识取当前进程 access token 的 `TOKEN_STATISTICS.AuthenticationId`。实现通过 `GetCurrentProcess`、`OpenProcessToken(TOKEN_QUERY)` 和 `GetTokenInformation(TokenStatistics)` 读取 `LUID`，组合为非零 `u64`，并在所有路径关闭 token handle。Microsoft 将 `AuthenticationId` 定义为标识 token 所代表登录会话的 LUID：[TOKEN_STATISTICS](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-token_statistics)、[GetTokenInformation](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-gettokeninformation)。
- `skin-core` 的会话消费 API 从 macOS 专用 `audit_session_id: u32` 泛化为 `login_session_id: u64`。macOS 将现有 audit session ID 无损转换为 `u64`；Windows 使用 AuthenticationId。去重、主程序运行时消费和原子写入语义不变。
- `auto-theme-session.json` 继续接受已有 schema v1 的 `audit_session_id` 字段作为只读 alias，并以新的 `login_session_id` 字段写回；不得让升级后的 macOS 用户再次触发同一登录会话的“登录时打开”。损坏、未知 schema 或零标识继续安全停止且保留原文件。

### Configuration transaction and recovery

- 启用是一个带补偿的跨文件/注册表事务：保存启用配置 → 写入/校验 Run 值 → 立即启动 helper → 刷新状态。后两步失败时删除新写 Run 值并恢复操作前配置；只有全部成功才显示已注册。
- 禁用优先保证“不再执行”：先保存禁用配置，再删除 Run 值。删除失败不把配置重新改回启用，因为那会让已运行 helper 继续工作；UI 保持父项关闭并持续显示未清理错误，允许重试精确注销。
- 成功恢复默认沿用相同禁用/注销序列。配置保存失败时不得先删除最后主题或宣称清理完成；注册/注销操作继续在后台线程执行，不阻塞 GPUI 渲染。
- 包移动后旧路径不会被后台静默重写。新位置主程序看到 `NotFound` 时，保留用户请求状态但将服务视为未就绪；用户明确切换关闭会清理固定值，再次开启写入新路径。该恢复不触碰旧包、官方豆包或其他 Run 值。

### Package layout and theme discovery

- 三个 Windows ZIP 均保持一个顶层入口：

  ```text
  Doubao-Skin-Windows-<arch>/
  ├── doubao-skin.exe
  ├── helpers/
  │   └── doubao-skin-agent.exe
  ├── themes/
  └── licenses/
  ```

- Windows package 脚本与 `cargo build --package doubao-skin-desktop` 同次构建 helper，把同 target triple 的 agent 放入 `helpers/`。现有“恰好一个顶层 exe”断言保持不变，新增“恰好一个 helpers agent”断言。
- PE 验证脚本以最小参数化复用：主 GUI 继续检查 icon/group-icon、GUI subsystem 和目标 Machine；helper 至少检查 GUI subsystem 和同一 Machine，不假定一定携带应用图标。ZIP 解包后再次验证相对布局、两个 PE 架构和 SHA-256。
- 现有主题目录解析必须补回归：从 `helpers/doubao-skin-agent.exe` 出发能找到同包顶层 `themes/`。helper 不复制主题资源，不访问当前工作目录，不联网下载主题。

### Dependencies and CI

- Windows 注册表使用仓库已采用的 `windows-registry`；token、mutex 与进程 API 使用固定版本 `windows-sys`，只启用所需的 `Win32_Foundation`、`Win32_Security`、`Win32_System_Threading` 等最小 features。不引入 crate 级跨平台进程管理框架。
- 纯函数/trait seam 覆盖命令构造、路径/长度校验、状态映射、补偿动作和重复注册，不在 macOS 测试机写真实 Windows 注册表。Windows 原生测试可在隔离的豆皮测试子键完成 Unicode round trip，并在 finally/guard 中精确删除。
- `windows-2025` 的 x64/x86/ARM64 matrix 均构建和打包 agent；原生检查验证每个 target 的两个 PE。至少 x64 跑 Windows 平台单元测试；三架构产物都执行解包结构检查。
- 最终行为必须在真实 Windows 11 VM 中从最终 ZIP 验收。优先覆盖 ARM64 VM；若没有 x64 VM，x64/x86 的运行风险必须由原生 CI 与 PE 检查明确记录，不能伪称三架构均已实机运行。

## Security and privacy

- 所有持久系统修改严格限于当前用户 HKCU Run 下一个固定、豆皮拥有的值；不写 HKLM、Services、Task Scheduler、Startup folder、系统策略、官方豆包目录或其他用户 profile。
- 注册命令不经过 shell，完整路径总是加引号并拒绝内嵌引号/超长值，避免空格、中文和命令拼接问题。注册表/API 错误只映射为有限中文产品错误。
- helper 无 listener、IPC、遥测或管理员权限。它继续只连接保存目标的专属 loopback CDP 端口，复用端口归属与目标可执行身份检查；不读取聊天正文、Cookie、账号、窗口标题或启动应用列表。
- helper 单实例和主程序检测只使用当前会话 mutex 与精确可执行路径，不结束其他 helper 或模糊同名进程。受控退出/重启的最大影响仍限于已验证的保存目标。
- VM 验收使用无私人内容的测试页；注册表、配置、进程和截图证据裁掉用户名/完整用户路径。测试前后记录官方豆包可执行文件签名/哈希未变化。

## Alternatives and non-goals

- 不使用 Startup folder `.lnk`：它需要额外 COM/快捷方式生成与清理逻辑，对当前“一个精确命令”没有优势；若公开 Run 机制在原生验收失败，必须回到新的 Spec 决策，不在实现中临时切换。
- 不使用 Task Scheduler、Windows service、HKLM、管理员提权或 MSIX `StartupTask`：当前产品是便携 ZIP，这些机制扩大权限、安装和卸载面。
- 不注册主 `doubao-skin.exe`：它会创建窗口，且无法满足“父开子关时登录无豆皮窗口”；继续使用职责单一的无窗口 helper。
- 不读取或写入未公开的 `StartupApproved` 状态，不绕过用户在 Windows“启动应用”中的外部禁用。公开 Run 值与真实下次登录行为之间的差异记录为平台限制。
- 不做安装器、自动更新迁移器、托盘常驻、通知、日志查看器、通用单实例框架或跨用户后台服务。
- 不承诺登录顺序、启动延迟或用户手动点官方图标时零帧默认外观；只承诺单 supervisor、最多一次受控重启和最终恢复。

## Areas of concern

- **Windows 外部禁用不可观测。** Windows 设置能阻止已注册启动项，但没有与本便携应用契约匹配的公开查询 API。UI 只能准确报告 Run 值是否注册；VM 必须验证真实登录行为，文案不得把注册等同于系统保证运行。
- **Run 命令长度。** Microsoft 的 260 字符上限包含便携包完整路径。深目录即使文件可运行也可能不能可靠注册；实现必须在写入前拒绝，而不是截断或创建 shell wrapper。
- **登录启动竞态。** Run 项顺序不保证且可能延迟。官方豆包先启动时，helper 可能进行一次可见重启；命名 mutex、session marker 和 supervisor 必须防止多个 helper/目标循环。
- **便携包移动。** Run 值保存绝对路径，移动后旧值失效。自动重写会覆盖用户外部禁用意图，因此恢复需要用户在新位置明确关闭/开启；此限制需要在错误反馈中说清。
- **父/helper 交接。** Windows 没有 macOS bundle-ID 回退，精确进程路径观察必须在 Windows 原生环境验证。最多一秒的轮询交接窗口不能出现两个活动 watcher；helper 必须先停自己的 watcher再等待。
- **AuthenticationId 生命周期。** LUID 适合标识登录会话，但 Fast User Switching、RDP 和锁屏/解锁需要实机证明：锁屏不应新开，真正新登录才消费新的 ID。API 失败必须停止主动打开，不以 PID、时间或随机数代替。
- **无控制台保证。** `windows_subsystem=windows`、PE header 和真实双击/登录启动三者都要验证；仅凭源属性或没有 stderr 输出不足以验收。
- **三架构差异。** x86 `/SAFESEH:NO` 既有例外不能扩散到其他 target。helper 与 GUI 必须来自同一 triple，不能把 x64 helper 混入 ARM64/x86 ZIP。
- **跨资源事务不是原子事务。** 文件、注册表和进程无法一次原子提交；实现必须有确定补偿与下一次启动的可见状态，测试每个失败点，不能靠“通常成功”。

## Acceptance criteria

1. 状态/事务测试覆盖：缺失值、精确值、陈旧路径、Unicode/空格、内嵌引号、260 UTF-16 边界、helper 缺失、重复注册/注销、写入失败、启动失败回滚、注销失败保持禁用和不修改相邻值。
2. 会话测试覆盖：旧 `audit_session_id` marker 兼容、新 `u64 login_session_id` round trip、零/损坏/未知 schema、同 AuthenticationId 一次、不同 ID 再次、注册当前会话消费、锁屏等不制造伪会话。
3. supervisor/helper 测试覆盖：Windows 单实例、父进程运行时零 watcher、父退出后接管、父重开先让出、子关登录零目标启动、子开每登录一次、helper 重启不重复、用户退出不重开、下一次手动启动只恢复一次。
4. Windows UI 测试覆盖：两个开关可用、父子依赖、`NotRegistered/Enabled/NotFound`、忙碌/失败反馈、无 macOS approval 文案、AccessKit switch/disabled 状态；macOS `Unsupported/SMAppService` 既有测试不回归。
5. `windows-2025` x64/x86/ARM64 均成功构建 GUI、CLI 和 agent；三个 ZIP 各有且仅有顶层 `doubao-skin.exe` 与 `helpers/doubao-skin-agent.exe`，两个 PE 的 Machine 匹配包标签且均为 GUI subsystem，主题/许可/校验和契约通过。
6. 最终 ZIP 在 Windows 11 VM 的空格和中文路径中可启动。开启父项后 HKCU Run 只有精确固定值，agent 进程存在且没有控制台/窗口/托盘；重复开启或手动重复启动仍只有一个 supervisor。
7. 当前会话开启父项不会启动目标。关闭主豆皮后 helper 接管；手动从桌面/开始菜单启动保存目标，最多一次受控重启后恢复正确主题、透明度和目标，官方可执行文件未改变。
8. 主豆皮重开时 helper watcher 停止且 UI watcher 独占；再次关闭后 helper 使用最终保存主题接管。进程/端口/视觉证据证明任一时刻最多一个 watcher，而不是只靠日志推断。
9. 父开子关的新登录会话没有豆皮/豆包窗口；稍后手动启动可恢复。父子都开的全新登录会话最多主动打开一次；helper 重启、锁屏解锁和官方自身启动竞态不产生重复目标或循环。
10. 用户主动退出目标并观察至少 30 秒不重开；父项关闭后 Run 值消失、helper 在一个轮询周期内退出且当前页面主题不被移除。恢复默认成功时同时清最后主题和自动化，失败时保持可恢复状态。
11. 包移动/外部删除 Run 值后不自动重建；UI 显示未就绪。用户在新位置明确关闭/开启后更新为新绝对路径，旧路径不再存在，其他 Run 值和官方启动设置完全不变。
12. 验收结束精确删除 `DoubaoSkinAutoTheme`、退出测试 helper、恢复测试前配置/主题/窗口；`cargo fmt --check`、定向测试、`./scripts/check.sh rust`、桌面测试、`./scripts/check.sh all`、`./scripts/check.sh workflow` 全部通过，`verification.md` 记录原生 CI、VM、注册表/PE/窗口证据、环境边界、残余闪白与 fresh-context verdict。

## Decision

等待用户明确批准本 Spec 后再进入 Plan。当前只确定公开 HKCU Run、Windows token AuthenticationId、无窗口单实例 helper、便携包布局和验证边界；尚未修改 Windows 产品代码、注册表或系统启动状态。
