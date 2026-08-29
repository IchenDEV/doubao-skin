---
id: "2026-08-29-deploy-online-theme-store"
stage: plan
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
based_on: spec.md
risk: "high"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Plan: deploy online theme store

## Files and ownership

- `crates/skin-core/src/theme.rs`：新增默认商店地址回归测试并把默认 URL 切到正式域名；不改下载、校验和解压逻辑。
- `apps/web/src/app/page.tsx`、`apps/web/src/app/themes/[id]/page.tsx`：把“通用版”改成与现有包一致的“Apple 芯片版”；不改页面结构或视觉样式。
- `README.md`：把公开画廊链接和下载架构文案改为正式域名与 Apple 芯片版。
- `docs/website-deployment.md`：记录当前无 GitHub Release 时使用站内下载包、正式域名、Preview/Production 环境变量和自定义域名验收方式。
- `.github/workflows/health.yml`：把健康检查入口改为正式域名，并增加 schema v1 目录 JSON 检查。
- `apps/web/scripts/sync-themes.mjs`：经产品负责人追加授权，消除预览重写、ZIP 时间扩展字段和当前时间元数据造成的非确定性；不改变主题内容、目录 schema 或包结构。
- `apps/web/data/themes.db`、`apps/web/public/themes/**`：只允许 `pnpm sync` 机械生成；若与同步前候选快照存在差异，逐项确认其来源是当前 `themes/`，不得人工编辑。
- `workflow/changes/2026-08-29-deploy-online-theme-store/{intent,spec,plan,verification}.md`：记录审批、执行证据、部署与最终验收。
- 外部状态：既有 Vercel 项目 `doubao-skin-gallery` 的 Preview/Production 环境变量、部署与自定义域名；Cloudflare 中仅 `doubao-skin.idevlab.dev` 的精确 A 记录。
- 明确不拥有其他主题源、桌面功能、Rust 模块、Git 历史、macOS 安装包内容或并行变更文件。

## Order of work

1. 记录 owned files 的变更前内容摘要、当前 Git 状态、Vercel 项目 ID、最近生产部署 `dpl_GnYWwZzZ5go1pocdajuvaKsBA7D6`、现网目录 404 和 DNS 未配置状态，形成可回滚基线。
2. 先在 `theme.rs` 增加回归测试，断言默认 URL 必须是新域名，并在测试专用互斥保护下验证环境变量仍可覆盖；先运行该测试证明它因旧默认值失败。
3. 修改 Rust 默认值、两处网页下载文案、README、部署文档和健康检查；先运行最小 Rust 测试、TypeScript 检查和 YAML/文本检查。
4. 记录生成目录的文件清单与 SHA-256；修复生成器的稳定时间和 ZIP 元数据后运行 `pnpm --dir apps/web sync` 两次，确认第二次结果稳定、主题仍为 26 套、包仍为 26 个，并核验目录中每个包的大小与 SHA-256。
5. 使用符合 Node 24 要求的运行环境执行 `pnpm --dir apps/web check`，再运行 `cargo test -p skin-core`、`./scripts/check.sh rust` 和 `./scripts/check.sh workflow`。若本机只能提供 24.14，保留本地结果并用 Vercel Node 24.x 构建补证，不能隐去版本偏差。
6. 在既有 Vercel 项目中为 Preview 与 Production 设置同一个 `NEXT_PUBLIC_APP_DOWNLOAD_URL`，值为正式域名下的 arm64 ZIP；读取回查环境列表，确认没有错误的 `NEXT_PUBLIC_REPO_URL`。
7. 从 `apps/web` 显式执行 Preview 部署。检查 Ready、首页、主题详情、目录 JSON、缩略图、主题包和桌面 ZIP；目录或资源任一项失败则停止，不进入 Production。
8. 把自定义域名关联到同一 Vercel 项目，但暂不让 DNS 对外指向；确认 Vercel 返回的仍是精确 A 记录 `76.76.21.21`。
9. 从同一候选工作区显式执行 Production 部署，检查 Ready、部署 ID 和 `doubao-skin-gallery.vercel.app` 生产别名已切到新部署。只有旧 Vercel 域名上的完整资源验收通过，才继续 DNS。
10. 在 Cloudflare 读取 `doubao-skin` 现有记录；无记录则新增，有冲突则只更新这个主机为 `A 76.76.21.21` 并设为 DNS only。等待权威 DNS、Vercel 域名验证和 TLS 证书全部生效。
11. 通过正式域名重复 HTTP/JSON/SHA-256 验收，并在浏览器以正常和窄视口检查首页、筛选、主题详情和下载入口。
12. 清除本次测试进程的 `DOUBAO_SKIN_THEME_STORE_URL`，启动真实 GPUI 桌面应用；在正常和窄窗口打开线上主题商店，确认 26 套卡片与缩略图来自正式域名，并保存截图证据。
13. 生成 `verification.md`，写入命令、版本、部署 ID、DNS、证书、HTTP、目录校验、截图和残余风险。由新的 fresh-context verifier 复跑关键检查并记录 verdict；主实施代理不代替 verifier 宣告通过。

