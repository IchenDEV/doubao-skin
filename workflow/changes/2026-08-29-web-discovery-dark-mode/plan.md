---
id: "2026-08-29-web-discovery-dark-mode"
stage: plan
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
based_on: spec.md
risk: "high"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Plan: web discovery dark mode

## Files and ownership

- `apps/web/layout-spec.json`（新）：单侧栏外壳、内容网格、间距、断点、指南阅读列和允许例外的几何事实来源。
- `apps/web/src/lib/theme-filters.ts` 与测试（新）、`apps/web/src/components/GalleryClient.tsx`、`SiteHeader.tsx`：类型/系列两个独立维度、URL 兼容和唯一筛选 UI；不再使用自定义 window event 或内容区 tabs。
- `apps/web/src/app/page.tsx`、`ThemeCard.tsx`、`themes/[id]/page.tsx`：移除首页底部使用区，下载相关入口统一导向 `/guide#download`。
- `apps/web/src/app/guide/page.tsx`、`contribute/page.tsx`（新）：下载安装/Windows/Skill 说明与创作投稿摘要；保持服务端组件。
- `docs/submitting-themes.md`（新）、`README.md`、`docs/README.md`、`CONTRIBUTING.md`、`docs/website-deployment.md`、`apps/web/.env.example`：投稿、Skill 安装和固定 GitHub 下载契约。
- `apps/web/src/lib/site.ts`、`app/layout.tsx`、`app/robots.ts`、`app/sitemap.ts`、必要的窄 JSON-LD helper：站点常量、metadata、robots、动态 sitemap 和结构化数据。
- `apps/web/src/app/globals.css`：布局 token、组合筛选、指南页、平台状态和完整浅/深语义颜色；不更改主题 mockup 内部色板。
- `apps/web/package.json` 与必要的 `apps/web/scripts/check-site.mjs`：无新依赖的筛选回归和运行站点断言。
- `.github/workflows/health.yml`：三个顶层页面、metadata 端点和 GitHub universal ZIP 的健康边界。
- `workflow/changes/2026-08-29-web-discovery-dark-mode/verification.md`：构建、HTTP、视觉、交互、Preview/Production Gate 和外部待验证项。
- 不手工编辑 `apps/web/data` 或 `apps/web/public/themes`；Skill/CLI 文件由相邻变更先完成，本 Plan 只读取其真实存在状态并写网站说明。

## Order of work

1. **依赖确认与布局规格**
   - 在主题 CLI/Skill Plan 至少完成两个真实 Skill 目录和安装契约后开始网页实现；若尚未完成，网页状态保持“发布后可用”。
   - 先写 `apps/web/layout-spec.json`：桌面 `264px + minmax(0,1fr)`、内容最大约 1296px、指南阅读列约 760px、`48/32/20px` 边距和既有 919/699px 断点。记录移动端同一筛选区折叠规则。
2. **先写筛选回归，再修正交互模型**
   - 新增纯函数测试，覆盖 type/series AND 组合、搜索叠加、非法参数回退、旧 `view` 映射和结果数；确认当前单一 `view` 实现不能满足测试。
   - 提取最小 `theme-filters` 模块。`SiteHeader` 用 URL 作为筛选状态源并只在主题库显示两个筛选组；`GalleryClient` 删除 tabs 与自定义事件，只维护搜索并计算结果。
   - 更新 URL 不丢失另一维，浏览器前进/后退可恢复；搜索有明确可访问名称，筛选用原生控件和 44px 目标。
3. **拆分使用与投稿任务**
   - 创建 `/guide`，将唯一 macOS universal 下载按钮、Windows Coming Soon、首次打开、应用、恢复和两个 Skill 安装/调用说明放入一个阅读流程。
   - 创建 `/contribute` 和 `docs/submitting-themes.md`，明确无网页直传，提供创建 → 检查 → sync → Fork/分支 → PR 的短流程和完整清单。
   - 首页删除 `usage-section`；页头、列表菜单和详情安装区改为内部指南链接。三个顶层导航在所有路由可达。
   - Skill 目录存在且通过验证时写“已随主题工具链提供”；否则严格显示“发布后可用”。豆包工作兼容边界按已验证事实表述。
4. **固定 URL、SEO/GEO 与深色 token**
   - `site.ts` 固定生产域名、GitHub repo、投稿文档、Skill 路径和 universal ZIP，移除旧下载覆盖分支；同步环境示例与部署文档。
   - 使用 App Router 原生 metadata/robots/sitemap；三个顶层页面与 26 个详情均有 canonical、OG/Twitter。JSON-LD 只从可见文案和主题数据库生成并转义 `<`。
   - 将产品界面硬编码浅色归并为最小语义变量，在 `prefers-color-scheme: dark` 覆盖；加入 `color-scheme`、两组 `theme-color`、`:focus-visible` 和减少动态效果。主题预览变量保持隔离。
5. **确定性检查与真实浏览器 QA**
   - 连续两次 sync 并确认生成清单不变；运行筛选测试、TypeScript/Next build、audit 和 workflow gate。
   - 启动 production build，本地检查三个顶层页面、代表详情、robots、sitemap、canonical、OG/Twitter、JSON-LD、目录和主题包；检查 HTML 中无旧 arm64 URL。
   - 在真实浏览器测试宽屏和约 390px、浅色/深色、两维组合、搜索、后退/前进、窄屏披露、详情、外链和键盘焦点；记录截图、DOM、控制台与滚动宽度。
