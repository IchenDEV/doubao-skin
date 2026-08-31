# 豆包工作接入外部模型：最终验证结论

日期：2026-09-01  
验证对象：豆包工作 2.27.6、本机 LM Studio、`qwen/qwen3.8-27b`

## 最终结论

实验已经证明：普通文本请求可以由本地 Gateway 接管，交给 LM Studio 中的外部模型，并以真实流式增量显示在豆包工作的原生助手气泡中。最终回合没有出现官方模型重复回复或自动重试。

实验没有证明：外部模型可以安全调用豆包工作的原生 Tool 或 Skill。当前无法把 OpenAI `tool_call` 转换成可由豆包原生 AgentInfra 接受的调用，也无法把原生执行结果可靠地回送给同一个外部模型回合。因此，“外部模型 + 豆包原生 Tool/Skill”仍不可用。

本次仓库清理只保留本文。实验 Gateway、配置界面、协议探针、测试夹具和验证脚本均已移除，所以当前仓库不再提供可直接启动的外部模型接管功能。

## 已验证能力

| 能力 | 结论 |
| --- | --- |
| LM Studio OpenAI Chat Completions | 通过 |
| 外部模型真实流式文本 | 通过，观察到多个有序增量 |
| 原生助手气泡显示 | 通过 |
| 官方模型不重复回答 | 通过 |
| 请求模型与实际返回模型识别 | 通过 |
| OpenAI 标准 Tool Call 及结果续跑 | 仅在 Gateway/LM Studio 自有工具循环中通过 |
| Azure/OpenAI/Responses 协议归一化 | 契约测试通过，未完成真实远端凭据验收 |
| 豆包原生 Lark Doc Skill | 官方豆包模型路径通过 |
| 外部模型调用豆包原生 Lark Doc | 未通过；缺少可复用的原生调用契约 |
| `内容创作` Skill | 不成立；实测是推荐分类和 opaque `mode_id`，不是可投影的 Skill 定义 |

## 文本接管链路

已验证的链路为：

```text
豆包工作普通文本请求
  -> 本机 Gateway 解码并重建白名单上下文
  -> LM Studio /v1/chat/completions
  -> OpenAI SSE 文本增量
  -> 豆包私有 SSE 事件
  -> 豆包工作原生助手气泡
```

安全边界包括：Gateway 只监听回环地址；只转发允许的普通文本；Cookie、官方请求头、工作区数据、附件、本地路径、连接器、原生 ID 和未知内容块不外发；遇到未知语义时放弃接管。

## 豆包原生 Lark Doc 的真实执行方式

官方豆包工作调用 Lark Doc 时，`lark-cli` 在用户 Mac 本机运行，不在 LM Studio、Gateway 或网页服务端运行：

```text
豆包工作
  -> Alice 下发签名 TOOL_CALL
  -> AgentInfra 创建本地受管运行实例
  -> neotix.sandbox.bash.run
  -> local_runtime:shell:run_task
  -> 本机 lark-cli
  -> 原生结果上传 Alice
  -> 官方模型继续生成
```

Skill 指令位于豆包工作 Profile 的本地 `.skills/lark-doc` 目录。运行工作目录位于当前会话对应的 `~/DoubaoWork/chats/<日期>/<会话>/`。CLI 通过 AgentInfra 管理的 connector runtime、PATH/shim 和环境变量解析；现有日志未证明最终可执行文件的固定绝对路径。

## OpenAI Tool Call 与 AgentInfra 的缺口

OpenAI 标准调用提供模型生成的 `call_id`、工具名称和 JSON 参数。要交给豆包原生 AgentInfra，还需要一个由豆包可信链签发并接受的原生调用事件，以及调用结果与同一回合之间的身份关联。

完整脱敏捕获只观察到文件操作 UI patch、原生执行日志和助手继续生成，没有观察到可重放的 Tool definition、Tool call、Tool result、外部/原生 call ID 对应关系或结果续跑协议。页面原生桥接函数也是只读、不可配置的，参数和回调结构没有形成安全契约。

因此不能通过简单 JSON 字段转换完成：

```text
OpenAI tool_call
  -X-> 可自行签发的豆包 AgentInfra 调用
  -X-> 可关联回原 OpenAI call_id 的原生结果
```

Gateway 也不能自行生成 `signedEnvelope`。它属于豆包可信服务端签发的原生调用封装，原生执行端会结合调用来源、会话/回合、工具身份及签名材料校验。现有证据没有提供可由本地第三方合法签发或注册的入口。

## 三条候选路线的验证结果

1. **让豆包服务端替外部模型选择的 Tool 签发事件：未发现入口。** 当前签发发生在官方模型和 Alice 编排链中，外部模型的 `tool_call` 无法提交给该签发链。
2. **寻找不依赖 `signedEnvelope` 的稳定原生 Tool 注册入口：未发现。** 自定义 MCP 能完成安装、初始化和 `tools/list`，但实测在 `tools/call` 到达本地服务前被原生 `agent_id` 编排错误阻断；它也不是 AgentInfra 私有工具的直接注册接口。
3. **确认飞书能力的执行归属：已确认本机 AgentInfra 执行。** Alice 负责可信 Tool 调用与结果上传链；AgentInfra 在本机创建实例并启动 shell；Lark Doc 的具体读写由本机 `lark-cli` 完成。它不是 Samantha 单独在服务端完成的操作。

## 产品判断

- “豆包工作使用外部模型完成普通文本回答”在实验版本中已经端到端成立。
- “外部模型完美使用豆包原生 Tool/Skill”尚未成立，不能作为一期已交付能力声明。
- 若以后继续实现，应先取得正式的原生 Tool 注册/签发接口，或修复并验证自定义 MCP 的真实 `tools/call`；在此之前不应伪造 `signedEnvelope`，也不应由 Gateway 直接启动 `lark-cli` 冒充豆包原生 Tool。
