---
id: "2026-08-31-support-workbuddy"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-31"
based_on: spec.md
risk: "high"
approved_by: "product-risk-owner"
approved_at: "2026-08-31"
---

# Plan: 支持 WorkBuddy 实时主题

## Files and ownership

- `crates/skin-core/src/live.rs`：实现者负责 WorkBuddy 目标元数据、严格页面归属、准备状态、重启授权边界、watcher 端口丢失策略、恢复路径及同文件回归测试。
- `crates/skin-core/src/theme.rs`：实现者负责 v2 主题目标能力判断、WorkBuddy adapter CSS、禁止原始豆包 CSS/图标注入及同文件回归测试。
- `apps/desktop/src/main.rs`：实现者负责三目标偏好与选择器、`Command-3`、实验/版本提示、主题兼容状态、重启二次动作、active target 隔离及同文件 UI 状态测试。
- `docs/architecture.md`、`README.md`、`README.en.md`、`CHANGELOG.md`：仅在真实探针和双主题实窗验证通过后，记录 WorkBuddy 的实验支持、已验证版本、重启影响和不支持范围；不得提前宣传已完成。
- `workflow/changes/2026-08-31-support-workbuddy/verification.md`：实现者记录命令和证据；最终 `passed` verdict 由 fresh-context verifier 或人类验证者填写。
- 上述实现文件相互依赖，保持顺序修改，不拆成并行工作；不得改动 `protocol_bridge.rs`、WorkBuddy app bundle、主题包内容或生成的 web catalog。

## Order of work

1. **当前状态与可回滚探针。** 重新确认 Git 状态、WorkBuddy 版本/签名、`9224` 监听状态和精确主进程身份，不输出完整进程命令行。优先尝试不影响现有实例的独立启动探针；如果 WorkBuddy 的单实例或用户目录策略会触碰当前实例，立即停止。只有用户在执行当下明确确认没有需要保留的运行中任务后，才可优雅退出当前 WorkBuddy、用 `--remote-debugging-port=9224` 重启，并只读取 `/json` 的 target type/URL 与无内容的 DOM 结构/计算样式。探针结束后恢复普通启动。**Gate：** 端口只监听 `127.0.0.1`、至少一个主 renderer URL 严格匹配 Spec，且不需要改包或安全绕过；否则将 Plan 标记 blocked/revised，不进入 Build。
2. **先锁定核心失败用例。** 在 `live.rs` 增加失败优先测试：三目标元数据/端口互异、WorkBuddy URL 规范化与拒绝普通 file/webview、错误端口所有者、`RestartConfirmationRequired`、`relaunch_after_port_loss=false`、精确进程模式和恢复边界。在 `theme.rs` 增加 v2 支持/v1 拒绝、target scope、无原始 CSS、无 icon runtime 的测试；在 `main.rs` 增加三目标默认/偏好/快捷键语义和 active target 隔离测试。**Gate：** 新增测试在缺少实现时以预期原因失败，既有测试仍通过。
3. **实现最小 WorkBuddy 目标生命周期。** 只扩展现有 `TargetApp` 与 watcher：增加 WorkBuddy 常量和策略、严格身份函数、可测试的准备结果，并把实际退出/重启放到显式授权后的路径。端口已正确开放时直接复用；端口冲突不杀进程；用户主动关闭 WorkBuddy 后 watcher 正常结束且不拉起。豆包两目标沿用既有自动准备和端口丢失行为。**Gate：** `live.rs` 新旧测试通过，搜索确认没有宽泛 `pkill Electron` 或 WorkBuddy 协议桥引用。
4. **实现 v2 WorkBuddy adapter。** 按目标生成注入 CSS；WorkBuddy 只从现有结构化 v2 字段映射基础颜色、字体、间距、输入区、代码、弹层、滚动条和背景层，所有规则以 `html[data-skin][data-skin-target=workbuddy]` 为根。探针发现的选择器只用于主 renderer 宿主层，避开 iframe/webview/文档画布；bootstrap 对 WorkBuddy 跳过图标标记。豆包继续使用当前原始主题 CSS 路径。**Gate：** 字符串/结构测试证明 WorkBuddy 脚本不含主题原始 CSS sentinel、图标 data URI 或执行中的 `markIcons()`，恢复脚本仍只触碰工具自有状态。
5. **接入桌面端三目标体验。** 扩展安装检测、保存偏好、三段选择器、`Command-3` 和 WorkBuddy 预览身份；加入实验/版本提示、v1 不兼容状态和 WorkBuddy 重启二次动作。调用核心准备结果驱动 UI，不在界面复制进程/端口判断。正常与 720 px 布局都必须保留选择器、说明和主操作。**Gate：** 桌面状态测试及 Rust 小门禁通过，未安装/冲突/需重启/已就绪四类结果都有用户可执行文案。
6. **真实应用闭环。** 构建并运行桌面工具，在没有敏感内容的 WorkBuddy 空白任务中依次验证一款浅色 v2 主题和一款深色 v2 主题：应用、主界面检查、打开新空白任务/内部导航、恢复默认、用户主动退出不重启、重新应用。任何再次重启前重新确认当下没有运行中任务。**Gate：** 两款主题均满足 Spec 的宿主表面覆盖，普通网页/文档/webview/非主 renderer 未注入，恢复完整；否则修复并重跑，不降低验收标准。
7. **桌面工具视觉与回归。** 用真实桌面工具窗口检查正常和 720 px 宽度下的三段目标选择、未安装/选中/重启确认/实验提示，键盘与 VoiceOver 标签可辨；同时抽查「豆包」「豆包工作」目标选择与现有端口/恢复测试。**Gate：** 无截断、遮挡、错误激活态或既有目标回归。
8. **文档、全量适用门禁与证据。** 只有前述 Gate 通过后更新架构、双语 README 和 CHANGELOG 的实验支持说明；运行 `cargo fmt --all -- --check`、`./scripts/check.sh rust`、`./scripts/check.sh workflow`、`git diff --check`，以及因实际文档/桌面改动需要的最小附加检查。创建并填写 `verification.md`，记录 WorkBuddy 版本、命令、结果、视觉证据路径、偏差和剩余风险。**Gate：** 所有适用检查通过，工作区无意外文件或专有/用户内容。
9. **独立验证 Gate。** fresh-context verifier 或人类验证者对照 Spec 复核代码差异、自动检查、WorkBuddy 真实窗口证据、隐私边界和恢复结果，再决定 `verification.md` 的最终 verdict。实现者不自行越过高风险最终验证/发布 Gate，也不创建 Release、部署或修改生产。

