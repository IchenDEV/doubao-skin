---
id: "2026-08-31-support-workbuddy"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-31"
based_on: intent.md
risk: "high"
approved_by: "product-risk-owner"
approved_at: "2026-08-31"
---

# Spec: 支持 WorkBuddy 实时主题

## Requirements

1. 核心必须把 WorkBuddy 建模为独立于「豆包」和「豆包工作」的第三个目标应用，使用已实测的安装路径 `/Applications/WorkBuddy.app`、bundle id `com.workbuddy.workbuddy` 和主可执行文件 `Contents/MacOS/Electron`。
2. WorkBuddy 必须使用独立的默认回环 CDP 端口 `9224`，允许通过专用环境变量覆盖；不得复用「豆包工作」的 `9222` 或「豆包」的 `9223`。
3. 注入前必须在 `/json` 中找到类型为 `page` 且 URL 严格属于 `/Applications/WorkBuddy.app/Contents/Resources/app.asar/renderer/index.html` 的目标。只有查询参数或 hash 可以变化；普通 `file://` 页面、远程网页、webview、DevTools、扩展页、登录页、文档页和其他 Electron 应用都不得因为端口可用而获得注入。
4. 如果 WorkBuddy 已在正确端口运行，工具必须复用该实例；如果目标端口属于其他程序或没有 WorkBuddy 身份页，必须停止并给出可操作错误，不终止占用端口的进程。
5. 如果 WorkBuddy 正在运行但没有正确 CDP 端口，工具不得在第一次点击「应用主题」时静默退出它。界面必须明确说明重启会中断正在执行的任务，并要求用户点击一次语义清晰的「重启 WorkBuddy 并应用」后才执行退出、等待、精确进程清理和带端口重启。
6. WorkBuddy 被用户主动退出或调试端口消失后，watcher 必须停止并提示需要重新应用；不得像现有豆包目标一样自动重启 WorkBuddy。此差异属于目标生命周期策略，不改变现有两款豆包应用的行为。
7. WorkBuddy 首版只支持 `schemaVersion: 2` 主题。核心必须从结构化的 appearance、typography、layout、composer、content、effects、background 和 surfaceOpacity 字段生成 WorkBuddy 专用宿主 CSS；不得直接向 WorkBuddy 注入主题包中为豆包编写的原始 `theme.css`。
8. WorkBuddy 专用样式必须至少覆盖应用根背景、左侧导航、主内容面板、输入区、常规按钮/选中态、弹层、文本、代码块、选择色和滚动条，同时保持产品布局、交互热区和可访问名称不变。具体选择器必须来自获批后对真实 WorkBuddy 5.3.14 CDP DOM 的只读探测。
9. WorkBuddy 首版不得执行主题图标替换或品牌图替换，也不得给其 DOM 写入 `data-doubao-theme-icon`。背景层和输入区标记仍只能使用工具自有、可完整清理的运行时节点/属性。
10. 「恢复默认」必须在当前 WorkBuddy 目标页调用运行时 `destroy()`，移除工具注入的 style、backdrop 和自有标记，并恢复注入前的 root/body 主题属性；不得写 WorkBuddy 的 localStorage、IndexedDB、设置文件或 app bundle。
11. 桌面端必须将 active theme 与目标应用一起判定。切换目标时先停止旧 watcher 并尽力清理旧目标，再切换上下文；一款应用的已应用状态不得显示到另一款。
12. 新增 WorkBuddy 支持不得改变协议桥边界；`protocol_bridge` 继续只使用「豆包工作」目标及其既有端口和 URL 约束。

## User experience

- 主界面目标选择器显示三个并列目标：「豆包」「豆包工作」「WorkBuddy」，分别支持 `Command-1`、`Command-2`、`Command-3`。未安装项保留名称并标记「未安装」，不可点击；选中和不可用状态必须有 VoiceOver 文本，不能只靠颜色。
- 保持既有默认：用户保存过且仍安装的目标优先；没有偏好时，如果只安装 WorkBuddy 就选中 WorkBuddy，否则继续默认「豆包工作」，不因新增目标改变现有用户启动行为。
- 选中 WorkBuddy 后在主操作附近显示低干扰的「实验支持 · 已验证 5.3.14」说明。版本不等于已验证版本时仍允许进入探测，但必须显示版本差异提示，不作完整兼容承诺。
- WorkBuddy 目标下，v1/无法结构化适配的主题仍可浏览预览，但「应用主题」不可用并就地说明「该主题尚不支持 WorkBuddy」；不弹出阻断浏览的模态框。
- WorkBuddy 未运行时，正常的「应用主题」可直接以 `9224` 启动并应用；WorkBuddy 正在运行但没有调试端口时，第一次点击只展示风险与「重启 WorkBuddy 并应用」，第二个明确动作才可重启。
- 成功应用后仍使用既有的应用/恢复状态语义。用户关闭 WorkBuddy 后不自动拉起，桌面工具就地说明需重新点击应用。
- 三段选择器、风险提示和主操作在正常窗口及 720 px 最小宽度下必须完整可见、可点击、可用键盘操作，不遮挡 macOS 标准窗口按钮。

