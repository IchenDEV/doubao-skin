---
id: "2026-10-03-polish-desktop-ux"
stage: plan
status: accepted
owner: "claude"
created: "2026-10-03"
based_on: spec.md
risk: "low"
approved_by: "idevlab"
approved_at: "2026-10-03"
---

# Plan: 完善桌面端 UX

## Files and ownership

所有步骤在同一工作树中顺序进行（多个步骤共享 `apps/desktop/src/main.rs`，不并行、不拆 worktree）。

- `crates/skin-core/src/theme.rs`：新增 `resolve_user_theme_dir`、`is_user_theme`、`is_newer_version` 及其单元测试。仅新增，不改动现有函数。
- `apps/desktop/src/status.rs`（新）：`Notice`、`Tone`、`NoticeScope`、`ApplyPhase` 与失败原因归类，纯逻辑，含单元测试。
- `apps/desktop/src/trash.rs`（新）：`move_to_trash`，macOS 经 `cocoa`/`objc` 调用 `NSFileManager`，其他平台返回错误。
- `apps/desktop/src/search_input.rs`（新）：从 GPUI 示例移植的单行输入实体与事件，文件头保留 Apache-2.0 来源说明；文本缓冲层可单独测试。
- `apps/desktop/src/main.rs`：接入上述模块；替换 `applying/active_*/message/query/search_active`；扩展 `Msg::Done`；新增删除/Finder/重试/更新/空状态/提示卡渲染；扩展菜单与 `actions!`。
- `workflow/changes/2026-10-03-polish-desktop-ux/verification.md`：记录命令、结果、截图、偏差与剩余风险。
- 不修改 `crates/skin-core/src/live.rs`、`protocol_bridge.rs`、`themes/**`、`apps/web/**`、生成目录或官方应用；因不改主题内容，不运行 `pnpm --dir apps/web sync`，但在最终门禁中运行 `./scripts/check.sh web` 确认无影响。

## Order of work

每一步都是“先写会失败的测试 → 实现 → 跑最小相关检查”，步骤之间主分支始终可编译。

1. **基线**：记录 `git status`、当前分支（`main`）；运行 `cargo test -p doubao-skin-desktop -p skin-core --locked` 记录基线结果，确认所有后续失败都来自本变更。
2. **skin-core 辅助函数**（R3.12、R4.16）：
   - 先写测试：`resolve_user_theme_dir` 拒绝 `../x`、绝对路径、符号链接、不存在目录，接受合法用户主题；`is_user_theme` 区分内置与用户主题；`is_newer_version` 覆盖 `1.2.0>1.1.9`、`1.10.0>1.9.0`、相等、不可解析。
   - 再实现，`cargo test -p skin-core theme --locked`。
3. **状态模型**（R1、R2）：
   - 新建 `status.rs` 并先写测试：作用域不匹配不显示；`Applying` 在 `Applied` 之前不为 `Active`；应用失败回 `Idle`；监听线程自然结束清除 `Active`；失败原因归类（中文透传、非中文归为连接失败）；恢复失败不出现“应用失败”。
   - 实现纯逻辑后再接入 `main.rs`：`Msg::Done` 增加 `error`，用 `phase`/`notice` 取代 `applying`、`active_*`、`message`，保留 `theme_is_active` 语义与现有测试；替换所有 `message.contains("失败")`。
   - 完成后运行 `cargo test -p doubao-skin-desktop --locked`，并启动应用冒烟：应用、切换主题、恢复。
4. **主题管理**（R3）：
   - `trash.rs` 先写测试（临时目录移入废纸篓后原路径消失；不存在路径报错）。
   - 在 `main.rs` 增加“在 Finder 中显示”（`cx.reveal_path`）、仅用户主题的“删除”、内联确认状态 `confirm_delete: Option<String>`（切换选中/视图/Esc 取消）、活动主题禁用删除、删除后重载并选中相邻主题。
