# Codex 主题与热度研究快照

采集窗口：2026-08-28 02:48–02:52（Asia/Singapore），即 2026-08-27 18:48–18:52 UTC。

证据范围：OpenAI 官方文档与官方源码、GitHub REST API/仓库页面、npm Downloads API、Visual Studio Marketplace Gallery API。

口径：搜索只用于发现候选项目，不以搜索排序、转载榜单、README 中手写星数或作者写的 “Popular” 标签判断热度。

## 直接结论

1. **Codex 有官方主题机制，但没有官方热度榜。** Codex CLI/TUI 当前内置 32 个语法主题，并支持 `/theme` 预览和自定义 `.tmTheme`；桌面应用支持基础外观、强调色、背景色、前景色、字体和主题分享。OpenAI 没有公开主题选择占比、安装量榜单或官方主题市场。
2. **Codex 专用社区项目中，公开热度主要集中在桌面皮肤/CDP 注入工具。** `Fei-Away/Codex-Dream-Skin` 的 14,151 stars 是明显离群的第一名；第二、第三名只有 533 和 435 stars。
3. **官方原生导入主题与 CLI `.tmTheme` 社区仍小。** 采样到的相关仓库大多只有 1–11 stars；不能把皮肤工具的热度转述成某套原生配色的热度。
4. **Catppuccin、Dracula、Solarized、Gruvbox、Nord、One Half 可以称为高知名度经典主题家族，但不能称为 Codex 内最常用。** 它们的上游仓库 stars 很高，且都在 Codex 官方内置清单中；这些 stars 衡量的是跨编辑器/终端的主题家族关注度。

## 1. OpenAI Codex 官方主题有哪些

### 桌面应用

