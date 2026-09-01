---
id: "2026-08-31-remember-last-theme"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-31"
based_on: spec.md
risk: "high"
approved_by: "user"
approved_at: "2026-08-31"
---

# Plan: remember last theme

## Files and ownership

- `crates/skin-core/src/auto_theme.rs`、`crates/skin-core/src/lib.rs`：新增 v1 持久状态、严格校验、原子读写、每登录会话标记和可测试的 supervisor 决策；这是自动保持行为的唯一事实来源。
- `crates/skin-core/src/live.rs`、`crates/skin-core/src/live/platform.rs`：暴露明确的目标运行状态，并为 watcher 增加“目标退出即返回”策略；保留 CLI 现有兼容策略。
- `apps/desktop/Cargo.toml`、`apps/desktop/src/bin/doubao-skin-agent.rs`、`apps/desktop/Agent-Info.plist`：增加单一无界面 agent binary 与嵌套 Login Item bundle 元数据。
- `apps/desktop/src/app/platform.rs`：实现 macOS 13+ `SMAppService` 注册、注销、状态和打开系统设置的薄适配器；macOS 12/Windows 返回稳定 unsupported 状态。
- `apps/desktop/src/app/auto_theme.rs`、`apps/desktop/src/app/mod.rs`、`apps/desktop/src/app/types.rs`、`apps/desktop/src/app/theme_ops.rs`、`apps/desktop/src/app/helpers.rs`：连接设置加载、异步服务操作、成功应用提交、恢复默认清理、主 App watcher 策略和纯状态测试 seam。
- `apps/desktop/src/ui/widgets.rs`、`apps/desktop/src/ui/detail.rs`、`apps/desktop/src/i18n.rs`、`apps/desktop/src/ui_regression_tests.rs`：渲染两个可访问 switch、从属/系统状态和产品文案；不新建设置页或导航。
- `scripts/package/macos.sh`：host/universal 构建并嵌入 agent，版本同步，先签 helper 后签主包，并验证嵌套结构。Windows/Web 打包不改。
- `workflow/changes/2026-08-31-remember-last-theme/verification.md`：记录红绿测试、包结构/签名、真实窗口、系统状态、手动启动、主动退出、登录会话、恢复和剩余风险；实现者不自行填写 fresh-context 最终 verdict。
- 预计文件列表是当前获批设计的最小形状。若实现证明需要 XPC、socket、手写 LaunchAgent、macOS 12 兼容层、官方 App 修改或额外设置页，立即停止并更新 spec/plan 重新批准。

## Order of work

