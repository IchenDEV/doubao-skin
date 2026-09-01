---
id: "2026-08-31-theme-package-v3"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-31"
based_on: spec.md
risk: "high"
approved_by: "engineering-risk-owner"
approved_at: "2026-08-31"
---

# Plan: 主题包 v3 与全部内置主题迁移

## Files and ownership

本变更由 `codex` 顺序执行。当前工作区已有 WorkBuddy 支持相关未提交改动，并与 `theme.rs`、桌面 UI、文档重叠；不启用并行 worktree，也不让不同执行者同时修改共享文件。每一阶段都在当前工作区上保留既有改动并以小范围 diff 复核，不回退或覆盖其他变更。

| 文件或目录 | 责任 |
| --- | --- |
| `workflow/changes/2026-08-31-theme-package-v3/` | 保持 Intent、Spec、Plan、Verification 与实际实现同步；记录 30 主题迁移矩阵和 Gate。 |
| `design/theme-standard/theme-v3.schema.json`、`design/theme-standard/README.md`、`design/theme-standard/fixtures/` | v3 权威 Schema、字段说明、完整/最小示例、合法与恶意契约 fixtures。v2 Schema 保留用于兼容，不原地改写。 |
| `crates/skin-core/src/theme_package.rs`（新增） | 深模块：隐藏 v1/v2/v3 manifest 分派、严格字段解析、继承/`null` 删除、目标能力、路径/资源校验和 `ResolvedTheme` 生成。外部 interface 保持为“验证一个目录并按目标解析”，不把原始 JSON 结构泄漏给调用者。 |
| `crates/skin-core/src/theme_css.rs`（新增） | `theme_package` 的内部实现：使用真正的 CSS parser 校验语法、主题/目标作用域、属性白名单、保留变量、文件大小和加载链。它不是面向桌面/Web 的第二个公共 interface。 |
| `crates/skin-core/src/theme.rs`、`lib.rs`、`build.rs`、必要时 `live.rs` | 把现有运行时 `Theme` 接到已验证/已解析的数据；按目标生成固定 CSS 顺序，同时保持 v1/v2、背景、图标、恢复和 WorkBuddy 内置 adapter 行为。离线构建继续明确使用豆包工作目标。 |
| `crates/skin-core/src/authoring.rs`、`src/bin/doubao-theme.rs` | v3 创建、检查、预览、迁移、打包、安装与 JSON 报告；显式目标参数；主题包只包含引用文件。 |
| `crates/skin-core/Cargo.toml`、`Cargo.lock`、`THIRD_PARTY_NOTICES.md`、必要许可证文件 | 引入并固定兼容当前 Rust toolchain 的 CSS parser；记录许可证与依赖变化。 |
| `crates/skin-core/tests/`、模块内单元测试 | 契约、兼容、CSS 安全、路径、CLI、打包/导入和 30 主题全量扫描回归。测试通过深模块 interface，不复制解析实现。 |
| `apps/desktop/src/main.rs` | 用统一能力结果替换“WorkBuddy 仅支持 v2”硬编码；显示支持/专属适配/兼容模式与当前目标不可用原因。保留现有三目标和重启授权逻辑。 |
| `apps/web/scripts/sync-themes.mjs`、`apps/web/src/lib/`、`apps/web/src/components/`、相关页面/测试 | 先调用权威 CLI 校验/打包，再读取解析结果生成目录；保存目标支持元数据并提供目标筛选与详情标签。 |
| `apps/web/data/`、`apps/web/public/themes/` | 仅通过 `pnpm --dir apps/web sync` 重新生成；不得手工编辑数据库、catalog、预览或 ZIP。 |
| `themes/*/` | 迁移当前 30 个内置主题及实现期间新增主题；ID、名称、预览、排序、来源与素材保持，版本从 `1.0.0` 提升为 `2.0.0`。 |
| `README.md`、`README.en.md`、`docs/architecture.md`、`docs/submitting-themes.md`、`docs/README.md`、主题创建 Skill、`CHANGELOG.md` | 更新 v3 作者与兼容说明；不把未通过最终 Gate 的能力写成已经发布。 |

