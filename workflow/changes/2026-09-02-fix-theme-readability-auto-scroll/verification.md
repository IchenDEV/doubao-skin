---
id: "2026-09-02-fix-theme-readability-auto-scroll"
stage: verification
status: passed
owner: "codex"
created: "2026-09-02"
based_on: plan.md
commit: "134cb1d"
verification_mode: "human"
verified_by: "user"
verified_at: "2026-09-02"
---

# Verification: fix theme readability auto scroll

## Automated checks

- 红灯证据（修复前）：
  - `cargo test -p skin-core resolved_runtime_mode_controls_text_semantic_polarity --locked` 失败；dark resolved runtime 仍生成 `--s-color-text-primary:#000000!important`。
  - `cargo test -p skin-core retained_session_yields_to_foreign_theme_owner --locked` 编译失败；旧实现没有 retained-session 的 foreign owner 判定 seam。
  - `cargo test -p skin-core live_runtime_defers_expensive_markers_until_scroll_settles --locked` 失败；旧 bootstrap 仍把 observer 变化交给包含全量 marker 扫描的单一 `apply` 路径。
- 绿灯证据（修复后）：上述三个定向测试均通过；`live_theme_runtime_replaces_previous_observer` 通过。
- 全量主题矩阵：`cargo test -p skin-core loads_bundled_themes --locked` 通过，34 个内置主题的 resolved light/dark 文本语义极性均符合要求。
- Core：`cargo test -p skin-core --lib --locked` 通过，73 passed、0 failed。
- Desktop：`cargo test -p doubao-skin-desktop --bin doubao-skin-app --locked` 通过，32 passed、0 failed；其中 matching generation 保存、同 target watcher replacement 与失败保护均保持绿色。
- 格式与生成同步：`cargo fmt --all -- --check` 通过；`corepack pnpm --dir apps/web sync` 成功同步 34 个主题及 skill discovery，主题内容无差异，仅有的源文件 mtime 生成时间噪音未保留。
- 仓库 Gate：`./scripts/check.sh web`、`./scripts/check.sh rust`、`./scripts/check.sh workflow`、`./scripts/check.sh all` 全部退出 0。唯一警告为未修改依赖 `block v0.1.6` 的 Rust future-incompat 提示。

## Behavioral evidence

- 对比度真实探针：豆包工作 `data-theme=dark` 下应用 `doubao-dessert-giggle` 后，`data-skin` 与 runtime theme 均为该主题；`--s-color-text-primary=#ffffff`、`--dbx-text-primary=rgba(255,255,255,0.90)`，问候标题 computed color 为 `rgba(255,255,255,0.9)`，backdrop veil 为 `rgba(18,20,25,0.58)`。
- 所有权真实探针：A=`teyvat-dandelion-wind --watch` 运行时单次应用 B=`doubao-dessert-giggle`；A 输出 `another theme took ownership — previous watcher stopped` 并正常退出。等待 4.5 秒（超过两个旧 2 秒维护周期）后，页面 marker/runtime 均稳定为 B，没有回切 A。
- 滚动红色基线：一次与主题无关的 DOM 变化触发 12 次 document query、219 次 rect 读取、191 次 style 读取。
- 滚动绿色探针：20 次 DOM 变化并连续滚动期间 document query 为 0，停止后为 12，对应 1 次合并 marker pass。真实侧栏连续滚动 10 秒记录 1231 帧，滚动期间 document query 为 0，停止后只执行 1 次 marker pass。
- 现场恢复：验收结束后工作树 CLI 成功重新应用测试前主题 `teyvat-dandelion-wind`；测试前记录的不透明度为 64%，自动保持与登录打开均为关闭，测试没有写入自动配置或启动项。
- 解锁后最终 UI 复核：以完整路径 `/Applications/豆皮.app` 打开已安装豆皮，明确选中“风与蒲公英”，界面不透明度显示 64%，自动保持为 off，登录时打开为 off；随后以完整路径 `/Applications/DoubaoWork.app` 打开真实豆包工作窗口，确认对应壁纸、主题图标、透明表面和文本层级均已恢复。
- 原生列表补充验收：对“全部主题”列表连续执行 12 次上下往返滚动，共 7.553 秒；滚动后窗口立即可读，仍选中“风与蒲公英”，64% 与两个 off 状态均未漂移，没有观察到输入迟滞或列表冻结。

## Visual evidence

- 修复前用户截图主文字与实际背景像素对比度为 2.91:1，低于 4.5:1。
- 修复后真实豆包工作截图中，标题、侧栏、输入区均为清晰浅色层级；代表前景 `#EEEDF1` 与背景 `#333135` 的像素对比度为 11.05:1。壁纸构图、彩色图标和透明表面仍可辨。
- 原始真实截图包含最近会话标题，按隐私约束没有复制进仓库；verification 仅保留不含正文的 computed style、像素对比度和计数结果。

## Security and privacy evidence

- 产品改动只涉及共享 CSS/运行时调度和根节点 `data-skin` 所有权判定；没有新增网络、遥测、文件扫描、配置 schema、依赖或宿主应用写入。
- CDP 性能探针只统计 query/layout/style 调用次数和 runtime marker，不读取或记录聊天正文、会话标题、账号、Cookie、通知或输入内容。
- 未修改 `/Applications/DoubaoWork.app`、WorkBuddy 或 `/Applications/豆皮.app` 的安装文件；真实验收只通过既有 loopback CDP 应用主题。

## Deviations and residual risk

- 已批准范围内无产品实现偏差；没有修改主题包、GPUI 列表、helper、自动配置或 Web catalog 内容。
- 现场仍存在另一个用户拥有的 `/Users/chenli/.codex/worktrees/5014/doubao-work-skin/dist/豆皮.app` 进程；按并发安全约束未停止或替换。本次最终复核始终用完整应用路径定位 `/Applications/豆皮.app` 与 `/Applications/DoubaoWork.app`，因此没有把另一 worktree 窗口当作验收对象，也未观察到它干扰主题恢复。
- retained watcher 在让权后作为成功返回，当前 CLI 会继续打印通用“主题已应用”完成文案；实际 marker/ownership 已正确，但 CLI 文案略有误导，未纳入本次用户报告的桌面行为范围。
- 本次测得并修复的是注入后豆包工作页面/侧栏的主题扫描热路径。约 34 项的原生豆皮主题列表没有测得可独立归因的 GPUI 热点，因此没有进行虚拟化或列表框架改造；若解锁后仍能稳定复现原生列表卡顿，应另开带计时证据的变更。

## Verdict

Passed. 用户于 2026-09-02 对包含本变更及后续首次应用/GPUI 滚动修复的集成验收构建明确给出“验收通过”。自动化、真实 CDP/像素/滚动探针、主题所有权稳定性与实窗可读性均满足当前验收项；记录的残余风险被接受。
