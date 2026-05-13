# bob-nfr · per-story 非功能性需求 review skill 设计

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-14
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 相关:
> - phase 0 spec:`docs/superpowers/specs/2026-05-14-bob-survey-design.md`
> - phase 1 spec:`docs/superpowers/specs/2026-05-15-bob-stories-design.md`
> - phase 1.5 spec:`docs/superpowers/specs/2026-05-16-refactor-safety-net-design.md`
> - NFR 骨架来源:`/Volumes/ExternalSSD/Downloads/NFR-Cards-Summary.md`(13 张 NFR 启发性问题卡片)

---

## 0. 目的与一句话总结

新增 `/bob-nfr <spec-path>` skill 作为 bob 工作流 phase 2。在某个用例 spec 写完 + Superpowers TDD 实施完 + UT 完备**之后**,可选地跑一遍由浅入深的 NFR(非功能性需求)review。采用 NFR-Cards 的 13 张卡片作为提问骨架,LLM 按 per-story 上下文从中筛选相关卡片(5-8 张),逐张三段式追问到量化答案;不接受"系统要快"这种空话,但允许"待定 · 需压测后给"。产出 `docs/bob/04-nfr-<spec-slug>-<date>.md`,末尾出"建议新增 story 清单"——用户视情况手动调 `/bob-stories` 补 story。

> 让 bob 工作流的最后一公里从"TDD 跑绿就交付"变成"TDD 跑绿后,像 TL 一样追问 13 张卡片的 NFR,直到把质量缺口都摆到台面上"。

---

## 1. 背景与现状

### 1.1 现状

到 phase 1.5 为止,bob 工作流是:

```
survey → stories → identify → onion → spec → Superpowers TDD → UT 完备 → ???
```

UT 跑绿之后没有显式的下一步。NFR 散落在:
- `bob-spec` 的"交给 Superpowers 的开放问题"段(主要谈技术栈)
- `bob-onion` ADR 段(可选)
- 现实中,brainstorming / writing-plans 时才正经讨论

结果:**功能性需求**(GWT 场景)有完整链条管,**非功能性需求**(性能 / 安全 / 可观测性 / 并发 / 数据隐私 / 配置 / 兼容性...)没有专门工序。

### 1.2 之前的误解

最早的 phase 2 设计描述是 "NFR = 安全 / 性能 / 秒杀 / 分布式 / 微服务",这其实是 **项目级架构 NFR**——属于 `/bob-onion` 范畴,不是新 skill。

修正后的 NFR 范围是 **per-story 实施完后的技术质量 review**:并发扣减、分布式锁、分库分表、Spring Security、日志脱敏、DB 索引、MyBatis SQL 命中等。

### 1.3 关键约束

- **完全可选**:用户跳过不影响功能完整性,但 `bob-spec` 末尾会有 reminder
- **per-story 触发**:针对单个 spec(或 story)的 NFR,不是项目级一次性
- **NFR 骨架来源**:完全采纳 `NFR-Cards-Summary.md` 的 13 张卡片(Performance / Scalability / Capacity / Reliability / Monitoring / AuthN / AuthZ / Security / Data Privacy / Configurability / Extensibility / Portability / Compatibility)
- **量化优先 + 允许"待定"**:与 NFR-Cards 文档"量化是底线"哲学一致,但允许给不出数字的项标"待定 + 何时能给"
- **不动 ARCHITECTURE.md**:NFR 是 per-story 决策,不写项目 SSoT
- **NFR 缺口不自动 spawn story**:报告末尾出"建议新增 story 清单",用户自己决定调 `/bob-stories`
- **`superpowers-to-trae` 仅作 CLI 风格参考**:不导入任何代码 / 模板

---

## 2. CLI + 触发口

```
/bob-nfr <spec-path>                # 主入口
/bob-nfr --story <story-path>       # 退路:spec 未写时,从 story 拉上下文
/bob-nfr --refresh                  # 已有 04-nfr-*.md 时强制重跑
```

自然语言触发:
- "跑 NFR"
- "质量 review 一下"
- "过一遍 13 张卡"
- "看看有没有遗漏的 NFR"

### 2.1 前置条件

