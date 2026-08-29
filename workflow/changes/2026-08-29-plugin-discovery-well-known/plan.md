---
id: "2026-08-29-plugin-discovery-well-known"
stage: plan
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Plan: standard plugin discovery and well-known skills

## Files and ownership

- `.agents/plugins/marketplace.json`（新）：仓库级 Codex Marketplace，只收录 `doubao-skin`，源为 `./plugins/doubao-skin`。
- `.claude-plugin/marketplace.json`（新）：仓库级 Claude Code Marketplace，使用同一插件源。
- `plugins/doubao-skin/.codex-plugin/plugin.json`、`plugins/doubao-skin/.claude-plugin/plugin.json`（新）：两套宿主清单；名称、版本、作者、仓库和许可证保持一致，Codex 清单额外包含其正式 `interface`。
- `plugins/doubao-skin/skills/create-doubao-theme/`、`plugins/doubao-skin/skills/apply-doubao-theme/`：从根级 `skills/` 机械迁移的唯一权威 Skill，不改其执行语义；保留各自 `agents/openai.yaml`。
- `plugins/doubao-skin/assets/`（仅在 Codex manifest 校验要求展示素材时新增）：复用已有、权利清楚的项目图标副本，不生成新的品牌视觉。
- `apps/web/scripts/sync-skills.mjs`（新）：从插件权威 Skill 生成/检查 Well-Known Draft 0.2.0 产物，支持普通同步和只读 `--check`。
- `apps/web/scripts/skill-discovery.test.mjs`（新）：覆盖双清单一致性、Marketplace 路径、Skill frontmatter、index/digest 与篡改失败。
- `apps/web/public/.well-known/agent-skills/**`（生成）：两个原字节 `SKILL.md` 和一个 `index.json`；禁止手改。
- `apps/web/package.json`、`scripts/check.sh`：把 Skill 同步/测试接入现有 web gate，只增加与本变更直接相关的命令。
- `scripts/build-macos.sh`：将包内 Skill 来源切换到 `plugins/doubao-skin/skills/`，保持 App Resources 目标路径不变。
- `README.md`、`docs/submitting-themes.md`、`apps/web/src/lib/site.ts`、`apps/web/src/app/guide/page.tsx`：替换旧的单 Skill 安装路径，增加 Codex/Claude Code 插件安装、更新/卸载、macOS 前置条件、Windows Coming Soon 和 Well-Known Draft 链接。
- `workflow/changes/2026-08-29-plugin-discovery-well-known/verification.md`：记录实际版本、命令、隔离宿主安装、HTTP、视觉与包内容证据。
- 不拥有或修改 `themes/**`、`apps/web/data/**`、`apps/web/public/themes/**`、Rust 源码、桌面 UI、Vercel 生产配置或无关并行改动。

## Order of work

1. **冻结路径与失败条件**
   - 用 `rg` 重新枚举所有根级 `skills/` 调用方、App 打包来源、README/网站旧安装文案和现有并发修改。
   - 先添加 `skill-discovery.test.mjs`，断言目标插件目录、双清单、两个 Marketplace、workspace 版本一致性、Well-Known schema/URL/digest 和旧路径消失；确认测试在实现前因目标文件缺失而失败。
2. **建立最小双协议插件包**
   - 按 `plugin-creator` 的 `plugins/<name>` 形状创建 `plugins/doubao-skin/`，机械迁移两个 Skill，随后添加 Codex 与 Claude 清单。
   - 先运行 `quick_validate.py` 检查两个迁移后的 Skill，再用 `validate_plugin.py` 迭代 Codex manifest；不添加 App、MCP、Hook、Agent、监控或安装脚本。
   - 创建两份仓库 Marketplace，固定插件源为 `./plugins/doubao-skin`；运行 JSON 解析和测试，确保外层目录、插件名与 manifest 名一致。
3. **实现 Well-Known 单一来源同步**
   - 新增零依赖 Node 脚本，严格读取两个已知 Skill 的 frontmatter、复制原始字节并生成排序稳定的 Draft 0.2.0 index；普通模式原子写入，`--check` 只比较且在缺失、额外或不一致时失败。
   - 先让测试覆盖合法生成，再在隔离临时目录篡改摘要、digest、公开副本和额外目录，证明检查确实失败；随后接入 `pnpm sync` 与 web check。
