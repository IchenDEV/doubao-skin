---
id: "2026-08-30-fix-greeting-animation-overlay"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-30"
based_on: intent.md
risk: "medium"
approved_by: "user"
approved_at: "2026-08-30"
---

# Spec: fix greeting animation overlay

## Requirements

1. 豆包工作首页问候语容器必须裁切超出自身边界的动画伪元素，使官方 `::after` 遮罩在完成平移动画后不再覆盖问候语右侧的主题背景。
2. 修复必须保留问候语正文、主图标、推荐入口和现有约 200ms 揭幕动画；不得直接隐藏整个问候语节点。
3. 修复必须限定到 `data-skin-target=doubao-work`，不在未验证的标准版豆包页面上扩大行为范围。
4. 选择器可以使用官方 CSS Module 的稳定语义前缀 `greeting-text-`，但不得硬编码当前构建哈希 `Q0pGud`。
5. `--chat-bg-color`、`--chatarea-bg-color` 和运行时表面透明度保持现有值；不得通过恢复不透明页面色掩盖问题。
6. 所有背景主题统一获得修复，不得为「咕嘎管理员」「璃月华灯」或其他单一主题增加专属 CSS。
7. 修改只进入 `skin-core` 生成的运行时 CSS，不修改官方应用、主题包、主题商店或桌面静态预览。
8. 在产品代码修改前增加回归测试，证明豆包工作注入结果缺少裁切规则时失败、包含目标限定且无硬编码哈希时通过。

## User experience

- 用户打开豆包工作空白首页时，问候语右侧不再出现亮色矩形；Retina 2× 下原 `640 × 90 px` 色块消失。
- 问候语和角色主图标正常显示，揭幕动画仍从文字区域内完成，不产生新的闪烁、裁字或滚动条。
- 正常窗口和窄窗口行为一致；不同背景画作只影响色彩，不再改变是否出现色块。

## Technical design

- 在 `Theme::surface_opacity_css()` 生成的运行时 CSS 中追加一个直接规则：仅当根节点同时具有 `data-skin` 与 `data-skin-target=doubao-work` 时，给类名包含 `greeting-text-` 的元素设置 `overflow:clip!important`。
- 使用容器裁切而不是改写官方 `::after` 的 `background` 或 `animation`。这样遮罩在问候语矩形内继续参与揭幕动画，平移到矩形外后由浏览器停止绘制。
- 不新增观察器、定时器、DOM 属性或第二套注入路径。现有 bootstrap 已负责写入 `data-skin-target`，生成 CSS 可以直接复用该边界。
- 在 `crates/skin-core/src/theme.rs` 的现有主题运行时测试附近增加回归断言，调用真实主题的 `theme_js(..., TargetApp::DoubaoWork)` 或实际 CSS 生成 seam，检查目标选择器、`overflow:clip!important`、保留的半透明 `--chat-bg-color`，并拒绝完整哈希类名。
- 真实窗口通过 CDP 读取问候语容器的 computed `overflow` 和 `::after` 几何，再采集只包含无敏感内容首页中心区域的截图；像素判定验证原色块区域与相邻页面不再存在重复表面合成差异。

## Security and privacy

- 继续只通过 loopback CDP 注入本地 CSS，不增加网络访问、权限、Cookie、header、会话或附件读取。
- 真实窗口验收只使用空白首页；保存的证据裁掉侧栏、账户信息和对话列表，不记录用户会话内容。
- 不修改 `/Applications/DoubaoWork.app` 或其资源文件。

## Alternatives and non-goals

- 不把 `::after` 设为 `display:none` 或透明：这会直接取消官方揭幕遮罩，改变动画效果。
- 不把 `--chat-bg-color` 改回不透明：会让页面中心重新遮住主题壁纸。
- 不覆盖官方哈希 keyframes 或完整哈希类名：构建更新后脆弱，且没有必要。
- 不用 MutationObserver 在运行时寻找问候语：一个目标限定的 CSS 裁切规则已经足够。
- 不顺带调整主题颜色、背景图、图标、输入框或推荐卡片样式。

## Areas of concern

- `overflow:clip` 会同时约束官方 `::before` 滑块；真实窗口必须确认它在文字区域内仍自然显示。若当前 Chromium 的最终行为不符合预期，才回到 spec 更新方案，不能静默换成 JS 定时器。
- CSS Module 的语义前缀理论上也可能被官方更名；相比完整哈希更稳定，但需在验证中记录这是外部 DOM 契约。
- 另一任务正在同一项目的独立 worktree 修改 `theme.rs`；本修复不得覆盖其更改，后续集成需基于最新共同提交顺序处理。
- 色块只在页面表面半透明且壁纸有对比时明显；自动回归必须断言生成规则，真实像素验证使用能稳定显色的「咕嘎管理员」浅色主题。

## Acceptance criteria

1. 回归测试在未添加裁切规则时失败，修复后通过，并覆盖目标限定、语义前缀和无完整哈希三项契约。
2. 豆包工作运行时 CSS 包含 `html[data-skin][data-skin-target=doubao-work] [class*="greeting-text-"]{overflow:clip!important;}` 或语义等价的最小规则。
3. 生成 CSS 仍包含半透明运行时 `--chat-bg-color`，没有主题 ID、`Q0pGud`、官方 keyframe 名或 JS 计时器特判。
4. `cargo test -p skin-core theme::tests --locked`、相关定向测试、`cargo fmt --check` 和 `./scripts/check.sh workflow` 通过。
5. 真实豆包工作空白首页中，问候语容器 computed `overflow` 为 `clip`，`::after` 仍执行官方动画，但其最终 `320 × 45 CSS px` 遮罩不再绘制到容器外。
6. 「咕嘎管理员」浅色主题在正常和窄窗口的中心区域截图均不再出现原 `640 × 90 px` 色块；问候语、主图标、推荐入口和输入框可见且未裁切。
7. `verification.md` 记录红/绿命令、真实窗口证据、外部 DOM 前缀依赖和任何偏差；fresh-context verifier 或人类再记录最终 verdict。

## Decision

等待产品与风险负责人明确批准本 spec 后进入 plan。
