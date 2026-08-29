---
id: "2026-08-29-dmg-and-whale-theme-icon"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Plan: 增加 DMG、更新鲸鱼娘图标并校准标准版豆包透明度

## Files and ownership

- `crates/skin-core/src/theme.rs`：拥有透明度 profile、最终注入 CSS、bootstrap 目标标记和对应单元/主题遍历测试。
- `crates/skin-core/src/live.rs`：拥有 `TargetApp` 到实时 bootstrap 的传递、标准版目标标记、恢复清理和 CDP 相关回归测试。
- 当前审计命中的主题 CSS：`themes/{cyber-neon,doubao-dessert-giggle,doubao-snack-giggle,gallery-cozy-room,gallery-crimson-rain,gallery-moon-pine,gallery-neon-koi,gallery-whale-maid,gothic-void,mist-forest,pastel-flower-club,pastel-starry-room,pastel-tea-party,peach-sunset,qq-light-blue}/theme.css`。只调整会抢占运行时透明度的背景变量优先级，不改其他配色。
- `themes/gallery-whale-maid/icons/main.png` 与 `themes/gallery-whale-maid/theme.json`：拥有 ImageGen 最终图标和 light/dark 主图标引用；现有 `main.svg` 在确认无其他引用后才移至废弃状态或保留为未引用兼容资产。
- `apps/web/data/**`、`apps/web/public/themes/**`：仅由 `pnpm --dir apps/web sync` 生成；不手工修改。实现时只审查该命令产生的目标差异。
- `scripts/build-macos.sh`：拥有 DMG staging、系统 `hdiutil` 创建/校验、可选公证、清理、文件命名和 SHA-256。
- `.github/workflows/release.yml`：拥有 universal DMG/ZIP Actions artifact 与 GitHub Release 资产列表。
- `README.md`、`docs/development.md`、`docs/releasing.md`：拥有双格式构建、安装和发布验证说明；网站默认 ZIP 下载 URL 不在本次修改范围。
- `workflow/changes/2026-08-29-dmg-and-whale-theme-icon/verification.md`：由实现者记录命令、红绿灯、图像提示词、截图路径、DMG 证据和残余风险；最终 verdict 由 fresh-context verifier 或人工完成。
- 所有工作顺序执行并适配当前脏工作树；不得覆盖上述文件中与本变更无关的并行编辑。

## Order of work

1. **建立基线和红灯**
   - 重新记录目标文件的 `git status`，读取已接受的 intent/spec/plan，确认标准版豆包 9223 和豆包工作 9222 的实际可响应目标。
   - 把上一轮临时只读 CDP 探针收敛为可重复的测试/验收命令：只输出主题标记、CSS 变量、整窗覆盖率和背景 alpha。
   - 在旧实现上固定红灯：40% 时 profile 期望 page/sidebar `0.22/0.30`，标准版实算 `0.72/0.82`；记录当前连续整窗着色层。
   - 增加失败的 Rust 回归测试：目标标记未传入/未清理、最终 surface override 被主题高优先级声明压过、全主题审计仍能找到违规声明。
2. **修复共享透明度优先级**
   - 最小修改 `surface_opacity_css()` 的最终选择器与生成顺序，使 profile 值在浅色、深色、自动模式中成为最终值。
   - 逐个处理审计命中的主题，只移除受控背景变量上不必要的固定 `!important`，保留默认颜色和 alpha。
   - 运行核心单测和全主题审计，确认旧红灯转绿；检查未命中的主题没有被机械改写。
3. **适配标准版重复表面**
   - 将 `TargetApp` 传入 live bootstrap，在根节点添加 `data-skin-target`，并让 destroy/恢复默认清理它。
   - 使用已捕获的稳定 ID/标签 fixture 锁定标准版重复背景；加入 `data-skin-target="doubao"` 限定的最小 CSS，只透明化一个重复承担相同背景角色的整窗容器，不使用散列 class。
   - 再跑 40% 探针，确认变量和值正确、中心连续整窗着色层数量符合预览模型；对 9222 运行同一元数据探针确认豆包工作无目标错配或表面回归。
