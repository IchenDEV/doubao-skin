# 提交新主题

豆皮库通过 GitHub Pull Request 接收主题，不在网页上直接上传文件。每个主题必须是可以由桌面应用和 `doubao-skin` CLI 真实加载的独立目录。

## 1. 准备仓库和分支

Fork `IchenDEV/doubao-skin`，从最新主分支创建主题分支。主题目录使用小写 kebab-case，例如 `themes/morning-mist/`。

## 2. 创建主题

推荐先安装项目插件。Codex：

```bash
codex plugin marketplace add IchenDEV/doubao-skin
codex plugin add doubao-skin@doubao-skin
```

Claude Code（在 Claude Code 中输入）：

```text
/plugin marketplace add IchenDEV/doubao-skin
/plugin install doubao-skin@doubao-skin
```

然后调用 `$create-doubao-theme` 描述颜色、明暗、气质和使用场景。

没有安装插件的用户可以单独安装 CLI。Windows：

```powershell
scoop install https://github.com/IchenDEV/doubao-skin/releases/latest/download/doubao-skin.json
```

macOS / Linux：

```bash
curl -fsSL https://raw.githubusercontent.com/IchenDEV/doubao-skin/main/scripts/install-cli.sh | sh
```

然后直接运行：

```bash
doubao-skin create themes/<theme-id> \
  --name "主题名称" \
  --description "一句自然语言描述" \
  --accent "#3370eb" \
  --appearance both \
  --targets doubao,doubao-work,workbuddy \
  --author "作者名称"
```

源码用户也可以使用 `cargo run -p skin-core --bin doubao-skin --` 替代 `doubao-skin`。

生成器会创建 v3 `theme.json` 和 1200 × 675 `preview.jpg`，不会覆盖非空目录。`targets` 是唯一适用范围声明；结构化视觉已经完整时不需要 `theme.css`。完整规范见 [主题包 v3](../design/theme-standard/theme-v3.md)。

## 3. 检查和预览

```bash
doubao-skin check themes/<theme-id>
doubao-skin preview themes/<theme-id>
```

检查必须通过。请同时实际查看预览；合成预览不能替代在无私人内容的豆包工作窗口中进行的应用验证。

## 4. 素材和许可

- 不要复制豆包或豆包工作的官方资源。
- 背景图、字体和图标必须是原创、明确授权或允许再分发的素材。
- 需要归属说明时，把 `LICENSE`、`LICENSE.md` 或 `LICENSE.txt` 放在主题目录。
- 不要把私人对话、账号、工作区、附件或未公开内容放进主题与预览。
- 打包只会收入 `theme.json`、清单引用的 CSS/素材、`icon.icns` 和许可证文件。

## 5. 同步网站目录

主题检查通过后，从仓库根目录运行：

```bash
corepack pnpm --dir apps/web sync
```

同步命令会根据 `themes/` 中的真实主题源生成网站清单、预览资源和安装包。请把本次主题对应的 `apps/web/data` 与 `apps/web/public/themes` 变更连同主题源一起提交；这些是可重复生成的目录，不要手工修改其中的文件。再次运行同步命令后不应继续产生差异。

## 6. 本地打包验证

使用一个新的输出路径：

```bash
doubao-skin pack themes/<theme-id> dist/<theme-id>.doubao-skin.zip
```

如需安装或应用，请使用 `$apply-doubao-theme`。安装、应用、恢复、离线构建和删除都有副作用，Skill 会在执行前说明准确目标并等待确认。

## 7. Pull Request 内容

Pull Request 请包含：

- 主题名称、ID 和一句设计说明；
- 作者与版本；
- 素材来源和许可证；
- `doubao-skin check` 结果；
- `corepack pnpm --dir apps/web sync` 结果，以及本主题对应的生成文件；
- 预览图；
- 实际应用验证范围，以及未验证的部分。

只提交该主题源、必要许可文件和同步命令为它生成的目录变更，不要夹带其他主题或手工改写生成内容。Pull Request 的仓库检查与人工审核通过后，合并结果会成为下一次网站目录和主题包发布的来源。
