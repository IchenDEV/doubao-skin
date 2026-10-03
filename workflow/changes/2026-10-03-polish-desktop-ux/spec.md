---
id: "2026-10-03-polish-desktop-ux"
stage: spec
status: accepted
owner: "claude"
created: "2026-10-03"
based_on: intent.md
risk: "low"
approved_by: "idevlab"
approved_at: "2026-10-03"
---

# Spec: 完善桌面端 UX

## Requirements

### R1 状态反馈（对应 intent 问题 1）

1. 状态消息必须带作用域和语气：`Notice { scope, tone, text }`，`scope` 为「全局」或「某主题 + 某目标应用」，`tone` 为 信息 / 成功 / 错误。界面只显示作用域与当前选中主题、当前目标应用相符的消息；切换主题或目标应用后，不相符的消息不再显示。不得再用 `message.contains("失败")` 判定颜色。
2. 应用过程分为明确阶段：`Idle`、`Applying`、`Active`、`Restoring`。`Active`（按钮显示“正在使用”、标题旁显示“已应用”）只在收到注入成功信号（`Msg::Applied`）后进入；点击应用到注入成功之间为 `Applying`，按钮显示“正在应用…”并禁用。应用失败直接回到 `Idle`，不经过 `Active`。
3. 应用与恢复失败时显示可读原因和下一步建议。`live::run` / `live::restore` 现有的错误已是中文且可读（如“未找到豆包…”“端口 … 已被其他程序占用，请关闭占用后再试”“…未开放本地调试端口，请打开应用后再试”），直接透传其文本；不含中文的底层错误（如 `cannot launch app: …`、CDP 连接/超时）统一归类为“无法连接到{应用}，请确认应用已打开后重试”，原始文本仍写入内部日志。`Msg::Done` 增加 `error: Option<String>` 以携带原因。恢复失败不得再显示“应用失败”。
4. 后台监听线程正常结束且并非用户主动停止（例如目标应用被退出、调试端口消失）时，必须清除 `Active` 状态并提示“{应用}已关闭，主题不再生效”，避免界面继续显示“正在使用”。

### R2 恢复默认（对应 intent 问题 2）

5. **与 intent 的偏差（需确认）**：intent 期望“无已应用主题时恢复按钮禁用”。核对 `live::restore` 后改为不禁用：工具退出后注入的 CSS 仍留在官方应用页面中，重新打开工具时本进程并不知道这一点，禁用会让用户无法清理残留。改为：恢复可随时点击；进行中显示“正在恢复…”并禁用；成功显示“已恢复默认”，失败显示 R1.3 的原因（`live::restore` 对“未打开应用”“页面无响应”已有明确文案）。
6. 不新增二次确认：恢复是可逆、无数据损失的操作。

### R3 主题管理（对应 intent 问题 3）

7. 每个主题都提供“在 Finder 中显示”，调用 GPUI `App::reveal_path` 打开主题目录。
8. 仅用户安装的主题（`theme.path` 位于 `user_themes_dir()` 下）提供“删除”。内置主题不显示删除入口。若用户安装的主题覆盖了同 id 的内置主题，删除后列表回退为内置版本，确认文案说明这一点。
9. 删除为两步内联确认：点击“删除”后该位置变为“移入废纸篓？ [确认] [取消]”；切换选中主题、切换视图或按 Esc 自动取消。不使用模态对话框。
10. 删除通过 macOS 废纸篓完成（`NSFileManager trashItemAtURL`，桌面端 `cfg(target_os = "macos")`），可在废纸篓找回；移入失败时报告失败原因，**不得**回退为永久删除。非 macOS 构建返回“当前系统不支持移入废纸篓”。
11. 正在应用（`Applying`）或已应用（`Active`）的主题不可删除；“删除”置灰并说明“请先恢复默认”。
12. `skin-core` 新增 `resolve_user_theme_dir(id, installed_dir) -> Result<PathBuf, String>`：校验 id（沿用 `validate_theme_id`）、目录存在、不是符号链接、规范化后的父目录等于规范化后的 `installed_dir`，返回待移入废纸篓的路径。桌面端只对该返回值执行移入废纸篓。
13. 删除成功后重载列表并选中相邻主题；列表为空时进入 R6 的空状态。

### R4 主题商店（对应 intent 问题 4，已按核对结果缩减）

