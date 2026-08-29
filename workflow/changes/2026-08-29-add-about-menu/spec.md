---
id: "2026-08-29-add-about-menu"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: intent.md
risk: "low"
approved_by: "product-risk-owner"
approved_at: "2026-08-29"
---

# Spec: 增加“关于豆包主题”菜单

## Requirements

1. 应用启动时必须注册名为“豆包主题”的 macOS 应用菜单，其首项为“关于豆包主题”。
2. “关于豆包主题”必须调用 AppKit 原生标准关于面板，而不是打开 GPUI 自绘窗口、网页或主界面页面。
3. 关于面板必须从当前应用包读取正式 `AppIcon`、`CFBundleDisplayName`、`CFBundleShortVersionString`、`CFBundleVersion` 和 `NSHumanReadableCopyright`，不在 Rust 中复制版本号。
4. 应用菜单必须同时提供系统“服务”子菜单、“隐藏豆包主题”“隐藏其他”“全部显示”和“退出豆包主题”。
5. 标准快捷键必须生效：`Command-H` 隐藏应用、`Option-Command-H` 隐藏其他应用、`Command-Q` 退出应用。
6. 重复选择“关于豆包主题”必须把现有原生面板带到前台，不创建多个关于窗口。
7. 菜单注册不得改变主窗口创建、主题选择、应用/恢复、窗口固定尺寸或目标应用状态。

## User experience

- 用户从屏幕顶部菜单栏打开“豆包主题”，第一项即可找到“关于豆包主题”。
- 点击后显示系统熟悉的原生关于面板，包含当前应用图标、“豆包主题”、版本号、构建号和版权文本。
- 用户可正常关闭关于面板并继续使用主窗口；再次打开时复用系统面板行为。
- 应用菜单中的服务、隐藏、全部显示和退出遵循 macOS 标准位置与快捷键，不在主界面增加重复按钮。

## Technical design

- 使用 GPUI `actions!` 定义最小的 About/Hide/HideOthers/ShowAll/Quit 动作，并在 `App` 启动闭包中注册处理器与标准键绑定。
- 使用 `cx.set_menus()` 构造一个标准应用菜单：About、分隔线、Services、分隔线、Hide/Hide Others/Show All、分隔线、Quit。
- macOS About 动作通过现有 `cocoa`/Objective-C 桥向 `NSApplication` 发送 `orderFrontStandardAboutPanel:`；只在 `target_os = "macos"` 编译。
- `Info.plist` 增加 `NSHumanReadableCopyright = Copyright © 2026 豆包主题贡献者`。名称、图标与版本继续由现有 plist 和 `scripts/build-macos.sh` 提供，构建脚本仍以 workspace 版本覆盖短版本和构建号。
- 将菜单构造保留为 `main.rs` 中的小函数，方便测试菜单顺序和标签，不新增菜单管理模块或第三方 UI 依赖。

## Security and privacy

- 原生关于面板只读取应用包公开元数据，不访问网络、不采集设备信息、不写入用户数据。
- 菜单动作只调用本机应用生命周期能力，不启动、退出或注入豆包/豆包工作官方客户端。
- 不增加更新检查、遥测、反馈上传、许可证下载或外部链接。
- 真实截图仅显示应用公开元数据，不包含账户、会话或主题文件路径。

## Alternatives and non-goals

- 不自绘关于对话框：会重复 macOS 已有能力，并增加暗色、辅助功能、键盘和层级维护成本。
- 不在侧栏或标题栏增加“关于”按钮；应用身份信息属于系统应用菜单。
- 不增加没有实际设置界面的“设置…”空菜单项，也不伪造帮助、更新或反馈功能。
- 不引入 `muda` 等第二套菜单框架；复用当前 GPUI 菜单和 AppKit 面板。

## Areas of concern

- `cargo run` 不位于 `.app` 包内，原生关于面板可能缺少完整 bundle 元数据；最终验收必须从 `dist/豆包主题.app` 启动。
- GPUI 菜单快捷键依赖动作键绑定；测试需同时核对菜单标签和键位，并在真实菜单中验证显示。
- Objective-C 调用必须限制在主线程的应用动作回调，并使用标准面板选择器，避免自管 NSWindow 生命周期。
- 当前工作树存在并行构建与图标改动；关于面板必须复用最终打包图标，不覆盖图标资产。

## Acceptance criteria

1. 应用菜单模型首项名称为“关于豆包主题”，并包含 Services、Hide、Hide Others、Show All 和 Quit 的既定顺序与分隔。
2. `Command-H`、`Option-Command-H` 和 `Command-Q` 键绑定分别触发隐藏、隐藏其他和退出动作。
3. 打包后的 `Info.plist` 中 `CFBundleDisplayName` 为“豆包主题”，短版本与 workspace 版本一致，构建号存在，版权文本为“Copyright © 2026 豆包主题贡献者”。
4. 从 `dist/豆包主题.app` 打开菜单并选择“关于豆包主题”后，原生面板展示正确图标、名称、版本、构建号和版权；重复选择不产生多个面板。
5. 关闭关于面板后主窗口仍可操作；Services、隐藏、全部显示和退出行为正常。
6. `cargo test -p doubao-skin-desktop ui_regression_tests --locked`、`cargo check -p doubao-skin-desktop --locked`、`plutil -lint apps/desktop/Info.plist`、`./scripts/check.sh workflow` 与 macOS 打包签名检查通过。

## Decision

等待产品与风险负责人确认本规格后进入实施计划。
