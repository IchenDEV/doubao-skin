---
id: "2026-08-29-add-portal-github-claude-machine-themes"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Plan: 实现四套主题包并完成真实窗口验证

## Files and ownership

本变更只拥有以下文件；工作区其他改动均视为并行工作，不回退、不格式化、不归因：

- 主题源：
  - `themes/huaxia-blue/{theme.json,theme.css,preview.jpg}`
  - `themes/github-repository/{theme.json,theme.css,preview.jpg}`
  - `themes/claude-warm/{theme.json,theme.css,preview.jpg}`
  - `themes/machine-overseer/{theme.json,theme.css,preview.jpg,bg.jpg}`
- 网页同步产物：
  - `apps/web/data/themes.db`
  - `apps/web/public/themes/catalog.json`
  - 四个 ID 各自的 `.preview.jpg`、`.preview.card.jpg` 与 `packages/*.doubao-skin.zip`
  - `machine-overseer.jpg` 与 `machine-overseer.card.jpg`
- 过程与证据：
  - 本目录下的 `intent.md`、`spec.md`、`plan.md`、后续 `verification.md`
  - `evidence/` 下四套主题普通与窄窗口截图，以及必要的预览检查图

不修改 Rust、桌面应用、网页应用逻辑、主题 schema、设计标准、已有主题源或用户提供的参考图。

## Order of work

1. **建立可归因基线。** 记录 `git status`、四个目标目录不存在的事实、现有 catalog/数据库中四个 ID 的查询结果，以及目标生成文件的当前哈希。运行唯一的 bundled-theme 精确测试作为实现前基线。
2. **固定来源边界。** 从 Primer 官方仓库读取当前 HEAD 与许可，写入“代码仓库”的追溯字段；为“Claude 暖橙”记录官方产品参考地址和访问日期。只取色彩与视觉原则，不下载标志、字体、插画或页面代码。
3. **生成四个主题骨架。** 依次用现有 `doubao-theme create` 命令创建四个新目录，指定已批准的名称、描述、强调色、`appearance both` 和作者；该命令不得覆盖已有目录。
4. **完成 manifest 与 CSS。** 按 spec 逐个填写浅色/深色语义色、字体栈、行高、布局、输入框圆角、全局圆角比例、商店字段和严格按 ID 限制的 CSS。每完成一个主题立即执行 `check → preview → check`，并目视检查预览的色彩与层级。
5. **生成原创背景。** 使用内置图像生成能力，以用户参考图的幽默节奏、舞台感和低保真画面质感为风格输入，并通过明确禁用项避免复制其人物、舞台纹样、动作、文字和构图。选定图像后移入主题目录，转换为高质量 JPEG，控制为 16:9、至少 1920 px 宽、不超过 1.5 MB；检查内容、尺寸与 SHA-256 后再生成主题预览。
6. **运行最小自动化门。** 四个主题最终各跑一次 `check → preview → check`，再运行一次 bundled-theme 精确测试。没有 Rust、网页或桌面代码变化，因此不运行全量 Rust/Web/Clippy/桌面构建；这些检查不会增加对主题数据的有效覆盖，且会把共享脏工作区的无关状态混入结论。
7. **同步网页目录。** 运行 `corepack pnpm --dir apps/web sync`，只读回四个 ID 的 catalog 和 SQLite 行、包哈希及 ZIP 清单；记录同步前后的非目标差异，不回退并行改动。保存目标产物哈希后再次同步，要求第二次的目标哈希不变。
8. **真实窗口验证。** 在无私人内容的隔离会话中分别应用四个主题。每次 `apply` 前说明准确主题和可能重启 DoubaoWork 的影响，等待当前会话的明确批准；每个主题在普通与窄窗口各观察一次并保存截图。合成预览只证明配色和背景，字体与圆角以真实窗口的计算样式和截图为准。
9. **清理与复核。** 检查八张截图、四个主题包、参考图缺席、指定禁词零命中及目标 diff。应用验证完成后单独请求恢复默认的批准；若未批准，明确报告最后仍在使用的主题。
10. **记录证据。** 运行 `./scripts/devflow verify 2026-08-29-add-portal-github-claude-machine-themes`，在 `verification.md` 写入命令、结果、尺寸、哈希、ZIP 内容、截图、并行工作区边界、偏差和残余风险；由 fresh-context verifier 或人工填写最终 verdict。

## Test-first proof

- 本变更是新增数据包，不修复可复现缺陷，因此不新增会人为重复 authoring 契约的测试文件。
- 实现前运行：
  - `cargo test -p skin-core theme::tests::loads_bundled_themes --locked -- --exact`
  - 目的：确认当前加载器基线可用，避免把已有失败归因到四个新主题。
