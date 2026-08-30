---
id: "2026-08-30-windows-build-and-cli-rename"
stage: intent
status: accepted
owner: "Cursor implementation agent"
created: "2026-08-30"
source: "user"
risk: "critical"
approved_by: "idevlab"
approved_at: "2026-08-31"
---

# Intent: Windows 原生构建、兼容性与 CLI 重命名

## Problem

豆皮原本仅有 macOS 构建流水线。用户需要在 Windows 设备（x64、x86、ARM64）上验证桌面应用，但缺少由 Windows 主机执行的原生编译、打包脚本和 CI 作业。早期方案又把开发者 CLI 嵌入完整包并额外发布 CLI-only 包，造成多个入口和重复下载物。

## Proposed outcome

1. 由远端 Windows runner 原生生成 Windows x64、x86、ARM64 三个架构的桌面应用，每个包只有一个 `doubao-skin.exe` 入口。
2. CI release 工作流新增 Windows 构建作业，产出带校验和的 ZIP 包。
3. CLI 统一命名为 `doubao-skin`，不嵌入桌面包，通过独立的多平台 Release 资产安装。
4. Windows CLI 提供 Scoop 清单，macOS/Linux 提供自动识别平台并校验哈希的安装脚本。
5. Web 下载页在浏览器本地识别 macOS 或 Windows 与可用架构，推荐正确桌面包并保留手动选择。
6. 删除 GPUI vendored patch 与 macOS 交叉编译备用链，只保留上游 GPUI 和 Windows 原生构建。
7. 将桌面源码按 app、ui、preview、store 分组，避免单目录平铺大量文件。
8. 为 `v0.4.0` 统一工作区、Web 与插件版本；macOS universal CLI 在 `lipo` 合并后复用桌面 App 的长期稳定社区签名身份。

## Affected users and systems

- 需要 Windows 验证的开发者和测试用户。
- 所有阅读 README、网站指南、投稿文档和 Skill 说明的用户。
- CI/CD 流水线和构建脚本。
- 插件 Skill 中对 CLI 路径的硬编码引用。

## Constraints

- Windows 发布资产必须由真实 Windows runner 原生构建；不保留 xwin 或 GPUI 本地补丁备用链。
- 不修改技能名（create-doubao-theme、apply-doubao-theme），仅更新其内部对 CLI 二进制的引用。
- 不修改 DOM 注入属性名（data-doubao-theme-icon、data-doubao-theme-composer），它们是运行时协议的一部分。
- 优先使用上游 gpui_windows，不在仓库中保留整份 vendored 补丁。

## Out of scope

- Windows 代码签名和安装包（MSI/MSIX）。
- 重写 GPUI 上游着色器架构。

## Success signals

- 三个 Windows 架构 ZIP 包可构建且通过 CI。
- 每个 Windows ZIP 只有一个顶层 `doubao-skin.exe`，macOS 包不内嵌 CLI 或 Skills。
- 桌面包不包含 CLI；CLI 包不包含桌面应用或主题资源。
- Scoop 能根据 Windows 架构安装独立 CLI，macOS/Linux 安装脚本能选择对应 CLI 资产。
- Web 下载页不再把 Windows 标为 Coming Soon，并能在本地推荐匹配的桌面版本。
- `doubao-skin-cli-macOS-universal.tar.gz` 同时包含 x86_64/ARM64，严格签名校验通过，证书指纹与同版本 App 相同。
- 现有 macOS 构建和测试不受影响。
- workflow validate 通过。

## Open questions

- 无未决产品设计问题；最终发布仍受验收记录与生产签名作业约束。

## Decision

用户在 2026-08-30 明确要求修复审查问题、移除现有 gpui_windows vendored patch、整理桌面源码分组并产出 Windows 测试包；随后澄清桌面与 CLI 必须是互不嵌套的两条独立安装链，而不是取消 CLI 分发，并要求 Scoop 与 Web 平台识别覆盖多操作系统安装入口。2026-08-31，用户进一步明确准备发布 `v0.4.0`，要求先确保该版本正确，并让 macOS CLI 临时复用 App 已有的稳定自签名方案。
