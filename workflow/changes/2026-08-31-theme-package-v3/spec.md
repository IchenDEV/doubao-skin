---
id: "2026-08-31-theme-package-v3"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-31"
based_on: intent.md
risk: "high"
approved_by: "product-risk-owner"
approved_at: "2026-08-31"
---

# Spec: 主题包 v3 多宿主兼容规范

## Requirements

本文中的“必须”“不得”“应当”是规范性要求。`schemaVersion: 3` 简称 v3；主题自己的 `version` 仍使用 SemVer，两者不得混用。

1. v3 `theme.json` 必须包含 `schemaVersion: 3`、`id`、`name`、`description`、`version`、`author`、`preview`、`shared` 和 `targets`。发布到主题商店时还必须包含 `store`；本地主题可以省略 `store`。
2. v3 支持三个正式目标 ID：`doubao`、`doubao-work`、`workbuddy`。`targets` 必须是非空对象，只允许使用这三个键；键是否存在是支持范围的唯一事实源。不得再增加 `supportedTargets`、`enabled` 或另一份平行列表。
3. `targets` 中缺少某个目标键表示主题不支持该目标，加载器不得尝试共享 CSS、旧 CSS 或其他目标 CSS，也不得以“部分颜色可能生效”为由允许应用。
4. `targets.<id>` 可以是空对象。空对象表示作者明确声明该目标只使用共享结构化视觉和引擎内置宿主适配器。存在结构覆盖、CSS、外观覆盖或专用预览时表示该目标具有专门适配。
5. `shared` 必须承载跨目标复用的视觉定义。允许字段为 `appearance`、`background`、`surfaceOpacity`、`typography`、`layout`、`composer`、`content`、`icons`、`effects`、`variants` 和 `css`。v2 位于顶层的同名视觉字段在 v3 中不得继续出现在顶层。
6. `targets.<id>` 允许使用与 `shared` 相同的视觉字段，另可提供完整的 `preview`。目标字段只覆盖该目标，不得影响其他目标。目标 `preview` 整体替换顶层通用预览，不做字段级合并。
7. `shared.appearance` 必须存在，取值继续为 `light-only`、`dark-only` 或 `both`。目标可以用自己的 `appearance` 缩窄或扩展外观范围；每个声明支持的目标在其每个有效外观下都必须解析出完整、可读的语义样式。
8. `variants.light` 和 `variants.dark` 在 v3 中可以覆盖 `background`、`surfaceOpacity`、`typography`、`layout`、`composer`、`content`、`icons`、`effects` 和 `css`。variant 不得再次包含 `appearance`、`variants`、`preview` 或 `targets`。
9. 结构化层的合并顺序固定为：共享基础层 → 共享当前外观 variant → 当前目标基础层 → 当前目标当前外观 variant。普通标量后者替换前者；对象按已知字段递归合并；普通数组整体替换；目标层和 variant 中的显式 `null` 删除继承值。`css` 数组不参与结构字段替换，而是按第 13 条组成独立的加载链。共享基础层不得使用 `null`。
10. 对每个目标和有效外观完成合并后，必须验证最小语义样式。至少需要：
    - `composer.background`、`border`、`textColor`、`placeholderColor`、`caretColor`、`iconColor`、`sendButtonBackground`、`sendButtonIconColor` 和 `radius`；
    - `content.chatBackground`、`userMessageBackground`、`userMessageText`、`assistantMessageBackground`、`assistantMessageText`、`codeBackground`、`codeHeaderBackground`、`selectionColor`、`scrollbarColor` 和 `scrollbarHoverColor`。