新深模块的最小 interface 预期为：

```text
validate_theme_package(theme_dir) -> ValidatedThemePackage | ThemePackageError
ValidatedThemePackage.support(target) -> TargetSupport
ValidatedThemePackage.resolve(target, appearance) -> ResolvedTheme | ThemePackageError
ValidatedThemePackage.report() -> ValidationReport
```

调用者只需要目标、外观和结构化结果；Schema 分派、CSS AST、合并次序、文件反向引用集合与诊断行列都留在模块实现内部。文件系统属于本地可替代依赖，测试直接使用临时目录，不额外引入 port 或浅 adapter。

## Order of work

1. **冻结基线并确认重叠文件**
   - 记录当前 30 个主题 ID、版本、manifest/CSS/资源摘要、Web 目录数量和当前 WorkBuddy 变更状态。
   - 复核 `git status` 与重叠 diff，把现有 WorkBuddy 改动视为输入，不重新实现或回退。
   - Gate：基线清单与 Spec 的 30 个 ID 完全一致，30 个主题均为 v2/`1.0.0`/`appearance: both`；`./scripts/check.sh workflow` 通过。

2. **先落 v3 Schema、fixtures 与解析 interface**
   - 新增 `theme-v3.schema.json`、完整鲸鱼娘示例，以及仅单目标、双豆包、三目标、未知目标、未知字段、错误 variant、`null` 删除和路径逃逸 fixtures。
   - 先写 `theme_package` interface 测试，再实现 v1/v2 兼容分派、v3 严格解析、深层合并、有效外观与最小语义校验。
   - `ThemePackageError` 提供稳定错误类别与 JSON Pointer/目标/外观/文件位置；用户界面只消费简短分类，CLI 可输出细节。
   - Gate：所有合法/非法 manifest fixtures 通过；v1/v2 行为与当前兼容矩阵一致；`schemaVersion > 3` 失败关闭。

3. **引入 CSS AST 校验，不采用字符串扫描**
   - 先做一个受限依赖检查，优先采用成熟 Rust CSS parser；确认 Rust 1.97.1、许可证、构建体积和解析/序列化能力后固定依赖。若候选无法正确解析 selector 与 Unicode escape，则停止并修订 Plan，不退回正则校验。
   - 先写恶意 fixtures：无根作用域、错误目标、目标集合越界、`@import`、`url()`、`@font-face`、`@keyframes`、保留变量、交互/布局属性、注释/大小写/转义/嵌套绕过和超限文件。
   - 实现 CSS 文件反向目标引用集合、每个 selector 的主题/目标根校验、视觉属性白名单、重复有效路径与固定加载链。
   - Gate：合法共享/子集共享/目标 CSS 通过，全部恶意 fixtures 失败且诊断定位到文件与规则；未作用域 CSS 不可能进入运行时。

4. **把深模块接入运行时主题引擎**
   - `theme::load` 先取得 `ValidatedThemePackage`，再构造保持现有调用方式的运行时 `Theme`；桌面、live watcher、离线构建和 CLI 不再各自猜 manifest。
   - 根据最终结构化值生成宿主 adapter，再按 engine → shared → shared variant → target → target variant → runtime 拼接；同一目标链重复文件失败。
   - v1/v2 保持旧路径：豆包家族继续使用原始 CSS，v2 WorkBuddy 继续忽略原始 CSS；v3 只读取 manifest 声明的入口。
   - 背景、字体、图标、预览和 `surfaceOpacity` 全部从当前目标解析结果获取；WorkBuddy `icons: null` 不产生图标标记。
   - Gate：三个目标、两个外观、三类支持级别、CSS 顺序、恢复脚本、离线豆包工作构建与 v1/v2 回归测试通过。

