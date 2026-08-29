# 豆包工作接入其他模型：可行性研究

研究日期：2026-08-28
研究范围：豆包工作公开产品面、火山方舟 API、MCP、Chromium 网络层，以及代理与客户端改造的现实边界。本文不提供绕过鉴权、TLS 或平台风控的方法，也不构成法律意见。

## 直接结论

可以实现“让豆包工作使用别的模型完成一部分工作”，但目前没有证据支持把豆包工作的核心模型安全、稳定地透明替换成任意第三方 provider。

优先级如下：

1. **官方下发的多模型列表是条件最优解，但当前不可用。** 客户端具备 ActionBar V3、多模型候选 UI 和服务端模型路由结构；如果账号获正式下发所需模型，这是唯一能保留原生任务、工具与会话语义的低风险主模型切换。当前账号实测只收到 Auto、豆包 2.1 Turbo、豆包 2.1 Pro，不能靠改本地 flag 增加 provider。
2. **当前先验证自定义 MCP 连接器，再决定是否做外部模型工具。** 豆包工作仍负责对话、规划与工具选择，外部模型以 `ask_external_model` 一类工具承担明确子任务。这是最现实的自带 API 路线，也容易限制数据范围与撤销权限；它是“模型协作”，不是“核心模型替换”。当前安装包有完整实现线索，但账号入口尚未验证。
3. **MCP 入口不可用时，做独立伴生客户端。** 火山方舟公开支持 OpenAI/Anthropic 兼容接入，并有多家模型；在自己控制的界面中切 provider 是成熟路径。结果可以通过文件、MCP 或显式操作进入豆包工作，但体验不是原生模型切换。
4. **受控单轮 PoC 已成立：主输入框提交接管 → localhost adapter → 主对话中央列。** 页面 early hook 会在豆包处理器之前取消下一次普通文本提交，只把这一条文本发送给 OpenAI 兼容适配器，并把 SSE 分块逐段写入主对话区域；实机验收为 3 个增量片段、豆包提交 0 次、释放后无重试。它不读取或模拟私有请求协议，但仍不是完整核心模型替换：原生会话持久化、历史上下文、工具、附件和任务状态均未接入。
5. **不建议产品化：全局代理/MITM、CDP Network/Fetch 代理层、补丁重打包。** 它们依赖未公开协议、流式事件、工具调用、鉴权与客户端版本；未获授权时还可能落入用户协议有关干扰、修改或逆向的风险范围。

## 事实、推断与未验证项

### 官方与标准已确认

