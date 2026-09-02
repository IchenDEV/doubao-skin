---
id: "2026-09-02-fix-theme-readability-auto-scroll"
stage: spec
status: accepted
owner: "codex"
created: "2026-09-02"
based_on: intent.md
risk: "medium"
approved_by: "user"
approved_at: "2026-09-02"
---

# Spec: fix theme readability auto scroll

## Requirements

1. 所有内置 v3 主题在豆包和豆包工作中必须按当前解析后的运行时 appearance 生成同极性的共享文本语义变量；深色运行时不得因为主题预览被声明为 light、CSS 文件先出现 light 规则或壁纸较亮而生成黑色 `--s-color-text-*` / `--dbx-text-*` 兜底值，浅色运行时也不得反向生成白色兜底值。
2. 修复必须覆盖共享语义变量级联，而不是只修改“甜点偷笑”、截图标题类名或一组临时宿主哈希 selector。主题作者明确声明的 composer、消息气泡和强调色继续生效。
3. 主要正文、标题、侧栏标签和主要控件文本与其实际合成背景的 WCAG 对比度至少为 4.5:1；24px 及以上且足够粗的文字至少为 3:1。禁用、装饰和 placeholder 状态可以使用较低强调度，但不得与可操作正文混淆。
4. 当前主题 watcher 必须区分三种 retained-session 标记：自己的主题标记继续运行；导航后标记缺失时重新注入；出现另一个非空主题标记时认定已被新所有者接管并退出，不得把新主题改回旧主题。
5. “自动保持上次主题”开启时，连续成功应用主题 A、B 后，B 必须成为唯一当前主题及持久化的最后成功主题；helper、主程序或旧 apply generation 均不得重新夺回 A。失败或被取消的 B 不得覆盖已保存的 A。
6. 主题运行时必须把外观/CSS/背景维护与语义图标、composer 的昂贵 DOM 标记分开。普通子树变化或滚动事件不得立即触发整页 `querySelectorAll`、`getBoundingClientRect` 和 `getComputedStyle` 扫描。
7. 初次应用仍需立即完成必要的图标与 composer 标记。后续 DOM/标签变化允许在滚动或连续变更停止后的短暂安静窗口内合并刷新；外观切换、导航恢复和主题销毁必须保持现有语义。
8. 滚动事件监听必须为 passive，所有 observer、timer 和事件监听在主题被替换或恢复默认时完整清理；不得累积第二套 runtime 或后台轮询。
9. 豆皮原生主题列表只在修复共享注入热路径后仍能稳定复现独立卡顿时才进入实现范围。当前约 34 个主题不作为引入新列表框架或大规模虚拟化重构的理由。
10. 主题运行行为变化后必须同步 Web catalog，且不能修改豆包工作、豆包或 WorkBuddy 安装资源。

## User experience

- 用户应用主题后无需理解“预览外观”或 CSS 变量；宿主切换到深色外观时，标题、正文、侧栏和输入区在一次渲染收敛内切到清晰的浅色层级，不出现长期黑字压在暗色壁纸上的状态。
- 壁纸、图标、圆角和用户选择的不透明度保持原设计。修复只纠正错误的文本极性与运行时维护开销，不把所有壁纸主题改成高不透明纯色面板。
- 开启自动保持后再点另一个主题，允许一次正常的应用过渡，但最终只停留在新主题；不得每两秒回切、闪烁或需要用户先关闭自动模式。
- 页面、会话侧栏和长列表连续滚动时，主题图标可以在停止滚动后很短时间内完成补标，但滚动本身不能因为主题脚本周期性全树扫描而明显顿挫。
- 修复不新增设置、提示框、性能开关或“主题所有权”文案。发生真实应用失败时继续使用现有错误反馈。

## Technical design

### Runtime appearance and contrast

- `skin-core::theme` 继续是主题语义 CSS 的唯一生成点。把默认文本极性从“扫描 CSS/背景首个颜色推断表面明暗”改为优先使用已经解析的 `ThemeMode`：`Dark` 固定生成浅色文本语义，`Light` 固定生成深色文本语义，只有兼容旧包的 `Auto` 路径才保留现有亮度推断。
- `surface_opacity_css` 的 composer fallback 同样使用运行时 `ThemeMode`，不能使用仅服务商店预览的 `preview_mode`。预览声明继续只影响预览图和原生 mockup，不再影响当前 dark/light runtime 的文本或 surface fallback。
- 保留 v3 级联顺序“共享语义 → package CSS → surface opacity → runtime safety”，但 resolved dark/light runtime 生成的基础 `!important` 语义值必须与当前 appearance 一致，使宿主 Tailwind 类 `text-dbx-text-primary`、Semi 变量和旧 `N900` 别名不会分裂成相反极性。
- 增加生成 CSS 的回归测试：使用 preview 为 light、shared appearance 为 both 的“甜点偷笑”代表包，分别生成 DoubaoWork light/dark runtime；断言 dark 的基础 `--s-color-text-primary` / `--dbx-text-primary` 为浅色，light 为深色，并校验已声明 composer/消息色不被兜底覆盖。再对全部内置 both 主题做相同极性矩阵检查。