5. **升级作者 CLI、打包与原子安装**
   - `create` 默认生成 v3，并要求作者显式提供目标；生成完整最小语义层，不创建无意义空 CSS。
   - `check --json` 输出 schema、支持级别、声明来源、有效外观、CSS 顺序、资源、警告和可定位错误。
   - 增加受控 `migrate-v3`：默认 dry-run，输出字段移动、来源归并、CSS 违规和建议目标文件；只有显式写入参数才修改主题，并在写入前生成可比较的临时结果。它不得自动声称真实窗口已通过。
   - `pack` 复用已验证清单，只打包 manifest 传递引用与允许的 README/LICENSE/NOTICE；`install` 允许无根 `theme.css` 的 v3 包，并在临时目录完成全部校验后原子替换。
   - `apply`/`restore` 帮助和参数加入 WorkBuddy；应用不支持目标时返回 invalid-theme 类结果而不是尝试注入。
   - Gate：CLI JSON/退出码、create→check→preview→pack→install→list、失败更新保留旧版本和目录穿越回归通过。

6. **接入桌面能力状态**
   - 把当前布尔 `supports_target` 迁移为统一 `TargetSupport`；保留一个方便调用的布尔方法，但文案和禁用原因来自结构化状态。
   - 删除“WorkBuddy 仅支持 v2 主题”等格式硬编码，改成对当前目标通用的“不支持当前应用”；详情可显示“专属适配”“支持”“兼容模式”。
   - 切换目标时预览使用目标专用预览或通用回退；每个目标独立持有 apply/active/restore 状态、watcher 和 generation。切换管理上下文不停止其他目标；同一目标操作串行，恢复只清理明确选择的目标。
   - Gate：三目标主题筛选、可应用状态、VoiceOver 文本、Command-1/2/3、安装失败、跨目标并行 watcher、同目标恢复/重应用串行与按目标完成消息测试通过；正常和窄窗口实窗无截断。

7. **接入 Web 目录、筛选和权威打包**
   - Web 同步先构建/调用一次权威 `doubao-theme` CLI，对每个主题取得解析报告并用 CLI pack 生成 ZIP；删除 Node 直接 `zip -r` 整个目录和自行猜 v2 顶层字段的路径。
   - 预览/色板读取 v3 `shared` 与预览外观解析结果；数据库和 TypeScript 类型保存 `schemaVersion`、目标数组及支持级别。
   - 主题库增加低干扰目标筛选，主题卡/详情展示友好的三应用范围；查询参数与类型/系列筛选组合，不改变现有搜索。
   - Gate：Web 脚本/类型/筛选测试通过；浏览器在桌面和窄宽度验证筛选、详情、下载链接和无结果状态；目录与 ZIP 元数据来自同一解析报告。

8. **先迁移鲸鱼娘作为纵向样板**
   - 将 `gallery-whale-maid` 迁移为 v3/`2.0.0`：共享结构字段只保留一份，豆包与豆包工作共同引用豆包家族 CSS，WorkBuddy 只使用共享层和确有需要的专用 CSS，图标继承显式删除。
   - 删除旧 CSS 中由可信背景/runtime 承担的伪元素与禁止属性，补齐内置 adapter 暴露出的结构语义缺口；不为主题 ID 增加 Rust 分支。
   - Gate：Schema/CSS/pack/install/Web sync 通过，并在豆包、豆包工作、WorkBuddy 的 light/dark 真实窗口完成 6 场景；背景、侧栏、类型切换、输入区、按钮、代码/弹层、边框重量和恢复均通过。2026-08-31 用户在豆包应用缺失、其余 4/6 场景已通过后明确要求“先批量迁移”，因此允许步骤 9 提前执行；缺失的豆包明暗场景继续阻塞最终全量验收、目录切换和发布。

