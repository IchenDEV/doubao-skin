---
id: "2026-08-29-deploy-online-theme-store"
stage: spec
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
based_on: intent.md
risk: "high"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Spec: deploy online theme store

## Requirements

1. 发布源必须是当前工作区的 `apps/web`，并保持它与既有 `.vercel/project.json` 中的项目 ID `prj_JEnkkULvtOJVFLhOH57CGmk0Fp3V`、项目名 `doubao-skin-gallery` 一致。
2. 发布前运行主题同步和 Web 检查，确认本地目录为 schema v1、包含 26 套主题、每个条目都有可下载包且 SHA-256 与包内容一致。同步不得修改主题源。
3. Vercel Preview 与 Production 环境均设置 `NEXT_PUBLIC_APP_DOWNLOAD_URL=https://doubao-skin.idevlab.dev/downloads/Doubao-Skin-macOS-arm64.zip`；当前没有可用 GitHub 仓库或 Release，因此不设置虚假的 `NEXT_PUBLIC_REPO_URL`。
4. 网页下载文案必须与实际 arm64 安装包一致，使用“Apple 芯片版”，不得继续声称为通用版。安装包内容本身不在本变更中重建。
5. 先部署并验收 Preview，再将同一候选源显式部署到 Production；两个部署都必须为 Ready，Production 不能指向旧部署。
6. 把 `doubao-skin.idevlab.dev` 添加到现有 Vercel 项目，并在 Cloudflare 仅创建或更新 `A doubao-skin 76.76.21.21`，保持 DNS only；不改根域、名称服务器和其他记录。
7. `crates/skin-core` 的默认商店 URL 改为 `https://doubao-skin.idevlab.dev/themes/catalog.json`，继续允许 `DOUBAO_SKIN_THEME_STORE_URL` 覆盖。
8. README、网站部署文档与公开画廊健康检查统一使用新域名；不得保留会误导用户或监控的旧生产入口。
9. 线上目录、主题预览、主题包和桌面安装包必须能在新域名下通过 HTTPS 获取，不得依赖登录、Cookie 或应用私有请求头。

## User experience

- 用户访问 `https://doubao-skin.idevlab.dev` 后看到现有主题库，不发生额外跳转或 Vercel 身份验证。
- 用户可打开主题详情、查看预览，并从页面下载标注为 Apple 芯片版的 macOS 应用。
- 桌面主题工具打开主题商店时无需配置环境变量，直接显示线上 26 套主题、缩略图和可安装状态；开发者仍可用既有环境变量切换到测试目录。
- DNS 或证书传播期间不把未验证域名写入用户界面；只有新域名通过 HTTPS 验收后才把它视为正式入口。

## Technical design

- Web 发布目录为 `apps/web`。`pnpm sync` 从仓库根 `themes/` 生成 `data/themes.db`、`public/themes/catalog.json`、预览资源与 `public/themes/packages/*.doubao-skin.zip`；Vercel 构建只读取这些生成物，不在生产构建中跨目录抓取主题源。
- Vercel 继续使用现有项目的 Next.js preset 与 Node.js 24.x；本地链接文件只用于选择项目，不进入上传清单。
- Preview 和 Production 的公开下载变量使用 Vercel 环境变量保存。变量值不是秘密，但由平台配置统一提供，以满足 `site.ts` 对 Vercel 构建的显式配置要求。
- 自定义域名由 Vercel 项目持有，Cloudflare 只负责权威 DNS。采用 Vercel CLI 当前返回的推荐 A 记录 `76.76.21.21`，关闭 Cloudflare 代理，交由 Vercel 终止 TLS 并签发证书。
- Rust 端只替换 `DEFAULT_THEME_STORE_URL` 常量。新增回归测试同时固定新默认值和环境变量覆盖行为；目录获取、URL 解析、包大小限制、SHA-256 和安全解压逻辑保持不变。
- `health.yml` 监控首页、已知主题详情和桌面安装包；目录 JSON 另加入确定性检查，避免再次出现“首页 200、主题商店目录 404”而监控仍为绿色。

## Security and privacy

