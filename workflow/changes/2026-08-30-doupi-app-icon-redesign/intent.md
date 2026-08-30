---
id: "2026-08-30-doupi-app-icon-redesign"
stage: intent
status: accepted
owner: "codex"
created: "2026-08-30"
source: "user"
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-30"
---

# Intent: 以“豆皮”为主题重做应用图标

## Problem

当前正式 AppIcon 虽然已经接入 Icon Composer，但主视觉仍是蓝色环形标记叠加紫色工作台小图标，更像两个既有产品标识的组合；它没有直接表达应用已经更名为“豆皮”，小尺寸下右下角工作台细节也容易糊成色块。现有浅色与深色成品主要依赖底色切换，主体仍是同一套扁平合成素材，Icon Composer 的分层材质、Default/Dark/Mono 外观能力没有真正服务于品牌含义。

## Proposed outcome

以“豆皮 / 腐竹薄片”为唯一核心意象，创作一套原创的 macOS 应用图标：一张温润的金黄色豆皮被轻轻卷起并折成简洁的环形/字母 `d` 轮廓，辅以一枚小黄豆作为识别点；整体像真实豆制品的薄、柔、透，又保持工具类应用应有的清爽和现代感。Default、Dark、Mono 三种外观使用同一个语义分层结构，在真实 Icon Composer 2.0 中分别调校背景、材质、反差和单色表现，并继续通过仓库现有 `actool` 路径编译到 App、Finder 和 Dock。

## Affected users and systems

- 使用“豆皮”macOS 桌面应用的用户，以及在 Finder、Dock、应用切换器和关于面板中看到 AppIcon 的用户。
- `assets/app-icon/AppIcon.icon` 及其可编辑分层素材。
- 由该源文件派生的 Default/Dark 预览 PNG、`AppIcon.icns`、`Assets.car`、iconset 与 xcassets 回退资源。
- `scripts/build-macos.sh` 的现有 Icon Composer/`actool` 编译链路和最终应用包。

## Constraints

- 必须使用本机 `/Applications/Xcode-beta.app/Contents/Applications/Icon Composer.app`（当前版本 2.0）实际打开、编辑并保存 `.icon`；不能只手写 `icon.json` 或提交一张扁平 PNG 冒充分层图标。
- `.icon` 是唯一可编辑源；PNG、`.icns`、`Assets.car`、iconset 和 xcassets 只能由它派生，不能形成第二套竞争源。
- 主体使用原创的豆皮薄片和黄豆意象，不复制豆包、豆包工作或其他第三方官方标记，不包含文字、品牌字样、水印或来源不清的素材。
- 源素材保持方形、无预烘焙圆角底板；豆皮薄片、黄豆识别点和背景必须按效果需求拆成真实语义层，让 Icon Composer 负责平台遮罩、阴影、折射、高光和外观注释。
- 视觉以暖豆黄、奶白和少量焦糖阴影为主；Dark 外观保持温暖而不发灰，Mono 在小尺寸下仍能看出“卷起的薄片 + 黄豆点”轮廓。
- 沿用当前约 78% 的安全尺度作为起点，以真实 Dock/Finder 视觉重量为准；不得为截图方便改动 Dock 放大、系统外观或其他系统设置。
- 继续保留非 Xcode 构建机所需的已编译回退资源；不能改变当前应用名称、Bundle ID、签名、发布流程或主题资源。
- 只在仓库和构建产物中工作，不修改 `/Applications/Doubao.app` 或 `/Applications/DoubaoWork.app`。
- 保存所有与本需求无关的工作区改动。

## Out of scope

- 重命名应用、调整窗口 UI、网站 favicon、主题内图标或任何豆包/豆包工作官方应用资源。
- 新增图标切换设置、多套可选品牌皮肤或运行时 Dock 图标覆盖。
- 更改打包架构、签名、公证、Release 或下载地址。
- 为兼容性重写现有图标构建管线；只有发现实证缺陷时才做最小修复。

## Success signals

- `AppIcon.icon` 能被 Icon Composer 2.0 正常打开，包含可独立编辑的背景、豆皮薄片和黄豆识别层，并通过 `inspect-icon.sh` 检查。
- Icon Composer 中的 macOS Default、Dark、Mono 预览均无裁切、重影、双重阴影或糊边；16 px、32 px、64 px 预览仍能识别核心轮廓。
- 现有打包命令从该 `.icon` 成功生成 `Assets.car` 和 `AppIcon.icns`，构建出的实际 App 指向新图标资源。
- 同一构建产物在 Finder、Dock、应用切换器和关于面板显示新图标；正常与窄窗口下应用本身无图标相关回归。
- Default 与 Dark 使用相同语义构图但具备足够对比，Mono 不依赖颜色才能辨认；与相邻系统 App 图标相比视觉重量均衡。
- 生成源、Icon Composer 版本、编译命令、检查结果、真实界面截图和残余风险记录到 `verification.md`，最终 verdict 由新上下文 verifier 或人工确认。

## Open questions

无。用户所说的“一组 icon”按同一 AppIcon 的 Default、Dark、Mono 外观组理解，不扩展为多套可选品牌方案；“豆皮”按可食用豆制品薄片/腐竹意象理解，不沿用现有蓝色环形标记或紫色工作台小图标。

## Decision

等待产品负责人确认本意图后再进入规格设计；本阶段不修改 AppIcon 源、生成图像、编译回退资源或应用代码。
