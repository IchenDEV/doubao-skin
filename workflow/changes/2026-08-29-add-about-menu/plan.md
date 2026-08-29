---
id: "2026-08-29-add-about-menu"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: spec.md
risk: "low"
approved_by: "engineering-owner"
approved_at: "2026-08-29"
---

# Plan: 增加“关于豆包主题”菜单

## Files and ownership

- `apps/desktop/src/main.rs`：定义并注册应用菜单动作、标准快捷键和菜单模型；增加 AppKit 原生关于面板调用及菜单顺序回归测试。
- `apps/desktop/Cargo.toml`：在 macOS 目标依赖中显式声明已有 Objective-C 桥接 crate，避免依赖未声明的传递依赖。
- `apps/desktop/Info.plist`：增加原生关于面板使用的版权元数据；名称、图标和版本字段保持现有来源。
- `workflow/changes/2026-08-29-add-about-menu/verification.md`：记录菜单模型测试、plist/打包检查、真实菜单交互、关于面板截图和剩余风险。
- 不修改应用图标、主题包、Web 站点、官方客户端或主界面布局。

## Order of work

1. 修改前核对 GPUI 当前 revision 的 `Menu`/`MenuItem`/`SystemMenuType` API、现有 `Info.plist` 和并行 `main.rs` 改动。
2. 先增加失败测试，要求应用菜单首项为“关于豆包主题”，标准项顺序正确，快捷键定义为 `Command-H`、`Option-Command-H`、`Command-Q`。
3. 在 `main.rs` 定义最小动作集合和菜单构造函数，注册 About、Hide、HideOthers、ShowAll、Quit 处理器与键绑定。
4. 在 macOS 专用函数中通过 AppKit 标准选择器打开关于面板；不自建 NSWindow，不保存额外面板状态。
5. 向 `Info.plist` 增加 `NSHumanReadableCopyright`，在 macOS 目标依赖中显式声明 Objective-C 桥；运行格式化、定向测试、编译和 plist 校验。
6. 与固定窗口、主题颜色变更一起构建最终通用应用包，核对打包 plist 的名称、短版本、构建号、版权和签名。
7. 从最终 `.app` 启动，打开应用菜单和关于面板，重复触发并验证标准菜单/快捷键；保存无敏感内容的截图。
8. 将命令、菜单顺序、元数据读回、截图和残余限制写入 `verification.md`，实现会话不自行给出 fresh-context 最终结论。

## Test-first proof

- 菜单构造测试在实现前因没有应用菜单函数/动作而失败，建立对用户可见菜单结构的判红信号。
- 测试按枚举结构检查 About、Services、Hide、Hide Others、Show All、Quit 和分隔线顺序，不只做字符串全文搜索。
- 键绑定测试检查动作与标准按键映射，防止菜单存在但快捷键失效。
- `plutil -lint` 与 `PlistBuddy` 读回测试锁定版权文本；打包后再次读回版本和构建号，避免只验证源 plist。
- 实现后运行 `cargo test -p doubao-skin-desktop ui_regression_tests --locked` 和 `cargo check -p doubao-skin-desktop --locked`。

## Visual or integration proof

- 从最终 `dist/豆包主题.app` 启动，在系统菜单栏确认最左侧菜单名为“豆包主题”、首项为“关于豆包主题”。
- 点击 About，确认系统原生面板显示最终应用图标、名称、版本、构建号和版权；再次点击时窗口数量不增加且现有面板前置。
- 关闭关于面板后继续操作主题选择，确认主窗口与状态未被重置。
- 实际触发 `Command-H`、`Option-Command-H`、全部显示和 `Command-Q`，确认行为符合菜单标签；退出测试前不打开官方豆包客户端。
- 保存应用菜单展开和关于面板两张无敏感内容截图，并记录运行进程来自本次打包路径。

## Risks and mitigations

- `cargo run` 缺少 bundle 元数据：所有最终 About 内容只从打包 `.app` 验收，开发运行仅用于编译和菜单模型测试。
- Objective-C 选择器调用错误会在运行时失败：使用系统固定的 `orderFrontStandardAboutPanel:`，限制在主线程动作回调，并以真实点击验证。
- 快捷键未出现在菜单：在设置菜单前绑定键位，并用真实菜单文字/快捷键显示和行为双重验证。
- About 面板图标被并行资产改动影响：不编辑图标，打包后读取实际 bundle 图标并截图核对。
- `main.rs` 与另外两项变更共享文件：三个已批准变更顺序实施、每次局部补丁并统一运行回归，避免相互覆盖。

## Rollback

- 回滚只移除菜单动作/注册、AppKit About 辅助函数、目标依赖和 plist 版权键；不使用 `git reset --hard` 或 `git checkout --`。
- 回滚不修改图标、版本、主题、官方客户端、用户数据或其他并行改动。
- 从回滚后的源代码重新构建 `.app`，不直接编辑签名包内容。

## Deviations

无。若 GPUI 当前平台层无法稳定承载标准应用菜单，先更新规格和计划并重新确认，不引入第二套菜单框架作为临时绕过。

## Decision

等待工程负责人确认本计划后开始修改产品代码与测试。
