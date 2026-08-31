---
id: "2026-08-31-remember-last-theme"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-31"
based_on: intent.md
risk: "high"
approved_by: "user"
approved_at: "2026-08-31"
---

# Spec: remember last theme

## Requirements

1. macOS 13 及以上的桌面应用必须显示且只新增两个主题生命周期开关：“自动保持上次主题”和“登录时打开豆包”。不得为本功能新增设置页、菜单栏图标或第三个用户开关。
2. “登录时打开豆包”是“自动保持上次主题”的从属项。父项关闭时子项必须同时写为关闭、显示为不可操作，且任何后台进程不得主动启动目标应用。
3. macOS 12 保持当前手动应用能力，两个自动化开关显示为不可用并说明需要 macOS 13 或更高版本；不得提高整个 App 的 `LSMinimumSystemVersion=12.0`。Windows 不显示本组 macOS 设置且现有行为不变。
4. 每次主题首次成功注入后，应用必须原子保存最后成功的目标应用、主题 ID 和最终表面透明度。注入失败、中途取消、仅选择或仅预览主题不得覆盖旧记录。
5. 开启“自动保持上次主题”必须使用嵌入主 App 的无界面 Login Item helper 和 `SMAppService` 注册；系统状态为 `enabled` 前不得声称已生效。`requiresApproval`、`notRegistered` 与 `notFound` 必须映射为明确状态和可执行反馈。
6. helper 注册后可以随登录运行，但“登录时打开豆包”关闭时不得因登录、helper 启动、helper 重启、目标退出、CDP 断开或豆皮窗口关闭而主动打开目标应用。
7. 用户从 Dock、Finder、桌面或 URL 打开已保存目标时，helper 必须识别目标进程。若专属 loopback CDP 端口不存在，允许对已验证身份的目标执行一次正常退出、必要时结束残留进程、再带现有调试参数启动；同一轮不得重复重启。
8. 用户主动退出目标应用后必须保持退出。helper 需要先观察到目标已完全停止，再等待一次新的“未运行 → 运行”转换；不得把端口断开直接解释为重新打开请求。
9. “登录时打开豆包”开启时，每个 macOS 登录会话最多主动打开一次保存目标。首次注册发生在豆皮主程序仍运行的当前会话时不得立即打开目标；helper 崩溃或被系统重启也不得在同一登录会话重复打开。
10. 豆皮主程序运行期间继续由当前 UI 进程独占实时 watcher；helper 检测到主 bundle `dev.ichen.doubao-skin` 正在运行时停止自己的 watcher 并等待。主程序退出后 helper 才读取最终保存状态并接管，禁止两个 watcher 用不同主题互相覆盖。
11. 用户关闭“自动保持上次主题”时，应用必须写入关闭状态、强制关闭子项、注销 Login Item 并使仍在运行的 helper 自行结束，但不得移除当前页面主题。用户选择“恢复默认”且恢复成功后，必须清除最后主题、关闭两个自动化设置并注销 helper，避免后台再次应用刚恢复的主题。
12. 未安装目标、主题缺失或损坏、配置 schema 未知、端口属于其他应用、系统拒绝后台项、主 App 不在正确 bundle 中或 helper 签名/路径不合法时必须安全停止并给出有限错误；不得删除未知配置、扫描聊天 DOM 或进入启动循环。

## User experience