11. v3 不再隐式读取根目录 `theme.css`。CSS 入口只能来自各层的 `css` 数组；结构化字段完整时可以完全不包含主题 CSS。未被 manifest 引用的 `theme.css` 与其他样式文件不得自动发现或加载。
12. 每个 `css` 字段必须是按加载顺序排列的唯一相对路径数组，包含 1–8 个 `.css` 文件。路径区分大小写，必须使用 `/`，不得是绝对路径，不得包含空段、`.`、`..`、反斜杠、查询参数或 fragment。
13. 运行时 CSS 顺序固定为：
    1. 引擎根据最终结构化视觉生成的当前宿主基础适配 CSS；
    2. `shared.css`；
    3. `shared.variants.<appearance>.css`；
    4. `targets.<id>.css`；
    5. `targets.<id>.variants.<appearance>.css`；
    6. 引擎所有的运行时安全规则、用户“不透明度”覆盖和减少动态效果规则。

    每个数组内部保持 manifest 顺序。标准 CSS 级联仍决定不同优先级与 specificity；仅在 origin、importance 和 specificity 相同时由后加载规则覆盖前者。一个文件可以被多个目标引用以实现子集共享，但同一目标的有效加载链不得重复同一路径。
14. 共享 CSS 的每个选择器必须位于 `html[data-skin="<theme-id>"]` 作用域内，并且不得包含 `data-skin-target`。目标 CSS 的每个选择器必须同时包含主题 ID 和 `data-skin-target`；目标值只能来自引用该文件的目标集合。未作用域化的选择器使主题包校验失败。
15. v3 CSS 只允许视觉覆盖和自定义属性，不允许改变内容、交互或应用结构。必须禁止 `@import`、`@font-face`、`@keyframes`、所有 `url()`、远程资源、脚本协议、`content`、`display`、`visibility`、`pointer-events`、`position`、inset/坐标、`z-index`、尺寸与网格/弹性布局属性。背景、字体、图标、布局与动画必须通过结构化字段表达。允许的 CSS 范围包括颜色、背景色/渐变、边框、圆角、轮廓、阴影、透明度、滤镜、字体与文本属性、caret、fill/stroke、滚动条、accent-color、transition 以及非保留自定义属性。
16. 引擎保留 `--doubao-skin-runtime-*` 自定义属性和所有 `data-skin*` 运行时属性；主题 CSS 不得覆盖保留属性，也不得依赖应用进程、端口或身份识别信息。
17. `background`、字体资产、图标和预览图只能通过相应结构化字段引用包内文件。资源路径遵守与 CSS 相同的归一化和目录边界规则。共享资源通常可以被目标继承；目标层可以替换、局部覆盖或以 `null` 删除。当前 WorkBuddy 适配器不支持宿主图标替换，因此任何会从共享层解析出图标的 v3 主题都必须在 `targets.workbuddy.icons` 显式写 `null`，校验器必须拒绝仍解析出图标的 WorkBuddy 声明。
18. 每个 CSS 文件必须是合法 UTF-8 且不超过 512 KiB；单个目标解析后的 CSS 文件总量不得超过 2 MiB。现有压缩包 200 MiB、解压内容 512 MiB、最多 2,048 项的上限保持不变。
19. v3 JSON Schema 必须对规范字段使用 `additionalProperties: false`，以便发现拼写错误。来源与许可证迁移到显式 `provenance` 对象；不得依靠任意顶层字段扩展协议。未知目标 ID、未知字段、未知适配格式和缺失文件都必须拒绝安装，而不是忽略后继续声称兼容。
20. 支持级别必须从清单解析结果派生，不由作者重复填写：目标键缺失为 `unsupported`；目标键存在且目标层没有实质差异为 `shared`；目标层存在结构化差异、CSS、外观差异或专用预览为 `tailored`。另用 `declaration: explicit | legacy-inferred` 区分 v3 显式声明与旧主题推断，避免增加第四种支持级别。
21. v1/v2 不自动重写，兼容规则固定如下：v1 对 `doubao` 和 `doubao-work` 为 `legacy-inferred`，对 `workbuddy` 为 `unsupported`；v2 对三项目标均为 `legacy-inferred`，其中 WorkBuddy 继续只使用结构化字段和内置适配器、忽略 v2 原始 `theme.css`。v3 完全按 `targets` 精确加载。
22. 发现 `schemaVersion` 高于当前引擎支持值时必须失败关闭并提示升级豆皮；不得按 v1、v2 或缺省格式解析。旧版客户端无法识别 v3 时允许拒绝安装，不提供伪装成 v2 的双格式 manifest。
23. 主题商店目录必须由 `theme.json` 派生并输出 `schemaVersion`、`targets` 和各目标 `supportLevel`；目录不能手工维护第二份兼容列表。主题列表、商店筛选、CLI 校验和运行时能力判断必须使用同一解析结果。
24. 规范文档必须提供一个完整 v3 示例、三个最小支持范围示例、CSS 作用域示例、v2→v3 迁移步骤及错误示例。JSON Schema、Rust 解析器、Web 同步器和作者 CLI 的后续实现必须共享同一组契约 fixtures。规范 fixtures 必须使用 `fixture-` 前缀的合成 ID、合成名称与合成来源信息，不得冒充或复制某个真实内置主题。
25. v3 引擎、校验器与打包器通过契约测试后，必须迁移仓库 `themes/` 下的全部内置主题。当前评审基线为 30 个 v2 主题；合并前的确定性检查必须要求每个 `themes/*/theme.json` 都使用 `schemaVersion: 3`，因此迁移期间新增的内置主题也不能留下 v1/v2。
26. 30 个基线主题当前都支持 `appearance: both`，并通过 v2 兼容路径用于豆包、豆包工作和 WorkBuddy。迁移不得通过删除主题、减少 light/dark、移除预览/来源/素材或省略某个目标来降低现有能力；每个内置主题最终都必须显式声明三个目标。
27. 每个迁移主题必须把跨宿主结构化语义移入 `shared`，把原 `theme.css` 按真实职责拆为共享 CSS、豆包家族 CSS 和必要的 WorkBuddy CSS。背景伪元素、图标标记、布局与运行时安全规则中能够由引擎表达的内容必须删除，不得原样复制到三个目标文件。
28. 豆包与豆包工作可以在同一主题内共同引用一个 `doubao-family.css`；WorkBuddy 只能加载共享层与自己的目标层。只靠内置适配器已经达到验收标准的主题可以将 `targets.workbuddy` 保持为空对象，不得为满足目录形状而创建空 CSS 文件。
29. 每个主题自己的 `version` 必须按 SemVer 提升主版本，因为 v3 包会被旧版客户端拒绝；ID、面向用户的名称、描述、排序与合法来源信息保持稳定。资源路径可以整理，但素材内容不得无理由替换或重新压缩降质。
30. 内置主题迁移作为同一发布 Gate 原子完成：在全部主题通过 Schema、CSS 安全、资源、打包、目录同步和真实窗口验收前，不得把默认作者 CLI、商店目录或发布说明切换为“v3 已完成”。

