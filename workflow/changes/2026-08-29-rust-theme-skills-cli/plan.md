---
id: "2026-08-29-rust-theme-skills-cli"
stage: plan
status: accepted
owner: "Codex implementation agent"
created: "2026-08-29"
based_on: spec.md
risk: "high"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Plan: rust theme skills cli

## Files and ownership

- `crates/skin-core/src/authoring.rs`（新）：主题创建、严格检查、预览绘制和选择性打包的单一作者入口；不复制主题加载或安装实现。
- `crates/skin-core/src/lib.rs`、`crates/skin-core/src/theme.rs`、`crates/skin-core/src/live.rs`：只暴露 CLI 所需的最小复用接口、可用主题定位和可验证恢复语义；保留现有桌面调用方。
- `crates/skin-core/src/bin/doubao-theme.rs`（新）：手写参数解析、中文文本输出、稳定 JSON envelope 与退出码；调用 authoring/theme/live/build，不承载第二套业务逻辑。
- `crates/skin-core/tests/doubao_theme_cli.rs`（新）：从真实二进制验证创建、检查、预览、打包、安装、帮助、JSON 和失败退出码。
- `skills/create-doubao-theme/`、`skills/apply-doubao-theme/`（新）：各自的 `SKILL.md`、`agents/openai.yaml`；仅当高级创作确需语义变量说明时，为生产 Skill 增加一份聚焦 reference，不添加脚本层。
- `scripts/build-macos.sh`：按 host/universal 同时构建 GUI 和 `doubao-theme`，把 CLI 放入 `Contents/Resources/bin/`，把两个 Skill 放入 `Contents/Resources/skills/` 后统一签名。
- `README.md`、`docs/README.md`、必要的现有开发/发布文档：CLI、Skill 安装、发现、调用、兼容边界和包内位置；不创建重复的完整主题规范。
- `workflow/changes/2026-08-29-rust-theme-skills-cli/verification.md`：实际命令、测试、包内容、隔离 Live 应用/恢复和残余风险。
- 不拥有 `themes/*` 的视觉内容、`apps/web/data`、`apps/web/public/themes` 或无关桌面界面；与网页变更共享的 README/文档按本 Plan 先完成，网页变更随后顺序修改。

## Order of work

1. **冻结真实契约并写失败测试**
   - 从当前 `Theme`、`install_theme_package`、`live::run/restore`、`build::apply/remove` 和主题 v2 标准提取 CLI 需要的最小接口。
   - 先添加 authoring 单元测试与 CLI 集成测试：合法创建链、非空目录拒绝、ID/目录不一致、缺失资源、错误 CSS scope、不完整双外观、选择性打包、符号链接、JSON envelope 和退出码。确认测试先因功能缺失而失败。
2. **实现小型 authoring 模块**
   - 使用一个必要的创建选项结构和四个直白函数：create、check、preview、pack。所有命令先走现有 `theme::load`，严格作者规则只叠加在 v2 创作路径，不改变 v1 兼容加载。
   - `create` 写入最小完整 `theme.json`/`theme.css`，默认纯色或渐变；使用 `Theme::preview_style()` 与现有 `image` 依赖绘制 1200×675 界面预览。
   - `pack` 只收入清单实际引用资源与许可证，复用安装限制的路径安全边界，拒绝符号链接、路径逃逸、隐藏/无关文件和目标覆盖。
3. **实现 `doubao-theme` CLI**
   - 明确实现 `list/create/check/preview/pack/install/apply/restore/build/remove-build`、`--json`、`--target doubao|doubao-work` 和 `--watch`。
   - ID/路径定位顺序为显式目录 → 用户安装主题 → 内置主题；安装继续调用 `install_theme_package`，在线应用调用 `live::run`，离线构建只调用现有 DoubaoWork clone 路径。
   - `apply` 默认一次性并要求至少一个响应页面；`restore` 的 CLI 成功必须观察到实际清理，未开放端口或零响应页返回外部操作失败，不能把端口存在当成完成。
   - 文本输出使用普通中文；JSON stdout 只输出 `{ok, command, result|error}`，日志走 stderr；参数/主题/外部失败分别映射 2/3/4。
4. **创建两个聚焦 Skill**
   - 先读取 Skill Creator 的 `openai_yaml` 参考，再生成两个目录和一致 UI 元数据，保持默认自动发现。
   - `create-doubao-theme` 将自然语言需求转为 CLI 参数，必要时小范围修改生成 CSS/资源，然后强制 check → preview → pack；版权不清或素材缺失时回退纯色/渐变或请求素材。
   - `apply-doubao-theme` 的 list/check 为只读；install/apply/restore/build/remove-build 在执行前显示准确目标和影响并等待明确授权。找不到 CLI 时按 `DOUBAO_THEME_CLI` → PATH → App Resources 顺序诊断并停止。
   - 运行 `quick_validate.py`，再在隔离临时目录做独立自然语言创作 forward-test；不把测试主题写入真实用户主题目录。
5. **接入 macOS 包**
   - 为每个目标构建 CLI；universal 模式对 GUI 与 CLI 各自 `lipo`，host 模式直接复制。CLI 与 Skills 在 codesign 前进入 App Resources。
   - 增加 shell 语法和 host smoke：验证 CLI 可执行、两个 `SKILL.md` 存在、架构与 GUI 对齐、codesign 深度校验仍通过。