6. **Preview 与生产 Gate**
   - 移除 Vercel 旧 `NEXT_PUBLIC_APP_DOWNLOAD_URL` 或部署不读取它；先部署 Preview 并重复 HTTP/视觉 smoke。
   - Preview 验收通过后准备 Production，但在新的明确生产批准前停止；获批后部署 custom domain 并复测。本 Plan 不创建 GitHub repo/Release。
   - 写 Verification 并交给新上下文 verifier；GitHub ZIP/Skill 远程安装在仓库未公开时保持 pending，不伪造成功。

## Test-first proof

- 使用 Node 24 内置 test runner 测试纯筛选函数，不引入 Jest/Vitest。用真实主题 fixture 断言“纯色 + Codex”、背景 + 氛围、空结果和搜索交集。
- URL 测试覆盖 `type`/`series` 解析、非法值回退、保持另一维和旧 `view=codex|pure|background` 映射；新 URL 只写明确非默认参数。
- 构建后站点断言检查：首页不含 `usage-section` 文案重复；`/guide` 含精确 universal href 与不可点击 Windows 状态；`/contribute` 含 PR 流程和 Skill 状态。
- 对首页、两个指南页和代表主题解析 canonical、description、OG、Twitter、JSON-LD；解析 sitemap 并断言恰含三个顶层页面与数据库当前全部主题。
- 连续两次 `corepack pnpm --dir apps/web sync` 比较工作树目标文件，确保本变更未造成生成漂移。
- 完成命令：筛选 test、`./scripts/check.sh web`、`./scripts/check.sh workflow`；相关失败修复后重跑完整适用 gate。

## Visual or integration proof

- 宽屏截图证明只有一个左侧栏且筛选只出现一次；选择两维后，侧栏状态、URL、结果数和结果列表一致。
- 约 390px 截图证明导航与同一筛选区折叠为单入口，无第二套控件、横向滚动或主题行遮挡。
- 浅色/深色分别覆盖首页默认、组合筛选、搜索结果、行菜单、主题详情、`/guide`、`/contribute` 和焦点状态；主题预览图像素不因页面模式被滤镜改变。
- 浏览器交互检查主题查看/返回、列表指南入口、详情指南入口、复制色值、GitHub 文档链接、Skill 安装提示复制与浏览器后退/前进。
- 本地 production server、Vercel Preview 和获批后的 custom domain 均以 HTTP 200、DOM metadata、无控制台错误和截图为证；内置浏览器不替代用户指定的 Comet/真实浏览器验收。

## Risks and mitigations

- **两个筛选订阅不同步**：URL 是唯一状态源，不保留第二个 event/state 通道；纯函数与浏览器历史测试共同覆盖。
- **移动端出现第二套筛选**：复用同一组件/状态并通过布局折叠，不渲染桌面与移动两份独立控件。
- **指南内容漂移**：`/guide` 是使用说明来源，`docs/submitting-themes.md` 是完整投稿来源；其他位置只给短链接，不复制步骤。
- **Skill 尚未可安装**：根据仓库真实目录与相邻 Verification 写状态；GitHub 未公开时只显示未来安装提示并标注未验证。
- **深色局部亮块**：先归并硬编码产品色，再逐状态 screenshot；mockup 颜色变量独立，不进行全局 filter/invert。
- **SEO 污染体验**：信息放入 metadata、语义 heading 和指南页，不在主题列表中插入关键词段落；JSON-LD 与可见事实一致。
- **下载环境变量回退旧包**：移除代码覆盖路径并检查构建 HTML；部署前读取并清理 Vercel Preview/Production 配置。
- **脏工作树与共享文档冲突**：Skill 变更先提交其局部文档结果，网页变更顺序接续；只使用 apply_patch 合并相关段落，不覆盖无关修改。

## Rollback

- 本地/Preview 回退新增页面、筛选 helper/layout spec、metadata 文件和相关组件/CSS局部修改；恢复首页 usage 与旧导航仅作为最后手段，不触碰主题生成目录。
- Preview 失败时不推进 Production；Production 若获批后出现回归，重新 alias 到上一已验证部署，并恢复旧 Vercel环境配置快照。
- GitHub universal Release 未就绪不回退到 Vercel 大包；页面保留明确暂不可用状态或回退到上一网页部署，避免重新承担下载流量。
- 删除行为仅限本变更新增路由/测试文件或 Vercel 已确认旧环境变量；不删除用户主题、GitHub repo/Release 或 Cloudflare DNS。

## Deviations

- 当前无计划偏差。若 Next.js `useSearchParams` 导致不必要的大客户端边界，优先把 URL 解析留在小型筛选客户端而非将整页客户端化，并在 Verification 记录实现差异。
- 若相邻 Skill 变更尚未完成，网页允许先实现但必须显示“发布后可用”；不得为了满足文案验收伪造安装成功。
- 若 Comet 无法连接，使用本地真实浏览器做临时诊断，但最终视觉结论保持 pending，直到按用户指定浏览器复测。
- 2026-08-29 产品负责人明确追加授权：公开创建并推送 `IchenDEV/doubao-skin`，把仓库、Release、Marketplace、投稿与健康检查链接从原计划名称统一迁移到该仓库；提供中英文 README 与当前公开产品截图；Preview 验收后更新既有 Vercel Production，并核对 GitHub 触发的自动部署链路。该决定取代本 Plan 中“不创建 GitHub repo/Release”和“Production 仍待批准”的旧边界，但不授权发布未经验证的 Release 资产或修改其他 Vercel/Cloudflare 项目。

## Decision

本 Plan 已接受。2026-08-29 的追加产品决定进一步授权公开创建并推送 `IchenDEV/doubao-skin`，以及在候选通过 Preview 验收后更新既有 Vercel Production；GitHub Release 仍须由实际构建与发布验证单独证明。
