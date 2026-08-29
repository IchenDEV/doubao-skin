---
id: "2026-08-29-disable-window-resizing"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: intent.md
risk: "low"
approved_by: "product-risk-owner"
approved_at: "2026-08-29"
---

# Spec: 固定主题工具窗口尺寸

## Requirements

1. 主题工具主窗口的固定内容尺寸必须为 `1120 × 720`，启动时在当前显示器居中。
2. 主窗口必须声明为不可缩放；拖拽边缘或角落不得改变宽度和高度。
3. macOS 绿色窗口按钮、标题栏双击和系统全屏入口不得把主窗口切换到其他尺寸或全屏状态。
4. 主窗口必须继续允许用户移动、最小化和关闭。
5. 固定尺寸和不可缩放状态必须由同一份窗口创建配置提供，不得只用最小尺寸近似限制。
6. 不增加窗口尺寸偏好、持久化恢复尺寸或针对不同显示器的自适应尺寸。

## User experience

用户每次打开「豆包主题」都会看到居中的 `1120 × 720` 主窗口。窗口可以在桌面上移动，也可以最小化或关闭；鼠标移到边缘或角落时不能拖大、拖小，绿色窗口按钮和全屏快捷方式也不会改变排版尺寸。界面内容、主题选择、应用与恢复操作保持现状。

## Technical design

- 在 `apps/desktop/src/main.rs` 中复用一组明确的主窗口宽高常量，生成居中的 `WindowBounds::Windowed`。
- 使用 GPUI `WindowOptions.is_resizable = false` 移除 macOS 原生窗口的可缩放样式；保留 `is_movable` 和 `is_minimizable`。
- 将 `window_min_size` 对齐到固定尺寸，避免平台回退路径产生更小的内容区域；实际固定能力仍以 `is_resizable = false` 为准。
- 把可独立构造的窗口参数留在现有启动文件中，增加直接断言尺寸和 `is_resizable` 的回归测试，不新增窗口管理层。

## Security and privacy

- 不新增网络访问、文件访问、进程控制或持久化数据。
- 不修改、启动或注入 `/Applications/Doubao.app` 与 `/Applications/DoubaoWork.app`。
- 真实窗口验证只操作主题工具自身，不展示或采集用户会话内容。

## Alternatives and non-goals

- 不用不断纠正 `on_resize` 的方式把窗口拉回原尺寸；该方案会闪烁并制造无效布局过程。
- 不把最小尺寸和最大尺寸设成相同值来模拟固定窗口；当前 GPUI 只公开最小尺寸，原生不可缩放配置更直接。
- 不重做响应式布局，也不删除现有紧凑布局分支；这些代码在本变更中保持不动。
- 不限制两款官方豆包客户端的窗口尺寸。

## Areas of concern

- 需要在真实 macOS 窗口确认取消 `NSResizableWindowMask` 后，绿色按钮、标题栏双击和全屏快捷方式均不会改变尺寸。
- 在低于 `1120 × 720` 可用区域的显示器或极端缩放环境中，固定窗口可能无法完整显示；本次按用户确认的最佳尺寸执行，不另设降级尺寸。
- 窗口截图可能包含桌面背景，只保留无敏感内容的主题工具界面证据。

## Acceptance criteria

1. 窗口创建配置的宽度为 `1120`、高度为 `720`，`is_resizable` 为 `false`，最小尺寸与固定尺寸一致。
2. 启动真实应用后，窗口居中且内容区域保持 `1120 × 720`。
3. 分别尝试拖拽四条边和四个角，前后窗口尺寸保持不变。
4. 尝试绿色窗口按钮、标题栏双击和 `Control-Command-F`，窗口不进入其他尺寸或全屏状态。
5. 移动、最小化、还原和关闭仍正常工作。
6. `cargo test -p doubao-skin-desktop ui_regression_tests --locked`、`cargo check -p doubao-skin-desktop --locked` 与 `./scripts/check.sh workflow` 通过。

## Decision

等待产品与风险负责人确认本规格后进入实施计划。