1. 核对 worktree 仅含本 change artifact、intent/spec 已获用户批准、当前版本/打包脚本和官方豆包/豆包工作只读状态；记录测试前自动主题配置、`SMAppService` 状态、目标运行/主题状态，不改系统设置。
2. 在 `auto_theme.rs` 先写配置红灯：缺失默认、v1 round trip、非法 target/opacity、损坏/未知 schema、父关子关、原子替换失败保留旧文件和 audit-session marker 去重。只运行定向测试，确认旧代码因模块/API 不存在而失败。
3. 实现最小 v1 structs 与同目录原子写入，使同一组配置测试转绿；不在桌面层复制 JSON 或路径逻辑。
4. 在 `live.rs` 先写运行策略红灯：旧 watcher 可保持兼容，`stop_on_target_exit` 在端口消失时返回且不调用 launch；为 `TargetApp::is_running()` 的平台边界增加 macOS/Windows 路径测试或可注入命令结果测试。
5. 实现运行策略与只读运行判断，更新桌面调用为退出即停止，CLI/示例保留旧默认；运行 live 定向测试和现有 CLI 集成测试。
6. 为 helper supervisor 写纯状态红灯：父关零启动、子关登录零启动、注册当前会话消费、每 audit session 一次、helper 重启不重复、主 App 优先、目标退出不重开、完整退出后新手动启动一次、错误端口进入有限错误状态。
7. 实现 `doubao-skin-agent` 串行循环：加载状态、观察主 App/目标、管理一个 stop flag/thread、记录 session marker、复用 `skin-core` live；SIGTERM/配置关闭时干净结束。禁止在 agent 内加入 UI、网络 listener 或第二份主题 loader。
8. 为 ServiceManagement 适配器先建立纯状态映射和 unsupported 红灯；实现 Objective-C runtime 可用性检查、`notRegistered/enabled/requiresApproval/notFound` 映射、注册/注销与打开登录项设置。非 bundle 开发启动必须返回产品化错误而不是尝试伪造 helper 路径。
9. 为桌面状态写红灯：无 last theme 不能开启、父关强制子关、pending approval 子项禁用、匹配 generation 的 `Applied` 才提交、失败不覆盖、恢复成功清状态/注销、恢复失败保留。实现异步 `Msg` 和状态更新后转绿。
10. 在 UI 回归 helper 中锁定两个 switch 的可用/请求/从属状态和动态目标文案，再在现有 detail panel 的预览与操作区之间加入一个紧凑设置组。使用 GPUI `Role::Switch`、`aria_toggled`、焦点/tab-stop；正常、compact、short 三条渲染路径共享同一组件。
11. 更新 i18n 与反馈：无主题、macOS 12、注册忙碌、等待系统批准、系统拒绝、已启用、已关闭、保存失败和“打开系统设置”。错误只显示可执行动作，不暴露内部路径/selector/端口。
12. 更新 Cargo/bin 与 macOS 打包：按架构构建 agent，创建 `Contents/Library/LoginItems/豆皮后台服务.app`，同步主包版本/minimum OS，universal 用各架构 agent `lipo`，显式先签 helper 后签主包。新增 shell 内联断言或现有打包后检查，不制造第二套打包脚本。
13. 运行 `cargo fmt --all`，再跑配置/live/agent/desktop 定向测试、`cargo test -p skin-core --lib --locked`、`cargo test -p doubao-skin-desktop --bin doubao-skin-app --locked`、`./scripts/check.sh rust` 与 `./scripts/check.sh workflow`；只在相关代码稳定后跑全门。
14. 构建 host macOS 包并机械检查两个 plist、agent/main 架构、bundle ID、`LSUIElement`、版本/minimum OS、`codesign --verify --deep --strict`、ZIP/DMG 校验。若当前稳定自签名身份可用，按既有脚本验证指纹；没有身份时只记录 ad-hoc 边界，不声称真实注册已通过。
15. 启动最终 bundle，在 1120×720、compact/short、浅色/深色下检查两个 switch、说明、从属禁用、系统状态、透明度和底部按钮；用键盘与 VoiceOver/辅助功能树确认 switch 角色、焦点和状态。证据只截豆皮窗口。
16. 在不注销当前会话的情况下做真实服务验收：记录原状态，开启父项，确认 `SMAppService.status` 与 helper；覆盖 `requiresApproval` 时的系统设置跳转；主 App 运行时 helper 让出，关闭主 App 后接管；关闭父项后注销并退出。测试后恢复原服务/配置状态。
17. 用无私人内容的空白目标页面做手动启动验收：父开子关、目标完全退出、直接点官方图标、最多一次受控重启、主题/透明度恢复；随后主动退出并观察至少 30 秒不重开。前后只读验证官方 App 签名，结束后恢复主题和窗口状态。
18. 两项都开时先在当前 audit session 验证“注册不立即打开”和 agent 重启不重复。真正“新登录会话主动打开一次”的验收会中断用户桌面，因此到达此步必须再次向用户取得即时注销/重新登录许可；未获许可时不得注销，必须在 verification 标为唯一未闭合的人工门而不是伪造通过。
19. 完成允许范围内的真实验收后运行 `./scripts/check.sh all`、最终 host 包检查和 `git diff --check`；搜索调试标记、硬编码用户名、临时 bundle 与隐私证据。把命令、结果、系统恢复、偏差和残余闪白写入 `verification.md`，交给 fresh-context verifier/人类记录 verdict。

## Test-first proof

- 配置首个红灯：`cargo test -p skin-core auto_theme::tests --locked`。旧树没有 `auto_theme` 模块；加入测试后必须先因缺少契约失败，再由最小持久化实现转绿。
- live 首个红灯：`cargo test -p skin-core stop_on_target_exit_never_relaunches --locked`。用不可达的隔离端口/注入 platform seam 证明端口消失返回而不是调用启动命令；测试不得触碰真实豆包。
- supervisor 红灯：`cargo test -p skin-core auto_theme::supervisor_tests --locked`。输入离散 snapshot 与 audit session，断言唯一动作；不靠 sleep、真实进程或字符串日志判断状态机正确。
- desktop 红灯：`cargo test -p doubao-skin-desktop auto_theme --locked`。测试服务状态映射、父子依赖、成功 generation 提交和恢复事务；系统 API 通过 fake adapter，不注册真实登录项。
- 包检查先对旧 `./scripts/package.sh desktop-macos` 产物断言 helper 缺失而失败，再修改 `macos.sh`；相同 bundle-path/plist/arch/signature 断言在新包转绿。
- 所有自动测试使用 `DOUBAO_SKIN_DATA_DIR` 与不可达 CDP 端口隔离，禁止再次发生测试误连真实目标和清理用户主题的历史问题。

## Visual or integration proof