14. 核对代码后确认商店卡片、详情已经标记“已安装”，商店侧栏也有“已安装”，intent 中“不标记已安装”的描述不准确，本变更不再处理这一点。
15. 商店加载失败的错误面板内增加“重试”按钮（与顶部“刷新”调用同一入口），加载中禁用并显示“正在连接…”。
16. 新增“有更新”：商店主题版本高于本地已安装版本时，卡片、详情与侧栏将“已安装”改为“有更新”，按钮改为“更新”，复用现有安装流程。版本比较为点分数字比较，无法解析时视为无更新；`skin-core` 新增 `is_newer_version(candidate, installed) -> bool`。

### R5 键盘与可访问性（对应 intent 问题 5）

17. 搜索框改为真正的单行文本输入控件（`SearchInput`，GPUI `EntityInputHandler`），支持中文输入法组合输入、光标左右移动、Home/End、选择、复制/剪切/粘贴、点击定位光标。参考实现为所锁定 GPUI 版本（rev `c8e44cfa`）自带的 `crates/gpui/examples/input.rs`（Apache-2.0，与本项目 MIT 许可兼容），在新文件中保留版权与来源注释，并调整样式、占位符和清除按钮。
18. 搜索框之外的行为保持不变：⌘F 聚焦搜索；搜索聚焦时 ↑/↓ 切换选中、Enter 应用、Esc 退出搜索；未聚焦时方向键切换主题、Enter 应用；⌘O、⌘1、⌘2 不变。
19. 在 macOS 应用菜单栏提供这些操作的菜单项以显示快捷键：“文件 → 导入主题包… ⌘O”，“编辑 → 查找主题 ⌘F”，“视图 → 豆包 ⌘1 / 豆包工作 ⌘2”。菜单项复用现有处理函数，不新增行为。
20. 搜索框占位符为“搜索主题  ⌘F”；所有新增按钮设置 `role` 与 `aria_label`，沿用现有无障碍写法。

### R6 空状态与引导（对应 intent 问题 6）

21. 选中的目标应用未安装时，在详情区操作栏上方显示一条提示卡：“未检测到{应用}。请先安装并打开一次，再回来应用主题。”不内置下载链接（官方分发地址不由本项目维护）。应用按钮保持“尚未安装”禁用态。
22. 本地没有任何主题时，详情区显示“还没有主题”，并提供两个按钮：“浏览主题商店”（切到商店视图）与“选择主题包…”（⌘O 同一入口）。
23. 搜索无结果的文案保持现状。

## User experience

- 选中一个主题并点“应用主题”：按钮立刻变“正在应用…”，数秒内注入成功后变“正在使用”，标题旁出现“已应用”。如果豆包未运行且无法启动，则按钮回到“应用主题”，信息行以红色显示具体原因。
- 此后切到另一个主题，信息行不再残留上一条结果；切回原主题仍可看到它的状态（仅当它仍是同一目标应用下的活动主题）。
- 信息行右侧是低调的文字操作：“在 Finder 中显示”，以及（仅用户主题）“删除”。点“删除”后就地变成“移入废纸篓？确认 / 取消”。
- 商店加载失败时，错误面板中央出现“重试”；已安装但有新版本的主题显示“有更新”和“更新”按钮。
- 搜索框可以用拼音输入法输入中文、粘贴文本、用方向键移动光标；菜单栏可以看到各快捷键。
- 全新环境（无主题、无目标应用）看到的是带两个明确入口的空状态，而不是一行小字。
- 窗口仍为固定 `1120 × 720`（见 `2026-08-29-disable-window-resizing`），布局在该尺寸下设计和验收。

## Technical design