- 必须有 `<spec-path>` 或 `--story <story-path>` 之一(两者都无 → 拒绝)
- 项目在 git 仓库内(用于读 spec / story 等文件)

### 2.2 reminder 落点

`bob-spec` Template A / B / C 的 `## 5. 下一步`(Template A / B)和 `## 5. Guardrails / 下一步`(Template C)段各加 1 行 reminder:

> Superpowers TDD 完成 + UT 完备后,建议跑 `/bob-nfr <本 spec 路径>` 过 13 张 NFR 卡片(可选)。

3 处 reminder,3 处不同 anchor(不同模板)。

---

## 3. 13 张 NFR 卡片(完全采纳 + bob 扩展)

每张卡片在 skill 模板里组织成统一三段结构:

```markdown
### 卡 X · <英文名>(<中文名>)

**核心提问**(NFR-Cards §X):
- <从 NFR-Cards 原文逐字搬>

**思考要点**(浓缩 2-3 句):
- <NFR-Cards 思考要点的浓缩版>

**bob 实施细节追问**:
- <bob 场景下的具体问题>
- ...
```

### 3.1 完整 13 张卡片的 bob 扩展示例

| # | 卡片 | bob 实施细节追问 |
|---|---|---|
| 1 | **Performance** | DB 索引是否命中?MyBatis SQL EXPLAIN 看过?需要缓存吗(本地 / Redis)?需要异步化吗(MQ / Reactive)?P95 / P99 / P999 目标? |
| 2 | **Scalability** | 单点锁是否存在?共享 DB 写瓶颈?水平扩展时新增节点需做哪些适配?USL 拐点估算? |
| 3 | **Capacity** | 表预估行数?日 / 月 / 年增长率?分库分表阈值?MyBatis 分页是 limit-offset 还是游标?Redis 内存 / 连接池上限? |
| 4 | **Reliability** | 该 UseCase 是否幂等(idempotent key 设计)?需要分布式锁吗(选型:Redisson / ZK / DB 行锁)?Saga / 补偿事务?重试 + 退避?断路器? |
| 5 | **Monitoring** | 日志埋点含 traceId / spanId?业务指标(订单数 / 成功率 / GMV)是否上 metric?告警阈值?RED / USE 方法? |
| 6 | **Authentication** | 复用项目 SSO / OAuth / OIDC 还是自己加?需要 MFA / Passkey 吗?token 刷新机制? |
| 7 | **Authorisation** | 谁能调这个 UseCase?网关级 RBAC / UseCase 内 guard / 数据行级?管理员能否绕开?权限维护方? |
| 8 | **Security** | Spring 栈下用 Spring Security 还是 Sa-Token?接口签名校验(HMAC / JWT)?HTTPS / mTLS?SQL 注入 / XSS 防护? |
| 9 | **Data Privacy** | 日志是否脱敏(手机号 / 身份证 / 卡号)?接口返回值脱敏?DB 字段加密(AES / GPG)?测试环境用脱敏数据? |
| 10 | **Configurability** | 哪些参数运行时可调(超时 / 重试次数 / 限流阈值)?用配置中心(Nacos / Apollo / Consul)?特性开关(LaunchDarkly)? |
| 11 | **Extensibility** | 这个 UseCase 是否需要给外部插件钩子?Webhook?事件订阅?SDK? |
| 12 | **Portability** | DB 切换的概率(信创场景:Oracle → 达梦)?写原生 SQL 还是 JPA 抽象?OS 锁定的特性(io_uring 等)? |
| 13 | **Compatibility** | 要兼容老接口吗?数据格式标准(ISO / FHIR / HL7)?向前 / 向后兼容性?API 版本策略? |

### 3.2 卡片相关性的常见分布

经验法则(一个典型 feature spec 跑 NFR 时,大概率相关的卡片):

| 类型 | 常相关卡片 |
|---|---|
| 写操作 + 状态变更 UseCase(取消 / 审批 / 退款) | 1 Performance / 4 Reliability / 5 Monitoring / 9 Data Privacy(若涉及个人信息) |
| 查询 UseCase | 1 Performance / 3 Capacity / 5 Monitoring |
| 鉴权相关 UseCase | 6 AuthN / 7 AuthZ / 8 Security |
| 跨系统集成 UseCase | 8 Security / 13 Compatibility / 5 Monitoring |
| 项目首次跑 | 全部 13 张 |
| refactor story | 大多数 NFR 已在原版本固化 → 通常 0-2 张相关 |

