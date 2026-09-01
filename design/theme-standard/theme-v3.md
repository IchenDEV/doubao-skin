# 主题包 v3：多应用适配

v3 让一套主题明确声明适用于「豆包」「豆包工作」和 WorkBuddy 中的哪些应用，并允许共享视觉语义、按应用补充差异。字段的最终约束以 [theme-v3.schema.json](./theme-v3.schema.json) 为准；[valid-full.json](./fixtures/v3/valid-full.json) 是使用 `fixture-full` ID 的完整合成示例，仅用于文档与契约测试，不会进入主题目录或发布包。

## 支持范围

`targets` 的键是唯一支持声明：

```json
{ "targets": { "workbuddy": {} } }
```

```json
{ "targets": { "doubao": {}, "doubao-work": {} } }
```

```json
{ "targets": { "doubao": {}, "doubao-work": {}, "workbuddy": {} } }
```

这些片段只展示支持范围；完整主题仍需 `schemaVersion`、元数据、`preview`、`shared` 和其余必填字段。缺少目标键表示不支持，引擎不会尝试加载共享 CSS 或其他应用的样式。

支持级别由解析器派生，作者不填写：空目标层为“支持”；有结构覆盖、专用 CSS、外观差异或专用预览为“专属适配”。v1/v2 主题显示“兼容模式”。

## 结构与合并

跨应用的视觉语义放进 `shared`；目标差异放进 `targets.<id>`。每个有效外观按以下顺序合并：

1. `shared`
2. `shared.variants.<light|dark>`
3. `targets.<id>`
4. `targets.<id>.variants.<light|dark>`

后层标量覆盖前层，对象按已知字段递归合并，数组整体替换。目标层或 variant 的 `null` 删除继承值。目标 `preview` 整体替换通用 `preview`。

`shared.appearance` 必填；目标可用自己的 `appearance` 扩展或缩窄范围。每个目标的每种有效外观都必须解析出完整的输入区和内容区语义颜色。

## CSS 入口与作用域

v3 不会自动加载根目录 `theme.css`。CSS 只能通过各层的 `css` 数组引用，最终顺序为：引擎结构化样式 → 共享基础 CSS → 共享外观 CSS → 目标 CSS → 目标外观 CSS → 运行时安全规则。

共享 CSS 只限制在主题作用域：

```css
html[data-skin="my-theme"] {
  --my-accent-soft: rgba(51, 112, 235, 0.22);
}
```

目标 CSS 同时限制主题与目标：

```css
html[data-skin="my-theme"][data-skin-target="workbuddy"] .workbench-part {
  border-color: var(--my-accent-soft);
}
```

豆包家族可共享一个目标文件：

```css
html[data-skin="my-theme"][data-skin-target="doubao"],
html[data-skin="my-theme"][data-skin-target="doubao-work"] {
  --my-toolbar-tint: rgba(51, 112, 235, 0.12);
}
```

主题 CSS 只允许受控的视觉属性和非保留自定义属性。`@import`、`url()`、内容生成、交互、定位、尺寸以及网格/弹性布局属性都会校验失败；背景、字体、图标、布局和动态效果应使用结构化字段。

## 从 v2 迁移

先预演，不会改文件：

```bash
doubao-skin migrate-v3 themes/<theme-id> --json
```

确认目标范围和来源信息后写入：

```bash
doubao-skin migrate-v3 themes/<theme-id> --write --json
doubao-skin check themes/<theme-id> --json
doubao-skin preview themes/<theme-id> --json
doubao-skin pack themes/<theme-id> dist/<theme-id>.doubao-skin.zip --json
```

迁移会把通用视觉字段移到 `shared`，把来源整理到 `provenance`，将主题版本提升到下一个主版本，并显式声明三个现有应用。旧 `theme.css` 中符合 v3 白名单的视觉规则会重写作用域并保存在 `styles/doubao-family.css`，由豆包和豆包工作共同引用；布局、交互、远程资源和已经由结构字段承担的规则不会带入。若共享层声明了图标，迁移器还会为当前不支持图标替换的 WorkBuddy 写入 `icons: null`。随后必须人工复核：

- 确认被过滤的旧 CSS 确实只包含已由结构字段/宿主适配器承担或 v3 明确禁止的能力；
- 确认保留的豆包家族 CSS 在浅色和深色窗口中没有可读性回归；
- 用 `null` 删除不适用于目标的继承资源；
- 在三个真实应用的浅色/深色窗口分别验收后再发布。

## 常见错误

下面的写法不会“尽量加载”，而是直接失败：

```json
{ "targets": { "unknown-app": {} } }
```

错误：未知目标 ID。

```json
{ "shared": { "css": ["../outside.css"] } }
```

错误：路径穿越主题目录。

```css
.workbench-part { background: red; }
```

错误：选择器没有主题和目标作用域。

```css
html[data-skin="my-theme"] .panel { display: none; }
```

错误：主题 CSS 不得改变内容或交互结构。

校验错误会指出 JSON 指针、目标、外观、文件及可用的行列位置。不要通过删除目标声明或放宽 CSS 安全边界绕过错误。
