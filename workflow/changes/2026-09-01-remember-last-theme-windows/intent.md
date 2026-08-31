---
id: "2026-09-01-remember-last-theme-windows"
stage: intent
status: accepted
owner: "codex"
created: "2026-09-01"
source: "user"
risk: "high"
approved_by: "user"
approved_at: "2026-09-01"
---

# Intent: remember last theme windows

## Problem

上一项变更为 macOS 13+ 增加了“自动保持上次主题”和“登录时打开豆包”两个开关，但 Windows 明确返回 `Unsupported`。Windows 用户即使已经成功应用主题，从桌面、开始菜单或豆包自身开机启动再次打开目标时，仍会先得到没有主题的普通进程；关闭豆皮主窗口后也没有后台所有者继续观察目标。仓库虽然已有 Windows 目标识别、实时主题和 x64/x86/ARM64 原生打包链，但自动主题的启动注册、无窗口 helper、主程序交接和真实 Windows 验收尚未连接。

## Proposed outcome

让 Windows 10/11 用户看到并使用与 macOS 相同的两个开关，不增加第三个设置或 Windows 专用设置页：

- 开启“自动保持上次主题”后，只为当前 Windows 用户注册并立即启动一个无窗口的豆皮 helper。helper 在登录时运行、主豆皮运行时让出、主豆皮关闭后接管，并在用户之后手动启动保存目标时恢复最后成功主题。
- “登录时打开豆包”继续是父项的从属开关。关闭时登录只启动隐藏 helper，不启动豆包；开启时每次 Windows 登录会话最多主动打开一次保存目标。
- 关闭父项或成功“恢复默认”时，配置、登录启动项和 helper 生命周期形成同一可恢复事务，不移除当前页面主题、不修改官方豆包文件或官方启动设置。

## Affected users and systems

- Windows 10/11 上使用便携 ZIP 中豆皮 GUI 的 x64、x86 与 ARM64 用户。
- `skin-core` 的跨平台自动主题状态/supervisor、Windows 进程识别与 live watcher。
- Windows 桌面平台适配器、无窗口 agent binary、资源/打包脚本、Windows CI 与 Release 产物。
- 现有 macOS 两开关、SMAppService helper、Web 主题库和 CLI 行为必须保持不变。

## Constraints

- 仍然只有两个开关；父项启动的是豆皮后台 helper，不等于启动豆包，子项才控制登录后是否主动打开保存目标。
- 只使用当前用户、无需管理员权限、可由产品精确撤销的 Windows 登录启动机制；不得安装系统服务、计划任务、驱动、全局 daemon 或写入官方豆包目录。
- helper 必须无控制台窗口、无托盘图标、无网络 listener、无第二份主题加载器；复用 `skin-core` 严格配置、目标身份、单 watcher supervisor、停止策略和 audit/login-session 去重语义。
- Windows 包移动、路径含空格/中文、重复开启/关闭、目标或主题缺失、配置损坏、启动项被外部删除、helper 崩溃重启均需安全停止或幂等恢复，不产生启动循环和模糊进程结束。
- Windows 原生构建和 VM 验收是完成条件。Mac 上因缺少 MSVC SDK 的交叉检查失败只能记录为环境边界，不能替代 `windows-2025` CI 或真实 Windows 11 ARM/x64 运行证据。
- 不修改 `/Applications/DoubaoWork.app` 或 Windows 官方豆包安装文件；主题仍只通过已验证身份的 loopback CDP 路径工作。

## Out of scope

- Windows 安装器、MSIX/Store `StartupTask`、系统服务、管理员自启动、托盘常驻 UI、通知中心、设置页或自动更新迁移器。
- 修改豆包/豆包工作自身的开机启动选项，或保证与官方启动项竞态时零帧默认外观。
- Windows 7/8、Wine、Linux 自动保持，以及将 macOS SMAppService 替换为跨平台抽象框架。
- 本变更不发布、合并或修改生产 Release；只完成实现、原生包/VM 验证与可审核证据。

## Success signals

- Windows x64、x86、ARM64 原生 CI 均能构建 GUI、CLI 和对应架构的无窗口 helper；产物结构、PE 架构、资源、校验和与现有下载命名契约通过。
- Windows UI 中两个 switch 可操作且从属关系与 macOS 一致；父开子关时登录后没有豆包/豆皮窗口，父关后启动项不存在且 helper 退出。
- 当前会话实测：注册幂等、helper 无控制台、主程序运行时 helper 0 个 watcher、主程序关闭后 helper 接管、重开主程序后 helper 让出。
- 手动从开始菜单/桌面启动保存目标后，最多一次受控重启并恢复保存主题；用户主动退出目标后至少 30 秒不重开。
- 子项开启的全新 Windows 登录会话最多主动打开保存目标一次；helper 重启、主程序开关和官方启动项竞态不产生重复目标或双 watcher。
- 测试后精确注销启动项、退出 helper、恢复测试前主题/配置/窗口状态；官方豆包安装文件未改变。

## Open questions

- 当前便携 ZIP 最小且可逆的登录启动机制，优先评估当前用户 `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run` 的精确值；若它不能可靠覆盖路径、禁用状态或三架构原生验收，再回到 Spec 比较 Startup shortcut，而不直接升级为计划任务/系统服务。
- macOS 使用 audit session ID 去重。Windows 需要用当前登录会话可稳定取得且重启 helper 后不变的标识；具体 API/持久标记在 Spec 原型中确定。
- 现有 Windows ARM VM 可用于 GUI/进程/端口验收，但 x64/x86 的原生构建与 PE 检查以 Windows CI 为最低门；是否另有 x64 VM 不阻塞 Intent 接受。

## Decision

等待用户明确接受本 Intent。接受后再起草 Windows 平台注册、会话标识、包结构、测试矩阵和恢复路径的 Spec；不得把当前一句“兼容下 Windows”自行解释为已批准具体注册表/API 设计。
