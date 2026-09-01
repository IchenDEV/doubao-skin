---
id: "2026-09-01-beautiful-dmg-installer"
stage: plan
status: accepted
owner: "codex"
created: "2026-09-01"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-09-01"
---

# Plan: 精致的 macOS DMG 安装界面

## Files and ownership

- `assets/dmg/install-background.png` 与 `assets/dmg/install-background@2x.png`：拥有最终 1×/2× 背景视觉；只包含原创几何、中文安装提示和从现有 AppIcon 提取的项目配色。
- `scripts/package/macos.sh`：拥有背景预检、多分辨率 TIFF 生成、可写 DMG、精确挂载点、Finder/AppleScript 布局、`.DS_Store` 落盘、转换、失败清理及现有验证/公证衔接。
- `workflow/changes/2026-09-01-beautiful-dmg-installer/verification.md`：记录基线、原型、命令、产物、Finder 实窗截图、签名/架构/一致性证据、失败清理与残余风险；最终 verdict 由 fresh-context verifier 或人工给出。
- `workflow/changes/2026-09-01-beautiful-dmg-installer/evidence/`：只在需要提交产品级最终 Finder 截图时使用，不保留调试图、日志图或中间方案。
- 不修改 `.github/workflows/release.yml`、应用代码、AppIcon、主题、版本、网站或发布文档；现有 Release job 已调用同一 macOS 打包脚本并验证同名四个产物。
- 所有工作在当前 worktree 顺序进行；不清理或覆盖任务范围外的文件。

## Order of work

1. **建立基线**
   - 重新读取已接受的 intent/spec、记录 `git status` 和当前 `scripts/package/macos.sh` DMG 路径，确认它直接从 staging 生成 UDZO，未设置背景、坐标或 `.DS_Store`。
   - 运行 `bash -n scripts/package/macos.sh`、`./scripts/check.sh workflow` 和背景/布局断言；后者在旧实现上应因缺少 `.background`、UDRW/convert、AppleScript 和 Finder 元数据而失败，作为本变更的红灯。
2. **制作最小可逆视觉原型**
   - 在唯一临时目录内创建只含占位 app bundle、Applications 软链接和候选背景的测试镜像；不先改生产脚本，也不写真实 `/Applications`。
   - 用 ImageMagick 生成 1320×800 的暖白/柔金候选背景，再高质量缩放为 660×400，检查尺寸、文字抗锯齿、颜色对比、渐变条带和两侧安全区。
   - 使用系统 `tiffutil`、`hdiutil` 和 Finder AppleScript 把候选应用到临时可写镜像；在真实 Finder 窗口验证 660×428 bounds、约 120 pt 图标、左右坐标、标签空间和箭头对齐。必要时只迭代背景与坐标，方向稳定后删除原型产物。
3. **提交最终背景资产**
   - 只保留通过视觉验收的 `install-background.png` 和 `install-background@2x.png`；用 `sips`/`file`/`tiffutil` 检查 PNG、精确像素、同一构图和 2× 比例。
   - 原图不复制豆皮图标主体；背景文字是辅助说明，真实 Finder 项目与箭头仍独立表达安装动作。
4. **接入原生 DMG 布局**
   - 在 `scripts/package/macos.sh` 增加背景路径与窗口/图标常量，打包前 fail closed 检查两张 PNG 和系统工具。
   - staging 继续复制同一个已签名 `BUNDLE` 并创建 `/Applications` 软链接；新增隐藏 `.background` 并通过 `tiffutil -cathidpicheck` 写入临时 TIFF。
   - 先由 staging 创建 UDRW 临时镜像并挂载到 `mktemp -d` 的精确路径。通过 `osascript` 参数传入挂载点与 app 文件名，设置 icon view、窗口 chrome、背景、图标尺寸和坐标；更新并关闭 Finder 窗口后等待 `.DS_Store` 落盘。
   - 验证可写卷的 app、软链接、背景与 `.DS_Store` 后同步并卸载；用 `hdiutil convert -format UDZO` 生成现有压缩临时文件。之后保持当前 verify、可选 notary/staple、原子命名和 SHA-256 顺序。
   - 扩展单一 cleanup trap，精确清理已挂载卷、挂载目录、staging、UDRW 和 UDZO 临时文件；重复 detach 只作为 cleanup 的幂等兜底，正常路径的卸载失败仍阻止成功。
