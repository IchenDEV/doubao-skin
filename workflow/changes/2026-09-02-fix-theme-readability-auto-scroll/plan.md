---
id: "2026-09-02-fix-theme-readability-auto-scroll"
stage: plan
status: accepted
owner: "codex"
created: "2026-09-02"
based_on: spec.md
risk: "medium"
approved_by: "user"
approved_at: "2026-09-02"
---

# Plan: fix theme readability auto scroll

## Files and ownership

- `crates/skin-core/src/theme.rs`：修正 resolved runtime 的文本语义极性与 composer fallback；拆分注入脚本的 appearance 维护和 marker 刷新；在现有 `theme::tests` 中加入 light/dark 极性矩阵和热路径结构回归。该文件是本次对比度与滚动修复的唯一产品实现面。
- `crates/skin-core/src/live.rs`：加入 retained-session 的 `Own / Missing / Foreign` 纯判定及测试；把 foreign marker 连接到 watcher 正常让权。该文件是主题所有权修复的唯一产品实现面。
- `apps/desktop/src/ui_regression_tests.rs`：原则上不改产品逻辑；只复用现有 apply generation/自动配置测试。仅当测试无法表达“连续 A→B 只提交 B”时，补一个不新增桌面状态的回归用例。
- `apps/web/data`、`apps/web/public/themes`：按已批准 Spec 运行现有 `corepack pnpm --dir apps/web sync`。若运行时代码变更不产生主题源差异，保持生成目录无改动；不得手工编辑 catalog 或安装包。
- `workflow/changes/2026-09-02-fix-theme-readability-auto-scroll/verification.md`：记录三条红绿链、真实 computed style、CDP 调用计数、主题/窗口恢复、适用 Gate、偏差和残余风险。实施者不填写最终 fresh-context verified verdict。
- 预计不修改任何 `themes/*/theme.json`、主题 CSS、GPUI 列表、helper、自动配置 schema、打包脚本或宿主应用文件。若红绿证据要求这些范围，先停止并更新 Spec/Plan 重新批准。

## Order of work