[OpenAI 官方设置说明](https://learn.chatgpt.com/docs/reference/settings#appearance)确认，用户可以选择基础主题，调整强调色、背景色和前景色，修改 UI 与代码字体，并分享自定义主题。官方没有在该说明中发布各主题的使用量或排名。

### CLI/TUI

以下结论固定在 `openai/codex` 提交 `5f49aba876922d6f2f55caa153bbb0ed1b46feba`，避免主分支继续变化导致证据漂移：

- 约 250 种语言语法高亮、32 个内置颜色主题：[官方源码说明](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L1-L24)
- `/theme` 支持即时预览、取消恢复和确认保存：[官方主题选择器](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/theme_picker.rs#L1-L20)
- `$CODEX_HOME/themes/*.tmTheme` 支持自定义主题：[官方主题发现逻辑](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L370-L418)
- 浅色背景默认 `catppuccin-latte`，其他情况默认 `catppuccin-mocha`：[官方默认选择逻辑](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L189-L233)

32 个内置主题的官方清单如下：[源码清单](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L420-L454)

```text
1337
ansi
base16
base16-256
base16-eighties-dark
base16-mocha-dark
base16-ocean-dark
base16-ocean-light
catppuccin-frappe
catppuccin-latte
catppuccin-macchiato
catppuccin-mocha
coldark-cold
coldark-dark
dark-neon
dracula
github
gruvbox-dark
gruvbox-light
inspired-github
monokai-extended
monokai-extended-bright
monokai-extended-light
monokai-extended-origin
nord
one-half-dark
one-half-light
solarized-dark
solarized-light
sublime-snazzy
two-dark
zenburn
```

这里的“内置”只证明可选，不证明受欢迎。源码没有主题级选择计数。

## 2. 经典主题家族本身的热度

下表的 stars 来自各主题原作者/官方组织的上游仓库，而不是 Codex 主题仓库。它能说明主题家族跨编辑器与终端的知名度，不能说明 Codex 用户实际选择量。

| 主题家族 | 上游官方仓库 | Stars | Codex 官方内置 |
|---|---|---:|---|
| Dracula | [dracula/dracula-theme](https://github.com/dracula/dracula-theme) | 23,565 | `dracula` |
| Catppuccin | [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin) | 19,698 | 4 个 flavor |
| Solarized | [altercation/solarized](https://github.com/altercation/solarized) · [API](https://api.github.com/repos/altercation/solarized) | 16,010 | dark / light |
| Gruvbox | [morhetz/gruvbox](https://github.com/morhetz/gruvbox) · [API](https://api.github.com/repos/morhetz/gruvbox) | 15,705 | dark / light |
| Nord | [nordtheme/nord](https://github.com/nordtheme/nord) | 6,871 | `nord` |
| One Half | [sonph/onehalf](https://github.com/sonph/onehalf) · [API](https://api.github.com/repos/sonph/onehalf) | 1,897 | dark / light |

这组数据支持把 Dracula、Catppuccin、Solarized、Gruvbox 归为上游声量最高的一档，Nord 次之，One Half 较小。但项目年龄、仓库拆分方式和覆盖平台不同，stars 不应被解释为严格市场份额。

### 可得的官方市场分发指标

Visual Studio Marketplace Gallery API 的 `install` 字段在同一采集窗口返回：

| 官方发布者扩展 | Marketplace `install` | 评分 / 评分数 |
|---|---:|---:|
| [Dracula Official](https://marketplace.visualstudio.com/items?itemName=dracula-theme.theme-dracula) | 10,866,352 | 4.78 / 168 |
| [Catppuccin for VSCode](https://marketplace.visualstudio.com/items?itemName=Catppuccin.catppuccin-vsc) | 1,375,203 | 4.96 / 57 |
| [Nord](https://marketplace.visualstudio.com/items?itemName=arcticicestudio.nord-visual-studio-code) | 1,274,087 | 4.87 / 92 |

这些数值仅适用于对应 VS Code 扩展，不适用于 Codex。Solarized 等主题还可能作为编辑器内置主题而没有等价安装数，其他家族也可能由多个端口分散分发，因此 Marketplace 数据不能补成完整排名，也不能与 GitHub stars 直接相加。

## 3. Codex 专用社区项目热度

### 3.1 桌面皮肤、主题平台与管理器

数据来自每个仓库的 GitHub REST API，表格按 stars 降序。`订阅`是 `subscribers_count`，不是 GitHub 页面常把 stars 重复显示为 “watchers” 的 `watchers_count`。`未关闭`是 `open_issues_count`，其中可能包含未关闭 PR。

| 项目 | 定位与 Codex 专用边界 | Stars | Forks | 订阅 | 未关闭 | 最近推送 UTC |
|---|---|---:|---:|---:|---:|---|
| [Fei-Away/Codex-Dream-Skin](https://github.com/Fei-Away/Codex-Dream-Skin) · [API](https://api.github.com/repos/Fei-Away/Codex-Dream-Skin) | 本机 CDP 皮肤；以 Codex 为主 | 14,151 | 1,349 | 14 | 47 | 2026-08-27 16:17 |
| [zhulin025/Codex-QQ-Skin](https://github.com/zhulin025/Codex-QQ-Skin) · [API](https://api.github.com/repos/zhulin025/Codex-QQ-Skin) | QQ/图片皮肤生成器；Codex/ChatGPT 双目标 | 533 | 51 | 1 | 1 | 2026-08-11 13:36 |
| [HeiGeAi/heige-codex-skin-studio](https://github.com/HeiGeAi/heige-codex-skin-studio) · [API](https://api.github.com/repos/HeiGeAi/heige-codex-skin-studio) | CDP 换肤工作室；也支持 WorkBuddy | 435 | 65 | 2 | 14 | 2026-08-23 13:46 |
| [CodeDrobe/skills](https://github.com/CodeDrobe/skills) · [API](https://api.github.com/repos/CodeDrobe/skills) | AI 桌面应用换肤 Skills；Codex 是主要目标之一 | 245 | 22 | 2 | 1 | 2026-08-09 15:37 |
| [duxweb/ReTheme](https://github.com/duxweb/ReTheme) · [API](https://api.github.com/repos/duxweb/ReTheme) | ChatGPT/Codex 桌面主题管理器；非 Codex 独占 | 131 | 9 | 4 | 2 | 2026-07-24 03:04 |
| [freestylefly/codex-themes](https://github.com/freestylefly/codex-themes) · [API](https://api.github.com/repos/freestylefly/codex-themes) | Codex 主题创作、社区发布与 CDP 应用平台 | 128 | 30 | 1 | 2 | 2026-08-11 03:01 |
| [Finderchangchang/codex-autoskin](https://github.com/Finderchangchang/codex-autoskin) · [API](https://api.github.com/repos/Finderchangchang/codex-autoskin) | 从图片生成并应用 Codex 皮肤 | 117 | 22 | 0 | 4 | 2026-07-23 08:50 |
| [JasonSTong/codex-theme-studio](https://github.com/JasonSTong/codex-theme-studio) · [API](https://api.github.com/repos/JasonSTong/codex-theme-studio) | macOS Codex 可视化编辑与 CDP 热应用 | 106 | 2 | 2 | 1 | 2026-07-24 09:08 |
| [xnydl/codex-dream-skin](https://github.com/xnydl/codex-dream-skin) · [API](https://api.github.com/repos/xnydl/codex-dream-skin) | macOS/Windows Codex 换肤 Skill | 54 | 2 | 0 | 1 | 2026-07-17 02:49 |
| [aiwenjie777/codex-skin-skill](https://github.com/aiwenjie777/codex-skin-skill) · [API](https://api.github.com/repos/aiwenjie777/codex-skin-skill) | Codex 一键皮肤 Skill | 51 | 6 | 0 | 1 | 2026-07-23 06:57 |
| [aithink001/Codex-Dream-Skin-Themes](https://github.com/aithink001/Codex-Dream-Skin-Themes) · [API](https://api.github.com/repos/aithink001/Codex-Dream-Skin-Themes) | Codex Dream Skin 主题工具 | 46 | 4 | 0 | 2 | 2026-07-17 09:13 |
| [Wangnov/awesome-codex-skins](https://github.com/Wangnov/awesome-codex-skins) · [API](https://api.github.com/repos/Wangnov/awesome-codex-skins) | `.codexskin` 标准、工具链和图库；不是单一主题 | 43 | 4 | 1 | 1 | 2026-07-18 17:01 |

由 stars 可确认的结论只有：Dream Skin 仓库关注度远高于其余项目，当前社区声量集中在“整窗皮肤/背景/注入工具”，而不是官方颜色主题导入。不能从这些仓库 stars 推断某个颜色主题最常用。

### 3.2 GitHub Releases 资产下载

下表汇总各仓库全部公开 release asset 的 `download_count`。这是安装包/压缩包下载次数之和，会重复计算同一用户的升级、重装、不同平台包和自动化下载；不是独立用户数，也不是当前活跃安装量。

| 项目 | Releases | Release assets 累计下载 |
|---|---:|---:|
| [Fei-Away/Codex-Dream-Skin Releases](https://github.com/Fei-Away/Codex-Dream-Skin/releases) | 21 | 66,695 |
| [zhulin025/Codex-QQ-Skin Releases](https://github.com/zhulin025/Codex-QQ-Skin/releases) | 27 | 5,579 |
| [freestylefly/codex-themes Releases](https://github.com/freestylefly/codex-themes/releases) | 17 | 4,121 |
| [HeiGeAi/heige-codex-skin-studio Releases](https://github.com/HeiGeAi/heige-codex-skin-studio/releases) | 21 | 1,178 |
| [JasonSTong/codex-theme-studio Releases](https://github.com/JasonSTong/codex-theme-studio/releases) | 0 | 0 |
| [aithink001/Codex-Dream-Skin-Themes Releases](https://github.com/aithink001/Codex-Dream-Skin-Themes/releases) | 0 | 0 |

Stars 与 release assets 下载数并不一致：例如 freestylefly 的 stars 少于 HeiGeAi，但累计 release assets 下载更多。因此两者应作为不同的关注/分发信号并列展示，不应合成一个“热度分”。

### 3.3 官方原生导入主题、CLI 主题与社区目录

这些项目更接近 OpenAI 官方机制，但公开仓库关注度明显较小。

| 项目 | 类型 | Stars | Forks | 备注 |
|---|---|---:|---:|---|
| [mcpso/awesome-codex-themes](https://github.com/mcpso/awesome-codex-themes) · [API](https://api.github.com/repos/mcpso/awesome-codex-themes) | CLI、原生主题和皮肤目录 | 11 | 7 | 是索引，不是主题实现；README 手写星数已滞后 |
| [miniLV/Anthropic-codex-theme](https://github.com/miniLV/Anthropic-codex-theme) · [API](https://api.github.com/repos/miniLV/Anthropic-codex-theme) | 可直接导入的明/暗主题 | 10 | 0 | 单一主题 |
| [shaw-baobao/codex-themes](https://github.com/shaw-baobao/codex-themes) · [API](https://api.github.com/repos/shaw-baobao/codex-themes) | `codex-theme-v1` 导入字符串、预览与 schema | 9 | 1 | 原生桌面导入 |
| [jstxn/codex-themes](https://github.com/jstxn/codex-themes) · [API](https://api.github.com/repos/jstxn/codex-themes) | 早期桌面主题启动器 | 8 | 1 | 作者已标为历史方案 |
| [lafllamme/codex-themes](https://github.com/lafllamme/codex-themes) · [API](https://api.github.com/repos/lafllamme/codex-themes) | 终端色板转 Codex 原生 JSON | 6 | 1 | 原生桌面导入工具 |
| [ychampion/codex-themes](https://github.com/ychampion/codex-themes) · [API](https://api.github.com/repos/ychampion/codex-themes) | CLI `.tmTheme` 管理器 | 4 | 1 | 走官方 CLI 自定义机制 |
| [samuxbuilds/codex-themes](https://github.com/samuxbuilds/codex-themes) · [API](https://api.github.com/repos/samuxbuilds/codex-themes) | 生成 1000+ 导入主题的图库 | 3 | 1 | README 的 “Popular” 只是作者分类，无主题级使用数据 |
| [Nick2bad4u/codex-terminal-themes](https://github.com/Nick2bad4u/codex-terminal-themes) · [API](https://api.github.com/repos/Nick2bad4u/codex-terminal-themes) | 204 个 CLI TextMate 主题 | 1 | 0 | 格式也可用于 `bat` 等工具 |

`codex-terminal-themes` 在 npm 从 2026-06-08 至 2026-08-27 有 753 次包下载。[npm Downloads API](https://api.npmjs.org/downloads/point/2026-06-08:2026-08-27/codex-terminal-themes) 这是包下载次数，不是 OpenAI 官方安装量或唯一用户数。

## 4. 指标限制

- GitHub stars 是仓库关注度，受项目年龄、传播渠道、仓库合并方式和受众范围影响，不等于安装或活跃用户。
- Forks 可能包含试验、镜像和未使用分支；`open_issues_count` 包含未关闭 PR。
- Release asset 下载会累计升级、重装、多平台包与自动化抓取，不是唯一用户。
- npm 下载可能来自 CI、缓存或重复安装，不是 Codex 用户数。
- Marketplace `install` 属于 VS Code 扩展，不能外推到 Codex。
- 所有数值都是 2026-08-28 的快照；项目热度变化很快，后续引用必须重新采集。