## Test-first proof

- 第一个新增 Rust 测试在实现前必须因 `DEFAULT_THEME_STORE_URL` 仍为旧域名而失败；失败原因必须精确，不接受编译错误或无关失败。
- 实现后同一测试通过，并验证 `DOUBAO_SKIN_THEME_STORE_URL` 覆盖值优先于默认值；测试恢复原环境，避免污染并行测试。
- 健康检查新增目录断言：HTTP 200、`schemaVersion == 1`、主题数组非空且包含 `violet-night`；这会在当前生产目录 404 的基线上失败，在新部署后通过。
- 同步采用“第一次允许更新、第二次必须稳定”的判据；目录每个 `packageUrl` 都映射到本地文件，`packageSize` 与文件大小相等，`sha256` 与实际摘要相等。
- 不为一次 URL 替换新增 Manager、Service、Provider 或新的配置层。

## Visual or integration proof

- Web Preview：浏览器检查桌面宽度与约 390px 窄视口，覆盖首页、分类筛选、`/themes/violet-night`、下载按钮及无横向溢出；保存至少一张正常和一张窄视口截图。
- Web Production：正式域名完成相同关键路径抽查，并读取浏览器控制台错误；HTTP 工具同时验证内容类型、状态码和静态资源。
- 桌面应用：使用真实 GPUI 窗口，不设置商店覆盖变量，打开线上商店后确认主题数量、首屏缩略图、滚动与一个远程主题安装入口；正常和窄窗口各留截图。组件测试或只验证端口不算通过。
- 域名：分别检查 Cloudflare 权威 DNS、Vercel 项目域名状态和浏览器 HTTPS 锁定结果；单独看到 A 记录或单独看到 Vercel Ready 都不足以验收。

## Risks and mitigations

- **脏工作树与未跟踪迁移**：所有人工编辑限制在 owned files；操作前后使用 scoped diff、文件摘要和生成清单，绝不重置或提交其他改动。
- **部署错误项目**：每次部署前读取 `apps/web/.vercel/project.json` 并用 `vercel project inspect` 校验项目 ID；命令只在 `apps/web` 执行且显式指定 Preview/Production。
- **Preview/Production 环境变量漏配**：部署前分别回查环境列表；构建若触发 `site.ts` 缺少变量错误则停止。
- **静态目录再次被动态路由吞掉**：Preview 阶段必须直接解析 `/themes/catalog.json`，并验证一个相对 `packageUrl` 和 `thumbnailUrl`；不以首页 200 代替。
- **域名短暂指向旧生产**：先完成新 Production 验收，再创建 DNS；把外部可见的错误窗口压到 DNS 传播时间以内。
- **Cloudflare 代理干扰**：使用 DNS only；若控制台现有同名记录，先读后改，不创建 A/CNAME 冲突。
- **安装包架构误导**：只改文案为 Apple 芯片版，不承诺 Intel；真实 ZIP 用 Mach-O 检查确认 arm64。
- **本地 Node 版本偏低**：优先使用可用的 Node 24.20+；若无法无侵入获得，则记录本地 24.14 偏差并以 Vercel 24.x 构建和运行时验收补证。
- **Fresh-context verdict 与未提交基线**：本变更不创建 Git commit。验证工件记录原 HEAD、owned-files SHA-256、Vercel 部署 ID 和脏工作区事实，verifier 以这些不可混淆的快照边界复核。

