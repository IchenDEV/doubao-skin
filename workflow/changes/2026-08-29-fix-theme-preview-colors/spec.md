---
id: "2026-08-29-fix-theme-preview-colors"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: intent.md
risk: "medium"
approved_by: "product-risk-owner"
approved_at: "2026-08-29"
---

# Spec: 修复主题预览色差并审计全部主题

## Requirements

1. 预览颜色解析必须同时返回 RGB 和 alpha，支持 `#rrggbb`、`rgb(r,g,b)`、`rgba(r,g,b,a)`，也支持边框等字符串中嵌入的颜色表达式。
2. `rgba()` 的第四个分量必须接受 `0.16`、`.16`、`1` 等现有主题写法并限制在 `0...1`；未提供 alpha 的颜色按 `1.0` 处理。
3. 当前预览外观变体的 `content`、`composer` 和 CSS 变量颜色必须保留各自 alpha；变体优先、顶层回退和 CSS 回退顺序保持不变。
4. 预览画布的侧栏、主内容、浮层、输入框、边框、正文、占位文字、图标和强调色必须使用解析后的 alpha，不能在桌面端再次退化为纯 RGB。
5. 用户选择的界面不透明度必须与主题 alpha 相乘；主题为纯色时保持当前不透明度行为，主题本身半透明时不得被全局值覆盖成更浑浊的颜色。
6. 「鲸鱼娘」浅色预览必须保留主内容 `0.16`、输入框 `0.96`、输入框边框 `0.28` 和占位文字 `0.60` 等原始 alpha，整体仍以背景画作的浅蓝色为主。
7. 当前全部 26 个内置主题必须通过统一颜色投影审计；不得为特定主题增加渲染分支。
8. 本变更不得改变运行时主题 CSS、注入结果、不透明度滑杆范围或官方应用生命周期。

## User experience

- 用户选择「鲸鱼娘」后，大预览应与背景画作和实际主题保持同一浅蓝主色，不再覆盖大面积粉褐色；棕色仍可作为按钮、边框等小面积强调色。
- 其他浅色、深色、有背景和无背景主题也应保留各自半透明层次，不能因修复「鲸鱼娘」而变得过亮、过暗或文字不可读。
- 用户拖动界面不透明度时，预览继续即时变化；变化建立在主题原始 alpha 之上，不抹掉主题作者的透明关系。
- 不增加调试标签、色值说明或新的配置控件。

## Technical design

- 在 `crates/skin-core/src/theme.rs` 增加一个轻量的预览颜色值类型，持有 `rgb: u32` 与 `alpha: f32`；`PreviewColors` 和 `PreviewStyle` 中所有可见颜色统一使用该类型。
- 保留现有只需 RGB 的 `parse_color_value()` 调用边界；新增/重构预览解析函数返回完整颜色值，避免影响基色、色板和运行时 CSS 生成。
- `preview_css_color()` 增加等价的完整颜色投影，使 CSS 变量的外观作用域选择继续正确且保留 alpha。
- 在 `apps/desktop/src/main.rs` 用一个直白的 alpha 合成辅助函数统一计算 `theme_alpha × layer_alpha`，所有 `.bg()`、`.border_color()` 和 `.text_color()` 调用读取颜色值的 RGB 与合成 alpha。
- 不新增 Manager、Resolver 或第二套主题模型；颜色解析、外观选择和 UI 绘制仍分别留在现有核心与桌面文件。
- 在核心测试中遍历 bundled themes，对当前外观的预览可见字段逐项核对 RGB/alpha；桌面测试锁定 alpha 乘法和纯色兼容行为。

## Security and privacy

- 只解析已经通过主题包路径与 schema 校验的本地字符串，不访问网络、不执行 CSS/脚本、不加载新的外部资产。
- 不读取、展示或保存豆包会话、账户、Cookie、工作区数据或附件。
- 真实窗口验证使用主题工具静态预览；如需对照官方客户端，只使用无敏感内容的空白页面且不持久化会话截图。
- 不修改 `/Applications/Doubao.app` 或 `/Applications/DoubaoWork.app`。

## Alternatives and non-goals

- 不通过把「鲸鱼娘」的粉色字段直接改成蓝色来掩盖解析缺陷；该方式会让其他 25 个主题继续错误。
- 不全局降低预览不透明度；每个字段必须尊重主题声明的 alpha。
- 不采用完整 CSS 浏览器渲染器或复制豆包 DOM；GPUI 静态预览继续使用主题 v2 投影。
- 不重新生成背景画作、主题卡片预览或主题图标。
- 只有审计发现主题包声明与自身预览/描述存在独立且可复现的不一致时，才另行最小修正主题资产并同步 Web 目录。

## Areas of concern

- GPUI 的颜色 alpha 与元素整体 `.opacity()` 语义不同；实现必须对颜色做 alpha 合成，不能让文字和子元素随父容器重复变透明。
- 多层半透明背景会自然发生视觉合成；回归测试锁定单层输入，真实截图验证最终层叠结果。
- 部分主题使用 CSS 变量、manifest 变体和顶层字段的不同回退来源；全主题审计必须按 `preview.appearance` 选择，不能混入另一外观。
- 主题数以后可能变化；测试应遍历目录而不是硬编码 ID 列表，但本次验证需记录当前总数 26。
- 当前工作树存在并行编辑；主题文件默认只读审计，不覆盖尚未归属本变更的修改。

## Acceptance criteria

1. 临时复现程序和核心回归测试对「鲸鱼娘」得到 `main = (0xbd9999, 0.16)`、`input = (0xffffff, 0.96)`、`input_border = (0x7a4e29, 0.28)`、`composer_placeholder = (0x352970, 0.60)`。
2. `#rrggbb` 与 `rgb()` 解析结果 alpha 为 `1.0`；`rgba(...,.16)` 与嵌入 `1px solid rgba(...)` 的值保留正确 alpha，非法值安全回退。
3. 当前全部 26 个 bundled themes 的预览投影颜色 alpha 均为有限且位于 `0...1`，并与当前外观的 manifest/CSS 来源一致。
4. 桌面预览对主题 alpha 和层级 alpha 使用乘法；主题 alpha 为 `1.0` 时输出与现有纯色行为一致。
5. `cargo test -p skin-core theme::tests --locked`、`cargo test -p doubao-skin-desktop ui_regression_tests --locked`、`cargo check -p doubao-skin-desktop --locked` 和 `./scripts/check.sh workflow` 通过；若未修改主题包，无需重新生成 Web 目录。
6. 在真实 `1120 × 720` 主题工具窗口检查「鲸鱼娘」、一个浅色无背景主题、一个深色背景主题和一个纯暗主题：主色调正确、半透明层次可辨、文字和图标可读。
7. 「鲸鱼娘」修复后大预览以浅蓝为主，且与其 `preview.jpg`、`bg.jpg` 及无敏感内容的实际应用效果不存在明显色相偏离。

## Decision

等待产品与风险负责人确认本规格后进入实施计划。
