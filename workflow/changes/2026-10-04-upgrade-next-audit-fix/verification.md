---
id: "2026-10-04-upgrade-next-audit-fix"
stage: verification
status: pending
owner: "claude"
created: "2026-10-04"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: 升级 Next.js 修复 audit critical 漏洞

以下结果由实现者在分支 `cursor/upgrade-next-audit-fix`(基于升级前的 `main`)、提交前的工作区得到,日期 2026-10-04。

## Automated checks

| 命令 | 升级前 | 升级后 |
| --- | --- | --- |
| `pnpm --dir apps/web audit --audit-level=high` | 失败:1 个 critical,`next`,GHSA-vcvr-r3jv-pc5j,`>=16.2.0 <16.3.6` | 通过:`No known vulnerabilities found` |
| `pnpm --dir apps/web build` | 成功 | 成功,无 warn / deprecat 输出 |
| `./scripts/check.sh web`(类型检查、测试、构建、audit、`node --check`) | 末尾 audit 失败 | 通过(`fail 0`,退出码 0) |
| `./scripts/check.sh workflow` | 通过 | 通过 |

范围检查:

- `git diff --stat` 只涉及 `apps/web/package.json`(`next` `16.3.3` → `16.3.8`,精确版本)和 `apps/web/pnpm-lock.yaml`。
- lockfile diff 中变化的只有 `next` 及其 `@next/*` 平台包(`@next/env` 与各平台 swc 包);`react`、`react-dom`、`sharp`、`@types/node` 版本不变。
- `apps/web/src`、`next.config.ts`、`apps/web/data`、`apps/web/public/themes` 未改动;未运行 `sync`,工作区无生成物噪声。

## Behavioral evidence

- 升级前后 `next build` 的路由清单一致:`/`(Dynamic)、`/_not-found`、`/contribute`、`/guide`、`/icon.png`、`/robots.txt`、`/sitemap.xml` 为 Static,`/themes/[id]` 为 SSG(3 个预渲染路径 + 31 个其他路径)。
- `next start`(`16.3.8`,端口 3917)本地预览请求结果:`/` 200、`/themes/violet-night` 200(标题「暗夜紫 · 豆皮」)、`/guide` 200、`/sitemap.xml` 200;启动日志无错误,`next.config.ts`(含 `agentRules`、`outputFileTracingIncludes`)被正常接受。

## Visual evidence

未做截图。本变更只升级补丁版本、不改站点源码与样式;视觉检查仅限 HTTP 状态与页面标题。若验证者需要视觉证据,请在 Vercel 预览部署上抽查首页与一个主题详情页。

## Security and privacy evidence

- 目标 advisory GHSA-vcvr-r3jv-pc5j 的修复版本为 `>=16.3.6`;已升级到 `16.3.8`,audit 复测通过。
- 未新增依赖,未改动环境变量、凭据、网络请求或 Vercel 配置;audit 门槛保持 `--audit-level=high`。
- 站点源码里没有使用 `next/og` / `ImageResponse`(升级前已搜索),因此升级前的实际可利用性本就较低;升级主要是消除已知 critical 依赖。

## Deviations and residual risk

偏离:无。

残余风险:

- CI 的 `Web application` 与 Vercel 预览部署尚待 PR 上的结果;生产环境行为只能在人工合并后观察。
- audit 依赖在线 advisory 数据库,将来出现新的 high / critical advisory 会再次使该检查变红,不属于本变更。
- dependabot PR #39(`next` → 16.3.7)与本变更重复,合并本变更后应关闭 #39;其他 dependabot PR(react、sharp、`@types/node`)的 lockfile 可能需要 rebase。

## Verdict

[fill]
