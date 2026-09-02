---
id: "2026-09-02-add-open-source-notice-to-about"
stage: spec
status: accepted
owner: "codex"
created: "2026-09-02"
based_on: intent.md
risk: "low"
approved_by: "idevlab"
approved_at: "2026-09-02"
---

# Spec: 在“关于豆皮”中增加开源与官方免费声明

## Requirements

1. macOS 应用菜单中的“关于豆皮”必须继续打开 AppKit 标准 About Panel，不得改为 GPUI 自绘窗口。
2. macOS About Panel 必须在现有图标、应用名、版本、构建号和版权之外展示以下声明：“本软件开源，官方版本永久免费。如遇冒充官方收费，请勿购买，请从 GitHub 官方仓库核验并下载。”
3. macOS About Panel 必须展示可点击的官方仓库链接 `https://github.com/IchenDEV/doubao-skin`，由系统默认浏览器打开。
4. Windows 主窗口标题栏右侧必须提供低干扰的“关于”文字按钮；macOS 不增加重复的标题栏入口。
5. Windows 点击“关于”后必须在当前主窗口内显示单实例模态层，包含“豆皮”、当前构建版本、同一声明、完整 GitHub 链接和“关闭”按钮。
6. Windows 模态层必须能通过“关闭”按钮和 `Escape` 关闭；打开或关闭不得改变当前目标应用、主题来源、搜索、主题选择、应用状态或滚动位置。
7. 两个平台的声明和仓库 URL 必须来自同一组共享常量，Windows 版本必须读取构建时的 workspace package version，不得复制硬编码版本。
8. GitHub 链接只能在用户主动点击时调用 GPUI 已有的系统 URL 打开能力；应用不得因展示 About 内容自行联网。
9. “关于”入口、模态层、声明、链接和关闭按钮必须具有可读的辅助功能角色与标签，并在亮色、暗色、正常宽度和窄窗口下保持清晰。

## User experience

- macOS 用户继续从系统菜单栏选择“豆皮”→“关于豆皮”，看到熟悉的系统 About Panel；新增内容位于系统身份与版本信息下方，不改变窗口层级或重复创建面板。
- Windows 用户在标题栏右侧看到弱化但清晰的“关于”文字入口。点击后，当前内容上方出现居中的小型模态卡片和遮罩，焦点进入模态内容，背景不可误操作。
- Windows 模态信息按“应用名与版本 → 开源/官方免费声明 → 完整 GitHub 链接 → 关闭”排列，不加入图标墙、许可证全文、捐赠或反馈入口。
- 链接使用项目现有链接色和指针反馈；关闭按钮使用现有控件颜色与圆角。文案允许自然换行，不截断域名。

## Technical design

