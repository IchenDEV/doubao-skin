---
id: "2026-08-30-doupi-app-icon-redesign"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-30"
based_on: intent.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-30"
---

# Spec: 豆皮主题 AppIcon 分层设计与编译

## Requirements

1. `assets/app-icon/AppIcon.icon` 必须继续作为唯一可编辑 AppIcon 源，并由本机 Icon Composer 2.0 实际打开、编辑和保存；不得用手写 `icon.json` 代替 UI 编辑记录。
2. 新图标必须完全移除现有蓝色环形标记和紫色工作台小图标，改用原创的“多张不规则豆皮薄片叠放 + 局部卷边露出层理 + 小黄豆”构图，不出现文字、官方豆包标记、第三方品牌、水印或来源不明素材。
3. 构图必须至少包含三个可独立调校的语义层：后侧豆皮薄片、前侧折叠豆皮、小黄豆识别点；背景使用 Icon Composer 的 fill，不把圆角底板画进源素材。
4. 豆皮主体必须首先读成纸张般薄、不规则且可弯折的豆制薄片，而不是字母、圆环、吐司、面包或皮包。识别依靠错位叠片、数条可见薄边、局部卷角和少量宽幅褶皱/气孔；小尺寸可减少纹理，但必须保留叠片与薄边轮廓。
5. Default 外观使用奶白至浅豆浆色背景、暖豆黄主体和少量焦糖深度；Dark 外观使用深烘豆棕背景并保持主体温暖明亮；Mono 外观必须在不依赖色相时仍能分辨豆皮卷曲轮廓和黄豆点。
6. 每个导入层必须是 1024×1024 方形、无平台遮罩的 SVG 或无损 PNG。素材可以包含表达豆皮本体所需的宽褶皱、气孔和薄边，但投影、折射、镜面高光、整体透明度和平台圆角优先由 Icon Composer 配置，不在素材中重复烘焙。
7. 豆皮与黄豆组合的最终视觉主体应大致落在画布 68%–80% 范围内；各边保留安全区，小尺寸下不得裁切、贴边或因过小而只剩无法辨识的黄色色块。
8. Icon Composer 文件必须提供 macOS Default、Dark、Mono 三种可用外观注释；三种模式共享同一语义构图，不各自维护三套完全独立的扁平图片。
9. `scripts/build-macos.sh` 的消费路径保持不变：优先用 Xcode `actool` 从 `AppIcon.icon` 生成 `Assets.car` 与 `AppIcon.icns`，无兼容工具时继续使用仓库内已编译回退资源。
10. 所有已跟踪的图标派生物必须和新 `.icon` 同步，包括 `AppIcon.icns`、`Assets.car`、Default/Dark 预览 PNG、iconset 与 xcassets；不得保留仍展示旧蓝紫图标的回退文件。
11. 非 bundle 的 `cargo run` 开发模式继续通过嵌入的 `AppIcon.icns` 显示新图标；打包后的 `.app` 必须继续由 Info.plist 的 `CFBundleIconFile`/`CFBundleIconName` 与 `Assets.car` 驱动，不新增运行时覆盖。
12. 只有从同一份新 `.icon` 编译出的实际 App 通过 Finder、Dock、应用切换器和关于面板验证后，才能把图标实现判为视觉通过。

## User experience

- 用户在浅色系统外观下看到温暖、干净的豆浆底色，中央是数张错位叠放、带薄边和局部卷角的金黄豆皮，小黄豆只作为来源提示，不呈现卡通表情或餐饮菜单照片感。
- 深色外观将背景压到烘豆棕而不是纯黑；主体边缘仍有清楚层次，不出现灰雾、黑边、重复高光或与背景融成一块。
- 单色外观保留清楚的错位叠片、薄边与折叠关系，小黄豆点不能消失成噪点。
- 在 16 px、32 px、64 px 和常见 Dock 尺寸上，用户首先读到数张柔软薄片及一处卷角，其次通过暖黄色、气孔和黄豆感知“豆皮”；不能先读成字母、吐司或无意义黄色形状。
- 图标与相邻系统 App 的视觉重量接近，Dock 未悬停和未放大时不显得过大或过小。