## User experience

- 用户选择目标后，只显示该目标可应用的主题；不支持的主题仍可在“全部主题”或商店详情中浏览，但主操作禁用并说明“这个主题不支持当前应用”。不得尝试部分加载后让用户自行判断。
- 主题卡片默认只显示普通用户需要的“支持豆包 / 豆包工作 / WorkBuddy”。`tailored` 可以用低干扰文案显示“专属适配”；`shared` 显示“支持”。`legacy-inferred` 显示“兼容模式”，但不暴露 Schema、CSS 文件名或内部目标 ID。
- 切换目标时，预览优先使用 `targets.<id>.preview`，没有时使用顶层通用 `preview`。预览回退不改变兼容声明。
- 切换目标只改变桌面端管理上下文，已经应用到其他目标的主题继续由各自 watcher 维护。每个目标的 apply、active、restore 和完成 generation 必须独立；恢复一个目标不得停止或清除其他目标。
- 安装失败必须给出主题作者可操作的精确原因，例如“WorkBuddy 样式文件不存在”或“CSS 未限制在 WorkBuddy 作用域”；面向普通用户的桌面 UI 使用简短结果，并允许查看或复制开发者详情。
- 作者 CLI 的 `validate` 输出每个目标的支持级别、有效外观、解析后的 CSS 加载顺序和资源清单。`package` 只有在全部声明目标都通过时才能生成 `.doubao-skin.zip`。
- 商店允许按目标过滤；主题详情必须展示 manifest 声明的目标，不能把“v3”直接等同于“支持全部应用”。
- 已安装 v1/v2 主题继续显示并按兼容矩阵运行，不强制迁移，也不把旧主题无声明误写成 v3 显式支持。
- 内置主题迁移完成后，用户仍看到相同的 30 个主题、名称、描述、预览和排序；格式升级不制造重复主题，也不清空已选主题。主题 ID 保持不变，因此已有选择偏好可以继续命中迁移后的包。
- 内置主题以一次完整目录更新交付。若任一主题验证失败，产品继续使用当前兼容目录，不向用户展示一半 v2、一半 v3 的“迁移完成”状态。