1. 确认 Intent/Spec 均已由用户批准、Plan 获批前无产品代码改动，worktree 只包含本 change artifact；记录真实豆皮当前目标、主题 `teyvat-dandelion-wind`、64% 不透明度、自动保持关闭和豆包工作 dark appearance，避免测试后无法恢复。
2. 在 `theme::tests` 先加入对比度红灯：直接解析 preview=light、appearance=both 的“甜点偷笑”，分别构建 resolved light/dark runtime CSS；断言 dark 基础 `!important` 语义文本为浅色、light 为深色，并对全部内置 both 主题做同极性检查。旧实现必须因 dark 基础值仍为 `#000000!important` 失败。
3. 提取一个最小私有“当前 runtime 是否使用深色语义”判定：明确 `ThemeMode::Dark`/`Light` 直接决定极性，只有 `Auto` 使用现有背景亮度推断。让 `semantic_css` 和 composer surface fallback 共用该判定；不改作者声明的 composer、消息、强调色和 surface opacity 公式。运行同一测试转绿。
4. 在 `live::tests` 先加入 marker 三态红灯：`None→Missing`、相同 ID→Own、不同/空字符串→Foreign`；再加入 watcher 决策断言，证明 Foreign 不是 reinject。旧实现没有 foreign 分支，测试必须先失败。
5. 实现最小 `RetainedSessionAction` 纯函数并接入 retained CDP session：Own 保持、Missing 重注入、Foreign 有限日志后让整个 target watcher 正常返回。保留导航、cross-site 和 dead-session 现有路径，运行 live 定向测试转绿。
6. 在 `theme::tests` 加入滚动热路径红灯：生成的 bootstrap 必须具有独立 appearance/marker 调度、passive capture scroll listener 和完整 destroy 清理，并且 MutationObserver 不再把所有记录直接交给旧 `schedule→apply→markIcons`。旧脚本结构必须先失败。
7. 在 `render_bootstrap` 中保留初次同步 `apply appearance + mark`，随后拆为：根 appearance/media 变化合并维护 CSS/backdrop；childList/aria/title 只标记 marker dirty；scroll 持续推迟 marker；安静窗口后执行一次 `markIcons/markComposerIcons`；周期恢复也通过同一调度 seam。`destroy` 清理两个 pending timer、interval、observer、media 和 scroll listener。
8. 用生成脚本单测转绿后，在真实豆包工作当前旧版本页面先复跑计数基线，随后只运行工作树生成的主题 runtime 做隔离绿灯：连续滚动/20 次子树变化期间 document query、rect、style 读取不按事件线性增长，停止后只合并一次 marker refresh；destroy 后计数不再变化。探针只统计 API 次数，不读取文本。
9. 运行现有 desktop 回归，覆盖 same-target watcher replacement、matching generation 保存和失败不覆盖。若 A→B 配置行为已被现有用例完整覆盖，不新增桌面测试或实现；这一步是 Ponytail 对未证实状态重构的停止点。
10. 运行 `cargo fmt --all`，再依次执行新增三个定向测试、`cargo test -p skin-core --lib --locked`、相关 desktop 定向测试和 `cargo test -p doubao-skin-desktop --bin doubao-skin-app --locked`。失败只处理与本变更直接相关的回归。
11. 执行 `corepack pnpm --dir apps/web sync` 并检查 diff；若没有主题源变化，生成目录必须保持无实质差异。随后运行 `./scripts/check.sh web`、`./scripts/check.sh rust` 和 `./scripts/check.sh workflow`。
12. 做真实窗口验收前记录豆皮配置与根 marker，干净退出已安装豆皮 watcher，确保没有两个 watcher 并发；从工作树应用“甜点偷笑”到豆包工作 dark 空白主页，检查 `--s-color-text-primary`、`--dbx-text-primary`、标题 computed color、实际像素对比度、壁纸/图标/45% 透明度和连续滚动。
13. 在同一隔离窗口连续应用主题 A、B，观察至少两个原 2 秒检查周期，确认最终稳定为 B、A watcher 正常退出、页面仅一个 runtime；再切换系统 light/dark、导航、恢复默认和连续切换 10 次，检查 observer/timer/marker 清理。
14. 验收结束恢复测试前的“风与蒲公英 / 64% / 自动保持关闭”与原窗口状态，并重新打开原豆皮应用；确认官方豆包工作与豆皮安装文件未被写入。若任何恢复步骤失败，先恢复现场，不继续跑全门。
15. 运行 `./scripts/check.sh all`、`git diff --check` 和最终 diff/scope 审计；创建并填写 `verification.md`，逐项记录命令、红绿结果、截图/计数路径、恢复结果、偏差和残余风险。将 artifact 留给独立 fresh-context verifier 或人类作最终 verdict，实施者不自批 verified。

## Test-first proof

- 对比度红灯：`cargo test -p skin-core resolved_runtime_mode_controls_text_semantic_polarity --locked`。旧实现从包含 light 规则的 CSS/预览背景推断文本极性，dark resolved runtime 仍生成 `--s-color-text-primary:#000000!important`，因此新断言必须失败；实现后同时覆盖 light、dark 与全部 both 主题。
- 所有权红灯：`cargo test -p skin-core retained_session_yields_to_foreign_theme_owner --locked`。旧 retained session 对任何非匹配标记都重新执行自己的 JS；新测试要求 foreign/empty yield、None reinject，旧实现必须失败。
- 滚动红灯：`cargo test -p skin-core live_runtime_defers_expensive_markers_until_scroll_settles --locked`。旧生成脚本包含 `new MutationObserver(schedule)`、`schedule→apply→markIcons` 和每 2 秒直接全扫描，新结构断言必须失败。
- 浏览器计数红灯沿用已记录真实基线：一次无关 DOM 变化为 12 次 document query、219 次 `getBoundingClientRect`、191 次 `getComputedStyle`。绿灯探针使用同一页面、相同主题、相同 20 次变更与滚动窗口；不以字符串测试替代实际计数。
- desktop 保护命令：`cargo test -p doubao-skin-desktop replacing_one_target_stops_only_its_previous_generation --locked` 与现有 matching-generation/失败不覆盖测试。它们必须保持绿色；没有新失败就不增加桌面状态 seam。
- 所有自动化测试使用仓库主题 fixture、不可达端口或纯函数，不修改 `$HOME` 下自动配置，不连接真实目标。只有明确标记的真实验收步骤接触当前豆包工作页面，并在前后恢复。

## Visual or integration proof

