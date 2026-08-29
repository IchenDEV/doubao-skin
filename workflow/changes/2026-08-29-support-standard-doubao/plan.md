---
id: "2026-08-29-support-standard-doubao"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: spec.md
risk: "high"
approved_by: "engineering-owner"
approved_at: "2026-08-29"
---

# Plan: 同时支持「豆包」和「豆包工作」

## Files and ownership

- `crates/skin-core/src/live.rs`：归属于本变更。增加 `TargetApp`、双端口/双进程生命周期、目标归属检查、实时恢复 JS 和相关单元测试。
- `crates/skin-core/src/lib.rs` 与 `crates/skin-core/Cargo.toml`：只更新对外模块说明，明确主题支持两款应用，协议桥/离线模式仍是工作版边界。
- `apps/desktop/src/main.rs`：归属于本变更。增加安装检测默认、目标偏好、双段选择器、键盘/VoiceOver 行为、目标感知的应用/恢复/激活状态、动态预览文案和 UI 回归测试。
- `apps/desktop/Info.plist`、`apps/desktop/Cargo.toml`、`scripts/build-macos.sh`：把用户可见的产品/安装包名更新为「豆包主题」，保留 bundle id 和 Rust package/bin 名以避免不必要迁移。
- `themes/*/theme.json` 与 `crates/skin-core/src/theme.rs` 中的对应回归断言：只把可见作者名从旧产品名机械更新为「豆包主题」，不改主题样式、资产或 schema。现有并行修改必须逐文件保留。
- `README.md`、`apps/web/src/lib/site.ts`、`apps/web/src/app/page.tsx`、`apps/web/src/app/layout.tsx`、`apps/web/src/components/SiteFooter.tsx`：更新产品名、支持范围和安装/使用文案；不把两款官方应用混成一款。
- `apps/web/data/themes.db` 与 `apps/web/public/themes/**`：仅由 `pnpm --dir apps/web sync` 根据主题 manifest 再生成，不手工编辑。
- `workflow/changes/2026-08-29-support-standard-doubao/verification.md`：在计划获批后由 `devflow verify` 生成，记录命令、版本、PID/端口、真窗截图、安全边界、偏差与剩余风险。

## Order of work

1. 在任何产品代码修改前，记录目标文件的 `git status`/差异和当前两款官方应用版本，确认并行改动边界。
2. 先在 `live.rs` 写会失败的纯函数回归测试：两个目标元数据、目标 URL 归属、通用页不可单独证明归属、错误端口拒绝和恢复 JS 的限定删除集。
3. 用一个 `TargetApp` 枚举参数化现有 `live.rs` 生命周期，保留工作版 `9222` 行为，新增标准版 `9223` 与目标身份检查；实现只清理工具自有 DOM 标记的实时恢复函数。
4. 在 `main.rs` 先加安装组合默认、已保存目标解析、active-theme/target 组合状态和动态预览文案测试，再接入应用/恢复调用。
5. 在现有顶部区域增加紧凑的双段目标选择器：宽窗与现有工具栏并列，720 px 窄窗仍保留标准窗口按钮、两个目标、安装与搜索主操作。补全悬停/选中/不可用态、`Command-1/2`、VoiceOver 标签和就地错误文案。
6. 将产品名和主要支持文案更新为双应用版本；仅机械修改主题 manifest 的 `author` 值，然后运行官方同步脚本再生成 Web 目录和主题包。
7. 先运行定向 Rust/UI 测试和 Web 同步/构建，修复与本变更直接相关的失败；再运行完整 Rust、Web 和 workflow 门禁，将无关并行失败与本变更结果分开记录。
8. 构建并启动真实「豆包主题」，分别对「豆包」和「豆包工作」完成应用、导航持续、目标切换隔离、恢复默认、正常/窄窗布局和键盘可达性验证。
9. 把实际结果写入 `verification.md`；实现会话不自行把验证状态设为 `passed`，交给 fresh-context 或人工验证者做最终判定。

## Test-first proof