5. **商店**（R4）：错误面板加“重试”；按 `is_newer_version` 在卡片、详情、侧栏显示“有更新/更新”，复用 `install_store_theme`。
6. **空状态与提示卡**（R6）：目标应用未安装的提示卡；无主题时的两个入口。
7. **搜索输入与菜单**（R5）：
   - `search_input.rs` 先写文本缓冲层测试（UTF-8 中文、光标、选择替换、粘贴），再移植示例的元素与 `EntityInputHandler`。
   - 在 `SkinApp` 中以 `Entity<SearchInput>` 替换 `query/search_active` 与手写字符拼接，订阅 `Changed/Submit/Cancel/Focus`；保持 ⌘F、↑/↓、Enter、Esc 行为。
   - 扩展 `application_menu()` 与 `actions!`（⌘O、⌘F、⌘1、⌘2）；占位符“搜索主题  ⌘F”。
   - 在真实窗口用拼音输入法验证；若组合输入不稳，停止并按规格降级、先更新 `plan.md` 的 Deviations 再继续。
8. **全量门禁**：`cargo fmt --all`、`./scripts/check.sh rust`、`./scripts/check.sh workflow`、`./scripts/check.sh web`（确认未受影响）。
9. **真实窗口验证**：`cargo run -p doubao-skin-desktop`（`1120 × 720`），逐项执行规格的验收第 3 条并截图；关闭豆包后的失败、应用后退出豆包、断网商店等场景按需构造。
10. **记录**：把命令、结果、截图、偏差和残留风险写入 `verification.md`；`status: pending`，由全新上下文的验证者或人类给出最终结论；实现会话不自行宣布通过。

## Test-first proof

- 步骤 2、3、4、7 都在实现前运行新增测试并保存失败输出（编译失败或断言失败）到 `verification.md`，实现后重跑并记录通过。
- 状态模型改动前先固定现有行为：保留并运行已有 `active_theme_is_scoped_to_its_target`、`main_window_is_fixed_at_the_approved_size`、布局测试，确保重构不破坏。
- 可复现缺陷（旧提示残留、失败文案对恢复也显示“应用失败”、应用中提前显示“正在使用”、监听结束后仍显示“正在使用”）各自对应一条回归测试，且先于修复运行。

## Visual or integration proof

- 默认 `1120 × 720` 窗口截图：应用中/成功/失败；切换主题后信息行；删除确认态与废纸篓中可找回的 Finder 截图；商店错误态与“有更新”；空状态与提示卡；搜索框中文输入与菜单栏快捷键。
- 全程使用 `DOUBAO_SKIN_USER_THEMES_DIR` 指向临时目录来准备用户主题，不污染真实用户主题；商店断网通过指向不可达的 `DOUBAO_SKIN_THEME_STORE_URL` 复现；“有更新”通过本地构造较低版本的用户主题复现。
- 目标应用失败场景不修改官方应用：用端口占用或应用未安装（仅在测试逻辑中）复现；涉及官方应用的验证只读，不展示会话内容。
- 因窗口固定，不做窄窗口实窗验证；紧凑/矮布局仅依赖现有布局单元测试，并在 `verification.md` 说明。

## Risks and mitigations

- **输入法移植**：分步骤最后做，且与其他改动解耦；失败时可单独回退 `search_input.rs`，其余功能不受影响。
- **状态机重构面广**：先写纯逻辑测试、再接入；保留旧字段到接入完成再删除，每步保持可编译、可运行。
- **废纸篓 FFI**：封装在一个函数内，单独测试；失败不做任何回退删除。
- **`main.rs` 并行修改**：每次编辑前重新读目标片段并核对 `git status`；不回退他人改动，不使用 `git reset --hard`/`git checkout --`。
- **许可**：移植的 GPUI 示例为 Apache-2.0，保留版权与来源注释；不新增依赖。
- **范围蔓延**：任何超出规格的新增（例如 toast、模态框、新主题格式）先更新 spec/plan 并重新确认。

## Rollback

- 每个步骤独立提交（实现会话不自行提交，除非用户要求）；回滚按步骤反向移除对应新增代码，不使用 `git reset --hard`、`git checkout --`。
- 搜索输入可单独回退为旧的 `key_down` 拼接实现；删除功能可单独隐藏入口而保留纯函数；状态机若出现严重回归，在接入前的提交点回退。
- 不涉及数据迁移或持久化格式变化：用户偏好文件与主题目录结构不变，回滚无需清理用户数据。

## Deviations

相对于已确认的 intent，规格已记录两处偏差并经确认：恢复默认不禁用；商店不再处理“已安装”标记（已存在），改为“重试 + 有更新”。实现中若还需偏离规格，先在此更新并重新确认。

## Decision

等待工程负责人确认本计划后开始修改产品代码与测试。
