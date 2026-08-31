# 豆皮标准

这份标准同时约束主题选择器 UI 和 `themes/<theme-id>/` 主题包。目标是让第一次使用换肤工具的人只需要完成三件事：选择主题、查看预览、应用主题。

## 1. 产品界面

### 主界面只保留

1. 主题列表
2. 当前主题的大图预览
3. 主题名称和一句自然语言描述
4. 「应用主题」主按钮
5. 「恢复默认」次要操作
6. 带背景的主题显示「界面不透明度」调节；纯色主题不显示

主界面不显示日志、主题 ID、端口、文件路径、注入方式、构建模式或运行进程。系统自动选择应用方式，失败时只告诉用户结果和可执行的下一步。

### 信息结构

- 标准窗口：1120 × 720，可缩放，最小 720 × 560。
- 宽窗口：左侧 220–280 pt 主题列表，右侧为预览和操作。
- 窄窗口：主题列表收进工具栏弹出层，预览和主按钮保持可见。
- 预览使用 16:9，默认窗口下无需滚动即可看到主按钮。
- 不设置日志区、控制台、状态栏或开发者面板。

详细几何约束见 [layout-spec.json](./layout-spec.json)，设计令牌见 [tokens.json](./tokens.json)。

## 2. 小白文案

### 允许使用的词

| 场景 | 文案 |
| --- | --- |
| 默认主操作 | 应用主题 |
| 处理中 | 正在应用… |
| 成功 | 已应用 |
| 当前主题 | 正在使用 |
| 还原 | 恢复默认 |
| 普通失败 | 应用失败，请再试一次 |
| 豆包工作未打开 | 请先打开豆包工作 |
| 主题不可用 | 这个主题暂时不可用 |

### 禁止出现在用户界面

- Live、离线构建、注入、CDP、WebSocket
- 调试端口、进程、监控页面、目标页
- `sakura-night` 一类内部 ID
- `~/Applications` 一类文件路径
- 皮肤版、构建产物、签名、权限实现细节
- 原始错误栈和英文日志

错误详情写入开发日志，不放进主窗口。用户看到的错误必须包含一个明确动作，例如「再试一次」或「打开豆包工作」。

## 3. 视觉标准

- 使用原生 macOS 标题栏、交通灯和 SF Pro。
- 外壳使用系统语义色；主题颜色只进入预览、选中态和主按钮。
- 间距基于 4 pt，主要使用 8、12、16、20、24 pt。
- 普通控件高 28 pt，主按钮高 32 pt；避免移动端尺寸的大按钮。
- 主题行高 44 pt，名称单行显示，不展示内部 ID。
- 一屏只有一个高强调主按钮；「恢复默认」使用安静的文字操作。
- 不使用蓝紫渐变、发光、悬浮卡片、胶囊标签或大面积玻璃效果。
- 预览是视觉焦点，主题列表和操作区保持中性。

### 状态

- 默认：中性文字和表面。
- 悬停：表面亮度轻微变化，元素不位移、不缩放。
- 选中：主题强调色 18% 透明度背景，加清晰文字；不用发光边框。
- 键盘焦点：使用系统焦点环。
- 禁用：保留可读文字，整体不透明度不低于 45%。
- 应用中：按钮文案变为「正在应用…」，禁止重复点击。
- 已应用：主题行显示系统勾选符号，按钮文案变为「正在使用」。

## 4. 主题包契约

新主题使用 [主题包 v3 多应用规范](./theme-v3.md) 和 [v3 JSON Schema](./theme-v3.schema.json)。v3 用 `targets` 精确声明豆包、豆包工作和 WorkBuddy 的适用范围，共享视觉放在 `shared`，目标差异放在对应目标层；不再隐式读取根目录 `theme.css`。

下文保留 v1/v2 契约，供已安装的旧主题兼容使用。新建、投稿和内置主题均应使用 v3。

每个主题使用独立目录：

```text
themes/<theme-id>/
├── theme.json        必需
├── theme.css         必需
├── preview.jpg       上架主题商店时必需，1200 × 675
├── bg.jpg / bg.mp4   可选
├── fonts/            可选
├── icons/            可选
└── icon.icns         可选
```

旧主题包继续支持原有写法，不需要迁移：

