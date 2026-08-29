---
id: "2026-08-29-web-discovery-dark-mode"
stage: spec
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
based_on: intent.md
risk: "high"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Spec: web discovery dark mode

## Requirements

1. **单一信息架构与组合筛选**
   - 桌面只保留一个左侧栏。顶层导航固定为“主题库”“使用与下载”“创作与投稿”，移除当前会跳到任意主题详情的“发现主题”。
   - 主题库筛选只在侧栏出现一次，内容区删除现有 `theme-tabs`。筛选分为两个独立单选维度：主题类型（全部、纯色、有背景图）和主题系列（全部、氛围、经典、Codex、明亮，以及数据库真实存在的其他系列）。
   - 两个维度按 AND 组合，搜索再与组合结果叠加；任何维度都不能重置另一个维度。结果数量以 `aria-live="polite"` 更新。
   - 新 URL 使用 `type=<all|pure|background>` 与 `series=<category>` 保存状态。现有 `view=` 链接在本次发布中映射到等价新状态；筛选查询页 canonical 仍为首页。
   - 筛选组使用有可访问名称的原生控件和不小于 44px 的点击目标；结果区只保留搜索、结果数量、列表表头和主题行，不再显示第二套筛选按钮或伪 tab 语义。
2. **独立任务页面**
   - 新增 `/guide`“使用与下载”：按下载、安装、首次打开、应用主题、恢复默认、安装主题 Skill 的顺序组织，macOS 下载位于首屏可见区域。
   - 新增 `/contribute`“创作与投稿”：明确网站没有直传入口，主题通过 GitHub Fork/分支/Pull Request 提交，并用短流程连接详细投稿文档。
   - 首页删除底部 `usage-section`；页头“使用指南”、列表菜单和详情安装区统一链接 `/guide` 或 `/guide#download`，不在多处复制安装步骤。
   - 全站侧栏能直接到达三个顶层页面；详情页仍可返回主题库并访问使用与投稿入口。
3. **稳定下载与平台状态**
   - `/guide` 中唯一真实的 macOS 下载按钮使用 `https://github.com/IchenDEV/doubao-skin/releases/latest/download/Doubao-Skin-macOS-universal.zip`。
   - 页面称为“下载 macOS 通用版”，与 `./scripts/build-macos.sh --universal`、Release workflow 和 `Doubao-Skin-macOS-universal.zip` 资产名一致。
   - 下载不得经过 Vercel 或自定义域名转发；旧 Vercel arm64 ZIP 不再作为页面目标，也不能由环境变量重新覆盖。
   - macOS 主按钮旁显示不可点击、低强调的 `Windows · Coming Soon`；不得提供 Windows URL、表单、弹窗或发布日期。
   - GitHub 仓库和首个 Release 由产品负责人稍后上传；出现前的 404 不阻止网页上线，但 Verification 必须标为外部待验证项。
4. **创作者上传与主题 Skill**
   - 新增 `docs/submitting-themes.md` 中文指南，覆盖目录结构、必填元数据、CSS、预览与版权、创建分支、运行同步与校验、提交生成文件、发起 Pull Request 和审核后的上线过程。
   - `/contribute` 用 5 步摘要解释：创建主题 → 本地预览与检查 → 同步网站目录 → Fork/分支并推送 → 发起 Pull Request；明确“上传”不是把 ZIP 传到网页。
   - 页面区分 `create-doubao-theme`（创作、预览、检查、打包）与 `apply-doubao-theme`（列出、安装、应用、恢复）。
   - Codex 安装说明使用官方支持的 `$skill-installer`，给出可复制提示：`请用 $skill-installer 从 GitHub 仓库 IchenDEV/doubao-skin 安装 skills/create-doubao-theme 和 skills/apply-doubao-theme`；并给出 `$create-doubao-theme`、`$apply-doubao-theme` 调用示例。
   - 两个 Skill 尚未由 `2026-08-29-rust-theme-skills-cli` 交付时，页面显示“随主题工具链发布后可用”，不能把安装写成当前已成功；豆包工作没有公开外部 `SKILL.md` 导入协议时写明“暂未提供可验证安装方式”。
   - 完整字段规范继续由 `design/theme-standard/README.md` 和 schema 负责；新指南引用它们，不复制会漂移的定义。README、docs 索引和网站均能发现投稿指南。
5. **SEO 与生成式搜索可发现性**
   - 生产基址和 canonical 固定为 `https://doubao-skin.idevlab.dev`，不得因 Vercel Preview host 改写生产 canonical。
   - 根布局提供准确标题模板、中文描述、Open Graph、Twitter Card 和浅/深两组浏览器 `theme-color`。
   - canonical 覆盖 `/`、`/guide`、`/contribute` 和 `/themes/<id>`；主题详情标题、描述和预览图来自对应主题数据。
   - `/robots.txt` 允许抓取公开页面并声明 `/sitemap.xml`；sitemap 包含三个顶层页面和 SQLite 当前全部主题详情，不手写主题列表。
   - 首页输出 `WebSite`、`CollectionPage` 和主题 `ItemList` JSON-LD；详情输出对应主题 `CreativeWork`；指南页使用与可见内容一致的 `WebPage`。不得添加不可见评价、官方关系或不可验证字段。
   - GEO 只使用语义 HTML、真实说明、内部链接和标准结构化数据；不新增 AI 专用文本文件、关键词堆砌或隐藏内容。
