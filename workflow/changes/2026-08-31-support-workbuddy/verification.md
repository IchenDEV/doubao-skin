---
id: "2026-08-31-support-workbuddy"
stage: verification
status: pending
owner: "codex"
created: "2026-08-31"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: 支持 WorkBuddy 实时主题

## Automated checks

- `./scripts/devflow validate` — passed；15 组 artifact 状态有效（2026-08-31，Build 前预检）。
- `cargo test -p skin-core workbuddy --no-fail-fast` — 按预期失败（exit 101）；编译器只报告缺少 `TargetApp::WorkBuddy`、`PrepareState`、准备状态函数和 `Theme::supports_target` 等新契约，构成失败优先证据。
- `cargo test -p skin-core workbuddy --no-fail-fast` — 实现后 passed；3 项 WorkBuddy 窄测试通过。
- `cargo test -p skin-core --no-fail-fast` — passed；27 项核心、7 项 authoring、1 项 bundled path、4 项 CLI 测试通过。
- `cargo test -p doubao-skin-desktop ui_regression_tests --no-fail-fast` — passed；9 项桌面状态测试通过。
- `cargo fmt --all -- --check` — passed。
- `./scripts/check.sh rust` — passed；测试、示例编译和 clippy 全部通过，仅保留既有 `block v0.1.6` future-incompatibility 提示。
- `./scripts/check.sh workflow` — passed；15 组 artifact 与审批策略测试通过。
- `git diff --check` — passed。
- `./work/check-workbuddy-visual-delta.sh` — failed as intended after user visual review；在同一 600×150 输入区裁剪中，超过 10% 色差的像素只有 5.58%，低于本次诊断使用的 20% 明显变化阈值。连续三次结果一致，证明“主题肉眼几乎无变化”可稳定复现。
- `cargo test -p skin-core workbuddy_background_theme_exposes_artwork_and_surface_opacity -- --nocapture` — 修复前按预期失败（exit 101），缺少 WorkBuddy 背景宿主透明规则、表面 opacity 映射和正确 backdrop blur。
- `cargo test -p skin-core workbuddy --no-fail-fast` — 修复后 passed；4 项 WorkBuddy 窄测试通过，包括鲸鱼娘背景/透明度回归测试。
- `./work/check-workbuddy-whale-visual-delta.sh` — passed；同一空白窗口修复前后有 86.85% 像素出现超过 10% 的色差，高于本次诊断使用的 20% 明显变化阈值。
- `node work/check-workbuddy-surface-semantics.js` — 用户反馈后的失败优先实窗回路连续复现：类型轨道 alpha 1、未选项 0.443、输入外壳 0.76、内部编辑层 1、快捷按钮 0.443，因层级角色错误返回 RED；正式修复后返回 GREEN。
- `cargo test -p skin-core workbuddy_surface_roles_avoid_nested_panels_and_weak_controls -- --nocapture` — 新回归测试在实现前按预期失败（exit 101，缺少 `--wb-skin-control-track`）；实现后 passed。
- `cargo test -p skin-core workbuddy --no-fail-fast` — 最新实现 passed；5 项 WorkBuddy 测试通过。
- `./work/check-workbuddy-surface-visual-delta.sh` — passed；在同一 560×300 控件区域裁剪中，修复前后有 9.33% 像素出现超过 5% 的色差，高于本次控件层级修复的 8% 阈值。
- `./scripts/check.sh rust` — 最新实现 passed；桌面 9 项、skin-core 29 项、authoring 7 项、bundled path 1 项、CLI 4 项及 examples/clippy 全部通过，仅保留既有 `block v0.1.6` future-incompatibility 提示。
- `node work/check-workbuddy-sidebar-semantics.js` — 用户指出侧栏混乱后，失败优先实窗回路复现了四层背景叠加：sidebar/list/header/content 均为 alpha 0.51，未选导航为 0.443、选中导航为不透明棕色，分区标题仍是官方不透明灰色，因层级角色错误返回 RED。正式修复、重新构建并应用后返回 GREEN：只有 sidebar shell 为 0.706，list/header/content、未选导航、任务行和 ghost button 均为 0；选中导航为 0.87，分区标题为 0.553，任务标题使用主题正文色；等待 150ms 过渡完成后，未选导航 hover 为 0.553。
- `cargo test -p skin-core workbuddy_sidebar_uses_one_shell_and_transparent_navigation_rows -- --nocapture` — 新回归测试在实现前按预期失败（exit 101，缺少 `--wb-skin-sidebar-shell`）；实现后 passed。
- `cargo test -p skin-core workbuddy --no-fail-fast` — 侧栏修复后 passed；6 项 WorkBuddy 测试通过。
- `./scripts/check.sh rust` — 侧栏正式修复后的最终 Gate passed；桌面 9 项、skin-core 30 项、authoring 7 项、bundled path 1 项、CLI 4 项及 examples/clippy 全部通过，仅保留既有 `block v0.1.6` future-incompatibility 提示。
- `./scripts/check.sh workflow` — 侧栏验收记录更新后 passed；15 组 artifact 与审批策略测试通过。
- `git diff --check` — 最终 passed。
- `pnpm --dir apps/web sync` — completed；主题源文件本次没有变化，生成器只改动生成时间和 30 个 ZIP 的非确定性哈希，因此精确恢复这批生成产物，未把无关目录噪声留在变更中。
- `cargo test -p skin-core workbuddy_sidebar_uses_airy_theme_driven_visual_rhythm -- --nocapture` — 用户认为第一版侧栏仍不美观后，新视觉节奏回归测试先按预期失败（exit 101，缺少主题驱动 sidebar accent、开放分区、选中标记和 promo card 收束规则）；实现后 passed。
- `cargo test -p skin-core workbuddy --no-fail-fast` — 第二轮侧栏视觉重构后 passed；7 项 WorkBuddy 测试通过。
- `node work/check-workbuddy-sidebar-semantics.js` — 正式构建、签名并重新应用后 GREEN：严格 WorkBuddy renderer URL、title、complete readyState、4 个有效 landmark 和无 framework overlay 均通过；sidebar shell alpha 0.635294，内部 list/header/content、普通行、分区标题和分组标题均为透明；选中导航 alpha 0.87 并有不透明主题色圆点，hover alpha 0.12，分区渐隐线和分组细引导轨均存在，临时 prototype/privacy style 均不存在。启用 CDP Log/Runtime 时 WorkBuddy 回放了 1 个既有历史 exception/error 事件；清空该基线后，目标 hover 交互期间没有新增 exception 或 error log。
- `./scripts/check.sh rust` — 第二轮侧栏视觉重构最终 Gate passed；桌面 9 项、skin-core 31 项、authoring 7 项、bundled path 1 项、CLI 4 项及 examples/clippy 全部通过，仅保留既有 `block v0.1.6` future-incompatibility 提示。
- `./scripts/check.sh workflow` — 第二轮视觉证据更新后 passed；15 组 artifact 与审批策略测试通过。
- `git diff --check` — 第二轮最终 passed。
- `cargo test -p skin-core workbuddy_chrome_uses_soft_edges_instead_of_hard_outlines -- --nocapture` — 用户指出边框过重后，新回归测试先按预期失败（exit 101，sidebar divider、选中导航、状态点、分组轨、类型轨道和快捷按钮仍使用旧权重）；实现后 passed。
- `cargo test -p skin-core workbuddy --no-fail-fast` — 低边框修复后 passed；8 项 WorkBuddy 测试通过。
- `node work/check-workbuddy-sidebar-semantics.js` — 正式低边框构建实窗 GREEN：侧栏右分隔线 alpha 0.08，选中导航只保留 alpha 0.07 的外投影且不存在 inset outline，分组轨为 1px/0.10，hover/选中/正文层级继续通过；测试交互期间无新增 exception 或 error log。
- `node work/check-workbuddy-surface-semantics.js` — 正式低边框构建实窗 GREEN：类型轨道边框 alpha 0.12、选中项 outline alpha 0.35、快捷按钮边框 alpha 0.14、输入框边框 alpha 0.18；选中项和快捷按钮文字对比度仍分别为 7.12:1 与 12.33:1。
- `./scripts/check.sh rust` — 低边框正式修复最终 Gate passed；桌面 9 项、skin-core 32 项、authoring 7 项、bundled path 1 项、CLI 4 项及 examples/clippy 全部通过，仅保留既有 `block v0.1.6` future-incompatibility 提示。
- `./scripts/check.sh workflow` — 低边框验收记录更新后 passed；15 组 artifact 与审批策略测试通过。
- `git diff --check` — 低边框最终 passed。
- 代码评审复现了两个多目标状态缺口：恢复线程未登记导致同目标 restore/run 可并发，以及另一个目标推进全局 generation 后 WorkBuddy 退出提示被吞。失败优先桌面测试在缺少 `begin_applying`、`begin_restoring`、`mark_applied` 和按目标 completion API 时编译失败；修复后 `cargo test -p doubao-skin-desktop ui_regression_tests --no-fail-fast` 15/15 passed。桌面现在按目标持有 `Applying / Active / Restoring` 状态，同目标操作串行、其他目标不受阻塞，WorkBuddy 完成消息只比较自己的 generation。
- `./scripts/check.sh all` — 本次状态修复后 passed；workflow 18 组、桌面 15 项、skin-core 48 项、authoring 9 项、bundled path 3 项、CLI 4 项、v3 契约 12 项、Schema 2 项、examples/clippy、Web 16 项、TypeScript、42 页 Next build 和依赖审计均通过。既有 `block v0.1.6` future-incompatibility 提示保持不变。

