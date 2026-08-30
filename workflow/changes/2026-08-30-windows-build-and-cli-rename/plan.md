---
id: "2026-08-30-windows-build-and-cli-rename"
stage: plan
status: accepted
owner: "Cursor implementation agent"
created: "2026-08-30"
based_on: spec.md
risk: "critical"
approved_by: "idevlab"
approved_at: "2026-08-30"
---

# Plan: Windows 原生打包与 CLI 重命名

## Files and ownership

- `scripts/package.sh` — 桌面、CLI、Scoop 与签名校验的唯一公开打包入口
- `scripts/package/` — 按产物职责分组的内部实现；Windows 入口只允许在 Windows 主机执行
- `scripts/checks/` — 工作流与跨平台静态回归检查
- `.github/workflows/ci.yml` — Windows 编译检查作业
- `.github/workflows/release.yml` — 独立桌面/CLI 矩阵、Scoop 清单与统一发布门禁
- `Cargo.toml` — 移除 gpui_windows 本地 patch
- `apps/desktop/Cargo.toml` — 二进制名改为 doubao-skin-app
- `apps/desktop/build.rs`, `assets/app-icon/AppIcon.ico` — Windows PE 应用图标资源
- `crates/skin-core/src/bin/doubao-skin.rs` — CLI 源文件（从 doubao-theme.rs 重命名）
- `crates/skin-core/tests/doubao_skin_cli.rs` — 测试文件（从 doubao_theme_cli.rs 重命名）
- `apps/desktop/src/{app,ui,preview,store}/` — 按职责整理桌面源码
- `apps/desktop/src/main.rs`, `apps/desktop/src/ui/assets.rs` — 平台标题栏与本地图片资源类型
- `crates/skin-core/src/live.rs` — Windows 双应用发现、启动和退出
- `crates/skin-core/src/ws.rs`, `crates/skin-core/Cargo.toml` — 跨平台 WebSocket 随机源与 Windows 首次注入失败回报
- `scripts/package/verify-windows-exe.sh` — 打包前验证 GUI 与图标资源
- `scripts/package/macos.sh` — 移除内嵌 CLI、Skills 和独立 CLI 压缩包
- `scripts/package/cli.sh`, `scripts/install-cli.sh` — 独立多平台 CLI 打包与安装链
- `scripts/package/verify-macos-signature.sh` — App 与 CLI 共用的稳定证书指纹验证器
- `scripts/package/generate-scoop-manifest.mjs` — 从当次 Windows CLI 哈希生成 Scoop 清单
- `apps/web/src/components/DesktopDownloads.tsx`, `apps/web/src/lib/downloads.ts` — 浏览器本地桌面平台推荐
- `apps/web/scripts/skill-discovery.test.mjs` — Cargo、Web 与两份插件清单的版本一致性门禁
- `README.md`, `README.en.md`, `CHANGELOG.md` — CLI 名称更新
- `Cargo.toml`, `Cargo.lock`, `apps/web/package.json`, `plugins/doubao-skin/.{codex-plugin,claude-plugin}/plugin.json` — `0.4.0` 版本统一
- `docs/development.md`, `docs/submitting-themes.md`, `docs/releasing.md` — CLI 名称更新
- `plugins/doubao-skin/skills/*/SKILL.md` — CLI 引用更新
- `apps/web/public/.well-known/agent-skills/*/SKILL.md` — CLI 引用更新
- `AGENTS.md` — cargo run 命令更新

## Order of work

1. Windows 构建脚本和 CI 配置
2. gpui_windows 最小临时交叉编译修正
3. CLI 二进制重命名（源码 + Cargo）
4. 全仓库文档/脚本/网站 CLI 引用更新，并把安装方式收敛为源码开发工具
5. 测试文件重命名和更新
6. 运行检查验证
7. 由 Windows runner 原生构建并检查三架构 ZIP
8. 根据 Windows 实机反馈增加四项红灯回归：标题栏、文件图片、安装发现、PE 图标
9. 修复 Windows 运行时分支并重新产出全部三架构 ZIP
10. Windows 桌面包收敛为唯一 `doubao-skin.exe`，macOS 只在应用包内保留 GUI
11. 按用户澄清恢复独立 CLI 发布链，加入 Scoop、macOS/Linux 安装脚本和 Web 桌面平台识别
12. 根据 ARM64 实机应用主题一直停在“正在应用”的复现，修复 Windows WebSocket 随机源，并让首次注入连续失败在 30 秒后结束而不是无限等待
13. 根据覆盖安装后的 ARM64 实机复现，优先选择 `Application/` 下的官方外层启动器，并把 Windows 目标应用的启动工作目录固定为该 EXE 所在目录，避免直接运行 `Application/app/` 内部二进制后触发 `mcp_helper.dll` 缺失
14. 全仓库兼容性审计：用户数据目录改用跨平台目录 API；live/offline 能力在进入路径和系统命令前明确分流；移除示例中的外部 `curl` 依赖；Windows 不注册 macOS 菜单和快捷键
15. 将脚本整理为 `package/` 与 `checks/` 两组并保留单一 `scripts/package.sh` 入口；将 CI 与 release 的 Windows 构建迁移到 `windows-2025` 原生 runner
16. 将发布候选统一为 `0.4.0`；在 `lipo` 合并后重签 macOS universal CLI，普通 CI 用 ad-hoc 签名检查结构，Release 与 App 共用长期身份和固定证书指纹

