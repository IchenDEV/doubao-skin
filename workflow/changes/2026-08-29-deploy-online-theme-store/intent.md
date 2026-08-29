---
id: "2026-08-29-deploy-online-theme-store"
stage: intent
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
source: "user"
risk: "high"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Intent: deploy online theme store

## Problem

主题画廊目前虽然已有一个 Vercel 生产项目和 `doubao-skin-gallery.vercel.app` 地址，但尚未使用产品域名 `doubao-skin.idevlab.dev`。桌面应用的主题商店默认地址也仍硬编码为旧的 Vercel 域名，品牌入口和本地产品没有形成稳定的一致链路。

当前仓库同时包含大量尚未提交的迁移、桌面应用和主题资源改动。本次发布必须明确使用经过验证的 `apps/web` 快照，不能误把旧提交或不完整生成物部署到生产。

## Proposed outcome

将当前主题画廊部署到既有 Vercel 项目 `ichendevs-projects/doubao-skin-gallery` 的生产环境，把 `doubao-skin.idevlab.dev` 绑定为正式域名，并让本地桌面应用默认从 `https://doubao-skin.idevlab.dev/themes/catalog.json` 获取线上主题目录。

上线前先通过本地 Web 构建与主题目录校验；上线后验证首页、主题详情、目录 JSON、预览图和主题安装包，并从 Rust 核心真实拉取同一线上目录。

## Affected users and systems

- 主题画廊访客与桌面主题工具用户。
- Vercel 项目 `doubao-skin-gallery` 的生产部署、环境变量与自定义域名。
- Cloudflare 托管的 `idevlab.dev` DNS 区域。
- `apps/web` 的构建产物和 `crates/skin-core` 的默认商店地址。
- 发布文档、健康检查和本次变更的验证工件。

## Constraints

- 复用既有 Vercel 项目，不创建同名或重复项目；生产发布必须显式使用 `--prod` 并先核对本地 `.vercel/project.json`。
- 只发布 `apps/web`，不把仓库根目录、桌面二进制、凭据、对话内容或本机 Vercel 元数据上传到 Vercel。
- Vercel 生产环境必须有可用的公开下载地址配置；不在代码或工件中记录令牌、Cookie 或账号凭据。
- DNS 只新增或更新精确主机名 `doubao-skin.idevlab.dev`，不改动 `idevlab.dev` 根域及其他记录；Cloudflare 代理模式服从 Vercel 自定义域名的证书与校验要求。
- 桌面商店继续保留 `DOUBAO_SKIN_THEME_STORE_URL` 环境变量覆盖能力；只改变默认地址，不放宽现有目录、下载大小、校验和、路径与压缩包安全限制。
- 主题源或元数据若在发布快照中发生变化，必须先运行 `pnpm --dir apps/web sync` 并保证生成目录可复现。
- 保留工作树中与本需求无关的所有未提交改动；不提交、合并或发布 macOS Release。

## Out of scope

- 不创建新的 Vercel 账号、团队或 Cloudflare 区域。
- 不把当前仓库推送到 GitHub，也不建立新的 GitHub/Vercel 自动部署集成。
- 不发布新的 macOS 安装包或 GitHub Release，不修复与本次上线无关的主题视觉或桌面功能。
- 不改变官方豆包或豆包工作应用，不扩展模型代理、会话读取或数据转发能力。

## Success signals

- Vercel 生产部署状态为 Ready，且生产别名指向本次已验证部署。
- `https://doubao-skin.idevlab.dev/` 使用有效 HTTPS 返回主题画廊，页面资源与至少一个主题详情可正常访问。
- `https://doubao-skin.idevlab.dev/themes/catalog.json` 返回 schema v1 目录，主题数量与本地生成目录一致；预览图和主题包均可下载。
- Rust 核心默认商店地址为新域名，相关回归测试通过；不设置覆盖变量时可真实拉取线上目录并完成校验。
- Web、Rust、workflow 的适用检查通过，部署 ID、DNS/证书状态、HTTP 证据、真实窗口截图和残余风险写入 `verification.md`。

## Open questions

无。使用当前 `apps/web` 工作区作为候选发布快照；只有在本地同步与检查全部通过后才允许进入生产发布。

## Decision

产品负责人已接受本 Intent，授权进入 Spec；尚未授权修改产品代码、部署生产或变更 DNS。
