---
id: "2026-09-01-beautiful-dmg-installer"
stage: spec
status: accepted
owner: "codex"
created: "2026-09-01"
based_on: intent.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-09-01"
---

# Spec: 精致的 macOS DMG 安装界面

## Requirements

1. `./scripts/package.sh desktop-macos` 的 host 与 `--universal` 模式必须继续输出当前 ZIP、DMG 和各自 SHA-256；文件名、架构标签、签名、公证条件和 ZIP 内容保持兼容。
2. DMG 打开后必须显示一个 660×428 pt 的 Finder 窗口，内容背景画布为 660×400 pt；窗口使用图标视图，隐藏工具栏、路径栏、侧栏和状态栏，不允许自由排列覆盖固定坐标。
3. `豆皮.app` 位于左侧、`Applications -> /Applications` 位于右侧，图标中心分别约为 `(170, 220)` 与 `(490, 220)`，图标尺寸约 120 pt；两者之间必须有明确的由左向右拖拽箭头，并为 Finder 文件名保留不重叠的标签空间。
4. 背景必须复用当前 AppIcon 的暖白/柔金色视觉语言，使用克制的浅色渐变、简短中文提示“拖到 Applications 完成安装”和充足留白；不得使用第三方图片、官方豆包资源、复杂插画、品牌水印或仅靠颜色表达动作。
5. 背景源资产必须同时提供 660×400 的 1× PNG 与 1320×800 的 2× PNG。打包时使用系统 `tiffutil` 生成多分辨率 TIFF，使普通与 Retina 显示缩放下的文字、曲线和渐变保持清晰。
6. 最终镜像的可见根目录只能有 `豆皮.app` 与 `Applications`；背景 TIFF 放在隐藏的 `.background` 目录，Finder 布局写入 `.DS_Store`，不得暴露构建脚本、说明日志或中间资源。
7. 打包流程必须先生成可写临时镜像并挂载，在该卷中设置 Finder 布局和图标位置，确认 `.DS_Store` 与背景存在后可靠卸载，再转换为压缩只读 UDZO 镜像。最终文件只有在 `hdiutil verify` 成功后才可命名并生成校验和。
8. AppleScript/Finder、挂载、卸载、转换、验证或可选公证的任何一步失败，都必须让打包失败并清理准确的临时挂载点、可写镜像和压缩临时镜像；不得静默退回未设计的普通 DMG。
9. 现有 app 公证顺序保持不变；有完整 Apple 凭据时，转换后的最终结构 DMG 继续执行 notarytool、staple、validate 与再次 `hdiutil verify`。无凭据时继续支持现有本地签名路径，不宣称 Apple 已公证。
10. 本变更必须记录 shell/工作流门禁、host 与 universal 实际构建、只读挂载内容、软链接、签名、版本/架构、ZIP/DMG app 一致性、Finder 实窗截图、清理状态和残余风险。

## User experience

用户双击 DMG 后直接进入一个紧凑、安静的 Finder 窗口。顶部只提供一句安装提示；左侧是现有豆皮 AppIcon 和 `豆皮` 文件名，右侧是系统 Applications 文件夹与文件名，中间箭头建立唯一的操作方向。用户拖动应用后由 Finder 提供原生拖拽、复制进度、同名冲突和权限反馈。