## Test-first proof

- `TargetApp::WorkBuddy` 的 id、bundle id、安装/二进制路径、环境变量、默认端口 `9224`、进程模式、启动标记必须有精确断言，并与另两目标两两不同。
- URL 表驱动测试至少包含：已验证主 renderer（含无 hash、hash、query）、同 app 下非 renderer file URL、其他 app.asar、普通本地文件、`http(s)`、webview/DevTools/extension，以及豆包两目标 URL；只有 WorkBuddy 主 renderer 可建立身份。
- 准备状态测试以合成端口/进程状态覆盖：未安装、未运行、正确端口已就绪、运行中但端口缺失、端口被其他程序占用。WorkBuddy 运行中端口缺失只能返回 `RestartConfirmationRequired`，不得直接调用退出；豆包既有分支保持原语义。
- watcher 策略测试锁定 WorkBuddy 端口消失后结束，豆包目标仍可沿当前策略重启；协议桥测试/搜索锁定只接受 `DoubaoWork`。
- 主题 adapter 测试构造带唯一原始 CSS sentinel 和图标资源的 v2 Theme，断言 WorkBuddy JS 有结构化颜色/目标 scope/恢复 runtime，却没有 sentinel、图标 data URI 或 WorkBuddy 图标标记；v1 Theme 返回不支持。对应豆包 JS 仍包含原始 CSS，防止误删既有能力。
- 桌面纯函数测试扩展为三安装布尔值/集合，覆盖保存目标仍安装、只装 WorkBuddy、三款都装仍默认豆包工作、保存目标被卸载的回退，以及 active theme 必须同时匹配 target 和 theme id。
- 每次实现迭代先运行最窄的单元测试；在行为稳定后再跑 `./scripts/check.sh rust`，避免用全量门禁代替失败定位。

## Visual or integration proof

