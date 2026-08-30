---
id: "2026-08-30-windows-build-and-cli-rename"
stage: spec
status: accepted
owner: "Cursor implementation agent"
created: "2026-08-30"
based_on: intent.md
risk: "critical"
approved_by: "idevlab"
approved_at: "2026-08-30"
---

# Spec: Windows 原生打包与 CLI 重命名

## Requirements

1. 对外只保留 `scripts/package.sh` 打包入口；内部按桌面平台、CLI、校验和可选本地交叉构建分组。
2. CI 和 release 工作流必须在真实 Windows runner 上原生编译 x64、x86、ARM64 Windows 资产，不以 macOS 交叉编译作为通过证据。
3. CLI 名统一为 `doubao-skin`，不进入桌面发布包，通过独立的 macOS、Linux 与 Windows CLI-only 资产发布。
4. 移除仓库中的 `patches/gpui_windows` vendored patch，优先使用固定 revision 的上游 GPUI。
5. 桌面应用 Cargo 二进制改名为 `doubao-skin-app` 避免与 CLI 冲突。
6. 桌面源码按 `app/`、`ui/`、`preview/`、`store/` 分组，根目录只保留入口、国际化和回归测试。
7. Windows 平铺 ZIP 只含一个顶层 `doubao-skin.exe`，并能发现同级 `themes/`。
8. Windows 桌面窗口保留原生标题栏及关闭、最小化按钮，但标题栏不重复显示“豆皮”；macOS 继续使用透明标题栏与自定义交通灯位置。
9. 主题预览图、背景图和位图图标在 Windows 上必须按本地文件路径加载，不能把盘符误判为 URL scheme。
10. Windows 安装检测同时覆盖豆包和豆包工作：默认系统本地数据目录、Windows Apps/ARP 注册表、自定义路径环境变量，并能用解析出的可执行文件启动目标应用；产品代码不得直接读取平台目录环境变量。
11. 打包后的 `doubao-skin.exe` 必须内嵌多尺寸应用图标，并在归档前验证 `ICON` 与 `GROUP_ICON` PE 资源。
12. Windows CLI 提供 Release 生成的 Scoop 清单，按 x64、x86、ARM64 选择资产并校验 SHA-256。
13. macOS/Linux CLI 安装脚本必须自动识别 macOS 通用版、Linux x64 或 Linux ARM64，并校验 sidecar SHA-256。
14. Web 下载页必须只在浏览器本地检测平台，不上传用户代理信息；推荐 macOS 或对应 Windows 架构，同时保留全部桌面版本的手动下载链接。
15. macOS universal CLI 必须在合并 x86_64 与 ARM64 切片后重新签名；Release 使用与 App 相同的长期社区身份和固定 SHA-256 证书指纹，普通 CI 使用 ad-hoc 身份验证签名结构但不得访问生产密钥。
16. `v0.4.0` 的 Cargo 工作区、Web 包和 Codex/Claude 插件清单版本必须一致；Tag 校验继续以 Cargo 工作区版本为权威来源。

## User experience

- 用户从 GitHub Release 下载 Windows ZIP，解压即可运行。
- 桌面用户不需要区分 GUI 和 CLI；Windows 解压后只看到一个可执行入口。
- CLI 用户不需要安装桌面应用；Agent 可先发现 PATH 中的 `doubao-skin`，缺失时按当前系统选择 Scoop 或安装脚本。
- 浏览器用户首先看到当前设备推荐的桌面版本，但可以手动改选架构。

## Technical design

