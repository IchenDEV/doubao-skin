---
id: "2026-08-29-web-discovery-dark-mode"
stage: intent
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
source: "user"
risk: "high"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Intent: web discovery dark mode

## Problem

当前网站已经能浏览主题，但公开入口仍有一组影响长期使用和理解的问题：

- “全部/背景/纯色”同时出现在侧栏和内容区；“类型”与“主题系列”本应是可组合的两个维度，当前却共享一个互斥状态。界面虽然整洁，但筛选逻辑不符合用户逐步缩小结果的操作方式。
- 使用说明和下载区埋在主题列表底部，访客必须穿过完整列表才能找到；页面导航也没有把“浏览主题”“安装使用”“创作投稿”分成清楚任务。
- 下载按钮直接绑定当前 Vercel 静态 ZIP，会让 Vercel 和自定义域名承担安装包下载流量；项目目前还没有可用的 `IchenDEV` GitHub Release，但页面需要先使用最终约定的 GitHub Release 地址。
- 页面只展示 macOS 下载，没有向 Windows 访客说明平台支持状态，容易让人误以为遗漏了 Windows 版本。
- “如何提交新主题”只零散写在英文贡献指南中，网站没有说明主题不是直接上传到网页，而是通过 GitHub Pull Request 提交，也没有给出创建、校验、同步和上传的完整路径。
- 仓库正在设计 `create-doubao-theme` 与 `apply-doubao-theme` 两个 Skill，但网站没有说明它们分别做什么、在发布后如何安装，或豆包工作当前缺少公开导入协议的边界。
- 页面只有基础标题与描述，没有生产域名 canonical、robots、sitemap、完整社交预览和与可见内容一致的结构化数据；主题详情虽可静态访问，但搜索引擎和生成式搜索不容易稳定发现全量主题。
- 站点目前只有浅色视觉令牌，在系统深色外观下仍显示大面积亮底。

## Proposed outcome

- 重新整理为单一左侧栏：顶层只提供“主题库”“使用与下载”“创作与投稿”三个任务入口；主题库内只保留一套筛选控件，按“主题类型”和“主题系列”两个独立维度组合筛选，主内容区不再重复筛选标签。
- 新建独立的“使用与下载”页面，集中放置 macOS/Windows 平台状态、下载安装、首次打开、应用主题、恢复默认和 Skill 安装说明；主题库首页移除底部使用区。
- 网站实际的 macOS 下载按钮集中在 `/guide`，并统一指向 `https://github.com/IchenDEV/doubao-skin/releases/latest/download/Doubao-Skin-macOS-universal.zip`。安装包由 GitHub Release 托管，Vercel 只承担网页和主题目录流量；在首个 Release 上传前允许链接暂时返回 404。
- 在独立使用页的下载区域紧邻 macOS 下载入口展示不可点击的 `Windows · Coming Soon` 状态，明确 Windows 版尚未提供，同时不削弱 macOS 主操作。
- 新建“创作与投稿”页面和一份中文“提交新主题”文档：页面用短流程说明创建、校验、同步、Fork/分支和 Pull Request，明确没有网页直传；文档保存完整可执行清单并作为唯一详细来源。
- 在使用与投稿页面说明 `create-doubao-theme` 与 `apply-doubao-theme` 的职责、Codex 安装入口和调用示例。Skill 尚未随仓库发布时标注“发布后可用”；豆包工作没有公开外部 Skill 导入协议时只说明限制，不伪造安装目录。
- 补齐生产域名 canonical、Open Graph/Twitter 元数据、`robots.txt`、包含全部主题详情的 `sitemap.xml`，以及与页面可见事实一致的 `WebSite`、主题集合和主题详情结构化数据。
- 深色模式默认跟随 `prefers-color-scheme`，使用现有 CSS 令牌适配背景、文字、边框、浮层、按钮和浏览器 `theme-color`；不增加首屏 JavaScript、弹窗或强制主题切换器。
- GEO 采用可抓取的语义 HTML、清晰事实、内部链接和标准结构化数据，不做关键词堆砌，不新增 Google 明确表示不需要的 AI 专用文本文件或特殊 schema。

## Affected users and systems

- 浏览主题、查看详情或下载 macOS 应用的访客。
- 需要安装、首次运行、恢复默认或了解 Windows 支持状态的新用户。
- 使用系统深色外观、键盘导航或高对比度偏好的访客。
- 想通过手工流程或 Agent Skill 创作并提交新主题的贡献者，以及审核 Pull Request 的维护者。
- Google、Bing 与其他遵循 robots、sitemap 和标准 HTML/JSON-LD 的搜索及生成式检索系统。
- `apps/web`、相关仓库文档、健康检查和 Vercel Preview/Production 配置。