## Technical design

### 1. Manifest 形状

v3 使用“元数据 + 共享视觉层 + 目标差异层”。下面是鲸鱼娘的规范示例；它共享主要视觉语义，豆包与豆包工作共用一份宿主 CSS，WorkBuddy 使用自己的 CSS 并删除共享图标：

```json
{
  "$schema": "../../design/theme-standard/theme-v3.schema.json",
  "schemaVersion": 3,
  "id": "gallery-whale-maid",
  "name": "鲸鱼娘",
  "description": "浅青晴空与蓝鲸少女，明亮又俏皮",
  "version": "2.0.0",
  "author": "豆皮",
  "preview": {
    "image": "preview.jpg",
    "aspectRatio": "16:9",
    "appearance": "light",
    "accent": "#9b7a5e"
  },
  "store": {
    "category": "brand",
    "tags": ["浅色", "蓝色", "角色"],
    "sortOrder": 20
  },
  "provenance": {
    "inspiredBy": "DreamSkin DeepSeek-鲸鱼娘",
    "sourceUrl": "https://dreamskin.cc/themes/ver_cb557ececaa5de3f3dbe",
    "sourceVersion": "ver_cb557ececaa5de3f3dbe",
    "license": "MIT",
    "artwork": "Original ImageGen adaptation"
  },
  "shared": {
    "appearance": "both",
    "surfaceOpacity": 0.68,
    "background": {
      "type": "image",
      "src": "assets/bg.jpg",
      "fit": "cover",
      "position": "center",
      "opacity": 1,
      "veil": 0.04,
      "blur": 0,
      "animation": "none",
      "durationSeconds": 20
    },
    "typography": {
      "ui": "-apple-system, BlinkMacSystemFont, PingFang SC, sans-serif",
      "body": "-apple-system, BlinkMacSystemFont, PingFang SC, sans-serif",
      "code": "SFMono-Regular, Menlo, monospace",
      "scale": 1.01,
      "lineHeight": 1.62
    },
    "composer": {
      "background": "rgba(255,255,255,0.96)",
      "border": "1px solid rgba(122,78,41,0.28)",
      "textColor": "#352970",
      "placeholderColor": "rgba(53,41,112,0.60)",
      "caretColor": "#7a4e29",
      "iconColor": "rgba(53,41,112,0.82)",
      "sendButtonBackground": "#7a4e29",
      "sendButtonIconColor": "#ffffff",
      "radius": 22
    },
    "content": {
      "chatBackground": "rgba(189,153,153,0.16)",
      "userMessageBackground": "#7a4e29",
      "userMessageText": "#ffffff",
      "assistantMessageBackground": "rgba(255,255,255,0.96)",
      "assistantMessageText": "#352970",
      "codeBackground": "#f0f6ff",
      "codeHeaderBackground": "#e6edf8",
      "selectionColor": "rgba(122,78,41,0.24)",
      "scrollbarColor": "rgba(122,78,41,0.26)",
      "scrollbarHoverColor": "rgba(122,78,41,0.42)"
    },
    "icons": {
      "main": "assets/icons/main.png"
    },
    "effects": {
      "radiusScale": 1.15,
      "shadow": "0 12px 34px rgba(65,43,50,0.14)",
      "blur": 18,
      "motion": "gentle",
      "transitionMs": 180
    },
    "css": ["styles/shared.css"],
    "variants": {
      "light": {},
      "dark": {
        "composer": {
          "background": "rgba(29,31,37,0.96)",
          "border": "1px solid rgba(155,122,94,0.38)",
          "textColor": "#f7f8fa",
          "placeholderColor": "rgba(247,248,250,0.58)",
          "caretColor": "#9b7a5e",
          "iconColor": "rgba(247,248,250,0.82)",
          "sendButtonBackground": "#9b7a5e",
          "sendButtonIconColor": "#ffffff"
        },
        "content": {
          "chatBackground": "rgba(18,20,25,0.88)",
          "userMessageBackground": "#9b7a5e",
          "userMessageText": "#ffffff",
          "assistantMessageBackground": "rgba(29,31,37,0.94)",
          "assistantMessageText": "#f7f8fa",
          "codeBackground": "rgba(0,0,0,0.30)",
          "codeHeaderBackground": "rgba(0,0,0,0.42)",
          "selectionColor": "rgba(155,122,94,0.28)",
          "scrollbarColor": "rgba(155,122,94,0.28)",
          "scrollbarHoverColor": "rgba(155,122,94,0.46)"
        }
      }
    }
  },
  "targets": {
    "doubao": {
      "css": ["styles/doubao-family.css"]
    },
    "doubao-work": {
      "css": ["styles/doubao-family.css"]
    },
    "workbuddy": {
      "icons": null,
      "css": ["styles/workbuddy.css"],
      "preview": {
        "image": "preview-workbuddy.jpg",
        "aspectRatio": "16:9",
        "appearance": "light",
        "accent": "#9b7a5e"
      }
    }
  }
}
```