- 在 `apps/desktop/src/app/actions.rs` 定义共享的 `OFFICIAL_REPOSITORY_URL` 与 `OPEN_SOURCE_NOTICE`。macOS `show_about_panel` 改用 `orderFrontStandardAboutPanelWithOptions:`，通过 `NSAboutPanelOptionCredits` 传入带链接属性的 `NSAttributedString`；继续让系统从应用包读取图标、名称、版本、构建号和版权。
- 在 `SkinApp` 增加一个布尔状态表示 Windows About 模态是否打开，并提供最小的打开、关闭和打开仓库方法；不引入通用对话框框架或新状态管理层。
- 在现有 `render_header` 的右侧占位区域仅对 Windows 渲染“关于”按钮。点击处理必须阻止标题栏拖动事件继续传播，避免按钮点击触发窗口移动。
- 在顶层 `render` 末尾按状态叠加当前窗口内的模态遮罩与卡片。模态采用 GPUI 既有布局、颜色、角色和点击能力，不创建第二个操作系统窗口。
- Windows 版本显示使用 `env!("CARGO_PKG_VERSION")`，与 workspace/build 版本来源保持一致。链接点击调用 GPUI `cx.open_url(OFFICIAL_REPOSITORY_URL)`，不增加 shell crate 或平台命令依赖。
- 在键盘处理的共享入口中，Windows About 打开时优先消费 `Escape` 并关闭模态；其他既有快捷键和输入行为保持原样。
- macOS 设计遵循 Apple HIG 的 [菜单栏](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar/)、[菜单](https://developer.apple.com/design/human-interface-guidelines/menus/)、[模态](https://developer.apple.com/design/human-interface-guidelines/modality/)与[辅助功能](https://developer.apple.com/design/human-interface-guidelines/accessibility/)原则：保留系统应用菜单与原生 About Panel，将自定义 UI 限定在缺少同等系统入口的 Windows。

## Security and privacy

- 声明和仓库地址是随应用分发的静态公开信息，不读取账户、会话、设备标识或用户文件。
- About 展示过程不得产生 HTTP 请求；只有用户点击固定的 HTTPS URL 时才交给操作系统默认浏览器。
- URL 不从主题、远端目录、环境变量或用户输入拼接，防止链接替换或任意协议打开。
- 真实窗口截图不得包含用户账户、会话内容、主题本地路径或其他敏感信息。

## Alternatives and non-goals

- 不把 Windows About 放进独立页面、设置导航、系统托盘或右键菜单；当前应用没有这些信息架构，为一个静态说明新增它们属于过度设计。
- 不在 macOS 标题栏重复增加“关于”按钮；系统应用菜单已经提供标准、可发现的入口。
- 不为 Windows 创建第二个原生窗口；当前窗口内模态足以承载短信息，并避免额外窗口生命周期和焦点管理。
- 不增加许可证全文查看器、自动正版校验、举报、更新检查、捐赠或付费功能。
- 不使用“任何收费即为盗版”的绝对表述；MIT/GPL 开源许可都不等同于禁止有偿分发，产品只警示冒充官方收费。

## Areas of concern

- AppKit credits 的富文本链接必须在真实打包 `.app` 中验证；开发态 `cargo run` 的 bundle 元数据不足以证明最终 About 内容。
- Windows 标题栏由应用接管拖动，按钮区域必须停止事件传播并保留系统最小化、最大化和关闭按钮的可用区域。
- 模态层不能依赖仅在 macOS 可用的 AppKit 类型；共享常量与纯状态可跨平台编译，AppKit 构造保持 `cfg(target_os = "macos")`。
- 模态打开时需明确键盘焦点和 `Escape` 优先级，避免按键继续落到主题搜索或快捷操作。
- 当前主窗口不可调整尺寸，但仍需在项目定义的正常与窄布局宽度验证文案换行、链接完整性和关闭按钮可见性。

## Acceptance criteria

1. macOS 打包应用的“关于豆皮”仍是单实例原生 About Panel，并显示准确图标、名称、版本、构建号、版权、完整声明与可点击 GitHub 链接。
2. macOS 点击 GitHub 链接只打开 `https://github.com/IchenDEV/doubao-skin`；关闭面板后主窗口和标准应用菜单行为不变。
3. Windows 标题栏右侧显示“关于”，macOS 标题栏不显示重复入口；入口不会触发窗口拖动。
4. Windows 模态显示应用名、与 workspace 一致的版本、完整声明和完整 URL，且同一时刻只能存在一个。
5. Windows 的关闭按钮与 `Escape` 均能关闭模态；打开/关闭前后的目标、来源、搜索、主题选择和应用状态一致。
6. 屏幕阅读器可识别 Windows 入口、模态、链接和关闭按钮；macOS VoiceOver 可读取 credits 声明并识别链接。
7. 单元/回归测试锁定共享文案、固定 HTTPS URL、平台入口可见性、版本来源、模态状态与 Escape 行为；定向桌面测试、`cargo check`、workflow 检查通过。
8. macOS 真实打包窗口与 Windows 真实打包窗口分别完成正常/窄宽度、亮色/暗色、链接打开和关闭交互验证，证据写入 `verification.md`。

## Decision

等待产品与风险负责人明确接受本规格后进入实施计划；本阶段不修改产品代码。