4. **生成并接入鲸鱼娘图标**
   - 使用 `imagegen` 技能先查看 `gallery-whale-maid/bg.jpg` 和“馋嘴豆包”现有角色图标，再以二者为受控参考生成透明背景原创图标。
   - 查看生成原图，检查 1024×1024、RGBA/alpha、无文字水印、角色占比和四边安全区；制作临时 52 px/20 px 缩放用于肉眼检查。若表情或鲸鱼特征不可读则继续生成迭代。
   - 将通过验收的文件复制为 `themes/gallery-whale-maid/icons/main.png`，更新 light/dark manifest 引用并运行主题加载/打包测试。
5. **同步所有主题生成物**
   - 运行 `pnpm --dir apps/web sync`，审查 catalog、鲸鱼娘包和受影响主题包的生成差异与 SHA-256；不接受无关主题内容变化。
   - 运行 Web 检查，验证新 PNG 能从本地主题包和 Web 目录加载。
6. **增加 DMG 打包**
   - 在现有 app 完成资源装配、签名、严格 codesign 和可选 app 公证之后，增加临时 staging、Applications 软链接、压缩只读 DMG、清理 trap、`hdiutil verify` 和独立校验和。
   - 凭据完整时追加 DMG notarytool、staple、validate，并在装订后重新 `hdiutil verify` 再生成 SHA-256；部分凭据继续失败。
   - 先运行 host 包缩短反馈，再运行 `--universal` 最终门禁；只读挂载检查 bundle、软链接、签名、Info.plist 和双架构，然后可靠卸载。
7. **接入 Release 与文档**
   - 更新 Release workflow 的 artifact 和 `gh release upload/create` 四个 universal 文件；保持现有 ZIP 文件名和网站 URL。
   - 更新 README、开发和发布文档，明确 ZIP/DMG 两种选择、校验命令以及本地 ad-hoc 不等于已公证。
8. **完整验证与交接**
   - 依次运行 scoped Rust/Web/workflow 检查，再运行适用的完整 gate。
   - 在固定 1120×720 工具窗口与两款真实客户端完成三主题、三档透明度、浅/深色、恢复默认和导航重注入检查，保存不含敏感数据的截图。
   - 填写 `verification.md` 为 pending，列明所有命令、结果、ImageGen 提示词/资产、CDP 元数据、DMG 挂载证据、偏差与残余风险，交给 fresh-context verifier 或人工裁决。

## Test-first proof

- **透明度红灯**：在修复前运行 profile/CDP 测试必须得到等价错误：`slider 0.40 expects page/sidebar 0.22/0.30, standard Doubao computes 0.72/0.82`。保存错误和测试名，再修改代码。
- **CSS 优先级红灯**：核心测试构造包含深色高优先级固定背景变量的主题，断言最终受控变量来自 `surface_opacity_profile(0.40)`；旧代码应失败，修复后 page/sidebar/layer/input 分别为 `0.22/0.30/0.26/0.48`。
- **目标隔离红灯**：live 单测先断言旧 bootstrap 缺少 `data-skin-target` 且 destroy 未清理；实现后分别锁定 `doubao`、`doubao-work` 和离线 snippet 无目标猜测。
- **重复表面 fixture**：使用不含文本的标准版 DOM 骨架覆盖 `#chat-route-layout`、`#chat-route-main` 和直接主内容，断言只有 `data-skin-target="doubao"` 会中和一层重复表面；工作版 fixture 和普通元素不受影响。
- **全主题审计红灯**：遍历 `themes/*/theme.css`，在旧树中报告当前命中的固定高优先级受控 alpha；修改后结果为零，同时每套主题仍可加载并保留 raw RGB/default alpha。
- **图标自动检查**：验证新文件格式、1024×1024、含 alpha 通道、四角透明、manifest light/dark 同一路径、主题包内存在该资源；视觉可读性另由小尺寸截图判断。
- **打包门禁**：`bash -n scripts/build-macos.sh` 和 Release YAML 结构检查先行；实际 host/universal 构建验证 DMG 是新增绿灯。对失败路径使用临时目录/受控环境检查最终 DMG 与 SHA 不会在 `hdiutil create/verify` 失败时出现。
- **回归命令**：至少运行 `cargo test -p skin-core --locked`、桌面目标测试、`pnpm --dir apps/web sync && ./scripts/check.sh web`、`./scripts/check.sh rust`、`./scripts/check.sh workflow`；完整 gate 的无关历史失败必须与本变更测试分开记录。

## Visual or integration proof