## Technical design

- 在 `crates/skin-core/src/live.rs` 的现有 `TargetApp` 中增加 `WorkBuddy`，继续用一个小型枚举承载显示名、bundle id、路径、精确进程匹配、端口、启动标记、身份 URL 和生命周期策略；不新增 Manager、Factory、插件注册表或第二套 watcher。
- WorkBuddy 元数据使用 `workbuddy` id、`DOUBAO_SKIN_WORKBUDDY_CDP_PORT` 环境变量和默认端口 `9224`。身份 URL 只接受规范化后路径等于 `file:///Applications/WorkBuddy.app/Contents/Resources/app.asar/renderer/index.html` 的页面，忽略 hash/query，不使用通用 `file://` 前缀。
- 将当前「确认端口、必要时重启、开始 watcher」的入口拆成可测试的准备结果，至少区分 `Ready`、`NotRunning`、`RestartConfirmationRequired`、`WrongPortOwner` 和 `NotInstalled`。豆包目标保持当前自动准备路径；WorkBuddy 的 `RestartConfirmationRequired` 交给桌面端二次动作授权。
- 为目标增加显式的 `relaunch_after_port_loss` 策略：现有两款豆包为 `true`，WorkBuddy 为 `false`。WorkBuddy 端口消失时退出 watcher，不触发 `launch_app`。
- 将主题注入改为按目标选择 CSS：豆包与豆包工作继续使用现有 `injected_css()`；WorkBuddy 走新的 v2 adapter 生成器，只输出工具生成的变量和宿主作用域规则。adapter 规则必须以 `html[data-skin][data-skin-target=workbuddy]` 为根，并避免选择 iframe、webview 和文档内容容器。
- WorkBuddy adapter 直接映射已有结构化主题字段，不新增第三套主题文件、不按 theme id 分支，也不要求修改全部主题包。`schemaVersion < 2` 返回“不支持该目标”的能力结果。
- bootstrap 继续保存并恢复原始属性，但在 `TARGET=workbuddy` 时跳过 `markIcons()`；恢复脚本继续只删除工具自有 runtime、style、backdrop 与标记。
- `apps/desktop/src/main.rs` 扩展初始目标选择、偏好、三段选择器、`Command-3`、预览身份、兼容性状态和重启二次动作。WorkBuddy 运行状态只展示用户可执行结果，不暴露端口、CDP、路径或进程术语。
- 不修改 `protocol_bridge.rs` 的目标、端口、payload 或网络边界；测试应锁定其仍不接受 WorkBuddy。

## Security and privacy

- CDP 继续只绑定 `127.0.0.1`，端口取自受控常量或本机环境变量；不监听局域网地址。
- 页面身份必须先由 CDP 列表的精确主页面 URL 证明，再连接 WebSocket；不能根据窗口标题、页面文本、可访问名称或主题内容猜测。
- 进程操作只允许精确匹配 `/Applications/WorkBuddy.app/Contents/MacOS/Electron`，且只在用户点击「重启 WorkBuddy 并应用」后执行；不得使用宽泛的 `Electron` 或 `WorkBuddy` 模糊匹配终止其他进程。
- DOM 探测只用于确定稳定的结构/类与渲染边界，不采集或写入任务标题、提示词、回复、账号、Cookie、localStorage、IndexedDB、工作空间、文件、插件、MCP、网络请求或遥测。
- 自动化测试使用合成 `/json` fixtures 和合成 DOM/字符串断言，不提交真实 WorkBuddy 页面快照、官方代码、app.asar 内容、用户截图或专有资源。
- 真实窗口验证使用新建空白任务，不输入敏感内容；截图在提交前检查并裁掉/遮盖现有任务列表、账号信息和通知。
- 主题包提供的原始 CSS 和图标资产不会进入 WorkBuddy 页面，降低为豆包编写的宽泛选择器误伤 WorkBuddy 或替换官方品牌的风险。

## Alternatives and non-goals

- 不修改或抽取 WorkBuddy 的 `app.asar`：这会破坏签名/更新链路并触及专有资源。
- 不实现 WorkBuddy 扩展或写入其用户设置：这会形成持久状态并扩大权限边界。
- 不把所有 Electron 页面视为可注入目标，也不复用通用 side-panel/popup 规则：WorkBuddy 首版只覆盖已证明的主 renderer。
- 不直接复用主题包原始 CSS：它面向豆包 DOM 和 Semi Design 变量，局部颜色碰巧生效不构成兼容。
- 不新增主题 manifest 的宿主列表：首版用现有 v2 结构化字段生成统一 adapter，避免要求所有内置主题逐个迁移；若后续需要主题作者提供 WorkBuddy 专属 CSS，再另开变更设计 schema。
- 不在首版替换 WorkBuddy 图标、品牌、编辑器/文档画布内部内容，也不支持协议桥、模型路由、MCP 或插件。
- 不承诺 5.3.14 之外的版本已兼容；版本提示与真实回归证据必须一致。