4. **迁移打包与人类安装入口**
   - 修改 `scripts/build-macos.sh` 的源路径并保留 App 内既有 `Contents/Resources/skills/<name>` 结构；用 shell 语法检查和 host 包内容 smoke 防止迁移漏包。
   - 更新 README、投稿文档、站点常量和使用页。页面只增加紧凑的 Agent Skill 安装区块，提供两套可复制命令、能力说明、更新/卸载、信任提示和 Windows Coming Soon；首页不加安装命令或协议说明。
5. **隔离验证两套宿主**
   - Codex 使用临时 `CODEX_HOME`，从当前仓库添加 Marketplace，依次执行 list/add/remove 并检查缓存只复制插件子目录；不触碰用户个人 Marketplace、插件开关或缓存。
   - Claude Code 优先使用本机现有 CLI；若仍不存在，只在临时缓存/配置下运行官方临时 CLI，不做全局安装，并执行 `plugin validate`、Marketplace add/install/list/uninstall。若官方 CLI 无法取得或需要无法提供的登录，记录 blocked，不以手写 schema 检查冒充运行证据。
6. **网页、HTTP 与视觉验收**
   - 执行同步和 web gate，启动生产构建，在本机 GET/HEAD 三个 Well-Known 地址，核对状态、Content-Type、响应字节和 SHA-256。
   - 在用户浏览器中检查使用页正常/窄屏与浅色/深色四种组合，验证命令可复制、无横向溢出、导航与首页主题浏览无回归；截图只包含公开页面。
7. **收口与独立复核**
   - 运行 workflow gate，记录所有实际检查和未覆盖项到 Verification。
   - 交给新上下文或人工 verifier 复核清单、路径、安装缓存、Well-Known digest 和页面证据。若使用 GPT-5.6 Sol，明确要求它只运行下面的最小适用检查，因为本变更未触及 Rust/主题运行时，重复全仓 Rust 测试不会提高该变更的置信度。

## Test-first proof

- 首个失败测试检查：`plugins/doubao-skin` 尚不存在、根级 Skill 尚未迁移、Marketplace/manifest/Well-Known 产物缺失；失败原因必须与本变更一致，而不是环境故障。
- 双清单测试解析两份 JSON，断言 `name/version/description/author/homepage/repository/license/keywords/skills` 一致；版本与 `Cargo.toml` workspace version 一致，Codex `interface` 必填字段完整且只引用插件内部现有资源。
- Marketplace 测试断言两个 catalog 均只有一个 `doubao-skin` 条目、源精确为 `./plugins/doubao-skin`；Codex policy/category 完整，Claude 不重复组件定义。
- Skill 测试断言插件下恰好存在两个 Skill、frontmatter 名称与目录一致、描述非空、根级旧路径不存在；官方 `quick_validate.py` 作为独立 schema oracle。
- Well-Known 测试断言 `$schema`、条目数量、排序、`type: skill-md`、description、URL 和 `sha256(raw SKILL.md bytes)`；测试在临时副本中分别制造错误 digest、错误描述、篡改公开文件和额外 Skill，`--check` 必须非零退出。
- 插件宿主测试使用完全隔离的配置目录，验证 marketplace → install → cache → remove 完整链，而不是只运行 JSON parser；Codex 缓存不得出现仓库 `themes/`、`crates/` 或 `apps/`。
- 只执行与变更相关的自动化：plugin/skill validators、Node tests、web gate、workflow gate、`bash -n scripts/build-macos.sh` 与 host 包内容 smoke。不运行 `./scripts/check.sh rust` 或 `./scripts/check.sh all`，原因是没有 Rust 源码、主题行为或桌面运行时变化。

## Visual or integration proof