- 在真实“豆包主题”窗口检查鲸鱼娘列表图标和大预览：1120×720 固定窗口中 20–52 px 图标清楚、透明背景、无裁切，与浅蓝背景协调；同时检查深色外观下轮廓没有消失。
- 在标准版豆包 9223 的无敏感内容页面应用“馋嘴豆包”“鲸鱼娘”和一套浅色背景主题，分别记录 40%、主题默认值、100% 的计算 alpha 与截图。40% 必须不再出现原截图中的整窗深褐蒙层。
- 在豆包工作 9222 对同组主题执行应用、拖动、浅/深色切换、一次页面导航和恢复默认；记录 `data-skin-target="doubao-work"`、单个 style/backdrop 节点及截图。
- 对比工具预览与两个目标时只允许官方客户端布局本身的差异；若数值正确但截图仍明显偏暗，继续检查实际中心合成层，不以单元测试代替视觉验收。
- 对最终 universal DMG 执行 `hdiutil verify`、只读挂载、Finder/目录内容检查、内部 app `codesign --verify --deep --strict`、GUI/CLI `lipo -archs`、Info.plist 版本检查、Applications 软链接解析和卸载确认。
- ImageGen 原图、20 px/52 px 对照、工具预览、两款客户端和 DMG 挂载证据均保存到仓库允许的验证图片目录或 verification 引用的本地绝对路径；不提交含真实会话内容的截图。

## Risks and mitigations

- **主题默认值被误删**：只降低受控变量的声明优先级，不删除 raw RGB 或默认 alpha；以无 runtime override 的加载测试保护静态默认外观。
- **标准版官方 DOM 更新**：仅使用当前已验证的稳定 ID/元素关系并加目标限定；不用散列 class。实窗失配时停止验收并记录版本，不扩大选择器范围猜测。
- **豆包工作被连带透明化**：标准版规则必须依赖 `data-skin-target="doubao"`，并在 9222 运行对称探针与截图回归。
- **图标生成质量或授权漂移**：只引用仓库内原创生成资产，保留提示词与生成证据；检查透明通道、小尺寸和无品牌/水印，不满意就迭代而非后期拼贴第三方素材。
- **DMG 中 app 与 ZIP 不一致**：两者都来自同一个最终 `BUNDLE`；验证内部可执行文件哈希/版本和签名，不运行第二次 app 组装。
- **临时卷或半成品残留**：使用唯一 staging/临时文件和 trap，最终命名前先 verify；所有测试结束后检查 `hdiutil info` 和 dist 文件集合。
- **公证外部失败**：凭据完整时失败即阻止产物校验和/发布；本地无凭据只报告 ad-hoc、绝不宣称 notarized。
- **脏工作树冲突**：每次修改前检查目标文件差异，使用小补丁，生成目录只接受 sync 结果；不清理、不重置、不归因无关改动。

## Rollback

- 透明度回滚：撤销 `theme.rs`/`live.rs` 的目标标记和表面规则，并恢复本变更调整过的主题声明；重新运行 Web sync，使生成目录与回滚后的主题一致。
- 图标回滚：把鲸鱼娘 light/dark `icons.main` 指回 `icons/main.svg`，移除本变更新增的 `main.png`，再同步 Web 目录。背景画作和其他图标始终不动。
- DMG 回滚：移除构建脚本的 DMG 段、Release 四文件列表中的 DMG 两项和对应文档；ZIP 构建与既有下载 URL 保持可用。
- 构建产物均可由源代码重建，不提交 `dist/*.dmg`、ZIP、挂载点或临时 staging。若已在本地生成，只删除精确的本变更产物；不触碰其他 dist 文件。
- 本计划不发布 Release、不上传产物、不修改官方 app bundle 或用户数据，因此回滚不需要外部数据迁移。

## Deviations

当前无偏差。若真实 DOM 证明重复层不是 `#chat-route-main` 关系、ImageGen 无法稳定输出透明 PNG、或 Apple 工具要求改变公证顺序，先把证据和最小替代方案同步回 spec/plan 并请求确认，不静默扩大实现。

## Decision

等待工程负责人确认本计划后开始实现。确认只授权本计划列出的本地代码、生成资产、测试、文档和验证改动，不授权发布、上传 Release、使用生产凭据或修改官方豆包应用资源。