- 每个主题在编辑循环中运行：
  - `cargo run -p skin-core --bin doubao-theme -- check themes/<id>`
  - `cargo run -p skin-core --bin doubao-theme -- preview themes/<id>`
  - 再次运行同一 `check`
  - 目的：覆盖 v2 字段、作用域、必需语义变量、资源路径、预览尺寸、图片解码与实际加载。
- 四个主题完成后只再运行一次：
  - `cargo test -p skin-core theme::tests::loads_bundled_themes --locked -- --exact`
  - 目的：覆盖 bundled discovery、两种外观、商店字段和最终有效 CSS。该精确测试与四次作者检查已覆盖本次全部代码路径风险。
- 网页同步验证：
  - `corepack pnpm --dir apps/web sync` 连续两次
  - 使用 `jq`、`sqlite3`、`shasum -a 256`、`unzip -l` 只读验证四个目标 ID、哈希和包内容。
- 明确跳过 `./scripts/check.sh rust`、`./scripts/check.sh web`、`./scripts/check.sh all`、workspace Clippy 和桌面构建：本变更不改代码，这些门无法替代真实主题应用，也会扩大到当前大量无关改动。该最小范围由 Sol 针对当前脏工作区和主题契约审阅确认。

## Visual or integration proof

- 用 `view_image` 检查四张最终 `preview.jpg` 与原创 `bg.jpg`：画面完整、无水印、无烘焙文字；背景叙事明确但中央阅读区低细节。
- 用 `sips` 或等价只读工具记录 `bg.jpg`、四张预览和八张窗口截图的像素尺寸；用 SHA-256 固定最终素材证据。
- 每个主题在普通窗口与窄窗口分别检查：
  - 侧栏、标题、正文、代码、输入区和按钮字体是否符合约定，中文无缺字、英文数字无异常跳宽，行高不裁切。
  - 卡片、消息气泡、输入框、按钮和浮层的圆角性格是否一致；焦点环清楚，点击区域不因方角或圆角变化而缩小。
  - 浅色/深色切换后背景、正文、边框、主强调色和 raw RGB 一致，无上一个主题残留色。
  - 720 × 560 附近仍无横向溢出、遮挡或冻结。
- “机械工头”额外检查：机器人催工的喜剧焦点可见、人物为虚构成年人且无伤害；画面明亮、滑稽而不压抑；无参考图文字、舞台纹样、角色、动作、标志或近似构图；表面和遮罩使正文保持至少 4.5:1 对比度。
- 每次 live 应用必须返回至少一个 responsive page；仅端口监听不算成功。验证不得读取页面文字、Cookie、请求头、账号、附件或工作区内容。

## Risks and mitigations

- **共享工作区污染生成目录。** 同步前记录目标和非目标哈希；只对四个新 ID 作结论，任何由已有主题改动触发的生成差异单独记录，不回退也不归入本变更。
- **第三方外观过度相似。** 只使用语义色角色和整体气质，manifest 保留来源边界；不打包标志、字体、图标、吉祥物或页面代码。
- **字体回退导致视觉漂移。** 字体栈以 macOS 系统字体起始并以通用族结束；在中英文混排、数字、代码和粗体中以真实窗口验证。
- **圆角变量覆盖不完整。** 同时设置 `effects.radiusScale`、顶层与两种 variant 的 `composer.radius`，并读取真实窗口计算样式，不依赖合成预览。
- **原创背景贴近参考图。** 不把参考图传给生成器；提示词只保留叙事和色调，明确禁用原图文字、舞台、动作和构图，生成后进行并排人工检查。
- **live 应用影响当前工作。** 每个主题应用前单独告知可能启动或重启目标应用并等待批准；只使用无私人内容的隔离会话，结束后另行获得恢复批准。

## Rollback

- 若实现未达到验收标准，只删除四个明确的新主题目录和它们对应的新生成图片/ZIP，再运行网页同步恢复 catalog 与数据库；不删除整个生成目录，不触碰已有主题。
- 若 live 应用造成异常，在获得明确批准后运行现有 `restore --target doubao-work`，要求至少一个 responsive page 被清理；若 watcher 仍在运行，先报告而不擅自终止未知进程。
- 不使用 `git reset --hard`、`git checkout --` 或任何会覆盖共享工作区的回退方式。
- 回滚不发布、不删除用户安装目录，也不修改 `/Applications/DoubaoWork.app`。

## Deviations

- 尚未实施，因此当前没有偏离已批准 spec 的内容。
- 实现中若必须修改主题引擎、网页同步脚本、预览生成器或任何已有主题，立即停止并更新 spec/plan，重新取得批准后再继续。
- 合成预览当前不证明字体与圆角；这是已知验证边界，不通过修改预览器扩大本次代码范围，改由真实窗口计算样式与截图验收。

## Decision

等待工程负责人确认文件范围、最小检查集、原创素材流程、逐主题 live 审批和回滚边界后，再开始创建主题目录。
