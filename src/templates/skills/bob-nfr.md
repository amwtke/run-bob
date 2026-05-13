---
name: bob-nfr
description: |
  触发条件:用户输入 /bob-nfr <spec-path>(主入口:对一个用例 spec 跑 NFR review),
  或 /bob-nfr --story <story-path>(退路:spec 未写时,从 story 拉上下文),
  或 /bob-nfr --refresh(已有 04-nfr-*.md 时强制重跑)。

  在某个用例 spec 写完 + Superpowers TDD 实施完 + UT 完备**之后**,
  可选地跑一遍由浅入深的 NFR review。采用 NFR-Cards 的 13 张卡片
  作为提问骨架,LLM 按 per-story 上下文从中筛选相关卡片(5-8 张),
  逐张三段式追问到量化答案;不接受"系统要快"这种空话,但允许
  "待定 · 需压测后给"。产出 docs/bob/04-nfr-<spec-slug>-<date>.md。

  适用于 Bob 4 环 Clean Architecture 工作流的 phase 2:per-story
  实施完后的技术质量 review。当用户说"跑 NFR"、"质量 review 一下"、
  "过一遍 13 张卡"、"看看有没有遗漏的 NFR"时也应触发此技能。
---

# Bob NFR Review Skill

## 触发

```
/bob-nfr <spec-path>                # 主入口:跑这个 spec 的 NFR review
/bob-nfr --story <story-path>       # 退路:spec 未写时,从 story 拉上下文
/bob-nfr --refresh                  # 已有 04-nfr-*.md 时强制重跑
```

或自然语言触发:"跑 NFR"、"质量 review 一下"、"过一遍 13 张卡"、"看看有没有遗漏的 NFR"。

## 前置条件

- **必须有** `<spec-path>` 或 `--story <story-path>` 之一(两者都无 → 拒绝运行,提示用户至少提供一个)
- 项目位于 git 仓库内(用于读取 spec / story / ARCHITECTURE.md 等)
- 建议:Superpowers TDD 已完成 + UT 跑绿后再启动 NFR review

## 提问规约(强制)

任何需要用户选择的问题,**必须**按下面三段式输出。**禁止**抛开放问题。

格式:

> **[问题序号] [问题]**
>
> **推测**:<你的判断>
> **理由**:<一句话>
> **推荐选择**:`<具体一个选项>`
>
> 是否同意?(回"是"走推荐;回"否,理由是 ..."重判;回"否,我选 X"切到 X)

## 目标

per-story NFR review。**只**回答两个问题:

1. **这个 spec / story 实施完后,有哪些非功能性需求该过一过?**
2. **每条 NFR 的量化目标 + 落地建议是什么?**

**不写代码、不出新 spec、不画架构**。产出一份 NFR review 报告,记录决策 + 建议新增 story 清单。

## 工作流(5 个 Stage)

```
Stage 0. 输入归并(读 spec 或 story + 自动定位上下文)
Stage 1. 三段式确认是否要跑(可立即退出)
Stage 2. LLM 筛选相关卡片(从 13 张挑 5-8 张)
Stage 3. 逐张卡片由浅入深三段式追问(核心提问 → 思考要点 → bob 细节 → 量化收口)
Stage 4. 写报告 + 建议新增 story 清单
```

---

## Stage 0. 输入归并

读取以下输入,按优先级:

1. `<spec-path>` 参数 → 读 `docs/specs/spec-*.md`
2. 或 `--story <story-path>` → 读 `docs/bob/02-stories/*.md`
3. 自动从 spec 提取:
   - GWT 场景(推断 4 Reliability / 5 Monitoring 的相关性)
   - 涉及 Entity / Port / UseCase 类名
   - "交给 Superpowers 的开放问题"段(技术栈线索 → 6 AuthN / 8 Security 卡片相关性)
   - 是否含并发场景 → 触发 1 Performance / 4 Reliability

向用户三段式通报输入归并结果。

---

## Stage 1. 入口确认

> **Q0: 这个 spec 要不要跑 NFR review?**
>
> **推测**:建议跑。spec 涉及订单写操作 + 并发不超卖场景,有 Performance / Reliability / Data Privacy 三个常见 NFR 维度值得过一遍。
> **理由**:bob TL 风:实施完代码就交付,容易把 NFR 推到生产事故才发现。10-15 分钟过 13 张卡里相关的几张,投入产出比高。
> **推荐选择**:`跑`
>
> 是否同意?(回"是"进入 Stage 2;回"不需要" → skill 退出,留 `docs/bob/04-nfr-<slug>-<date>.md` 写"用户选择跳过 NFR review")