- 新增 `apps/desktop/src/status.rs`：纯逻辑、无 GPUI 依赖，包含 `Notice`、`Tone`、`NoticeScope`、`ApplyPhase` 以及“失败原因归类”函数，便于单元测试。`SkinApp` 用 `phase: ApplyPhase`（含 target、theme id、opacity）和 `notice: Option<Notice>` 取代 `applying`、`active_target`、`active_theme`、`active_surface_opacity`、`message`。现有 `theme_is_active` 的语义保留并由 `phase` 驱动。
- `Msg::Done` 增加 `error: Option<String>`；`apply_selected` / `restore_default` 的线程把 `Err(String)` 传回。监听线程是否由用户主动停止，用 `generation` 与 `stop` 标志区分。
- 新增 `apps/desktop/src/search_input.rs`：基于 GPUI 示例的 `SearchInput` 实体，对外通过 `EventEmitter<SearchEvent>` 发出 `Changed` / `Submit` / `Cancel` / `Focus`。`SkinApp` 持有 `Entity<SearchInput>` 并订阅事件，替换 `query`、`search_active` 与 `key_down` 中的手写字符拼接。上下方向、Enter、Esc 未绑定到输入控件，保持由主视图处理。
- 新增 `apps/desktop/src/trash.rs`：`move_to_trash(path) -> Result<(), String>`，macOS 下通过已有依赖 `cocoa` / `objc` 调用 `NSFileManager trashItemAtURL:resultingItemURL:error:`，其他平台返回错误。
- `crates/skin-core/src/theme.rs` 新增 `resolve_user_theme_dir`、`is_user_theme(theme, installed_dir)`、`is_newer_version`，均为纯文件系统或纯字符串逻辑，附单元测试。
- 菜单项在现有 `application_menu()` 中扩展，并为 ⌘O / ⌘F / ⌘1 / ⌘2 增加 `actions!` 动作，处理函数转发到现有方法；现有 `key_down` 里的 ⌘ 快捷键判断保留，避免菜单与按键双触发时重复执行（动作处理后 `stop_propagation`）。
- 所有新增界面沿用 `UiPalette`，不引入新颜色常量，除“错误/成功”语气复用 `danger` 与 `muted`/强调色。
- 不改动 `live.rs` 的注入逻辑，不改动主题 manifest、`doubao-skin://` 处理与商店 catalog 解析；因此无需 `pnpm --dir apps/web sync`。

## Security and privacy

- 删除操作只作用于 `resolve_user_theme_dir` 返回的路径：该函数拒绝非法 id、符号链接和位于用户主题目录之外的路径；删除走废纸篓，不做永久删除。回归测试覆盖 `../`、绝对路径、符号链接。
- “在 Finder 中显示”只打开本机主题目录，不读取内容。
- 错误文案只展示 `live` 已有的用户可读信息；底层原始错误仅写入内部日志，不展示路径以外的系统细节、不含会话内容。
- 不新增网络访问、不修改 `/Applications/Doubao*.app`，不触及协议桥接。
- 借用的 GPUI 示例代码为 Apache-2.0；在文件头保留来源说明，不引入新的第三方依赖。

## Alternatives and non-goals

- 不采用 `gpui-component` 的输入组件：已在 `~/.cargo` 缓存中存在，但会引入较大的新依赖和样式体系；使用 GPUI 自带示例即可满足需求。
- 不做永久删除、也不做应用内“撤销”：废纸篓已提供找回路径。
- 不做模态对话框：GPUI 当前无现成模态抽象，内联确认足够。
- 不禁用“恢复默认”（见 R2.5）。
- 不处理商店「已安装」标记（已存在）、不做商店骨架屏（已有加载态文案）。
- 不新增 toast 系统；信息行 + 作用域已能满足需要。
- 不做 Web 画廊、窗口尺寸、视觉风格重做。

## Areas of concern

- **输入控件移植风险最高**：焦点（搜索框与根视图各自 `FocusHandle`）、⌘F 聚焦、方向键冒泡、IME 候选框位置（`bounds_for_range`）都要在真实窗口里验证中文输入法。若在计划内无法稳定，降级方案：保留可聚焦的输入控件但暂不承诺组合输入，并在 `verification.md` 记录偏差，不静默放弃。
- 状态机重构触及 `apply_selected`、`restore_default`、`handle_msg` 与多处渲染，是回归风险点；需在重构前先补纯逻辑测试，并在真实应用里验证“应用 → 切换主题 → 恢复”“应用中途切换主题/目标应用”“应用后退出豆包”三条路径。
- NSFileManager 调用经 `objc` 手写，需在真实环境验证废纸篓里能找到被删主题，且失败路径（只读目录）不会永久删除。
- 未安装豆包或豆包工作的环境难以直接复现；可用 `DOUBAO_SKIN_USER_THEMES_DIR` 与测试注入覆盖主题相关场景，目标应用缺失场景用纯函数测试 + 真实窗口手动验证。
- 版本比较只做数字点分格式；商店返回非规范版本字符串时保守地不提示更新。

## Acceptance criteria

1. 单元/回归测试（先写失败测试，再实现）：
   - `status`：作用域不匹配的 `Notice` 不显示；`ApplyPhase` 在 Applied 前不是 `Active`；应用失败回到 `Idle`；监听线程自然结束时清除 `Active`；失败原因归类（中文透传、英文归为连接失败）。
   - `skin-core`：`resolve_user_theme_dir` 拒绝 `../x`、绝对路径、符号链接、不存在的目录，接受合法用户主题；`is_user_theme` 区分内置与用户主题；`is_newer_version` 覆盖 `1.2.0 > 1.1.9`、`1.10.0 > 1.9.0`、相等、不可解析。
   - `search_input`：文本插入/删除按字符边界、UTF-8 中文、光标移动、粘贴、选择替换（可在不依赖窗口的纯文本缓冲层测试）。
   - `trash`：对临时目录移入废纸篓后原路径消失（macOS 条件测试），对不存在路径返回错误且不删除其他内容。