## Technical design

### Canonical source and layer model

- 在 Icon Composer 中打开现有 `assets/app-icon/AppIcon.icon`，用新的语义素材替换旧层并立即保存到同一路径，不新建第二个正式 `.icon`。
- 素材在导入前使用确定性的 1024×1024 SVG 构造；导入成功后，Icon Composer 包内 `Assets/` 的副本成为正式可编辑源。用于中转导入的临时文件不作为另一套正式源保留。
- 建议层级从后到前为：`Yuba Back Sheet`、`Yuba Front Fold`、`Soybean Accent`。前后豆皮分组分别承担折叠遮挡和材质深度；黄豆单独分组，以便使用较轻的阴影与更低的折射。
- 豆皮 SVG 使用错位的不规则片形、少量平滑渐变、数条可见薄边和宽幅褶皱/气孔表达材质；避免照片纹理、随机噪声、厚面包边、描边式阴影和已烘焙玻璃模糊。黄豆为小型饱满椭圆，位置只负责说明原料，不形成角标或主轮廓。

### Appearance annotations

- Default fill 使用暖奶白/浅豆浆色；豆皮后层偏浅、前层偏金黄，Icon Composer 负责中性阴影、轻微 specular 和克制的折射/半透明。
- Dark fill 使用深烘豆棕；主体保持相同几何结构，仅通过 Icon Composer 的 Dark 注释提高边缘反差并压低过亮高光，不把一张独立深色 PNG盖在背景上。
- Mono 使用 Icon Composer 的单色注释或 automatic 结果作为起点；若黄豆点或折叠负空间丢失，只调整分组的单色明度/透明度/可见性，不修改 Default 几何。
- 在保存前遍历工具当前提供的 macOS 平台、Default/Dark/Mono 与所有设计 generation 预览。若 Icon Composer 2.0 的字段名称与当前文档不同，以真实 UI 暴露能力为准并在 verification 记录偏差。

### Derived artifacts and build integration

- Icon Composer 保存后先运行 `inspect-icon.sh` 检查包类型、JSON 可读性、分组、层名和素材存在性。
- 使用 `/Applications/Xcode-beta.app/Contents/Developer` 的 `actool` 编译到隔离目录，确认同时得到 `AppIcon.icns` 与 `Assets.car`；随后从同一编译结果更新仓库回退文件。
- Default/Dark 预览由 Icon Composer 的导出能力或同一 `.icon` 的 Apple 工具链导出；iconset 由新的 `.icns` 派生，xcassets 只复制对应尺寸，不反向作为输入重建 `.icon`。
- 构建脚本、Info.plist、Bundle ID、签名和部署目标没有实证问题时不修改。开发模式嵌入的 `.icns` 因回退资源更新而自然同步。

### Validation

- 静态检查：`inspect-icon.sh`、`file`/`plutil`、各 iconset 画布与 alpha footprint、`git diff --check`。
- 编译检查：现有 host 打包命令必须从 `.icon` 成功生成自适应资源，且 bundle 内文件时间和内容哈希与本次输出一致。
- 真实表面检查：从新构建 bundle 启动应用，核对 PID/可执行路径；在同一显示比例、Dock 大小、外观和悬停状态下捕获 Finder、Dock、应用切换器与关于面板证据。
- 真实窗口回归：应用正常宽度和窄宽度各检查一次，确保只有系统图标表面变化，窗口 UI、菜单和启动行为不受影响。

## Security and privacy

- 所有创作素材在本地生成，不上传聊天内容、账户数据、Cookie、钥匙串、签名材料、应用资源或屏幕中的私人内容。
- 不读取、复制或修改 `/Applications/Doubao.app`、`/Applications/DoubaoWork.app` 或其他官方应用资源；新标记只使用原创几何和通用豆制品意象。
- Icon Composer、Xcode `actool` 和系统自带图标工具是唯一外部编译工具，不新增依赖或下载来源不明的生成器。
- 真实界面截图只包含“豆皮”应用与必要系统表面，不展示通知、聊天正文、文件名或其他无关个人信息。