LLM 应在 Stage 2 自动筛选,不要强制问 13 张。

---

## 4. 工作流(5 Stage)

```
Stage 0. 读 spec(或 story)+ 自动定位上下文
Stage 1. 三段式追问"要不要跑 NFR?"(用户立即说"不需要" → skill 退出)
Stage 2. LLM 筛选相关卡片(三段式确认)
Stage 3. 逐张卡片由浅入深三段式追问(核心提问 → 思考要点 → bob 细节 → 量化收口)
Stage 4. 写报告 + "建议新增 story 清单"
```

### 4.1 Stage 0 输入归并

读取以下输入,按优先级:

1. `<spec-path>` 参数 → 读 docs/specs/spec-*.md
2. 或 `--story <story-path>` → 读 docs/bob/02-stories/*.md
3. 自动定位 spec 里的:
   - GWT 场景(用于推断 4 Reliability / 5 Monitoring 的相关性)
   - 涉及 Entity / Port / UseCase 类名
   - "交给 Superpowers 的开放问题"段(技术栈线索 → 6 AuthN / 8 Security 卡片相关性)
   - 是否含并发场景 → 触发 1 Performance / 4 Reliability

### 4.2 Stage 1 入口确认

三段式追问:

> **Q0: 这个 spec 要不要跑 NFR review?**
>
> **推测**:建议跑。spec 涉及订单写操作 + 并发不超卖场景,有 Performance / Reliability / Data Privacy 三个常见 NFR 维度值得过一遍。
> **理由**:bob TL 风:实施完代码就交付,容易把 NFR 推到生产事故才发现。10-15 分钟过 13 张卡里相关的几张,投入产出比高。
> **推荐选择**:`跑`
>
> 是否同意?(回"是"进入 Stage 2;回"不需要" → skill 退出,留 docs/bob/04-nfr-<slug>-<date>.md 写"用户选择跳过 NFR review")

### 4.3 Stage 2 卡片筛选

LLM 从 13 张中挑出相关的,三段式确认:

> **Q1: 这个 spec 跑哪几张卡片?**
>
> **推测**:5 张。1 Performance / 3 Capacity / 4 Reliability / 5 Monitoring / 9 Data Privacy。
> **理由**:GWT 场景 4 是"事务回滚"、场景 5 是"并发不超卖" → 触发 Performance + Reliability;订单数据 → Data Privacy;余下 8 张要么继承项目级(2/6/8/12)、要么跟当前 story 不沾(11/13)、要么基础设施已定(10)。
> **推荐选择**:`5 张:1, 3, 4, 5, 9`
>
> 是否同意?(回"是"走推荐;回"否,加 7 Authorisation"调整;回"否,只跑 1 + 4"切到 2 张)

筛掉的卡片在报告 §4 列出 + 简短理由。

### 4.4 Stage 3 逐张追问(由浅入深 + 量化收口)

对每张相关卡片 X:

#### Step 3.1 核心提问(NFR-Cards 原文 1-2 问)

直接搬 NFR-Cards 卡片原文。

#### Step 3.2 思考要点(简述)

LLM 提示用户思考维度,不强制回答。

#### Step 3.3 bob 实施细节追问(逐项,可拆)

从 §3.1 表里挑该卡片对应的 bob 追问,每条三段式过一次,用户可"否,我不在意"跳过个别细节。

#### Step 3.4 量化收口

每张卡片至少要追问一个**可量化指标**。LLM 模板:

> **追问(直到给出数字)**:
> - 你希望 cancelOrder 的 P95 是多少毫秒?
>   - 用户给"快" → 反问 "用户能接受 100ms / 500ms / 2s 中哪一档?"
>   - 用户给"200ms" → ✓ 记录
> - 系统峰值 QPS?
>   - 用户不知道 → 标 `⚠ 待定 · 需压测后确定` + follow-up:在 §5 建议新增 story 清单 加 "为 cancelOrder 压测出 P95/QPS 基线"

**连续 2 次给不出数字 → 标 `⚠ 待量化`,不阻断流程**。

### 4.5 Stage 4 写报告

详细模板见 §5。

---

## 5. 报告产出 `docs/bob/04-nfr-<spec-slug>-<YYYYMMDD>.md`

`<spec-slug>` 由 spec 文件名 / 一行话生成(3-5 个汉字 / 英文 kebab)。

模板:

```markdown
# NFR Review · <spec 一行话>
日期 · <YYYY-MM-DD> · spec 路径 · docs/specs/spec-3-cancel-order.md · 跑过 N 张卡

## 1. 上下文(3 行)
- 用户 / 系统角色:管理员
- 主要 UseCase:CancelOrderUseCase
- 关键场景:并发不超卖、事务回滚、状态机迁移 SHIPPED → CANCELLED

## 2. 卡片筛选
**跑了 5 张**:1 Performance / 3 Capacity / 4 Reliability / 5 Monitoring / 9 Data Privacy
**跳过 8 张**:
- 2 Scalability — 继承项目级(ARCHITECTURE.md §6)
- 6 Authentication — 公司 SSO 已对接
- 7 Authorisation — 网关层已配
- 8 Security — 项目级 TLS / mTLS 已定
- 10 Configurability — 暂无运行时可调项
- 11 Extensibility — 当前 UseCase 不开放扩展
- 12 Portability — 不切换 DB
- 13 Compatibility — 内部 API,无版本约束

## 3. 卡片明细

### 卡 1 · Performance
- **量化目标**:P95 < 200ms · QPS 峰值 500 · ⚠ 待量化(QPS 需压测后给基线)
- **决策**:
  - 加 Redis 缓存 OrderRepository.findById(TTL 60s,失效策略:写时淘汰)
  - MyBatis SQL 加 status 索引(已确认 EXPLAIN 命中)
- **实施建议**:
  - SQL migration:`sql/V_3_2__add_status_idx_on_order.sql`
  - OrderRepository.findById 改 `@Cacheable("order")`

### 卡 3 · Capacity
- **量化目标**:日订单 100w · 表行数 10y 内 < 40 亿,可控
- **决策**:暂不分库分表;分页改游标
- **实施建议**:OrderRepository.findRecent 用 `WHERE id > <lastId> LIMIT 50`

### 卡 4 · Reliability
- **量化目标**:幂等键 = clientRequestId · 重试 3 次退避 1s / 5s / 30s · ⚠ 待量化(SLO 目标 ?)
- **决策**:
  - 加 Redisson 分布式锁 `lock:order:{orderId}`,TTL 5s
  - cancelOrder 落库时检查 clientRequestId 幂等
- **实施建议**:
  - OrderUseCaseConfig 加 `@Bean` redissonClient
  - CancelOrderUseCase 包装 LockGuardDecorator

### 卡 5 · Monitoring
- **量化目标**:traceId 100% 覆盖;orderCancelRate 业务 metric / orderCancelP95 技术 metric
- **决策**:
  - 业务指标:`order_cancel_total{result=success|failure}` Counter
  - 技术指标:`order_cancel_latency_p95` Histogram
- **实施建议**:Micrometer + Prometheus + Grafana 看板

### 卡 9 · Data Privacy
- **量化目标**:日志中 cancelComment / 手机号 / 卡号 全脱敏
- **决策**:
  - 脱敏方案:`MaskingLayout` 替换关键字段
  - DB:cancelComment 不加密(非敏感),不变更
- **实施建议**:logback-mask.xml 加规则;CancelOrderTest 加日志脱敏 case

## 4. 建议新增 story 清单
> 用户视情况手动调 `/bob-stories <这些需求>` 补 story。

- **N1** · 加 cancelOrder Redisson 分布式锁(优先级:High)
  - 详细需求:Redisson + lock:order:{orderId} + TTL 5s + LockGuardDecorator 包装
- **N2** · 为 cancelOrder 压测出 P95 / QPS 基线(优先级:Medium)
  - 详细需求:JMeter / k6 脚本,模拟 100/500/1000 QPS,P95 < 200ms 验证
- **N3** · 加 OrderRepository.findById 缓存层(优先级:Medium)
  - 详细需求:Redis @Cacheable + 写时淘汰 + 缓存命中 metric
- **N4** · 加日志脱敏 MaskingLayout(优先级:High)
  - 详细需求:logback-mask.xml 配置 + 测试 case

## 5. 待办 / 待量化
- 卡 1 Performance:QPS 峰值阈值 待压测后给(关联 N2)
- 卡 4 Reliability:SLO 目标 (99.9 / 99.99) 待业务方确认

## 6. 跳过的卡片(用户选择)
(无)
```

---

## 6. 与现有 skill 集成

### 6.1 bob-spec 微改

3 处加 reminder(Template A / B / C 的 §5 段)。详细 anchor 在 plan 阶段确定。

reminder 文本统一:

> Superpowers TDD 完成 + UT 完备后,建议跑 `/bob-nfr <本 spec 路径>` 过 13 张 NFR 卡片(可选)。

### 6.2 其他 skills

不改动:`bob-survey` / `bob-stories` / `bob-identify` / `bob-onion`。

### 6.3 HARNESS_ASSETS 注册

新加 1 项:

```rust
Asset {
    rel_path: &[".claude", "skills", "bob-nfr", "SKILL.md"],
    content: include_str!("templates/skills/bob-nfr.md"),
    category: Category::Skill,
    included_in_minimal: true,
    upgrade_safe: true,
},
```

自动走 `init` / `status` / `upgrade` 全流水线。`upgrade_safe_field_matches_category_policy` 自动满足。`init_minimal_skips_archunit_and_shared_and_anchors` 测试需扩 list 加 bob-nfr。

---

## 7. 测试(token-only)

继续 phase 0/1/1.5 风格:

| 测试名 | 断言 token |
|---|---|
| `init_creates_bob_nfr_skill` | `name: bob-nfr` / 13 张卡片名(中英) / `Stage 0` ... `Stage 4` / `量化优先` / `允许待定` / `建议新增 story 清单` / `三段式` / `推测` / `推荐选择` |
| `bob_spec_mentions_nfr_reminder` | bob-spec.md 各模板的 §5 段含 `/bob-nfr` |

`init_minimal_skips_archunit_and_shared_and_anchors` 测试扩 `["bob-identify", "bob-onion", "bob-spec", "bob-survey", "bob-stories", "bob-nfr"]`。

Fixture / LLM 行为验证延后,沿用前面阶段。

---

## 8. 决策记录

| 维度 | 决策 |
|---|---|
| skill 范围 | per-story 实施后 NFR review,完全可选 |
| 触发位置 | spec → Superpowers TDD → UT 完备后 |
| reminder 落点 | bob-spec Template A/B/C 的 §5 段 |
| 输入 | `<spec-path>` 优先,`--story <story-path>` 备份 |
| NFR 骨架 | 完全采纳 NFR-Cards 13 张 + bob 实施细节追问 |
| 上下文筛选 | LLM 从 spec / story 选 5-8 张相关卡片(三段式确认) |
| 量化强度 | 量化优先 + 允许"待定"(连续 2 次给不出 → ⚠ 标记,不阻断) |
| 产出 | `docs/bob/04-nfr-<spec-slug>-<date>.md` 单文件 |
| ARCHITECTURE.md 改动 | 无 |
| NFR 缺口落地 | 报告 §4 出"建议新增 story 清单",不自动 spawn |
| HARNESS_ASSETS 改动 | 加 1 项 bob-nfr.md(upgrade_safe + included_in_minimal) |
| 测试 | token + 文件落位 |

---

## 9. 实施草图(供 writing-plans 起步)

预计 3 个 task + 1 个可选:

1. **bob-nfr 新模板**:创建 `src/templates/skills/bob-nfr.md`(13 张卡片 + 5 Stage + 量化规则 + 报告模板),注册到 `HARNESS_ASSETS`,扩 minimal 测试,加 `init_creates_bob_nfr_skill` token 测
2. **bob-spec 微改**:3 处 §5 段加 NFR reminder,加 `bob_spec_mentions_nfr_reminder` token 测
3. (可选)README 把"five skills" → "six skills",加 `/bob-nfr` 段
4. (可选)final smoke + push

详细拆解由 `writing-plans` 出。

---

## 10. 转交

设计定稿后转交 `superpowers:writing-plans`,产出可执行实施计划。