6. **文档与隔离运行验收**
   - 更新 README/docs 的源码运行、GitHub `$skill-installer` 提示、发现/调用示例、包内位置和豆包工作未公开导入协议的边界。
   - 完成 Rust/workflow gates；随后只在无私人内容的独立豆包工作窗口执行 apply/restore，记录 DOM、计算样式和截图。若需要重启，先确认没有正在进行的工作。
   - 记录 Verification，交给新上下文 verifier；不创建 GitHub Release、不发布 Skill、不修改生产网站。

## Test-first proof

- `authoring` 单元测试先覆盖：最小 v2 主题、both variants、kebab-case、目录名、资源存在/越界、CSS 同时覆盖 html/body、必需语义变量、预览尺寸、可重复预览和选择性 ZIP 条目。
- CLI 集成测试使用独立临时目录和 `DOUBAO_SKIN_THEMES_DIR`/`DOUBAO_SKIN_USER_THEMES_DIR`，验证完整 create → check → preview → pack → install → list 链；不写真实用户目录。
- 为未知命令、缺参、非法主题和不可达目标断言精确退出码 2/3/4；`--json` 每次只解析一个 JSON 对象，stdout 不混日志。
- 保留并扩展 `install_theme_package` 的路径穿越、符号链接、大小、文件数和更新回滚测试；新 pack 产物必须能被现有安装函数重新读取。
- `live::restore` 增加脚本所有权和零响应失败回归；实际 CDP 行为不使用仅字符串匹配代替最终验收。
- 迭代命令：`cargo test -p skin-core --lib authoring`、`cargo test -p skin-core --test doubao_theme_cli`；完成后运行 `./scripts/check.sh rust` 与 `./scripts/check.sh workflow`。

## Visual or integration proof

- 检查生成 `preview.jpg` 为 1200×675、与输入强调色/外观一致且无解释型文案；使用本地图像查看工具留证。
- 对两个 Skill 各运行 `quick_validate.py`；生产 Skill 在临时目录完成真实自然语言 dry-run，应用 Skill 证明 list/check 可直接执行且 install/apply/restore 在未授权时停止。
- host App 包检查 `Contents/Resources/bin/doubao-theme`、两个 Skill 目录、CLI `--help`、Mach-O 架构和 `codesign --verify --deep --strict`。
- 真实 Live 验收只针对独立豆包工作窗口：应用后 `data-skin`、主题 style 与关键 computed style 匹配；恢复后运行时、backdrop、style、图标/输入区标记消失且原外观属性恢复。
- 通用包若本机工具链可用则实际构建并用 `lipo -info` 检查双架构；签名/notarization 凭据不存在时不伪造 notarization 成功。

## Risks and mitigations

- **严格检查破坏旧主题**：严格规则只用于 authoring 命令，现有 `Theme::load` v1 路径保持兼容，并保留全部主题加载回归。
- **打包遗漏或带入多余文件**：由解析后的清单资源白名单生成 ZIP，再用现有安装器回读；不递归打包整个目录。
- **恢复错误清空用户外观**：runtime 只恢复自身记录的原始属性；先做合成脚本测试，再做同一真实页面 apply/restore 对照。
- **Skill 绕过授权**：副作用命令集中在 apply Skill，正文要求临执行授权；自动发现不等于自动执行。
- **CLI 与 GUI 架构/签名不一致**：两者在同一目标循环构建、分别 lipo、统一在复制资源后签名，包内 smoke 明确断言。
- **豆包工作 Skill 协议不明确**：交付标准 Skill 与 Codex 安装路径，只记录兼容边界，不逆向或宣称原生可安装。
- **脏工作树重叠**：只编辑列出的核心、Skill、打包和文档文件；发现并发修改时停在冲突文件，不覆盖用户工作。

## Rollback

- 在未发布状态下，回退本变更新增的 authoring、CLI、Skill 和对应测试，并恢复 `lib.rs`/打包脚本/文档的局部接线；不触碰其他工作树改动。
- 若 CLI 包集成导致 GUI 包失败，先从 App Resources 复制步骤撤出 CLI/Skill，恢复原 GUI 构建链，再保留源码 CLI 继续诊断。
- 若真实 Live 测试失败，立即运行已验证的 restore 路径或退出测试进程；离线 clone 只删除 `~/Applications/DoubaoWork-Skin.app`，官方 `/Applications/DoubaoWork.app` 永不修改。
- 本 Plan 不涉及线上发布，因此不需要 GitHub Release 或网站回滚。

## Deviations

- 当前无计划偏差。若现有主题模型无法安全枚举全部清单资源，优先为 `Theme` 增加一个窄的只读资源枚举接口，不在 authoring 中复制第二套 JSON schema 解析器；需记录在 Verification。
- 若未获得真实豆包工作独立窗口，自动化、Skill dry-run 和包验证可完成，但 Live apply/restore 与最终 verdict 必须标为 blocked/pending，不能写 verified。

## Decision

等待产品与风险负责人接受、要求修改或拒绝本 Plan。接受只授权按上述边界实现、运行隔离测试和准备分发产物；不授权推送 GitHub、创建 Release、安装到豆包工作未公开目录或发布生产内容。