## Alternatives and non-goals

- 不保留现有蓝紫标记作为角标、底纹或过渡兼容层；这会继续造成品牌含义不清。
- 不使用写实豆皮照片或 AI 生成的单张扁平位图作为正式源；它们在小尺寸和 Mono 外观下不可控，也无法满足真实语义分层。
- 不仅替换 `AppIcon.icns`；那会让 `Assets.car`、Icon Composer 源和开发模式输出相互漂移。
- 不直接编辑 `.icon/icon.json` 来伪造 UI 操作；必要的最终效果必须在 Icon Composer 中保存。
- 不新增运行时 `setApplicationIconImage` 覆盖来掩盖 bundle 或缓存问题，也不通过重置全局 LaunchServices/Dock 缓存制造通过假象。
- 不扩展为网站 favicon、主题包图标或多套用户可选图标。

## Areas of concern

- Icon Composer 2.0 来自 Xcode beta，文件格式和材质默认值可能与早期版本不同；必须以当前机器真实打开、保存和重新打开为准。
- 当前最低部署目标为 macOS 12.0，而新自适应外观依赖更新系统能力；`actool` 仍必须生成向后兼容的 `.icns` 回退，旧系统不能只剩 `Assets.car`。
- 暖黄食品意象容易变成吐司、可丽饼或餐饮 App；必须用纸张般的薄边、错位叠片、不规则边缘、气孔和黄豆来源提示消除歧义，同时保持克制的工具 App 质感。
- Mono 会消除豆皮渐变带来的前后关系；叠片错位、薄边和卷角必须先在几何上成立。
- Finder 和 Dock 可能展示缓存图标；出现旧图标时先核对运行 bundle、资源哈希和完整退出/重启，不把缓存状态误判为源文件失败。
- 当前仓库保存多种派生格式；任何一个未更新都会让无 Xcode 构建机或开发运行继续显示旧图标。

## Acceptance criteria

- `assets/app-icon/AppIcon.icon` 的 UTI 仍为 `com.apple.iconcomposer.icon`，由 Icon Composer 2.0 重新打开后可见 `Yuba Back Sheet`、`Yuba Front Fold`、`Soybean Accent` 三个真实语义层/组。
- `inspect-icon.sh assets/app-icon/AppIcon.icon` 通过，全部引用素材存在；没有旧的 `01-doubao-work-mark.png` 或 `02-skin-fold.svg` 引用。
- Icon Composer macOS Default、Dark、Mono 预览在 16/32/64 px 均可识别为叠放豆皮薄片而非吐司/字母/无意义黄色形状、无裁切，并保存至少一张无敏感内容的三模式对照证据。
- `actool` 从 canonical `.icon` 成功生成非空 `AppIcon.icns` 与 `Assets.car`，打包日志明确走 `Compiled adaptive app icon from AppIcon.icon` 而非 fallback。
- 仓库内所有正式图标派生物都不再显示旧蓝色环/紫色工作台；`.icns` 各 representation 画布正确，alpha footprint 与真实 Dock 视觉重量匹配。
- 构建后的 App 的 Info.plist 仍声明 `CFBundleIconFile=AppIcon`、`CFBundleIconName=AppIcon`，资源哈希与本次编译结果一致，代码签名严格验证通过。
- 从该 App 启动的 PID 指向预期 bundle；Finder、Dock、应用切换器、关于面板都显示新图标，正常/窄窗口启动与交互无回归。
- `verification.md` 分开记录 Icon Composer UI 保存、源检查、编译、真实表面视觉验收与残余风险；最终 verdict 由新上下文 verifier 或人工给出。

## Decision

本规格已根据 2026-08-30 的真实预览反馈重写视觉识别要求，等待产品负责人重新确认；确认前只允许保留当前 Icon Composer 原型与证据，不编译或覆盖正式派生资源，也不启动系统表面验收。
