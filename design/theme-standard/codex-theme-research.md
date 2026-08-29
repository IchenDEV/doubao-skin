# Codex 常见主题研究与豆包工作版选型

研究日期：2026-08-28
范围：只研究主题来源、色板与授权，不代表已实现或已验收任何皮肤。

## 结论

建议豆包工作版首批采用 6 个主题方向：

1. 豆包柔紫（基于 Catppuccin Mocha）
2. 豆包莓夜（基于 Dracula）
3. 豆包极光（基于 Nord）
4. 豆包暖木（基于 Gruvbox Dark）
5. 豆包深海（基于 Solarized Dark）
6. 豆包清蓝（基于 One Half Dark）

这不是“热度排名”。选择依据是：它们均出现在 Codex CLI/TUI 的官方内置主题清单中，同时有主题原作者维护的色板和明确许可证，适合形成一组差异明显、可追溯的产品主题。

面向平台小白用户时，界面只显示上面的中文产品名和一句感受描述，不显示 `Catppuccin`、`Gruvbox`、`.tmTheme`、终端或语法高亮等技术词。原主题名、许可证和颜色出处只保留在设计标准与代码注释中。

## 事实、推断与未确认项

### 已确认事实

- OpenAI 官方设置说明允许选择基础外观，调整强调色、背景色和前景色，修改界面与代码字体，并分享自定义主题。[OpenAI 设置说明](https://learn.chatgpt.com/docs/reference/settings#appearance)
- 以官方仓库固定提交 `5f49aba` 为准，Codex CLI/TUI 使用 `two_face` 提供约 250 种语言的语法高亮和 32 个内置颜色主题。[OpenAI 源码说明](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L1-L24)
- 32 个内置主题中明确包含 `catppuccin-mocha`、`dracula`、`gruvbox-dark`、`nord`、`one-half-dark` 和 `solarized-dark`。[OpenAI 内置主题清单](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L420-L454)
- CLI/TUI 在浅色终端上默认选择 `catppuccin-latte`，其他情况默认选择 `catppuccin-mocha`。[OpenAI 默认主题选择逻辑](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L189-L233)
- CLI/TUI 的 `/theme` 选择器提供即时预览、取消恢复和确认保存；也会读取 `$CODEX_HOME/themes/` 下的自定义 `.tmTheme` 文件。[OpenAI 主题选择器](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/theme_picker.rs#L1-L20)、[自定义主题发现逻辑](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L370-L418)

### 推断

- “常见”在本文中指“Codex 官方内置清单里辨识度高、且上游资料完整的主题家族”，不是用户量或下载量排名。
- 豆包工作应借用色彩气质和色板关系，不应照搬开发工具的代码预览、终端结构或技术命名。
- 首批保留 6 个主题足以覆盖柔和、鲜艳、冷静、温暖、低对比和清爽六种偏好；继续增加主题会提高选择成本和维护成本。

### 未确认

- OpenAI 没有提供官方主题流行榜或用户选择占比，因此不能宣称上述顺序代表 Codex 用户偏好。
- 桌面应用的完整界面主题与 CLI/TUI 的语法高亮主题是两套机制，不能假定文件格式、字段或颜色会一一兼容。
- “Codex 官方内置”不代表 OpenAI 拥有这些第三方主题；采用其名称、色板或代码时仍需遵守各自上游许可证。

## 豆包工作语义色映射

下面所有十六进制值均直接取自对应主题的原作者仓库；这里只改变产品中的语义用途，不创造“近似色”。`画布 / 卡片 / 边界 / 主文字 / 次文字 / 强调 / 成功 / 提醒 / 错误 / 信息` 是豆包工作统一令牌，不应让各主题自行新增角色。

| 豆包主题 | 画布 | 卡片 | 边界 | 主文字 | 次文字 | 强调 | 成功 | 提醒 | 错误 | 信息 |
|---|---|---|---|---|---|---|---|---|---|---|
| 豆包柔紫 | `#1e1e2e` | `#313244` | `#585b70` | `#cdd6f4` | `#9399b2` | `#cba6f7` | `#a6e3a1` | `#f9e2af` | `#f38ba8` | `#89b4fa` |
| 豆包莓夜 | `#282a36` | `#44475a` | `#6272a4` | `#f8f8f2` | `#6272a4` | `#ff79c6` | `#50fa7b` | `#ffb86c` | `#ff5555` | `#8be9fd` |
| 豆包极光 | `#2e3440` | `#3b4252` | `#4c566a` | `#eceff4` | `#d8dee9` | `#88c0d0` | `#a3be8c` | `#ebcb8b` | `#bf616a` | `#81a1c1` |
| 豆包暖木 | `#282828` | `#3c3836` | `#504945` | `#ebdbb2` | `#928374` | `#d79921` | `#b8bb26` | `#fabd2f` | `#fb4934` | `#83a598` |
| 豆包深海 | `#002b36` | `#073642` | `#586e75` | `#839496` | `#586e75` | `#268bd2` | `#859900` | `#b58900` | `#dc322f` | `#2aa198` |
| 豆包清蓝 | `#282c34` | `#313640` | `#474e5d` | `#dcdfe4` | `#5c6370` | `#61afef` | `#98c379` | `#e5c07b` | `#e06c75` | `#56b6c2` |

注意：该表是色板选型，不等于可访问性验收。落地时仍需按真实字号、字重、透明度和背景组合逐项验证文字与控件状态对比度；不能仅凭主题原项目的描述推定豆包工作界面合格。

## 主题来源、原始色板与许可证

### 1. 豆包柔紫 / Catppuccin Mocha

Catppuccin 自述为社区驱动的柔和粉彩主题，包含 4 个 flavor；Codex 深色终端的当前自适应默认是 Mocha。[Catppuccin 官方项目](https://github.com/catppuccin/catppuccin)、[Codex 默认选择逻辑](https://github.com/openai/codex/blob/5f49aba876922d6f2f55caa153bbb0ed1b46feba/codex-rs/tui/src/render/highlight.rs#L189-L233)

原始 Mocha 色板（完整 26 色）：

```text
rosewater #f5e0dc  flamingo #f2cdcd  pink #f5c2e7  mauve #cba6f7
red #f38ba8        maroon #eba0ac    peach #fab387  yellow #f9e2af
green #a6e3a1      teal #94e2d5      sky #89dceb    sapphire #74c7ec
blue #89b4fa       lavender #b4befe  text #cdd6f4   subtext1 #bac2de
subtext0 #a6adc8   overlay2 #9399b2  overlay1 #7f849c overlay0 #6c7086
surface2 #585b70   surface1 #45475a  surface0 #313244 base #1e1e2e
mantle #181825     crust #11111b
```

色值来源：[Catppuccin 官方 palette.json（固定提交）](https://github.com/catppuccin/palette/blob/07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd/palette.json)
许可证：MIT。[许可证全文](https://github.com/catppuccin/palette/blob/07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd/LICENSE)

产品定位：柔和、亲近、长时间使用不显压迫；作为默认推荐。强调色使用紫色，避免把全部粉彩色同时铺满界面。

### 2. 豆包莓夜 / Dracula

Dracula 官方仓库称其覆盖 Visual Studio Code、iTerm、Vim、Terminal.app 等 400 多个应用，并给出统一 OSS 色板。[Dracula 官方项目](https://github.com/dracula/dracula-theme)

原始 Dracula 色板（完整 12 色）：

```text
Background #282a36  Current Line #44475a  Selection #44475a
Foreground #f8f8f2  Comment #6272a4       Cyan #8be9fd
Green #50fa7b       Orange #ffb86c        Pink #ff79c6
Purple #bd93f9      Red #ff5555           Yellow #f1fa8c
```

色值来源：[Dracula 官方 README 色板（固定提交）](https://github.com/dracula/dracula-theme/blob/2985f660b04e6961b0887ffac2f8d3f35f431698/README.md#dracula)
许可证：MIT。[许可证全文](https://github.com/dracula/dracula-theme/blob/2985f660b04e6961b0887ffac2f8d3f35f431698/LICENSE)

产品定位：鲜明、活泼、辨识度最高。大面积仍使用深灰背景，粉色只用于主操作、选中和少量高价值提示。

### 3. 豆包极光 / Nord

Nord 官方定义为 16 色的北极蓝色板，并把 `nord8` 定义为 UI 主强调色，把 `nord11`、`nord13`、`nord14` 分别用于错误、提醒和成功语义。[Nord 官方项目](https://github.com/nordtheme/nord)、[Nord 官方 CSS 说明](https://github.com/nordtheme/nord/blob/1cef71605416a222e57225b544540ce0fcec18d4/src/nord.css)

原始 Nord 色板（完整 16 色）：

```text
nord0 #2e3440   nord1 #3b4252   nord2 #434c5e   nord3 #4c566a
nord4 #d8dee9   nord5 #e5e9f0   nord6 #eceff4   nord7 #8fbcbb
nord8 #88c0d0   nord9 #81a1c1   nord10 #5e81ac  nord11 #bf616a
nord12 #d08770  nord13 #ebcb8b  nord14 #a3be8c  nord15 #b48ead
```

色值来源：[Nord 官方 nord.css（固定提交）](https://github.com/nordtheme/nord/blob/1cef71605416a222e57225b544540ce0fcec18d4/src/nord.css)
许可证：MIT。[许可证全文](https://github.com/nordtheme/nord/blob/1cef71605416a222e57225b544540ce0fcec18d4/license)

产品定位：冷静、清晰、低刺激，适合偏办公感的用户。不要添加冰山、代码或极客装饰，差异只由颜色和预览内容表达。

### 4. 豆包暖木 / Gruvbox Dark

Gruvbox 原作者将其描述为具有柔和“复古律动”色彩、支持明暗切换的 Vim 主题，重点是颜色易于区分、对比足够且观感舒适。[Gruvbox 原作者仓库](https://github.com/morhetz/gruvbox)

豆包映射使用的原始核心色：

```text
dark0 #282828       dark1 #3c3836       dark2 #504945       gray #928374
light1 #ebdbb2      neutral_yellow #d79921
bright_red #fb4934  bright_green #b8bb26  bright_yellow #fabd2f
bright_blue #83a598 bright_purple #d3869b bright_aqua #8ec07c
bright_orange #fe8019
```

完整原始色值来源：[Gruvbox 官方 gruvbox.vim palette（固定提交）](https://github.com/morhetz/gruvbox/blob/5d15b2765f59754d7ac263c88a0f6e3e58124951/colors/gruvbox.vim#L88-L143)
许可证：MIT/X11，由原作者 README 明示。[许可证说明](https://github.com/morhetz/gruvbox/blob/5d15b2765f59754d7ac263c88a0f6e3e58124951/README.md#license)

产品定位：温暖、沉稳、有纸张与木质感，但不增加仿木纹和复古拟物装饰；保持现代卡片结构。

### 5. 豆包深海 / Solarized Dark

Solarized 原作者将其定义为面向终端和图形应用的 16 色方案，包含 8 个中性色与 8 个强调色，并提供明暗两种模式。[Solarized 原作者仓库](https://github.com/altercation/solarized)

原始 Solarized 色板（完整 16 色）：

```text
base03 #002b36  base02 #073642  base01 #586e75  base00 #657b83
base0 #839496   base1 #93a1a1   base2 #eee8d5    base3 #fdf6e3
yellow #b58900  orange #cb4b16  red #dc322f      magenta #d33682
violet #6c71c4  blue #268bd2    cyan #2aa198     green #859900
```

色值来源：[Solarized 官方 README 色表（固定提交）](https://github.com/altercation/solarized/blob/62f656a02f93c5190a8753159e34b385588d5ff3/README.md#the-values)
许可证：MIT。[许可证全文](https://github.com/altercation/solarized/blob/62f656a02f93c5190a8753159e34b385588d5ff3/LICENSE)

产品定位：低对比、专注、深海感。次要文字不得继续降透明度，否则会放大该色板本身的低对比特性。

### 6. 豆包清蓝 / One Half Dark

One Half 原作者称其基于 Atom 的 One，提供明暗方案并覆盖 Sublime Text、Vim、iTerm、VS Code 和多种终端。[One Half 原作者仓库](https://github.com/sonph/onehalf)

原始 One Half Dark 核心色：

```text
background #282c34  line highlight #313640  selection #474e5d
foreground #dcdfe4  whitespace #5c6370
red #e06c75         green #98c379          yellow #e5c07b
blue #61afef        purple #c678dd         cyan #56b6c2
```

色值来源：[One Half 官方 VS Code 主题（固定提交）](https://github.com/sonph/onehalf/blob/75eb2e97acd74660779fed8380989ee7891eec56/vscode/onehalf-dark/themes/onehalf-dark-color-theme.json)、[官方终端色板](https://github.com/sonph/onehalf/blob/75eb2e97acd74660779fed8380989ee7891eec56/windowsterminal/OneHalfDark.json)
许可证：MIT。[许可证全文](https://github.com/sonph/onehalf/blob/75eb2e97acd74660779fed8380989ee7891eec56/LICENSE.txt)

产品定位：最接近现代通用深色界面的清爽蓝色方案；作为不喜欢紫色或高饱和色用户的稳妥选择。

## 面向小白用户的展示标准

主题选择页只传达三件事：看起来怎样、是否已选、如何应用。

| 内部来源 | 用户可见名称 | 用户可见一句话 | 禁止出现的词 |
|---|---|---|---|
| Catppuccin Mocha | 豆包柔紫 | 柔和的紫色，久看也舒服 | Catppuccin、Mocha、默认终端主题 |
| Dracula | 豆包莓夜 | 鲜明的莓红，让重点更醒目 | Dracula、语法高亮、OSS |
| Nord | 豆包极光 | 清爽的蓝色，安静又清晰 | Nord、nord8、16 色板 |
| Gruvbox Dark | 豆包暖木 | 温暖的棕金色，更有亲切感 | Gruvbox、Vim、复古代码主题 |
| Solarized Dark | 豆包深海 | 柔和的深蓝色，更适合专注 | Solarized、CIELAB、终端色 |
| One Half Dark | 豆包清蓝 | 干净的蓝色，简单耐看 | One Half、Atom、VS Code |

交互标准：

- 默认只展示主题卡片和即时预览，不展示颜色代码、来源、许可证或技术设置。
- 整张主题卡片可点击；选中状态同时使用描边、勾选和文字，不能只靠颜色。
- 主按钮统一为“应用这个主题”；成功反馈为“已换成豆包柔紫”等具体结果。
- “恢复默认”放在次要位置，不要求用户理解当前配置、文件或运行方式。
- 预览使用真实的豆包工作内容，例如对话列表、消息气泡、输入框和按钮；不要用代码、日志、命令行或开发工具截图。

## 授权落地要求

- 在仓库的第三方声明中记录原主题名、作者/项目、来源固定提交和许可证链接。
- 若复制或改写上游主题文件、源码或大段配置，随分发物保留对应许可证与版权声明。
- 产品界面可以使用中文产品名，但内部主题元数据必须保留 `derivedFrom`、`sourceUrl`、`sourceCommit` 和 `license`，确保可追溯。
- 本文只记录上游仓库声明，不构成法律意见；正式对外发布前应由项目负责人确认品牌命名和许可证归属要求。
