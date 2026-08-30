---
id: "2026-08-30-doupi-app-icon-redesign"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-30"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-30"
---

# Plan: 创作、编译并验证豆皮主题 AppIcon

## Files and ownership

### Canonical editable source

- `assets/app-icon/AppIcon.icon/`
  - 由 Icon Composer 2.0 保存的唯一可编辑图标包。
  - 包内 `Assets/` 只保留新豆皮语义层；移除旧蓝色环/工作台层。
  - `icon.json` 只接受 Icon Composer UI 保存产生的变化，不在终端中直接编写或修补。

### Derived app-icon artifacts

- `assets/app-icon/AppIcon.icns`
- `assets/app-icon/Assets.car`
- `assets/app-icon/AppIcon-Default-1024.png`
- `assets/app-icon/AppIcon-Dark-1024.png`
- `assets/app-icon/AppIcon-Light-512@2x.png`
- `assets/app-icon/AppIcon-Dark-512@2x.png`
- `assets/app-icon/AppIcon.iconset/`
- `assets/app-icon/AppIcon-Dark.iconset/`
- `assets/app-icon/AppIcon.xcassets/`
- `assets/app-icon/layers/`：旧的外置重复层如果没有任何消费方则删除，避免与 `.icon` 形成第二套源；若工具链实证需要该目录，只保留从 canonical 包同步的只读副本并在 verification 说明。

### Workflow and evidence

- `workflow/changes/2026-08-30-doupi-app-icon-redesign/verification.md`
- `workflow/changes/2026-08-30-doupi-app-icon-redesign/evidence/`
  - 保存 Icon Composer 三模式预览、小尺寸对照、Finder、Dock、应用切换器、关于面板和真实窗口截图。

### Expected untouched files

- `scripts/build-macos.sh`、`apps/desktop/Info.plist`、`apps/desktop/src/main.rs` 预期不修改；只有验证发现当前消费路径无法正确使用新 canonical 源时，才提出最小偏差并先同步 change artifacts。
- 不修改主题目录、Web 目录、Bundle ID、签名和发布文件。

## Order of work

1. **建立基线。** 记录当前 Git 状态、Icon Composer 2.0/Xcode beta 路径和版本、现有 `.icon` 分组、派生文件哈希、`.icns` representation/alpha footprint，并保存旧图标仅供变更对照；不把它继续用作新设计素材。
2. **准备原创矢量层。** 在隔离的证据目录创建 1024×1024 的 `Yuba Back Sheet`、`Yuba Front Fold`、`Soybean Accent` 三个 SVG。用不规则错位薄片、数条可见薄边、局部卷角、宽幅褶皱/气孔与小黄豆建立字面豆皮识别，再通过本地渲染检查透明边界和 16/32/64 px 轮廓；构图不依赖文字、照片或旧品牌标记。
3. **用真实 Icon Composer 编辑。** 通过 Computer Use 打开 canonical `AppIcon.icon`，获取最新界面状态后隐藏旧组，逐一导入或替换三个 SVG 渲染层，按前后遮挡配置独立分组。设置 Default、Dark、Mono fill、缩放、位置、阴影、specular、折射和透明度，每次操作后重新读取 UI 状态。旧组的永久删除须在视觉方向确认后另行获得操作时确认。
4. **在 Icon Composer 内预览并收敛。** 遍历 macOS 及工具当前暴露的设计 generation 与三种外观；重点检查 16/32/64 px 是否先读成叠放薄片、薄边/卷角是否成立、黄豆是否只点题，以及是否误读成吐司、面包、字母或无意义黄色形状。检查裁切、双重阴影和视觉重量，只做单一变量的小步调整，直到三模式同时成立。
5. **保存、重开并导出。** 在 Icon Composer 中保存到原 `AppIcon.icon`，关闭/重开或重新读取文档确认层级和外观仍存在；从该文档导出 Default/Dark 对照和三模式预览。删除导入中转文件，确保正式可编辑源只在 `.icon` 包内。
6. **检查 canonical 源。** 运行 `inspect-icon.sh`、UTI/JSON/素材存在性检查和旧 asset 名称负向扫描。任何手写 JSON 痕迹、丢层或工具重开失败都回到第 3 步处理。
7. **编译派生资源。** 先在隔离目录用 Xcode beta `actool` 从 `.icon` 生成 `AppIcon.icns` 与 `Assets.car`；确认成功后机械同步仓库回退文件，再从同一输出更新预览 PNG、默认/深色 iconset 和 xcassets。逐个扫描，确保没有旧蓝紫图标残留。
8. **运行构建门。** 执行 workflow 校验、图标 inspector、最小相关 Rust/打包检查，并用现有 host 打包命令构建实际“豆皮.app”。日志必须显示走 canonical `.icon` 编译路径；核对 Info.plist、bundle 资源哈希与严格 codesign。
9. **真实系统表面验收。** 启动新 bundle 并核对 PID/可执行路径。用 Computer Use 在一致的系统外观、显示比例、Dock 大小、放大和悬停状态下检查 Finder、Dock、应用切换器、关于面板；同时验证应用窗口正常启动。若产品当前禁止缩放窗口，不为制造窄窗口证据而修改该行为，改为记录不适用及固定窗口实际状态。
10. **整理验证。** 将命令、结果、图标版本/哈希、三模式截图、系统表面截图、任何偏差和残余风险写入 `verification.md`。最终 verdict 留给新上下文 verifier 或人工确认。

