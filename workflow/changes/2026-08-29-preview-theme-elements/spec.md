---
id: "2026-08-29-preview-theme-elements"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: intent.md
risk: "medium"
approved_by: "product-risk-owner"
approved_at: "2026-08-29"
---

# Spec: preview theme elements

## Requirements

1. 主题详情预览必须使用 `preview.appearance` 对应的浅色或深色变体，而不是直接读取主题顶层字段。
2. 图标按字段合并：当前外观的 `variants.<appearance>.icons` 优先，未填写的图标回退到顶层 `icons`，两层都未填写时使用中性占位图标。
3. 主图标、侧栏入口、顶栏操作、首页推荐和输入框中当前可见的图标都必须读取同一份合并结果，避免同一个预览里混用主题图标与错误占位块。
4. 图标预览必须兼容主题标准允许的彩色位图主图标和单色 SVG；SVG 继续使用主题界面色着色，位图保持自身色彩和透明背景。
5. 输入框必须反映当前外观下的背景、边框、占位文字色、图标色、圆角、最小高度、内边距、间距和图标尺寸；变体字段优先、顶层字段回退。
6. 预览中已存在的侧栏宽度、页面留白和疏密程度必须读取相应 `layout` 配置，并限制在预览画布的安全范围内，不能挤压或遮挡输入框。
7. 未配置的字段必须保持稳定的系统回退值；旧版主题和只有部分 v2 字段的主题仍能生成完整预览。
8. 本变更不得改变主题运行时 CSS/JS、官方应用生命周期、主题包结构或主题应用结果。

## User experience

- 用户选中主题后，大预览立即展示该主题声明的预览外观和对应图标，不需要额外切换模式或刷新。
- 「甜点偷笑」应显示其 `variants.light.icons` 中的流口水豆包主图标与导航/输入框图标；「馋嘴豆包」继续显示顶层完整图标集。
- 输入框的颜色和几何差异应在当前 16:9 预览中直接可见，但预览仍是静态界面，不增加调试标签、字段说明或配置入口。
- 正常窗口和 720 × 560 最小窗口继续保持主题名称、预览和主操作可见；窄窗口允许压缩细节，但不得出现图标重复、溢出或内容互相覆盖。

## Technical design

- 在 `Theme::preview_style()` 中生成一次面向 UI 的解析结果：合并当前预览外观的 `composer`、`content` 与 `icons`，并把预览实际使用的布局/效果数值限制到 schema 允许范围。
- `PreviewStyle` 持有解析后的 `ThemeIcons` 和输入框/布局预览值。桌面渲染只读取 `ThemeRow.preview`，不再绕过预览投影直接访问 `row.theme.icons`。
- 图标渲染根据文件扩展名选择 `svg()` 或 `img()`；只有 SVG 使用 `currentColor`，位图使用 `ObjectFit::Contain` 保留原貌。
- 复用当前 `preview_icon`、`preview_main_icon`、`preview_nav_item`、`preview_action_item` 与 `render_preview`，不新增 Manager、Resolver、Factory 或第二套主题模型。
- 在 `skin-core` 增加回归断言，覆盖变体覆盖、顶层回退和元素字段解析；桌面端保留编译检查并通过真实正常/窄窗口截图验收。

## Security and privacy

- 预览只读取主题包安装时已校验并限制在主题目录内的本地资产路径，不访问网络、不执行资产内容中的脚本。
- 不读取、展示或保存豆包会话、账户、Cookie、工作区或附件；真实窗口验收只操作主题工具自身的静态预览。
- 不修改 `/Applications/Doubao.app`、`/Applications/DoubaoWork.app` 或用户主题文件。

## Alternatives and non-goals

- 不采用主题专属桌面分支：会让预览与主题标准继续漂移。
- 不解析或执行任意 `theme.css` DOM 规则来伪造完整客户端；颜色变量的现有解析仍作为标准字段的兼容回退。
- 不在一个静态画面中穷举发送、停止、代码块、选中文字、滚动条、悬停和动画等所有状态；只准确呈现当前预览画面中实际可见的元素。
- 不为无法在 GPUI 中注册的 Web 字体伪造字体效果；本变更聚焦图标、颜色和几何属性。

## Areas of concern

- 部分 SVG 可能内嵌固定颜色而不响应 `currentColor`；预览应尊重文件自身结果，不改写主题资产。
- 极端侧栏宽度、输入框高度和间距会在缩放后的 16:9 画布中失真；使用统一比例与安全上下限，而不是逐主题修正。
- `appearance: both` 的主题仍只展示 `preview.appearance` 声明的一个静态外观；这与主题包预览契约一致，不代表另一外观未受支持。
- 当前工作树有并行变更；实现仅修改预览投影、桌面预览调用和本变更工件，并在打包前核对源文件与产物时间。

## Acceptance criteria

1. `Theme::preview_style()` 对「甜点偷笑」返回浅色变体主图标和 `newTask` 等图标路径，对「馋嘴豆包」返回顶层对应路径。
2. 构造“变体只覆盖主图标、顶层提供发送图标”的测试主题时，预览结果同时包含变体主图标与顶层发送图标。
3. 变体输入框的背景、边框、占位文字色、图标色、圆角、高度、间距和图标尺寸覆盖顶层值；未覆盖字段继续回退。
4. 桌面端预览所有现有图标调用均来自解析后的 `PreviewStyle.icons`，彩色 PNG 主图标保持比例，SVG 不重复叠加。
5. `cargo test -p skin-core theme::tests`、`cargo test -p doubao-skin-desktop ui_regression_tests` 和 `cargo check -p doubao-skin-desktop` 通过。
6. 正常窗口和 720 × 560 窄窗口分别检查「甜点偷笑」与「馋嘴豆包」：自定义图标可见，输入框样式可辨认，无重复、裁切、溢出或文字不可读。

## Decision

等待产品与风险负责人确认本规格后进入实施计划。