用户回"不需要" → Stage 4 仍写一份**只含跳过决策**的简短报告,**留痕**。

---

## Stage 2. 卡片筛选

LLM 从 13 张中挑出相关的,三段式确认:

> **Q1: 这个 spec 跑哪几张卡片?**
>
> **推测**:5 张。1 Performance / 3 Capacity / 4 Reliability / 5 Monitoring / 9 Data Privacy。
> **理由**:GWT 场景 4 是"事务回滚"、场景 5 是"并发不超卖" → 触发 Performance + Reliability;订单数据 → Data Privacy;余下 8 张要么继承项目级、要么跟当前 story 不沾、要么基础设施已定。
> **推荐选择**:`5 张:1, 3, 4, 5, 9`
>
> 是否同意?(回"是"走推荐;回"否,加 7 Authorisation"调整;回"否,只跑 1 + 4"切到 2 张)

筛掉的卡片在报告 §4 列出 + 简短理由(项目级 / 基础设施已定 / 当前 story 不沾)。

### 卡片相关性的常见分布

经验法则(一个典型 feature spec 跑 NFR 时,大概率相关的卡片):

| 类型 | 常相关卡片 |
|---|---|
| 写操作 + 状态变更 UseCase(取消 / 审批 / 退款) | 1 Performance / 4 Reliability / 5 Monitoring / 9 Data Privacy(若涉及个人信息) |
| 查询 UseCase | 1 Performance / 3 Capacity / 5 Monitoring |
| 鉴权相关 UseCase | 6 Authentication / 7 Authorisation / 8 Security |
| 跨系统集成 UseCase | 8 Security / 13 Compatibility / 5 Monitoring |
| 项目首次跑 | 全部 13 张 |
| refactor story | 大多数 NFR 已在原版本固化 → 通常 0-2 张相关 |

LLM 应自动筛选,不要强制问 13 张。

---

## Stage 3. 逐张卡片追问(由浅入深 + 量化收口)

对每张相关卡片 X,4 个 Step:

### Step 3.1 核心提问(NFR-Cards 原文 1-2 问)

直接从卡片定义里搬。

### Step 3.2 思考要点(简述)

LLM 提示用户思考维度,不强制回答。

### Step 3.3 bob 实施细节追问(逐项,可拆)

从下面 13 张卡片的"bob 实施细节追问"段挑该卡对应的细节,**每条三段式过一次**。用户可"否,我不在意"跳过个别细节。

### Step 3.4 量化收口

每张卡片至少要追问一个**可量化指标**。

> **追问(直到给出数字)**:
> - 你希望 cancelOrder 的 P95 是多少毫秒?
>   - 用户给"快" → 反问 "用户能接受 100ms / 500ms / 2s 中哪一档?"
>   - 用户给"200ms" → ✓ 记录
> - 系统峰值 QPS?
>   - 用户不知道 → 标 `⚠ 待定 · 需压测后确定` + follow-up:在 §5 建议新增 story 清单 加 "为 cancelOrder 压测出 P95/QPS 基线"

**量化规则**:

- 用户给得出数字 → ✓ 记录
- 用户连续 2 次给不出数字 → 标 `⚠ 待量化`,记录"何时能给"(需压测 / 需业务方确认 / 需 ADR / 需上线后埋点统计)
- **量化优先 + 允许待定**:bob TL 风:逼 但不堵

---

## 13 张 NFR 卡片(详细)

### 卡 1 · Performance(性能)

**核心提问**(NFR-Cards §1):
- 重要的吞吐量 / 响应时间要求是什么?
- 系统在多大负载下会失效(崩溃点)?
- 是否需要专门规划性能测试?
- 哪些环节需要考虑异步化处理?

**思考要点**:
- 性能不是标量,而是 P50 / P95 / P99 / P999 的分布。"快"必须落到具体分位数
- 异步化是性能优化核心武器(MQ / Reactive / io_uring),但代价是一致性变弱

**bob 实施细节追问**:
- DB 索引是否命中?MyBatis SQL EXPLAIN 看过?
- 需要缓存吗(本地 Caffeine / Redis)?TTL / 失效策略?
- 需要异步化吗(MQ / Reactive)?业务能容忍最终一致吗?
- P95 / P99 / P999 目标各是多少毫秒?