## Test-first proof

- 重命名测试文件后运行 `cargo test -p skin-core` 验证 CLI 行为契约不变。
- 在修复 Windows 实时注入前加入随机源和首次注入超时回归；测试先因缺少超时实现而编译失败，再完成修复。
- 运行 `./scripts/check.sh workflow` 验证 SDLC 工件。
- 修复前解包 universal CLI 后运行 `codesign --verify --strict`，确认 `lipo` 输出报 `code object is not signed at all`；修复后要求双架构、严格签名、校验和、归档内容和 `--version` 全部通过。

## Visual or integration proof

- Windows 包由远端真实 Windows runner 编译并检查归档内容，再交给用户实机验证；本机交叉编译只作为可选调试手段。
- 用户首轮实机截图作为失败证据；修复包继续由同一 Windows 环境复测窗口控件、预置图片、双应用检测与 EXE 图标。
- macOS 构建通过 CI 验证。

## Risks and mitigations

- 风险：遗漏某处 doubao-theme 引用。缓解：使用 rg 全量搜索验证。
- 风险：测试中硬编码二进制名。缓解：同步更新 CARGO_BIN_EXE 引用。

## Rollback

- git revert 整个变更集即可回退。

## Deviations

- 用户在审查后明确要求移除原计划中的整份 `patches/gpui_windows` vendored patch。
- 原实现将桌面源码平铺为 26 个文件；按用户要求改为四个职责目录，保留拆分后的可读性。
- 早期方案曾从 Windows runner 改回 macOS + xwin；用户随后明确否决该方向，要求远端 CI 必须在 Windows runner 上编译 Windows 版本。最终删除 xwin 与 GPUI 临时补丁链，只保留 Windows 原生构建。
- 实际打包发现完整包与 CLI 包只差大小写会在 macOS/Windows 上互相覆盖；CLI-only 包改为 `doubao-skin-cli-Windows-<arch>.zip`。
- i686 的 psm 汇编对象不带 SafeSEH 表，仅该架构关闭 SafeSEH 链接检查；ARM64 的 ring 构建脚本固定调用 clang，因此仅该架构改用 clang 参数风格。
- 首轮 Windows 实机测试暴露四项跨平台缺口：GPUI 透明标题栏在 Windows 隐藏了原生控件；盘符路径字符串被识别为 URI；live 模式仍硬编码 macOS 应用路径与进程命令；PE 仅有 manifest、没有图标资源。用户明确要求在同一变更内修复并重打包。
- 第二轮 Windows 实机截图确认原生窗口控件、图标、主题图片和应用检测已恢复；按用户反馈移除标题栏里与内容区重复的“豆皮”文字，保留系统图标与窗口控件。
- 用户随后澄清要求不是取消 CLI，而是桌面与 CLI 使用互不嵌套的两条安装链。桌面包继续保持单入口且不内嵌 CLI/Skills；独立 CLI 恢复为多平台 Release 资产，Windows 由 Scoop 发现安装，macOS/Linux 使用独立安装脚本。Windows 的内部 Cargo GUI 产物仍名为 `doubao-skin-app` 以避开源码 CLI 冲突，但发布时复制为唯一桌面入口 `doubao-skin.exe`。
- ARM64 实机复现确认 `/json` 能发现三个正确的豆包页面，但 native Windows 不存在 `/dev/urandom`；原 WebSocket 握手因此在写出请求前失败。实现改用 `getrandom` 的系统随机源，并保留最后一次首次注入错误，在 30 秒内仍无任何页面成功时返回失败，防止桌面端永久显示“正在应用”。
- ARM64 覆盖安装验证确认官方豆包直接启动正常，但主题工具为打开 CDP 端口而重启后立即报 `mcp_helper.dll` 缺失。安装目录同时存在 `Application/Doubao.exe` 外层入口和 `Application/app/Doubao.exe` 内部二进制；Windows 分支原先优先选中内部二进制并继承主题工具工作目录。现改为优先外层入口、从入口所在目录启动，并加入入口优先级与工作目录回归测试。
- 用户要求进一步减少系统强相关实现并全项目扫描。审计后把用户数据与缓存路径交给跨平台目录库，将无法跨平台的实时应用和 macOS 离线克隆明确隔离在能力边界内，并清除外部 `curl` 与非 macOS 平台上的 macOS 菜单注册。
- 用户要求整理多语言脚本。对外入口收敛到 `scripts/package.sh`，实现按 `package/` 和 `checks/` 分组；远端 Windows CI 直接调用同一入口，避免 CI 与本地脚本分叉。
- 用户准备发布 `v0.4.0`，要求 CLI 暂时沿用 App 的签名办法。实现将 universal CLI 留在同一 macOS Release 作业中，在一次证书导入后分别签名 App 与合并完成的 CLI，并对两者执行同一固定指纹校验；普通 CI 只做 ad-hoc 结构验证，不接触生产签名材料。

## Decision

用户在 2026-08-30 明确要求按上述修复与分组方案继续，并立即产出 Windows 测试包；同日进一步确认桌面发布包只保留简洁入口，同时 CLI 必须保留为完全独立、可由 Agent 自动发现安装的跨平台链路。2026-08-31 用户明确准备发布 `v0.4.0`，要求先确认版本正确，并让 macOS CLI 临时复用 App 已有的稳定签名方案。