- Windows CI 工具链：`windows-2025` runner 安装相应 MSVC target，直接执行 Cargo 构建。三个架构均在 Windows 操作系统上构建。
- Windows 打包脚本在非 Windows 主机上直接失败；仓库不保留 GPUI 补丁或 cargo-xwin 备用路径，避免出现两套构建行为。
- 桌面应用内部 Cargo 二进制名为 `doubao-skin-app`，Windows 打包时复制为唯一用户入口 `doubao-skin.exe`。
- macOS 应用包只含 GUI、主题和许可证；独立 CLI 构建矩阵另行发布 macOS universal、Linux x64/ARM64 和 Windows x64/x86/ARM64。
- Scoop 清单由当次 Windows CLI 包的真实哈希生成并作为 `doubao-skin.json` Release 资产发布。
- Windows 使用非透明 GPUI 标题栏恢复系统窗口控件；macOS 标题栏策略不变。
- 所有主题位图向 GPUI 传递 `PathBuf`/文件资源，不再把绝对路径降级为字符串资源。
- Windows 目标应用解析优先使用显式覆盖和默认用户目录，再读取卸载注册表与常用程序目录；运行控制使用目标 EXE、`taskkill` 和 CDP 参数，不调用 macOS 命令。
- 桌面构建脚本从现有产品图标生成的 `.ico` 编译 PE 资源；Windows 打包脚本在归档前强制检查资源表。
- 跨平台目录使用 `dirs` API；平台进程和应用发现集中在 `live/platform.rs`，macOS 离线克隆实现集中在 `build/macos.rs`，并由编译期模块边界隔离。

## Security and privacy

- 不引入新的网络连接或权限需求。
- 不扩大渲染或网络安全边界。

## Alternatives and non-goals

- 不把 macOS 交叉编译结果当作 Windows CI 或发布门禁证据。
- 不提供 MSI/MSIX 安装包。
- 不改名 Skill 标识符（create-doubao-theme 等保持不变）。

## Areas of concern

- Windows ARM64 虚拟机 DirectX 兼容性需实机验证。
- clang-cl 包装脚本对新依赖可能需要扩展。

## Acceptance criteria

- 三架构 Windows 桌面与 CLI ZIP 均由 Windows runner 构建，并在普通 PR CI 中持续编译与留存测试产物。
- 发布任务只有在 macOS 与真实 Windows runner 上的全部 Windows 矩阵成功后才允许上传 Release。
- 普通 PR CI 必须构建、校验和、解包并严格验证 macOS universal CLI 的双架构与签名；Release 还必须核对 CLI 与 App 的同一证书指纹。
- 每个完整 Windows 包只运行顶层 `doubao-skin.exe`，归档内不得出现第二个顶层 EXE。
- Release 同时上传桌面资产和独立 CLI-only 资产，但 macOS 应用包不得含 `Contents/Resources/bin` 或打包 Skills，Windows 桌面 ZIP 也不得含 CLI-only 文件。
- Scoop 清单的三种架构 URL 和哈希必须与同一 Release 的 CLI-only 资产一致。
- macOS/Linux 安装脚本不得下载桌面资产；Web 平台识别不得触发自动下载。
- `rg 'doubao-theme'` 在产品代码/文档中仅匹配 DOM 属性和 Skill 名称，不再匹配 CLI 引用。
- 现有测试通过。
- Windows 标题栏、盘符路径资源分类、双应用默认目录及注册表身份隔离有回归测试。
- 每个 Windows GUI EXE 在归档前通过 PE `ICON`/`GROUP_ICON` 和 GUI subsystem 检查。
- `scripts/checks/portability.sh` 对直接平台目录环境变量、硬编码 `/Applications`、相对数据目录、运行时外部 `curl`、平台命令越界和大小写路径碰撞进行静态回归检查。

## Decision

用户在 2026-08-30 明确确认上述构建目标；同日实机测试发现无图标、无关闭按钮、预置主题图片空白和目标应用误报未安装，并明确要求修复后重新产出测试包。后续用户明确废止“macOS 交叉编译作为 CI”的方案，要求整理多语言脚本并由远端真实 Windows 设备完成 Windows 编译验证。2026-08-31 用户明确指定下一版本为 `0.4.0`，并要求 macOS CLI 的临时签名处理与 App 保持一致。