## Constraints

- 保持主题列表的清晰密度、卡片预览、详情主流程和整体视觉语言；允许重组侧栏、筛选和帮助页面，不做无关的全站视觉重设计。
- 桌面只出现一个左侧栏，筛选只出现一次；“主题类型”和“主题系列”必须独立组合，不得再次合并成一个互斥枚举。窄屏使用同一筛选状态的单一折叠入口，不复制第二套控件。
- 使用与下载、创作与投稿必须成为可直接访问、可被导航和搜索发现的独立页面；首页不再在主题列表末尾重复这些内容。
- Windows 状态只能作为低强调的静态提示，不提供无效下载链接、订阅表单、弹窗或虚假的发布时间。
- 深色模式必须保持正文、辅助文字、边框、焦点环和交互控件的可读对比度；主题预览图自身颜色不被反相或重绘。
- 重要内容保留在服务端渲染 HTML 中；不得为元数据或主题模式引入重型客户端依赖、分析脚本或额外网络请求。
- 结构化数据必须与页面可见内容一致；canonical 和 sitemap 只使用 `https://doubao-skin.idevlab.dev`。
- 保留现有可复现主题目录生成链和脏工作树边界，不手工编辑生成文件。
- GitHub 仓库名固定为 `IchenDEV/doubao-skin`，Release 资产名固定为 `Doubao-Skin-macOS-universal.zip`；现有 Release 工作流必须继续产出这个精确文件名。
- Skill 安装说明以当前可验证的宿主契约为准：Codex 可以说明使用 `$skill-installer` 从 GitHub 路径安装；豆包工作在没有公开导入契约前不得声称可安装。

## Out of scope

- 不创建或公开 GitHub 仓库，不推送当前脏工作树，也不发布首个 GitHub Release；这些操作由产品负责人稍后完成。
- 不新增账号、评论、在线主题投稿表单、数据库写入、分析/广告脚本或 Search Console/Bing Webmaster 所有权验证。
- 不改变主题内容、分类、排序、包 schema 或桌面应用安装逻辑；不把本轮信息架构修正扩大成新的营销首页或视觉品牌重做。
- 不构建、打包或发布 Windows 应用；本次仅呈现真实的平台路线状态。
- 不在本变更中实现两个主题 Skill 或 Rust CLI；它们继续由 `2026-08-29-rust-theme-skills-cli` 变更交付，网站只呈现经验证的当前状态和安装路径。
- 不实现手动明暗切换与持久化偏好；本次先提供零脚本的系统外观跟随。

## Success signals

- `/guide` 的 macOS 下载按钮指向约定的 GitHub Latest Release 直链；首页、主题列表和详情统一引导到该页面。首个 Release 上传后，直链无需改动即可匿名下载有效 ZIP，下载流量不经过 Vercel。
- 主题库不再重复显示筛选控件；类型与系列可独立选择并组合，搜索继续叠加，结果数量与 URL 状态一致。
- `/guide` 独立承载下载与使用说明，清楚显示 macOS 可下载和 `Windows · Coming Soon`；首页主题列表结束后直接进入页脚。
- `/contribute` 明确说明通过 GitHub Pull Request 投稿，并能进入完整中文文档；创作者能按文档完成创建、同步和校验流程。
- 使用与投稿页面能区分两个主题 Skill，并给出 Codex 发布后的安装/调用方式与豆包工作当前兼容边界，不把未实现能力写成已可用。
- `/robots.txt` 和 `/sitemap.xml` 返回 200，sitemap 覆盖首页、两个指南页和 26 个主题详情；公开页面 HTML 含正确 canonical、描述、社交预览和可解析 JSON-LD。
- Google 官方建议的基础要求成立：允许抓取、重要内容文本化、内部链接可发现、结构化数据与可见内容一致；不依赖 AI 专用文件。
- 在浅色与系统深色模式下，首页、组合筛选、主题详情、使用页和投稿页无不可读文字、闪白、横向溢出或交互退化；键盘焦点清晰。
- Node/Next 构建、站点 HTTP/元数据断言、桌面与约 390px 窄视口浏览器验收通过，且控制台没有相关错误或警告。

## Open questions

- 两个主题 Skill 的实现由相邻变更推进。若本网站先上线，页面必须将其标为“发布后可用”，保留经 OpenAI 官方文档验证的 Codex 安装方式；不得为了填满页面而提供当前无法执行的豆包工作安装步骤。

## Decision

等待产品负责人接受、要求修改或拒绝本 Intent；未接受前不编辑网页产品代码，也不变更 Vercel Production 或 GitHub 外部状态。