- 豆皮 UI：最终安装包正常 1120×720、compact 宽度与 short 最小高度各验证一次；浅/深系统外观各抽查。截图不得包含内部日志或未来计划，只显示产品开关和必要系统反馈。
- 可访问性：Tab/Shift-Tab 到达两个 switch，Space/Enter 切换；父项关闭时子项跳过或宣布不可用；VoiceOver/AccessKit 显示稳定 ID、Switch 角色、名称、说明和 true/false 状态。
- ServiceManagement：从最终签名 bundle 调用注册/注销，记录四态中实际出现的状态、helper bundle/executable 的签名与运行 PID；系统批准必须由用户操作，不自动点击。
- 父开子关：重新启动 helper 不开目标；手动启动目标后最终根标记等于保存 theme/target，透明度匹配；受控重启计数不超过一次。主动退出后 30 秒内进程和端口保持关闭。
- 主 App/helper 交接：helper watcher 活动时打开豆皮，helper 在一秒轮询窗口后停止；应用另一个主题无旧主题重新覆盖；关闭豆皮后 helper 接管新主题。
- 父开子开：当前会话注册不触发启动，helper 同会话重启不触发；经即时批准的新登录会话只启动保存目标一次。若官方自身登录项产生竞态，允许一次受控重启但不得循环。
- 隐私/完整性：目标验收使用空白页，只记录 theme/target marker、端口归属与进程次数，不读取正文。`codesign --verify --deep --strict` 在官方豆包前后均成功，官方 bundle 没有仓库工具写入。

## Risks and mitigations

- **ServiceManagement 真实包要求未知。** 先完成自动/包验证，再在最终签名包注册；失败时保留明确状态并停下，不回退到手写 LaunchAgent 或降低签名要求。
- **注册即启动导致意外打开。** helper 启动先判断主 App 和 audit-session marker，当前会话先消费主动打开机会；这两条均有纯状态测试和真实当前会话复验。
- **用户退出被误判为需要重开。** 桌面/helper 使用 `stop_on_target_exit`，端口断开立即结束 watcher；supervisor 必须看到完整 stopped→started 转换才重新应用。
- **双 watcher 争用。** 主 App bundle 存活是最高优先级，helper 先 stop/join 再等待；UI 成功写盘只发生在匹配 generation，交接用真实换主题场景验收。
- **配置半提交。** 写入同目录临时文件、sync、rename；失败保留旧文件。恢复默认只有 DOM 恢复成功后才清配置/注销，失败不进入一半状态。
- **旧系统/跨平台回归。** 运行时检查 macOS 13，主包仍部署 12.0；Windows 不渲染/打包 agent，CLI run wrapper 保留旧策略。全 workspace all-targets 和 Windows CI 作为门禁。
- **固定窗口拥挤。** 设置组使用固定紧凑两行，预览保持现有 minimum；normal/compact/short 都必须真实渲染，不通过降低可读字号或隐藏说明制造通过。
- **官方登录项竞态。** 不修改官方设置；只允许一次身份确认后的重启并提示用户。若重启次数无法稳定限制，停止上线自动打开子项。
- **注销会话破坏用户工作。** 任何真实 logout/relogin 只在到达验收点后再次取得即时许可；无许可时保留清楚的人工门，不以 launchctl 模拟冒充登录成功。
- **测试污染真实用户状态。** 每个系统验收前记录豆皮配置、服务状态、目标进程/主题；测试后注销测试 helper、恢复设置/主题/窗口。临时产物移动到废纸篓，不强删用户文件。

## Rollback

- 运行时优先通过产品开关关闭自动保持，确认 `SMAppService.status=notRegistered`、helper 退出且目标不被重新打开；再用“恢复默认”只清理豆皮拥有的页面层。
- 若 UI 无法启动，使用当前构建中的最小注销诊断入口或已验证的系统“登录项”面板禁用 `dev.ichen.doubao-skin.agent`；不得删除系统数据库或编辑全局 plist。
- 用 `apply_patch` 移除新增 `auto_theme` 模块、agent bin/plist、ServiceManagement adapter、两个 switch、运行策略调用和 macOS 打包段；恢复现有 `live::run` 调用兼容。不得使用 `git reset --hard` 或 `git checkout --` 覆盖用户工作。
- 配置文件属于豆皮自身。回滚代码默认忽略 `auto-theme.json`，不强删；若用户明确要求清理，再把精确文件移动到废纸篓。官方豆包文件和用户数据无需迁移或回滚。
- 如果测试曾安装单独的豆皮测试 App，先注销其 helper，再把精确测试 bundle 移到废纸篓；不覆盖或删除既有 `/Applications/豆皮.app`。
- 回滚后运行 live/desktop 定向测试、`./scripts/check.sh rust`、workflow 和旧 macOS 包签名检查，证明手动应用/恢复与跨平台打包恢复原状。

## Deviations

- 实机交接发现：ServiceManagement 管理的 helper 仅依赖 `pgrep`/精确可执行路径时，能接管目标但无法稳定识别后来打开的主 App。实现仍先使用已批准的精确外层可执行路径，并增加 spec 已允许的固定主 bundle ID `dev.ichen.doubao-skin` 的 `NSRunningApplication` 回退；没有枚举窗口、标题或其他应用内容，也没有引入 IPC。新增回归测试后，最终包通过 helper 接管和主 App 重新接管的完整实机链路。
- 非注册临时探针已从 `/tmp` 移到用户废纸篓，没有进入仓库或系统登录项。其余实现未偏离获批 spec。

## Decision

等待工程与风险负责人明确批准本 plan 后开始产品代码、测试、打包和当前会话系统注册验收；真实注销/重新登录仍保留为到达该步骤时的即时人工许可门。