最小支持范围写法：

```json
{ "targets": { "workbuddy": {} } }
```

```json
{ "targets": { "doubao": {}, "doubao-work": {} } }
```

```json
{ "targets": { "doubao": {}, "doubao-work": {}, "workbuddy": {} } }
```

这些片段只展示 `targets`，完整主题仍必须包含 v3 的其余必需字段。

### 2. 解析与能力模型

加载器先验证 Schema 和文件边界，再为当前目标解析 `ResolvedTheme`。`ResolvedTheme` 只包含当前目标、当前外观、已合并视觉字段、按顺序排列的 CSS 文件、解析后的资源路径、`supportLevel` 和 `declaration`。应用层不得再次读取原始 JSON 猜测兼容性。

目标支持判断为纯函数：

```text
schema v3 + target key absent              -> unsupported / explicit
schema v3 + target key present, no delta   -> shared / explicit
schema v3 + target key present, has delta  -> tailored / explicit
schema v1/v2 inferred by compatibility map -> shared or tailored / legacy-inferred
```

`targets` 键同时驱动安装校验、桌面主题可用性、商店筛选和目录导出。目标层中与共享层完全相同的冗余值应产生作者警告，并在派生支持级别时视为无实质差异。

### 3. CSS 作用域与子集共享

共享 CSS 示例：

```css
html[data-skin="gallery-whale-maid"] {
  --whale-accent-soft: rgba(122, 78, 41, 0.24);
}
```

WorkBuddy 专用 CSS 示例：

```css
html[data-skin="gallery-whale-maid"][data-skin-target="workbuddy"] .workbench-part {
  border-color: var(--whale-accent-soft);
  box-shadow: 0 8px 24px rgba(65, 43, 50, 0.10);
}
```

豆包与豆包工作共享同一文件时，该文件可以使用两个受控分支：

```css
html[data-skin="gallery-whale-maid"][data-skin-target="doubao"],
html[data-skin="gallery-whale-maid"][data-skin-target="doubao-work"] {
  --semi-color-primary: #7a4e29;
}
```

校验器根据所有 manifest 引用反向计算某 CSS 文件允许出现的目标集合。文件被 `doubao` 和 `doubao-work` 共同引用并不使其成为全局共享 CSS；它仍然不会被 WorkBuddy 加载。

### 4. v2 到 v3 迁移

1. 将 `schemaVersion` 改为 `3`，提升主题自身 `version`。
2. 保留元数据、通用预览和商店字段；把来源字段整理进 `provenance`。
3. 把原顶层 `appearance`、背景、透明度、字体、布局、输入框、内容、图标、effects 和 variants 移入 `shared`。
4. 不再依赖根目录 `theme.css`。把真正跨三个目标的根变量移入 `shared.css`；把豆包 DOM/Semi Design 规则移入由 `doubao`、`doubao-work` 引用的 CSS；把 WorkBuddy 规则移入 `workbuddy` CSS。
5. 在 `targets` 中只声明实际验证过的目标。未验证的目标保持缺失，不能先声明再靠运行时回退。
6. 对不适用于某目标的继承字段使用 `null` 删除，例如 WorkBuddy 当前不使用图标替换时可写 `"icons": null`。
7. 分别解析每个目标的 light/dark 有效配置，运行 Schema、CSS 安全、文件完整性和真实窗口验收后再发布。