### Single theme ownership

- 在 `skin-core::live` 的 retained CDP session seam 新增一个小型纯状态判定：`Own`、`Missing`、`Foreign`。输入只包含期望主题 ID 与页面根节点的可选 `data-skin`，不引入新进程锁、IPC 或全局协调服务。
- `Own` 保持 session alive；`Missing` 沿用现有导航后重注入；`Foreign` 记录有限诊断并让当前 `live::run_with_policy` 正常返回，使旧 UI watcher 或 helper watcher主动让出整个目标的所有权。空但存在的外来标记按 `Foreign` 处理，只有属性真正不存在才按 `Missing` 恢复。
- 已有 UI apply generation、成功后持久化和 helper 主程序优先规则保持不变。新增测试把两层契约接在一起：A watcher 看到 B 必须 yield；A 标记缺失必须 reinject；连续 A→B 只有 B generation 能提交自动配置；失败 B 保留 A。

### Scroll hot path

- 在 `render_bootstrap` 内把当前单一 `apply()` 拆成廉价的 appearance/runtime 维护与昂贵的 marker refresh。appearance 负责选择 CSS、维护根标记和 backdrop；marker refresh 才运行 `markIcons()` / `markComposerIcons()`。
- 初次 start 立即执行两部分。媒体/根 appearance 属性变化只合并调度 appearance；普通 `childList`、`aria-label`、`title` 变化只把 marker 标记为 dirty，并在最后一次变化后的短 debounce 窗口执行一次刷新。
- 在 document capture 阶段注册 passive `scroll` 监听；滚动期间持续推迟 marker refresh，停止后执行一次。现有周期恢复只调用相应的轻量维护并通过同一个 dirty/debounce seam 请求 marker，不得绕过它直接全树扫描。
- `destroy()` 清理 appearance timer、marker timer、interval、MutationObserver、media listener 和 scroll listener。新主题 bootstrap 仍先 destroy 旧 runtime，保证任一页面只有一个 observer/timer 集合。
- 回归 harness 对运行时脚本的真实浏览器 DOM 计数：初次标记仍发生；连续滚动/子树变更期间每帧不得出现全树扫描，安静窗口后至多一次合并扫描；destroy 后继续滚动或变更不得再增加计数。真实豆包工作探针复用相同计数口径。

## Security and privacy

- 不新增网络端点、遥测、文件扫描、进程权限或宿主资源写入；继续只连接已验证目标的 loopback CDP。
- marker 分类只读取根节点 `data-skin` 和当前 URL，不读取、记录或匹配聊天正文、会话标题、账号、Cookie、通知或输入内容。
- DOM 性能验证只统计 API 调用次数、元素数量和时间，不导出 textContent。真实截图使用空白主页或裁掉账号、最近会话与通知区域。
- watcher 让出所有权只停止豆皮当前 watcher，不退出、重启或修改官方应用；缺失标记仍保留导航恢复能力。

## Alternatives and non-goals

- 不逐个提高主题 `surfaceOpacity`：真实探针证明 dark runtime 已选中，但共享 `!important` 文本变量仍由错误的 light 推断生成；提高不透明度会掩盖而不能修复相反文本极性。
- 不给截图中的 `greeting-text-*`、当前 hash class 或单个主题写选择器。宿主类会漂移，且侧栏与其他正文使用同一错误的 `--dbx-text-primary`。
- 不关闭自动保持、取消导航恢复或在每次手动应用后杀 helper；用 foreign marker 明确让出即可满足单所有者规则。
- 不移除主题图标识别。只把全树标记移出滚动热路径并合并执行，保留初次与动态内容支持。
- 不引入 requestIdleCallback polyfill、Web Worker、虚拟 DOM、列表框架、通用调度器或第二个 watcher 协议。
- 不承诺优化官方应用自身不受豆皮控制的渲染、网络加载或超长会话性能；验收只归因于主题 runtime 的增量开销。