- 运行时探针只记录 `type`、脱敏后的 URL 结构、监听地址、目标数量和宿主 CSS/landmark 名称；不执行读取正文、存储、Cookie、网络或插件状态的脚本。
- WorkBuddy 实窗使用新建空白任务。证据至少包括浅色和深色主题各一张正常宽度、一张窄窗口，以及恢复默认后的对照；截图只保留验证所需窗口区域，现有任务列表、账号、通知和任何用户内容不得进入仓库。
- 视觉检查覆盖：侧栏与选中项、主内容背景/表面、输入区边界和可读性、常规按钮、一个无敏感内容的弹层、代码样式、滚动条、可选背景层，以及 macOS 窗口按钮/拖拽区。
- 交互检查覆盖：`Command-3`、鼠标选择 WorkBuddy、v1 主题不可用说明、首次重启风险、明确「重启并应用」、应用成功态、内部导航持续注入、恢复默认和主动退出不拉起。
- 安全检查通过 `/json` 和只读 DOM marker 证明仅主 renderer 有 `data-skin-target=workbuddy`；普通网页、文档/webview、DevTools 和其他 page target 均没有工具 style/backdrop/marker。
- 视觉证据存放在 Git 忽略的 `work/verification/2026-08-31-support-workbuddy/` 或等价临时目录，`verification.md` 只引用脱敏证据路径和观察结论，不提交 WorkBuddy 官方资源或用户内容。

## Risks and mitigations

- **远程调试参数被忽略或禁用：** 探针立即停止；不修改 app.asar、不重签名、不打开安全绕过，Plan 改为 blocked/revised 并报告证据。
- **单实例启动影响当前工作：** 非干扰探针失败后不继续；在真正退出前取得执行当下的人类确认，先让用户保存或结束任务。
- **进程匹配误杀 Electron 子进程/其他应用：** 只使用完整 WorkBuddy bundle 二进制路径和 bundle id；测试禁止宽泛模式，端口冲突绝不 kill。
- **CDP 暴露面：** 固定 `127.0.0.1`、独立端口、先做身份归属；不把端口或 WebSocket 暴露到产品 UI/日志之外的系统。
- **哈希类与版本漂移：** 优先稳定 landmark/ARIA/data 属性和宿主布局层，记录 5.3.14 证据；版本变化显示实验警告，不宣称已验证。
- **主题 CSS 误伤 WorkBuddy：** WorkBuddy 不注入主题原始 CSS和图标，只使用结构化 v2 adapter；所有规则带 target scope，并避开嵌入内容。
- **恢复不完整：** runtime 在注入前保存原属性，恢复删除 style/backdrop/自有标记并回写原值；实窗对照和重新普通启动是发布阻塞证据。
- **豆包现有行为退化：** 新生命周期差异使用目标策略而非改写全局默认；保留并扩充豆包目标/JS/恢复测试，运行完整 Rust 门禁。
- **隐私泄露：** 禁止完整进程命令行、页面正文、存储或网络取证；截图限空白任务并脱敏，不提交 app.asar、官方资源或用户资料。

## Rollback

- 运行时：停止主题 watcher；若正确 WorkBuddy CDP 页面仍存活则执行现有 `restore()`，确认工具 style/backdrop/marker 消失；随后只终止本次明确启动的 WorkBuddy 实例并通过 LaunchServices 普通重开。官方 bundle、签名和用户设置从未修改，不需要文件级回滚。
- 代码：在未发布前撤销本 change 对 `live.rs`、`theme.rs`、`main.rs` 和文档的局部改动，删除 WorkBuddy 目标入口即可；保留与失败事实有关的 artifact/verification 记录。不得用 `git reset --hard` 或覆盖用户无关改动。
- UI：如果核心探针或恢复 Gate 未通过，WorkBuddy 不出现在可用目标中，或保持明确不可用的实验状态；不能留下一个只识别安装但无法安全应用/恢复的入口。
- 发布：本计划不授权 commit、push、PR、Release、部署或生产审批。任何发布动作仍需单独的人类授权。

## Deviations

当前无偏差。若真实探针发现主 URL、启动参数、监听地址、单实例行为或 DOM 边界与 Spec 不符，先同步 Intent/Spec/Plan 并重新取得受影响阶段的批准；不得以扩大 URL 匹配、改包、持久插件、全局 Electron kill 或读取用户内容作为临时绕过。

## Decision

待产品负责人明确接受本 Plan 后进入 Build。Plan 获批只授权按上述顺序实现；第一次可能中断现有 WorkBuddy 的重启探针仍需要执行当下的人类确认，且最终验证/发布 Gate 保持关闭。