- Codex CLI 记录版本、临时 `CODEX_HOME`、Marketplace 解析路径、安装后的插件根与两个 Skill；卸载后临时配置不再启用该插件。
- Claude Code 记录实际 CLI 版本、`plugin validate` 输出、临时 Marketplace/插件列表和两个命名空间 Skill。CLI 不可用时保留明确 blocked 证据，不将官方文档推断写成通过。
- 生产 Next 服务对 `index.json`、`create-doubao-theme/SKILL.md`、`apply-doubao-theme/SKILL.md` 的 GET 与 HEAD 均返回成功；下载字节重新计算的 digest 与 index 完全一致。
- 使用页在桌面与窄屏、浅色与深色下截图；检查命令行容器换行/横向滚动、复制按钮、焦点状态、段落层级和 Windows Coming Soon。首页只做无变化烟测。
- host App 包检查两个目标 Skill 的 `SKILL.md` 与 `agents/openai.yaml` 均存在，并与 `plugins/doubao-skin/skills/` 原文件逐字一致；不重新运行真实豆包工作主题应用，因为执行语义未改。
- 如果准备 Vercel preview，只验证预览 URL 的 Well-Known 与使用页；本 Plan 不授权生产部署、域名切换、GitHub push 或公共插件目录提交。

## Risks and mitigations

- **迁移后旧路径失效**：先用 `rg` 建立调用方清单，机械移动后更新全部真实引用；测试强制旧路径消失且 App 包仍有相同目标文件。
- **两套 manifest 漂移**：共享字段与 workspace version 由一个 Node 测试统一比较；供应商专属字段只留在各自清单。
- **安装复制过多仓库内容**：Marketplace 始终指向 `plugins/doubao-skin`；隔离 Codex 缓存检查明确拒绝 `themes/`、`apps/`、`crates/`。
- **Well-Known 输出陈旧**：同步负责生成，`--check` 在 CI/web gate 只读比较；digest 按最终公开原字节计算，不经过换行或编码归一化。
- **Claude 兼容性未实测**：使用当前官方 CLI 做验证；本机缺失不触发全局安装，无法取得时把 Claude 运行项标为 blocked，保留 Codex 与静态 schema 已验证边界。
- **公开文案误称标准**：安装区把 Marketplace 称为正式宿主入口，把 Well-Known 明示为 Draft 0.2.0；不出现“官方已支持自动安装”的表述。
- **页面信息过载**：安装命令只放使用页，采用现有 CommandRow/CopyButton 组件和现有栅格；正常与窄屏视觉验收阻止横向溢出。
- **并发脏工作树覆盖**：只触碰 Files and ownership 列出的文件；共享 README/guide/build script 在写前后检查 diff，保留不相关改动并避免全文件格式化。

## Rollback

- 在未发布状态下，删除新增 Marketplace、plugin manifests、Well-Known 生成器/测试与生成产物，把两个 Skill 机械移回原 `skills/`，并恢复文档、站点常量和打包脚本的相关路径；不触碰主题或其他网页改动。
- 若 Codex 或 Claude 的一个清单失败，不删除另一套已验证实现来掩盖问题；停止发布并修正失败宿主，或把整个双协议变更回滚到裸 Skill 状态。
- 若 Well-Known 路由在生产构建不可访问，移除公开链接与生成接线，保留插件 Marketplace；不得发布 digest 不一致的 index。
- 若使用页出现布局回归，恢复该区块并保留 README 安装说明继续诊断；首页和主题目录无需回滚。
- 本 Plan 不执行生产发布，因此没有线上回滚动作；后续部署仍需独立生产 Gate。

## Deviations

- 路径冻结时发现 `apps/web/src/app/contribute/page.tsx` 也是旧 `SKILL_INSTALL_PROMPT` 的真实调用方。实现将该页一并切换到新 Codex 插件命令，避免创作者入口保留失效安装方式；未改变投稿行为或页面结构。
- 当前 Codex validator 未要求计划外资产；插件保持无 App/MCP/Hook/Agent 的最小边界。
- 若 Claude Code 当前版本不接受 Marketplace 与插件都位于同仓库的相对路径，优先使用其正式 `git-subdir`/GitHub source 表达 `plugins/doubao-skin`，并同步 Spec/Plan 后重新请求方向；不复制第二个插件仓库。
- 若 `agents/openai.yaml` 使 Well-Known 消费者必须使用 archive 才能获得功能，经实际验证后可把对应条目改为 archive；在没有证据前保持最小 `skill-md`。
- 若 App host 打包因工作树其他并发构建问题无法完成，仅可把包 smoke 标为 blocked；不能用 `bash -n` 代替实际包内容证据。

## Decision

等待产品所有者接受、要求修改或拒绝本 Plan。接受只授权按上述文件范围实施、运行隔离验证并准备预览证据；不授权修改个人插件环境、全局安装 Claude Code、推送 GitHub、创建 Release、提交公共插件目录或生产部署。