- `cargo test -p skin-core live::tests --locked`：在实现前先证明标准版元数据、目标匹配、端口归属和恢复 JS 用例会失败，实现后变为通过。
- `cargo test -p doubao-skin-desktop ui_regression_tests --locked`：先覆盖四种安装组合、保存偏好回退、双 target active state 与预览身份，然后实现 UI 状态机。
- 针对现有「豆包工作」行为保留回归断言：端口仍为 `9222`、目标模式仍包含 `doubaowork://` / `chrome://doubaowork`、注入 JS 字节不因 target 重复生成。
- 针对「恢复默认」加回归断言：只删除 `doubao-skin-*`、`data-skin`、`data-doubao-theme-*` 标记，不包含读取聊天正文、未知 DOM 或删除官方持久数据的逻辑。
- `pnpm --dir apps/web sync` 后用定向查询确认 SQLite、catalog 和 zip manifest 的作者名一致，然后运行 `./scripts/check.sh web`。

## Visual or integration proof

- 开发工具窗口：在默认 1120×720 与最小 720×560 两种尺寸下截图，检查双段选择器、标准窗口按钮、安装/搜索/应用操作、文字截断、选中/不可用/悬停态和深浅外观。
- 标准版「豆包」：记录版本、PID 和 `9223` 的回环监听；在新建空白会话中应用一个纯色主题与一个背景主题，验证主页/主对话、导航后重注入、恢复默认和重启还原，同时证明「豆包工作」PID 未被误终止。
- 「豆包工作」：记录版本、PID 和 `9222` 的回环监听；在隔离会话中重复应用、导航持续、窗口唤醒、恢复默认和重启还原，同时证明标准版「豆包」PID 未被误终止。
- 两款应用都运行时往返切换一次，用 `/json` 目标 URL、监听端口与进程命令行证明 target 归属；不读取或保存聊天正文、Cookie 或请求数据。
- 截图只包含空白或无敏感测试界面，不包含用户会话或官方应用资产；证据路径与验证结果写入 `verification.md`。

## Risks and mitigations

- 误终止另一款应用：所有 quit/kill/open/activate 调用都只从 `TargetApp` 取受控定值；单元测试断言两套路径不交叉，真窗验证记录另一款 PID。
- 端口被错误对象占用：使用 `9222/9223` 分离加目标 URL 归属检查；检查不通过时停止并报错，不尝试杀掉端口占用者。
- 恢复 JS 过度删除：仅删除工具固定 id/attribute 且用字符串回归测试锁定；实测前使用空白会话。
- 两款 DOM/设计令牌不同：先以通用语义令牌和实际截图为验收；只在已证实有共享调用时调整现有 CSS，不为假想差异新建主题分支。
- 产品改名导致安装/更新不连续：保留 `dev.ichen.doubao-skin`、Rust package/bin 和 Application Support 路径，只改可见名称；打包后检查 Info.plist、签名和应用启动。
- 主题 manifest 与生成文件有并行改动：只修改 `author` 值，通过官方 sync 重生成，重生成后核对差异，不覆盖其他主题字段或资产。
- 无关工作流工件使全局检查失败：保留完整输出，同时用隔离 workflow root 验证本变更；不修改无关工件。

## Rollback

- 代码回滚：只回退本变更的目标差异，不使用会覆盖工作区的 `git reset --hard` 或 `git checkout --`；保留所有无关并行改动。
- 运行时回滚：停止主题 watcher，分别对两个可达 CDP 目标运行实时恢复，然后正常退出两款官方应用。不删除官方应用或用户数据。
- 产品回滚：保留原 bundle id 使旧版可覆盖安装；目标偏好文件为可忽略小文本，旧版不会读取它。

## Deviations

无。实现中如需移动文件、改变端口、扩大协议桥/离线边界、改变一次只操作一个目标的交互，或无法提供两款真实应用证据，必须先同步规格/计划并重新获得接受。

## Decision

工程负责人已确认本计划，同意按测试先行、双目标隔离、可访问原生交互与两款真实应用证据的顺序实施。