### 卡 2 · Scalability(可扩展性)

**核心提问**(NFR-Cards §2):
- 当用户量变化时,如何提升吞吐 / 响应?
- 如何测试可扩展性?

**思考要点**:
- 扩展方式:Scale Up(垂直)vs Scale Out(水平)
- 水平扩展受 USL 制约——争用 + 协调开销在拐点之后会让"加机器"反而变慢
- 与 Performance 区别:Performance 关注"单点跑多快",Scalability 关注"加资源能不能跑更多"

**bob 实施细节追问**:
- 单点锁是否存在?共享 DB 写瓶颈?
- 水平扩展时新增节点需做哪些适配(session / 文件存储 / 本地缓存)?
- USL 拐点估算?

### 卡 3 · Capacity(容量)

**核心提问**(NFR-Cards §3):
- 是否有特定的存储需求?
- 是否需要考虑峰值负载?
- 系统需要处理多大的数据量?
- 多少并发用户同时使用?

**思考要点**:
- 容量四维:存储 / 计算(QPS/TPS)/ 网络带宽 / 并发连接数
- 日均 ≠ 峰值——电商大促、春运峰值可达均值 10~100 倍
- 容量直接决定成本,是 NFR 中少有的能直接换算成钱的项

**bob 实施细节追问**:
- 表预估行数?日 / 月 / 年增长率?
- 分库分表阈值?
- MyBatis 分页是 limit-offset 还是游标?
- Redis 内存上限?连接池上限?

### 卡 4 · Reliability(可靠性)

**核心提问**(NFR-Cards §4):
- 出错(数据不准 / 服务不可用)的业务代价是多少?
- 需要在多大程度上保障可靠性?

**思考要点**:
- "航天飞机 vs 手机游戏"——可靠性不是越高越好,与业务代价匹配才合理
- 每多一个 9 的可用性,成本通常数量级增长
- 工程手段:幂等 / 重试 + 退避 / 断路器 / 舱壁隔离 / 混沌工程

**bob 实施细节追问**:
- 该 UseCase 是否幂等?幂等键(idempotent key)怎么设计?
- 需要分布式锁吗?选型:Redisson / ZK / DB 行锁?
- Saga / 补偿事务?
- 重试 + 退避策略?断路器?

### 卡 5 · Monitoring(监控 / 可观测性)

**核心提问**(NFR-Cards §5):
- 是否已有现成的工具和基础设施?
- 应该测哪些业务指标 / 技术指标?
- 监控怎么做?
- 需要哪些告警?

**思考要点**:
- 现成工具决定选型(Prometheus + Grafana + Loki + Jaeger 还是云方案)
- 业务指标 ≠ 技术指标。技术 = CPU/QPS/Latency,业务 = 订单数 / 支付成功率 / GMV
- 告警遵循 USE(Utilization/Saturation/Errors)或 RED(Rate/Errors/Duration)方法

**bob 实施细节追问**:
- 日志埋点是否含 traceId / spanId?
- 业务指标(订单数 / 成功率 / GMV)是否上 metric?
- 技术指标 P95 / 错误率告警阈值?
- 告警渠道(钉钉 / 飞书 / PagerDuty)?

### 卡 6 · Authentication(身份认证)

**核心提问**(NFR-Cards §6):
- 用户如何被识别?
- 应遵循什么标准,或复用哪些已有认证系统?

**思考要点**:
- 行业标准:OAuth 2.0 / OIDC / SAML 2.0 / LDAP / Active Directory / WebAuthn / Passkeys
- 复用已有(企业 SSO / 社交登录)往往优于自研
- 现代趋势:无密码(Passkeys / WebAuthn)、MFA 默认开、风险自适应

**bob 实施细节追问**:
- 复用项目 SSO / OAuth / OIDC 还是自己加?
- 需要 MFA / Passkey 吗?
- token 刷新机制?Refresh token?

### 卡 7 · Authorisation(授权)

**核心提问**(NFR-Cards §7):
- 需要哪些角色和权限?
- 谁来维护这些权限?
- 在哪些层级上生效(网关 / 服务 / 数据行)?

**思考要点**:
- 模型选型:RBAC / ABAC / ReBAC(Google Zanzibar)
- 层级是关键决策——API Gateway / 微服务自判 / 数据行级
- 权限维护常被忽视(用户离职 / 调岗时谁回收)