## Behavioral evidence

- `/Applications/WorkBuddy.app` 已安装，`CFBundleShortVersionString=5.3.14`、`CFBundleIdentifier=com.workbuddy.workbuddy`、`CFBundleExecutable=Electron`。
- 只读进程检查确认 WorkBuddy 当前正在运行；未输出完整命令行。
- 用户在执行当下明确回复“开，开始执行。”，授权本次优雅退出和 CDP 重启探针。
- WorkBuddy 通过 `Command-Q` 优雅退出；轮询确认主进程正常消失，没有使用强制结束、宽泛 `pkill Electron` 或端口占用进程终止。
- 通过 LaunchServices 以 `--remote-debugging-address=127.0.0.1 --remote-debugging-port=9224` 重启；`lsof` 确认只有 `127.0.0.1:9224` 在监听。
- `/json` 返回一个主 `page`，URL 严格为 `file:///Applications/WorkBuddy.app/Contents/Resources/app.asar/renderer/index.html`；另有一个 `https://www.workbuddy.cn/space/home?...` 的 `iframe`，后者不属于允许注入的 page target。
- 无内容 DOM 探针确认主 renderer 存在稳定宿主 landmark：`.teams-container`、`.conversation-list`、`.main-content`、`.chat-container`、`.wb-home-composer`、`.wb-button`；未读取元素正文。
- 当前实现增加独立 `9224` 目标、严格 renderer URL、准备状态、二次重启授权、端口丢失不拉起、v2 adapter、三目标偏好与 `Command-3`；协议桥没有新增 WorkBuddy 引用。
- WorkBuddy 正确 CDP 实例上实测 `pure-dark`：`data-skin=pure-dark`、`data-skin-target=workbuddy`、`data-theme=dark`、style length 7992，主背景 `rgba(18,20,25,0.88)`，composer `rgba(29,31,37,0.96)`。
- 同一空白任务实测 `qq-light-blue`：`data-skin=qq-light-blue`、`data-theme=light`、style length 8072，主背景 `rgba(242,247,253,0.70)`，composer `rgba(255,255,255,0.97)`。
- 恢复默认后只读探针得到 `skin=null`、`target=null`、style length 0、无 WorkBuddy adapter 变量；官方 bundle 未被修改。
- 应用主题后从 WorkBuddy 执行正常 `Command-Q`，连续 12 秒精确主进程计数和 `9224` 监听数均为 0；watcher 没有拉起应用，桌面工具回到“应用主题”并显示监听已停止。
- WorkBuddy 普通启动且 `9224` 未监听时，首次“应用主题”只把按钮切换为“重启 WorkBuddy 并应用”；复核仍有 WorkBuddy 进程且监听数为 0，证明第一次操作没有退出或重启。
- 用户随后明确授权“你直接帮我验收吧，我现在 WorkBuddy 你可以随便用”，满足第二次重启的执行当下确认。点击“重启 WorkBuddy 并应用”后，桌面工具进入“正在使用”；`lsof` 再次确认只有 `127.0.0.1:9224` 监听，且 `/json` 仍只有严格主 renderer `page` 与一个不允许注入的远程 `iframe`。
- 第二次重启后的只读页面探针确认：主 `page` 有 `data-skin=qq-light-blue`、`data-skin-target=workbuddy`、style length 8072，输入区计算背景为 `rgba(255,255,255,0.97)`；远程 `iframe` 的 skin/target/style 均为空。
- 在已应用主题时从“日常办公”切换到“代码开发”，主 `page` 的 skin/target/style 仍保持不变，证明内部导航不会丢失注入；没有输入或发送内容。
- 再次应用 `pure-dark` 后，结构化 adapter 仍为 style length 7992；当前 WorkBuddy/系统外观为 light，因此选择其 light variant。此前 dark appearance 实窗探针已确认同一主题的 dark variant 为主背景 `rgba(18,20,25,0.88)`、composer `rgba(29,31,37,0.96)`。
- 最终点击“恢复默认”后，主 `page` 的 skin/target/style 均清空，官方主内容背景恢复为 `rgb(255,255,255)`；随后优雅退出本次 CDP 实例并通过 LaunchServices 普通重开，WorkBuddy 主进程存在而 `9224` 无监听。临时 QA app 已关闭，原 `/Applications/豆皮.app` 已恢复普通运行。
- 用户要求尝试现有“鲸鱼娘”（`gallery-whale-maid`）主题。修复前实测 backdrop 已连接、背景为 212510 字符 data image，但 `.teams-content-wrapper` 的直接父宿主仍为 `rgb(255,255,255)`，完整遮住背景。
- 通用 adapter 修复后不依赖主题 ID 或 CSS module hash：使用 `.teams-container :has(> .teams-content-wrapper)` 清除该白色宿主层；html/body 透明，单层 `.teams-container` 承担主色；背景采用 `BackgroundSpec.blur` 而不是误用 `effects.blur`；`surface_opacity` 分别映射 page/sidebar/layer/input。
- 鲸鱼娘 68% 实窗探针：`data-skin=gallery-whale-maid`、`data-skin-target=workbuddy`、style length 8313、backdrop 为 data image、主层 `rgba(189,153,153,0.373)`、输入区 `rgba(255,255,255,0.76)`。临时 inline 原型已清除，白色宿主层仍由正式 adapter 计算为 transparent。
- 将透明度从 68% 调为 40% 并重新应用后，主层从 0.373 变为 0.22，输入区从 0.76 变为 0.48；证明桌面滑块已真实控制 WorkBuddy。验证后恢复为 68%。
- 用户继续指出鲸鱼娘存在“该透明的不透明、该不透明的很透明”，尤其中间三个类型切换和部分配色不清。无正文 DOM 探针证明主因不是文字颜色本身：旧类型轨道仍是 WorkBuddy 官方不透明灰色，未选项又叠加主题半透明白色；输入区同时叠加 0.76 外壳、0.76 input slot 和 1.0 白色编辑层；快捷能力按钮只有 0.443 表面。
- 新通用表面角色将类型切换改为单一 0.553 磨砂轨道、透明未选项、0.87 强选中面；选中项使用主题 accent 文字/描边，实测 7.12:1，对其余控件文字采用主题正文色。输入区外壳为 transparent，只保留 0.87 input slot 磨砂面；快捷能力按钮为 0.733，文字对比度 12.33:1。表面 blur 取主题 `effects.blur`，不再和背景图 blur 混用。
- 从“日常办公”切换到“代码开发”后重新执行表面语义探针仍为 GREEN，随后恢复原选择；未输入或发送内容。临时 prototype style 已删除，最终数据来自正式 adapter 和重新构建、签名的临时 QA app。
- 用户继续指出展开侧栏仍然混乱。实窗计算样式确认根因是同一半透明白色同时应用于 sidebar、list、header、content，形成约 88% 的有效叠底；导航行、分区条、任务行和 ghost button 又各自使用不一致的官方/主题表面。
- 侧栏现在改为单一 0.706 磨砂 shell；内部 list/header/content、普通导航、任务行和 ghost button 静止时透明；当前导航使用 0.87 强表面、主题 accent 文字和左侧 accent 标记；任务/分组标题统一为 0.553 控制轨道，任务正文恢复主题文字色。悬停态在过渡完成后使用同一 0.553 轨道。最终 renderer 未残留 prototype 或 privacy-mask style。
- 用户认为第一版虽规整但仍不美观。第二轮没有继续堆透明面板，而是从主题正文色与 accent 动态混合出 sidebar accent：shell 降为约 0.635 并加入极弱双径向色雾；选中项改为 34px 紫灰玻璃胶囊、细内描边、柔和投影和 5px 主题色状态点；普通图标统一为主题正文色的 72%；任务/空间分区改成开放文字加渐隐分隔线；可折叠分组取消白色框，只保留 2px 低对比引导轨；任务行统一 30px 节奏和 9px hover 区域。
- WorkBuddy shadow DOM 内的 `daily-checkin` 内容没有被侵入修改；只在宿主层使用 0.88 opacity、12px 圆角、裁切和轻投影，把官方活动卡收束成独立 promo card，同时保留其内容与交互。
- 用户认可第二轮方向但指出边框仍过重。第三轮保持布局和透明层级不变，只降低 chrome edge weight：侧栏分隔线 12%→8%，选中导航移除 13% inset outline、投影降为 7%，状态点光环 3px/10%→2px/6%，分区线 16%→10%，分组引导轨 2px/18%→1px/10%；类型轨道 22%→12%，选中类型 outline 100%→35%，快捷按钮 30%→14%，输入框统一为 18%。