### 5. 内置主题迁移基线

本次“全部现有主题”固定包含以下 30 个 ID：

```text
claude-warm
codex-catppuccin
codex-dracula
codex-gruvbox
codex-nord
codex-one-half
codex-solarized
cyber-neon
doubao-dessert-giggle
doubao-snack-giggle
forest
gallery-cozy-room
gallery-crimson-rain
gallery-moon-pine
gallery-neon-koi
gallery-whale-maid
github-repository
gothic-void
huaxia-blue
machine-overseer
mist-forest
ocean-cyan
pastel-flower-club
pastel-starry-room
pastel-tea-party
peach-sunset
pure-dark
qq-light-blue
sakura-night
violet-night
```

迁移清单由 `themes/*/theme.json` 实时生成并与该基线比较。基线主题缺失是删除回归；新增目录仍必须遵守 v3。批量工具只能生成机械、安全的字段移动与路径建议，最终目标层、CSS 删除/保留和支持声明必须按主题检查。

每个主题都必须保存一条迁移记录，至少包含：原/新主题版本、目标声明、共享/目标 CSS 文件、删除的旧运行时规则、保留资源、light/dark 解析结果、三应用实窗结果与偏差。该记录可以集中写入后续 `verification.md` 的矩阵，不要求在 30 个目录中复制文档。

### 6. 兼容矩阵

| 格式 | 豆包 | 豆包工作 | WorkBuddy | 声明来源 |
| --- | --- | --- | --- | --- |
| v1 | 兼容原始 CSS | 兼容原始 CSS | 不支持 | `legacy-inferred` |
| v2 | 结构字段 + 原始 CSS | 结构字段 + 原始 CSS | 仅结构字段 + 内置适配器 | `legacy-inferred` |
| v3 | 仅当 `targets.doubao` 存在 | 仅当 `targets.doubao-work` 存在 | 仅当 `targets.workbuddy` 存在 | `explicit` |

v3 的某个目标即使只写空对象，也仍会经过该目标的引擎内置适配器；这不代表加载其他目标的 CSS。

### 7. 文件与错误处理

- Schema 错误、未知字段/目标、CSS 解析失败、CSS 越界、保留属性覆盖、缺失资源、路径逃逸、重复有效 CSS、无目标或任一已声明目标解析不完整，均为主题包级致命错误。
- 当前目标不存在于 `targets` 是正常的不兼容结果，不是损坏的主题包。
- 包内未引用文件永不加载；作者打包器省略它们并给出警告。`README*`、`LICENSE*` 和 `NOTICE*` 可以作为非运行时文件保留。
- 导入器先在临时目录完整校验，再原子替换用户主题目录；失败时不得留下半安装主题。
- CLI 和开发者详情必须指出 JSON Pointer、目标、外观、文件和行列（适用时），便于修复。

## Security and privacy

- v3 主题继续是无执行能力的数据包。不得提供 JavaScript、HTML、WASM、原生库、命令、启动参数、进程规则或网络配置入口。
- CSS 只能影响当前已由运行时确认身份的目标页面，并受主题 ID 与目标 ID 双重根作用域约束。目标 CSS 不得进入 iframe、webview、文档正文隔离区或不属于当前目标的页面。
- 禁止 CSS 网络能力和 `url()`，所有素材只能来自经过路径校验和类型/大小校验的包内结构化资源，避免远程追踪、数据外带和更新后内容漂移。
- CSS 的视觉属性白名单必须由解析器执行，不能只用字符串搜索。注释、转义、嵌套 at-rule、大小写和 Unicode 逃逸都不得绕过校验。
- 背景、字体、图标与预览资源必须验证真实文件类型而不只信任扩展名；SVG 必须继续执行现有的安全清理规则，不允许脚本、外链或事件处理器。
- v3 不改变 CDP 回环绑定、目标 URL 身份验证、进程精确匹配、恢复默认或用户重启授权。manifest 中出现类似字段必须因未知字段被拒绝。
- UI 不读取或展示用户会话、任务、账号、Cookie、工作空间、附件或网络内容来判定主题兼容性；兼容性只来自包清单与引擎能力。
- 主题更新必须重新执行完整包校验。已安装旧版本在新包验证完成前保持可恢复，失败更新不得覆盖可用版本。

