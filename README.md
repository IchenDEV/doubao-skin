# doubao-work-skin

**给「豆包工作」macOS 桌面端换个皮肤。**

外部主题 / 换肤工具 · 不修改官方安装包 · 灵感来自 [Codex Dream Skin](https://github.com/Fei-Away/Codex-Dream-Skin)

![violet-night](docs/screenshot.png)

> 非字节跳动官方产品。原版 `/Applications/DoubaoWork.app` 不会被做任何修改。

## 效果

纯色主题：

| 主题 | 说明 |
| --- | --- |
| `violet-night` 暗夜紫 | 深紫罗兰底 + 紫罗兰强调色（默认，实测验证） |
| `ocean-cyan` 海洋青 | 石墨蓝底 + 青色强调色 |
| `forest` 墨绿 | 深墨绿底 + 翠绿强调色 |
| `pure-dark` 纯暗 | 只强制深色模式，不改颜色 |

Codex 常见配色（纯色豆包工作版）：

| 主题 | 说明 |
| --- | --- |
| `codex-catppuccin` 豆包柔紫 | 柔和的紫色，久看也舒服 |
| `codex-dracula` 豆包莓夜 | 鲜明的莓红，让重点更醒目 |
| `codex-nord` 豆包极光 | 清爽的蓝色，安静又清晰 |
| `codex-gruvbox` 豆包暖木 | 温暖的棕金色，更有亲切感 |
| `codex-solarized` 豆包深海 | 柔和的深蓝色，更适合专注 |
| `codex-one-half` 豆包清蓝 | 干净的蓝色，简单耐看 |

来源、色板和许可证见 [主题研究与选型](design/theme-standard/codex-theme-research.md)。

DreamSkin 热门主题灵感版（2026-08-28 下载榜快照，背景图均为原创生成）：

| 主题 | 说明 |
| --- | --- |
| `gallery-cozy-room` 暖室暮光 | 暖灯木色，像在家里工作 |
| `gallery-neon-koi` 霓虹游鱼 | 青紫霓虹游鱼，醒目又利落 |
| `gallery-moon-pine` 月下松岚 | 深蓝月夜松林，安静又沉稳 |
| `gallery-crimson-rain` 雨夜花影 | 暗红雨夜花影，沉静又有张力 |

灵感来源：[DreamSkin Gallery 下载最多](https://dreamskin.cc/gallery?community=popular)。角色和品牌主题只保留色彩气质，不打包原站人物、品牌或背景素材。

品牌视觉灵感主题（背景图均为原创生成）：

| 主题 | 说明 |
| --- | --- |
| `qq-light-blue` QQ轻蓝 | 浅蓝气泡与柔光，轻快又熟悉 |
| `gallery-whale-maid` 鲸鱼娘 | 浅青晴空与蓝鲸少女，明亮又俏皮 |

QQ 主题参考 [Mac QQ 官方页面](https://apps.apple.com/cn/app/qq/id451108668?mt=12) 的轻盈浅色视觉；鲸鱼娘参考 [DreamSkin 原主题](https://dreamskin.cc/themes/ver_cb557ececaa5de3f3dbe) 的浅青蓝白氛围。两款主题都使用原创背景，不包含原站人物、文字或品牌素材。

氛围主题（带背景图，容器半透明透出画面，灵感来自 [Codex Dream Skin](https://github.com/Fei-Away/Codex-Dream-Skin)）：

| 主题 | 说明 |
| --- | --- |
| `gothic-void` 哥特虚空 | 虚空教堂尖塔 × 紫雾，紫罗兰强调色 |
| `sakura-night` 夜樱 | 夜色樱花林 × 粉紫月光，樱粉强调色 |
| `cyber-neon` 赛博霓虹 | 雨夜霓虹都市，青色强调色 |
| `mist-forest` 雾林 | 晨雾针叶林光束，翠绿强调色 |

![sakura-night](docs/screenshot-sakura-night.png)
（live 模式 + sakura-night 夜樱主题，原版 App 实测截图）

换肤范围：主聊天窗口、启动器、侧边面板等全部内嵌页面（约 20 个）。

## 使用

需要 macOS、已安装的「豆包工作」、Python 3（无第三方依赖）。

**两种方式：**

### A. live 模式（CDP 注入，推荐体验）

不改任何 App 文件：给原版 App 带调试端口启动，通过 Chrome DevTools Protocol 把主题注入每个内嵌页面。

```bash
python3 -m doubao_skin live violet-night   # 守护模式：启动 App 并持续注入
python3 -m doubao_skin live forest --once  # 只对当前页面注入一次（热切换）
```

![live-forest](docs/screenshot-live-forest.png)
（live 模式 + forest 墨绿主题，原版 App 实测截图）

- 热切换：直接再跑一条 `live <别的主题> --once` 即可即时换色
- 需要守护进程常驻（新页面/新窗口才会被注入）；App 退出后主题消失，下次需重新跑
- 注意：运行期间 localhost 会开一个调试端口，本地进程可借此控制 App 页面

### B. 离线构建（改副本，一次到位）

```bash
python3 -m doubao_skin list                 # 列出主题
python3 -m doubao_skin apply violet-night   # 构建皮肤版应用
python3 -m doubao_skin remove               # 删除皮肤版应用
```

`apply` 会在 `~/Applications/` 生成 **`DoubaoWork-Skin.app`**，直接打开即可，永久生效、无需守护进程。原版应用不受影响，随时共存。

首次启动时 macOS 会请求钥匙串访问（“DoubaoWork Safe Storage”）：输入开机密码并选择 **始终允许**。这是重签名改变应用身份所致；每次重新构建（换主题）会再弹一次。

### C. Rust + GPUI 桌面应用（`app/`）

![app-ui](docs/app-ui.png)

`app/` 是一个 Rust workspace，把 Python 实现完整移植为原生 GUI（同一套主题文件、同一套注入逻辑）：

- `skin-core`（库）：主题加载、pak v5 解析/重建、离线构建流水线、CDP live 注入（std-only 的 WebSocket/CDP 客户端，无 tokio）。
- `skin-ui`（GPUI 界面）：主题卡片（迷你界面预览 + 强调色按钮）+「Live 应用」「离线构建」「移除皮肤版」+ 日志区。

```bash
cd app
cargo run -p skin-ui                     # 打开窗口（首次编译较久）
cargo run -p skin-ui -- --live violet-night   # 启动后立即对该主题执行 Live 应用
cargo test -p skin-core                  # 核心逻辑测试（含对真实 resources.pak 的往返重建）
```

依赖：[GPUI](https://github.com/zed-industries/zed)（pin 在 zed `v1.15.1`，需 stable Rust ≥ 1.95）；macOS 上 `gpui_platform` 必须开 `font-kit` feature，否则文字不渲染；`runtime_shaders` feature 可免去对 Xcode metal 工具链的依赖。主题目录默认定位在仓库 `themes/`，可用环境变量 `DOUBAO_SKIN_THEMES_DIR` 覆盖。

## 原理

live 模式和离线构建共用同一套主题（CSS 变量覆写），只是送达方式不同：

1. **主题 = CSS 变量覆写**：应用自身已有完整深色主题和设计令牌体系（`--N*` 中性色板、`--B*` 品牌色板、`--s-color-*`、`--dbx-*` 表层令牌）。皮肤通过强制 `data-theme="dark"` + 高优先级选择器覆写这些令牌实现换肤，不碰组件样式。
2. **live 模式**：App（Chromium 内核）接受 `--remote-debugging-port`，用 CDP 的 `Runtime.evaluate` 在每个内嵌页面执行注入 JS（装 `<style>` + MutationObserver 守住 `data-theme`/`data-skin` 属性），并用 `Page.addScriptToEvaluateOnNewDocument` 保证导航后仍生效。内置了 stdlib 极简 WebSocket/CDP 客户端（`doubao_skin/ws.py`）。
3. **离线构建**：
   - *克隆而非修改*：`/Applications` 里的原版受 macOS App Management（MACL）保护，属主也无法写入。用 APFS clonefile 克隆到 `~/Applications`，瞬间完成、不占额外磁盘。
   - *主界面在 resources.pak 里*：主聊天 UI 不是散文件，而是 gzip 压缩后打进 Chromium 的 `resources.pak`。本工具内置了一个极简 pak v5 解析/重建器（`doubao_skin/pak.py`），把主题 CSS 注入包内全部页面；磁盘上的本地入口 HTML（侧边面板等）也一并注入。
   - *ad-hoc 重签名*：改资源后签名失效会被 Gatekeeper 判“已损坏”，重新 ad-hoc 签名后即可运行。

## 自定义主题

复制一个内置主题目录（或新建），改两个文件：

```
themes/my-theme/
├── theme.json   {"id": "my-theme", "name": "我的主题", "description": "...",
│                 "background": "bg.jpg"   （可选）氛围背景图}
├── theme.css    CSS 覆写规则
├── icon.icns    （可选）自定义应用图标
└── bg.jpg       （可选）氛围背景图，注入时变成 var(--skin-bg-image)
```

`theme.css` 会被注入到所有内嵌页面。注意几点（都是踩过的坑）：

- 用 `html[data-skin][data-theme=dark], html[data-skin][data-theme=dark] body` 作为选择器。应用把部分令牌直接定义在 `html, body` 上，只覆写 `html` 会被 body 自己的规则压过。
- 很多令牌有 `-raw` 孪生变量（`rgba(var(--x-raw), .5)` 用），改色时两个都要覆写。
- 带背景图时，把主要 surface 令牌改成半透明，并用 `body::before` 铺 `var(--skin-bg-image)`（参考内置氛围主题）。引擎会把 `veil` 压暗层**烘进图片本身**（`theme.json` 的 `"veil": 0~1`，Rust 端），所以容器 alpha 直接用 0.5~0.65 即可，不用自己再叠 veil——透图度 = 容器 alpha 一件事，可调可预期。图本身选深色、主体偏离画面中心的效果最好。
- 背景图编码差异：Rust 引擎（`app/`）会把图片缩放到 ≤1920 宽、重编码为 JPEG(q75) 并烘焙 veil；Python CLI 无第三方依赖，**原样嵌入不缩放也不烘焙**——自己控制好文件大小。

然后 `python3 -m doubao_skin apply my-theme`（或传主题目录路径）。

## 已知限制

- live 模式：主题随 App 退出而消失；运行期间本地调试端口开放；App 若已在运行会被重启一次（为了带调试参数）。
  - App 被遮挡（切到别的 Space / 隐藏）时渲染进程会被系统冻结、CDP 暂时不应答——正常现象。watcher 发现页面失活会先唤醒（activate），唤不醒才重启 App；失活目标每 ~30 秒才探测一次（每次 CDP 连接都要建完整 DevTools 会话，探测过密会把渲染进程打到高 CPU）。
- 离线构建：原版应用升级后需重新 `apply`（皮肤版不会自动跟进）；ad-hoc 签名的 cdhash 每次构建都变，钥匙串授权每次重建要重新点一次。
- 主题只覆盖深色模式（皮肤本身强制 dark）。

## 许可

[MIT](LICENSE)