## Visual evidence

- 早期只读界面勘察确认当前 WorkBuddy 是带左侧导航、主内容区和输入区的桌面主界面；为避免把现有任务列表写入仓库，本阶段未保存视觉证据。
- 当前构建的临时 QA app 实窗显示三段目标选择器，`Command-3` 可选中 WorkBuddy，VoiceOver/AX 标签分别为 `Command-1`、`Command-2`、`Command-3`；实验提示显示“已验证 WorkBuddy 5.3.14”。
- 深色 `pure-dark`、浅色 `qq-light-blue` 和恢复默认均在 WorkBuddy 新建空白任务、收起任务侧栏后检查；布局、可读性和 iframe 隔离正常，但用户复核指出浅色主题肉眼几乎无变化。重新并排检查确认，当前变化主要限于蓝色边框、阴影和选中态，不能满足“主题效果明显”的产品验收标准。内部导航持续注入只证明运行时稳定，不证明视觉主题成立。
- 本地忽略目录证据：`work/verification/2026-08-31-support-workbuddy/desktop-three-targets.jpeg`、`desktop-workbuddy-selected.jpeg`、`desktop-restart-gate.jpeg`、`desktop-after-user-quit.jpeg`、`desktop-after-confirmed-restart.jpeg`、`workbuddy-qq-light-blue-after-confirmed-restart.jpeg`、`workbuddy-code-navigation-themed.jpeg`、`workbuddy-restored-official.jpeg`、`workbuddy-normal-reopen-official.jpeg`。深色实窗已在 Computer Use 中即时检查，但早期系统临时截图在复制前被清理，没有进入仓库。
- WorkBuddy 窄窗口实窗和豆皮 720 px 实窗尚未完成：Computer Use 的边缘拖拽没有改变窗口，当前豆皮窗口合同仍固定为 1120×720；不得把纯函数 compact 测试当作实窗证据。
- 诊断对照图位于 `work/verification/2026-08-31-support-workbuddy/diagnostic-official-vs-themed.jpeg`；左侧官方、右侧 `qq-light-blue`，两者主区域大面积保持白色。
- 鲸鱼娘修复前后证据：`work/verification/2026-08-31-support-workbuddy/workbuddy-whale-before-fix.jpeg`、`workbuddy-whale-after-fix.jpeg`。修复后背景清晰覆盖完整窗口，主标题、场景标签、输入区和 macOS 窗口区域保持可读且可点击。
- 表面层级修复后的安全实窗证据：`work/verification/2026-08-31-support-workbuddy/workbuddy-whale-surface-hierarchy-fixed.jpeg`；近景前后对照为 `workbuddy-whale-surface-before-after-closeup.jpeg`。最终单图已收起侧栏并关闭产品活动浮层，不包含用户任务或账号信息；选中类型、快捷能力按钮和单一编辑面在人物背景上边界清晰。
- 侧栏前后安全对照为 `work/verification/2026-08-31-support-workbuddy/workbuddy-whale-sidebar-before-after.jpeg`，正式修复单图为 `workbuddy-whale-sidebar-fixed.jpeg`。取证前通过仅限当前 renderer 的临时 privacy mask 将任务、分组和用户名称替换为“示例”文本并隐藏时间/头像，保存后立即移除；正式运行时只保留 adapter CSS。相同 264×768 侧栏裁剪的 RMSE 为 0.102582，且肉眼可见旧版的多层白块、实心棕色导航和灰色分区条已被单层 shell 与透明列表层替代。
- 第二轮视觉概念为 `workbuddy-whale-sidebar-polish-concept.png`，正式构建为 `workbuddy-whale-sidebar-polished.png`，上一版与第二版对照为 `workbuddy-whale-sidebar-aesthetic-before-after.png`；三张均为结构级脱敏证据。概念与正式图在同一 1186×768 实窗逐项比对：导航几何、主题色状态、分区线、开放分组、任务行距、promo card 和主画面融合一致；上一版与第二版相同 264×768 侧栏裁剪 RMSE 为 0.0942724。
- 低边框正式构建为 `workbuddy-whale-soft-edges.png`，第二轮与低边框对照为 `workbuddy-whale-edge-weight-before-after.png`；均在保存前通过结构级 privacy mask 验证并在保存后移除。相同 1186×500 首屏裁剪 RMSE 为 0.0133228，变化集中在边界/阴影而非布局、背景或文字，符合本次“减轻边框而不重做结构”的范围。