**AuthN vs AuthZ**:AuthN 回答"你是谁",AuthZ 回答"你能做什么"——严格分清,实现解耦。

**bob 实施细节追问**:
- 谁能调这个 UseCase?(角色清单)
- 网关级 RBAC / UseCase 内 guard / 数据行级?
- 管理员能否绕开?
- 权限维护方:运营 / IAM 系统 / 代码?

### 卡 8 · Security(安全)

**核心提问**(NFR-Cards §8):
- 需要建立什么样的安全审计 / 渗透测试流程?
- 企业安全指南是什么?
- 是否需要 SSL / VPN?

**思考要点**:
- 安全治理层面(流程 / 合规 / 传输加密),与 Data Privacy 卡片(数据本身)互补
- 渗透测试应定期化(每季度 / 每大版本),不是临门一脚
- SSL/TLS 是底线;mTLS / Zero Trust / Service Mesh 加密是更进一层

**bob 实施细节追问**:
- Spring 栈下用 Spring Security 还是 Sa-Token?
- 接口签名校验(HMAC / JWT)?
- HTTPS / mTLS?
- SQL 注入 / XSS 防护?

### 卡 9 · Data Privacy(数据隐私)

**核心提问**(NFR-Cards §9):
- 哪些数据应该加密?对终端用户、对运维人员可见还是隐藏?
- 开发 / 测试环境能用生产数据吗?如果不能,如何脱敏 / 移除?

**思考要点**:
- 数据分级:公开 / 内部 / 敏感 / 绝密——不同级别对应不同加密、访问、留存策略
- "运维可见 vs 用户可见"易忽视——DBA 能不能直接查到用户手机号 / 地址?
- 测试数据脱敏:静态(批量替换)/ 动态(查询时屏蔽)/ 合成数据

**bob 实施细节追问**:
- 日志是否脱敏(手机号 / 身份证 / 卡号 / IP)?
- 接口返回值是否脱敏?
- DB 字段加密(AES / GPG / 国密)?哪些字段?
- 测试环境用脱敏数据?

### 卡 10 · Configurability(可配置性)

**核心提问**(NFR-Cards §10):
- 用户或管理员能否配置功能行为?
- 通过什么方式管理配置:配置文件 / UI / API?

**思考要点**:
- 配置类型:启动时(file)/ 运行时(Nacos / Apollo / Consul)/ 特性开关(LaunchDarkly)
- 谁是配置的"用户"?管理员 / 运营 / 最终用户——不同界面与权限
- 配置变更也要可审计、可回滚

**bob 实施细节追问**:
- 哪些参数运行时可调(超时 / 重试 / 限流阈值 / 缓存 TTL)?
- 用配置中心(Nacos / Apollo / Consul)还是 application.yml?
- 特性开关(LaunchDarkly / Unleash / 自建)?
- 配置变更可审计 / 可回滚?

### 卡 11 · Extensibility(可扩展 / 插件能力)

**核心提问**(NFR-Cards §11):
- 是否需要提供插件能力?面向谁(内部团队 / 合作伙伴 / 第三方 / 最终用户)?
- 需要提供哪些支持(SDK / 文档 / 市场)和限制 / 安全约束?

**思考要点**:
- 插件本质是把扩展点暴露给外部——需要稳定 API/SBI、版本管理、生命周期
- 不同对象不同策略:内部团队用 SPI,生态伙伴用 Webhook + OpenAPI,第三方用沙箱
- 安全约束:运行权限边界 / 资源消耗限制 / 代码审核

**Extensibility vs Scalability**:Scalability 是承载量级的扩展,Extensibility 是功能能力的扩展。完全不同。

**bob 实施细节追问**:
- 这个 UseCase 是否需要给外部插件钩子?
- Webhook / 事件订阅?
- SDK?面向谁?

### 卡 12 · Portability(可移植性)

**核心提问**(NFR-Cards §12):
- 切换到另一个数据库 / 操作系统有多重要?

**思考要点**:
- 决定架构的"抽象层厚度"——可移植代价是性能损失 + 代码复杂度
- 典型决策:JPA/Hibernate(可移植)vs 原生 SQL/存储过程(性能好但绑定)
- 现实经验:绝大多数系统永远不会真换 DB——为不会发生的事过度抽象是浪费。但合规 / 国产化场景下 DB 切换会真实发生

