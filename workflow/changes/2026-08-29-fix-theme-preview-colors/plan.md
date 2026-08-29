---
id: "2026-08-29-fix-theme-preview-colors"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: spec.md
risk: "medium"
approved_by: "engineering-owner"
approved_at: "2026-08-29"
---

# Plan: 修复主题预览色差并审计全部主题

## Files and ownership

- `crates/skin-core/src/theme.rs`：增加保留 RGB/alpha 的预览颜色值，按当前外观解析 manifest 与 CSS 回退；扩展解析、变体和 bundled-theme 审计测试。保持运行时 CSS、主题加载和注入逻辑不变。
- `apps/desktop/src/main.rs`：让大预览所有颜色调用读取完整预览颜色值，并统一计算主题 alpha 与层级 alpha；增加纯色兼容和半透明合成测试。
- `workflow/changes/2026-08-29-fix-theme-preview-colors/verification.md`：记录判红复现、26 个主题审计、自动检查、真实窗口对照、截图和剩余风险。
- `themes/**` 默认只读审计；只有测试证明主题包自身另有独立错误时才做被证据锁定的最小修改，并同步 `apps/web` 生成目录。

## Order of work

1. 保存当前 `/tmp/doubao-theme-color-repro` 的判红输出，并核对目标 Rust 文件和并行改动边界。
2. 先在 `theme.rs` 增加失败测试：完整颜色解析、嵌入边框颜色、「鲸鱼娘」四个 alpha 字段和当前外观选择。
3. 增加轻量预览颜色值并实现完整解析；保留现有只需 RGB 的调用，使运行时主题生成不受影响。
4. 扩展 `PreviewColors`/`PreviewStyle` 和 CSS 外观作用域解析，确保 manifest、变体、顶层和 CSS 回退都返回 RGB/alpha。
5. 在桌面端增加一个 alpha 乘法辅助函数，逐项迁移大预览的背景、边框、文字与图标颜色调用；不修改主题卡片静态图片和非主题 UI 配色。
6. 增加遍历 bundled themes 的审计断言，记录当前 26 个主题的解析结果；只有审计明确失败才检查对应主题包声明。
7. 运行格式化、核心定向测试、桌面 UI 测试、桌面编译、Clippy 和 workflow；若主题包未修改，确认 Web catalog 无需重生成。
8. 与已批准的固定窗口变更一起构建通用 macOS 包，在真实 `1120 × 720` 窗口检查「鲸鱼娘」和代表性浅色/深色/纯色主题，并与静态预览和背景资产对照。
9. 删除临时复现目录，确认无调试输出残留，将命令、主题审计表、截图和限制写入 `verification.md`。

## Test-first proof

- 已有临时 harness 命令 `cargo run --quiet --manifest-path /tmp/doubao-theme-color-repro/Cargo.toml` 在实现前以 `E0610` 判红，证明 `PreviewStyle` 仍只携带 `u32`，无法表达 alpha。
- 核心回归先断言 `rgba(189,153,153,0.16)` 和 `1px solid rgba(122,78,41,0.28)` 的 RGB/alpha，再实现解析，确保不是只调桌面固定透明度。
- 对「鲸鱼娘」断言 `main/input/input_border/composer_placeholder` 的预期值；对合成函数断言 `0.16 × 0.60 = 0.096` 以及 `1.0 × 0.60 = 0.60`。
- bundled-theme 审计动态遍历主题目录，逐个验证当前外观颜色 alpha 有限、在 `0...1` 且与解析来源一致；本次输出应覆盖 26 个主题。
- 实现后重新运行临时 harness 和正式回归测试，确认原始判红场景转绿，再删除 `/tmp/doubao-theme-color-repro`。

## Visual or integration proof

- 固定尺寸 `1120 × 720` 下选择「鲸鱼娘」，确认背景天空/海水仍为浅蓝主色，大面积粉褐遮罩消失，棕色只保留在强调控件。
- 对照 `themes/gallery-whale-maid/bg.jpg`、`preview.jpg` 与无敏感内容的实际主题效果，确认不存在明显色相漂移。
- 另检查一个浅色无背景主题、一个深色背景主题和「纯暗」，确认半透明表面层次、输入框、边框、文字和图标可读。
- 将不透明度滑杆从最低到最高分段拖动，确认颜色连续变化、主题间不会跳成相同浑浊覆盖色。
- 截图使用最终打包应用；若对照官方客户端，只保留空白/退出登录页面，不记录会话内容。

## Risks and mitigations

- 迁移遗漏某个颜色调用：通过 `rg` 搜索 `style.colors`、`style.input`、`style.text` 等全部预览引用，并用编译类型变化迫使调用方逐项更新。
- alpha 在父元素与颜色上重复应用：只在颜色值上乘层级 alpha，不对承载子元素的容器整体调用 `.opacity()`。
- 纯色主题视觉退化：`#rrggbb`/`rgb()` 的 alpha 固定为 `1.0`，桌面兼容测试锁定原输出。
- CSS 外观作用域误选另一变体：复用现有 `preview_mode` 选择逻辑，并保留浅/深互斥回归。
- 并行工作覆盖主题或 Rust 文件：局部补丁、目标 mtime 核对、禁止回退他人改动；主题包默认不写。

## Rollback

- 回滚只把预览颜色投影恢复为原 `u32` 字段，并还原桌面预览的颜色调用；不使用破坏性 Git 命令。
- 不回滚或重写主题包、运行时 CSS、用户不透明度偏好、官方应用状态和其他并行变更。
- 从回滚后的源代码重新生成测试/打包产物，不手工编辑 `.app`、Web catalog 或主题 ZIP。

## Deviations

无。若真实对照证明问题来自主题包配色而非 alpha 丢失，先更新规格和计划并重新确认，不在本实现中凭视觉批量调色。

## Decision

等待工程负责人确认本计划后开始修改产品代码与测试。
