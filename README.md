<div align="center">

[简体中文](README.md) · [English](README.en.md)

<img src="apps/web/public/app-icon.png" alt="豆皮图标" width="112" height="112">

# 豆皮

**给 macOS 版「豆包」与「豆包工作」换一套真正好用的主题，并实验支持 WorkBuddy。**

[在线主题库](https://doubao-skin.idevlab.dev) · [使用与下载](https://doubao-skin.idevlab.dev/guide#download) · [创作与投稿](https://doubao-skin.idevlab.dev/contribute)

[![CI](https://github.com/IchenDEV/doubao-skin/actions/workflows/ci.yml/badge.svg)](https://github.com/IchenDEV/doubao-skin/actions/workflows/ci.yml)
[![Website](https://img.shields.io/badge/website-doubao--skin.idevlab.dev-5b7ee5)](https://doubao-skin.idevlab.dev)
[![License: MIT](https://img.shields.io/badge/license-MIT-2f81f7)](LICENSE)

</div>

![豆皮桌面应用](docs/images/app.png)

![豆皮在线主题库](docs/images/gallery.png)

> 当前支持 macOS；Windows 版稍后推出。本项目不是豆包或 WorkBuddy 的官方产品，不会修改 `/Applications` 中的官方应用包。

## 真实页面效果

以下截图来自本机真实的「豆包工作」窗口，主题通过 live 模式呈现。仅对作者头像与身份、公司名和电脑设备名打码，其余界面与内容保留原样；原始截图未纳入仓库。

<table>
  <tr>
    <td width="50%">
      <img src="docs/images/doubao-work-qq-light-blue-redacted.png" alt="QQ 轻蓝主题应用于真实豆包工作窗口的脱敏截图">
      <br><sub>QQ 轻蓝 · 真实会话页</sub>
    </td>
    <td width="50%">
      <img src="docs/images/doubao-work-starry-room-redacted.png" alt="星光书房主题应用于真实豆包工作窗口的脱敏截图">
      <br><sub>星光书房 · 会话页</sub>
    </td>
  </tr>
</table>

## 功能

- 原生 macOS 桌面应用：浏览、预览、安装、应用和恢复主题。
- 30 套内置主题，覆盖纯色、氛围背景、编辑器配色和品牌灵感。
- 在线主题商店与可验证的 `.doubao-skin.zip` 主题包。
- 支持 Apple 芯片与 Intel Mac 的通用 ZIP/DMG。
- Rust CLI、Codex 插件与 Claude Code 插件共用同一套主题工具链。
- live 注入与离线克隆两种模式；官方 App 本体始终保持不变。
- 实验支持 WorkBuddy 5.3.14：v2 主题走兼容模式，v3 主题按 `targets.workbuddy` 精确加载；仅限 live 模式，不接入协议桥，也不支持离线克隆。
- 网站提供组合筛选、深色模式、主题详情、使用指南和投稿说明。
- 桌面应用与网站共用同一组分层豆皮图标，并适配浅色、深色与单色系统外观。

## 下载与使用

从 [GitHub Releases](https://github.com/IchenDEV/doubao-skin/releases/latest) 下载：

- `Doubao-Skin-macOS-universal.dmg`：推荐，打开后拖入 Applications。
- `Doubao-Skin-macOS-universal.zip`：解压后直接使用。
- `.sha256`：对应安装包的 SHA-256 校验文件。

首次运行时，如 macOS 提示无法打开，请前往「系统设置 → 隐私与安全性」，在“安全性”部分点击“仍要打开”并输入管理员密码。

发布包使用同一枚社区自签名证书持续签名，并非 Apple 公证；后续版本会保持该签名身份，除非发生已公告的安全轮换。

1. 打开“豆皮”。
2. 选择「豆包」「豆包工作」或实验性的 `WorkBuddy`（也可使用 `Command-1` / `Command-2` / `Command-3`）。
3. 选择主题并查看预览。
4. 点击“应用主题”；需要还原时点击“恢复默认”。

WorkBuddy 已在 5.3.14 上验证。首次应用时，如果 WorkBuddy 已经运行，豆皮会先说明影响并把按钮改为“重启并应用”；只有再次明确点击才会退出并重启 WorkBuddy。请先保存正在进行的任务。WorkBuddy 后续主动退出时，豆皮不会自动拉起它。

主题商店可以直接安装线上主题。本地主题包也可以拖入窗口，或通过“安装主题…”与 `Command-O` 导入。已安装主题保存在 `~/Library/Application Support/Doubao Skin/themes/`。

## 主题

完整主题库和动态筛选见 [doubao-skin.idevlab.dev](https://doubao-skin.idevlab.dev)。当前内置 30 套主题，包括：

- **纯色与编辑器配色**：暗夜紫、海洋青、墨绿、纯暗、桃气日落、华夏蓝，以及 Catppuccin、Dracula、Nord、Gruvbox、Solarized、One Half 风格。
- **氛围背景**：哥特虚空、夜樱、赛博霓虹、雾林、暖室暮光、霓虹游鱼、月下松岚、雨夜花影、机械工头。
- **明亮与品牌灵感**：QQ 轻蓝、鲸鱼娘、奶茶茶会、花园同伴、星光书房、馋嘴豆包、甜点偷笑、代码仓库、Claude 暖橙。

主题素材、来源和许可证写在各自的 `theme.json`、[主题研究](design/theme-standard/codex-theme-research.md) 与 [第三方声明](THIRD_PARTY_NOTICES.md) 中。

## Agent 插件

仓库同时提供 Codex 与 Claude Code Marketplace 插件，安装后可使用 `$create-doubao-theme` 与 `$apply-doubao-theme`。

Codex：

```bash
codex plugin marketplace add IchenDEV/doubao-skin
codex plugin add doubao-skin@doubao-skin
```

Claude Code：

```text
/plugin marketplace add IchenDEV/doubao-skin
/plugin install doubao-skin@doubao-skin
```

- `$create-doubao-theme`：从自然语言创建、检查、预览和打包主题。
- `$apply-doubao-theme`：列出、安装、应用、恢复和管理主题。

插件源码位于 [`plugins/doubao-skin`](plugins/doubao-skin)。网站还公开 [Agent Skills Discovery Draft 0.2.0 索引](https://doubao-skin.idevlab.dev/.well-known/agent-skills/index.json)。

## Rust CLI

`doubao-theme` 是一个独立的命令行工具，不依赖 Node.js、Python 或 GPUI。

### 安装

一行命令安装（需要 macOS）：

```bash
curl -fsSL https://raw.githubusercontent.com/IchenDEV/doubao-skin/main/scripts/install-cli.sh | sh
```

安装脚本从 GitHub Release 下载预编译的 universal 二进制，验证校验和，并放到 `/usr/local/bin`。可通过 `INSTALL_DIR` 变量指定安装位置，通过 `VERSION` 变量指定版本：

```bash
VERSION=v0.1.0 INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/IchenDEV/doubao-skin/main/scripts/install-cli.sh | sh
```

也可以从 [GitHub Releases](https://github.com/IchenDEV/doubao-skin/releases/latest) 手动下载 `doubao-theme-macOS-universal.tar.gz` 解压。

### 用法

```bash
doubao-theme list
doubao-theme create themes/my-theme \
  --name "我的主题" --description "安静耐看的深色主题" \
  --accent "#5b7ee5" --appearance both --author "本地用户"
doubao-theme check themes/my-theme
doubao-theme preview themes/my-theme
doubao-theme pack themes/my-theme dist/my-theme.doubao-skin.zip
```

完整命令还包括 `install`、`apply`、`restore`、`build` 和 `remove-build`。运行 `doubao-theme --help` 查看参数。

## 从源码构建

需要 macOS、Rust 1.97.1+、Node.js 24.19+ 与 pnpm 12。

```bash
# 测试与检查
./scripts/check.sh all

# 运行桌面应用
cargo run -p doubao-skin-desktop

# 构建当前架构安装包
./scripts/build-macos.sh

# 构建 Apple 芯片 + Intel 通用安装包
./scripts/build-macos.sh --universal
```

网站：

```bash
corepack pnpm --dir apps/web install --frozen-lockfile
corepack pnpm --dir apps/web sync
corepack pnpm --dir apps/web dev
```

`sync` 会从 `themes/` 生成 SQLite 目录、统一预览、主题商店 JSON 和每个主题的安装包；不要手工编辑 `apps/web/data` 或 `apps/web/public/themes`。

## 项目结构

```text
apps/desktop        GPUI 原生桌面应用
apps/web            Next.js 在线主题库
crates/skin-core    主题引擎、CLI、live/离线构建与协议桥
themes              内置主题与素材
plugins             Codex / Claude Code 插件
design              主题 Schema 与设计规范
docs                架构、开发、投稿、发布与部署文档
workflow            Artifact 驱动的研发与验证记录
```

## 自定义主题与投稿

推荐使用 `doubao-theme create` 或 `$create-doubao-theme` 创建符合 `schemaVersion: 3` 的主题，并通过 `--targets` 明确声明豆包、豆包工作与 WorkBuddy 的适用范围。规范见 [主题与界面标准](design/theme-standard/README.md)、[v3 多应用规范](design/theme-standard/theme-v3.md) 与 [theme-v3.schema.json](design/theme-standard/theme-v3.schema.json)。v1/v2 用户主题仍可读取。

投稿前运行：

```bash
cargo run -p skin-core --bin doubao-theme -- check themes/my-theme
corepack pnpm --dir apps/web sync
./scripts/check.sh all
```

完整流程见 [提交新主题](docs/submitting-themes.md) 与 [贡献指南](CONTRIBUTING.md)。网站不提供直接上传，主题通过 GitHub Pull Request 投稿。

## 网站部署

`apps/web` 部署在 Vercel，生产域名为 [doubao-skin.idevlab.dev](https://doubao-skin.idevlab.dev)。GitHub `main` 与 Vercel 项目连接后，推送自动更新 Production，其他分支和 Pull Request 生成 Preview。完整配置与验收步骤见 [网站部署](docs/website-deployment.md)。

## 文档

- [文档索引](docs/README.md)
- [工程架构](docs/architecture.md)
- [本地开发](docs/development.md)
- [研发工作流](docs/development-workflow.md)
- [macOS 发布](docs/releasing.md)
- [安全报告](SECURITY.md)

## 许可

Rust 核心、网站、项目文档与自有主题定义使用 [MIT License](LICENSE)。第三方素材和依赖按 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) 与 `LICENSES/` 中的对应条款分发。