## Areas of concern

- Electron 通常接受 `--remote-debugging-port`，但 WorkBuddy 5.3.14 是否保留该参数、是否在单实例转发后仍开放端口尚未实测。获批后的第一步必须是可回滚探针；若失败或必须改包，停止实现并回报阻塞，不绕过应用安全策略。
- WorkBuddy 会运行代理任务、CLI 和插件进程，重启可能中断工作；因此二次动作和精确进程边界是发布阻塞项，不得以普通提示替代。
- 主页面使用 `file://.../app.asar/...`，版本更新可能改变路径、DOM 类或窗口结构。页面 URL、选择器和已验证版本必须记录在 verification，更新后需要重新验证。
- WorkBuddy 的 React/CSS 类可能哈希化；adapter 应优先使用稳定的 landmark、ARIA、data 属性和上层布局容器，少量必要的类片段必须有真实 DOM 证据与回归探针。
- WorkBuddy 内可能包含文档、浏览器或插件 webview；即使主页面已确认，也必须避免跨 frame 注入或选择器穿透内容画布。
- 强制 light/dark 属性可能与 WorkBuddy 自身主题状态竞争；adapter 应以工具作用域变量覆盖为主，恢复必须还原原属性，不能持久更改应用设置。
- 结构化 v2 字段能覆盖基础视觉语义，但不保证每款主题的装饰性豆包 CSS 或图标在 WorkBuddy 重现。首版验收以安全、一致、可读的宿主表面为准，不宣称像素级跨应用一致。

## Acceptance criteria

1. WorkBuddy 5.3.14 未运行时，获批后的隔离探针能用 `127.0.0.1:9224` 启动它，`/json` 至少出现一个严格匹配主 renderer 的 page target；错误端口、只有普通 `file://` 页面或其他应用页面时必须拒绝。
2. WorkBuddy 已运行但没有正确调试端口时，第一次应用不会退出它；只有点击「重启 WorkBuddy 并应用」后才重启。验证中记录重启前后主进程身份，并确认不以宽泛 `Electron` 匹配终止其他应用。
3. 用户主动退出已应用主题的 WorkBuddy 后，工具不会自动重启它；界面说明需重新应用。「豆包」和「豆包工作」的既有 watcher 行为不变。
4. 三目标安装检测、偏好回退、目标切换、active theme 隔离、`Command-1/2/3`、VoiceOver 名称及未安装状态均有回归测试，并在正常/720 px 窗口中可用。
5. WorkBuddy 目标只允许 v2 主题应用；v1 主题显示明确的不兼容状态。生成的注入脚本包含 `TARGET="workbuddy"` 和 WorkBuddy adapter，但不包含主题包原始 CSS、图标 data URI 或 `markIcons()` 执行路径。
6. 至少使用一款浅色 v2 主题和一款深色 v2 主题，在 WorkBuddy 空白任务真实窗口验证左侧导航、主内容、输入区、按钮/选中态、弹层、文本、代码块、滚动条和可选背景层；正常与窄窗口均清晰可读且不改变布局/点击区域。
7. WorkBuddy 页面刷新、打开新空白任务或内部导航后主题持续存在；普通网页、文档/webview、DevTools 和非主 renderer 不出现 `data-skin`、style 或 backdrop。
8. 「恢复默认」立即移除 WorkBuddy 页面中的 runtime、style、backdrop 和自有标记并恢复原始 root/body 属性；退出主题工具并重新打开 WorkBuddy 后保持官方外观，app bundle、签名和用户设置未改变。
9. 验证过程中不输入敏感内容，不记录任务正文、凭据、Cookie、存储、工作空间、文件、插件、MCP 或网络数据；提交的视觉证据不含用户现有任务、账号或通知。
10. Rust 测试覆盖三目标元数据、端口隔离、严格 URL 归属、错误端口拒绝、WorkBuddy 重启 Gate、端口丢失策略、v2 adapter 和恢复脚本；适用桌面 UI 测试覆盖三段选择和兼容状态。
11. `./scripts/check.sh rust`、`./scripts/check.sh workflow`、`git diff --check` 以及由实际改动触发的其他最小适用检查通过；真实命令、结果、截图路径、WorkBuddy 版本、偏差和剩余风险写入 `verification.md`。

## Decision

待产品负责人明确接受本 Spec 后进入 Plan；当前只记录已接受的 Intent 和待审 Spec，不修改产品代码，也不重启 WorkBuddy。
