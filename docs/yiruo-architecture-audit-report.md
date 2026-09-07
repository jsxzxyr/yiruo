# Yiruo 壹若架构审计报告

**项目：** Yiruo 壹若
**Slogan：** Ran for Xuan, Always One. / 为萱而燃，始终如一
**阶段：** Phase 0 源码与架构安全审计
**基线：** 2026-09-07 从官方 Goose `main` 获取的源码归档；`Cargo.toml` 版本为 1.49.0，未取得不可变 commit SHA
**范围：** 静态源码、配置、桌面 IPC、存储、网络和日志审计；没有修改 Goose 核心源码

## 执行结论

Goose 适合作为 Yiruo 的底座，可复用 Agent 状态机、Provider 类型与格式适配、上下文管理、SQLite 会话、MCP 协议和桌面 ACP 通信。但当前 Goose 还不能直接作为 Yiruo 的安全边界：Provider、MCP sampling、HTTP MCP、Shell、文件系统、诊断导出、请求日志和遥测之间存在多个独立数据路径。

Yiruo 应采用安全优先的 Fork 路线，先建立 Model Gateway、Policy Engine、Tool Router、数据分类/DLP、工作区隔离、元数据审计和 Token Budget，再增加 Agent 能力。Phase 0 到此完成，现暂停等待审阅；没有开始 Phase 1。

## 1. 哪些部分可以直接复用？

- `crates/goose-agent` 的状态机协议、Operation/Inference/Effect 抽象。
- `crates/goose-provider-types` 的消息、流式响应、格式转换和 usage 类型。
- Goose 已有的上下文压缩、tool-pair compaction 和 Token 计量算法，作为受限组件使用。
- `SessionManager` 的 SQLite schema 迁移、索引和 usage ledger 思路，但必须重新设计敏感数据留存。
- MCP 协议客户端和传输适配，但只能作为不可信适配层，不能作为授权边界。
- Electron/React 的桌面壳、ACP 通信和类型化 UI 基础设施。
- 能在隔离 Worker 中运行的确定性文档处理依赖。
- 现有 security、egress、permission inspector，作为检测信号和兼容层，而非最终安全边界。

“复用”表示放在 Yiruo 安全接口之后复用，不表示保留可绕过 Gateway 的旧调用路径。

## 2. 哪些部分需要 Fork？

- 产品名称、品牌、导航、引导和普通用户文案。
- 配置系统和 Provider Registry，增加数据、成本、Token、网络和留存策略。
- Task/Session 元数据，增加 workspace、分类级别、保留期限、预算、来源和证明链。
- Extension Registry，增加来源、版本、hash、能力、允许路径/主机和失效时间。
- ACP/IPC 文件和网络命令，使主进程成为路径与网络策略唯一执行者。
- 敏感数据本地优先、Telemetry/Analytics/Crash Upload 默认关闭的默认值。
- 在保留 `LICENSE`、版权和第三方声明的前提下完成品牌替换。

## 3. 哪些部分必须重写或隔离？

1. 所有 Provider 请求入口必须收敛到 Model Gateway。
2. 所有 Tool dispatch 必须经过 Policy Engine 和 Tool Router。
3. Shell 和任意文件系统能力必须进入沙箱、任务工作区和短期 capability；默认关闭。
4. Streamable HTTP MCP 必须使用 host allowlist，并禁止直接获得 Provider 或 sampling 通道。
5. Request log、Full diagnostics、Session export 必须改成默认脱敏、元数据优先。
6. 网络访问必须改为按用途划分的 allowlist Gateway。
7. Telemetry 和 OpenTelemetry 必须移除默认内容捕获路径。
8. 敏感原文和可逆脱敏映射必须加密、本地保存、分离留存。

## 4. 主要安全风险

| 等级 | 发现 | 影响 |
|---|---|---|
| Critical | 没有单一强制 Model Gateway；MCP sampling 还存在另一条模型路径 | 敏感数据可能绕过策略，Provider 凭据边界不清晰 |
| Critical | Shell/stdio Extension 以宿主用户身份运行 | 任意命令、文件破坏和数据外发 |
| High | HTTP MCP 可配置远端 URI 并返回内容 | 不可信服务可接收数据或污染上下文 |
| High | Request log 序列化 model config 和请求 payload | Prompt、Tool Result、文档和配置可能落盘 |
| High | Full diagnostics 包含 session/config/log/prompt/schedule | 用户可能把敏感内容整体导出 |
| High | SQLite `messages.content_json` 保存序列化原始消息 | 本地数据库、备份或诊断泄露完整对话 |
| High | Egress 检查主要是命令模式识别，不能阻断全部网络 | 脚本、二进制、重定向、DNS 等路径可绕过 |
| Medium | OpenTelemetry 可在启用时记录消息、Tool 参数和结果 | 外部 Trace 后端可能获得原文 |
| Medium | Desktop preload 暴露较宽的文件和外部 URL 操作 | Renderer 失陷后本地/网络权限扩大 |
| Medium | Prompt injection 检测可配置且不是权限来源 | 恶意内容仍可能影响 Tool 选择 |

