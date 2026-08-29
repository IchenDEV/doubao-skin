---
id: "2026-08-29-rust-theme-skills-cli"
stage: spec
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
based_on: intent.md
risk: "high"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Spec: 面向普通用户的主题生产与应用工具链

## Requirements

1. 新增独立 Rust 命令 `doubao-theme`，由 `skin-core` 构建，不依赖 Python、Node.js 或 GPUI。源码运行、预编译分发和 Skill 调用必须走同一个二进制与同一套核心逻辑。
2. CLI 提供以下稳定命令：
   - `list`：列出内置与用户安装主题。
   - `create <theme-dir>`：根据名称、描述、强调色、外观和作者生成可直接加载的 v2 主题、基础 CSS 和 1200 × 675 预览图；目标目录存在且非空时拒绝覆盖。
   - `check <theme-dir>`：按现行主题标准严格检查主题目录。
   - `preview <theme-dir>`：从真实主题配置重新生成标准预览图。
   - `pack <theme-dir> [output]`：检查后生成 `.doubao-skin.zip` 安装包。
   - `install <package>`：安装或更新用户主题包。
   - `apply <theme>`：对当前豆包工作窗口应用主题，默认完成一轮可验证应用后退出；`--watch` 为显式开发者选项。
   - `restore`：清理在线主题运行时并恢复应用前的页面外观，不删除主题包。
   - `build <theme>` / `remove-build`：保留旧 Python CLI 的离线克隆与移除能力；仍不修改官方 App。
3. `--json` 为一次性命令输出一个稳定 JSON 对象；正常 stdout 不混入日志。文本模式使用中文结果与下一步建议。
4. 退出码固定为：`0` 成功，`2` 命令参数错误，`3` 主题或主题包无效，`4` 应用、文件系统或外部系统操作失败。
5. 新增两个标准 Agent Skill：
   - `create-doubao-theme`：理解自然语言主题需求，创建、修改、预览、检查和打包主题。
   - `apply-doubao-theme`：列出、检查、安装、应用、恢复和管理离线克隆。
6. Skill 自动发现保持启用；只读动作可直接执行，安装、应用、恢复、离线构建或删除前必须说明准确目标与影响并取得用户明确授权。
7. macOS 分发包同时包含 GUI、`doubao-theme` 二进制和两个 Skill 目录；源码用户可直接通过 Cargo 运行 CLI。
8. 两个仓库内 Skill 路径必须能被 Codex 的 `$skill-installer` 从公开 GitHub 仓库安装；README 与网站使用页给出仓库路径、安装提示、安装后发现方式和各自一条调用示例。仓库未公开前不得声称远程安装已验证。

## User experience

- 普通用户可以说“做一个暖黄色、适合夜间阅读的豆包工作主题”，生产 Skill 负责选择安全默认值并交付主题目录或安装包，不要求用户理解 JSON、CSS、端口或注入方式。
- 用户没有提供作者时，生成包使用中性的“本地用户”，不会读取或写入系统账户名；发布前 Skill 会提示改成真实署名。
- 纯色或渐变主题是默认路径。背景图、字体和图标只在用户提供素材、当前宿主具备明确生成能力，或来源与许可证清楚时加入。
- 应用 Skill 在重启豆包工作前提醒用户保存正在进行的工作；用户拒绝或没有明确同意时停在检查结果，不执行应用。
- 正常成功文案使用“主题已创建”“检查通过”“主题已安装”“主题已应用”“已恢复默认”。普通失败给出可执行动作，例如“请先打开豆包工作”或“请检查主题文件后重试”。
- `--json` 面向 Agent 和开发者，格式为成功 `{ok, command, result}` 或失败 `{ok, command, error: {code, message}}`。
- CLI 帮助和普通输出不要求用户提供项目根目录；主题可以由 ID 或目录路径定位。

## Technical design

### Rust 核心

