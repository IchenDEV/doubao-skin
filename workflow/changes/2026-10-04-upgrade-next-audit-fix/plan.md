---
id: "2026-10-04-upgrade-next-audit-fix"
stage: plan
status: accepted
owner: "claude"
created: "2026-10-04"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-10-04"
---

# Plan: 升级 Next.js 修复 audit critical 漏洞

## Files and ownership

- `apps/web/package.json`:`next` 改为 `16.3.8`。
- `apps/web/pnpm-lock.yaml`:由 pnpm 重新生成,不手改。
- `workflow/changes/2026-10-04-upgrade-next-audit-fix/*`:本变更 artifact(含 `verification.md`)。
- 不触及 `apps/web/src`、`next.config.ts`、`apps/web/data`、`apps/web/public/themes`、`crates/`、`apps/desktop/`。

所有文件互相依赖、改动很小,按顺序单人完成,不使用额外 worktree。

## Order of work

1. 基线(升级前,从干净的 `main` 新分支 `cursor/upgrade-next-audit-fix` 开始):
   - `corepack pnpm --dir apps/web install --frozen-lockfile`
   - 记录 `pnpm --dir apps/web audit --audit-level=high` 当前失败输出。
   - 记录 `pnpm --dir apps/web build` 的路由清单与警告,作为对比基线。
2. 升级:`pnpm --dir apps/web add next@16.3.8 --save-exact`。确认 `git diff --stat` 只含 `package.json` 与 `pnpm-lock.yaml`;检查 lockfile diff 中只有 `next` 及其必要传递依赖变化。若 pnpm 顺带改动 `react`/`sharp` 等版本,回退并改用只更新 `next` 的方式。
3. 检查:`pnpm --dir apps/web audit --audit-level=high`、`./scripts/check.sh web`、`./scripts/check.sh workflow`。
4. 对比升级后 `build` 的路由清单与警告,确认与基线一致,且无新的弃用提示(特别是 `agentRules`、`outputFileTracingIncludes`)。
5. 本地预览:启动 `next start`(或 `dev`),请求首页和一个主题详情页,记录状态码与控制台输出。
6. 若 `pnpm --dir apps/web sync` 被触发或产生二进制噪声,用 `git checkout -- apps/web/data apps/web/public/themes` 还原,不提交。
7. 写 `verification.md`(命令、结果、残余风险),状态保持 `pending`,结论留给全新上下文验证者或人工。
8. 提交到该分支,开 draft PR(按 `.github/PULL_REQUEST_TEMPLATE.md`),等待 CI 的 `Web application` 与 Vercel 预览;合并由人工决定。合并后关闭 dependabot PR #39。

## Test-first proof

本变更没有新增行为,不写新的单元测试。"先失败"的证明是:升级前 `pnpm audit --audit-level=high` 对 `next` critical 失败(步骤 1 记录);升级后同命令通过(步骤 3)。构建、类型检查与现有测试作为回归保护。

## Visual or integration proof

- 路由清单前后对比(步骤 1 与 4)。
- 本地预览首页与一个主题详情页的状态码,必要时附截图(步骤 5)。
- PR 上的 Vercel 预览部署与 CI `Web application` 结果。

## Risks and mitigations

- 补丁升级引入构建行为变化:路由清单对比 + 本地预览。
- lockfile 噪声或其他依赖被连带升级:检查 `git diff --stat` 与 lockfile diff,必要时回退重做。
- 在线 advisory 数据在合并前后变化:记录 audit 运行日期,残余风险写入 `verification.md`。
- 无法在本机验证 Vercel 生产环境:依赖 PR 预览部署,生产验证由人工合并后观察。

## Rollback

还原这次提交(仅 `package.json` 与 lockfile 的改动),即回到 `next@16.3.3`。回滚会让 audit 再次失败,但站点功能不受影响。

## Deviations

[none yet]

## Decision

按上述顺序执行;任何偏离(例如需要改 `next.config.ts`、连带升级其他依赖)先回到 spec/plan 修订并重新确认,再继续。
