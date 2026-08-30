---
id: "2026-08-30-fix-greeting-animation-overlay"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-30"
based_on: spec.md
risk: "medium"
approved_by: "user"
approved_at: "2026-08-30"
---

# Plan: fix greeting animation overlay

## Files and ownership

- `crates/skin-core/src/theme.rs`：在现有运行时表面 CSS 中加入豆包工作问候语容器裁切规则，并在同模块测试中锁定目标选择器、声明、透明变量保留和禁止完整哈希。只修改这一处产品代码。
- `workflow/changes/2026-08-30-fix-greeting-animation-overlay/verification.md`：记录红/绿测试、Rust/workflow 门禁、真实 DOM computed style、中心区域截图、像素判定、恢复结果和残余风险。
- `workflow/changes/2026-08-30-fix-greeting-animation-overlay/evidence/`：只保存裁掉侧栏和账户信息的空白首页中心区域正常/窄窗口截图。
- 不修改 `themes/**`、`apps/**`、生成 Web 目录或 `/Applications/DoubaoWork.app`。
- 另一任务正在独立 worktree 修改同一个 `theme.rs`。本任务在它结束产品代码编辑前不进入实现；开始前读取其最终状态并基于最新共同提交工作，绝不回退或复制覆盖对方改动。

## Order of work

1. 核对当前 worktree、获批 intent/spec/plan 和另一个 Doubao Skin 任务状态；若对方仍在编辑 `theme.rs`，等待其完成后再继续，保持共享文件顺序执行。
2. 重新运行已建立的截图像素回放，确认用户原图仍稳定判红：上层矩形与页面 RGB 距离大、下层与页面接近；记录 `640 × 90` 物理像素症状。
3. 在 `theme.rs` 现有运行时表面测试附近先增加 `doubao_work_greeting_animation_mask_is_clipped` 回归测试：断言语义前缀、Work 目标范围、`overflow:clip!important`、半透明 `--chat-bg-color` 保留，并拒绝 `Q0pGud`。只运行该测试并确认修复前失败。
4. 在 `Theme::surface_opacity_css()` 的格式化字符串末尾追加获批的一条目标限定 CSS；不新增函数、类型、JS 或主题分支。
5. 运行同一回归测试确认转绿，再运行 `cargo fmt --check`、`cargo test -p skin-core theme::tests --locked`、`./scripts/check.sh rust` 和 `./scripts/check.sh workflow`。
6. 等使用豆包工作的并行验收任务释放应用后，用当前修复二进制将「咕嘎管理员」浅色主题应用到空白首页。通过 CDP 读取问候语容器与伪元素，只保留 `overflow`、几何、动画名和背景色，不读取会话文本。
7. 在正常窗口与约 916 × 768 的窄窗口分别检查：伪元素仍为约 `320 × 45 CSS px` 且动画存在，容器 computed `overflow=clip`；用 Computer Use 获取截图，再机械裁切为仅含首页中心区域的证据。
8. 对两张中心截图运行 ImageMagick 像素判定，确认问候语右侧没有原来的重复表面色块；人工核对角色图标、问候语、推荐入口和输入框未裁切。
9. 恢复进入验收前的主题、外观与窗口尺寸，删除临时探针，搜索 `[DEBUG-`，把命令、结果、证据、偏差和外部 DOM 前缀风险写入 `verification.md`。实现者不自行填写 fresh-context 最终 verdict。

## Test-first proof

- 失败命令：`cargo test -p skin-core doubao_work_greeting_animation_mask_is_clipped --locked`。旧实现生成半透明 `--chat-bg-color`，但没有约束问候语动画遮罩越界绘制，因此新断言必须失败。
- 通过命令保持完全相同，只在加入一条 CSS 后转绿。
- 测试读取真实 `surface_opacity_css()` 生成结果，而不是复制一段期望 CSS 到独立 helper；这条 seam 就是最终注入官方页面的样式来源。
- 断言同时保留 `--chat-bg-color:rgba(...)`，避免把“页面重新变不透明”误判为修复。
- 不为像素截图增加脆弱的仓库快照测试；真实 CDP + 中心截图像素判定作为原始场景反馈环，Rust 测试锁定可持续的生成契约。

## Visual or integration proof

- 目标主题：并行变更中的「咕嘎管理员」浅色版本，因为用户原图与现有实窗证据均能稳定显示色块；从其实际主题目录加载，不复制进本 worktree。
- 目标页面：豆包工作空白首页，不打开或保存任何对话正文。
- DOM 证据：根节点 `data-skin-target=doubao-work`；问候语类只记录 `greeting-text-` 语义前缀；父容器 `overflow=clip`；`::after` 尺寸、最终 transform、animation 和 background 仍存在。
- 正常窗口与窄窗口各一张中心裁切截图；窄窗口覆盖用户原始布局。证据文件不得包含侧栏、账户、最近对话或通知。
- 像素判定以问候语右侧原伪元素落点和邻近页面采样为准；修复前可见矩形的颜色距离必须消失，不能只依赖肉眼描述。
- 验收后用同一 CDP 结构探针确认临时样式/调试属性不存在，并恢复原主题与窗口状态。

## Risks and mitigations

- `overflow:clip` 同时裁切 `::before`：正常/窄窗口都观察揭幕动画和最终文字，不自然则停止并更新 spec，不临时换方案。
- 官方语义前缀将来可能变化：不使用完整哈希，且在 verification 明确记录这个外部 DOM 契约；未来失效会表现为回归探针找不到目标。
- 并行任务共享 `theme.rs` 和真实豆包工作窗口：通过任务状态等待和实施前 diff 核对保持顺序；验收前记录主题/外观/尺寸，结束后原样恢复。
- 截图泄露最近对话或账户：只保存中心裁切，裁切完成后逐张查看；原始整窗截图不进入仓库证据。
- 规则意外影响标准版豆包：选择器必须包含 `data-skin-target=doubao-work`，回归测试锁定；不对 `doubao` 目标作行为声称。
- `overflow:clip` 在旧 Chromium 不支持：当前豆包工作 Chromium 147 已实测支持；若产品最低支持版本不同，真实兼容证据不足时标记残余风险而非改用更宽副作用的 `hidden`。

## Rollback

- 用 `apply_patch` 删除 `surface_opacity_css()` 新增的一条选择器和对应回归测试，不使用 `git reset --hard` 或 `git checkout --`。
- 重新运行 Rust 定向测试和 workflow 检查，确认只恢复原问候语遮罩行为；主题包、用户数据和官方应用没有需要迁移或回滚的状态。
- 真实窗口若仍保留本任务应用的主题，仅调用现有 `doubao-theme apply` 恢复进入验收前的主题，不修改官方应用资源。

## Deviations

无。若需要改写官方伪元素动画、引入 JS、修改主题包、扩大到标准版豆包或改变 `--chat-bg-color`，必须先更新 spec/plan 并重新取得批准。

## Decision

等待工程负责人明确批准本 plan 后开始产品代码和测试修改。
