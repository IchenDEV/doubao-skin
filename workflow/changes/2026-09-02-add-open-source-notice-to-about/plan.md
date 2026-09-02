---
id: "2026-09-02-add-open-source-notice-to-about"
stage: plan
status: accepted
owner: "codex"
created: "2026-09-02"
based_on: spec.md
risk: "low"
approved_by: "idevlab"
approved_at: "2026-09-02"
---

# Plan: 在“关于豆皮”中增加开源与官方免费声明

## Files and ownership

- `apps/desktop/src/app/actions.rs`：保存跨平台共享的官方仓库 URL 与声明；为 macOS 标准 About Panel 构造带可点击链接的 credits 富文本。
- `apps/desktop/src/app/mod.rs`：在 `SkinApp` 中增加唯一的 About 打开状态并初始化为关闭。
- `apps/desktop/src/app/input.rs`：About 打开时优先处理 `Escape`，阻止事件继续触发搜索或主题操作。
- `apps/desktop/src/i18n.rs`：增加“关于”、版本、GitHub 与关闭所需的用户文案，避免在渲染代码中散落中文。
- `apps/desktop/src/ui/mod.rs`：在 Windows 标题栏右侧挂载 About 入口，并在顶层布局叠加模态内容。
- `apps/desktop/src/ui/about.rs`：只承载 Windows About 入口与模态的渲染和打开/关闭/打开仓库交互；不发展成通用对话框框架。
- `apps/desktop/src/ui_regression_tests.rs`：覆盖共享文案和固定 URL、平台入口可见性、构建版本来源、单实例状态与 `Escape` 决策。
- `workflow/changes/2026-09-02-add-open-source-notice-to-about/verification.md`：记录命令、真实 macOS/Windows 窗口、链接、辅助功能和残余风险证据。
- 不修改网站、主题包、发布流程、许可证、应用图标、窗口尺寸或官方豆包/WorkBuddy 客户端。

## Order of work

1. 修改前记录 `git status`，核对当前 `SkinApp`、标题栏、按键处理、AppKit About 动作和 i18n 字段，避免覆盖并行改动。
2. 先增加回归测试，要求固定 HTTPS 仓库 URL、准确声明、Windows-only 标题栏入口、workspace 构建版本和 About 打开时 `Escape` 关闭；运行测试并保存预期失败。
3. 在 `actions.rs` 增加共享常量，并将 macOS About 调用改为带 `NSAboutPanelOptionCredits` 的标准面板 options；不覆盖系统读取的图标、名称、版本、构建号或版权。
4. 为 `SkinApp` 增加最小布尔状态，在 `ui/about.rs` 实现 Windows 入口、同窗口模态卡片、链接和关闭按钮；使用现有 palette、GPUI `cx.open_url` 与辅助功能角色。
5. 将 Windows 入口接到标题栏右侧，占用原有空白区域并在鼠标处理器中停止传播；将模态叠加到顶层布局，背景遮罩拦截点击。
6. 在 `key_down` 最前面处理 About 打开状态：仅 `Escape` 关闭，其余输入不落到搜索、目标切换或主题操作；完成后运行格式化、定向测试和桌面 crate 检查。
7. 构建 macOS 最终 `.app`，从打包路径打开并验证原生 About 内容、单实例、VoiceOver 语义与固定 GitHub 链接；点击前确认没有 About 引起的网络请求，点击后确认只交给默认浏览器。
8. 构建 Windows 对应架构的正式包并在真实 Windows 环境运行，验证标题栏点击不拖窗、模态焦点/遮罩/`Escape`、亮暗色、正常与窄布局、版本和 GitHub 链接。
9. 运行完整适用 gate，将命令、结果、截图路径、辅助功能读回、URL 打开证据、偏差和残余风险写入 `verification.md`，不自行把 fresh-context verdict 标为通过。

## Test-first proof