## Alternatives and non-goals

- 不使用 `supportedTargets` 数组加 `targets` 对象的双声明方案；两份范围会漂移，`targets` 键已经足够表达支持。
- 不继续把根目录 `theme.css` 当作所有宿主的默认入口；它无法表达宿主隔离，也会让 WorkBuddy 再次意外加载豆包规则。
- 不强制每个目标复制一份完整主题。共享结构字段是推荐主路径，同一 CSS 文件也可以被目标子集共同引用。
- 不把任意自定义 JavaScript称为“高级主题能力”；宿主适配由可信引擎维护，主题只提供声明式视觉数据和受限 CSS。
- 不允许目标条目覆盖引擎适配器名称或版本。当前支持的宿主集合由应用定义，不把 v3 变成任意应用插件注册表。
- 不自动改写仓库外的 v2 主题。仓库内 30 个基线主题必须迁移为三目标 v3，但迁移仍以每个主题的真实验证结果为依据，不能只做字符串批量替换。
- 不在本规范中定义主题组合、父主题继承、远程依赖、按应用版本选择 CSS 或目标 DOM selector API 稳定性承诺。

## Areas of concern

- WorkBuddy 与两款豆包的 DOM 会随官方版本变化。`tailored` 表示包内有专用层，不代表未来版本永久兼容；商店和验证证据仍需记录实测应用版本。
- 允许目标 CSS 意味着 WorkBuddy 不再只接收引擎生成 CSS。安全性依赖严格 CSS parser、属性白名单、双重作用域、无网络资源和真实窗口回归，不能退化成当前的文本包含检查。
- `!important`、高 specificity 与宿主官方 CSS 可能导致作者期望的顺序不成立。CLI 应报告重复/冲突的关键声明，但不能把“文件后加载”描述为无条件覆盖。
- 同一 CSS 文件被多个目标引用可以减少重复，也容易误写目标选择器。反向引用集合与逐选择器校验必须是一等契约测试。
- `null` 删除带来清晰的“不继承”能力，也要求解析器区分缺失与显式删除；直接反序列化到全部 `Option<T>` 会丢失这个语义。
- v3 严格拒绝未知字段会增加迁移修复量，但可以阻止拼写错误造成假兼容。发布文档和 CLI 必须提供明确迁移诊断。
- 老客户端可能只认识必需的 `theme.css` 并拒绝 v3。主题商店必须根据客户端支持的最高 Schema 过滤包，不能让下载成功代替兼容性判断。
- 当前工作区已有尚未完成最终 Gate 的 WorkBuddy 变更。v3 规范与后续实现必须建立在其最终行为上，但不能把那项变更的待验收结果写成已经稳定的宿主契约。
- 30 个主题 × 3 个目标 × 2 种外观形成 180 个基本实窗场景。可以用自动化启动、应用、截图、像素/对比度探针和联系表减少人工操作，但每个场景都必须来自真实应用窗口，抽样不能替代逐主题通过记录。
- 旧 CSS 中存在背景伪元素、`display`、`position`、`pointer-events`、`content` 与 `url()` 等 v3 禁止能力。迁移必须把相应职责转回结构化背景或可信运行时，不能放宽 v3 安全规则来迁就旧文件。

## Acceptance criteria

