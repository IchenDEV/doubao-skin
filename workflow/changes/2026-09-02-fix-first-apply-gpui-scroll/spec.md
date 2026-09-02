---
id: "2026-09-02-fix-first-apply-gpui-scroll"
stage: spec
status: accepted
owner: "codex"
created: "2026-09-02"
based_on: intent.md
risk: "medium"
approved_by: "user"
approved_at: "2026-09-02"
---

# Spec: fix first apply gpui scroll

## Requirements

1. `skin-core::live` 必须把“尚未成功注入”与“已经成功后的长期监听”视为不同阶段。首次成功前，暂时失败的页面必须在下一个现有 watcher 周期重新探测，不能进入约 30 秒的稳定期 dead-page 节流。
2. 首次注入阶段必须有覆盖全部状态的整体期限：无匹配页面、页面缺少可用 target 信息、连接失败和脚本执行失败都必须在期限内成功或返回错误，不能无限等待。
3. 现有 30 秒 `INITIAL_INJECTION_TIMEOUT` 继续作为目标身份已经建立后的首次注入期限；目标应用启动/端口发现仍使用现有有界 `ensure_running`。首次成功后清除首次失败状态并恢复每约 30 秒一次的 dead-page 探测，避免稳定 watcher 持续占用 CDP。
4. 桌面 apply generation 必须在首次注入成功后进入 active；首次期限到期、取消或真实错误后离开 busy。失败 generation 不得保存为自动保持主题，也不得覆盖此前成功主题。
5. GPUI 渲染函数不得调用 `TargetApp::is_installed`、`registered_macos_bundle`、`osascript` 或任何同步子进程。三个目标应用的安装状态必须在 `SkinApp` 初始化的非热路径一次探测并用于目标切换器和详情按钮渲染。
6. 点击切换目标或应用主题时，继续通过现有 `switch_target`、`prepare_state` / `is_installed` 路径做权威校验；渲染缓存只决定当前窗口的展示状态，不能替代执行前校验。
7. 原生主题列表继续使用现有 GPUI `overflow_scroll` 和约 34 个条目；本修复不引入虚拟列表、图片管线、异步 runtime、新依赖或后台安装探测轮询。
8. 当前主题缩略图、选中态、搜索、正常与紧凑布局、键盘快捷键、目标可用性标签和可访问性名称保持现有行为。

## User experience

- 用户冷启动目标应用后第一次点击“应用主题”，短暂的 renderer/CDP 未就绪会自动恢复，不再需要第二次点击；按钮在成功时显示“正在使用”。
- 如果目标在期限内始终无法注入，按钮必须退出“正在应用…”，显示现有失败反馈，用户仍可再次尝试；此前成功主题和自动保持设置不发生漂移。
- 在“我的主题”正常或窄窗口连续滚动时，滚轮/触控板输入不再因每帧启动多个 `osascript` 子进程出现数百毫秒迟滞。
- 官方目标应用在豆皮窗口已经打开后才被安装时，目标切换器允许在重开豆皮后刷新；真正执行主题操作仍以点击时的实时校验为准。

## Technical design

### Initial injection phase

- 保留现有 watcher 循环、`dead` 集合、2 秒周期和 15 tick 稳定期节流，只提取两个小型私有判定：首次阶段是否立即探测 dead target，以及首次阶段是否已整体超时。
- `applied_once == false` 时，dead target 每个 watcher 周期都可重新 `probe`；一旦任一匹配页面成功注入，`applied_once` 变为 true，清理首次错误，并恢复现有 15 tick 节流。
- 首次期限从 `ensure_running` 成功返回、watcher 开始观察目标页面时计算。期限到达时优先返回最近一次注入错误；如果从未得到具体注入错误，则返回“未找到可注入页面”一类有限诊断。错误通过现有 `Done` generation 清理 busy，不增加第二个 watchdog 线程。
- `once` 模式只有实际注入至少一个页面才可成功；空页面不能打印零页面成功。这与桌面 watcher 共用同一首次阶段判定。

### Render-time installation state

- 在 desktop app 层增加一个小型三目标安装状态值，初始化时复用目前已经执行的三次 `TargetApp::is_installed` 结果，同时供 `initial_target`、`render_target_switch` 和 `render_theme_detail` 使用。
- render 只读取内存中的布尔值。`switch_target` 和 `apply_selected` 保留实时探测，因此安装状态缓存不进入 core、不改变 launch/prepare 的权威判断，也不需要失效协议或跨线程共享。
- 不修改 `Assets`、`render_theme_thumbnail` 或主题图片。现有采样中图片加载只占 1 个样本，而 587/605 个绘制样本位于安装探测；Ponytail 边界是在已测根因处停止。

