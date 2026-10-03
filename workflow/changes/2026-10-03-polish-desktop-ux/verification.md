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

实现基于最新 `main` 的模块结构,在分支 `cursor/polish-desktop-ux-v2` 上重做(旧分支 `cursor/polish-desktop-ux` / PR #42 因 `main.rs` 被拆分为模块而冲突,已弃用)。以下结果均为实现者在工作区未提交状态下得到。

## Automated checks

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --all` | 通过 |
| `./scripts/check.sh rust` | 通过(fmt、clippy `-D warnings`、全部测试);`doubao-skin-desktop` 53 个测试,`skin-core` 80 个测试(含 `theme.rs` 45 个),其余套件均 ok |
| `./scripts/check.sh workflow` | 通过 |
| `./scripts/check.sh web` | 构建与测试通过;**末尾 `pnpm audit` 报告 1 个 critical(`next`,GHSA-vcvr-r3jv-pc5j,需 >=16.3.6)而失败**。本次未改动 `apps/web` 源码、依赖或生成物,属于既有依赖问题,需另开变更 |
| Windows 交叉 clippy | **未能本地验证**(`ring` 需要 MSVC 头文件);依赖 CI。非 macOS 路径的处理:快捷键使用 `secondary-`,删除入口隐藏(`trash::SUPPORTED`),文案为「在资源管理器中显示」 |

测试均与实现一并写入(未严格先写失败测试),属于对 AGENTS.md「先写回归测试」的偏离;覆盖点:

- `crates/skin-core/src/theme.rs`:`resolve_user_theme_dir_*`、`is_user_theme_*`、`newer_version_*`
- `apps/desktop/src/status.rs`:失败文案归纳、商店安装状态(安装 / 未安装 / 有更新)
- `apps/desktop/src/trash.rs`:真实移入废纸篓并清理 / 缺失路径报错
- `apps/desktop/src/search_input.rs`:`TextBuffer` 字符边界、UTF-16 换算、组合输入标记、退格 / 删除
- `apps/desktop/src/ui_regression_tests.rs`:菜单项 / 快捷键(`secondary-`)断言、被任一目标使用的主题不可删除

## Behavioral evidence

- 失败原因:`Msg::Done.error` 经 `describe_failure` 归纳后进入 `ThemeSessions` 对应的目标消息;无法连接时显示「无法连接到{应用}…」。
- 删除只会移入废纸篓(NSFileManager),无永久删除回退;路径经 `resolve_user_theme_dir` 校验(合法 id、拒绝以 `.` 开头、拒绝符号链接、规范化后父目录必须是用户主题目录)。正在被任一目标使用的主题,删除被阻止并提示。
- 商店:主面板与侧边栏共用 `render_retry_store_button`;「有更新 / 更新」基于 `is_newer_version`,`install_store_theme` 仅在已安装且无更新时阻止。
- 键盘:⌘O / ⌘F / ⌘1 / ⌘2 / ⌘3 与菜单项(文件 / 编辑 / 视图)绑定;搜索框为真实输入控件(移植自 GPUI 官方 input 示例),组合输入期间不发出 `Changed`;Esc / Tab 离开搜索框,上下键 / 回车在搜索框内仍可选主题。

## Visual evidence

真实窗口(仅 window-id 截图,不入库,位于实现者本机 `/tmp/dsk/`):

- 「我的主题」:缺失目标应用横幅(「请先安装豆包工作」)、「在 Finder 中显示」、搜索框 `⌘F` 提示、三目标切换、35 个主题。
- 详情底部操作行(用户反馈「按钮位置奇怪」后修复):
  - 「在 Finder 中显示」不再被推到左栏最右侧,紧跟状态消息。
  - 透明度滑块轨道与「恢复默认 / 应用」按钮同一水平中线(像素扫描:按钮中线 y≈1398,滑块轨道中线 y≈1402,2x 像素,偏差约 1pt;标签位于按钮上方)。按钮自身高度原本一致。

**未能在真实窗口验证,需人工 / fresh-context 验证者完成:**

1. 窄窗口(最小宽度)下的详情操作行、侧边栏与商店。
2. 商店错误面板与「重试」按钮(v2 上未重新截图;旧分支上侧边栏重试已验证)。
3. 删除确认条与移入废纸篓的实际效果、「有更新 / 更新」展示、空状态(无主题 / 无搜索结果)的两个按钮。
4. 菜单栏快捷键的实际触发。
5. **中文输入法(IME)组合输入**:上屏、候选窗位置、组合期间不触发过滤。
6. 应用成功路径:本机未安装豆包系应用,只验证了失败与缺失目标路径。
7. VoiceOver 对搜索框(`Role::TextInput`,标签「搜索主题」)与按钮标签的朗读。
8. Windows 构建与表现。

## Security and privacy evidence

- 未修改 `/Applications/DoubaoWork.app`;未改动协议桥;未新增第三方依赖(`Cargo.lock` 与基线一致)。
- `search_input.rs` 派生自 GPUI 官方示例(Apache-2.0),文件头已注明来源。
- 失败文案不包含对话内容或凭据。
- 自动化操作披露(旧分支验证阶段):曾用 `screencapture` 全屏 / 区域截图,意外截到用户其他窗口,已立即删除并改为仅 window-id 截图;另有一次合成键盘事件被投递到用户前台的游戏进程而非测试窗口,此后停止一切合成键盘输入。

## Deviations and residual risk

- 重做于重构后的 `main`:放弃自有的 `ApplyPhase` / `Notice`,复用 `ThemeSessions` 与 `message`(见 spec「Revision」与 plan「Deviations」)。**该偏离需要用户重新确认**。
- 「恢复默认」不在无应用主题时禁用(需要清理以往会话残留的注入 CSS)。
- 商店范围缩小:卡片 / 详情 / 侧边栏原本已有「已安装」,本次只新增重试与「有更新 / 更新」。
- IME 采用移植方案,未做无头 GPUI 测试(会向 `Cargo.lock` 引入传递依赖,违反 spec)。
- 审批人按用户指定记为 `idevlab`。
- 既有 `next` critical 漏洞使 `check.sh web` 末尾失败,`block v0.1.6` 的 future-incompat 警告同为既有问题。

残余风险:IME、窄窗口、删除到废纸篓、Windows 的真实交互未经人工验证;合入前应按上面清单走一遍。

## Verdict

[fill]
