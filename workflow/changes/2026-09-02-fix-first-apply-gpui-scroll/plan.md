---
id: "2026-09-02-fix-first-apply-gpui-scroll"
stage: plan
status: accepted
owner: "codex"
created: "2026-09-02"
based_on: spec.md
risk: "medium"
approved_by: "user"
approved_at: "2026-09-02"
---

# Plan: fix first apply gpui scroll

## Files and ownership

- `crates/skin-core/src/live.rs`：拥有首次注入阶段、dead target 重探测、整体期限、`once` 成功条件及其纯状态回归。复用现有 watcher、常量和测试模块，不新增线程或公共 API。
- `apps/desktop/src/app/types.rs`：增加三目标安装状态的小型值类型，支持一次探测和按 `TargetApp` 读取；不持有路径、锁或后台任务。
- `apps/desktop/src/app/mod.rs`：在 `SkinApp::new` 非渲染路径探测一次安装状态，复用同一结果选择初始目标，并把状态存入 view。
- `apps/desktop/src/ui/widgets.rs`、`apps/desktop/src/ui/detail.rs`：只把 render 中的实时 `is_installed()` 替换为内存状态读取；不改布局、样式、图片和交互结构。
- `apps/desktop/src/ui_regression_tests.rs`：覆盖三目标只探测一次、索引映射和现有 generation/失败保护；不建立 GPUI 测试框架。
- `workflow/changes/2026-09-02-fix-first-apply-gpui-scroll/verification.md`：记录红绿测试、fake CDP、采样对比、真实窗口状态、偏差与残余风险。上一变更 artifact 保持历史证据，不回写新结论。
- 预计不修改 `themes/`、`apps/web/`、自动主题 schema、安装器、helper、打包或官方应用文件。若实现需要这些路径，停止并重新批准 Spec/Plan。

## Order of work

1. 确认 Intent 与 Spec 已由用户批准、Plan 批准前不改产品代码；记录当前验收应用进程、`甜点偷笑`、52% 透明度、自动保持关闭、登录打开关闭和豆包工作主题 marker，避免测试后无法恢复。
2. 在 `live::tests` 先加入红灯：首次成功前 dead target 下一 tick 必须允许 probe；成功后第 1–14 tick 禁止、第 15 tick允许；旧实现只有统一 15 tick 逻辑，测试先失败。
3. 加入首次整体期限红灯：29.999 秒无错误不超时，30 秒时最近错误原样返回；30 秒从未得到注入错误时返回“未找到可注入页面”；旧 helper 对 `None` 永远不超时，测试先失败。
4. 在现有 watcher 中实现两个最小私有判定：`!applied_once` 立即 probe dead target，`applied_once` 后沿用 15 tick；首次期限从 `ensure_running` 返回后开始，任何首次状态到期都返回错误。只让 `once && applied_once` 返回成功。
5. 运行 live 定向测试，再用隔离 loopback fake CDP 做一次真实调用链红绿：第一次 WebSocket 注入失败，下一 watcher 周期恢复；同一个 `run --once` 调用成功退出。另一个永不恢复/无有效 target 场景在测试期限模型中失败，不连接真实应用。
6. 在 desktop 类型层先加入安装状态红灯：注入计数 closure 后三个目标各调用一次，读取状态不增加计数；同时覆盖 Doubao/DoubaoWork/WorkBuddy 映射。旧实现没有缓存类型，测试先编译失败。
7. 实现三布尔安装状态值，在 `SkinApp::new` 一次探测并复用到 `initial_target`。把 `render_target_switch` 和 `render_theme_detail` 改为只读缓存；保留 `switch_target`、`apply_selected` 和 `prepare_state` 的实时校验。
8. 运行 desktop 定向测试和现有 generation tests，证明失败 B 不覆盖 A、成功 B 仍提交、旧 generation 的 Done 不清除当前状态；不因已有覆盖充分而新增第二套 session 模型。
9. 执行 `cargo fmt --all`、core/desktop 完整测试、`./scripts/check.sh rust` 和 `./scripts/check.sh workflow`。迭代期间只修复本次改动直接造成的失败。
10. 从当前工作树构建新的唯一验收 app bundle，不覆盖 `/Applications/豆皮.app`。先在不应用主题的窗口复跑相同 12 次上下滚动和 `sample`，确认 render/draw 栈中 `registered_macos_bundle` 与 `osascript` 为 0，并记录耗时相对 9.181 秒红色基线的变化。
11. 在正常和窄窗口各连续滚动至少 10 秒，检查主题条目、缩略图、搜索、目标切换器、选中态和按钮响应；如果仍明显卡顿，重新采样后只处理新的确定热点，不猜测虚拟化。
12. 干净停止旧验收 watcher，使用新 bundle 对真实豆包工作做至少 5 轮冷启动首次应用；每轮只点击一次，记录成功时间或有限失败，确认没有无限 busy。不得同时运行两个争抢同一 target 的 watcher。
13. 复验超时/失败 desktop generation 离开 busy、不会保存失败主题；成功后按钮为“正在使用”。随后恢复 `甜点偷笑 / 52% / 自动保持关闭 / 登录打开关闭`，把新验收窗口留给用户继续检查。
14. 运行 `./scripts/check.sh all`、`git diff --check` 和 scope 审计；确认没有主题、Web 生成物、官方安装资源或无关用户改动被纳入。
15. 创建 `verification.md`，逐项记录命令、红绿证据、采样摘要、真实窗口结果、恢复状态和残余风险。保持 verdict `pending`，交给用户或独立 fresh-context verifier 作最终事实判断；不发布、不合并。