## Test-first proof

- 本变更只替换视觉资产，不改变 Rust、打包或运行时逻辑，因此不新增为覆盖率服务的代码测试。
- 修改前先固定可机器验证的失败条件：canonical 源当前引用 `01-doubao-work-mark.png`/`02-skin-fold.svg`，派生图像当前包含蓝色环与紫色工作台；实现后这些引用和视觉资产必须全部消失。
- 新源保存后先运行 source inspector 和旧名称负向扫描，再更新派生物。这样即使 `actool` 能编译，也不能把错误的旧图标判为通过。
- 打包后比较 canonical 编译输出、仓库 fallback 和 bundle 内资源哈希，防止构建静默走旧 fallback。
- 不因纯视觉变更添加快照测试；小尺寸和外观模式由可复查导出图及真实系统表面证据保护。

## Visual or integration proof

- 保存 `evidence/icon-composer-appearances.png`：同一文档的 macOS Default、Dark、Mono 对照，并能看到真实 Icon Composer 界面。
- 保存 `evidence/icon-small-sizes.png`：16、32、64 px 的 Default/Dark/Mono 接触表，检查错位薄片、可见薄边、卷角、黄豆点和安全区，并记录是否仍可能误读成吐司。
- 保存 `evidence/finder.png`、`evidence/dock.png`、`evidence/app-switcher.png`、`evidence/about-panel.png`：全部来自同一新构建 bundle，截图前核对可执行路径和资源哈希。
- 保存 `evidence/app-window.png`：应用在当前受支持窗口尺寸真实启动；若窗口由既有设计固定不可缩放，verification 明确说明窄尺寸不适用而不是伪造截图。
- 对 Finder/Dock 比较只使用相同显示、系统外观、Dock 大小、放大和悬停状态；不改变系统设置以迁就结果。
- Icon Composer UI 保存、静态源检查、`actool` 编译、bundle 校验和真实表面视觉接受分别给出结论，不用单一截图替代全部证据。

## Risks and mitigations

- **Icon Composer 2.0/Xcode beta 格式漂移。** 只通过真实 UI 编辑并在保存后重开；以当前版本实际暴露属性为准，任何与 spec 字段名的差异记录在 verification。
- **SVG 在 Icon Composer 中渲染与本地预览不同。** 使用简单 path/gradient，避免滤镜、mask 嵌套和字体；导入后以 Icon Composer 小尺寸预览作为最终权威。
- **Dark/Mono 丢失折叠层级。** 优先调整语义分组的材质、明度和遮挡，不为各外观复制一套扁平图片。
- **暖黄构图像吐司、可丽饼或餐饮 App。** 使用纸张般的薄边、错位片形、不规则边缘、气孔和小黄豆建立字面豆皮识别；不增加碗、筷子、人物、厚面包边或卡通表情。
- **旧派生物继续被开发模式或无 Xcode 构建使用。** 在同一阶段更新 `.icns`、`Assets.car`、iconsets 和 xcassets，并用旧蓝紫像素/文件名扫描做闭环。
- **Dock/Finder 缓存显示旧图标。** 先核对 bundle 路径、文件时间、哈希与完整退出/重启；不清理全局 LaunchServices 或改变系统设置。
- **误改签名或打包逻辑。** 默认不触碰脚本和代码；若发现实证问题，先更新 spec/plan 的 deviation 再做最小修复。
- **工作区并行改动。** 每次写入前检查目标路径，保存无关更改；不使用工作区级 reset、checkout 或清理命令。

## Rollback

- 若新设计未通过视觉验收，只恢复 `assets/app-icon/` 下本变更涉及的 canonical 包及派生文件，不回滚其他工作区内容。
- canonical `.icon` 与派生物必须作为一个原子集合恢复；不能只恢复 `.icns` 或单张预览。
- 通过 Git 中的前一版目标文件或变更前已记录哈希进行精确恢复，不执行仓库级 `git reset --hard`、广泛 `checkout` 或系统缓存清理。
- 恢复后重新运行 inspector、host 打包和 bundle 图标检查，确认旧 AppIcon 与其回退资源一致。

## Deviations

2026-08-30 的首轮真实预览被明确判定为“看不出豆皮，只是黄色形状”；第二轮卷边原型仍有吐司/小包误读风险。因此构图已从抽象 `d`/卷片改为字面的不规则叠片、薄边、褶皱/气孔与局部卷角，旧组暂时隐藏而非永久删除。本修订等待重新审批。后续若 Icon Composer 2.0 不提供 spec 所用的某个外观/材质控件、当前固定窗口使窄尺寸验收不适用，或 `actool` 不能从该版本源生成 macOS 12 回退，先记录实证、同步本节与 verification，再选择最小调整。

## Decision

等待产品负责人重新确认本计划后继续正式实现。确认前仅保留当前 Icon Composer 原型、迭代素材与预览证据，不编译或覆盖派生图标资源，不永久删除旧组，也不启动打包或真实系统表面验收。
