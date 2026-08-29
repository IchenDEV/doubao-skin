---
id: "2026-08-29-dmg-and-whale-theme-icon"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: intent.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Spec: DMG、鲸鱼娘主题图标与标准版豆包透明度一致性

## Requirements

1. `./scripts/build-macos.sh` 的 host 与 `--universal` 模式必须继续生成现有 ZIP，同时新增同架构标签的 `Doubao-Skin-macOS-<label>.dmg` 和独立 `.sha256`。
2. DMG 必须装入构建流程已经完成签名、严格校验及可选公证/装订的同一个“豆包主题.app”，并在卷根目录提供指向 `/Applications` 的软链接；不得重新构建或修改第二份 app。
3. DMG 必须先写入临时路径并通过 `hdiutil verify`，成功后才能成为最终产物。任一步失败时不得留下新 DMG、错误校验和或仍挂载的临时卷。
4. GitHub Release 工作流必须上传并发布 universal ZIP、DMG 及两份校验和；现有 ZIP 文件名和下载链接保持兼容。
5. 如果提供完整的 Apple 公证凭据，最终 app 与 DMG 都必须使用现有 Developer ID 流程公证并完成 stapler 校验；无凭据的本地构建继续支持 ad-hoc 签名和可验证 DMG。
6. 使用内置 ImageGen 生成一枚原创的鲸鱼娘主题主图标：透明背景、正方形高分辨率 PNG、蓝鲸少女特征、轻微馋嘴流口水表情、无文字/品牌/水印，在 `20–52 px` 仍清晰可辨。
7. `gallery-whale-maid` 的 light/dark `icons.main` 必须共同引用新 PNG；不得替换工具 AppIcon、主题背景或其他语义图标。
8. 主题更新后必须通过正式同步命令重建 Web 目录和主题包，不能手工编辑 `apps/web/data` 或 `apps/web/public/themes`。
9. 界面不透明度滑块必须是运行时背景表面 alpha 的最终权威来源。主题内固定的浅/深色 `!important` 声明不得覆盖 `surface_opacity_profile()` 生成的 page/sidebar/layer/input 值。
10. 标准版豆包必须带有明确的运行时目标标记，并只在该目标中消除已证实的重复整窗背景表面；豆包工作的现有 DOM 表面和注入生命周期不得改变。
11. 40% 滑块在标准版豆包中必须得到 page `0.22`、sidebar `0.30`、layer `0.26`、input `0.48`；页面中心不得再由 `0.72/0.82` 固定表面或三个同义整窗背景连续合成。
12. 全部内置主题必须接受结构化审计，找出并处理会抢占运行时透明度的高优先级背景变量；不得只为“馋嘴豆包”写特例。

## User experience

- 用户运行一次打包命令即可同时获得 ZIP 和常见的 DMG 安装介质。打开 DMG 后可把“豆包主题.app”拖入 Applications，不增加许可证页、自定义背景或额外安装器。
- GitHub Release 同时保留 ZIP 和 DMG，已有网站 ZIP 下载地址不失效；偏好 DMG 的用户可在 Release 资产中直接选择。
- “鲸鱼娘”主题在主题列表、主预览和实际豆包侧栏中显示同一枚蓝色系角色图标。图标以头肩或头像构图为主，小滴口水是俏皮食欲感，不呈现哭泣、脏污或夸张成人化表达。
- 用户拖动不透明度滑块时，预览与当前选择的目标应用继续即时更新。标准版豆包不再在相同百分比下明显更暗，切换浅色、深色或自动外观后仍保持同一数值语义。
- 修复不增加“标准版补偿”“额外蒙层”等用户可见设置，也不改变现有 35%–100% 范围和按钮文案。

## Technical design

### DMG 打包

- 继续以 `scripts/build-macos.sh` 为唯一 macOS 组装入口。完成 app 资源复制、签名、`codesign --verify --deep --strict` 及可选 app 公证/装订后，再从该 bundle 生成 ZIP 与 DMG。
- DMG 使用系统自带 `hdiutil`，不增加第三方依赖。脚本创建位于 `dist` 下的唯一临时 staging 目录，把“豆包主题.app”复制到卷根目录并创建 `Applications -> /Applications`，以压缩只读格式生成临时 DMG。
- 用 shell `trap` 统一清理 staging、临时 DMG 和可能的临时挂载点；最终 DMG 只在创建和 `hdiutil verify` 成功后原子移动到 `dist/Doubao-Skin-macOS-$ARCHIVE_LABEL.dmg`。
- 公证凭据完整时，在最终文件定名后提交 DMG、等待成功、执行 `xcrun stapler staple` 与 `xcrun stapler validate`，再计算 SHA-256。部分凭据继续立即失败；无凭据时跳过 DMG 公证但仍校验内部 app 签名和镜像结构。
- `.github/workflows/release.yml` 的 Actions artifact 和 `gh release upload/create` 显式列出 DMG 及其校验和。README、开发文档与发布清单补充双格式，但网站默认 ZIP URL 保持不变。