## Rollback

- 产品代码：用反向 `apply_patch` 恢复 owned files 的原值；重新运行最小测试确认旧行为恢复。不得使用 `git reset --hard` 或 `git checkout --`，避免清除用户改动。
- Vercel：若新部署有问题，把生产别名重新指向基线部署 `dpl_GnYWwZzZ5go1pocdajuvaKsBA7D6`；保留失败部署供诊断。若需要删除域名或环境变量，因属于云端删除操作，执行前再次取得用户确认。
- DNS：若正式域名导致用户影响，将精确 A 记录恢复为变更前状态；若变更前无记录，删除该记录前再次取得用户确认。根域和其他记录始终不动。
- 桌面应用：紧急情况下可用 `DOUBAO_SKIN_THEME_STORE_URL=https://doubao-skin-gallery.vercel.app/themes/catalog.json` 临时覆盖，但只有旧目录恢复为 200 时才可使用；否则保持新默认并修复线上部署。

## Deviations

- 仓库文档推荐 GitHub 集成，但当前仓库没有 Git remote、对应 GitHub 仓库或 Release；按已接受 Spec，本次使用已链接的本地 `apps/web` 通过 Vercel CLI 发布，不创建远程仓库。
- 当前所有新版 `apps/web`、`crates/` 与 workflow 文件均属于未提交迁移的一部分；本次不创建 Git commit。验证以 HEAD `7ccc5a5` 加 scoped 文件摘要和部署 ID 标识候选快照。
- 现有桌面包仅 arm64，因而把 UI 文案从“通用版”纠正为“Apple 芯片版”；不在本次构建 universal 包。
- 首次实施同步发现 ZIP 时间字段、SQLite `built_at` 与目录 `generatedAt` 使相同输入每次产生不同摘要。产品负责人随后明确回复“确认生成器修复”，授权把 `apps/web/scripts/sync-themes.mjs` 加入范围；修复仅保证相同主题源连续同步可复现。
- Vercel 当前自动安装器不完整支持项目固定的 pnpm 12：首次 Preview 生成了不可用的 pnpm 包装器，启用 Corepack 后默认安装器又追加 pnpm 12 不接受的 `--unsafe-perm`。为保持 `packageManager: pnpm@12.0.0`，已按 Vercel 支持的项目级构建配置设置 `ENABLE_EXPERIMENTAL_COREPACK=1`，并把 Install Command 改为 `corepack pnpm install --frozen-lockfile`；未改包管理器版本或依赖。
- Preview 部署启用了 Vercel Deployment Protection，普通匿名请求会进入登录页。Preview 的首页、详情、目录和静态资源因此通过 Vercel CLI 生成的短时保护绕过完成验收；Production 在公开 Vercel 别名和正式域名上重复完整验收，不把受保护 Preview 的匿名浏览器结果冒充公开访问证据。
- 除上述已知差异外，实施不得偏离 Spec。出现需要扩大文件范围、改变 DNS 架构、创建仓库或重建桌面包的情况时停止并重新请求产品决定。

## Decision

产品负责人已接受本 Plan，授权实施上述 scoped 产品修改、Vercel Preview/Production 环境变量与部署、自定义域名关联，以及 Cloudflare 中精确的 `doubao-skin` DNS 记录变更；不授权其他生产、仓库、发布或删除操作。
