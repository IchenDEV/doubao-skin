# doubao-work-skin

**给「豆包工作」macOS 桌面端换个皮肤。**

外部主题 / 换肤工具 · 不修改官方安装包 · 灵感来自 [Codex Dream Skin](https://github.com/Fei-Away/Codex-Dream-Skin)

![violet-night](docs/screenshot.png)

> 非字节跳动官方产品。原版 `/Applications/DoubaoWork.app` 不会被做任何修改。

## 效果

| 主题 | 说明 |
| --- | --- |
| `violet-night` 暗夜紫 | 深紫罗兰底 + 紫罗兰强调色（默认，实测验证） |
| `ocean-cyan` 海洋青 | 石墨蓝底 + 青色强调色 |
| `forest` 墨绿 | 深墨绿底 + 翠绿强调色 |
| `pure-dark` 纯暗 | 只强制深色模式，不改颜色 |

换肤范围：主聊天窗口、启动器、侧边面板等全部内嵌页面（约 20 个）。

## 使用

需要 macOS、已安装的「豆包工作」、Python 3（无第三方依赖）。

```bash
python3 -m doubao_skin list                 # 列出主题
python3 -m doubao_skin apply violet-night   # 构建皮肤版应用
python3 -m doubao_skin remove               # 删除皮肤版应用
```

`apply` 会在 `~/Applications/` 生成 **`DoubaoWork-Skin.app`**，直接打开即可。原版应用不受影响，随时共存。

首次启动时 macOS 会请求钥匙串访问（“DoubaoWork Safe Storage”）：输入开机密码并选择 **始终允许**。这是重签名改变应用身份所致；每次重新构建（换主题）会再弹一次。

## 原理

1. **克隆而非修改**：`/Applications` 里的原版受 macOS App Management（MACL）保护，属主也无法写入。用 APFS clonefile 克隆到 `~/Applications`，瞬间完成、不占额外磁盘。
2. **主界面在 resources.pak 里**：主聊天 UI 不是散文件，而是 gzip 压缩后打进 Chromium 的 `resources.pak`。本工具内置了一个极简 pak v5 解析/重建器（`doubao_skin/pak.py`），把主题 CSS 注入包内全部页面；磁盘上的本地入口 HTML（侧边面板等）也一并注入。
3. **主题 = CSS 变量覆写**：应用自身已有完整深色主题和设计令牌体系（`--N*` 中性色板、`--B*` 品牌色板、`--s-color-*`、`--dbx-*` 表层令牌）。皮肤通过强制 `data-theme="dark"` + 高优先级选择器覆写这些令牌实现换肤，不碰组件样式。
4. **ad-hoc 重签名**：改资源后签名失效会被 Gatekeeper 判“已损坏”，重新 ad-hoc 签名后即可运行。

## 自定义主题

复制一个内置主题目录（或新建），改两个文件：

```
themes/my-theme/
├── theme.json   {"id": "my-theme", "name": "我的主题", "description": "..."}
├── theme.css    CSS 覆写规则
└── icon.icns    （可选）自定义应用图标
```

`theme.css` 会被注入到所有内嵌页面。注意两点（都是踩过的坑）：

- 用 `html[data-skin][data-theme=dark], html[data-skin][data-theme=dark] body` 作为选择器。应用把部分令牌直接定义在 `html, body` 上，只覆写 `html` 会被 body 自己的规则压过。
- 很多令牌有 `-raw` 孪生变量（`rgba(var(--x-raw), .5)` 用），改色时两个都要覆写。

然后 `python3 -m doubao_skin apply my-theme`（或传主题目录路径）。

## 已知限制

- 原版应用升级后需重新 `apply`（皮肤版不会自动跟进）。
- ad-hoc 签名的 cdhash 每次构建都变，所以钥匙串授权每次重建要重新点一次。
- 主题只覆盖深色模式（皮肤本身强制 dark）。

## 许可

[MIT](LICENSE)