**bob 实施细节追问**:
- DB 切换的概率(信创场景:Oracle → 达梦 / Spring Boot → 东方通)?
- 写原生 SQL 还是 JPA 抽象?
- OS 锁定的特性(io_uring / eBPF)?

### 卡 13 · Compatibility(兼容性)

**核心提问**(NFR-Cards §13):
- 需要与哪些其他系统集成?
- 需要遵循哪些行业标准?
- 需要考虑哪些已有数据格式?

**思考要点**:
- 集成的"反向约束力"被低估——下游 API / 数据格式 / 调用契约会反过来塑造架构
- 行业标准:金融(ISO 20022 / SWIFT)、医疗(HL7 / FHIR)、电信(3GPP)、电商(EDI)
- 与 Portability、Extensibility 微妙区别:Compatibility 强调"与外部系统协同",Portability 强调"切换底层平台",Extensibility 强调"被他人扩展"

**bob 实施细节追问**:
- 要兼容老接口吗?数据格式标准(ISO / FHIR / HL7 / 自定义)?
- 向前 / 向后兼容性?API 版本策略(URL v1/v2 / header / content negotiation)?

---

## Stage 4. 写报告

报告路径:`docs/bob/04-nfr-<spec-slug>-<YYYYMMDD>.md`

`<spec-slug>` 由 spec 文件名 / 一行话生成(3-5 个汉字 / 英文 kebab),`<YYYYMMDD>` 是 UTC 日期。

模板:

```markdown
# NFR Review · <spec 一行话>
日期 · <YYYY-MM-DD> · spec 路径 · <spec-path> · 跑过 N 张卡

## 1. 上下文(3 行)
- 用户 / 系统角色:<...>
- 主要 UseCase:<...>
- 关键场景:<...>

## 2. 卡片筛选
**跑了 N 张**:1 Performance / 3 Capacity / 4 Reliability / 5 Monitoring / 9 Data Privacy
**跳过 M 张**:
- 2 Scalability — 继承项目级
- 6 Authentication — 公司 SSO 已对接
- ...

## 3. 卡片明细

### 卡 1 · Performance
- **量化目标**:P95 < 200ms · QPS 峰值 500 · ⚠ 待量化(QPS 需压测后给基线)
- **决策**:
  - 加 Redis 缓存 OrderRepository.findById(TTL 60s,失效策略:写时淘汰)
  - MyBatis SQL 加 status 索引(已确认 EXPLAIN 命中)
- **实施建议**:
  - SQL migration:`sql/V_3_2__add_status_idx_on_order.sql`
  - OrderRepository.findById 改 `@Cacheable("order")`

(其他卡片同结构)

## 4. 建议新增 story 清单
> 用户视情况手动调 `/bob-stories <这些需求>` 补 story。

- **N1** · <story 名> · 优先级 <High/Medium/Low>
  - 详细需求:<...>
- **N2** · <...>
- ...

## 5. 待办 / 待量化
- 卡 X · <子项> · 待 <压测 / 业务方确认 / ADR / 上线后埋点统计>
- ...

## 6. 跳过的卡片(用户选择)
(无 / 或列出用户主动跳的)
```

---

## TL 风对话

跑完每张卡片,**主动 raise concern**(像真 TL 一样):

- "你这张 Performance 卡 P95 给了 200ms,但 cancelOrder 内部要调 PaymentGateway.refund——那个超时是 3s。200ms 内做完事实上不可能,要重新对齐期望。"
- "Data Privacy 给的脱敏方案是'日志层 MaskingLayout',那运维直接 SQL 查 DB 怎么办?数据隐私不止是日志,要不要顺手讨论一下 DB 字段加密?"
- 给推荐时**明确说出代价**:"加分布式锁有性能代价(每次 cancelOrder + Redis 一次 RTT),用户能接受 ~5ms 抖动吗?如果不能,要考虑 DB 行锁 + 事务隔离的方案。"

用户回"否"或"我先这样" → 尊重决定,在报告 §5 待办段追加 "用户选择不做 X,后果:..."。

## 与 bob-stories / bob-spec 的关系

- 上游:`/bob-spec <用例名>` 写完 + Superpowers TDD 实施完 + UT 完备
- 下游:报告 §4 列出"建议新增 story 清单",由用户决定是否调 `/bob-stories` 补 story
- **不动其他 skill 的产出**(ARCHITECTURE.md / identity 文档 / stories 索引等)

不自动 spawn 任何新文件除了 04-nfr 报告本身。
