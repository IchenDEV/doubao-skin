---
id: "2026-10-04-upgrade-next-audit-fix"
stage: spec
status: accepted
owner: "claude"
created: "2026-10-04"
based_on: intent.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-10-04"
---

# Spec: 升级 Next.js 修复 audit critical 漏洞

## Requirements

1. `apps/web/package.json` 中 `next` 从 `16.3.3` 固定为 `16.3.8`(精确版本,与现有写法一致)。
2. `apps/web/pnpm-lock.yaml` 随之更新;`next` 及其 `@next/*` 平台包、`@swc/helpers` 等传递依赖只发生该升级所必需的变化。`react`、`react-dom`、`sharp`、`@types/*` 等版本不变。
3. `pnpm --dir apps/web audit --audit-level=high` 返回 0。
4. `./scripts/check.sh web`(类型检查、测试、构建、audit、两个脚本的 `node --check`)全部通过。
5. 不修改 `apps/web/src`、`next.config.ts`、`apps/web/data`、`apps/web/public/themes`;若升级迫使配置变化,先回到本 spec 修订并重新确认。

## User experience

站点访客无可见变化。首页与主题详情页的路由、元数据、静态导出行为保持一致。

## Technical design

- 在 `apps/web` 中使用 pnpm 修改版本并更新 lockfile(`pnpm --dir apps/web add next@16.3.8 --save-exact` 或等价的手工编辑 + `pnpm install --lockfile-only`),使用仓库既定的 `pnpm@12.0.0`。
- 升级后对比构建输出:路由清单(`Static` / `SSG` / `Dynamic`)与升级前一致。基线取升级前在同一台机器上执行 `pnpm --dir apps/web build` 的输出。
- 现有 `next.config.ts` 使用了 `agentRules: false` 与 `outputFileTracingIncludes`;需确认 16.3.8 仍接受这两个选项且无新的弃用警告。
- 本地启动预览(`next start` 或 `next dev`),检查首页与一个主题详情页 HTTP 200 且无控制台错误;数据库文件 `data/themes.db` 仍出现在构建追踪中。

## Security and privacy

- 目标:消除 GHSA-vcvr-r3jv-pc5j(`>=16.2.0 <16.3.6` 的 `next/og` `ImageResponse` 远程代码执行)。16.3.8 高于修复版本 16.3.6。
- 不引入新依赖,不改变网络请求、环境变量、凭据或 Vercel 配置。`.vercel` 本地元数据不入库。
- audit 门槛保持 `--audit-level=high`,不靠降级门槛让检查变绿。

## Alternatives and non-goals

- 合并 dependabot PR #39(升到 16.3.7):可行,但与其他 dependabot PR 共享 lockfile,易冲突,且 16.3.8 已发布;本变更以 16.3.8 取代,合并后关闭 #39。
- 使用 pnpm `overrides` 固定 `next`:会掩盖 `package.json` 中真实依赖版本,不采用。
- 降低 audit 门槛或在 CI 中忽略该 advisory:不采用,会隐藏真实风险。
- 非目标:升级 React、Node 或 pnpm;处理其他 dependabot PR;修改站点功能。

## Areas of concern

- 补丁升级可能带来构建行为变化(静态生成、`outputFileTracingIncludes`、Turbopack 默认行为)。用路由清单对比与本地预览兜底。
- 没有 Vercel 生产环境的验证入口;只能依赖 PR 预览部署(Vercel 检查)。生产部署由人工合并后观察。
- audit 结果依赖在线 advisory 数据库;将来新 advisory 出现会再次使该检查变红,不属于本变更。

## Acceptance criteria

- [ ] `apps/web/package.json` 中 `next` 为 `16.3.8`;lockfile 与之一致,`git diff --stat` 只涉及 `apps/web/package.json` 与 `apps/web/pnpm-lock.yaml`(以及本变更的 `workflow/` 文件)。
- [ ] `pnpm --dir apps/web audit --audit-level=high` 返回 0。
- [ ] `./scripts/check.sh web` 全部通过。
- [ ] 升级前后 `pnpm --dir apps/web build` 的路由清单一致,无新增警告。
- [ ] 本地预览首页与一个主题详情页均可打开(截图或 HTTP 状态记录在 `verification.md`)。
- [ ] CI 的 `Web application` 与 Vercel 预览检查为绿。
- [ ] `verification.md` 记录命令、结果、残余风险,最终结论由全新上下文的验证者或人工填写。

## Decision

采用独立变更,将 `next` 精确固定到 `16.3.8`,不改动站点源码与配置,不降低 audit 门槛;合并后关闭 dependabot PR #39。
