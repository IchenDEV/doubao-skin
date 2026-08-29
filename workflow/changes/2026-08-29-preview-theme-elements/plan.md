---
id: "2026-08-29-preview-theme-elements"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: spec.md
risk: "medium"
approved_by: "engineering-owner"
approved_at: "2026-08-29"
---

# Plan: preview theme elements

## Files and ownership

- `crates/skin-core/src/theme.rs`：扩展 `PreviewStyle`，按 `preview.appearance` 合并变体/顶层图标和输入框、布局预览值；增加主题投影回归测试。保留主题运行时生成逻辑不变。
- `apps/desktop/src/main.rs`：让大预览只读取解析后的 `ThemeRow.preview`，支持 SVG/位图图标，并把输入框和侧栏的可见几何属性映射到 16:9 画布。不改变主题应用、恢复和双目标状态机。
- `workflow/changes/2026-08-29-preview-theme-elements/verification.md`：记录自动检查、正常/窄窗口截图、资产回退结果、偏差与剩余风险。
- 不修改 `themes/**`、`apps/web/**`、生成目录或官方应用；发现主题资产本身有问题时只记录，不在本变更顺手修复。

## Order of work

1. 在修改产品代码前核对目标文件、运行中主题工具和打包产物时间，保存并行改动边界。
2. 先在 `theme.rs` 增加会失败的回归断言：变体图标覆盖、顶层图标回退、变体输入框字段覆盖与顶层字段回退。
3. 扩展现有 `Theme::preview_style()`，一次生成 UI 所需的解析结果；使用直白的逐字段合并，不新增抽象层。
4. 修改桌面预览的图标与可见元素调用，全部读取 `PreviewStyle`；根据文件扩展名渲染 SVG 或位图，并对几何字段做统一缩放和安全限制。
5. 运行格式化、定向核心测试、桌面 UI 回归测试和桌面编译检查，修复本变更引入的问题。
6. 构建通用 macOS 应用包并检查签名、压缩包和 `arm64/x86_64` 架构；构建前后核对源文件时间，避免并行改动导致陈旧产物。
7. 启动真实「豆包主题」，在正常窗口和 720 × 560 窄窗口分别查看「甜点偷笑」与「馋嘴豆包」，保存无敏感内容的截图并核对图标、输入框、侧栏和文字可读性。
8. 将命令、结果、截图和残余限制写入 `verification.md`；实现会话不自行把 fresh-context 验证状态改为通过。

## Test-first proof

- 新增测试主题断言：顶层 `send` 与浅色变体 `main` 同时出现在 `preview_style().icons`，证明覆盖和回退可以共存。
- 对 bundled themes 断言：「甜点偷笑」的浅色变体 `main/newTask/voice` 进入预览结果；「馋嘴豆包」顶层 `main/dailyWork/readAloud` 保持可见。
- 断言浅色变体的 `placeholderColor`、`iconColor`、`radius`、`minHeight`、`padding`、`gap`、`iconSize` 覆盖顶层，未覆盖的布局字段从顶层进入预览。
- 实现前运行目标测试并确认新断言失败，实现后运行 `cargo test -p skin-core theme::tests --locked`、`cargo test -p doubao-skin-desktop ui_regression_tests --locked` 与 `cargo check -p doubao-skin-desktop --locked`。

## Visual or integration proof

- 默认 1120 × 720：分别选择「甜点偷笑」和「馋嘴豆包」，确认主图标、侧栏图标、顶栏图标、推荐图标和输入框图标来自对应主题；不存在图标叠加或占位块误回退。
- 最小 720 × 560：重复两主题检查，确认自定义主图标保持比例，侧栏/输入框不溢出，主题名称、描述和主按钮仍可用。
- 两个尺寸都核对输入框的背景、边框、占位文字色、图标色、圆角、高度和间距差异可辨认，背景图与不透明度预览不退化。
- 截图仅包含主题工具的静态模拟界面；不需要打开或读取豆包/豆包工作的真实会话。

## Risks and mitigations

- 变体合并遗漏字段：使用完整字段列表的结构字面量并以“变体覆盖 + 顶层回退”测试锁定，不采用散落在 UI 中的临时判断。
- 位图误走 SVG 着色：按小写扩展名分流；未知格式使用 `img()` 的保持比例回退，不改写文件。
- 极端几何参数破坏窄窗口：所有输入来自已校验 schema，再按统一比例限制到预览安全范围；正常/最小窗口截图双验收。
- 并行工作覆盖文件：修改前后检查目标片段与 mtime，只用 `apply_patch` 做局部编辑；不回退任何不属于本变更的修改。
- 预览与任意 CSS 仍可能不同：产品只承诺主题 v2 标准字段；在验证中明确记录静态预览不执行 DOM/CSS 细节。

## Rollback

- 代码回滚只移除 `PreviewStyle` 新字段、图标合并和桌面预览读取改动，不使用 `git reset --hard` 或 `git checkout --`。
- 回滚后恢复当前中性占位图标和固定预览几何；主题包、运行时 CSS/JS、用户偏好和官方应用均不受影响。
- 打包产物可重新运行 `scripts/build-macos.sh --universal` 从回滚后的源代码生成，不手工编辑 `.app` 或 zip。

## Deviations

无。若实现需要改变 schema、主题资产、运行时注入或新增预览模式切换，先更新规格和计划并重新确认。

## Decision

等待工程负责人确认本计划后开始修改产品代码与测试。
