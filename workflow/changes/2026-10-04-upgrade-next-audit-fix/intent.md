---
id: "2026-10-04-upgrade-next-audit-fix"
stage: intent
status: accepted
owner: "claude"
created: "2026-10-04"
source: "user"
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-10-04"
---

# Intent: 升级 Next.js 修复 audit critical 漏洞

## Problem

`apps/web` 固定使用 `next@16.3.3`。npm advisory GHSA-vcvr-r3jv-pc5j(Next.js: Remote Code Execution in `next/og` `ImageResponse`,critical)影响 `>=16.2.0 <16.3.6`,修复版本为 `>=16.3.6`。

后果(均已核实):

- `./scripts/check.sh web` 与 CI 的 `Web application` 任务在最后一步 `pnpm --dir apps/web audit --audit-level=high` 失败。PR #43 合并时该检查即为红色,与 PR 内容无关。
- 之后所有触及网站的 PR 都会被同一个既有问题挡住,掩盖真正的回归。
- 站点部署在 Vercel;`apps/web` 源码里没有使用 `next/og` / `ImageResponse`(已搜索),因此当前可利用性低,但依赖仍处于已知存在 critical 漏洞的版本。

另:dependabot 已有 PR #39(`next` 16.3.3 → 16.3.7);npm 当前最新补丁为 16.3.8。该 PR 可能还要和其他 dependabot PR 的 lockfile 变更相互冲突。

## Proposed outcome

把 `apps/web` 的 `next` 升到同一次要版本内最新的已修复补丁版本(目标 `16.3.8`,至少 `16.3.6`),使 `pnpm audit --audit-level=high` 通过,`./scripts/check.sh web` 与 CI `Web application` 任务恢复绿色,且站点行为不变。

## Affected users and systems

- `apps/web`(`package.json`、`pnpm-lock.yaml`);间接影响 Vercel 构建与站点访客。
- CI 的 `Web application` 任务和本地 `./scripts/check.sh web`。
- 不影响桌面应用、`crates/`、主题包与协议桥。

## Constraints

- 只做补丁级升级(16.3.x),不跨次要版本,不顺带升级 `react`、`@types/*`、`sharp` 等无关依赖(它们有各自的 dependabot PR)。
- 不手改 `apps/web/data`、`apps/web/public/themes` 的生成文件。若 `pnpm --dir apps/web sync` 产生二进制噪声,不提交。
- 不新增依赖;lockfile 只含 `next` 及其传递依赖的必要变更。
- 保持 `pnpm@12.0.0` 与当前 Node 版本(`.nvmrc`)。

## Out of scope

- 升级到 Next 17 或其他次要版本。
- 处理其他 dependabot PR(#34、#40、#41 等)。
- 重新设计网站、调整 audit 门槛(`--audit-level=high` 保持不变)。

## Success signals

- `pnpm --dir apps/web audit --audit-level=high` 返回 0。
- `./scripts/check.sh web` 全部通过(类型检查、测试、构建、audit)。
- CI 上 `Web application` 任务为绿。
- 构建产物页面清单与升级前一致;至少在本地 `next start`/预览中确认首页与一个主题详情页可正常打开。

## Open questions

无(已决议,见 Decision)。

## Decision

- 目标版本取 `next@16.3.8`(同一次要版本内的最新补丁,高于修复版本 16.3.6)。
- 在本变更中独立升级,并在合并后关闭 dependabot PR #39 作为被取代;不直接合并 #39。
- 其余约束与范围按上文执行。