## Security and privacy

- 不新增网络、遥测、文件扫描、进程权限、宿主读写或配置 schema；仍只通过现有 loopback CDP 连接已验证目标。
- 首次注入诊断只包含目标应用、页面 URL 的既有限长片段和 CDP 错误，不读取或记录聊天正文、会话标题、账号、Cookie、通知或输入内容。
- GPUI 性能采样只记录本地进程栈和耗时；仓库不保存用户界面正文或当前对话截图。

## Alternatives and non-goals

- 不把 dead-page 全局探测间隔从 30 秒改成 2 秒。高频只限首次成功前，长期 watcher 继续低频探测。
- 不新增独立 UI watchdog、取消按钮、进度条或“重试中”状态；同一个 watcher 状态机已有足够信息完成有界重试。
- 不把 `osascript` 换成另一种每帧同步的 LaunchServices API。安装状态在一个窗口生命周期内近似稳定，缓存现有结果是更小且可验证的修复。
- 不虚拟化 34 项列表、不预压缩 18 MB 主题资源、不重写图片缓存。采样没有把图片或条目数定位为主要瓶颈。
- 不在本变更重新处理深色对比度、自动主题所有权或注入页面 DOM 扫描；这些属于关联但独立的 `2026-09-02-fix-theme-readability-auto-scroll` 变更。

## Areas of concern

- **冷启动误判。** 期限只在 `ensure_running` 已确认目标身份后启动，避免把端口启动时间重复计入；使用现有 30 秒常量，既消除无限 busy，也不缩短已有容错窗口。
- **首次重试负载。** 每 2 秒仅探测当前 dead target，成功后立即恢复 30 秒节流。测试必须证明稳定期没有被意外改成高频。
- **空页面成功。** 现有 `once` 分支可能在零注入时返回成功；修复必须让它遵循同一首次期限或明确错误，不能产生假绿日志。
- **缓存过期。** 窗口打开期间安装/卸载官方目标很少见；渲染允许暂时显示旧状态，但执行前必须实时校验。若未来需要即时发现安装变化，应由独立需求加入显式刷新事件，而不是恢复每帧探测。
- **UI generation 顺序。** 超时 `Done` 只能完成匹配 generation；旧线程或旧消息不得清除新操作、提交失败主题或覆盖 active session。
- **错误可见性。** 本次至少保证离开 busy 并显示现有失败反馈；扩展具体错误文案只有在验收证明通用反馈不足时再进入范围。

## Acceptance criteria

1. 新的状态测试在旧实现判红并覆盖：首次失败后的下一 watcher tick 立即允许 probe；首次成功后只有第 15 tick 允许 probe；停止或 generation 替换仍能退出。
2. 首次期限测试覆盖最近错误、从未出现可注入页面和 29.999/30 秒边界；30 秒前不失败，达到期限后返回有限错误。
3. 可控 fake CDP 场景第一次连接/执行失败、后续恢复时，单次 `run` 调用在下一 watcher 周期成功，不需要第二次用户操作；始终失败或无有效页面时在期限内返回错误。
4. `once` 模式零注入不得返回成功；成功注入一个或多个页面的现有 CLI 行为不回归。
5. desktop generation 回归证明失败/超时操作离开 busy，失败 B 不覆盖已保存 A；成功 B 仍能成为 active 并提交自动保持配置。
6. 安装状态探测回归证明每个目标在 `SkinApp` 初始化只调用一次检测，渲染目标切换器和详情不会再次调用检测；点击应用仍执行实时 `prepare_state`。
7. 真实验收应用冷启动豆包工作并重复至少 5 次“一次点击首次应用”，每轮成功或有限失败，任何一轮都不无限显示“正在应用…”。
8. 修复前相同真实 GPUI 滚动基线保留为 12 次上下滚动 9.181 秒、30 次安装查询 2.68 秒、605 个绘制样本中 587 个阻塞于 `registered_macos_bundle`。修复后 12 次操作明显缩短，滚动采样的 render/draw 路径中该符号与 `osascript` 均为 0。
9. 正常和窄窗口各连续滚动至少 10 秒，主题选择、52% 透明度、两个关闭开关和当前 active 主题不漂移，窗口无明显输入迟滞。
10. 定向 core/desktop 测试、`cargo fmt --check`、`./scripts/check.sh rust`、`./scripts/check.sh workflow` 与 `./scripts/check.sh all` 通过；verification 记录红绿证据、真实窗口结果和残余风险，并由独立 fresh-context verifier 或人类给出最终 verdict。

## Decision

等待用户明确批准本 Spec 后进入 Plan。当前只记录了 Intent 批准并编写可执行规格，没有修改产品代码、当前主题配置或官方应用资源。