## Areas of concern

- **旧包 Auto 兼容。** v1/v2 或没有明确 resolved appearance 的主题仍可能依赖亮度推断；变更必须只让明确 `Light`/`Dark` 走确定极性，保留 Auto fallback 并跑旧包测试。
- **作者色与共享兜底。** package CSS 可以声明品牌/消息色，但基础宿主文本不能被相反极性的 `!important` 值锁死。测试需同时证明 dark 主文本修正和用户气泡/强调色未被抹平。
- **foreign 与导航缺失。** 若把属性缺失也当成外来所有者，会破坏刷新/导航恢复；若把任意非匹配值当缺失，就会复现主题争抢。纯状态测试必须覆盖 `None`、相同、不同和空字符串。
- **多页面 target。** 一个 watcher 同时维护目标的多个页面；任一已注入主页面出现 foreign 标记即表示目标被新主题接管，应让出整个 watcher，避免旧主题继续留在侧页。
- **延迟标记。** debounce 太短仍会在滚动中扫描，太长会让动态按钮短暂保留官方图标。Plan 需用红色计数 harness 选择最小稳定窗口，而不是凭感觉写常量。
- **页面重建。** head/body 被宿主替换时仍要恢复 style/backdrop。周期轻量维护与 marker dirty 调度不能因拆分而丢失导航恢复。
- **主题列表归因。** 真实红色基线来自豆包工作页面/侧栏。若豆皮原生主题商店仍卡，需要单独计时证据和新的最小实现；本变更不以主观感觉宣称已经优化未测得的 GPUI 热点。

## Acceptance criteria

1. 新回归测试在旧实现上证明：“甜点偷笑” dark runtime 的基础 `--s-color-text-primary:#000000!important` 覆盖了包内浅色文本，修复后同位置为浅色；light runtime 保持深色文本。全部内置 both 主题通过极性矩阵。
2. 真实豆包工作深色窗口应用“甜点偷笑”后，根节点仍为 `data-theme=dark`，`--s-color-text-primary`、`--dbx-text-primary`、标题 computed color 同为浅色极性；主要标题/正文/侧栏/输入区实测至少 4.5:1，大号标题至少 3:1，且壁纸、图标和 45% 用户透明度保持可辨。
3. watcher 状态测试覆盖 own→alive、missing→reinject、foreign/empty→yield；真实或隔离 CDP 场景连续应用 A、B 并观察至少两个原 2 秒检查周期，页面稳定为 B、A 不再重注入。
4. 自动保持开启的集成测试证明 A 成功保存后 B 成功成为唯一配置；B 失败/取消保持 A；helper/UI watcher 交接、恢复默认和现有 `only matching generation updates saved theme` 测试不回归。
5. 浏览器性能 harness 在初次 start 后成功标记代表图标/composer；连续滚动与 20 次子树/标签变化期间，全页扫描不按事件线性增长，安静窗口后至多一次合并 marker refresh；destroy 后计数保持不变。
6. 真实豆包工作空白页和侧栏连续滚动至少 10 秒，计数探针不再出现旧基线“一次无关变化触发 12 次 document 扫描、219 次布局读取、191 次 style 读取”的行为，窗口无肉眼明显周期性卡顿。
7. 真实豆皮正常与窄窗口主题列表连续滚动无明显输入迟滞。若仍可稳定复现独立 GPUI 卡顿，verification 明确判为未完成并回到新证据/Plan，不把官方页面优化冒充原生列表修复。
8. 主题切换、系统 light/dark 切换、页面导航、恢复默认、连续切换 10 次后，页面最多一个 `__doubaoSkinRuntime`、一个 observer/timer 集合；无残留 scroll listener、backdrop 或旧主题标记。
9. Web catalog 与内置主题同步检查通过；`cargo fmt --check`、定向 Rust/桌面测试、`./scripts/check.sh rust`、`./scripts/check.sh all` 和 `./scripts/check.sh workflow` 通过。
10. `verification.md` 记录三项红绿证据、真实窗口恢复前后状态、偏差与残余风险；最终 verdict 必须由独立 fresh-context verifier 给出，实施者不自批 verified。

## Decision

等待用户明确批准本 Spec 后进入 Plan。当前只完成只读代码审计、临时且已恢复的真实主题探针和 artifact 编辑，尚未修改产品代码、主题包、自动配置或宿主应用资源。