- 在 `skin-core` 内新增小型 authoring 模块，公开创建、严格检查、预览和打包函数；主题解析、安装、在线应用和离线构建继续复用现有模块，不创建第二套解析器。
- 严格检查至少覆盖：v2 schema 标记、kebab-case ID、ID 与目录名一致、名称和描述长度、版本、作者、外观、双外观 variants、素材路径存在且位于主题目录、CSS 同时覆盖 `html` 与 `body`、必需语义变量及强调色状态变量。
- 本地加载继续兼容 v1 主题；只有 `create`、`check`、`preview` 和 `pack` 使用当前 v2 作者标准，避免破坏已安装旧主题。
- `pack` 只收入 `theme.json`、`theme.css`、`icon.icns`、许可证文件和清单真实引用的预览、背景、字体与图标；拒绝符号链接、路径逃逸和超出既有限制的内容，不把目录中的无关文件或隐藏文件带入安装包。
- 预览使用现有 `Theme::preview_style()` 与 `image` 依赖绘制 1200 × 675 界面缩略图；有背景时按配置裁切并叠加表面，不加入解释文字或实现说明。

### 在线应用与恢复

- 主题 bootstrap 在安装运行时前记录页面原有的 `data-theme`、`theme-mode` 与相关属性。
- runtime `destroy` 必须停止 observer/timer，删除动态 style、背景层、主题图标/输入框标记和 `data-skin`，并只恢复自己改动过的原始外观属性。
- `live::restore` 只连接回环调试目标并调用同一清理契约；成功必须至少清理一个响应页面，零响应页面返回失败，不能把“端口存在”报告成完成。
- `apply` 默认一次性应用，避免 Skill 留下不可管理的后台 watcher；`--watch` 在前台运行直到中断。

### CLI 与 Skill

- CLI 位于 `crates/skin-core/src/bin/doubao-theme.rs`，使用项目现有的手写参数解析方式，不新增参数解析依赖。
- 两个 Skill 位于 `skills/create-doubao-theme/` 与 `skills/apply-doubao-theme/`，各自包含简洁的 `SKILL.md` 和 `agents/openai.yaml`；机械操作全部交给 CLI，不再添加脚本层。
- `agents/openai.yaml` 只提供与 Skill 名称、描述和默认调用一致的界面元数据，保持默认自动发现；有副作用的操作由 Skill 正文在执行前请求授权，而不是通过关闭发现来隐藏。
- Skill 查找 CLI 的顺序为：`DOUBAO_THEME_CLI`、`PATH` 中的 `doubao-theme`、已安装“豆包工作主题”App 的 `Contents/Resources/bin/doubao-theme`。找不到时停止并给出安装/构建指令。
- `scripts/build-macos.sh` 构建对应架构的 CLI，通用包用 `lipo` 合并，并把 CLI 与 Skill 复制进 App Resources 后再签名。

### 豆包工作兼容边界

- 2026-08-29 本机豆包工作版本为 `2.26.7`。产品界面存在技能和自定义流程能力，但官方产品页没有公开外部 `SKILL.md` 包的导入格式、安装目录或随包本机命令权限；本机 App bundle 与配置目录也没有发现公开的 `SKILL.md` 入口。
- 因此本变更交付标准 Agent Skill，并保证其不依赖 Codex 私有提示。若豆包工作当前或后续允许导入该目录并执行本机命令，可直接使用；在获得可复现安装证据前，文档不得宣称“已原生安装到豆包工作”。

## Security and privacy

- CLI 与 CDP 只连接 `127.0.0.1`，不新增公网监听、远程上报或模型请求。
- 应用主题不读取页面文本、会话内容、Cookie、请求头、账号、附件、工具或工作区数据；运行结果只统计目标页与主题标记。
- 官方 `/Applications/DoubaoWork.app` 永不写入；离线 `build` 只调用现有克隆、补丁和签名路径。
- 安装继续使用现有 ZIP 大小、解压大小、文件数量、路径穿越、符号链接和完整性约束。
- `create` 和 `pack` 不覆盖既有目录/文件；用户必须选择新路径或自行删除旧产物。
- Skill 不保存凭据，不建议关闭系统安全功能，不把用户目录名写进主题元数据。
- 真实应用验证只使用独立、无私人内容的豆包工作窗口或会话；截图不得包含私人对话、账号或工作区内容。