9. **按风险批次迁移其余 29 个主题**
   - 批次 A：无背景的纯色主题，验证色板/对比度与图标继承。
   - 批次 B：普通背景主题，验证背景、veil、surfaceOpacity、深浅 variant 与移除旧伪元素后的等价性。
   - 批次 C：`doubao-dessert-giggle`、`doubao-snack-giggle`、`peach-sunset` 等多图标/特殊规则主题，逐项核对图标、运行时标记与 CSS 白名单。
   - 每个主题先运行 `migrate-v3` dry-run，人工复核结构/CSS职责后写入；版本统一从 `1.0.0` 到 `2.0.0`，三个目标显式存在。共享 adapter 足够时不创建 WorkBuddy CSS。
   - 每完成一个批次立即运行全量主题扫描、Rust 契约、CLI pack/install 和 Web sync dry comparison；发现结构语义缺口时优先扩展通用结构字段或可信宿主 adapter，禁止主题 ID 分支或放宽安全规则。
   - Gate：30 个主题全部 v3/三目标/两外观可解析，ID/名称/描述/预览/排序/来源/许可证/资源无回归，旧禁止 CSS 为零。

10. **生成目录并执行 30 主题包闭环**
    - 对每个主题在干净临时目录执行 check、pack、install、list/target report；验证包只含引用文件，SHA/大小生成稳定。
    - 运行 Web sync，确认数据库/catalog/30 个 ZIP/预览数量一致，目录中每个主题目标元数据与 CLI 报告一致。
    - Gate：30/30 打包与干净安装通过；生成文件只由 sync 产生；第二次 sync 无非确定性 diff。

11. **完成真实应用 180 场景矩阵**
    - 使用无敏感内容的空白测试会话，依次对 30 个主题 × 3 个目标 × light/dark 应用、取证、恢复。每次只操作明确目标，不修改官方 app bundle。
    - 自动记录应用版本、主题 ID、目标、外观、运行时标记、加载 CSS 文件、关键计算色/透明度/对比度和截图路径；截图在本地忽略证据目录中，仓库只记录脱敏索引与结果。
    - 所有 180 个基本场景来自真实窗口。布局窄宽度另外按三个目标 × 纯色/背景/多图标三类代表主题 × 两外观验证；因为 v3 CSS 禁止布局属性，结构几何风险由宿主/类别矩阵覆盖。
    - 任一场景失败就回到对应通用 adapter 或主题层修复，随后重跑该主题六场景及受影响类别；不删除目标声明绕过。
    - Gate：180/180 通过，窄窗口代表矩阵通过，恢复默认无残留，截图无用户任务/账号/凭据/工作空间内容。

12. **文档、完整门禁与独立 verdict**
    - 更新主题标准、投稿指南、架构、双语 README、创建主题 Skill、CHANGELOG；明确 v1/v2 外部兼容与 v3 内置目录状态。
    - 运行最小检查后执行 `./scripts/check.sh all`，另执行生成目录一致性、全量主题检查、30 包 round-trip、依赖许可证和 `git diff --check`。
    - 将命令、结果、180 场景矩阵、浏览器/桌面实窗证据、应用版本、偏差和剩余 DOM 版本风险写入 `verification.md`。
    - 高风险最终 verdict 由 fresh-context verifier 或人类记录。只有所有自动/实窗 Gate 通过后才可标记 passed；发布、合并或上传仍需另行授权。

## Test-first proof

- **Manifest 契约先于实现**：先提交合法/非法 v3 fixtures，再让 `theme_package` 通过。测试只经深模块 interface 断言 support、resolved fields、CSS order 与错误，不检查私有中间结构。
- **CSS 安全回归先于 parser**：先覆盖注释、转义、大小写、嵌套 at-rule、错误目标、禁用属性、远程/数据 URL 与保留变量，再实现 AST 校验；任何字符串包含式替代都会使 Gate 失败。
- **兼容矩阵先于重构**：为 v1、v2 豆包、v2 豆包工作、v2 WorkBuddy 写固定回归，证明 v3 接入没有改变旧包路径。
- **层叠顺序先于主题迁移**：合成不同颜色/同 specificity/`!important` fixtures，锁定标准 CSS 级联和六层拼接顺序；重复路径、`null` 删除和 appearance 选择均有独立测试。
- **包安全先于同步**：构造包含未引用文件、符号链接、路径穿越、缺失资源、伪图片/SVG 外链和失败更新的 ZIP，证明 check/pack/install fail closed 且不覆盖旧主题。
- **能力状态先于 UI**：用纯函数测试 `unsupported/shared/tailored` 与 `explicit/legacy-inferred`，桌面和 Web 只渲染结果，不复写 schema 判断。
- **鲸鱼娘样板先于批量**：只有纵向样板完成 Rust→CLI→桌面→Web→三应用实窗闭环，才允许机械迁移其余 29 个主题。
- **全量扫描防止漏迁移**：测试枚举 `themes/*/theme.json`，要求目录集合不丢基线 ID、全为 v3/`2.0.0`/三目标/两外观、无禁止 CSS；新增主题自动进入同一检查。
- **生成确定性**：Web sync 连续运行两次，第二次不得产生 diff；30 个 ZIP 的文件列表只能来自验证报告。