本界面遵循 macOS 熟悉的拖放模型，不发明新的安装控件；具体参考 Apple HIG 的 [拖放](https://developer.apple.com/cn/design/human-interface-guidelines/drag-and-drop/)、[布局](https://developer.apple.com/cn/design/human-interface-guidelines/layout/)、[颜色](https://developer.apple.com/cn/design/human-interface-guidelines/color/)、[图像](https://developer.apple.com/cn/design/human-interface-guidelines/images/)和[辅助功能](https://developer.apple.com/cn/design/human-interface-guidelines/accessibility/)。其中固定左右布局、暖金品牌背景与具体尺寸是本项目基于 HIG 原则作出的设计选择，并非 Apple 对 DMG 的强制模板。

背景中文字只作辅助说明，不能成为唯一安装线索；VoiceOver 和键盘用户仍可通过两个真实 Finder 项目的名称、类型和标准复制操作完成安装。背景保持高对比度、无闪动、无透明叠字和无依赖颜色区分的状态信息。

## Technical design

- 在 `assets/dmg/` 保存经视觉验收的 1×/2× PNG。背景只包含项目原创的几何形、文字和从现有 AppIcon 提取的暖金配色，不复制 AppIcon 主体到背景，以免与真实应用图标争夺层级。
- `scripts/package/macos.sh` 继续只组装一次 `BUNDLE`。ZIP 完成后，新的 DMG 段建立唯一 staging；复制同一 app、创建 `/Applications` 软链接、准备隐藏背景目录并用 `tiffutil -cathidpicheck` 生成背景 TIFF。
- 使用 `hdiutil create -format UDRW` 生成可写临时镜像，并挂载到 `mktemp -d` 产生的明确路径。AppleScript 通过 POSIX 挂载路径取得 Finder 文件夹窗口，不依赖卷名查找，因此同名卷不会改变目标。
- AppleScript 设置 `icon view`、窗口 bounds、隐藏 chrome、背景图片、约 120 pt 图标尺寸与两个项目坐标；请求 Finder `update` 后关闭窗口并等待元数据落盘。脚本返回后必须验证卷内 `.DS_Store`、`.background/install-background.tiff`、app 和软链接。
- 通过 `hdiutil detach` 卸载并 `hdiutil convert -format UDZO` 生成压缩临时镜像；验证、公证与最终原子命名沿用当前 fail-closed 逻辑。trap 同时跟踪挂载状态、挂载目录、staging、可写临时镜像和压缩临时镜像。
- 不提交构建出的 `.dmg`、`.DS_Store` 或 TIFF；仓库只提交两张背景 PNG、打包脚本和变更工件。文档只有在现有“DMG 包含 app 与 Applications”描述因实现需要变化时才修改。

## Security and privacy

- 不读取 Finder 之外的用户窗口、文件或账户数据，不申请自动化控制其他应用。AppleScript 只操作本次临时挂载目录的 Finder 窗口。
- 不自动写入 `/Applications`，不覆盖已安装的 `豆皮.app`，不修改任何官方豆包应用；验证复制使用临时隔离目录。
- 不引入网络请求、第三方二进制、下载内容、凭据或新签名材料。背景资产不得含真实用户数据、第三方商标或不明授权内容。
- 精确引用所有清理目标，不对宽泛目录、`/Applications`、工作区根目录或未解析变量执行递归删除。
- 签名与公证边界保持现状：自签名或 ad-hoc 成功不等于 Apple 信任或公证，验证记录必须继续明确这一点。

## Alternatives and non-goals

- 不采用 PKG/DMG 安装向导，因为当前 App 只需拖拽复制，向导会增加签名、公证、卸载和维护面。
- 不引入 `create-dmg`、`dmgbuild` 或 Node/Python 打包依赖；现有系统工具已能完成镜像、Finder 元数据和多分辨率背景。
- 不提交预生成 `.DS_Store` 模板；它是难审查的二进制且容易和文件名、背景路径或布局漂移，构建时由 Finder 生成更直接。
- 不增加深色背景、多语言背景、卷图标、许可证弹窗、动画、音效、README 文件或网站展示重设计。
- 不更改 app AppIcon、应用 UI、主题、版本、Release 工作流资产列表或下载 URL。

## Areas of concern

- GitHub `macos-26` runner 上 Finder/AppleScript 的可用性必须通过真实打包验证；若自动化权限失败，不能以跳过布局继续发布，而应缩小探针并修复当前原生流程或回到规格评审。
- Finder 将窗口元数据异步写入 `.DS_Store`。脚本必须以明确更新、短暂等待和文件存在检查降低竞态，并在实际重新挂载后确认布局，而不是只验证 AppleScript 退出码。
- Finder 窗口 bounds 包含标题栏而背景对应内容区域；实施时允许在不改变 660 pt 宽度、左右关系和信息层级的前提下，根据实窗截图微调高度与坐标，并把最终值同步回 plan/verification。
- 当前 app 最低支持 macOS 12，但本机/CI 只直接生成与验证当前 Finder 元数据。若老系统实测出现布局兼容问题，应作为发布阻断记录，不在本次增加版本专用分支。
- 多分辨率 TIFF、系统 Applications 图标、AppIcon 与背景色在不同显示缩放下可能产生细微视觉差异；普通与 Retina 实窗截图均需检查清晰度、留白和标签安全区。
- 现有签名可能是社区自签名而非 Developer ID；DMG 美化不得掩盖首次打开警告或被表述为已消除 Gatekeeper 提示。

## Acceptance criteria

- 新 DMG 通过 `hdiutil verify`，只读重新挂载后存在同一个已签名的 `豆皮.app`、有效的 `Applications -> /Applications`、隐藏背景和 `.DS_Store`，且没有额外可见项目。
- Finder 从最终只读 DMG 打开时恢复 660×428 左右的固定窗口、浅色品牌背景、约 120 pt 两个图标与清楚的左到右箭头；无工具栏/侧栏/状态栏，无裁切、重叠、标签截断或意外滚动区域。
- 1× 与 2× 背景尺寸、格式和视觉内容正确，多分辨率 TIFF 生成成功；正常与 Retina 显示缩放的截图中文字及曲线清晰。
- host 与 `--universal` 打包都成功，ZIP 与 DMG 内 app 的版本、主可执行文件哈希、签名和目标架构一致；两份 checksum 通过。
- 将镜像内 app 复制到临时隔离的 `Applications` 目录后，bundle 完整且严格 codesign 仍通过；测试不写入真实 `/Applications`。
- 至少一次受控失败验证 trap 能卸载临时卷并移除 staging、挂载目录、可写镜像、压缩临时镜像和错误 checksum；`hdiutil info` 不出现本变更遗留挂载。
- `bash -n scripts/package/macos.sh`、`./scripts/check.sh workflow`、适用的 macOS/完整仓库门禁及 `git diff --check` 通过，验证证据写入 `verification.md`。
- 新鲜上下文验证者或人工依据最终 DMG 与截图给出 verdict；实现者不能仅凭源代码或 staging 目录宣称视觉完成。

## Decision

等待产品负责人确认本规格后进入实现计划；确认只授权本地设计资产、打包脚本、必要测试/文档和验证工件，不授权发布、上传 Release、修改真实 `/Applications` 或使用生产凭据。