- 两个开关放在现有主题预览与底部主题操作之间的一个紧凑设置组中，不增加导航层级。每行使用标题、单行说明和右侧 switch；整行可点击，switch 支持键盘焦点、Space/Enter 与 VoiceOver `Switch`/开关状态。
- 第一行文案为“自动保持上次主题”，说明为“关闭豆皮后，下次打开仍会恢复当前主题”。没有任何成功应用记录时该项不可开启，并提示先应用一个主题。
- 第二行文案为“登录时打开豆包”，说明动态使用当前保存目标名称，例如“登录 Mac 后自动打开豆包工作”。父项未开启或系统尚未允许后台项时，整行降级显示且不响应点击。
- switch 表示用户请求状态；服务状态另以同组内的一行短反馈呈现。等待系统批准时显示“需要在系统设置中允许豆皮后台运行”，并提供“打开系统设置”文字操作；不得用绿色、勾号或“已开启”描述未获批准的状态。
- 成功状态只显示轻量的“已开启”或不额外提示；注册、注销和状态刷新不得冻结窗口。错误继续使用现有产品语言，不暴露 selector、端口、plist、helper 路径或原始系统错误栈。
- 父项开启、子项关闭时，开机只启动不可见 helper，不显示豆包或豆皮窗口。用户稍后直接点官方豆包图标，可能短暂看到默认外观并被受控重启一次，最终恢复保存主题。
- 两项都开启时，后续登录会话由 helper 主动启动保存目标并恢复主题。若官方豆包自身也配置开机启动，豆皮只提示可能重复启动，不修改对方或系统设置。
- 关闭父项不会立即改变已经显示的主题；“恢复默认”才移除页面主题。这个区别必须通过开关说明、操作结果和真实行为保持一致。
- macOS 交互遵循 Apple HIG 的 [Toggles](https://developer.apple.com/design/human-interface-guidelines/toggles/)、[Settings](https://developer.apple.com/design/human-interface-guidelines/settings/) 与 [Feedback](https://developer.apple.com/design/human-interface-guidelines/feedback/)：使用明确的二态控件、显示依赖关系，并及时说明需要用户完成的系统动作。

## Technical design

### Persisted state

- 在 `skin-core` 新增一个小型 `auto_theme` 模块，权威文件为 `theme::app_data_dir()/auto-theme.json`。schema v1 只包含：`schema_version`、`last_applied`、`keep_requested`、`open_at_login`；`last_applied` 包含 `target`、`theme_id` 和可选 `surface_opacity`。
- 读取时严格验证 schema、目标 ID、非空主题 ID和 `surface_opacity` 的有限数值/范围；缺失文件返回默认关闭状态，损坏或未知版本返回错误且保留原文件。
- 保存使用同目录临时文件、`sync_all` 和原子 `rename`；测试通过 `DOUBAO_SKIN_DATA_DIR` 隔离。所有 setter 强制 `keep_requested=false => open_at_login=false`。
- UI 在启动时读取设置和最后主题，但不把“曾经成功应用”伪装成当前页面仍已应用。`active_theme` 继续表示本进程观察到的实时状态。

### macOS service adapter

- 在 `apps/desktop` 的 macOS platform seam 中加入最小 `AutoThemeService` 适配器。通过 Objective-C runtime 在 macOS 13+ 获取 `SMAppService.loginItem(identifier:)`，调用注册、注销、状态和打开“登录项”系统设置；其他平台和 macOS 12 返回明确 `Unsupported`，不增加旧 `SMLoginItemSetEnabled` 兼容路径。
- helper bundle ID 固定为 `dev.ichen.doubao-skin.agent`，位于主包 `Contents/Library/LoginItems/豆皮后台服务.app`，`LSUIElement=true`，无 Dock 图标和窗口。Apple 文档要求 login-item bundle 位于该目录，并提供 `notRegistered/enabled/requiresApproval/notFound` 四种可查询状态：[SMAppService](https://developer.apple.com/documentation/servicemanagement/smappservice)、[Status](https://developer.apple.com/documentation/servicemanagement/smappservice/status-swift.enum)。
- `apps/desktop` 增加第二个 Rust binary `doubao-skin-agent`。共享配置、目标识别和 live 行为继续来自 `skin-core`；helper 本身不依赖 GPUI 状态，不开网络服务、不提供菜单栏或 IPC。
- 2026-08-31 的非注册临时探针已证明：在 `MACOSX_DEPLOYMENT_TARGET=12.0` 下可同时编译 `SMAppService` 可用性分支与旧符号引用；`Contents/Library/LoginItems/*.app` 的两个 plist 可通过 `plutil`，嵌套 helper 先签、主包后签可通过 `codesign --verify --deep --strict`。探针只实例化 service 对象，没有调用注册/注销；真实注册仍是实现后的必需验收，不能以此探针代替。

### Watcher ownership and launch policy

- 为现有 `live::run` 增加一个显式运行策略 seam，区分“旧 CLI watcher 可在断开后重启”和“自动保持 watcher 在目标退出后返回”。CLI 与现有示例默认保持当前兼容行为；桌面主程序和 helper 使用 `stop_on_target_exit`，满足用户主动退出不重开的新产品契约。
- `TargetApp` 暴露只读 `is_running()`，沿用已解析的目标二进制/Bundle 身份，不按模糊应用名匹配。helper 只有观察到目标从未运行变为运行时才调用 live 路径；`ensure_running` 仍负责对“已运行但没有正确端口”的实例执行现有单次受控重启和目标页身份确认。
- helper 主循环每秒检查：配置是否仍请求保持、主豆皮 bundle 是否正在运行、保存目标是否运行、当前 watcher 是否需要停止。所有启动动作由一个串行 supervisor 发出；不并行启动两个目标或两个 watcher。
- helper 启动时使用当前 macOS audit session ID 作为登录会话标识，并在 app data 中原子记录本会话是否已消费“登录时打开”动作。注册时主 App 仍运行的会话立即标记为已消费；新登录会话且主 App 未运行、`open_at_login=true` 时才主动启动一次。相同 helper 的崩溃重启读取同一标记，不重复启动。
- helper 观察到主 App 运行时设置其 watcher stop flag 并等待线程结束；主 App 退出后重新读取持久状态。这样 UI 应用新主题时只有 UI watcher 工作，成功写盘后 helper 再接管最终版本，无需新增 XPC、socket 或锁文件协议。

### Desktop state and packaging

- `SkinApp` 新增已保存设置、服务状态、服务操作进行中和 pending successful apply 的最小字段；服务注册/注销在后台线程执行，通过现有 `Msg` 通道返回，渲染线程只更新状态。
- `Msg::Applied(generation)` 仅在 generation 匹配时提交 pending `last_applied`；写盘失败不撤销页面已应用事实，但显示“主题已应用，无法保存自动恢复设置”，且不覆盖旧文件。
- 两个 switch 复用 GPUI/AccessKit 已有的 `Role::Switch`、`aria_toggled`、焦点和 tab-stop 能力，抽成一个仅服务本设置组的渲染 helper；不引入 UI 组件库。
- macOS host/universal 打包同时构建 agent binary，把对应架构或 `lipo` 结果放进嵌套 Login Item bundle，先签 helper 再签主包；版本号和最低系统版本与主包同步。Windows 打包不复制 agent。
- 最终包测试必须解析两个 plist、验证 helper 可执行文件架构、bundle ID、`LSUIElement`、版本同步和深度签名；仅从 `cargo run` 启动的非 bundle 开发版本不得注册服务。

## Security and privacy

- helper 以当前登录用户身份运行，不请求管理员权限，不安装 daemon，不写 `/Library`、`~/Library/LaunchAgents` 或官方 App bundle。
- 继续只连接 `127.0.0.1` 的目标专属端口，连接前确认端口页面属于保存目标。后台配置不保存账号、会话、Cookie、认证字段、窗口标题或对话内容。
- 进程观察仅使用明确 bundle ID/已解析可执行文件判断“是否运行”，不枚举或记录其他应用列表；日志不包含用户名、用户目录、页面正文或完整系统错误对象。
- helper 能触发的破坏性上限是对已确认的目标应用执行现有 graceful quit、残留结束和一次重启；重启次数、状态转换和错误退避必须有自动测试。
- `SMAppService` 注册是用户可在系统设置查看和撤销的登录项。系统返回 `requiresApproval` 时只引导用户自行批准；程序不模拟点击、不绕过批准、不静默修改豆包自身登录项。
- 真实验收只使用无私人内容的空白页；截图裁掉账号、侧栏和最近会话。官方应用文件哈希/签名在前后保持不变。

## Alternatives and non-goals

- 不注册主 App 自身作为登录项：现有 GPUI 主程序总是创建窗口且最后窗口关闭即退出，把它改造成兼具隐藏登录启动、窗口重开和 watcher 的双重生命周期会扩大主程序职责。
- 不使用手写 LaunchAgent plist、cron、Shell 登录脚本或管理员 daemon：这些路径难以映射系统批准状态，也违反用户级可撤销边界。
- 不制作替代官方豆包图标、别名或改写 Dock/Finder 入口；它们无法覆盖用户继续点击官方图标的习惯。
- 不加入 XPC、Unix socket、通用进程管理器或双向 IPC；通过“主 App 运行时 helper 让出所有权、主 App 退出后读最终配置”的单所有者规则即可满足当前需求。
- 不让一次性 CLI `apply` 默认留下后台 watcher，也不把两个桌面开关暴露为 CLI、Skill 或主题包字段。
- 不支持 macOS 12 的旧登录项 API。主 App 最低版本保持 12.0，但本自动化功能需要 macOS 13+ 的可查询 `SMAppService` 状态。
- 不承诺用户直接点官方图标时零帧默认外观；未带调试参数的既有进程必须受控重启才能应用主题。

## Areas of concern

- **系统批准与签名。** 临时 ad-hoc 签名只证明 bundle 结构，不证明最终自签名/Developer ID 包可注册。真实安装包中的 `SMAppService.status=enabled`、helper 进程和注销结果是阻塞验收。
- **注册立即启动。** Apple 说明 Login Item 注册后会立即启动并在后续登录启动。helper 必须在发现主 App 运行时消费当前 audit session 的主动打开机会，否则用户只打开开关就可能意外看到豆包。
- **官方自身开机启动竞态。** 两个登录项顺序不可假定。若官方先普通启动，helper 可能进行一次可见重启；目标身份、单次限制和提示必须真实验证。
- **主动退出与崩溃相似。** CDP 端口消失不能单独区分退出/崩溃，因此自动保持策略一律不立即重开；只有新进程转换或新登录会话的明确设置才能启动。
- **主 App/helper 交接。** helper 轮询发现主 App 的最多一秒窗口内可能仍检查旧主题标记；实现必须先停止 helper watcher、再允许任何重新注入，不得用两个主题竞争的视觉结果当成功。
- **应用更新。** 已注册 helper 指向嵌套 bundle。覆盖安装后需验证系统使用当前包内的新 helper，旧进程能退出；本变更不新增自动更新迁移器。
- **恢复默认语义。** 若只清 DOM 而保留最后主题，helper 会再次应用。因此必须在恢复成功后清最后主题并关闭自动化；恢复失败时保留旧设置，避免页面与持久状态一半提交。
- **固定窗口高度。** 新设置组会压缩预览区域。正常 1120×720 和短紧凑布局都必须真实渲染，不能降低现有预览最小高度、遮住透明度或主题操作按钮。

## Acceptance criteria

1. 配置测试覆盖：缺失默认值、v1 round trip、原子替换、损坏 JSON、未知 schema、非法 target/opacity、父关子关、失败写入保留旧文件和隔离数据目录。
2. 应用测试证明只有匹配 generation 的首次成功注入才保存 `last_applied`；失败应用不覆盖；恢复默认成功清除最后主题并关闭/注销，失败恢复保持原设置。
3. switch 状态测试覆盖：无最后主题、macOS 12、不支持平台、服务四种状态、父子依赖、异步忙碌和打开系统设置动作；AccessKit 树包含稳定 ID、`Role::Switch`、正确 `aria_toggled` 和 disabled 语义。
4. supervisor/运行策略测试覆盖：父关零启动、子关登录零启动、子开每 audit session 一次、注册当前会话不启动、helper 崩溃不重复、主 App 优先、用户退出不重开、下一次手动启动触发一次、错误端口零重启循环。
5. `cargo run -p doubao-skin-desktop` 的非 bundle 开发版本无法注册但窗口保持可用；Windows Rust/桌面构建和现有 CLI `--watch` 行为不回归。
6. host 与 universal 包均含 `Contents/Library/LoginItems/豆皮后台服务.app`；helper/主程序架构、版本、最低系统版本和 bundle ID 正确，`plutil`、`codesign --verify --deep --strict`、ZIP/DMG 完整性与现有自签名检查通过。
7. 从最终安装包开启父项后，实际 `SMAppService.status` 为 `enabled`；关闭后为 `notRegistered` 且 helper 结束。`requiresApproval` 场景显示未生效反馈和可用的“打开系统设置”动作。
8. 父开子关：结束主 App 和目标应用后重新登录，豆皮与豆包窗口均不出现；稍后点击官方目标，最多一次受控重启后主题、透明度和目标标记恢复。随后主动退出目标并观察至少 30 秒，不会重开。
9. 两项都开：新登录会话只主动打开保存目标一次并恢复主题；helper 重启、豆皮主程序打开/关闭以及官方自身开机启动竞态均不产生循环或双 watcher 主题争用。
10. 正常 1120×720 和短紧凑布局真实窗口中，两个开关、说明、系统状态、透明度和底部操作均清楚可用；父子层级、焦点、键盘操作、VoiceOver 状态、浅/深外观通过人工检查。
11. `/Applications/Doubao.app` 与 `/Applications/DoubaoWork.app` 未被写入，目标签名保持有效；loopback 监听、隐私边界和恢复脚本检查通过。
12. `cargo fmt --check`、定向测试、`./scripts/check.sh rust`、桌面测试、`./scripts/check.sh all`、`./scripts/check.sh workflow` 全部通过；`verification.md` 记录探针、红绿测试、包/系统/窗口证据、偏差、残余闪白与 fresh-context verifier 最终 verdict。

## Decision

等待产品与风险负责人明确批准本 spec 后进入 plan；当前只完成无注册技术探针和 artifact 编辑，尚未修改产品代码、打包脚本或系统登录项。