## Visual or integration proof

- 桌面主题工具本身：正常窗口与最小/窄窗口检查三目标切换、主题可用性、支持标签、专用预览、安装错误、键盘与 VoiceOver。使用临时 QA 构建，不能覆盖 `/Applications/豆皮.app`。
- 网站：使用 in-app Browser 优先进行本地页面 QA，检查目标筛选与类型/系列/搜索组合、主题卡、详情、下载和响应式布局；记录控制台错误与截图。
- 官方应用：分别使用豆包、豆包工作、WorkBuddy 的真实窗口。只在经身份确认的 loopback CDP 页面应用主题；每次保存主题/目标/外观/应用版本与恢复结果。
- 180 场景使用固定无正文画面，至少包含侧栏、主内容、类型切换、输入区、常规/选中按钮、代码/弹层或相应可见替代、选区和滚动条。计算对比度与透明度探针辅助判断，但不替代肉眼截图检查。
- 截图只保存在被 Git 忽略的 `work/verification/2026-08-31-theme-package-v3/`，文件名使用 `<theme>--<target>--<appearance>.png`；联系表可用于人工快速复核，仓库不提交官方界面资源或用户内容。
- 视觉通过标准：正文对比度至少 4.5:1，大号文字 3:1；背景层不被白色宿主遮挡；该透明/不透明的表面层级一致；类型切换与按钮可读；侧栏不过度描边；没有主题 CSS 引起的布局移动或交互遮挡。
- 恢复验证：用户明确恢复某个目标或最终退出工具时，对相应目标调用既有 `destroy()`，确认 style/backdrop/data attributes 与图标标记清零；仅切换桌面端目标选择不触发恢复，WorkBuddy 用户退出后不被自动拉起。

## Risks and mitigations

- **与进行中的 WorkBuddy 变更重叠**：先固定其最终接口和回归，所有修改基于现有 diff；不复制 watcher/adapter，也不改协议桥边界。若该变更的最终 verdict 改变宿主契约，先修订本 Spec/Plan。
- **CSS parser 依赖可能过重或许可证不合适**：先做小型编译/fixture Gate；确认后才写业务代码，并更新 notices。不能以“减少依赖”为由退回正则安全检查。
- **旧 CSS 使用 v3 禁止能力**：把背景/图标/布局职责移到结构字段和可信 runtime。若确有无法表达的安全视觉需求，先提出最小结构字段修订并重新走 Spec，而不是放宽白名单。
- **`null` 与缺失值混淆**：manifest 层使用能保留三态的内部表示，解析后才生成普通运行时类型；测试覆盖对象叶子和整对象删除。
- **30 主题批量改动掩盖个别回归**：按纯色、普通背景、多图标批次推进，每批保留 ID/资源/预览摘要与 diff；纵向样板和每主题六场景都必须通过。
- **180 场景耗时且应用状态会漂移**：建立可恢复的顺序验收 harness，单场景有独立结果与重试，失败不清空已通过证据；每 60 秒内输出进度，不用长阻塞等待。
- **官方应用 DOM/版本变化**：记录三应用版本和目标 URL；主题专用 CSS只使用实测稳定语义选择器。版本变化时把结果标为待复核，不能宣称永久兼容。
- **旧客户端不识别 v3**：保留 v1/v2 用户包读取；商店按客户端最高 Schema 过滤，目录正式切换前完成回滚演练。内置主题 v3 不伪装成 v2。
- **Web 目前直接 zip 整个目录**：改为权威 pack 输出并测试未引用文件不会进入包；避免 Node 与 Rust 两套校验漂移。
- **生成文件体积与非确定性 diff**：只在源主题和同步逻辑稳定后生成；连续两次 sync 验证稳定，不手改数据库/ZIP/预览。
- **隐私风险**：所有实窗使用空白测试上下文，自动探针不得读取正文、Cookie、账号、网络或工作空间。出现用户内容立即删除截图并重新取证。
- **范围失控**：不新增任意应用插件框架、主题组合、远程依赖或 JS；发现旁支问题只在阻塞本 Spec 时修复，否则记录为后续 Intent。