- **可读性。** 在豆包工作 dark 空白主页应用“甜点偷笑”45%，记录根 `data-theme=dark`、runtime theme ID、基础语义变量和问候标题 computed color；主要正文/侧栏/输入区达到 4.5:1，大号标题达到 3:1。截图裁掉账号、最近会话和通知。
- **主题视觉保真。** 与用户截图对照：壁纸构图、彩色图标、圆角、45% 透明度保持，变化只应是深色外观文本层级恢复清晰；浅色 appearance 抽查仍使用深色文字且没有白字反转。
- **所有权。** A watcher 运行时应用 B，至少观察 5 秒；CDP marker 始终为 B，A 线程正常结束，不出现 A/B 往返。自动配置集成测试最终只记录 B；真实验收不擅自开启用户的登录项或系统自动启动。
- **滚动。** 在主页、最近会话侧栏和可滚动内容区分别连续滚动至少 10 秒；计数器证明滚动期间昂贵 marker 扫描被延后，停止后一次收敛，肉眼无旧版周期性顿挫。仅在豆皮原生主题列表仍能独立稳定复现时记录为未完成后续，不临时重写 GPUI。
- **生命周期。** light/dark 切换、导航、恢复、10 次换主题后，检查单一 `__doubaoSkinRuntime`、style/backdrop/marker 与 listener/timer 清理；恢复默认不残留主题资源。
- **现场恢复。** 最终再次确认 `teyvat-dandelion-wind`、64%、自动保持关闭、原豆皮进程/窗口状态与测试前一致；官方应用签名/文件只读状态无变化。

## Risks and mitigations

- **Auto 旧包极性回归。** 只让明确 Light/Dark 绕过亮度推断，Auto 继续用现有算法；全量 core、旧主题和 CLI 测试保护兼容路径。
- **兜底变量压过作者色。** 修复只纠正基础语义极性，不重排已接受的 v3 cascade，也不改 composer/消息/强调色字段；代表主题同时断言这些声明保持。
- **foreign 误判导航。** 可选 marker 的 `None` 与非空/空字符串严格分支；缺失继续重注入，foreign 才让权。多页面 target 任一已注入主页面被 foreign 接管时让出整个旧 watcher，避免侧页残留。
- **marker 延迟过长或过短。** 用真实计数 harness 选择最小稳定 debounce；初次标记仍同步，动态内容只在滚动结束后短暂收敛。若需要超过可感知延迟才能稳定，停止并回到 Spec 而不是硬加长 timer。
- **页面重建丢失 style/backdrop。** appearance 维护保留周期恢复和导航脚本；marker 分离不删除现有 new-document 注入。导航/恢复/销毁都有测试和真实复验。
- **测试与用户 watcher 争抢。** 真实验收前干净退出已安装豆皮，只允许工作树 watcher；不 kill 官方豆包工作、不并发两个 runtime。验收后恢复主题、配置和原应用。
- **原生主题列表仍卡。** 当前证据只定位到注入页面/侧栏，且商店约 34 项。若共享修复后原生列表仍稳定卡顿，记录残余问题并新开有计时证据的变更，不把未经测量的虚拟化塞进本 Plan。
- **Web 生成噪音。** 不手改生成目录；sync 只验证源一致性。若仅时间戳造成无关大 diff，记录工具行为并保留源未变事实，不提交无意义产物。

## Rollback

- 在真实验收中先调用现有“恢复默认”或 watcher destroy，确认主题 runtime、style、backdrop 和 marker 清理；再恢复测试前主题。无需退出或修改官方豆包工作文件。
- 用 `apply_patch` 反向移除 `theme.rs` 的 runtime mode/调度修改和 `live.rs` 的 foreign marker 分支；保留 change artifact 与验证证据说明回滚原因。不得使用 `git reset --hard`、`git checkout --` 或覆盖用户其他改动。
- 本变更不迁移配置、不改 schema、不新增数据文件，因此代码回滚无需用户数据迁移。若 Web sync 意外产生无关文件，逐项确认其仅由本次 sync 生成后再用补丁恢复，不能清空生成目录。
- 回滚后运行三项旧行为相关测试、`cargo test -p skin-core --lib --locked`、desktop generation 测试、`./scripts/check.sh rust` 与 `./scripts/check.sh workflow`，并确认真实窗口恢复为测试前主题且不再有工作树 watcher。

## Deviations

- 计划阶段无偏差。任何主题清单特判、GPUI 列表重构、新依赖、新后台协调机制、helper/config schema 修改或宿主 selector 补丁都超出批准范围，必须暂停并重新走 Spec/Plan Gate。

## Decision

等待用户明确批准本 Plan 后，才开始新增失败测试和修改产品代码。当前只记录了 Spec 批准并编辑 Plan；真实豆包工作已恢复原主题，自动配置与安装资源未修改。