### 鲸鱼娘图标

- ImageGen 参考项目内已生成的 `themes/gallery-whale-maid/bg.jpg` 来保持浅青、天蓝、深蓝的角色语言，并参考 `themes/doubao-snack-giggle/icons/main-anime.png` 的小尺寸表情可读性；参考仅用于角色和构图方向，不复制另一个角色。
- 最终源文件保存为 `themes/gallery-whale-maid/icons/main.png`，目标为 1024×1024 RGBA PNG。角色占画布约 70%–82%，四边保留安全区，透明像素覆盖背景区域，不烘焙圆角方形底板、投影文字或水印。
- 生成后先检查原始尺寸、alpha 通道和边缘，再制作 52 px 与 20 px 临时缩放对照；如口水、鲸尾发饰或脸部在小尺寸不可辨，继续迭代生成而不是用 CSS 补画。
- `theme.json` 只把两个变体的 `icons.main` 从现有 SVG 改为新 PNG；保留现有主题来源、许可证、背景和其他字段。同步命令负责更新 Web 卡片、包和校验和。

### 透明度与标准版表面

- 保留 `surface_opacity_profile()` 为唯一数值模型。先增加回归测试，证明主题深色规则可把 40% 运行时值覆盖为 0.72/0.82，并锁定修复后最终 CSS 中受控变量由 profile 胜出。
- `surface_opacity_css()` 继续在主题 CSS 之后生成，并把受控背景变量集中声明在同时含 `data-skin` 与 `data-theme` 的根/正文选择器上。对所有主题移除受控背景变量上不必要的固定 `!important`；没有运行时 override 时仍保留主题自身的默认颜色和 alpha。
- 受控集合至少包括 `--N00`、`--N50`、`--N100`、`--N200`、`--s-color-bg-{body,secondary,base,tertiary,quaternary,primary,content-base}`、`--dbx-bg-{base-web,base-2,base-5,body-web,body-white,body-mac}`、`--chat-bg-color`、`--chatarea-bg-color` 和输入框/消息表面变量。
- 将实时入口改为把现有 `TargetApp` 传入主题 bootstrap，并在根节点写入工具自有的 `data-skin-target="doubao"` 或 `"doubao-work"`。恢复默认和 runtime `destroy()` 同时清理该标记；离线 snippet 不猜测目标。
- 在共享的运行时 CSS 中，用 `data-skin-target="doubao"` 限定标准版适配。依据已捕获 DOM，保留一个 page 表面和一个内容表面，把 `#chat-route-main` 与其直接主内容中重复承担相同背景角色的一层设为透明；最终选择器由回归 fixture 和真实 CDP 元数据共同确认，不能依赖散列 class 名。
- 新增不读取页面文本的 CDP 验收探针，只返回 `data-skin`、`data-theme`、`data-skin-target`、受控 CSS 变量、元素 ID/标签、viewport coverage、背景 alpha、层级及注入节点数量。探针在 40% 时把标准版 page/sidebar 实际值不等于 0.22/0.30 视为失败，并限制中心点连续整窗着色层数量。
- 全主题测试遍历 `themes/*/theme.css`，拒绝受控变量继续以可能抢占运行时值的高优先级固定 alpha 出现；修改到的主题通过正式 Web 同步生成目录。

## Security and privacy

- 打包只处理仓库资源、构建产物和临时 staging，不访问或修改用户 Applications 目录；DMG 中的 Applications 仅是软链接。
- Apple ID、团队 ID、app 专用密码和签名证书继续只来自环境变量/GitHub Secrets，不写入命令输出、DMG、校验和、仓库或 verification。
- ImageGen 输入只使用仓库内已声明为原创生成的视觉资产；最终图标不得包含第三方品牌、官方豆包资源、文字、水印或来源不明素材。
- CDP 继续仅绑定 loopback。透明度探针和运行时适配不读取聊天正文、消息节点文本、Cookie、请求头、账号、附件、工作区数据或 local/session storage。
- 目标标记只能由代码内 `TargetApp` 常量产生，主题包和页面内容不能提供任意选择器或脚本参数。
- 不修改 `/Applications/Doubao.app` 或 `/Applications/DoubaoWork.app`；真实验证使用现有实时注入与无敏感内容页面。

## Alternatives and non-goals

- 不用第三方 DMG 制作器，也不增加自定义背景、图标坐标、Finder AppleScript、许可证弹窗或安装守护程序。
- 不用 DMG 替换 ZIP，不切换网站默认下载资产；本变更只增加并发布并行格式。
- 不重新设计工具 AppIcon，也不把鲸鱼娘图标做成官方豆包 logo 的变体。
- 不通过整体调亮预览或给标准版单独换一个滑块公式掩盖叠层问题；两个目标共享同一数值模型。
- 不删除主题的背景图、veil、浮层、输入框或消息气泡透明度；只处理会覆盖滑块的固定规则和标准版重复整窗表面。
- 不为每个主题增加目标分支，不复制两套 bootstrap，也不把标准版 DOM 适配放入桌面 UI 组件。
- 不改变窗口尺寸、主题商店、模型桥接、离线 PAK 克隆或正式发布审批流程。