## Security and privacy evidence

- `codesign` 只读检查确认 app identifier 为 `com.workbuddy.workbuddy`、TeamIdentifier 为 `FN2V63AD2J`、hardened runtime 版本为 `15.4.0`；未修改 bundle 或签名。
- 预检只读取 Info.plist、签名摘要、精确路径匹配得到的 PID 和 `9224` 监听状态；未读取任务正文、账号、Cookie、localStorage、IndexedDB、工作空间、文件、插件、MCP 或网络内容。
- 没有执行宽泛 `pkill Electron`、端口占用进程终止、app.asar 解包/修改或远程调试安全绕过。
- 实际注入目标只有严格匹配的 `page`；先前探针出现的 `https://www.workbuddy.cn/space/home?...` 类型为 `iframe`，而运行循环同时要求 `type=page` 与严格 URL，单元测试也拒绝该 URL、普通 file、DevTools 和 extension 页面。
- 第二次重启后的页面级探针再次证明隔离：严格主 `page` 有工具 marker/style，远程 `iframe` 的 marker/style 均为空；没有读取正文、Cookie、存储或网络数据。
- WorkBuddy adapter 不包含主题原始 CSS sentinel，并以 `if(TARGET!=='workbuddy')markIcons();` 禁止 WorkBuddy 图标标记；只使用 v2 结构字段和工具自有 style/backdrop/attribute runtime。
- 侧栏视觉证据未保存真实任务、分组或账号名称；语义探针只读取计算样式、几何位置和主题 marker，不读取元素正文。最终探针确认临时 prototype/privacy-mask style 均不存在。
- 第二轮首次 Computer Use 预览时，privacy mask 的文字 `color` 规则被正式 adapter 更高 specificity 的 `!important` 覆盖，工具输出短暂显示现有任务标题；该截图未保存到仓库，没有打开任务或读取正文。随后停止取证，将遮罩提升为结构级 `visibility:hidden!important`，并在每次截图前用 CDP 验证所有 task/group/footer 文本节点均为 hidden；最终保存的概念、正式图和对照图只含“示例”文字。此事件计入第五次侧栏/AX 取证隐私偏差，需独立验证者知情复核。