5. **验证失败路径与 host 包**
   - 先运行 shell 语法和结构断言。使用临时 PATH 中受控失败的 `osascript` 运行一次 host 打包，确认命令非零且没有最终 DMG/checksum、挂载或临时目录残留；不为测试增加生产配置开关。
   - 恢复真实系统工具后运行 `./scripts/package.sh desktop-macos`。校验 ZIP/DMG checksum、`hdiutil verify`、隐藏内容、软链接、严格 codesign、版本、arm64 架构和 ZIP/DMG 内 app 关键文件哈希。
6. **最终 Finder 与复制验收**
   - 从最终只读 host DMG 打开真实 Finder 窗口，截取不含用户文件或其他窗口的产品级证据；检查背景、清晰度、窗口尺寸、图标/文件名、箭头、无意外滚动和隐藏资源不显示。
   - 将挂载 app 用 `ditto` 复制到临时隔离的 `Applications` 目录，验证 bundle 结构、签名和启动包元数据；不触碰真实 `/Applications`。
   - 检查 1× 背景原图与当前 Retina Finder 实窗。若当前没有非 Retina 显示器，以 1× 原图的逐像素预览与 2× TIFF 信息作为普通缩放证据，并在 verification 明确限制。
7. **universal 与完整门禁**
   - 方向稳定后运行 `./scripts/package.sh desktop-macos --universal`，重复 checksum、只读挂载、软链接、签名、版本、`x86_64 arm64` 和 ZIP/DMG app 一致性检查。
   - 运行 `./scripts/check.sh all`、`./scripts/devflow validate`、`git diff --check`；检查 `hdiutil info` 与 `dist` 中不存在本变更临时命名。
   - 创建并填写 `verification.md`，附最终截图路径、命令结论和残余风险，交给 fresh-context verifier 或人工裁决；不发布、不上传、不变更 tag。

## Test-first proof

- **旧实现红灯**：固定断言要求脚本引用两张背景、`tiffutil -cathidpicheck`、UDRW、明确 mountpoint、Finder AppleScript、`.DS_Store` 检查与 UDZO convert；旧脚本必须失败，修改后转绿。该断言保护打包结构，不能替代实际镜像验收。
- **原型先行**：在生产脚本修改前，用临时占位 app 证明背景尺寸、窗口 bounds、图标坐标和 Finder 元数据链可工作；若 AppleScript/TCC 或 Finder 异步落盘假设错误，先在原型中缩小问题，不扩散到完整构建。
- **资源检查**：`file` 与 `sips -g pixelWidth -g pixelHeight` 分别得到 PNG 660×400 和 1320×800；`tiffutil -cathidpicheck`/`-info` 证明高低分辨率表示可组合。
- **失败清理红灯**：临时 PATH 中的 `osascript` 固定返回非零。新脚本必须整体失败，且精确检查最终 `.dmg`、`.sha256`、UDRW/UDZO 临时文件、staging、mountpoint 与 `hdiutil info` 均无残留。
- **实际行为绿灯**：成功 host/universal 包重新只读挂载后，`.DS_Store`、`.background/install-background.tiff`、同一 app 和绝对 Applications 软链接存在；Finder 真实打开能恢复接受的布局。
- **兼容性回归**：现有 ZIP 文件名和内容、DMG 文件名、checksum、公证条件与 Release workflow 引用不变；`bash -n`、workflow gate、完整 gate 和 current Release verification steps 继续通过。

## Visual or integration proof