```json
{
  "id": "sakura-night",
  "name": "夜樱",
  "description": "深色樱花背景，柔和粉色点缀",
  "background": "bg.jpg",
  "veil": 0.25
}
```

需要调整字体、布局、输入框、消息、图标或动态背景时，使用第二版配置。完整字段及取值范围以 [theme-v2.schema.json](./theme-v2.schema.json) 为准，可直接参考 [`桃气日落`](../../themes/peach-sunset/theme.json)：

```json
{
  "schemaVersion": 2,
  "id": "my-theme",
  "name": "我的主题",
  "description": "温暖柔和，适合长时间阅读",
  "version": "1.0.0",
  "author": "主题作者",
  "preview": {
    "image": "preview.jpg",
    "aspectRatio": "16:9",
    "appearance": "light",
    "accent": "#d85f76"
  },
  "store": {
    "category": "pure",
    "tags": ["浅色", "粉色", "渐变"],
    "sortOrder": 100
  },
  "appearance": "both",
  "surfaceOpacity": 0.68,
  "background": {
    "type": "video",
    "src": "bg.mp4",
    "poster": "poster.jpg",
    "veil": 0.24,
    "animation": "none"
  },
  "typography": {
    "body": "My Sans, sans-serif",
    "code": "My Mono, monospace",
    "scale": 1,
    "lineHeight": 1.6,
    "assets": [
      { "family": "My Sans", "src": "fonts/my-sans.woff2", "weight": "normal" }
    ]
  },
  "layout": {
    "sidebarWidth": 252,
    "chatMaxWidth": 920,
    "composerMaxWidth": 760,
    "selfMessageMaxWidth": 420,
    "chatMargin": 28
  },
  "composer": {
    "background": "rgba(255,255,255,.88)",
    "border": "1px solid rgba(0,0,0,.12)",
    "textColor": "#282522",
    "placeholderColor": "rgba(40,37,34,.48)",
    "caretColor": "#d85f76",
    "iconColor": "#75545b",
    "sendButtonBackground": "#d85f76",
    "sendButtonIconColor": "#ffffff",
    "radius": 22,
    "minHeight": 52,
    "padding": 14,
    "gap": 10,
    "iconSize": 20
  },
  "content": {
    "chatBackground": "rgba(255,250,248,.72)",
    "userMessageBackground": "#d85f76",
    "userMessageText": "#ffffff",
    "assistantMessageBackground": "rgba(255,255,255,.84)",
    "assistantMessageText": "#4c3037",
    "codeBackground": "rgba(89,54,61,.08)",
    "selectionColor": "rgba(216,95,118,.24)"
  },
  "icons": {
    "main": "icons/main.svg",
    "newTask": "icons/new-task.svg",
    "scheduled": "icons/scheduled.svg",
    "send": "icons/send.svg",
    "stop": "icons/stop.svg",
    "attach": "icons/attach.svg",
    "voice": "icons/voice.svg",
    "tools": "icons/tools.svg",
    "knowledge": "icons/knowledge.svg",
    "moreSkills": "icons/more-skills.svg",
    "dailyWork": "icons/daily-work.svg",
    "contentCreation": "icons/content-creation.svg",
    "research": "icons/research.svg",
    "design": "icons/design.svg",
    "readAloud": "icons/read-aloud.svg",
    "copy": "icons/copy.svg",
    "sidebar": "icons/sidebar.svg"
  },
  "variants": {
    "light": {
      "composer": {
        "background": "rgba(255,255,255,.92)",
        "border": "1px solid rgba(40,37,34,.14)",
        "textColor": "#282522"
      }
    },
    "dark": {
      "composer": {
        "background": "rgba(33,29,31,.94)",
        "border": "1px solid rgba(255,255,255,.16)",
        "textColor": "#fff8fa"
      },
      "icons": {
        "main": "icons/main-dark.svg"
      }
    }
  },
  "effects": {
    "radiusScale": 1.1,
    "shadow": "0 12px 34px rgba(0,0,0,.14)",
    "blur": 12,
    "motion": "gentle",
    "transitionMs": 180
  }
}
```