6. **系统深色模式**
   - 网站声明 `color-scheme: light dark`，通过 `prefers-color-scheme: dark` 跟随系统，不增加主题脚本、客户端存储或手动切换器。
   - 产品界面的页面背景、侧栏、卡片、正文、辅助文字、边框、悬停、选中、输入框、浮层、阴影和焦点环改用语义 CSS 变量并提供深色值。
   - 主题预览 mockup、主题自身色板和预览图片保持原色，不做反相、滤镜或自动重绘。
   - 兼顾 `prefers-reduced-motion` 和清晰 `:focus-visible`；浅色、深色、桌面与 390px 窄视口不得出现不可读文字、闪白或横向溢出。
7. **部署与健康检查**
   - Vercel Preview/Production 使用固定站点与 GitHub 仓库常量，不再让旧 `NEXT_PUBLIC_APP_DOWNLOAD_URL` 覆盖下载目标。
   - 产品健康检查覆盖三个顶层页面、主题详情、目录、robots、sitemap，并将桌面包目标对齐 GitHub universal Release；首个 Release 尚未创建造成的已知 404 与网站自身故障分开记录。
   - 不手工修改生成目录；本变更不改变主题源数据时，连续两次 sync 必须保持清单稳定。

## User experience

- 桌面外壳使用单一 `264px` 左侧栏和流动内容区；内容最大宽度延续当前约 `1296px`，左右内边距在宽屏为 `48px`。筛选、搜索和列表共享稳定边缘，不靠绝对定位修补。
- 侧栏顶部是三个任务入口，主题库页面在其下出现两个有标题的筛选组。选中“纯色 + Codex”时必须得到条件交集，而不是最后一次点击覆盖前一次选择。
- 主内容区标题下只有搜索与结果反馈，不再重复侧栏筛选。现有主题行密度、预览比例、查看按钮和更多菜单保持熟悉。
- `≤919px` 进入单列：导航成为顶部区域，原筛选区收进一个“筛选”披露控件，仍然只有一套状态和控件。`≤699px` 列表继续使用现有窄屏卡片表达，页面最小宽度 320px。
- `/guide` 和 `/contribute` 使用约 `760px` 的正文阅读列，短段落、编号步骤和可复制命令优先，不做大块营销卡片墙。
- `/guide` 的 macOS 按钮是唯一主操作，Windows 状态明显但不可点击；主题详情和列表只引导到该页，避免多个版本不一致的安装说明。
- Skill 区先说明“它能做什么”，再显示兼容宿主、当前状态、安装提示和调用示例；不能让访客从 Coming Soon 文案误判为已经安装。
- 深色外观保持当前清晰、克制的产品语言，不改变主题预览颜色、列表密度或详情层级。

## Technical design

- `apps/web/src/lib/site.ts` 作为生产站点、GitHub 仓库、投稿文档、两个 Skill 路径和 universal ZIP 的唯一事实来源；移除旧 arm64 下载覆盖逻辑。
- `SiteHeader` 保留一个侧栏并按 pathname 显示三个任务入口；只在主题库路由显示两个筛选组。筛选状态以 URL 为单一来源，`GalleryClient` 只维护搜索文本并依据 URL 的 type/series 计算交集，删除自定义 window event 与重复 tabs。
- 新增 App Router 服务端页面 `app/guide/page.tsx` 与 `app/contribute/page.tsx`。首页移除 `usage-section`；详情安装面板与主题行菜单改为内部链接 `/guide#download`。
- 执行 Plan 必须先新增或更新网站 layout spec：桌面外壳 `264px + minmax(0, 1fr)`，内容边距 `48/32/20px` 对应宽屏/平板/手机，断点复用现有 `919px` 与 `699px`，不新增冲突断点。
- 复用 Next.js Metadata API，在根布局和页面 metadata 中设置 canonical 与社交信息；新增原生 `robots.ts` 和 `sitemap.ts`，不引入 SEO 包。
- sitemap、首页 ItemList 和筛选标签直接读取 `getAllThemes()`/现有分类；详情 metadata 与 JSON-LD 读取 `getTheme(id)`，不存在主题继续 `notFound()`。
- JSON-LD 作为服务端渲染的 `application/ld+json` script，序列化时转义 `<`，只使用站点常量和已校验数据库字段。
- 深色模式在 `globals.css` 中把产品界面硬编码色归并为最小语义变量集，再由深色媒体查询覆盖；主题 mockup 内部变量不参与页面深色转换。
- `.github/workflows/release.yml` 已正确构建并上传 universal 资产，不重写打包流程；健康检查和文档只对齐既有契约。

## Security and privacy