- 首次原型和最终 host DMG 都在真实 Finder 窗口中检查，不以源 PNG、AppleScript 退出码、staging 文件或 `.DS_Store` 存在代替视觉验收。
- 最终窗口目标为约 660×428，内容背景 660×400；左侧 app 与右侧 Applications 图标约 120 pt，标签完整，中间箭头与实际拖拽路径对齐，顶部提示不与图标争夺注意力。
- 背景需在原尺寸 1× 查看，并在当前 Retina 显示缩放下从最终 DMG 截图；检查文字边缘、渐变、金色对比、AppIcon 与系统 Applications 图标的视觉平衡。
- Finder chrome 只保留系统标题栏和标准窗口控制；工具栏、侧栏、路径栏、状态栏、隐藏 `.background` 和 `.DS_Store` 不可见。
- 拖拽动作的真实复制语义由 Finder 项目验证；为避免覆盖用户安装，实际数据校验使用临时隔离 Applications 目录，不向真实系统目录写入。
- 产品级最终截图保存到变更 evidence，调试截图、其他 Finder 内容、桌面图标和验证日志不进入仓库。

## Risks and mitigations

- **Finder 自动化权限或 CI 无 GUI 会话**：先用本机临时原型验证；生产脚本失败即阻断产物，不静默生成普通 DMG。若 macos-26 runner 实际失败，暂停并回到 spec，不提交预生成 `.DS_Store` 或引入依赖规避证据。
- **`.DS_Store` 异步竞态**：AppleScript 明确 update/close，使用短而有界的等待，再检查文件；最终 DMG 重新挂载并打开验证，而非只信任可写卷。
- **窗口内容高度与 titlebar 偏差**：原型实窗先校准；允许小幅调整高度/坐标，但保持 660 pt 宽、左右关系和背景比例，并把最终值记录在 verification。
- **Retina TIFF 兼容性**：两个 PNG 使用完全一致构图和 2× 尺寸，以系统 `tiffutil` 合并；失败时不改用模糊单图兜底。
- **破坏签名或 ZIP/DMG 不一致**：背景只放卷的隐藏目录，app 在签名后原样复制；对 ZIP 与 DMG 内 Info.plist、主可执行文件哈希、codesign 和架构做对称验证。
- **清理误伤**：每个删除目标都来自当前 `DIST_DIR` 下固定文件名或本次 `mktemp` 返回值，先确认非空/存在；不对工作区根目录、真实 Applications 或宽泛变量执行递归删除。
- **验证成本**：先占位原型与 host，坐标稳定后只运行一次 universal 和一次完整 gate；不为同一视觉微调反复重编译。
- **签名信任误解**：保留现有签名，不把 DMG 美化描述为公证或 Gatekeeper 修复；verification 明示本次实际签名模式。

## Rollback

- 源代码回滚只需恢复 `scripts/package/macos.sh` 的原 DMG 段并移除 `assets/dmg/` 两张背景；ZIP、应用构建、签名、现有基础 DMG 文件名和 Release workflow 均保持可用。
- 删除本变更生成的精确 `dist/Doubao-Skin-macOS-{arm64,universal}.dmg` 与对应 checksum 前先确认文件名；所有构建产物可重建且本变更不提交 dist。
- 若测试时仍有本变更挂载点，先使用记录的精确 mountpoint 执行 `hdiutil detach`，再删除临时目录；不使用广泛卷名匹配或强制卸载无关卷。
- 背景和截图是无用户数据的项目资产；回滚不涉及迁移、远端状态、真实 `/Applications` 或官方豆包应用。
- 本计划不创建 tag、GitHub Release、部署或生产公证，因此不需要外部回滚。

## Deviations

当前无偏差。若 Finder 原型证明多分辨率 TIFF、POSIX 路径 AppleScript 或 `.DS_Store` 在受支持构建环境中不可行，先记录最小失败证据并回到 spec 请求确认，不自动引入第三方工具、预生成二进制模板或发布普通 DMG。

## Decision

等待工程负责人确认本计划后开始实现。确认授权本计划列出的本地背景资产、打包脚本、测试/验证工件和临时镜像操作；不授权发布、上传 Release、写入真实 `/Applications`、修改官方豆包应用或使用生产凭据。
