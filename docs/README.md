# 项目文档

这里是仓库文档的统一入口。README 面向使用者；本目录记录当前工程事实、开发流程和发布方式；`design/` 只保留可直接约束实现的产品与主题规范。

## 产品与架构

- [工程架构](architecture.md)：模块边界、运行路径、主题数据流和许可证边界。
- [主题与界面标准](../design/theme-standard/README.md)：桌面产品规则、主题包格式、Schema、设计令牌和验收清单。
- [主题 Schema](../design/theme-standard/theme-v2.schema.json)：`theme.json` 第二版的机器可读契约。

## 开发与交付

- [本地开发](development.md)：环境、常用命令和本地打包。
- [研发工作流](development-workflow.md)：intent → spec → plan → verification → PR → release。
- [macOS 发布](releasing.md)：GitHub Release、签名、notarization 和发布检查。
- [网站部署](website-deployment.md)：Vercel 设置、预览与生产上线流程。
- [提交新主题](submitting-themes.md)：主题生成、检查、素材许可与 Pull Request 流程。
- [贡献指南](../CONTRIBUTING.md)：提交主题、资产与代码的规则。

## 研究记录

`research/` 保存与特定豆包工作版本、账号或公开资料绑定的时效性结论。它们是证据快照，不是稳定接口承诺。

- [其他模型接入可行性](research/model-integration-feasibility.md)
- [豆包私有协议到 OpenAI 的开源方案核查](research/openai-protocol-bridge.md)

## 文档约定

- 当前行为只写一次：架构事实放在 `architecture.md`，主题契约放在 `design/theme-standard`。
- README 只保留使用入口和一个当前版本产品截图；主题视觉由各主题自己的 `preview.jpg` 展示。
- 概念图、调试截图和阶段性验收图不进入正式文档；需要保留的验证证据放进对应 `workflow/changes/<id>/verification.md`。
- 研究文档标明时间和验证边界，不能把本地观察描述成官方长期能力。
