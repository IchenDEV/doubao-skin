---
id: "2026-08-29-preview-theme-elements"
stage: intent
status: accepted
owner: "codex"
created: "2026-08-29"
source: "user"
risk: "medium"
approved_by: "product-owner"
approved_at: "2026-08-29"
---

# Intent: preview theme elements

## Problem

主题详情的大预览目前只读取主题顶层 `icons`，没有合并 `preview.appearance` 对应的 `variants.light.icons` 或 `variants.dark.icons`。因此像「甜点偷笑」这样把图标放在外观变体里的主题会显示占位块。输入框虽然已经预览背景、边框和圆角，但占位文字色、图标色、图标尺寸、高度和间距等标准主题字段仍未反映，导致预览与实际界面不一致。

## Proposed outcome

让主题详情预览直接使用主题引擎解析后的当前预览外观：变体图标覆盖顶层图标，未覆盖字段继续回退；输入框、侧栏和推荐项尽量采用主题标准中已有的颜色与几何配置。主题作者只需维护 `theme.json`，无需额外维护一份桌面端预览配置。

## Affected users and systems

- 浏览、安装和制作主题的 macOS 用户与主题作者。
- `crates/skin-core/src/theme.rs` 的预览投影数据与回归测试。
- `apps/desktop/src/main.rs` 的原生主题预览及正常/窄窗口视觉验收。

## Constraints

- 预览必须保持数据驱动，不为单个主题写分支。
- 变体选择必须跟随主题包声明的 `preview.appearance`，与主题卡片预览一致。
- 未配置图标或元素属性时保留稳定、可读的系统回退样式。
- 不覆盖当前工作区中的并行改动，不修改主题运行时注入和官方应用安装包。
- 预览继续保持 16:9，并在 720 × 560 最小窗口中可用。

## Out of scope

- 不编辑、重画或补齐任何主题资产。
- 不执行任意 CSS 或复制豆包 DOM；只预览主题 v2 标准已建模的字段。
- 不改变主题应用、恢复、双目标选择、主题商店或打包协议。
- 不承诺在静态预览里重现视频动画、交互状态和所有聊天页面状态。

## Success signals

- 「甜点偷笑」浅色预览能显示其变体中的主图标和导航/输入框自定义图标，不再回退成占位块。
- 「馋嘴豆包」继续显示顶层完整图标集，顶层与变体的回退规则有自动化测试覆盖。
- 输入框的占位文字色、图标色、尺寸、高度、间距和圆角等已配置属性在预览中可辨认。
- 正常窗口和 720 × 560 窄窗口中，图标不重复、不溢出，预览与主题配色保持可读。

## Open questions

无。预览只承诺复用主题 v2 标准字段；任意 CSS 细节仍以实际客户端为准。

## Decision

等待产品负责人确认本意图后进入规格设计。