## Deviations and residual risk

- 非干扰的第二实例探针没有执行：没有证据证明第二实例会使用隔离用户目录而不触碰现有单实例会话，用户已改为明确授权安全重启。
- 运行时可行性 Gate 已通过：参数有效、监听仅限回环、主 renderer URL 符合 Spec，且不需要改包或安全绕过。
- 四次 AX/取证状态短暂包含现有任务标题：一次来自早期定位“新建任务”入口时过滤范围过宽，一次来自窄窗口尝试中右键误开输入区上下文菜单并使侧栏重新展开，一次来自鲸鱼娘修复前基线启动时侧栏状态被 WorkBuddy 恢复，最新一次来自正式表面修复截图时 WorkBuddy 再次恢复展开侧栏。最后一张临时截图已立即从本地证据目录删除并确认不存在，随后收起侧栏重新保存安全截图；没有打开任务、读取正文或提交相关截图。该偏差需要独立验证者知情复核。
- 第二次重启的执行当下确认、明确二次动作、成功应用、恢复和普通重开均已完成。
- 视觉差异不足的根因已修复：背景 data image 原本被 WorkBuddy 白色网格宿主遮住，`effects.blur=18` 又被错误用于整张背景，且 `surface_opacity` 未进入 adapter。三项均已改为通用宿主规则/结构字段映射，不需要鲸鱼娘专属 CSS 或独立主题文件。
- 用户复核指出的表面割裂已修复：类型轨道、未选项、选中项、快捷能力按钮、输入外壳和编辑面现在使用六个明确角色；结构选择器来自 WorkBuddy 5.3.14 无正文 DOM 探针，不依赖哈希类或主题 ID。实窗语义回路和控件区域视觉差异检查均通过。
- 用户复核指出的侧栏混乱已修复：背景只存在于一个稳定 shell，导航、分区、任务和 footer 按交互角色分层；选择器基于 WorkBuddy 5.3.14 稳定语义类，不依赖任务内容、哈希类或鲸鱼娘主题 ID。实窗静止/hover 语义回路和隐私脱敏前后截图均通过。
- 用户第二次复核指出的“仍不大美观”已按开放列表方向重构；视觉原型先在真实 renderer 通过，再以主题驱动变量固化到通用 adapter。没有新增鲸鱼娘专属分支、图片资产或独立主题，且正式渲染与原型在七个具体视觉点一致。
- 用户第三次复核指出的边框过重已修复：侧栏、中间类型切换、快捷按钮和输入区使用一组分级的 8%–35% 软边界；主状态仍由填充、文字、状态点和透明度承担。实窗静止、hover、选中和对比度语义回路均通过。
- 当前剩余 Gate：WorkBuddy 窄窗口实窗、固定尺寸豆皮窗口与 Plan 中“720 px 宽度”冲突的处置，以及 fresh-context/human 最终 verdict。Computer Use 的窗口边缘拖动和 WorkBuddy 的原生 Zoom 均未产生可证明的窄窗口尺寸，因此没有把正常宽度截图冒充窄窗口证据。

## Verdict

Pending。用户指定的鲸鱼娘主题已在正常宽度实窗通过背景、主内容表面、低边框侧栏视觉层级、控件对比度和透明度 Gate，生命周期、安全隔离和自动检查继续通过；仍需补充修复后的深色主题、窄窗口实窗以及 fresh-context/human 最终验证，才能把整个 WorkBuddy 变更标记为 passed。
