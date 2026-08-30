---
id: "2026-08-30-fix-greeting-animation-overlay"
stage: intent
status: accepted
owner: "codex"
created: "2026-08-30"
source: "user"
risk: "medium"
approved_by: "user"
approved_at: "2026-08-30"
---

# Intent: fix greeting animation overlay

## Problem

豆包工作首页的问候语使用 `::after` 遮罩执行约 200ms 的揭幕动画。动画结束后，遮罩被平移到问候语右侧但仍保持绘制。豆皮为背景主题把 `--chat-bg-color` 投影为半透明页面色后，这个本应与不透明页面融为一体的遮罩会在壁纸上重复合成，形成约 `320 × 45 CSS px` 的亮色矩形；在 Retina 2× 截图中对应用户标注的 `640 × 90 px` 色块。

## Proposed outcome

保留问候语文字与揭幕动画，同时确保遮罩离开问候语区域后不再绘制到可见页面。修复应统一作用于使用运行时透明表面的背景主题，不为「咕嘎管理员」或其他单一主题增加专属分支。

## Affected users and systems

- 使用豆皮给豆包工作应用背景主题、并停留在首页空白会话的用户。
- `skin-core` 生成的运行时 CSS；主题包、桌面主题预览和官方应用文件保持不变。

## Constraints

- 不修改 `/Applications/DoubaoWork.app`，只通过现有 loopback CDP 注入路径修复。
- 不把 `--chat-bg-color` 恢复为全局不透明色，以免遮住主题背景。
- 不依赖完整哈希类名；官方构建更新后应尽量保持兼容。
- 先增加能捕获遮罩越界绘制的回归测试，再修改生成 CSS。
- 在真实豆包工作首页用背景主题验证正常与窄窗口，确认色块消失且问候语仍可读。

## Out of scope

- 重设计首页问候语动画、输入框或主题配色。
- 修改主题图片、图标、主题商店和网页 Gallery。
- 修复官方应用中与本遮罩无关的其他动画或 DOM 行为。

## Success signals

- 生成的运行时 CSS 对问候语动画遮罩建立明确的绘制边界，回归测试在修复前失败、修复后通过。
- 用户截图中的 `640 × 90 px` 亮色矩形在真实 Retina 窗口中不再出现。
- 问候语、主图标、推荐入口和底部输入框未被裁切或隐藏。
- `cargo test -p skin-core theme::tests --locked` 与最小相关 workflow 检查通过。

## Open questions

- `overflow: clip` 与 `overflow: hidden` 对官方 `::before` 滑块动画的兼容性需要通过真实窗口判定；优先选择不建立额外滚动容器且当前 Chromium 支持的最小声明。

## Decision

等待产品负责人明确批准本 intent 后进入 spec。