## Alternatives and non-goals

- 不新增可视化主题编辑器、后台守护服务、LaunchAgent、自动更新器或主题商店发布命令。
- 不把 Node 画廊同步逻辑搬进 Rust；CLI 预览只服务本地作者和安装包，线上目录仍由现有 `pnpm --dir apps/web sync` 生成。
- 不改变桌面应用的主题选择界面，不批量迁移现有主题。
- 不复制或改写豆包工作官方资源，不逆向定义未公开的 Skill 安装协议。
- 不把两个 Skill 合并为一个全能 Skill：生产与有副作用的应用保持独立触发边界。
- 不引入 Clap、JSON Schema 运行时或第二套 CSS 解析依赖；当前契约使用现有 serde 数据和小型确定性检查完成。

## Areas of concern

- 当前 `Theme::load` 对旧版字符串背景缺失会静默忽略；严格作者检查必须报错，但兼容加载行为是否调整需要以不破坏旧主题为准。
- 恢复外观必须区分页面原有属性与主题运行时写入属性，避免把用户原本的深浅色选择错误清空。
- `--watch` 与其他主题应用进程并存时会重新写入主题；恢复命令需要检测或明确报告仍有 watcher 的情况，不能假装永久恢复。
- App Resources 内的 CLI 路径带中文与空格，Skill 必须按完整参数执行，不能拼接未引用的 shell 字符串。
- 通用二进制打包、签名和最低 macOS 版本必须与 GUI 保持一致。
- 主题预览是合成图，不等于真实豆包工作验收；实际应用仍需 DOM、计算样式和截图证据。

## Acceptance criteria

- [ ] `cargo run -p skin-core --bin doubao-theme -- create ...` 在临时目录生成完整 v2 主题、CSS 和 1200 × 675 预览，且不会覆盖非空目录。
- [ ] `check` 接受生成主题；对目录/ID 不一致、缺失素材、错误 CSS scope、缺失必需变量和不完整 `both` variants 返回退出码 `3` 与明确诊断。
- [ ] `preview` 可重复生成预览；`pack` 生成可由现有 `install_theme_package` 安装的包，且不会包含未引用文件、隐藏文件或符号链接。
- [ ] `list`、`create`、`check`、`preview`、`pack`、`install`、`apply`、`restore`、`build`、`remove-build` 的文本帮助、JSON 结果和退出码有 CLI 集成测试。
- [ ] v1 主题仍可由桌面应用和 CLI 列出、安装及应用；v2 作者命令不会放宽当前标准。
- [ ] 在线应用在真实豆包工作独立窗口中至少命中一个响应页面；DOM `dataset.skin` 与关键计算样式匹配所选主题，并有无私人内容截图。
- [ ] 在线恢复后相同页面不再有 `data-skin`、`#doubao-skin-style`、`#doubao-skin-backdrop` 或主题标记，原有外观属性恢复。
- [ ] 两个 Skill 通过 `quick_validate.py`，在隔离临时目录完成一次自然语言主题生产 dry-run；应用 Skill 的只读流程可执行，所有有副作用动作在授权前停止。
- [ ] README 与网站安装说明使用两个真实仓库路径和 `$skill-installer`，Skill 名称可通过 `$` 或技能选择器发现；GitHub 尚未公开时远程安装验收明确标为外部待验证。
- [ ] macOS host 包含可执行 CLI 与两个 Skill；通用包构建逻辑对 GUI 和 CLI 都执行 `lipo`，签名验证覆盖完整 App bundle。
- [ ] `./scripts/check.sh workflow`、`./scripts/check.sh rust` 及针对打包脚本的语法/host smoke check 通过，命令与证据记录在 `verification.md`。

## Decision

等待产品与风险审阅。接受本 Spec 只授权生成并审阅实施 Plan；在 Plan 被人工接受前不修改产品代码、Skill 或打包逻辑，也不对真实豆包工作执行应用。