2. `./scripts/check.sh rust` 与 `./scripts/check.sh workflow` 通过；`cargo test --workspace --locked` 无新增失败。
3. 真实窗口（`cargo run -p doubao-skin-desktop`，默认 `1120 × 720`）逐项验证并截图：
   - 应用成功/失败（关闭豆包且让其无法启动，或占用端口）的按钮与信息行；
   - 切换主题后旧消息消失；恢复默认成功与失败文案；
   - 应用后退出豆包，界面退出“正在使用”并提示；
   - 删除用户主题（内联确认 → 废纸篓中可找回）；内置主题没有删除；“在 Finder 中显示”可用；
   - 商店断网时的错误面板与“重试”；构造较低本地版本时的“有更新/更新”；
   - 用拼音输入法输入中文、粘贴、方向键移动光标；菜单栏快捷键可见且可用；
   - 空主题目录下的空状态两个入口；目标应用缺失时的提示卡。
4. 紧凑/矮布局代码路径仍通过 `cargo test -p doubao-skin-desktop` 中现有布局测试；因窗口固定尺寸，不要求窄窗口实窗验证，但在 `verification.md` 中说明。
5. `verification.md` 记录所有命令、结果、截图证据、与本规格的偏差和剩余风险；由全新上下文的验证者或人类给出最终结论。

## Revision: rebased onto the restructured `main`

PR #42 无法合并：`main` 在本变更之后把 `apps/desktop/src/main.rs` 拆成 `app/`、`ui/`、`store/`、`preview/` 模块，新增 `i18n.rs`、Windows / WorkBuddy 支持、按目标应用的会话状态机 `ThemeSessions`。本变更以最新 `main` 为基础在新模块结构上重做，行为需求 R1–R6 不变，但实现与下列几点随之调整（均不扩大范围）：

- **R1 状态模型**：`main` 的 `ThemeSessions`（`Applying` / `Active` / `Restoring`，带 generation，按目标应用隔离）已经满足 R1.2、R1.4 以及“应用过程分阶段”的要求，因此**不再新增** `ApplyPhase` / `Notice` / `NoticeScope`；沿用现有 `message` 字符串和“失败”关键字着色。仍然新增：`Msg::Done.error` 携带失败原因、`describe_failure`（中文透传 / 其他归为连接失败）、恢复失败不再显示“应用失败”。作用域规则由“切换主题 / 目标应用即清空 `message`”保证（现有行为）。
- **R3 删除**：删除入口只在 macOS 显示（`trash::SUPPORTED`）。Windows 构建没有等价的、可找回的废纸篓实现，宁可不提供删除，也不做永久删除。“在 Finder 中显示”在 Windows 显示为“在资源管理器中显示”。“正在使用”判断改为任何目标应用上 `Applying` / `Active` 的该主题（`ThemeSessions::uses_theme`），比只看当前目标更保守。
- **R5 快捷键**：⌘O / ⌘F / ⌘1 / ⌘2 / ⌘3 改为 `secondary-` 键绑定与动作（macOS 为 ⌘，Windows 为 Ctrl，与 `target_shortcut` 展示一致），原先 `key_down` 中基于 `modifiers.platform` 的手写判断移除；About 弹窗打开时这些动作不生效。菜单栏仍只在 macOS 设置。新增“视图 → WorkBuddy ⌘3”。占位符按平台显示 `⌘F` / `Ctrl+F`。所有新增文案进入 `i18n.rs`。
- **R6 提示卡**：目标应用缺失 / 主题不兼容时，沿用 `main` 已有的信息行文案（“请先安装…”），改为危险色强调，不再新增独立提示卡。空主题库的两个入口按原规格实现。
- **窗口尺寸**：窗口仍固定 `1120 × 720`；紧凑布局分支保留，仅由现有布局测试覆盖。

## Decision

等待产品负责人确认本规格后进入实施计划。需要重点确认的偏差：(a) R2.5 恢复按钮不禁用；(b) R4 因商店已具备“已安装”标记，改为“重试 + 有更新”；(c) 输入法支持采用 GPUI 自带示例移植，并设有降级方案。
