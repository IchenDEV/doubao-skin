---
id: "2026-09-01-beautiful-dmg-installer"
stage: intent
status: accepted
owner: "codex"
created: "2026-09-01"
source: "user"
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-09-01"
---

# Intent: 制作精致的 macOS DMG 安装界面

## Problem

当前 DMG 虽然可以安装，但挂载后只是系统默认文件列表：应用与 `Applications` 软链接没有明确的视觉关系，也没有品牌背景、固定窗口尺寸或经过设计的图标位置。用户需要自行理解安装动作，整体观感与已经完成的“豆皮”应用图标和正式发布包不匹配。

## Proposed outcome

把现有 DMG 升级为简洁、原生且有品牌辨识度的拖拽安装界面：打开后直接看到左侧“豆皮.app”、右侧系统 `Applications` 文件夹，以及清楚的拖拽方向；使用与现有暖金色豆皮图标协调的浅色背景、克制文案和充足留白。继续复用同一个已经签名的应用包，并由现有 macOS 打包命令自动产出可校验的最终 DMG。

## Affected users and systems

- 下载 macOS DMG 并通过 Finder 安装“豆皮”的用户。
- `scripts/package/macos.sh` 的 DMG staging、挂载、Finder 布局、压缩与清理流程。
- 新增的 DMG 背景视觉资产，以及打包/发布验证文档中与镜像内容相关的说明。
- GitHub Release 后续生成的 host/universal DMG；本变更本身不发布或替换任何 Release。

## Constraints

- 保留用户熟悉的“将 App 拖到 Applications”安装方式，卷根目录只显示应用和系统 Applications 入口。
- 复用现有 `豆皮.app`、AppIcon、签名、公证、架构标签、校验和与文件名，不重新构建第二份应用。
- 使用 macOS 自带的 `hdiutil`、Finder/AppleScript 与现有命令，不新增 `create-dmg`、Node 包、Homebrew 工具或其他第三方打包依赖。
- 视觉采用中性暖白底、豆皮图标的柔和金色点缀、清楚的单向箭头与简短中文提示；不能依赖低对比度、复杂插画或过多装饰传达安装动作。
- Finder 窗口使用固定图标视图、经过验证的窗口尺寸和图标坐标，隐藏不必要的工具栏、状态栏和隐藏资源目录。
- 临时可写镜像、挂载点与 Finder 进程交互必须可清理；失败时不得留下已命名为成功产物的 DMG、校验和或占用中的卷。
- 不修改 `/Applications/DoubaoWork.app`、`/Applications/Doubao.app` 或用户的 `/Applications` 内容；镜像中仍然只是软链接。
- 保存当前工作区中与本需求无关的改动，不跨越生产发布审批门。

## Out of scope

- 不改“豆皮”应用本身的窗口、功能、Dock/Finder AppIcon 或主题视觉。
- 不增加 PKG 安装器、安装向导、许可证弹窗、自动复制、首次启动流程或卸载器。
- 不增加自定义卷图标、音效、动画、多语言背景或深浅两套 DMG 外观。
- 不更改 macOS 签名身份、公证凭据、Release 版本、GitHub Release 资产或网站下载链接。

## Success signals

- 挂载新 DMG 时，Finder 以设计好的固定窗口打开；左侧应用和右侧 Applications 文件夹清晰可辨，拖拽方向无需额外说明即可理解。
- 背景、标题、箭头、应用图标和文件名在普通与 Retina Mac 上均清晰，无裁切、重叠、标签截断或隐藏文件暴露。
- 实际将应用拖入一个隔离的测试 Applications 目录能够完成复制，镜像内 `Applications` 仍解析为 `/Applications`。
- `hdiutil verify`、只读挂载、内容清单、软链接、严格 codesign、架构/版本与 SHA-256 检查全部通过；镜像内应用与 ZIP 中的同一构建保持一致。
- host 模式与 `--universal` 模式都能生成布局一致的 DMG，失败路径不会残留临时卷或半成品。
- Finder 实窗截图在正常显示缩放下通过人工视觉验收，验证证据和残余风险记录在本变更的 `verification.md`。

## Open questions

无。默认以当前“豆皮”暖白/金色 AppIcon 为品牌基准，采用单一浅色 DMG 背景；若后续明确需要多语言或深色背景，应另开变更，不扩大本次打包范围。

## Decision

等待产品负责人确认本意图后进入规格设计；当前阶段不修改打包脚本、视觉资产或发布流程。