- 第一组失败测试锁定 `OFFICIAL_REPOSITORY_URL` 的完整 HTTPS 值和 `OPEN_SOURCE_NOTICE` 的批准文案；实现前因常量不存在而失败。
- 平台可见性测试要求 Windows 标题栏显示 About 入口而 macOS 不重复显示；实现前因 helper/渲染决策不存在而失败。
- 输入决策测试要求模态打开时 `Escape` 返回关闭并消费事件，其他键不进入原有搜索/主题分支；实现前因 About 状态与优先分支不存在而失败。
- 版本测试要求 Windows 展示值来自 `env!("CARGO_PKG_VERSION")`，并与 workspace package version 一致，不接受另一个数字字面量。
- 实现后先运行 `cargo test -p doubao-skin-desktop ui_regression_tests --locked`，再运行 `cargo test -p doubao-skin-desktop --locked` 和 `cargo check -p doubao-skin-desktop --locked`；只在相关代码变化后重跑。
- AppKit 富文本与系统浏览器交互保留为真实打包集成验证，不为了伪造可单测性抽象 Objective-C 运行时。

## Visual or integration proof

- macOS：运行本次构建的 `.app`，展开应用菜单并打开 About Panel；核对图标、名称、版本、构建号、版权、声明、完整 GitHub 链接及单实例。使用辅助功能读回确认声明可读、链接可识别，保存无敏感信息截图。
- macOS：在点击链接前后观察网络/浏览器行为，证明展示面板不联网、点击只打开批准的仓库 URL；关闭后验证主窗口与 `Command-H`、`Command-Q` 等标准行为未回归。
- Windows：运行本次正式打包目录中的 `doubao-skin.exe`，分别在正常布局和项目窄布局探针下点击标题栏 About；确认按钮不启动拖窗，模态遮挡背景且只出现一次。
- Windows：在亮色和暗色下核对文案换行、完整域名、对比度、焦点、关闭按钮和 `Escape`；点击 GitHub 后确认默认浏览器地址精确匹配，关闭前后目标、来源、搜索、主题选择与应用状态不变。
- 若真实 Windows 环境不可用，不以跨编译、DOM/元素树或 macOS 上的条件渲染替代验收；在 `verification.md` 明确保持 Windows 验收未完成。

## Risks and mitigations

- Objective-C credits options 构造错误可能导致 About 面板空白或崩溃：只使用系统标准 `NSAboutPanelOptionCredits`/`NSAttributedString`，将调用限定在 macOS 主线程动作，并以打包应用真实点击验证。
- Windows 标题栏入口可能与拖动命中区冲突：入口自身在鼠标按下和点击时停止传播，真实验证拖窗和系统窗口按钮。
- 模态可能泄漏键盘输入到背景：按键入口优先检查 About 状态并消费事件；用已填写搜索词、已选主题和目标切换快捷键回归验证。
- 共享文案可能在平台实现中漂移：只保留 `actions.rs` 一份声明和 URL，两个渲染路径引用它们，测试锁定准确值。
- 富文本链接颜色或辅助功能语义受系统/GPUI 平台实现影响：以亮暗色真实窗口和 VoiceOver/Windows 辅助功能读回为准，不靠源代码推断通过。
- 为了一个 About 模态引入通用组件会扩大范围：本次允许一个 `ui/about.rs` 深度小模块；只有后续出现第二种真实模态需求时才考虑抽象共享对话框。

## Rollback

- 回滚只移除共享声明/URL、AppKit credits options、Windows About 状态、标题栏入口、模态渲染、相关 i18n 字段和测试。
- 不使用 `git reset --hard` 或 `git checkout --`，不覆盖用户或并行任务的未提交改动。
- 回滚后重新运行桌面定向测试与 `cargo check`，并重新构建 macOS/Windows 包确认原有 About 与标题栏行为恢复。

## Deviations

无。若 GPUI 当前版本不能可靠提供同窗口遮罩、焦点或系统 URL 打开能力，先以最小可逆探针验证限制并更新规格/计划重新确认，不引入新 UI 框架或 shell 依赖绕过。

## Decision

等待工程负责人明确接受本计划后开始测试与产品代码修改。