## Areas of concern

- 公证耗时且依赖外部 Apple 服务；本地 ad-hoc 验证不能替代带凭据的 Release 公证。verification 必须分别标明本地镜像校验与生产公证证据。
- `hdiutil` 对同名卷、残留挂载和中断较敏感；临时卷名、路径和清理 trap 必须避免影响用户已挂载的其他镜像。
- 生成图像可能自带不透明底色、细节过密或风格漂移；必须用 alpha 检查与真实 UI 小尺寸截图验收，不能只查看 1024 px 原图。
- 主题 CSS 中的固定 alpha 有时表达默认外观而非错误；迁移只能移除其抢占运行时值的优先级，不能无差别删除颜色声明。
- 标准版 DOM 可能随官方版本变化；优先使用当前稳定 ID 和目标标记，避免散列 class。选择器失效时应回退为略有色差，而不是遮挡、禁用输入或影响消息内容。
- 预览采用抽象层合成，真实应用存在原生容器差异。验收以 CSS 变量、连续整窗层数量和截图三类证据共同判断，不能只看单个数值或主观截图。
- 工作区包含大量并行改动；实现只修改计划列出的打包、Release、主题、核心运行时、测试、生成目录和文档文件，不覆盖无关编辑。

## Acceptance criteria

1. `./scripts/build-macos.sh` 与 `./scripts/build-macos.sh --universal` 分别按 host/universal 标签生成 ZIP、ZIP SHA-256、DMG、DMG SHA-256；脚本语法检查通过。
2. `hdiutil verify` 通过；以只读方式挂载后卷根目录仅含预期的“豆包主题.app”和有效 `Applications -> /Applications`，卸载后没有残留挂载。
3. DMG 内 app 通过 `codesign --verify --deep --strict`，Info.plist 版本正确；universal 产物中 GUI 和内置 CLI 均由 `lipo -archs` 报告 `arm64 x86_64`。
4. Release workflow 的 artifact 与 GitHub Release 发布命令同时包含 `Doubao-Skin-macOS-universal.{zip,dmg}` 及两份 `.sha256`；现有 ZIP 路径保持不变。
5. 有完整凭据的 Release 记录 app 与 DMG 的 notarytool 成功、stapler staple/validate 成功；无凭据构建明确跳过公证且不伪称已公证。
6. 新 `main.png` 为 1024×1024 RGBA、背景透明、无文字/水印；52 px 和 20 px 对照中脸、蓝鲸特征和小滴口水可辨，轮廓无方形底色或裁切。
7. 鲸鱼娘 light/dark `icons.main` 均指向 `icons/main.png`；核心主题加载/打包测试通过，Web 同步后的目录、预览与主题包校验和一致。
8. 回归测试先在旧实现上复现“40% 期望 0.22/0.30、实际被覆盖为 0.72/0.82”的失败，再证明最终注入 CSS 的 page/sidebar/layer/input 等于 profile 值。
9. 全主题结构化检查不再发现受控背景变量以高优先级固定 alpha 抢占运行时 override；浅色、深色和自动模式测试均通过。
10. 标准版豆包真实 CDP 元数据显示 `data-skin-target="doubao"`、单个主题 style/背景节点，40% 下 page/sidebar 为 0.22/0.30，中心点不再出现三个同义整窗着色层；未读取页面文本或私有数据。
11. 在标准版豆包真实窗口检查“馋嘴豆包”“鲸鱼娘”和一套浅色背景主题：40%、默认值及 100% 三档均随滑块单调变化，40% 与工具预览不存在截图中原有的大幅变暗。
12. 在豆包工作真实窗口对同一组主题完成应用、拖动、浅/深色切换、恢复默认与导航后重注入；透明度、文字可读性、输入框和消息气泡没有回归。
13. `cargo test -p skin-core --locked`、桌面相关测试、`pnpm --dir apps/web sync && ./scripts/check.sh web`、`./scripts/check.sh rust`、`./scripts/check.sh workflow` 及 macOS 打包验证通过；若全仓已有无关失败，verification 必须提供目标检查通过证据并逐项隔离说明。
14. `verification.md` 记录 ImageGen 提示词与最终资产、红灯/绿灯命令、主题审计结果、DMG 校验与挂载证据、两款应用截图、实测版本、偏差和残余风险；最终 verdict 仍由 fresh-context verifier 或人工填写。

## Decision

等待产品与风险负责人确认本规格后进入实施计划；在确认前不生成图像、不修改主题、打包脚本、Release 工作流或运行时代码。