- 豆包服务范围明确包含“豆包工作端”；企业版豆包工作与飞书账号、组织权限关联。[豆包用户协议](https://www.doubao.com/legal/terms)
- 2026-08-28 下载的豆包工作一方公开前端构建中存在 `enable_switch_moa_model`、`model_config_default_model_name`、`model_config_list` 等灰度/配置字段。[豆包工作公开页面](https://www.doubao.com/work)、[对应一方构建资源](https://lf-flow-web-cdn.doubao.com/obj/flow-doubao/doubao/desktop_online_web/static/js/5788.8634f201.js)
- 同一产品的一方公开语言资源含“新建自定义连接器”“自定义连接器仅支持在本地电脑中使用”“传输类型”“服务器 URL”“自定义 Headers”“命令”“参数”“环境变量”等文案，并给出 `https://mcp.example.com/mcp` 形式的示例 URL。[一方语言资源](https://lf-flow-web-cdn.doubao.com/obj/flow-doubao/doubao/desktop_online_web/static/js/9747.389d1714.js)
- MCP 的标准定位是让 host 中的模型访问 server 暴露的工具、资源与提示词；标准传输包括 stdio 与 Streamable HTTP。host 负责权限、上下文聚合与模型交互。[MCP 架构](https://modelcontextprotocol.io/specification/2025-06-18/architecture)、[MCP Server 能力](https://modelcontextprotocol.io/specification/2025-06-18/server)、[MCP 传输](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)
- 火山方舟公开提供 `https://ark.cn-beijing.volces.com/api/v3`，可用 OpenAI SDK 调用 Responses API；官方第三方工具接入文档还提供 OpenAI 与 Anthropic 兼容配置。[方舟快速开始](https://www.volcengine.com/docs/82379/1795150)、[Responses API](https://www.volcengine.com/docs/82379/1958524)、[接入第三方工具](https://docs.volcengine.com/docs/82379/2160841)
- 火山引擎 Agent Plan 的一方页面公开列出豆包、GLM、DeepSeek、Kimi、MiniMax 等模型，说明方舟侧存在正式的多模型供给路径。[火山引擎 Agent Plan](https://www.volcengine.com/activity/agentplan)
- Chromium 网络栈支持系统代理、`--proxy-server`、PAC 和 bypass rules。HTTP 代理转发 HTTPS 时使用 CONNECT，端到端 TLS 不会因此自动解密。[Chromium 网络设置](https://chromium.googlesource.com/website/+/refs/heads/main/site/developers/design-documents/network-settings/index.md)、[Chromium 代理实现说明](https://chromium.googlesource.com/chromium/src/+/HEAD/net/docs/proxy.md)
- Chrome DevTools Protocol 的 Fetch 域可以暂停、改写或直接 fulfill 经过对应调试 target 的请求。[CDP Fetch](https://chromedevtools.github.io/devtools-protocol/tot/Fetch/)
- Chromium 的证书验证由独立 `CertVerifier` 抽象承担，embedder 可接入自己的验证实现；当前豆包定制壳是否增加额外验证策略尚未确认，不能直接从 Chromium 默认行为推断。[Chromium 证书验证](https://chromium.googlesource.com/chromium/src/+/master/net/cert/)
- 豆包已发布的协议文本禁止未经授权的插件、外挂、系统或第三方工具干扰、修改或影响服务，也限制未经许可探查、扫描或测试系统/网络弱点，以及逆向工程、反编译和尝试发现源代码、模型、算法或系统组件；违规处理可包括限制功能、暂停或终止服务。[豆包用户协议第 5 条及违约条款](https://www.doubao.com/legal/terms)

### 本地只读/可逆探针（不是官方契约）

以下结果来自 2026-08-28 对本机 `/Applications/DoubaoWork.app` 2.25.18 的只读检查，以及通过启动参数/CDP/本地测试服务完成的可逆探针。它们只能描述这一安装包、当前账号和本次运行，不能外推为长期公开 API。本轮没有发送模型 prompt，也没有读取或记录 cookie、token、API Key 等认证值；原始应用未被修改，探针结束后调试端口和本地测试服务均已关闭。

- **应用形态已确认不是标准 Electron。** 主程序是 arm64 原生 Mach-O，直接链接 AppKit/Foundation；包内是自有 `DoubaoWork Browser.app` 与 `DoubaoWork Browser Framework.framework`，没有 Electron Framework，也没有 asar。bundle id 为 `com.work.pc.doubao`，Developer ID 为 `96L78H6LMH`，使用 hardened runtime 与 sealed resources。后文 Electron API 只能用于类比，不能据此声称豆包工作支持 `session.setProxy()` 或 Electron debugger。
- **CDP 可用范围。** `--remote-debugging-port` 实测成立；Chrome 147.0.7727.149、CDP 1.3 的 `/json` 返回 background、launcher/chat 及 chat 响应页面等多个 target。仓库当前实现会筛选内嵌页面，并通过 `Page.addScriptToEvaluateOnNewDocument` 与 `Runtime.evaluate` 注入任意页面 JS。[调试端口](../../crates/skin-core/src/live.rs#L22-L23)、[target 获取](../../crates/skin-core/src/live.rs#L87-L93)、[注入实现](../../crates/skin-core/src/live.rs#L436-L454)、[bootstrap JS](../../crates/skin-core/src/theme.rs#L1046)
- **官方模型列表是封闭的服务端下发。** 当前页面 bootstrap 的 `PREFETCHED_DATA...modeSelectData` 中 `use_mode_model_list=true`，实际只下发 Auto（key 9）、豆包 2.1 Turbo（key 4）、豆包 2.1 Pro（key 5）。包内静态 ai-views JS 还预置了“工作任务 GPT / Gemini / DeepSeek / GLM”、DeepSeek V4 Flash/Pro、GLM 5.2、Orange 5.0、豆包 2.1 Pro/Turbo/Pro-1M 等名称和文案；这些资源不能证明模型已经启用或后端可以路由，当前账号也没有收到它们。
- **模型选择是会话配置，不是 provider 注册。** 实际列表来自 `/alice/bot/action_bar_v3/list` 的 `menu_conf_v2`，包含 `mode_list`、`model_list`、`support_models`、`default_model`、`model_item_key`；提交会写入 `mode_id`、`agent_mode`、`model_item_key`、`model_extra_params`、`reasoning_effort`、`moa_model_name`、`use_deep_think`。页面 AB keys 虽有 `enable_model_select_actionbar`、`enable_switch_moa_model`、`model_config_list`，但未发现外部 provider、`base_url` 或 `api_key` 配置。
- **安装包含较完整的 MCP 客户端与工具调用实现。** 包内存在 `mcp-helper` 与 `libmcp_helper.dylib` 0.1.16；支持 STREAMABLE_HTTP、SSE_HTTP、STDIO，支持 endpoint/headers 或 command/params/env，认证类型含 APIKEY/OAUTH2/NONE，并实现 `tools/list`、`tools/call`。当前 `resources.pak` 的 gzip 资源 28386 与 29045 也包含“新建自定义连接器”“仅支持在本地电脑中使用”“服务器 URL”“环境变量”“传输类型”等完整文案。含实现与文案仍不等于当前账号入口已经启用或已完成真实调用。
- **当前 bundle 的聊天 runtime 不是 OpenAI-compatible 流式 schema。** 代码实现 `POST /samantha/chat/completion`，使用 `content-type: application/json`、`credentials: same-origin` 并解析 `text/event-stream`；每条 data 仍用私有 `{event_type,event_data}` 包裹，并分 CMPL/FIN/ERR/CMD。另有 `/chat/completion`、`/samantha/chat/async/stream` 等任务路径，所以替换不只是一处 endpoint 转发。本轮没有发送测试 prompt，完整请求体和 CMPL message schema 尚未验证。
- **页面到本地适配器的传输条件已证实。** 聊天页在正确设置 CORS 与 Private Network Access headers 后，直接 fetch `http://127.0.0.1:18765` 成功，无需绕过 CSP；本地合成 SSE 的 3 个 chunk 约在 3/83/168ms 被页面收到。这证明 early hook → localhost adapter 具备必要的网络与分块传输条件，不证明私有主模型协议已经适配。
- **CDP 拦截还需要新的事件循环。** `Fetch.enable` 与 `Network.setRequestInterception` 命令均受支持；但仓库现有 CDP client 是同步 caller、会忽略事件，并在注入后关闭连接，不能直接承担持续拦截。[同步 CDP client](../../crates/skin-core/src/ws.rs#L245-L294)
- **持久 patch 会脱离官方更新身份。** 包内有自研 `saman_updater` 与签名 manifest 校验；修改 nested bundle/资源会破坏签名并随升级覆盖。仓库离线模式也明确是克隆应用、修改 HTML/`resources.pak`、再 ad-hoc 重签，而不是保留官方签名。[本地入口](../../crates/skin-core/src/build.rs#L37-L43)、[PAK 修改与重签](../../crates/skin-core/src/build.rs#L118-L217)

### 推断

- `model_config_list` 与本机 ActionBar V3 数据共同表明它是服务端下发的模型/模式配置，而不是本地可编辑 provider 注册表。不能把改一个前端变量等同于后端接受任意模型。
- 自定义连接器文案与 MCP 标准字段高度吻合，因此“把外部模型封装成 MCP 工具”是当前最可信的扩展路线。由于 MCP server 提供的是工具/资源，豆包原模型仍会参与是否调用、如何组织上下文和怎样展示结果。
- 方舟兼容 OpenAI/Anthropic API 只说明它能接入**允许自定义 endpoint/provider 的客户端**，并不构成豆包工作的 provider seam；当前 bundle 的聊天 runtime 使用私有请求与 SSE 事件格式，不能直接把 endpoint 改到兼容 API 完成替换。
- 包内可检索到 Chromium 代理相关 flag，只能证明定制壳包含对应代码线索；当前 NetworkContext 是否应用这些 flag、覆盖哪些进程和流量尚未实测。即使路由成立，普通代理也不会自动解密 HTTPS、识别模型请求或完成私有协议转换。
- CDP Fetch 只能控制已连接且响应的 target 所看到的请求；后台进程、其他 target、native 网络栈和升级后的协议都可能绕过它。把普通文本响应改写成看似可用，不等于任务、工具调用、附件和长时流式链路完整兼容。

### 尚未确认

- 官方是否以及会向哪些账号/组织开放静态资源中预置的第三方模型名称，以及选择后由哪家后端实际执行。
- 当前账号是否显示“自定义连接器”，组织策略是否允许使用；虽已确认包内 transport/auth/tool 实现，本轮尚未完成真实 `tools/list` / `tools/call`。
- 主推理请求的完整请求体、CMPL message schema、附件/工具/任务事件及所有相关 endpoint。本轮只确认了入口和事件外层，没有发送测试 prompt，也没有做 MITM。
- 客户端是否使用额外的自定义证书校验、请求签名或设备证明。Chromium/定制 embedder 具备相关实现能力，不等于豆包工作已经使用。
- 第三方 provider 的 OpenAI-compatible 实现能否完整覆盖豆包工作所需语义。OpenAI 官方 Chat Completions 与 Responses 本身包含不同的角色、工具、流式与状态字段；“兼容”需要逐项契约测试，不能只测一条纯文本请求。[OpenAI Chat Completions](https://developers.openai.com/api/reference/cli/resources/chat/subresources/completions)、[OpenAI Responses](https://developers.openai.com/api/reference/cli/resources/responses/methods/create)

## 路线对比

| 路线 | 能否用别的模型 | 是否替换核心模型 | 稳定性 | 账号/合规风险 | 判断 |
|---|---|---|---|---|---|
| 官方 ActionBar V3 模型选择 | 取决于服务端列表 | 是 | 高 | 低 | **条件最优**；当前账号没有目标第三方模型 |
| 自定义 MCP 连接器 → 外部模型 | 能 | 否，外部模型是工具 | 中高 | 低到中 | **先验证入口**，可用时最值得做 BYOK PoC |
| 独立伴生客户端 → 方舟/其他 API | 能 | 不涉及豆包核心 | 高 | 低 | **当前可控、稳妥的 provider 方案** |
| 注入式 overlay / 精准 page fetch hook → localhost adapter | 条件成立时能 | 局部 | 中低 | 中高 | 网络/SSE 条件已证实，私有 schema 未验证 |
| CDP Network/Fetch 拦截 | 仅对应 target 的请求 | 局部 | 低 | 高 | 需新事件循环；不适合作为产品架构 |
| 系统/Chromium HTTP 代理或 MITM | 条件成立时能转发 | 未必 | 低 | 高 | 只能先做无敏感数据的路径诊断 |
| 修改/重打包客户端 | 理论上最深 | 可能 | 极低 | 很高 | 不建议；升级、签名、权限、协议与条款成本都高 |
| DNS/hosts 重定向 | 几乎不能单独完成 | 否 | 极低 | 高 | TLS/SNI、鉴权和 schema 都没有被解决 |

## 推荐验证与最小原型

零开发验证已经确认：当前 ActionBar V3 没有目标第三方模型。后续可以定期复查官方列表，但不修改 AB flag 或 bootstrap；本地值不能替后端注册 provider。

当前可执行顺序是：先确认账号是否显示自定义连接器；入口可用则做下面的 MCP 原型，入口不可用则直接做独立伴生客户端。只有在明确需要注入式体验时，才进入 page hook 的受控 PoC。

### 目标

验证“豆包工作能否把一个明确任务交给用户自有的外部模型，并把结果带回会话”，不尝试伪装官方模型，也不改客户端。包内有完整的 transport 与 tool 协议实现线索；原型需要补上的证据是当前账号入口、权限和一次真实调用。

```text
用户
  ↓
豆包工作（仍负责对话与工具选择）
  ↓ MCP 工具调用
本地 ask_external_model server
  ↓ 用户授权的 HTTPS API
方舟 / 其他 OpenAI-compatible provider
  ↓ 结构化结果
豆包工作展示并继续处理
```

建议只暴露一个窄工具，例如：

```text
ask_external_model(task, input, output_format?) -> { result, model, warnings }
```

第一轮用合成文本验证：

1. 能否创建并启用自定义连接器，重启后是否保留。
2. 工具是否真的被调用，而不是豆包自己回答。
3. 超时、取消、限流、API 错误能否清楚返回。
4. 是否只发送工具参数，而不是把整个飞书/企业上下文隐式外发。
5. 外部结果中的提示注入内容是否会被当作数据而不是指令。
6. API Key 是否只存在本地环境变量或系统凭据存储中，不进入仓库、聊天、日志和前端 bundle。

如果产品目标是“每轮都完全由指定模型回答、原生流式显示、继续使用豆包全部内置工具”，MCP 原型不会满足需求；此时应优先做独立伴生客户端，而不是继续加深对官方客户端的修改。

## 代理与魔改的技术边界

### 仅设置代理

包内可检索到标准 proxy/load-extension flags，但尚未实测当前豆包进程对这些参数的应用范围。标准 Chromium 的 `--proxy-server=host:port` 可改变对应 NetworkContext 的路由，HTTPS 通常通过 CONNECT 隧道发送；这不会自动把私有协议转换成 OpenAI 协议，也不会给代理任意读取 TLS 明文的能力。当前应用不是 Electron，不能假定存在 `session.setProxy()` 这样的运行时 API。[Chromium 网络设置](https://chromium.googlesource.com/website/+/refs/heads/main/site/developers/design-documents/network-settings/index.md)、[Chromium 代理实现说明](https://chromium.googlesource.com/chromium/src/+/HEAD/net/docs/proxy.md)

因此，一个代理适配器至少还需满足以下全部条件：

- 请求确实经过可控网络栈；
- TLS 信任链允许检查内容，且没有应用级额外校验；
- 能识别会话、附件、工具调用、心跳、重试和流式事件；
- 能把目标模型输出无损转换回豆包工作期望的格式；
- 鉴权、设备状态、风控字段不依赖服务端签名；
- 升级后继续通过完整契约测试。

任一条件不成立，就只能得到“流量走了代理”，不能得到“模型被替换”。

### CDP 请求替换

CDP Fetch 比普通代理更接近 renderer 的明文请求，支持修改请求或直接构造响应。本机已验证 `Fetch.enable` 与 `Network.setRequestInterception` 被接受，但现有同步 client 会丢弃事件并立即关闭连接，真正拦截需要常驻事件循环。它仍不是 provider 插件机制：连接一个页面不代表覆盖所有进程；一次 SSE 通道成功不代表流式语义、工具、附件和任务状态机成功。CDP 的 tip-of-tree 协议还明确可能随 Chromium 版本变化。[CDP Fetch](https://chromedevtools.github.io/devtools-protocol/tot/Fetch/)、[CDP 版本说明](https://chromedevtools.github.io/devtools-protocol/)

### 页面 early hook

现有 CDP 注入机制能在新文档加载前执行 JS，本机又已证明聊天页到 localhost 的 CORS/PNA 与 SSE 分块通道可用。因此“只 hook 明确 fetch → 转给本地 adapter → 按私有事件外层返回”在技术上比全局 MITM 更可控。尚缺完整请求体、CMPL/CMD schema、多 endpoint 与工具链验证；在这些证据补齐前，只能称受控 PoC，不能称主模型替换成功。

### 修改客户端

在自有客户端中可以于受控网络层设计稳定的 provider seam，但豆包工作是原生壳加定制 Chromium，且没有公开的 provider 扩展接口。其官方包有 hardened runtime、sealed resources、自研 updater 与签名 manifest；改 `resources.pak` 或 nested bundle 后必须重新签名，并会脱离官方签名/更新身份、在版本升级后被覆盖。技术上的“可改”不等于可维护或获授权。

## 账号、组织数据与服务边界

- 2026-08-28 访问到的豆包协议页面标注“2026-08-24 更新、2026-08-31 生效”；部署前应再次确认届时实际生效文本及企业协议。本文只据该公开版本提示风险。
- 企业版受组织管理员配置、飞书账号与组织权限影响。测试应使用明确获授权的测试账号和无敏感数据的测试空间，不应拿生产组织做流量拦截实验。
- 把飞书文档、企业知识、附件或对话交给外部模型会新增数据处理方。即使技术可行，也应先确认组织的数据出境、保留期、日志、训练使用和删除政策。
- 已发布的用户协议文本对未授权第三方工具干扰、修改、系统弱点探查和逆向作出限制，并保留限制或终止服务的处理措施。代理改写、CDP 拦截或重打包可能落入相关风险范围；是否构成违约取决于授权、目的与具体方式，不应把它们当作低风险常规集成。
- 方舟 API 是官方商业接口，适合在自有应用或获授权连接器中使用；具体模型、数据条款与区域应以购买时适用的协议为准。[火山方舟模型服务协议](https://www.volcengine.com/docs/82379/1142195)

## 本机验证待办

以下检查都应先只读或使用测试数据，结果补入本文后再决定是否写原型：

- [x] 安装包类型、版本、签名、Chromium/CDP 版本与主要 target。
- [x] 当前账号实际模型列表、ActionBar V3 来源与提交字段。
- [x] 包内 MCP helper、传输/认证类型、tool RPC 与对应资源文案。
- [x] 页面到 localhost 的 fetch 与 SSE 分块通道。
- [ ] 当前账号自定义连接器入口、权限提示与配置持久化位置。
- [ ] 连接一个只返回固定文本的无网络 MCP server，确认真实调用链。
- [ ] 使用测试 prompt 确认完整请求体、CMPL/CMD schema、工具与任务事件。
- [ ] 精确列出所有推理/任务 endpoint 与所属 target/NetworkContext。
- [ ] 不解密内容时确认哪些域名走 Chromium proxy；不要由此推断 endpoint 语义。
- [ ] 是否观察到自定义证书校验或请求签名；未观察到也不能证明不存在。

## 证据快照与可复现性

本机脱敏快照：

```text
DoubaoWork.app                 2.25.18
签名 Team ID                  96L78H6LMH
DoubaoWork Browser Framework  147.0.7727.149
resources.pak sha256          a181d790138b313056615f360948421a85f86ef7742ae0e559e3e4eff6481a37
当前模型菜单                  Auto / 豆包 2.1 Turbo / 豆包 2.1 Pro
localhost 合成 SSE            3 chunks，约 3 / 83 / 168 ms
```

下列哈希仅标识本次研究下载的一方公开构建快照；这些资源会随发布变化，不是稳定 API：

```text
https://www.doubao.com/work
sha256 a55119a4c4384c1ea761e4c62ce571dac04babbba9c0fc0bcbaaa97240ec0c38

.../static/js/5788.8634f201.js
sha256 730b084d315d4f4ee7f4b70298f9ec84ece1163b69dc2aa22b84b69a25f1c95b

.../static/js/9747.389d1714.js
sha256 3ade543d2af5a530f8ad7d0aeb13550b00eade7b9325215acd6f7531e1e9da21
```

公开 bundle 中出现某个字段或文案，只能证明该版本包含相关代码或翻译，不能证明当前账号已启用、后端接受任意配置，或该机制会长期保持兼容。
