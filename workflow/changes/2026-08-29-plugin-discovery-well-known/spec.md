---
id: "2026-08-29-plugin-discovery-well-known"
stage: spec
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
based_on: intent.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Spec: 标准插件分发与 Skill Well-Known 发现

## Requirements

1. 将现有 `create-doubao-theme` 与 `apply-doubao-theme` 作为一个 `doubao-skin` 插件发布，插件版本初始与 workspace 版本 `0.1.0` 一致。两个 Skill 的职责、CLI 查找顺序和副作用授权门保持不变。
2. 插件根固定为 `plugins/doubao-skin/`，其中只包含插件清单、两个 Skill、必要展示素材和许可证说明；不得把整个仓库作为插件安装包复制进宿主缓存。
3. `plugins/doubao-skin/.codex-plugin/plugin.json` 遵循当前 Codex Plugin 清单，至少包含名称、严格 semver 版本、描述、作者、主页、仓库、许可证、关键词、`./skills/` 和完整 `interface` 展示元数据；不声明不存在的 App、MCP 或 Hook。
4. `plugins/doubao-skin/.claude-plugin/plugin.json` 遵循当前 Claude Code Plugin 清单，使用相同名称、版本、描述、作者、主页、仓库、许可证、关键词与 `./skills/`；Claude 专属字段仅在当前插件真实使用时出现。
5. 仓库级 `.agents/plugins/marketplace.json` 提供 Codex Marketplace `doubao-skin`，插件源为 `./plugins/doubao-skin`，安装策略为 `AVAILABLE`、认证时机为 `ON_INSTALL`，不设置未请求的产品限制。
6. 仓库级 `.claude-plugin/marketplace.json` 提供 Claude Code Marketplace `doubao-skin`，插件源同为 `./plugins/doubao-skin`。使用 GitHub 仓库或 Git URL 添加 Marketplace 时必须可解析该相对路径。
7. 原 `skills/create-doubao-theme/` 与 `skills/apply-doubao-theme/` 迁移到插件根的 `skills/` 下，成为唯一权威源；README、网站、文档和 macOS 包构建引用全部改到新路径，不保留复制品或跨插件根符号链接。
8. 新增确定性 Skill 发现同步，读取两个权威 `SKILL.md`，在 `apps/web/public/.well-known/agent-skills/` 生成 Draft 0.2.0 `index.json` 与两个原字节 `SKILL.md` 产物。索引只列这两个 Skill，条目使用 `type: "skill-md"`、与 frontmatter 相同的名称/描述、站内 URL 和 `sha256:<64 lowercase hex>`。
9. 网站同步命令同时刷新主题目录和 Skill 发现产物；检查模式必须检测缺失、陈旧摘要、错误 URL、错误摘要值、额外 Skill 或手工修改的公开副本，失败时不得静默重写后报告通过。
10. README 与“使用与下载”页提供经过验证的安装、更新、卸载和调用说明。Codex 使用 `codex plugin marketplace add IchenDEV/doubao-skin` 与 `codex plugin add doubao-skin@doubao-skin`；Claude Code 使用同仓库 Marketplace 和 `claude plugin install doubao-skin@doubao-skin`。文档说明插件只提供 Agent 工作流，主题 CLI/桌面 App 当前仍为 macOS 能力，Windows 标为 Coming Soon。
11. 页面可公开链接 `https://doubao-skin.idevlab.dev/.well-known/agent-skills/index.json`，但必须称其为 Agent Skills Discovery Draft 0.2.0；Codex 与 Claude Code 的正式安装入口仍是各自 Marketplace，不把 Well-Known 写成两者已支持的安装协议。

## User experience

- 用户在 Codex 或 Claude Code 中只需先添加一次 GitHub Marketplace，再安装一个“豆包主题”插件；安装完成后可获得“创作豆包主题”和“应用豆包主题”两项能力，不需要分别粘贴两个 Skill 路径。
- Codex 中以插件名 `doubao-skin` 展示，提供中文名称、简短说明、项目网站、GitHub 仓库和最多三条真实可执行的入门提示；不展示 App、MCP、联网账户或认证能力。
- Claude Code 中以同一插件名和版本展示，两个 Skill 以插件命名空间发现；文档使用当前 Claude Code 命令，不把 OpenAI 的 `$skill-installer` 写成 Claude Code 安装方式。
- 安装插件不会执行主题命令。只读的主题列出/检查可按 Skill 现有规则进行；安装、应用、恢复和离线构建仍须在当前会话中说明目标、影响并等待明确确认。
- 如果本机没有 `doubao-theme`，Skill 停止并指向 macOS App 或源码构建说明。Windows 用户可以先收藏或索引插件，但页面明确显示 CLI 与桌面端“Windows 即将支持”，不暗示当前可完成主题应用。
- 使用页在现有信息层级内增加一个紧凑的“安装 Agent Skill”区块，不把命令堆到首页，不添加遮挡主题浏览的浮层、弹窗或大段协议说明；深色模式、键盘访问和窄屏命令换行保持可用。