## Rollback

- Build 未开始前无需回滚；Plan 拒绝或修改只更新 Artifact。
- 实现使用顺序检查点：Schema/fixtures、深模块、runtime/CLI、桌面/Web、鲸鱼娘、三批主题、生成目录。某阶段失败时只撤销该阶段由本变更新增的文件/改动，保留此前通过阶段和工作区原有改动；不用 `git reset --hard`、`git checkout --` 或宽泛删除。
- 主题批量迁移在目录/发布切换前都留在 Git 可审 diff 中。单主题失败就回退该主题的 v3 源改动到迁移前内容或继续修复，不发布混合“完成”状态。
- Web 目录保持旧生成结果直到所有 30 个主题源和权威打包通过；同步失败时恢复上一份已验证目录产物，不上传或部署新目录。
- 安装/更新测试在临时目录进行，原子安装失败会恢复备份；不触碰真实用户主题目录。
- 实窗失败立即对当前目标执行恢复默认并停止 watcher；不得通过修改官方应用包修复。
- 若已经发布后发现阻断问题，优先撤回 v3 目录/下载入口并继续让客户端读取 v1/v2 用户包；修复通过全部相关 Gate 后用新主题补丁版本重新发布。发布和撤回都需要单独的人类授权。

## Deviations

- 用户在 Intent 接受后扩大范围，要求迁移全部现有主题；已在 accepted Intent 决策记录和 accepted Spec 中同步，基线锁定为当前 30 个主题。
- 2026-08-31 鲸鱼娘已通过 DoubaoWork 与 WorkBuddy 的明暗 4/6 场景，本机仍缺少 Doubao。用户随后明确要求“先批量迁移”，授权步骤 9 在剩余两个豆包实窗场景前执行。此授权只调整 Build 顺序，不把缺失证据视为通过，也不解除最终目录切换、Review 或 Release Gate。
- 当前计划不把 180 场景简化为抽样；只有窄窗口几何验证按“目标 × 主题结构类别 × 外观”覆盖，因为 v3 目标 CSS禁止布局属性。若实现中允许任何布局属性，必须把窄窗口扩展到受影响的全部主题并修订 Plan。
- `gallery-whale-maid` 是纵向样板，不是主题 ID 特判；任何为它新增的能力必须是可复用的结构字段或宿主 adapter 行为。
- 新 CSS parser 和新增结构字段的最终选择可能因 Rust toolchain/许可证或真实旧 CSS需求调整。任何会放宽安全边界、改变 manifest 形状或减少三目标范围的调整都必须先修订 Spec 并重新获得接受。
- 本计划不创建 PR、不合并、不部署、不发布，不修改 `/Applications/Doubao.app`、`/Applications/DoubaoWork.app` 或 `/Applications/WorkBuddy.app`。这些外部动作不随 Plan 接受自动授权。

## Decision

工程风险负责人已于 2026-08-31 明确接受本 Plan，Build 已获授权。用户又于 2026-08-31 明确接受先批量迁移、后补缺失豆包实窗证据的顺序偏离。该接受授权上述仓库内实现、测试、30 个主题迁移和本机三应用无敏感内容验收；不授权提交 PR、合并、部署、发布或修改官方应用安装包。