- `id`：必需，小写 ASCII kebab-case，必须与目录名一致。
- `name`：必需，2–8 个中文字符，面向普通用户。
- `description`：必需，一句话说明视觉感受，不写实现方式。
- `version` / `author`：上架商店时必需；版本使用语义化的 `1.0.0` 格式。
- `preview`：主题包自带的 16:9 界面预览。`appearance` 指定预览采用浅色还是深色，`accent` 用于商店按钮和占位状态。
- `store`：上架信息。分类、标签和排序都随包分发，不再由商店按主题 ID 硬编码。
- 第一版 `background`：可选，相对主题目录的图片路径；`veil` 范围 0–1，建议 0.15–0.35。
- `surfaceOpacity`：背景上方聊天区、侧栏、消息和输入框的默认不透明度，范围 0.35–1；越低越容易看清背景，文字、图标、菜单和强调按钮不跟随变透明。
- `appearance`：`light-only` 仅浅色、`dark-only` 仅深色、`both` 同时适配深浅色并跟随所选客户端当前外观；只有豆包页面尚未提供模式标记时才回退到系统外观。
- `mode`：旧主题兼容字段，`light`、`dark`、`auto` 分别映射到上述三种能力；新主题无需再填写。
- `icons`：所有字段均可选。除通用入口外，也可替换企业知识、更多技能、四类首页推荐、自动播报、复制和侧栏图标；深浅色需要不同素材时放在 `variants.light.icons` 与 `variants.dark.icons`。
- 第二版 `background`：图片、视频或渐变；视频必须提供静态封面，系统开启「减少动态效果」时停止背景动画。
- `typography`：界面字体、正文字体、代码字体、字号比例、行高和主题内字体文件。
- `layout`：侧栏、聊天区、输入区、自己发送的消息和两侧留白宽度。
- `composer`：输入框的颜色、边框、圆角、高度、间距，以及输入与发送按钮的颜色和尺寸。
- `content`：聊天背景、双方消息、代码块、文字选中和滚动条颜色。
- `icons`：可替换主品牌、侧栏导航、发送、停止、附件、语音和工具图标，推荐单色 SVG；`main` 也可使用彩色 SVG。
- `variants.light` / `variants.dark`：分别覆盖该模式下的 `composer`、`content` 与 `icons`；未填写的字段继续使用主题顶层配置。`appearance` 为 `both` 时必须同时提供两项。
- `effects`：全局圆角比例、阴影、模糊、过渡速度和动态偏好。
- 第三方色板主题额外保留 `derivedFrom`、`sourceUrl`、`sourceCommit` 和 `license`，只用于来源与许可证追溯，不在产品界面展示。

第二版字段由主题引擎统一映射到豆包工作，不要求主题作者查找应用内部变量。`theme.css` 只负责主题的基础色板和个别无法由标准字段表达的细节。

### CSS 作用域

所有变量覆写必须同时作用于 `html` 和 `body`：

```css
html[data-skin][data-theme=dark],
html[data-skin][data-theme=dark] body {
  /* theme variables */
}
```

主题至少提供以下变量，供预览和主界面识别：

- `--dbx-bg-body-web`：侧栏主色
- `--s-color-bg-body`：内容区主色
- `--semi-color-primary`：唯一强调色
- `--semi-color-primary-hover`
- `--semi-color-primary-active`
- `--semi-color-primary-disabled`

完整主题继续覆盖豆包工作现有的 `--N*`、`--B*` 和相关语义表面变量；不要新增第二套平行命名体系。

### 色彩与可读性

- 一个主题只使用一个主强调色，hover 和 active 从同一色相调整明度。
- 正文与背景对比度至少 4.5:1，大号文字至少 3:1。
- 有背景图的主题，内容表面必须保持足够不透明度；文字区域不能直接压在高频细节上。
- 不把文字、品牌标记或渐变按钮烘焙进背景图。
- 背景图建议至少 1920 px 宽、16:9 或更宽、无水印，文件建议不超过 1.5 MB。
- 纯色主题不需要伪造背景图。

## 5. 验收清单

- 主题名称、描述对普通用户可理解。
- 主窗口没有任何技术术语或日志。
- 720 × 560 下仍能选择主题、看预览并应用。
- 主题列表支持方向键，Return 执行「应用主题」，Esc 关闭弹出层。
- 主题预览、名称和按钮共享同一左右边界。
- Light/Dark、增加对比度、粗体文本、减少动态效果均有可用状态。
- 主题正文和按钮满足对比度要求。
- 应用失败时不暴露内部错误，只给出明确下一步。