## 5. 第三方 API 数据流

当前 Provider 请求可包含 system prompt、会话历史、Tool 定义、Tool Result、图片和文档内容，并经 Provider HTTP client 发送至选定服务。Provider 的名称、模型、Endpoint、Header、重试和留存必须由 Gateway 管理；Agent 永远不能获得 API Key。

Yiruo 的 Provider Policy 对每个 Provider 分别配置 L0-L4、原文/脱敏文件、联网、Tool Calling、上下文留存、用途、Token 和费用。OpenRouter、代理、中转站和自建 API 都是独立的不可信网络边界，不能视为透明可信传输。

## 6. MCP 数据流

`extension_manager` 可连接 builtin、platform、stdio 和 Streamable HTTP MCP。Tool/Resource Result 回到 Agent Context；MCP Server 还可能发起 sampling，再次触达 Provider。因此 sampling 必须转换成 Model Gateway 的统一请求，并附带 MCP Server 身份、Task 和 Workspace。

MCP Server 默认不可信：必须有 Registry、来源/版本校验、能力白名单、路径与主机范围、网络/读写/执行权限和过期时间。MCP 内容永远标记为 `UNTRUSTED_DATA`，先做 Prompt Injection 检测再进入 Context。

## 7. 文件数据流

桌面选择或 Task Workspace 中的文件会流经 Parser、OCR、MCP 和 Model Message，随后可能进入 SQLite、Request Log、Diagnostics、Session Export、Cache、Provider 或生成文件。每一个外发或持久化边界都要经过分类和 DLP。

Yiruo 的默认目录为 `input/`、`working/`、`output/`、`temp/`、`cache/`、`audit/`。新文件最低按 L1_INTERNAL 处理。L2-L4 默认只走本地模型或保持本地；任何脱敏外发都要经过 Provider Policy 和用户确认。敏感原文不得进入 logs、analytics、crash reports 或普通审计记录。

## 8. Token 数据流

Goose 已有 Session usage、累计 usage、usage ledger、自动 Context Compaction 和 Tool Pair Compaction，但还没有完整的 Yiruo 预算模型。Yiruo 必须在每一次 Model/Tool/Search 调用前估算并原子预留，调用后按实际 usage 结算。

预算至少包括 `max_input_tokens`、`max_output_tokens`、`max_tool_tokens`、`max_history_tokens`、`max_document_chars`、`max_search_results`、`max_turns`、`max_cost` 和 `max_wall_time`，并同时执行 Task、Provider、Daily、Monthly 预算。超出时停止并询问用户，不能静默升级模型或继续循环。

## 9. GUI 应如何重新设计？

默认界面应是办公软件：Home、新建任务、最近任务、文件、草稿/结果、来源、审核和导出。核心流程是“上传材料 -> 描述目标 -> 查看数据外发摘要 -> 查看事实/结构 -> 审阅草稿 -> 确认导出 DOCX/PDF”。

普通模式使用“文档处理、公文写作、网络搜索、我的知识库、文件比较、数据分析”等产品语言。专业模式再显示 Security、Provider、Model、Tokens、Tools、Context、Network、Audit 和 Policy。

任何外发前都要展示 Provider、Model、数据等级、发送内容类型、脱敏字段、预计 Token、预计费用和留存策略。

## 10. 如何实现 Model Gateway？

建立唯一的类型化 `ModelRequest`：包含 Task、用途、分类级别、Provider capability、经过限制的 messages/tools/attachments 和预算预留。Gateway 依次执行：

1. 校验调用者、Task 和 Workspace。
2. 读取 Provider Policy。
3. 扫描并分类每条 Message、Attachment、Tool Schema 和 Tool Result。
4. 按用途最小化内容并执行可逆脱敏。
5. 预留 Token/费用预算。
6. 按策略要求用户确认。
7. 仅在 Gateway 内解析 Credential。
8. 经 Network Policy Gateway 调用本地或远端模型。
9. 清理 Secret，返回类型化结果，只记录元数据。