## Technical design

### Canonical package layout

```text
.agents/plugins/marketplace.json
.claude-plugin/marketplace.json
plugins/doubao-skin/
  .codex-plugin/plugin.json
  .claude-plugin/plugin.json
  skills/
    create-doubao-theme/
      SKILL.md
      agents/openai.yaml
    apply-doubao-theme/
      SKILL.md
      agents/openai.yaml
```

Marketplace 是仓库级索引，`plugins/doubao-skin/` 是宿主复制到缓存的最小插件边界。2026-08-29 使用 Codex CLI `0.150.1` 的隔离 `CODEX_HOME` 验证表明仓库根也可作为本地插件源，但会把整个仓库复制到插件缓存，因此明确不采用该方案。

### Metadata and version contract

- `Cargo.toml` 的 workspace `version` 是发布版本事实来源；两份 `plugin.json` 必须与其一致，Marketplace 不重复固定版本，避免三处独立升级。
- 两份清单共享 `name: "doubao-skin"`、作者 `IchenDEV`、MIT、项目网站与 GitHub URL。Codex 清单保留其 `interface`；Claude 清单只使用 Claude schema 支持的通用元数据。
- Codex `interface` 只声明由两个 Skill 真实提供的交互/本机写入能力，图标必须是插件目录内的已授权 PNG；不引用 `apps/web/public` 或插件目录外文件。
- Marketplace 名称与插件名称均为 `doubao-skin`，相对源路径固定为 `./plugins/doubao-skin`。安装说明不得依赖本机绝对路径或个人 Marketplace。

### Well-Known generation

- 增加一个小型 Node 脚本，从 `plugins/doubao-skin/skills/*/SKILL.md` 读取 YAML frontmatter、复制原字节并计算 SHA-256；不引入新的运行时依赖，不用正则重写 Skill 正文。
- 输出为：
  - `apps/web/public/.well-known/agent-skills/index.json`
  - `apps/web/public/.well-known/agent-skills/create-doubao-theme/SKILL.md`
  - `apps/web/public/.well-known/agent-skills/apply-doubao-theme/SKILL.md`
- `index.json` 使用 `$schema: "https://schemas.agentskills.io/discovery/0.2.0/schema.json"`。URL 使用 `/.well-known/agent-skills/<name>/SKILL.md`；digest 对应 HTTP 返回文件的原始字节。
- 公开副本是可提交的生成产物，禁止手工编辑。`sync` 负责刷新；测试/检查以权威 Skill 重新计算并比较，确保 Vercel 不运行同步时仍部署已经核验的静态文件。
- 静态 `.json` 和 `.md` 由 Next/Vercel `public` 目录直接提供，不新增 API、数据库、鉴权、Cookie 或动态服务器函数。

### Packaging and documentation integration

- `scripts/build-macos.sh` 从 `plugins/doubao-skin/skills/` 复制两个 Skill 到 App Resources；App 内继续只携带 Skill，不额外嵌入 Marketplace，也不改变签名范围之外的行为。
- README、`docs/submitting-themes.md`、`apps/web/src/lib/site.ts` 和 `apps/web/src/app/guide/page.tsx` 的旧 GitHub Skill 路径统一替换为插件安装方式；投稿页只在需要处链接安装说明，不复制完整命令表。
- 网站元数据与 sitemap 不为每个静态 Skill 文件制造可浏览页面；Well-Known 供机器发现，面向人的规范和安装说明保留在使用页。

## Security and privacy

- 插件只包含声明的 Skill、展示素材和许可证，不包含主题包、应用二进制、凭据、构建缓存、对话、Cookie、账号、工作区数据或官方应用资源。
- Codex 与 Claude Code Marketplace 插件会被复制到本机缓存；因此所有 Skill 路径必须留在插件根内，不使用 `../`、外部符号链接或安装后失效的仓库相对路径。
- 插件清单不声明 MCP、App、Hook、Agent、后台监控或自动安装脚本；安装本身不执行代码，也不扩大两个 Skill 已有权限。
- Well-Known 内容全部公开，不包含认证信息。公开文件必须使用 HTTPS、正确 Content-Type，并允许普通 GET/HEAD；无需启用宽泛 CORS 凭据或动态写入。
- 同步检查对 Skill 名称执行 Agent Skills 命名约束，对 URL 固定为站内路径，对 digest 使用原始字节 SHA-256；发现未知目录、重复名称、非法 frontmatter 或摘要不一致时失败。
- 文档提醒用户插件属于高信任本机扩展，只应从 `IchenDEV/doubao-skin` 官方仓库添加；不建议关闭宿主安全策略或清理全局缓存。

## Alternatives and non-goals

