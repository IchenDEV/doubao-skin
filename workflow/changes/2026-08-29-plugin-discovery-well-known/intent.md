---
id: "2026-08-29-plugin-discovery-well-known"
stage: intent
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
source: "user"
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Intent: 标准插件分发与 Skill Well-Known 发现

## Problem

仓库已经有 `create-doubao-theme` 与 `apply-doubao-theme` 两个可工作的 Agent Skill，但目前只以 `skills/<name>/SKILL.md` 裸目录和应用包内资源的形式存在。用户需要先理解仓库路径或手工复制目录，Codex 与 Claude Code 的插件管理器也没有可读取的项目清单、版本、作者、安装源和更新入口；网站同样没有稳定的机器发现地址。

当前两个主流插件宿主使用不同但可并存的正式入口：Codex/ChatGPT 插件以 `.codex-plugin/plugin.json` 和 `.agents/plugins/marketplace.json` 为主，Claude Code 插件以 `.claude-plugin/plugin.json` 和 `.claude-plugin/marketplace.json` 为主。Agent Skills 的 `/.well-known/agent-skills/` 发现机制截至 2026-08-29 仍是 0.2.0 Draft，而不是 Codex 或 Claude Code 已正式要求的插件安装入口。若只实现一套私有 JSON 或把 Draft 误写成正式标准，索引器和用户都会得到不可靠的安装体验。

## Proposed outcome

把两个现有 Skill 作为一个名为 `doubao-skin` 的可版本化插件发布单元，保持一份 Skill 源内容，同时提供：

- 标准 Codex 插件清单与仓库 Marketplace，使 Codex/ChatGPT 能索引、展示并安装两个 Skill。
- 标准 Claude Code 插件清单与 Marketplace，使 Claude Code 能从同一 GitHub 仓库发现、安装和更新插件。
- 网站下的 `/.well-known/agent-skills/index.json` 及其引用的 Skill 产物，按 Agent Skills Discovery Draft 0.2.0 提供名称、稳定 URL、类型与 SHA-256 完整性信息；页面和文档明确标注该发现协议仍为 Draft。
- 使用与下载页提供面向普通用户的 Codex、Claude Code 两条最短安装路径、安装后可用能力以及卸载/更新提示，不要求用户手工寻找 `SKILL.md`。

插件只封装现有两个 Skill 和必要展示元数据，不新增第三套执行逻辑；真正的主题创建、检查、安装和应用继续调用现有 `doubao-theme` Rust CLI。

## Affected users and systems

- 希望在 Codex、ChatGPT 或 Claude Code 中直接安装豆包主题能力的普通用户与主题创作者。
- 需要抓取、校验或收录 Agent Skill 的搜索引擎、目录服务和自动化工具。
- 仓库根级插件元数据、两个现有 Skill、网站静态路由/构建、README 与使用文档。
- GitHub 仓库 `IchenDEV/doubao-skin` 与计划使用的线上域名 `doubao-skin.idevlab.dev`；本变更不改变主题商店数据协议和桌面应用运行时。

## Constraints

- `.codex-plugin/plugin.json`、`.agents/plugins/marketplace.json`、`.claude-plugin/plugin.json` 与 `.claude-plugin/marketplace.json` 必须分别通过当前官方格式或官方 CLI 校验；不依靠宿主宽松解析接受无效字段。
- 两套插件清单必须指向同一份 `skills/` 内容，名称、版本、仓库、许可证和能力文案保持一致；不得复制出会漂移的第二份 Skill。
- 保留 `create-doubao-theme` 与 `apply-doubao-theme` 的职责分离和现有副作用授权门。安装插件不等于授权安装、应用、恢复主题或修改豆包工作。
- 插件安装后仍需可定位 `doubao-theme` CLI；找不到 CLI 时 Skill 必须按现有契约停止并提供可执行安装提示，不能退回手写脚本。
- Well-Known 使用 `/.well-known/agent-skills/` Draft 0.2.0，并通过生成或检查保证 URL、摘要与源 Skill 一致；不得把 Draft 描述成 Codex、Claude Code、IETF 或 Agent Skills 已正式批准的标准。
- 所有公开 URL 使用 HTTPS；清单和索引不得包含凭据、本机绝对路径、用户目录、对话内容或未发布文件。
- 现有工作树含并行主题和网页改动；实现只触碰本变更拥有的插件、Skill 元数据、索引生成/校验和安装文档，不覆盖无关内容。
- 本地实现与预览部署可用于验证；推送 GitHub、提交官方插件目录或切换生产域名仍受各自发布 Gate 约束。

## Out of scope

- 不在本变更提交 Codex/ChatGPT Universal Plugin Directory、Anthropic 官方或社区 Marketplace 审核。
- 不自动安装插件到用户全局 Codex/Claude Code 配置，不修改个人 Marketplace，也不清理已有插件缓存。
- 不把两个 Skill 合并成一个全能 Skill，不重写 Rust CLI、主题标准、桌面主题选择器或在线主题目录。
- 不发明新的通用插件协议、注册新的 IANA well-known suffix，或承诺第三方索引器已经采用 Draft 0.2.0。
- 不在没有单独生产授权的情况下更新线上域名或正式发布 GitHub Release。

## Success signals

- 从全新临时配置开始，Codex 可添加仓库 Marketplace、发现并安装 `doubao-skin`，且能看到并触发两个 Skill；Claude Code 可完成同等的添加、安装、校验和发现流程。
- 两个宿主安装到缓存后的插件都只包含一份有效 Skill 内容，Skill 内引用的 CLI 发现与授权门保持有效，不依赖仓库外相对路径或失效符号链接。
- 官方/随宿主提供的 Codex 插件校验、`claude plugin validate`、两个 Skill 校验和仓库 workflow 检查全部通过；错误清单有确定性失败测试。
- 本地网站构建能提供 `/.well-known/agent-skills/index.json`，其 schema、URL、Content-Type 和每项 SHA-256 与实际公开产物一致；索引中恰好包含两个现有 Skill。
- README 与“使用与下载”页各给出可复制的 Codex 与 Claude Code 安装命令、更新方式、Skill 名称、CLI 前置条件和 Draft 发现地址，同时不增加首页干扰或破坏深色模式、窄屏和现有 SEO/GEO。
- 若 GitHub 仓库或生产域名尚未发布，Verification 明确区分本地/预览成功与公网待验证，不把计划中的可安装性写成已上线。

## Open questions

- 两套 Marketplace 能否在仓库根目录直接把同一目录作为插件根，还是 Codex 当前校验器要求 `plugins/doubao-skin/` 子目录，需要在 Spec 前用当前 CLI 和隔离临时仓库验证。若必须使用子目录，应迁移权威 Skill 路径而不是复制内容。
- Codex 与 Claude Code 对双清单共存时的版本优先级和显示元数据差异，需要以当前校验器与干净安装结果为准；公共版本号的唯一事实来源将在 Spec 中确定。
- Well-Known Draft 的 Skill 条目应直接发布单文件 `SKILL.md`，还是发布包含 `agents/openai.yaml` 的归档，需要根据索引消费者兼容性和两个 Skill 当前无脚本的事实选择最小形式。

## Decision

等待产品所有者审阅。接受本 Intent 只授权进入 Spec 并完成目录形状与当前宿主行为验证；不授权修改插件/网站实现、安装到个人环境、推送 GitHub、提交公共目录或生产部署。
