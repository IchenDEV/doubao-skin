# 豆包工作私有协议到 OpenAI：开源方案核查

研究日期：2026-08-28
研究对象：豆包工作 `POST /chat/completion` 请求、私有 SSE 响应，以及 OpenAI Chat Completions / Responses 上游。
证据范围：GitHub 仓库源码、README、项目许可证与官方 API 文档；不采用二手教程。

## 结论

**没有找到可以直接安装后完成“豆包工作私有协议 → 外部 OpenAI 模型 → 豆包工作私有 SSE”的开源适配器。**

截至本次检索：

- 精确搜索豆包工作请求中的 `general_task_param + user_context`，只命中一个豆包**网页版**请求构造器；它把请求发给豆包，不会转给外部模型。[GitHub 代码搜索](https://github.com/search?q=%22general_task_param%22+%22user_context%22&type=code)
- 精确搜索本机实测响应中出现的 `STREAM_TIMEOUT_CONTROL`，没有命中相关豆包实现。[GitHub 代码搜索](https://github.com/search?q=%22STREAM_TIMEOUT_CONTROL%22&type=code)
- 能找到的项目都只完成以下三块中的一块或两块：构造豆包请求、解析豆包响应、转发 OpenAI 协议。**没有项目生成豆包工作原生 UI 所需的完整 SSE 状态机。**

因此现实方案是自行补一个很窄的双向转换器。现有开源代码能明显减少字段摸索和代理基础设施工作，但不能直接宣称“换模型成功”。

## 候选项目

| 项目 | 判断 | 可以复用 | 不能解决 | 许可证 |
| --- | --- | --- | --- | --- |
| [`hackerFish/dsh-video-studio`](https://github.com/hackerFish/dsh-video-studio) | **可借鉴，最接近请求端** | 其 [`doubao-web.ts`](https://github.com/hackerFish/dsh-video-studio/blob/e758f4aa8a0b64ed7d643ad76fd694e1bbbef0ec/src/providers/doubao-web.ts#L53-L93) 构造同一路径 `/chat/completion`，且请求含 `messages[].content_block`、`general_task_param`、`model_config.model_item_key`、`aggregate_params.provider_id`、`user_context`。 | 方向是“自建客户端调用豆包网页版”，不是“豆包工作调用外部模型”；响应只聚合文本，不生成私有 SSE，也不覆盖工作任务的工具和状态。源码明确依赖抓包参数。 | MIT |
| [`wangchuxiaoji-oss/doubao2api`](https://github.com/wangchuxiaoji-oss/doubao2api) | **可借鉴，响应解析最完整** | [`client.py`](https://github.com/wangchuxiaoji-oss/doubao2api/blob/95beb0788338ba268e13ca4c80890ceeae6ff055/doubao2api/client.py#L847-L1147) 覆盖 `/chat/completion` 请求体、`SSE_ACK`、`CHUNK_DELTA`、`STREAM_CHUNK`、文本/思考/搜索块和会话 ID；适合做字段类型、容错和测试用例参考。 | 项目标题和源码都明确是反向把豆包变成 OpenAI API；没有 OpenAI → 豆包 SSE encoder。其工具调用是自己在 OpenAI 兼容层实现，不能代替豆包工作的原生工具状态机。 | Apache-2.0 |
| [`null-object-0000/ai-clash`](https://github.com/null-object-0000/ai-clash) | **可借鉴，响应样本价值高** | 提供一份完整的[豆包网页 SSE 样本](https://github.com/null-object-0000/ai-clash/blob/deb84da5305e3156042c753e05d2a90e57db561a/packages/inject/examples/doubao/no_think_no_search.sse)和[解析器](https://github.com/null-object-0000/ai-clash/blob/deb84da5305e3156042c753e05d2a90e57db561a/packages/inject/src/providers/doubao.ts#L112-L229)。样本显示启动序列为 `SSE_HEARTBEAT → SSE_ACK → FULL_MSG_NOTIFY → STREAM_MSG_NOTIFY`；正式文本同时出现 `CHUNK_DELTA` 和 `STREAM_CHUNK patch_object=111/tts_content`；结束包含 `patch_object=3` 与 `SSE_REPLY_END` 的 `end_type=1,2,3`。 | 是浏览器扩展的“读取网页回答”实现，不会合成响应给豆包工作；样本来自豆包网页而非当前豆包工作版本，且缺少当前实测的 `STREAM_TIMEOUT_CONTROL`。 | GPL-3.0；不要把源码或 fixture 直接复制进本仓库的 MIT 代码，除非接受相应许可证义务。 |
| [`bifrost-proxy/bifrost`](https://github.com/bifrost-proxy/bifrost) | **可借鉴，代理传输层** | README 确认支持 HTTPS/SSE、路由、请求与响应改写、Mock、QuickJS 脚本；其 [`doubaoLikeSse.ts`](https://github.com/bifrost-proxy/bifrost/blob/fdb9cf9d9be6a3644faac941f54368a883fe27a7/web/src/components/AiResponse/parsers/doubaoLikeSse.ts#L12-L223) 已识别七类豆包事件及主要 block/patch 字段。 | 内置代码仍是观测/解析，不是 OpenAI → 豆包工作 SSE 转换器；HTTPS 路线还涉及 CA/TLS 拦截，不能替代已经验证过的 CDP 明文改道。 | MIT |
| [`higress-group/higress` AI Proxy](https://github.com/higress-group/higress/tree/bdeb9e6a8f319a8fc565728ac39ceedea358fe2a/plugins/wasm-go/extensions/ai-proxy) | **可借鉴，上游模型路由层** | 官方 README 展示以 OpenAI 协议代理豆包/方舟 provider、配置 token 和模型映射的方式；可承担外部 OpenAI-compatible provider 的鉴权、模型映射、超时和流式转发。[Doubao 配置](https://github.com/higress-group/higress/blob/bdeb9e6a8f319a8fc565728ac39ceedea358fe2a/plugins/wasm-go/extensions/ai-proxy/README_EN.md#L882-L892) | 输入和输出边界仍是 OpenAI 协议，不认识豆包工作请求，也不会生成私有 SSE。对单机 PoC 来说部署过重。 | Apache-2.0 |

`LLM-Red-Team/doubao-free-api`、`doubao2api` 的其他派生仓库以及火山方舟官方 OpenAI SDK 示例均不属于直接方案：前两类大多是“豆包网页 → OpenAI API”的反向网关，后者只证明自有客户端可以调用方舟。方舟官方目前确实允许 OpenAI SDK 以 `https://ark.cn-beijing.volces.com/api/v3` 调用 Responses API，但这与豆包工作的私有前端协议无关。[方舟快速开始](https://www.volcengine.com/docs/82379/1795150)

## 建议的最小转换器

不引入完整网关；沿用已经跑通的 CDP `Fetch` 改道，在 localhost 做两个纯函数加一个流式转发器：

```text
豆包工作 /chat/completion
  → decodeDoubaoRequest(body)
  → OpenAI-compatible /v1/chat/completions（stream=true）
  → encodeDoubaoSse(openaiDelta, requestContext)
  → 豆包工作原生主对话
```

### 1. 请求转换

第一阶段只支持普通文本：

- 遍历 `messages[].content_block[]`，只接收 `block_type=10000` 的 `content.text_block.text`。
- 生成固定 provider/model 的 OpenAI `messages=[{role:"user",content:text}]`，不要信任豆包请求中的 `provider_id` 或 `model_item_key` 来决定任意外部地址。
- `general_task_param`、`user_context`、workspace、技能路径和附件默认不外发；后续逐字段做允许列表，否则会把本地路径或企业上下文发送给第三方模型。
- 第一版明确拒绝附件、工具块、多消息分支和无法识别的 block，而不是静默丢失语义。

OpenAI Chat Completions 官方契约规定请求以 `messages` 表示对话，`stream=true` 时返回 chat completion chunk SSE，并最终发送 `[DONE]`。[OpenAI Chat Completions](https://platform.openai.com/docs/api-reference/chat/object)

### 2. 响应转换

不能把 OpenAI SSE 原样返回。依据上面的源码与公开 fixture，文本 PoC 至少应生成：

1. `SSE_HEARTBEAT`：需要等待外部模型首 token 时作为保活；具体间隔以当前豆包工作原生流为准。
2. `SSE_ACK`：复用请求的 local IDs，并产生本轮 `conversation_id`、`section_id`、question ID。
3. `FULL_MSG_NOTIFY`：回显用户消息。
4. `STREAM_TIMEOUT_CONTROL`：按当前豆包工作有效响应的字段形状生成；网页 fixture 没有这个事件，不能照抄。
5. `STREAM_MSG_NOTIFY`：创建 assistant `message_id` 和一个文本 content block。
6. 每个 OpenAI `choices[0].delta.content` 编码为同一 `message_id` 的两类 `STREAM_CHUNK`：
   - `patch_object=1` 的文本 content block 增量，供原生消息气泡渲染。
   - `patch_object=111`，`patch_value.tts_content` 为同一增量，供朗读内容同步。
   本机原生主对话实测中，若再并行发送相同文本的 `CHUNK_DELTA`，第二段起会重复追加；只发 `CHUNK_DELTA` 又得到空气泡，因此 clean-room encoder 不同时发送这条网页兼容事件。
7. 完成时发送 `STREAM_CHUNK patch_object=3`，再依次发送 `SSE_REPLY_END end_type=1`、`2`、`3`；建议功能关闭时 `has_suggest=false`。

所有 `id:` 必须单调递增，conversation / section / message / block IDs 在整条流中保持一致。公开 fixture 只能作为字段假设，最终必须以当前豆包工作 2.26.7 的真实无敏感测试流做契约测试。

若改用 OpenAI Responses，上游 delta 应取 `response.output_text.delta`，再进入同一个 encoder；Responses 有自己的事件状态机，不能与 Chat Completions chunk 混用。[OpenAI Responses 流式事件](https://platform.openai.com/docs/api-reference/responses-streaming)

### 3. 这版能证明什么

成功标准是：豆包工作主对话出现真实 assistant 气泡、能看到至少两个增量阶段、完成后不报错且不自动重试。它只证明“纯文本单轮替换”。以下仍未解决：

- 豆包服务端会话持久化和重启后的历史恢复；本地生成 ID 不等于服务端已有记录。
- 豆包工作原生工具调用、技能、附件、审批、沙箱状态和多 Agent 调度。
- 外部模型 tool call 到豆包私有工具事件的双向映射；本次开源检索没有找到实现。
- 升级后的协议兼容性。

## 建议取舍

最短路线是自己写约 200–400 行的**纯文本 clean-room encoder**，从 `dsh-video-studio` 借字段名称、从 Apache-2.0 的 `doubao2api` 借解析与容错思路、用 `ai-clash` fixture 只做人工对照，不复制 GPL 代码。代理仍使用现有 CDP 方案；无需先引入 Bifrost 或 Higress。

如果纯文本 encoder 仍不能让主对话完成，下一步应对比“本机一条成功原生流”和“合成流”的字段级差异，而不是继续增加网关层。若必须保留工具和任务调度，则需要单独抓取并实现工具事件状态机；当前没有可直接复用的开源捷径。