- 不以仓库根作为插件根：虽然当前 Codex 本地验证可安装，但会把整个代码库和主题素材复制到缓存，扩大体积与供应链暴露面。
- 不保留根级 `skills/` 与插件内 `skills/` 两份副本，也不依赖符号链接；安装缓存和跨平台 ZIP 对符号链接处理不一致。
- 不只提供 `.claude-plugin/marketplace.json` 让 Codex 走 legacy 兼容路径；两边各用当前原生 Marketplace，清单差异显式可审。
- 不把 Well-Known `index.json` 当作插件 Marketplace，不新增私有 `/.well-known/codex-plugin.json` 或 `/.well-known/claude-plugin.json`。
- 不为两个纯 `SKILL.md` 产物生成 ZIP/TAR；`agents/openai.yaml` 是 Codex 展示元数据，不是 Skill 执行所需资源，完整安装由插件 Marketplace 负责。
- 不安装 Claude Code 到用户全局环境，不提交公共插件目录，不推送 GitHub，不创建 Release，不执行生产部署。
- 不修改 Rust CLI、主题运行时、主题内容、在线目录数据库或桌面界面。

## Areas of concern

- 本机当前有 Codex CLI `0.150.1`，但没有 `claude` 可执行文件。Claude 清单和 Marketplace 可依据当日官方 schema 设计；最终 `claude plugin validate` 与隔离安装证据必须在可用的临时 CLI 环境完成，否则 Claude 运行验收标为 blocked，不能声称已实装验证。
- `plugin-creator` 的默认脚手架假设插件位于 `plugins/<name>`，与本 Spec 一致；项目已有 Skill 迁移会影响 macOS 打包和旧文档路径，所有调用点必须通过 `rg` 与包内容检查收口。
- 插件版本若只改清单而未改 workspace 版本，Claude Code 可能因缓存键不变而跳过更新；一致性检查必须在发布前阻止该状态。
- GitHub Marketplace 相对路径只在通过 Git/GitHub 添加仓库时可靠；Claude Code 直接添加远程 `marketplace.json` URL 时相对插件源无法解析，因此公开说明只推荐 GitHub 仓库安装。
- `public/.well-known` 是点目录，需要验证 Next 开发服务器、生产构建和 Vercel 预览均实际返回文件，而不仅是文件存在于源码。
- Skill frontmatter description 为英文，网站安装说明为中文；索引必须原样使用权威 frontmatter，不为了 SEO/GEO 另写会漂移的摘要。
- 现有工作树包含并行网页与主题改动；移动 Skill 和编辑共享文档时若发现并发变化，必须保留现有内容并只改相关段落。

## Acceptance criteria

- [ ] `plugins/doubao-skin/` 是唯一插件根且只含两项 Skill、两份清单、必要素材/许可证；仓库根不再有旧 `skills/create-doubao-theme` 或 `skills/apply-doubao-theme` 副本。
- [ ] Codex 插件通过 `plugin-creator` 的 `validate_plugin.py`；两个 Skill 分别通过 `quick_validate.py`，名称、版本、路径、interface 与 Marketplace 策略符合当前 schema。
- [ ] 使用隔离临时 `CODEX_HOME` 从仓库级 Marketplace 完成 `marketplace add → plugin list → plugin add → plugin remove`；缓存只包含 `plugins/doubao-skin/` 的内容，两个 Skill 均可被发现。
- [ ] `claude plugin validate plugins/doubao-skin` 和 `claude plugin validate .` 通过；在隔离配置中从本地或临时 Git Marketplace 安装后能列出 `doubao-skin` 及两个命名空间 Skill。若执行环境没有 Claude Code，该项明确 blocked，Spec 不允许以 JSON 解析代替最终证据。
- [ ] Node 同步测试证明 Well-Known 索引只有两个条目，`$schema`、名称、类型、描述、URL 和 digest 精确匹配权威 `SKILL.md`；篡改公开副本或 digest 时检查确定性失败。
- [ ] 本地生产构建后，GET/HEAD `/.well-known/agent-skills/index.json` 与两个 `SKILL.md` 返回 200、预期 Content-Type 和原始字节；重新计算 SHA-256 与索引一致。
- [ ] README 和使用页的 Codex/Claude Code 安装、更新、卸载命令与当前 CLI help/官方文档一致；明确 macOS 前置条件、Windows Coming Soon、Well-Known Draft 状态和副作用授权边界。
- [ ] 使用页在正常与窄屏宽度、浅色与深色模式下完成浏览器视觉检查；命令可复制、不会横向撑破页面，首页主题浏览体验不发生变化。
- [ ] macOS host 包内仍有 `Contents/Resources/skills/create-doubao-theme/SKILL.md` 与 `apply-doubao-theme/SKILL.md`，且内容与插件权威源一致；不要求重跑无关 Rust 测试，因为 Rust 行为未改。
- [ ] `./scripts/check.sh workflow`、插件/Skill 校验、Well-Known 单测和 `pnpm --dir apps/web sync && ./scripts/check.sh web` 通过；命令、版本、隔离目录、HTTP/视觉证据和残余风险写入 `verification.md`。

## Decision

等待产品所有者接受、要求修改或拒绝本 Spec。接受只授权生成实施 Plan；在 Plan 被明确接受前不迁移 Skill、不创建插件清单、不修改网站、不安装宿主插件，也不执行 GitHub 或生产发布。