1. `theme-v3.schema.json` 能接受规范中的完整示例，并拒绝空 `targets`、未知目标、未知字段、顶层 v2 视觉字段、错误路径和错误 variant 结构。
2. 契约 fixtures 分别覆盖仅 WorkBuddy、仅两款豆包和三目标主题，三者的支持范围与 UI/目录导出结果完全一致。
3. 解析器对同一 fixture 按规定顺序合并共享/variant/target/target-variant；测试覆盖标量替换、对象递归合并、数组替换和 `null` 删除。
4. 对每个声明目标与有效外观，解析器都生成完整 `ResolvedTheme`；任一最终语义必需字段缺失时整个 v3 包校验失败，并指出目标与外观。
5. 一个无 CSS 的结构化 v3 主题可以通过内置适配器应用；根目录存在但未声明的 `theme.css` 不会被加载。
6. CSS 顺序测试精确证明 engine → shared → shared variant → target → target variant → runtime；数组内部顺序稳定，单目标有效链重复文件被拒绝。
7. CSS parser 拒绝未作用域选择器、错误目标、`@import`、`url()`、脚本/远程资源、保留变量及交互/结构属性；注释、转义和嵌套写法不能绕过。
8. 同一 `doubao-family.css` 被豆包与豆包工作共同引用时，两者可加载、WorkBuddy 不加载；文件出现 `workbuddy` 选择器时校验失败。
9. 所有 CSS 和资源路径都经过规范化、主题根目录边界、符号链接与大小检查；目录穿越、绝对路径、反斜杠和缺失文件被拒绝。
10. 支持级别正确派生为 `unsupported`、`shared` 或 `tailored`，声明来源独立派生为 `explicit` 或 `legacy-inferred`；manifest 不存在第二份可漂移状态。
11. v1/v2 回归 fixtures 按兼容矩阵保持现有行为；`schemaVersion > 3` 明确失败，不按旧格式回退。
12. 作者 CLI 对每个目标输出外观、支持级别、CSS 顺序和资源，任何声明目标失败都会阻止打包；生成包只包含 manifest 引用资源和允许的说明/许可文件。
13. Web 同步目录从 manifest 派生目标信息；目标筛选、详情标签、下载资格和桌面应用能力判断使用同一 fixture，不能手工硬编码主题 ID。
14. 鲸鱼娘迁移 fixture 证明共享结构只定义一次、豆包家族 CSS 只保存一份、WorkBuddy 只加载自己的 CSS、WorkBuddy 图标继承被明确删除。
15. 三个目标分别完成浅色和深色真实窗口验证，检查背景、侧栏、主内容、类型切换、输入区、按钮、弹层、代码块、选区和滚动条；文字对比度、透明层次、边框重量和恢复默认均通过。
16. `./scripts/check.sh rust`、`pnpm --dir apps/web sync && ./scripts/check.sh web`、`./scripts/check.sh workflow`、JSON Schema fixtures、主题打包/导入测试以及触及范围的完整检查通过，并在后续 `verification.md` 记录证据和剩余应用版本风险。
17. 基线中的 30 个主题目录全部仍存在、ID 不变，并全部使用 `schemaVersion: 3`；确定性测试扫描整个 `themes/`，发现任一 v1/v2 或缺失三目标声明即失败。
18. 30 个主题的名称、描述、预览、商店分类/排序、来源/许可证和必要素材迁移前后保持等价；Web 同步后的目录主题数不变且没有重复 ID。
19. 每个主题的版本按 SemVer 提升主版本，全部结构化语义可为三个目标解析 light/dark；原 `theme.css` 中被 v3 禁止的背景/布局/交互规则已删除或由可信引擎等价承接。
20. 每个主题都完成豆包、豆包工作和 WorkBuddy 的浅色/深色实际窗口检查，共 180 个基本场景；验证矩阵逐项记录 pass/fail、应用版本、截图证据和偏差，任何失败都阻塞“全部迁移完成”。
21. 对 30 个主题逐一执行创建可安装包、从干净临时主题目录导入、目标能力读取和资源完整性校验；随后同步 Web 目录并证明应用与网站使用相同的三目标元数据。

## Decision

产品与安全风险负责人已明确接受包含“全部 30 个内置主题迁移”的本 Spec，同意进入 Plan。当前批准的是扩展后的数据契约、迁移范围、兼容行为与验收边界；Spec 接受本身尚未授权修改加载器、主题包、商店数据或产品 UI。
