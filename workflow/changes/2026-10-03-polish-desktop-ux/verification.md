---
id: "2026-10-03-polish-desktop-ux"
stage: verification
status: pending
owner: "claude"
created: "2026-10-03"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: polish desktop ux

## Automated checks

实现者(agent)在工作区未提交状态下运行的结果:

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --all` | 通过 |
| `./scripts/check.sh rust` | 通过(fmt、clippy `-D warnings`、全部测试);`doubao-skin-desktop` 34 个测试,`skin-core` 28 个集成测试,其余套件均 ok |
| `cargo clippy -p doubao-skin-desktop -p skin-core --all-targets --locked -- -D warnings` | 通过,无新增警告 |
| `./scripts/check.sh workflow` | 通过(13 个 artifact 集合,`test-devflow` 通过) |
| `./scripts/check.sh web` | 构建与静态页面生成成功;**末尾 `pnpm audit` 报告 1 个 critical(`next`,GHSA-vcvr-r3jv-pc5j,需 >=16.3.6)而失败**。该问题与本次变更无关(未改动 `apps/web` 源码或依赖),属于既有依赖问题,需另开变更处理 |
| `pnpm --dir apps/web sync` | 未改任何主题。执行后 `apps/web/public/themes/packages/*.zip` 与 `themes.db` 出现二进制差异(疑似非确定性打包),已 `git checkout -- apps/web` 还原;未提交生成产物 |

新增/重写的测试(均在对应实现之后与实现一并写入;状态机、路径校验、版本比较、文本缓冲为纯逻辑,可单测):

- `crates/skin-core/src/theme.rs`:`resolve_user_theme_dir_*`(接受真实用户主题 / 拒绝穿越与缺失 / 拒绝符号链接)、`is_user_theme_distinguishes_bundled_from_installed`、`newer_version_compares_numeric_segments`
- `apps/desktop/src/status.rs`:11 个测试(阶段机、通知作用域、失败文案归纳、商店安装状态)
- `apps/desktop/src/trash.rs`:真实移入系统废纸篓并清理 / 缺失路径报错
- `apps/desktop/src/search_input.rs`:10 个 `TextBuffer` 测试(字符边界、UTF-16 换算、组合输入标记、退格/删除)
- `apps/desktop/src/main.rs`:`menus_surface_the_import_find_and_target_shortcuts`、`menu_shortcuts_are_bound_to_their_actions`;`active_theme_is_scoped_to_its_target` 改写为基于 `ApplyPhase`(旧版依赖已删除的 `active_*` 字段)

## Behavioral evidence

- 应用阶段(`ApplyPhase`)与通知作用域(`Notice`)由单测覆盖:切换目标 / 切换主题不会把 A 的状态展示到 B;陈旧线程结果由 generation 丢弃。
- 删除只会移入废纸篓(NSFileManager `trashItemAtURL`),无永久删除回退;路径经 `resolve_user_theme_dir` 校验(主题 id 合法、拒绝以 `.` 开头、拒绝符号链接、规范化父目录必须等于用户主题目录)。
- 商店:面板内「重试」按钮(侧边栏与主面板两处共用 `render_retry_store_button`);「有更新 / 更新」基于 `is_newer_version`。
- 键盘:⌘O / ⌘F / ⌘1 / ⌘2 绑定与菜单项断言;搜索框为真实输入控件(移植自 GPUI 官方 input 示例),组合输入期间不发出 `Changed`。

## Visual evidence

在真实窗口(1120×721,仅用 window-id 截图)中确认,截图位于实现者本机 `/tmp/dsk/`(不入库):

- 「我的主题」:缺失目标应用横幅(「未检测到豆包工作…」)、「在 Finder 中显示」、「恢复默认」保持可用、主按钮显示「尚未安装」、搜索框显示 `⌘F` 提示。
- 「主题商店」(商店地址指向不可达端口):侧边栏显示「暂时无法连接」+ 原因 + 「重试」按钮。(首次截图发现侧边栏缺少重试按钮,已修复并复验。)

**未能在真实窗口验证,需人工/fresh-context 验证者完成:**

1. 窄窗口(最小宽度)下的上述界面。
2. 删除确认条与移入废纸篓的实际效果(列表过长且用户桌面其他窗口反复遮挡测试窗口,鼠标自动化不稳定)。
3. 空状态(无主题 / 搜索无结果)与「有更新」展示。
4. 菜单栏 ⌘O/⌘F/⌘1/⌘2 的实际触发。
5. **中文输入法(IME)组合输入**:搜索框能否正常上屏、候选窗位置、组合期间不触发过滤。
6. 应用成功路径:本机未安装豆包 / 豆包工作,只能验证失败与缺失目标路径。
7. VoiceOver 对搜索框(`Role::TextInput`,标签「搜索主题」)与按钮标签的朗读。

## Security and privacy evidence

- 未修改 `/Applications/DoubaoWork.app`;未改动协议桥;未新增第三方依赖(曾尝试给 `gpui` 增加 `test-support` dev-feature 做无头测试,会向 `Cargo.lock` 引入约 100 行传递依赖,违反 spec,已回滚,`Cargo.lock` 与 `Cargo.toml` 与基线一致)。
- `search_input.rs` 派生自 GPUI 官方示例(Apache-2.0),文件头已注明来源。
- 删除路径校验见上;失败文案不包含对话内容或凭据。
- 自动化操作披露:验证过程中曾用 `screencapture` 全屏 / 区域截图,意外截到用户其他窗口,已立即删除并改为仅 window-id 截图;另有一次合成键盘事件被投递到用户前台的游戏进程(Anno1800.exe)而不是测试窗口,此后停止一切合成键盘输入。

## Deviations and residual risk

偏离(均已在 spec 阶段向用户说明并获确认):

- 「恢复默认」不在无应用主题时禁用(需要清理以往会话残留的注入 CSS)。
- 商店范围缩小:卡片 / 详情 / 侧边栏原本已有「已安装」,本次只新增面板内重试与「有更新 / 更新」。
- IME 采用移植 GPUI 示例的方案,未做无头 GPUI 测试(见上),IME 行为只由 `TextBuffer` 单测与待补的真机验证覆盖。

其他:

- 审批人最初按系统用户名记为 `chenli`;用户事后明确指定审批人为 `idevlab`,三个阶段的 `approved_by` 已据此更正(用户在对话中逐阶段回复了「确认」)。
- 重写了一个旧测试(见上),因其依赖的字段被状态机取代。
- 既有 `next` critical 漏洞使 `check.sh web` 末尾失败,非本变更引入。
- `block v0.1.6` 的 future-incompat 警告为既有依赖问题。

残余风险:IME、窄窗口、删除到废纸篓的真实交互未经人工验证;合入前应由人工按上面「未能验证」清单走一遍。

## Verdict

[fill]