MCP sampling、ACP 和普通 Agent inference 都必须调用同一个接口。应增加静态检查，禁止运行时 crate 直接构造 Provider HTTP Client。

## 11. 如何实现 Policy Engine？

策略输入为 `identity、task、capability、resource、action、data_level、destination、purpose、budget、user_consent`，结果只有 `ALLOW`、`DENY`、`ASK`。每一个结果绑定 request hash，避免旧确认授权新 payload。

Tool Router、Model Gateway、Export、File Operation 和 Network Adapter 都必须在副作用发生前调用它。Shell、任意文件系统、浏览器、网络、外部 MCP、删除、覆盖、批处理、高费用模型默认拒绝或询问。现有 Goose Inspector 只能作为信号，不得成为绕过入口。

## 12. 如何实现 DLP？

先用确定性代码检查 PII、凭据、身份证明、手机号、金融数据、隐藏文本、嵌入文件、宏和 Prompt Injection，再按文档上下文保守升级分类。检测失败或未知情况采用更高等级。

批准脱敏时，使用本地加密且 Task-scoped 的 `[PERSON_001]`、`[DATE_001]` 等占位符。映射表不进入 Prompt、Tool、模型、日志、遥测、分析系统和普通数据库；模型返回后先校验占位符，再在本地恢复。碰撞、缺失、伪造或模型猜测映射均视为失败。

## 13. 如何实现 Token Budget？

在 SQLite 中增加带 reservation/settlement 的事务预算账本，记录 Task、Provider、Model、估算量、实际量、费用、时间和状态，不记录 Prompt 正文。文档先解析、分块、检索；网页先正文抽取、去重、排序、压缩；旧历史转换成结构化摘要；Tool Result 只保留有界的类型化结果和证据。

默认限制 Agent turns、Tool calls、Search Top K、重试、Wall Time、输入/输出/Tool tokens 和费用。UI 显示本次 Input、Output、Total 和费用估计，但不显示原始 Prompt。

## 14. 如何实现 Document Agent？

为 PDF、DOCX、XLSX、PPTX、TXT、Markdown、HTML、图片和扫描件建立本地 Parser/OCR 适配器。先规范化为 sections、tables、metadata、entities、chunks 和 source spans，再只把相关且有界的 chunks 给模型。

`read_document`、`extract_text`、`extract_table`、`extract_metadata`、`summarize`、`compare_documents`、`rewrite`、`proofread`、`generate_docx`、`generate_pdf` 都应是类型化操作。解析、OCR、diff、Token 统计、格式检查和导出优先使用确定性代码。

## 15. 如何实现 Official Document Agent？

通知、请示、报告、函、纪要、决定、意见、方案、总结、讲话稿、汇报材料、调研报告和工作简报分别建立版本化 Template、Structure、Style Guide 和 Validation Rules。

流程固定为：材料 intake -> 提取事实/数据/人物/单位/日期 -> 用户确认 -> 选择文种和模板 -> 生成结构 -> 初稿 -> 一致性与格式校验 -> 人工审阅 -> 润色 -> 用户确认 -> DOCX/PDF 导出。模型只填写类型化文档模型，字体、标题、段落、页眉、页脚、落款、日期和编号由确定性渲染与校验负责。

## 证据索引

- Agent Loop：`crates/goose-agent/src/machine.rs`、`crates/goose/src/agents/state_machine/`。
- Provider：`crates/goose-provider-types/`、`crates/goose-providers/src/api_client.rs`。
- MCP：`crates/goose/src/agents/extension.rs`、`crates/goose/src/agents/extension_manager.rs`。
- 安全检查：`crates/goose/src/security/`、`crates/goose/src/permission/`、`crates/goose/src/tool_inspection.rs`。
- Shell：`crates/goose/src/agents/platform_extensions/developer/shell.rs`。
- 存储：`crates/goose/src/session/session_manager.rs`。
- 诊断：`crates/goose/src/session/diagnostics.rs`、`crates/goose/src/acp/server/diagnostics.rs`。
- 桌面 IPC：`ui/desktop/src/preload.ts`。
- 请求日志：`crates/goose-provider-types/src/request_log.rs`、`crates/goose/src/providers/utils.rs`。
- 遥测和内容捕获：`crates/goose/src/posthog.rs`、`crates/goose/src/agents/gen_ai_telemetry.rs`。

## Phase 0 disposition

Phase 0 文档包已完成：源码审计、数据流清单、复用矩阵、威胁模型、DLP 设计、产品方向、路线图和验收矩阵均已建立。工作区仍保持 Goose 源码基线，未修改核心代码，下一步必须等待用户审阅和明确批准后才能进入 Phase 1。