- 不向 Vercel 或 Cloudflare 上传凭据、对话、工作区数据或桌面应用运行数据。
- Vercel 上传范围限制在 `apps/web`；`.vercel`、`.next`、`node_modules` 和本地环境文件必须被忽略。
- 主题目录只引用同源相对 URL；Rust 下载器继续拒绝非 HTTP(S) URL、过大目录、过大包、路径穿越、符号链接和 SHA-256 不匹配。
- DNS 变更精确限定在 `doubao-skin.idevlab.dev`。若已有冲突记录，先读取并确认目标，只更新该主机，不批量删除记录。
- 浏览器登录态只用于 Vercel/Cloudflare 控制台操作，不读取或导出 Cookie、令牌或密码。

## Alternatives and non-goals

- 不创建新的 Vercel 项目：会分裂生产别名、环境变量和部署历史。
- 不迁移 `idevlab.dev` 名称服务器到 Vercel：只为一个子域改权威 DNS 的影响范围过大。
- 不使用 Cloudflare 代理：会增加 Vercel 域名校验、证书和缓存行为的不确定性。
- 不创建 GitHub 仓库或 Release：用户没有授权发布源代码或新桌面版本；现有站内 arm64 ZIP 足以维持当前下载入口。
- 不把商店 URL 做成新的桌面设置项：现有环境变量已经覆盖开发和故障切换需求，本次只需提供正确默认值。
- 不修复安装包签名、公证、`__MACOSX` 元数据或通用二进制分发；这些属于独立发布工作。

## Areas of concern

- 当前生产首页为 200，但 `/themes/catalog.json` 返回 Next.js 主题详情 404；新部署必须证明 `public/themes/catalog.json` 被静态上传且不被 `[id]` 动态路由吞掉。
- Vercel 当前无环境变量；Preview 与 Production 任一环境漏配下载 URL 都会触发构建失败。
- 本地默认 Node.js 版本低于仓库声明的 `24.20.0`，验证必须使用符合 `.nvmrc`/package engines 的 Node 24.20+，或把版本偏差明确记录为残余风险并以 Vercel 24.x 构建补证。
- 自定义域名已在 Vercel 账号的 `idevlab.dev` 域下可见，但 DNS 尚未正确配置。Cloudflare DNS 与 Vercel 证书生效可能有传播延迟，必须轮询到权威 DNS、Vercel 验证和 HTTPS 三者一致。
- `apps/web`、生成目录和主题源目前都在大量未提交改动中；只允许对本变更列出的文件做人工编辑，主题同步产生的差异必须逐项核对来源。
- 站内下载包是 arm64 而不是通用包，页面文案必须同步纠正；Intel 支持不在本次承诺中。

## Acceptance criteria

1. `pnpm --dir apps/web sync` 后目录仍为 26 套主题、26 个包，第二次同步无差异；`pnpm --dir apps/web check` 通过。
2. `cargo test -p skin-core`、`./scripts/check.sh rust` 和 `./scripts/check.sh workflow` 通过，新增测试确认新默认 URL 与覆盖变量行为。
3. Vercel Preview 和 Production 部署均为 Ready；Production 部署 ID、别名和候选源一致，并设置了 Preview/Production 下载环境变量。
4. Vercel 项目显示 `doubao-skin.idevlab.dev` 配置有效；Cloudflare 权威查询返回 `A 76.76.21.21`，无同名冲突 CNAME，HTTPS 证书有效。
5. 新域名首页、至少一个主题详情、`/themes/catalog.json`、一个缩略图、一个主题包和 `/downloads/Doubao-Skin-macOS-arm64.zip` 都返回 200；目录 JSON 为 schema v1、26 套主题，下载包 SHA-256 与目录一致。
6. 在真实桌面应用中，不设置 `DOUBAO_SKIN_THEME_STORE_URL` 时可打开线上主题商店，看到远程主题卡片和缩略图；正常与窄窗口均完成截图检查，无明显布局破坏。
7. 旧 Vercel 域名可继续作为平台别名，但 README、部署文档、健康检查和 Rust 默认值都只使用正式域名。
8. `verification.md` 记录命令、输出摘要、部署 ID、DNS 与证书证据、截图路径、偏差和残余风险；最终 verdict 由 fresh-context verifier 或产品负责人记录。

## Decision

产品负责人已接受本 Spec，授权生成实施 Plan；尚不授权修改产品代码、Vercel 环境变量、生产部署或 DNS。
