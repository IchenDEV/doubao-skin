# 工程架构

豆皮是一个小型 monorepo：一套 Rust 核心驱动原生 macOS 应用，一套 Next.js 网站展示并分发同一批主题包。`themes/` 是两端共享的唯一主题源。

```text
themes/ ───────────────▶ crates/skin-core ─────────▶ apps/desktop
   │                           │                         │
   │                           ├─ 主题解析与 CSS 生成     └─ 浏览、安装、预览、应用
   │                           ├─ CDP live 注入
   │                           ├─ App 克隆与 PAK 重建
   │                           └─ 实验性协议桥
   │
   └─ apps/web/scripts/sync-themes.mjs
                 │
                 ├─ data/themes.db
                 ├─ public/themes/catalog.json
                 └─ public/themes/*.doubao-skin.zip
                              │
                              └──────────────▶ apps/web
```

## 目录职责

| 路径 | 职责 | 是否手工维护 |
| --- | --- | --- |
| `apps/desktop` | GPUI 原生 macOS 界面和应用包元数据 | 是 |
| `apps/web` | Next.js 主题画廊、详情页与下载入口 | 是 |
| `crates/skin-core` | 主题、PAK、CDP、WebSocket、安装和协议桥实现 | 是 |
| `themes` | 可移植主题包及其源素材 | 是 |
| `design/theme-standard` | 主题格式、界面规则、Schema 与设计令牌 | 是 |
| `docs` | 架构、开发、发布、部署和研究记录 | 是 |
| `scripts` | 本地检查、打包和研发流程入口 | 是 |
| `workflow` | 每项产品变更的 intent、spec、plan 与 verification | 是 |
| `apps/web/data`、`apps/web/public/themes` | 由 `pnpm sync` 生成的网站目录与安装包 | 否 |
| `target`、`dist`、`work` | 本地构建或实验产物，均被 Git 忽略 | 否 |

## 模块边界

- `skin-core` 不依赖 GPUI。所有主题解析、文件写入、CDP 通信和协议转换都在核心层完成。
- `apps/desktop` 只负责用户交互和状态呈现，不复制核心层的 PAK、网络或主题逻辑。
- `apps/web` 在构建时读取已生成的目录数据，运行时不调用 Rust 核心，也不扫描仓库主题目录。
- `themes` 是数据包。主题差异写进 `theme.json`、`theme.css` 和素材，不在桌面端增加按主题 ID 分支。
- 协议桥保持窄范围：只处理明确支持的普通文本请求；附件、工具、组织上下文和未知内容块必须拒绝接管。

## 三条运行路径

### 1. 原生主题管理

桌面应用读取内置与用户安装的主题，展示统一预览，并把选中的主题交给 `skin-core`。用户安装的包保存在 Application Support，不与应用升级产物混放。

### 2. Live 注入

核心以不同的回环端口启动用户选中的「豆包」或「豆包工作」，通过 CDP 向该目标的匹配页面注入主题 CSS。它不修改官方安装包；应用退出后效果消失。调试端口运行期间可读取或控制页面，只应在可信本机使用。

### 3. 克隆构建

核心克隆官方应用，修改克隆内的 HTML 与 `resources.pak`，再对克隆进行 ad-hoc 或开发者签名。原始 `/Applications/DoubaoWork.app` 不被覆盖。官方应用升级后需要重新生成克隆。

实验性协议桥与 Live 路径共用 CDP 生命周期，但使用独立的回环适配器。它不会转发官方 Cookie、认证头、工作区字段或未支持的内容块，也不应被描述为官方 provider 接口。

## 主题数据流

1. 主题作者按 `design/theme-standard` 编写 `theme.json`、`theme.css`、预览与可选素材。
2. Rust 测试验证主题加载、资源路径、Schema 兼容和生成结果。
3. `pnpm --dir apps/web sync` 从同一主题源生成 SQLite、目录 JSON、统一预览和安装包。
4. 网站只发布生成物；桌面应用在打包时直接包含主题源。

源主题和生成目录必须在同一次变更中提交。`dist/`、`target/` 与 `work/` 只属于本机，不进入版本控制。

## 许可证边界

核心、网站、文档与自有主题定义采用 MIT。桌面应用的已解析依赖图包含 GPL-3.0-or-later 的 `ztracing` 与 `zlog`，因此 `apps/desktop` 和分发的桌面二进制采用 GPL-3.0-or-later。两份许可证正文各自覆盖不同组件；具体清单见 [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md)。

## 变更原则

- 新行为优先进入现有模块；只有出现真实的第二个实现或稳定边界时才新增抽象。
- 用户界面不暴露 CDP、端口、路径、签名或协议字段，错误只给结果和可执行动作。
- 桌面 UI 变更必须通过真实窗口和窄窗口截图；协议桥必须在隔离对话中验证原生输入、消息气泡、增量流、结束和清理。
- 生产发布只来自锁定依赖、通过 CI 的提交；网站与 macOS 包分别按部署和发布文档执行。