- 不新增分析、广告、用户追踪、Cookie、本地存储、账号或投稿表单。
- GitHub 外链使用 HTTPS；普通文档外链在新窗口打开时带 `rel="noreferrer"`，下载保持浏览器原生导航。
- JSON-LD 不拼接未转义 HTML；不得暴露环境变量、内部 Vercel URL、工作区路径或构建信息。
- 不上传、内联或重新授权来源不明的预览资源；投稿指南要求说明资源来源和再分发权限。
- 网站只说明 Skill 安装，不代表用户执行安装；页面脚本不得调用终端、协议处理器或自动下载可执行文件。

## Alternatives and non-goals

- 不使用站内 `/download` 重定向，不保留 arm64 专用文案。
- 不保留内容区 horizontal tabs，也不把类型和系列压回一个 `view` 枚举。
- 不把使用说明继续放在首页底部，也不让详情页复制完整说明；`/guide` 是唯一使用说明来源。
- `/contribute` 只提供可浏览摘要，GitHub 文档仍是完整投稿清单；不建立网页上传 API、存储或审核后台。
- 不添加手动明暗切换、主题持久化、UA 平台识别、Windows 安装包或候补订阅。
- 不在本变更中实现或安装两个主题 Skill，不把它们包装成插件；功能由相邻变更交付，网站只准确显示状态和安装路径。
- 不创建 GitHub 仓库、不推送代码、不创建 tag/Release；不执行 Search Console、Bing Webmaster 所有权验证。

## Areas of concern

- GitHub 仓库和 Release 尚不存在，下载、投稿文档与 Skill 路径会在上传前返回 404；网页只能验证 URL 契约，不能声称匿名下载或安装成功。
- 两个 Skill 的 Spec 当前仍是 draft。若网站先上线必须显示“发布后可用”，后续真实发布后还需更新状态并执行安装验收。
- 当前 Vercel 环境仍配置旧 `NEXT_PUBLIC_APP_DOWNLOAD_URL`；部署时必须移除或确保代码不再读取它。
- 筛选迁到两个 URL 参数时容易造成侧栏与结果不同步；旧 `view` 兼容、前进/后退、搜索叠加和窄屏折叠都要真实测试。
- 当前 CSS 有较多硬编码浅色，遗漏浮层、选中态或窄屏导航会造成深色模式局部亮块。
- canonical 和 sitemap 容易在 Preview host 下漂移，必须用构建产物或 Preview HTML 断言生产域名。
- GitHub `latest` 只指向正式 Release；只有 draft/prerelease 时直链不可用。

## Acceptance criteria

- [ ] 桌面只有一个左侧栏；三个任务入口可达，内容区不再出现重复 `theme-tabs`。
- [ ] 类型与系列可独立组合，搜索继续叠加；URL、选中状态、结果和数量一致，前进/后退与旧 `view` 链接兼容。
- [ ] `/guide` 与 `/contribute` 可直接访问；首页不再包含底部使用区，列表和详情统一引导到 `/guide#download`。
- [ ] `/guide` 的 macOS 按钮是精确 GitHub universal ZIP；仓库不再把 `/downloads/Doubao-Skin-macOS-arm64.zip` 用作网站下载目标。
- [ ] Release workflow 仍全部使用 `Doubao-Skin-macOS-universal.zip`；网页显示“macOS 通用版”。
- [ ] `/guide` 显示不可点击的 `Windows · Coming Soon`；桌面、深色和 390px 下主次清楚且无溢出。
- [ ] `/contribute` 明确没有网页直传并展示 5 步 PR 流程；投稿文档可完成一次现有主题同步与校验，README/docs 索引可发现。
- [ ] 两个 Skill 的职责、Codex `$skill-installer` 提示和调用示例清楚；目录不存在时显示“发布后可用”，豆包工作不出现未经验证的安装路径。
- [ ] 首页、两个指南页和任一主题详情含生产域名下正确 canonical、中文描述、Open Graph、Twitter Card 和可解析 JSON-LD。
- [ ] `/robots.txt` 与 `/sitemap.xml` 返回 200；sitemap 包含三个顶层页面及当前 26 个主题详情且无 Preview host。
- [ ] JSON-LD 与可见名称、描述、作者、分类和 URL 一致，不含虚构评价、官方背书或不可验证统计。
- [ ] 浅色与系统深色下首页、组合筛选、行菜单、详情、色板复制、使用页和投稿页均可读可操作；主题预览颜色不变。
- [ ] 桌面和约 390px 视口无新增横向滚动、遮挡或布局跳变，键盘焦点清晰，控制台无相关错误或 hydration 警告。
- [ ] 连续两次 `pnpm --dir apps/web sync` 清单稳定，`./scripts/check.sh web` 与 `./scripts/check.sh workflow` 通过。
- [ ] Vercel Preview 通过三个顶层页面、详情、筛选、robots、sitemap、metadata、主题目录和主题包烟测后才更新 Production；自定义域名复测通过。
- [ ] GitHub 仓库/Release 未上传时，Verification 标记匿名 ZIP 下载为外部待验证，不将 404 误报为网页构建失败。

## Decision

等待产品负责人接受、要求修改或拒绝本 Spec；未接受前不生成执行 Plan，也不编辑网页产品代码或部署配置。