## Test-first proof

- `cargo test -p skin-core initial_dead_target_retries_before_steady_state_throttle --locked`：旧实现必须因第 1 tick 不 probe 而失败；修复后首次阶段立即允许，成功后仍严格 15 tick。
- `cargo test -p skin-core initial_injection_deadline_covers_missing_pages_and_last_error --locked`：旧实现对 `last_error=None` 返回 `None`，因此 30 秒空页面断言必须失败；修复后同时覆盖边界和最近错误。
- `cargo test -p skin-core once_mode_requires_a_successful_injection --locked`：旧分支在第一次循环零注入时仍返回成功；修复后只有 `applied_once` 可完成。
- 隔离 fake CDP harness 使用临时端口和 `DOUBAO_SKIN_DOUBAO_WORK_CDP_PORT`，只返回目标身份和合成 WebSocket 响应；第一次连接失败、第二次恢复。它验证真实 `targets → probe → inject_target` 调用顺序，不修改用户配置或官方应用。
- `cargo test -p doubao-skin-desktop target_installations_are_detected_once_and_read_from_memory --locked`：旧实现没有状态类型而判红；绿灯证明 detect closure 只被调用 3 次，后续读取为 0 次。
- 既有 `only_matching_generation_updates_saved_theme`、失败不覆盖、same-target replacement 测试必须保持绿色；若测试名有漂移，使用当前等价测试并在 verification 记录精确名称。

## Visual or integration proof

- **首次应用。** 对真实豆包工作执行至少 5 轮冷启动；每轮记录点击到 `正在使用` 的时间。任何错误都必须在有界时间离开 `正在应用…`，不通过第二次点击掩盖失败。
- **性能。** 在同一机器、同一正常窗口和相同 12 次上下滚动脚本复测。旧基线为 9.181 秒，采样 605 个绘制样本中 587 个阻塞在安装探测；新样本 render/draw 路径中 `registered_macos_bundle` / `osascript` 必须为 0。
- **交互。** 正常与窄窗口各连续滚动 10 秒后立即选择可见主题、操作搜索并切换已安装目标；状态变化应立即反馈，没有长达数百毫秒的主线程停顿。
- **视觉保真。** 缩略图、选中底色、active 勾选、目标“未安装”标签、详情预览和 52% 不透明度不变；本次不以重排或隐藏条目换取流畅。
- **现场恢复。** 最终新验收 app 显示 `甜点偷笑 / 52% / 两个开关关闭 / 正在使用`，豆包工作保留对应主题。官方应用与已安装豆皮文件不被写入。

## Risks and mitigations

- **首次 probe 重新变高频。** 条件必须显式绑定 `!applied_once`，成功后现有 15 tick 测试保护稳定期；不修改长期 interval 常量。
- **30 秒误伤慢启动。** 期限只在 `ensure_running` 确认目标身份后开始，保留原有 30 秒而不缩短；冷启动 5 轮验证实际余量。
- **错误后 busy 未清理。** 沿用匹配 generation 的 `Done` 路径，不新增 UI watchdog；desktop 回归覆盖成功、失败和过期消息。
- **缓存状态过期。** render 缓存只影响本窗口标签；点击执行仍实时校验。窗口打开后新安装官方目标需要重开豆皮，这是接受的最小实现上限。
- **误把图片当根因。** 原采样图片加载仅 1 个样本，先只移除占 587 个样本的同步安装探测。若绿灯样本出现新的主要热点，再拿证据回到新 Plan。
- **测试 watcher 争抢。** fake CDP 使用隔离端口；真实验收先停旧 watcher，只允许一个新 watcher控制豆包工作，结束后恢复原主题状态。
- **当前用户改动。** `live.rs` 已含上一批准变更的未提交修改；在同一文件顺序编辑，不回退、不覆盖，并用 diff 分段审计归属。

## Rollback

- 先停止新验收 watcher，通过现有应用/恢复路径把豆包工作恢复为 `甜点偷笑` 52%，确认用户窗口可继续使用。
- 用 `apply_patch` 反向移除 `live.rs` 的首次阶段判定与对应测试，以及 desktop 安装状态字段和 render 读取；不得使用 `git reset --hard`、`git checkout --` 或覆盖上一变更。
- 回滚不涉及配置迁移、主题资源或 schema；运行现有 live、desktop generation、Rust 和 workflow 检查，确认旧行为之外没有新残留。
- 临时 fake CDP、profile 和验收 bundle仅位于忽略的 `target/` 下；验证完成后可保留供本地复核，不纳入仓库交付。

## Deviations

- 计划阶段无偏差。虚拟列表、图片压缩、异步安装轮询、LaunchServices 重写、新依赖、错误 UI 重设计、主题资源或 Web catalog 修改均超出已批准 Spec，必须停止并重新走 Gate。
- 若真实绿灯采样移除 `osascript` 后仍存在可稳定复现的主要卡顿，本变更只记录新热点并请求更新 Plan，不以主观感觉继续扩写范围。

## Decision

等待用户明确批准本 Plan 后才新增失败测试和修改产品代码。当前只记录了 Spec 批准并编写执行计划，当前验收应用和官方目标状态未改变。
