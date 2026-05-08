# run-bob · Bob 4 环 Clean Architecture Harness 设计

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-08
> 适用仓库:`/Users/xiaojin/workspace/run-bob`
> 参考资料:
> - `/Users/xiaojin/Downloads/代码项目/atlas/atlas-output/BOB大叔的架构整洁之道这本书的精髓-20260505/架构整洁之道-atlas.md`(尤其 Stage 5 §4 三个落地动作 + Stage 6 Bob vs DDD 对照)
> - `/Users/xiaojin/workspace/ddd-run/`(工程形态对偶参考)

---

## 0. 总览

### 0.1 项目目标

`run-bob` 是一个 Rust CLI,通过 `init` 命令向目标项目注入一组 Claude Code skill 与锚点文档,把 Claude Code / Cursor 等 AI 编码工具约束在 **Bob 同心圆 4 环 Clean Architecture** 上产出代码。形态完全对偶 ddd-run(Rust CLI + 模板嵌入 + skill + 锚点文档 + ArchUnit 守卫),内容从 DDD 战术级切换到 Bob 4 环纯派。

**核心立场**:严格遵守 atlas Stage 5 §4 的三个落地动作——

1. **接口位置反转**:business interface 在 `usecase/port`,Adapter 来 implement
2. **框架边界外推**:Use Case 类零 Spring / 零 SLF4J / 零 Jakarta / 零 Lombok
3. **状态机上提**:业务规则在 Entity 内部,不在 Service

`@Transactional` 全工程**有且仅有一处**——`shared.framework.transaction.TransactionalUseCaseDecorator`。

### 0.2 三种入口模式

| 模式 | 场景 | skill 行为 |
|---|---|---|
| **G**(Greenfield) | 全新项目 | 从业务描述 → 4 环架构 → spec |
| **B1**(Brownfield 全量重构) | 已有 α/β 代码,要改成 γ | 身份测试现状 → α→γ 三动作重构计划 → 逐用例 spec |
| **B2**(Brownfield 增量新功能) | 已有项目(legacy α/β)+ 新需求 | **新代码必须 γ,legacy 不动**;新功能在 legacy 中开"清洁孤岛" |

B2 是 run-bob 区别于 ddd-run 的关键差异——任何引用 legacy 的需求必须通过 `usecase/port/` 端口 + `adapter/acl/` ACL 隔离,新 usecase **不允许** import legacy 的 `@Service` 类。

### 0.3 交付形态

用户在任意目录运行 `run-bob init`,即得:

```
your-project/
├── .claude/skills/
│   ├── bob-identify/SKILL.md           # 🔍 身份测试 — 区分核心 vs 配件
│   ├── bob-onion/SKILL.md              # 🧅 4 环架构设计 — 维护 ARCHITECTURE.md
│   └── bob-spec/SKILL.md               # 📝 用例 spec → Superpowers 桥接
├── CLAUDE.md                           # 🛡 项目级硬约束(R0-R12)
├── ARCHITECTURE.md                     # 📘 4 环架构 SSoT(替代 DOMAIN.md)
├── README-RUN-BOB.md                   # 📖 in-project 使用指南
├── docs/
│   ├── bob/                            # identify/onion 中间产物
│   └── specs/                          # bob-spec 输出
└── src/
    ├── main/java/com/example/shared/
    │   ├── usecase/UseCase.java                              # 通用 UseCase<C,R> 接口
    │   └── framework/transaction/
    │       └── TransactionalUseCaseDecorator.java            # 全工程唯一 @Transactional
    └── test/java/architecture/
        └── CleanArchitectureTest.java                        # ArchUnit 守卫
```

### 0.4 工作流

```
   业务需求 / 已有 α 代码 / 新功能需求
            │
   ┌────────▼────────┐  G:从需求抽核心 vs 配件
   │  /bob-identify  │  B1:对每段代码做身份测试,标 α/β/γ 评级
   └────────┬────────┘  B2:对新功能做身份测试 + 标 legacy 复用点
            │           输出: docs/bob/01-identity-<topic>.md
            ▼
   ┌────────▼────────┐  G:画 4 环 + 端口清单 + 状态机 → ARCHITECTURE.md
   │   /bob-onion    │  B1:出 α→γ 三动作重构计划
   └────────┬────────┘  B2:产出清洁孤岛布局 + Legacy ACL 表
            │           输出: ARCHITECTURE.md(SSoT)
            ▼           副作用: 回写 CleanArchitectureTest.java 黑名单
   ┌────────▼────────┐
   │   /bob-spec     │  生成单用例 spec(命令 / 查询 / 重构 三模板)
   └────────┬────────┘  输出: docs/specs/spec-<n>-<slug>.md
            │
            ▼
   superpowers:brainstorming(技术栈 → CLAUDE.md)
            ▼
   superpowers:writing-plans → executing-plans (TDD) → finishing-a-development-branch
```

### 0.5 与 ddd-run 的关键差异

| 维度 | ddd-run | run-bob |
|---|---|---|
| 锚点文档名 | `DOMAIN.md` | `ARCHITECTURE.md` |
| skill 命名 | ddd-storm / ddd-model / ddd-spec | bob-identify / bob-onion / bob-spec |
| 入口模式 | 绿地为主 | G + B1 + B2 三模式 |
| 配件清单机制 | 硬编码黑名单(R9 列举) | **R0 元规则:5 问决策树**;R7-R12 是 R0 的可执行实例 |
| 包命名 | `domain` | `entity`(贴近 atlas 用语) |
| Domain Event | 一等公民(R10) | **默认拒绝**(R10-bob,Bob 单 BC + 同步) |
| shared 骨架 | 仅文档说明 | **直接 ship Java 类**(`UseCase.java` + `TransactionalUseCaseDecorator.java`) |
| ArchUnit 黑名单 | 硬编码 Spring/Jakarta/SLF4J/Lombok | **参数化数组 `FORBIDDEN_IN_INNER`**,可由 `/bob-onion` 回写 |
| 提问风格 | 对话式,部分开放问题 | **强制三段式**:推测 + 理由 + 推荐 |

---

## 1. 核心机制:5 问决策树

技术选型从不收敛——`@Transactional` / SLF4J / Spring 只是 2026 年 Java 生态最常见的三个例子。换到 Kotlin 是 `kotlinx.coroutines.Flow`,换到 Go 是 `gorm.DB`,换到信创栈是达梦驱动 / 东方通 `ApusicContext`。Skill 不能写死黑名单;必须给出**通用判定方法**。

### 1.1 5 问决策树

每个候选对象 / import / 注解 / 类型,顺序问 5 个问题:

```
Q1. 这个东西换掉,产品的业务意义会变吗?(身份测试主问)
    │
    ├── 会变  → CORE  → 进 Entity 或 UseCase
    │
    └── 不会变 → 进 Q2

Q2. 它有没有"副作用"或"运行时依赖"?
    (网络 / 磁盘 / 时钟 / 容器生命周期 / 进程外资源)
    │
    ├── 有  → 进 Q3
    └── 没有 → 它是工具类(JDK util / Apache Commons / Guava)
              → 可以被 Entity/UseCase 直接 import

Q3. 它是"业务概念的翻译者"还是"应用流程的编排者"?
    │
    ├── 翻译者 → ADAPTER  例:JpaOrderRepository、WeChatPaymentAdapter
    └── 编排者 → FRAMEWORK 例:@Configuration、装饰器、main()

Q4. 它在 Entity/UseCase 的方法签名 / 字段类型 / 注解 里出现了吗?
    │
    ├── 是 → 违规!必须立刻提取端口接口(动作 1:接口位置反转)
    │       例:Entity 字段 `private LocalDateTime paidAt = LocalDateTime.now()`
    │           违规 → 抽 ClockPort,now() 在 framework
    │
    └── 否 → 当前合法

Q5. (棕地专用)它出现在 legacy α/β 代码里,但新功能要复用相关业务能力。怎么办?
    │
    └── 不允许 import legacy 的 @Service 类。
        在新 usecase/port 定义业务领域接口,
        在新 adapter/acl 写 adapter 包装 legacy。
```

### 1.2 副作用的精确含义

"副作用"是 Q2 的关键判定点,精确定义:

| 类别 | 含义 | 例子 |
|---|---|---|
| **读外部状态** | 行为依赖进程外世界 | 读数据库、读文件、读环境变量、读系统时钟 |
| **改外部状态** | 行为会让进程外世界变化 | 写库、发 HTTP、推消息、写日志 |
| **非确定性** | 同样输入不一定同样输出 | `LocalDateTime.now()`、`UUID.randomUUID()`、`Math.random()` |
| **依赖容器/框架生命周期** | 必须在某个 runtime 才能跑 | `@PostConstruct`、`ApplicationContextAware`、Servlet 容器 |

反例(无副作用,可直接 import 进 Entity/UseCase):

- `Objects.requireNonNull(x)`
- `Collectors.toList()`
- `Math.max(a, b)`
- `String.format(...)`
- 你自己写的 `Order.canBeCancelled()` 这种只看自身字段做判断的纯方法

### 1.3 标准配件清单(知识库,非黑名单)

ARCHITECTURE.md 与 CLAUDE.md 内置项目特化的配件分类表,引导识别但不闭合:

| 分类 | 典型出现 | 端口抽象 | 落地环 |
|---|---|---|---|
| 日志 | SLF4J / Logback / `@Slf4j` / `LoggerFactory.getLogger` | `LoggerPort` | adapter/logging |
| 事务 | `@Transactional` / `PlatformTransactionManager` | UseCase 装饰器(framework 唯一) | framework/transaction |
| 持久化 | JPA / MyBatis / JdbcTemplate / Spring Data / R2DBC / 达梦驱动 | `<Entity>Repository`(usecase 接口) | adapter/persistence |
| HTTP 出站 | RestTemplate / WebClient / OkHttp / Feign / 微信支付 SDK | 业务领域接口(`PaymentGateway`) | adapter/acl |
| 消息 | Kafka / RabbitMQ / `@EventListener` / `ApplicationEventPublisher` | (Bob 默认拒绝,需 ADR) | adapter/messaging |
| 缓存 | Redis(Lettuce/Jedis) / Caffeine | `CachePort` 或就近端口的 decorator | adapter/cache |
| 时钟 | `LocalDateTime.now()` / `Instant.now()` | `ClockPort` | adapter/time |
| ID 生成 | `UUID.randomUUID()` / Snowflake | `IdGenerator` | adapter/id |
| 序列化 | Jackson `@JsonProperty` / Gson / FastJSON | DTO 在 adapter,Entity 不带注解 | adapter/web 或 adapter/messaging |
| 校验 | Bean Validation `@Valid` / `@NotNull` | DTO 上加,Entity 自校验 throw `IllegalStateException` | adapter/web |
| DI 注解 | `@Autowired` / `@Inject` / `@Component` / `@Service` / `@Repository` | 构造器注入,无注解 | adapter / framework |
| 容器生命周期 | `@PostConstruct` / `ApplicationContextAware` / `InitializingBean` | 不让 UseCase 感知容器 | framework |
| 信创框架 | 东方通 `ApusicContext` / 金蝶 Apusic / 达梦方言 / 人大金仓驱动 | 同持久化 / HTTP | adapter/persistence 或 adapter/acl |
| 配置读取 | `@Value` / `@ConfigurationProperties` / `Environment` | `<Domain>Config` POJO + 端口 | framework 注入 |
| 文件 / IO | `Files.write` / `FileInputStream` | `FilePort` / `BlobStore` | adapter/io |
| **未知库**(用户新引入) | **跑 5 问决策树** | 必要时新建端口 | 由决策树定 |

### 1.4 决策树在三个 skill 的分布

| skill | 用决策树做什么 |
|---|---|
| `/bob-identify` | 把业务描述 / 已有代码里**每一个名词、每一个 import** 跑一遍决策树,产出分类表 |
| `/bob-onion` | 基于 identify 表画 4 环;遇到"配件"逐个回 Q3 决定它进 adapter 还是 framework;遇到"违规"开 α→γ 重构条目;ARCHITECTURE.md §配件清单按本项目特化更新 |
| `/bob-spec` | 用例 spec 内"端口接口"段列出该用例需要的所有端口;Guardrails 要求 Superpowers 实现时**遇到新 import 必须停下跑决策树** |

---

## 2. 全局提问规约

### 2.1 三段式必须

任何需要用户选择 / 决定的问题,**强制**按以下三段式输出。**禁止**抛开放问题。

```markdown
> [问题序号] [问题]
>
> 推测:<你的判断,基于上下文的最优解>
> 理由:<一句话,引用业务描述/代码事实/Bob 原则>
> 推荐选择:<具体一个选项>
>
> 是否同意?(回"是"走推荐;回"否,理由是 ..."重判;回"否,我选 X"切到 X)
```

### 2.2 应用范围

- 决策树每一问
- 4 环包路径选择
- 端口归属选择(usecase/port vs domain)
- 状态机迁移线
- α/β/γ 评级
- 清洁孤岛包路径
- Legacy ACL 接口签名

每个 skill 的 SKILL.md 头部都必须含此段提问规约,并在工作流的每个分支点示范一次。

---

## 3. skill 设计 · `/bob-identify`

### 3.1 触发

```
/bob-identify <业务描述>           # 模式 G
/bob-identify --refactor [path]    # 模式 B1
/bob-identify <新功能描述>         # 模式 B2(auto-detect:已有 src/main/java + 用户描述新功能)
```

自然语言:"做身份测试"、"区分核心和配件"、"这段代码哪些是核心哪些是框架"、"这个功能里什么是 Entity"。

### 3.2 目标

把每一个候选概念 / 类 / import / 注解都跑过 5 问决策树,产出分类表。**不画架构、不写代码、不出 spec**——只回答一个问题:**这个东西到底是什么?**

### 3.3 工作流

#### 模式 G(绿地)

- **G1**:复述业务描述
- **G2**:列出所有名词 + 动词
- **G3**:对每项**对话式**跑决策树(三段式提问)
- **G4**:对每个"动作 / 副作用"主动追问外部对接形态(支付通道?消息渠道?)
- **G5**:产出 `docs/bob/01-identity-<topic>.md`

#### 模式 B1(棕地全量重构)

- **B1.1**:扫描 `src/main/java`,分析包结构 / import / 注解 / 关键方法签名
- **B1.2**:对每个类跑决策树,标 4 个评级:
  - **γ** 已合规 → 跳过
  - **β** 包结构对了但 usecase 还碰框架 → 列入"框架边界外推"
  - **α** 业务规则散落 / 状态机在 Service / interface 在外层 → 列入"接口位置反转 + 状态机上提"
  - **violation** 严重违反(Entity 上有 `@Entity`、`@Slf4j`)→ 优先重构
- **B1.3**:产出文档,含"α/β/γ 评级分布"+ "重构优先级"

#### 模式 B2(棕地增量新功能)

- **B2.1**:**重要追问**——是否有 legacy 模块需要复用?
- **B2.2**:对新功能跑模式 G 流程
- **B2.3**:执行**清洁孤岛规则**——legacy 不动;在新 usecase/port 定义业务语言接口;在新 adapter/acl 写 adapter 包装
- **B2.4**:产出文档,含"清洁孤岛包路径"+"Legacy 隔离接口清单"+"ArchUnit 作用域提示"

### 3.4 产出文档模板

```markdown
# 身份测试:<主题>

> 模式:G / B1 / B2
> 由 /bob-identify 生成。所有"配件"必须在后续 /bob-onion 阶段被分配到 adapter 或 framework 环;
> 所有"违规"必须在 /bob-onion 给出重构方案。

## 1. 业务描述复述

## 2. 5 问决策树分类表

| # | 候选元素 | Q1 业务意义会变? | Q2 有副作用? | Q3 翻译者还是编排者? | Q4 出现在 inner 包? | 分类 | 建议端口名 | 备注 |
|---|---|---|---|---|---|---|---|---|
| 1 | Order | 会变 | — | — | — | CORE/Entity | — | 状态机:Created→Paid→Shipped |
| 2 | 微信支付 SDK | 不会变 | 有(网络) | 翻译者 | 否 | ADAPTER/acl | PaymentGateway | 业务接口在 usecase |
| 3 | SLF4J Logger | 不会变 | 有(磁盘) | 翻译者 | 是 ⚠️ | 违规 | LoggerPort | 当前在 OrderUseCase 必须删 |
| ... | | | | | | | | |

## 3. α/β/γ 评级(仅模式 B1)

## 4. 清洁孤岛 / Legacy 隔离(仅模式 B2)

| Legacy 类 | 业务领域接口(新建) | Adapter 实现(新建) |
|---|---|---|
| `LegacyOrderService.fetchByCustomer` | `usecase/port/CustomerOrderHistory` | `adapter/acl/LegacyCustomerOrderAdapter` |

## 5. 配件清单回写建议(给 /bob-onion 用)

本次新识别出的配件包(应禁止于 inner 包):
- `io.dameng..`
- `com.alibaba.fastjson..`
→ /bob-onion 回写到 CleanArchitectureTest.java 的 FORBIDDEN_IN_INNER

## 6. 开放问题

## 7. 下一步
> 运行 `/bob-onion` 基于本表画 4 环架构。
```

### 3.5 反模式

- ❌ 直接给"分类结果"而不跑决策树
- ❌ 闭目使用 R7-R12 黑名单做分类(R0 优先)
- ❌ 在 identify 阶段画 4 环 / 写代码
- ❌ 模式 G 假设业务里没有副作用
- ❌ 模式 B2 不主动追问 legacy 复用

### 3.6 文件落位

`docs/bob/01-identity-<topic-slug>.md`

---

## 4. skill 设计 · `/bob-onion`

### 4.1 触发

```
/bob-onion                          # 默认:读最新 docs/bob/01-identity-*.md
/bob-onion --identity <path>        # 指定 identity 文档
/bob-onion --refresh                # 跳过 identity,基于现有 ARCHITECTURE.md 增补
```

自然语言:"画 4 环架构"、"设计端口"、"出重构计划"、"画洋葱图"。

### 4.2 前置条件

`docs/bob/01-identity-*.md` 至少存在一份(否则提示先跑 `/bob-identify`)。例外:`--refresh` 模式只增补 ARCHITECTURE.md。

### 4.3 工作流

- **O1**:读 identity 表,核对配件清单
- **O2**:决定本上下文包结构(三模式分支):
  - G:`com.example.<bizname>` 单一上下文
  - B1:沿用现有顶级包,逐步淘汰 legacy
  - B2:`com.example.<feature>` 清洁孤岛
- **O3**:画 4 环 + 端口清单(对每个 UseCase 候选问需要哪些端口)
- **O4**:Entity 状态机上提(对每个有状态字段的 Entity 推导状态机)
- **O5**:决定装饰器边界——`UseCase<C, R>` 接口 + `TransactionalUseCaseDecorator`
- **O6**:回写 ArchUnit 黑名单 + ARCHITECTURE.md §配件清单
- **O7**(B1):产出 α→γ 重构计划(每条对应一个独立 spec)
- **O8**(B2):产出清洁孤岛布局 + Legacy ACL 表 + ArchUnit 作用域行
- **O9**:更新 ARCHITECTURE.md(追加 ADR,不覆盖)

### 4.4 ARCHITECTURE.md 模板

```markdown
# 架构(Bob 4 环)· <项目/上下文名>

> 本文档是本项目 Bob 4 环架构的 Single Source of Truth。
> 所有 Superpowers spec、代码命名、测试描述必须使用本文档定义的端口名 / Entity 名 / 状态名。
> 本文件由 /bob-onion 管理。

## 📌 状态
- 模式:**<G | B1 | B2>**
- [ ] 已完成身份测试(`/bob-identify`)
- [ ] 已完成 4 环设计(`/bob-onion`)
- [ ] 已生成至少一个 spec(`/bob-spec`)
- [ ] 已有代码实现(Superpowers)

## 1. 上下文(Context)
- 名称、职责、不负责

## 2. 4 环包结构

```
com.example.<bizname>/
├── entity/        Ring 1 — 实体 + 业务规则      (零框架)
├── usecase/       Ring 2 — Interactor + port/  (零框架)
├── adapter/       Ring 3 — Controller / Repo impl (允许 Spring/JPA/SDK)
└── framework/     Ring 4 — 装配 + 事务装饰器 + main
```

## 3. 核心 Entity 与状态机

### 3.x <Entity 名>
- 字段
- 状态机图
- 核心方法 + 前置/后置条件
- 不变量

## 4. 端口清单(usecase/port/)

| 端口名 | 签名摘要 | Adapter 实现 | 落位包 |
|---|---|---|---|
| OrderRepository | findById / save | JpaOrderRepository | adapter/persistence |
| PaymentGateway | pay / refund | WeChatPaymentAdapter | adapter/acl |
| ClockPort | now() | SystemClockAdapter | adapter/time |
| LoggerPort | info / error | Slf4jLoggerAdapter | adapter/logging |

## 5. UseCase 清单

| UseCase | Command record | Result record | 用到的端口 |
|---|---|---|---|
| PayOrderUseCase | PayOrderCommand | PayOrderResult | OrderRepo, PaymentGW, Inventory, Sms |

## 6. 配件清单(项目特化)

| 配件 | 根包 | 端口抽象 | 落位环 |
|---|---|---|---|
| Spring | org.springframework.. | (注解模式) | adapter / framework |
| SLF4J | org.slf4j.. | LoggerPort | adapter/logging |
| 达梦驱动 | io.dameng.. | (复用 OrderRepository) | adapter/persistence |

## 7. 装配点(framework/)

- TransactionalUseCaseDecorator:全工程唯一 @Transactional
- <Feature>UseCaseConfig:每个上下文一个,@Bean + 装饰器包裹

## 8. α/β/γ 评级与重构计划(仅 B1/B2)

## 9. ArchUnit 作用域

```java
// G/B1 默认:整工程作用域(覆盖业务包 + shared)
@AnalyzeClasses(packages = "com.example", importOptions = DoNotIncludeTests.class)

// B2 清洁孤岛:必须把 shared 加进数组,否则装饰器规则评估不到
// @AnalyzeClasses(packages = {"com.example.<feature>", "com.example.shared"},
//                 importOptions = DoNotIncludeTests.class)
```

## 10. ADR(架构决策记录)

### ADR-1:UseCase 用 `UseCase<C, R>` 接口 + 装饰器
### ADR-2:端口接口归属 usecase/port,不放 domain

## 11. 下一步
```

### 4.5 反模式

- ❌ 跳过 `/bob-identify` 直接画 4 环
- ❌ 把端口接口放 entity 包(纯 Bob 应放 usecase/port)
- ❌ 装饰器之外允许 `@Transactional`
- ❌ ARCHITECTURE.md 出现 framework 类型(`HttpServletRequest` 等)
- ❌ B2 不做 Legacy ACL,让新 usecase 直接 import legacy `@Service`

### 4.6 文件落位

- 设计过程记录:`docs/bob/02-onion-<topic>.md`
- SSoT:**更新 `ARCHITECTURE.md`**
- 副作用:**追加** `CleanArchitectureTest.java` 的 `FORBIDDEN_IN_INNER` 数组

---

## 5. skill 设计 · `/bob-spec`

### 5.1 触发

```
/bob-spec <用例名>                  # 默认:命令型
/bob-spec --query <查询名>          # 查询型(读模型)
/bob-spec --refactor <类名>         # B1 重构型
```

自然语言:"生成 spec"、"出 TDD 测试场景"、"准备给 Superpowers 的输入"。

### 5.2 前置条件

`ARCHITECTURE.md` §4 端口清单 + §5 UseCase 清单已填(否则提示先跑 `/bob-onion`)。

### 5.3 三类适用范围

| 类型 | 触发词 | 模板 |
|---|---|---|
| 命令(Command) | Pay/Cancel/Apply/Submit/Refund 等 | 模板 A(状态变化 + 事务装饰器) |
| 查询(Query) | View/List/Find/Browse/Get 等 | 模板 B(读模型) |
| 重构(Refactor) | `--refactor <类>` | 模板 C(α→γ 改造路径) |

### 5.4 工作流

- **S1**:读 ARCHITECTURE.md 定位用例归属 → 推荐 spec 类型
- **S2**:跨 Entity 检测(若涉多 Entity,拒绝引入 Domain Event,提醒 Bob 单 BC + 同步)
- **S3**:渲染对应模板(A / B / C)
- **S4**:回写检测——若 spec 暴露未登记端口,**停下**让用户回 `/bob-onion --refresh`

### 5.5 模板 A:命令型 spec

```markdown
# Spec: <用例名>

> 由 /bob-spec 生成。术语锚定到 ARCHITECTURE.md。
> 实现交给 Superpowers TDD 流程。
> 模式:<G | B1 | B2>

## 1. 归属
- Entity:Order(ARCHITECTURE.md §3.1)
- 状态迁移:CREATED → PAID
- 包路径:com.example.<bizname>.usecase.PayOrderUseCase

## 2. 用例描述

## 3. 参与者

## 4. 前置条件

## 5. 后置条件(成功路径)

## 6. 业务规则(Entity 不变量)
- INV-1(来自 ARCHITECTURE.md §3.1):...
- RULE-本用例-1:...

## 7. 测试场景(Given-When-Then)
### 场景 1:成功路径
### 场景 2:状态非法
### 场景 3:端口失败
### 场景 4:事务回滚

## 8. 接口约定

### Command(usecase 层 record,纯 Java)
```java
package com.example.<bizname>.usecase;
public record PayOrderCommand(String orderId) {}
```

### Result(usecase 层 record)
```java
public record PayOrderResult(String orderId, String status, String paidAt) {}
```

### UseCase 实现(usecase 层 POJO,零 Spring,零 SLF4J)
```java
package com.example.<bizname>.usecase;
import com.example.<bizname>.entity.*;
import com.example.<bizname>.usecase.port.*;
import com.example.shared.usecase.UseCase;
// 严禁 import org.springframework.* / jakarta.* / org.slf4j.* / lombok.*

public class PayOrderUseCase implements UseCase<PayOrderCommand, PayOrderResult> {
    private final OrderRepository repo;
    private final PaymentGateway paymentGateway;
    private final InventoryClient inventoryClient;
    private final SmsNotifier smsNotifier;

    public PayOrderUseCase(OrderRepository repo, PaymentGateway pg,
                           InventoryClient ic, SmsNotifier sn) {
        this.repo = repo;
        this.paymentGateway = pg;
        this.inventoryClient = ic;
        this.smsNotifier = sn;
    }

    @Override
    public PayOrderResult execute(PayOrderCommand cmd) {
        Order order = repo.findById(OrderId.of(cmd.orderId())).orElseThrow();
        order.payTo(paymentGateway, inventoryClient);
        smsNotifier.notifyOrderPaid(order.userPhone(), order.orderNo());
        repo.save(order);
        return new PayOrderResult(
            order.id().value(),
            order.status().name(),
            order.paidAt().toString()
        );
    }
}
```

### Entity 方法(状态机)
```java
public void payTo(PaymentGateway pg, InventoryClient ic) {
    ensureStatus(OrderStatus.CREATED, "已支付/已取消订单不能再支付");
    pg.pay(this.amount, this.userId);
    ic.decrease(this.productId, this.qty);
    this.status = OrderStatus.PAID;
    this.paidAt = clock.now();    // 注入 ClockPort
}
```

### 装配(framework 层,唯一事务点)
```java
@Configuration
class OrderUseCaseConfig {
    @Bean
    UseCase<PayOrderCommand, PayOrderResult> payOrderUseCase(
            OrderRepository repo, PaymentGateway pg,
            InventoryClient ic, SmsNotifier sn) {
        return new TransactionalUseCaseDecorator<>(
            new PayOrderUseCase(repo, pg, ic, sn));
    }
}
```

### Controller(adapter/web,只 import usecase 包)
```java
@RestController
class OrderController {
    private final UseCase<PayOrderCommand, PayOrderResult> payOrder;
    OrderController(UseCase<PayOrderCommand, PayOrderResult> payOrder) {
        this.payOrder = payOrder;
    }
    @PostMapping("/orders/{id}/pay")
    PayOrderResult pay(@PathVariable String id) {
        return payOrder.execute(new PayOrderCommand(id));
    }
}
```

## 9. Guardrails(给 Superpowers)

参考 CLAUDE.md R0 / R7-R12:
- ❌ 业务规则不得写进 Controller / UseCase 方法体(必须在 Entity)
- ❌ UseCase 类不得 import org.springframework.* / jakarta.* / org.slf4j.* / lombok.*
- ❌ UseCase 类不得加任何注解
- ❌ UseCase 不得调 LoggerFactory.getLogger(用 LoggerPort)
- ❌ Entity 不得 LocalDateTime.now()(用 ClockPort)
- ❌ @Transactional 必须仅在 TransactionalUseCaseDecorator
- ❌ Repository 实现不得放在 framework/(应在 adapter/persistence/)
- ✅ TDD 节奏:先红再绿再重构
- ✅ 每个 UseCase 写完后自检:
  - `grep -rE "org\.springframework|jakarta\.|org\.slf4j|lombok\." src/main/java/.../usecase/` 期望零命中
  - `mvn test -Dtest=CleanArchitectureTest` 期望全绿
- ✅ 遇到任何新 import → 停下跑 5 问决策树

## 10. 交给 Superpowers 的开放问题

留给 superpowers:brainstorming 在进入 writing-plans 之前回答,写回 CLAUDE.md ## 技术栈约定:
- 语言/运行时
- 应用框架
- 持久化
- 范围(仅后端 / 全栈)
- 对外交互形态
- 测试框架 / 构建工具
- 部署形态
- 非功能约束

## 11. 下一步
1. superpowers:brainstorming(首次)
2. superpowers:writing-plans
3. superpowers:executing-plans + TDD
4. superpowers:finishing-a-development-branch
```

### 5.6 模板 B:查询型(简化)

仅在跨上下文联合 / 投影 / 分页 / 独立 Read Model 表 时使用。简单 findById + DTO 不需要 spec。

```markdown
# Query Spec: <查询名>

## 归属
- 类型:读模型查询(无状态变化)
- 实现方式:<单 Entity 直查 / 跨上下文联合 / 绕端口 SQL / 独立 Read Model 表>

## 输入(Query Params)
## 输出(DTO)
## 约束(权限 / 分页 / 一致性)
## 测试场景
## 接口约定(REST + Application 入口签名)
## 禁止项
- ❌ 在查询路径里改状态
- ❌ 查询 DTO 泄露 Entity 内部可变对象
- ❌ 为优化绕过 Entity 时仍拼装业务规则
- ✅ 查询可直接 SQL,但 DTO 字段必须在 ARCHITECTURE.md §6 可追溯
```

### 5.7 模板 C:重构型(B1 专用)

针对一个 α/β 类,产出"现状 → 目标 → 三动作步骤 → TDD 覆盖"。

```markdown
# Refactor Spec: <类名>

## 1. 现状(α/β)
- 类、评级、关键违规

## 2. 目标(γ)
- 包重定向
- 3 动作落点(接口反转 / 边界外推 / 状态机上提)

## 3. 改造步骤(原子化,每步一次提交)

### Step 1:加测试覆盖现状(防止改坏)
### Step 2:抽端口
### Step 3:状态机上提到 Entity
### Step 4:框架边界外推 + 装饰器装配
### Step 5:删除 legacy Service

## 4. 测试场景(GWT)
- 场景 5:回归基线(refactor = 不改业务的代码迁移)

## 5. Guardrails / 下一步
- B1 模式重构每完成一步,必须 git commit;严禁批量合并
```

### 5.8 反模式

- ❌ spec 出现 ARCHITECTURE.md 之外的端口名 / Entity 名(必须先 `/bob-onion --refresh`)
- ❌ 命令 spec 引入 Domain Event(那是 DDD)
- ❌ 只有 happy path 没有失败路径
- ❌ 用技术语言("调用 API"、"写入 MySQL")替代业务语言
- ❌ 简单 GET 接口强行跑 spec
- ❌ 重构 spec 把多个动作合并一步

### 5.9 文件落位

`docs/specs/spec-<自增序号>-<slug>.md`

---

## 6. CLAUDE.md 模板:R0-R12

跟 ddd-run R1-R11 的关系:

| 规则 | 状态 | 内容(差异) |
|---|---|---|
| **R0** | **全新** | 通用判定优先于具体清单(5 问决策树是 R7-R12 的母规则) |
| **R1** | 改 | 战略层先行。步骤改为:`/bob-identify` → `/bob-onion`(更新 ARCHITECTURE.md)→ `/bob-spec` → superpowers |
| **R2** | 改 | 术语一致性。锚点从 DOMAIN.md → ARCHITECTURE.md §3 §4 §5 |
| **R3** | 改 | 富 Entity 模型 + UseCase 编排,禁止贫血。补充 atlas Stage 5 §4.1 的 `order.payTo(pg, ic)` 标准范式 |
| **R4** | 删 | ~~聚合边界~~ — DDD 概念,Bob 不强调。**用 R4-bob 替代**:Entity 状态迁移必须自封(状态机上提) |
| **R5** | 改 | Repository 端口名格式:`<EntityName>Repository`(不再叫"聚合根 Repository") |
| **R6** | = | TDD 节奏(由 Superpowers 执行) |
| **R7** | 改 | 包结构 4 环。包名:`entity / usecase / adapter / framework`(贴 atlas) |
| **R8** | = | 装配规则(`@Transactional` 唯一在 `TransactionalUseCaseDecorator`) |
| **R9** | 扩 | 反模式硬清单。新增条目:① Entity 不得 `LocalDateTime.now()` / `UUID.randomUUID()`(用 ClockPort / IdGenerator);② 决策树判为配件的库一律不得在 inner 包;③ B2:新功能不得 import legacy 的 `@Service` 类 |
| **R10** | 改 | ~~Domain Event 边界~~。改为 **R10-bob:跨上下文 / 异步 = 升级触发器**——出现该需求时停下,记 ADR,提示是否升级到 DDD 战术级,默认不引入 Outbox / `@EventListener` |
| **R11** | = | ArchUnit 守卫 |
| **R12** | **全新** | 增量新功能(B2)必须 γ,即使周围是 α/β legacy。新代码不得"复用" legacy `@Service`;新包内禁止"为兼容 legacy 风格"的妥协;ArchUnit 作用域要包含新包且仅新包 |

---

## 7. ArchUnit 模板(`CleanArchitectureTest.java`)

跟 ddd-run 模板的差异:
1. 包名 `..domain..` → `..entity..`
2. 黑名单参数化 `FORBIDDEN_IN_INNER` 数组
3. 作用域可缩放,B2 必须改

```java
package architecture;

import com.tngtech.archunit.junit.AnalyzeClasses;
import com.tngtech.archunit.junit.ArchTest;
import com.tngtech.archunit.lang.ArchRule;
import com.tngtech.archunit.core.importer.ImportOption.DoNotIncludeTests;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Repository;
import org.springframework.transaction.annotation.Transactional;

import static com.tngtech.archunit.lang.syntax.ArchRuleDefinition.*;
import static com.tngtech.archunit.library.Architectures.layeredArchitecture;

/**
 * Bob 4 环 Clean Architecture 守卫,由 run-bob init 生成。
 *
 * 维护规则:
 *   - 不要删除已有规则
 *   - 可在文件末尾追加项目特定规则
 *   - 引入新外部库时,跑 ARCHITECTURE.md §"5 问决策树",
 *     若判为配件,把根包加到 FORBIDDEN_IN_INNER
 *   - B2 模式(清洁孤岛)修改 @AnalyzeClasses 的 packages,
 *     只覆盖新功能包,不波及 legacy
 *
 * 调整 base 包:本模板假设 base 是 "com.example",含两个直接子包:
 *   - com.example.shared          (项目级共享:UseCase 接口 + TransactionalUseCaseDecorator)
 *   - com.example.<bizname>       (业务代码:entity / usecase / adapter / framework)
 * 把 "com.example" 替换为你的实际 base 包名。
 *
 * B2 模式(清洁孤岛)用多包数组替换 packages,例:
 *   packages = {"com.example.subscription", "com.example.shared"}
 * ↑ 必须把 shared 加进来,否则 transactional_methods_only_in_decorator 规则评估不到装饰器类。
 */
@AnalyzeClasses(
    packages = "com.example",
    importOptions = DoNotIncludeTests.class
)
public class CleanArchitectureTest {

    private static final String[] FORBIDDEN_IN_INNER = {
        "org.springframework..",
        "jakarta..",
        "org.slf4j..",
        "lombok..",
        // 信创预设(按需取消注释):
        // "io.dameng..",          // 达梦
        // "com.kingbase..",       // 人大金仓
        // "com.tongtech.apusic..",// 东方通
        // 项目自加(由 /bob-onion 回写):
        // "com.alibaba.fastjson..",
        // "redis.clients.jedis..",
    };

    // R7: 4 环依赖方向
    @ArchTest
    static final ArchRule layered_dependencies = layeredArchitecture()
        .consideringAllDependencies()
        .layer("entity").definedBy("..entity..")
        .layer("usecase").definedBy("..usecase..")
        .layer("adapter").definedBy("..adapter..")
        .layer("framework").definedBy("..framework..")
        .whereLayer("framework").mayNotBeAccessedByAnyLayer()
        .whereLayer("adapter").mayOnlyBeAccessedByLayers("framework")
        .whereLayer("usecase").mayOnlyBeAccessedByLayers("framework", "adapter")
        .whereLayer("entity").mayOnlyBeAccessedByLayers("framework", "adapter", "usecase");

    // R0/R3/R9: entity 层纯 Java
    @ArchTest
    static final ArchRule entity_pure_of_frameworks = noClasses()
        .that().resideInAPackage("..entity..")
        .should().dependOnClassesThat().resideInAnyPackage(FORBIDDEN_IN_INNER);

    // R0/R3/R9: usecase 层纯 POJO
    @ArchTest
    static final ArchRule usecase_pure_of_frameworks = noClasses()
        .that().resideInAPackage("..usecase..")
        .should().dependOnClassesThat().resideInAnyPackage(FORBIDDEN_IN_INNER);

    // R8: @Transactional 唯一在装饰器
    @ArchTest
    static final ArchRule transactional_methods_only_in_decorator = methods()
        .that().areAnnotatedWith(Transactional.class)
        .should().beDeclaredInClassesThat().haveFullyQualifiedName(
            "com.example.shared.framework.transaction.TransactionalUseCaseDecorator");

    @ArchTest
    static final ArchRule transactional_classes_only_in_decorator = classes()
        .that().areAnnotatedWith(Transactional.class)
        .should().haveFullyQualifiedName(
            "com.example.shared.framework.transaction.TransactionalUseCaseDecorator");

    // R9: Controller 不得 import entity
    @ArchTest
    static final ArchRule web_controller_no_entity = noClasses()
        .that().resideInAPackage("..adapter.web..")
        .should().dependOnClassesThat().resideInAPackage("..entity..");

    // R5/R7: Repository impl 在 adapter.persistence
    @ArchTest
    static final ArchRule repository_impl_location = classes()
        .that().areAnnotatedWith(Repository.class)
        .should().resideInAPackage("..adapter.persistence..");

    // R10-bob: 防 DDD 漂移(@EventListener 出现 = 风险信号)
    @ArchTest
    static final ArchRule no_event_listener_unless_decided = noClasses()
        .should().beAnnotatedWith(EventListener.class)
        .because("Bob 假设单 BC + 同步;若你确实需要异步事件,在 ARCHITECTURE.md ADR 记录"
              + "升级到 DDD 战术级的决定,然后修改本规则到 messaging 限定");

    // 项目特定规则在下面追加
}
```

---

## 8. shared 骨架(直接 ship 的 Java 类)

ddd-run 仅在文档讲述,run-bob **直接 ship**——这两个类是 R8 物理基础,缺了所有 spec 都跑不起来。

### 8.1 `shared/usecase/UseCase.java`

```java
package com.example.shared.usecase;

/**
 * 通用 UseCase 接口。所有 usecase 类必须 implements 此接口。
 * 不允许 import 任何框架代码。
 */
public interface UseCase<C, R> {
    R execute(C cmd);
}
```

### 8.2 `shared/framework/transaction/TransactionalUseCaseDecorator.java`

```java
package com.example.shared.framework.transaction;

import com.example.shared.usecase.UseCase;
import org.springframework.transaction.annotation.Transactional;

/**
 * 全工程唯一的 @Transactional 所在地。
 *
 * 用法:在 framework/config/<Feature>UseCaseConfig.java:
 *
 *   @Bean
 *   UseCase<MyCommand, MyResult> myUseCase(MyRepository repo, ...) {
 *       return new TransactionalUseCaseDecorator<>(
 *           new MyUseCase(repo, ...));
 *   }
 *
 * 命令、查询统一走装饰器,无例外。
 */
public class TransactionalUseCaseDecorator<C, R> implements UseCase<C, R> {
    private final UseCase<C, R> inner;

    public TransactionalUseCaseDecorator(UseCase<C, R> inner) {
        this.inner = inner;
    }

    @Override
    @Transactional
    public R execute(C cmd) {
        return inner.execute(cmd);
    }
}
```

落位:
- `<target>/src/main/java/com/example/shared/usecase/UseCase.java`
- `<target>/src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java`

ArchUnit 全限定名 `com.example.shared.framework.transaction.TransactionalUseCaseDecorator` 跟落位严格对齐。用户改 base 包名时同步改两处(README 提示)。

---

## 9. Rust CLI 实现差异

### 9.1 `Cargo.toml`

照搬 ddd-run,改:
- `name = "run-bob"`
- `description = "Bootstrap Bob's Clean Architecture + Superpowers harness for Claude Code projects"`
- 依赖完全照搬:`clap = "=4.5.4"`、`anyhow = "1.0"`、`colored = "2.1"`,dev-dep `tempfile = "3"`
- `[profile.release]` 照搬

### 9.2 `src/main.rs`

CLI 名 `run-bob`,子命令保持 `init` / `status`,flags 完全照搬(`--force` / `--minimal` / `--dir`)。

### 9.3 `src/commands/init.rs`

模板常量替换:

```rust
const SKILL_BOB_IDENTIFY: &str = include_str!("../templates/skills/bob-identify.md");
const SKILL_BOB_ONION:    &str = include_str!("../templates/skills/bob-onion.md");
const SKILL_BOB_SPEC:     &str = include_str!("../templates/skills/bob-spec.md");
const ROOT_CLAUDE_MD:     &str = include_str!("../templates/root/CLAUDE.md");
const ROOT_ARCHITECTURE:  &str = include_str!("../templates/root/ARCHITECTURE.md");
const ROOT_README:        &str = include_str!("../templates/root/README-RUN-BOB.md");
const ROOT_ARCHUNIT_TEST: &str = include_str!("../templates/root/CleanArchitectureTest.java");
const SHARED_USECASE:     &str = include_str!("../templates/root/UseCase.java");
const SHARED_DECORATOR:   &str = include_str!("../templates/root/TransactionalUseCaseDecorator.java");
```

安装步骤增加:
- 写 `UseCase.java` → `<target>/src/main/java/com/example/shared/usecase/UseCase.java`
- 写 `TransactionalUseCaseDecorator.java` → `<target>/src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java`

next-steps 文案改为 `/bob-identify <你的业务描述>` 起手。

### 9.4 `src/commands/status.rs`

校验项替换为以下 9+ 项:
- `.claude/skills/bob-identify/SKILL.md`
- `.claude/skills/bob-onion/SKILL.md`
- `.claude/skills/bob-spec/SKILL.md`
- `CLAUDE.md`
- `ARCHITECTURE.md`(替代 DOMAIN.md)
- `README-RUN-BOB.md`
- `src/test/java/architecture/CleanArchitectureTest.java`
- `src/main/java/com/example/shared/usecase/UseCase.java`(新增)
- `src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java`(新增)
- 工作目录 `docs/bob/`、`docs/specs/`

### 9.5 `tests/integration.rs`

测试名替换 + 新增测试:
- `init_smoke_creates_skill_files`(改名)
- `init_installs_archunit_test_at_correct_path`(照搬)
- `init_minimal_skips_archunit`(照搬)
- `status_reports_archunit_present_after_init`(照搬)
- `status_flags_missing_archunit_when_only_skills_installed`(照搬)
- **新增** `init_installs_shared_usecase_interface`:校验 `UseCase.java` 落到正确路径且包含 `interface UseCase<C, R>`
- **新增** `init_installs_transactional_decorator`:校验装饰器落位且包含 `@Transactional`
- **新增** `init_minimal_skips_archunit_and_shared`:`--minimal` 不安装 ArchUnit + shared

### 9.6 `.claude/skills/install/SKILL.md`

照搬 ddd-run,全文 `ddd-run` → `run-bob`。仓库根目录身份测试条改为 `Cargo.toml` 中 `[package].name == "run-bob"`。

---

## 10. README-RUN-BOB.md 模板差异

结构 1:1 镜像 ddd-run 的 `README-DDD-HARNESS.md`,做以下内容替换:

| ddd-run README 段 | run-bob README 段 |
|---|---|
| Clean Arch 速览(寄居在 DDD harness) | Bob 4 环速览(主体)+ atlas Stage 5 §4 三动作摘要 |
| `/ddd-storm` 介绍 | `/bob-identify` 介绍(身份测试 + 5 问决策树 + G/B1/B2 三模式) |
| `/ddd-model` 介绍 | `/bob-onion` 介绍(4 环设计 + ARCHITECTURE.md SSoT) |
| `/ddd-spec` 介绍 | `/bob-spec` 介绍(命令 / 查询 / 重构 三模板) |
| DOMAIN.md 介绍 | ARCHITECTURE.md 介绍(端口清单 + 状态机 + 配件清单) |
| 完整工作流图 | 同形态,标签替换 |
| 一个完整示例 | 用 atlas Stage 5 OrderUseCase 例子(payOrder / shipOrder / cancelOrder)走通 G 模式 |
| 常见问题 | 增加:"Q: 我已有 Spring 项目要加新功能,要全部重构吗?A: 不必。用 B2 模式开清洁孤岛,只对新功能严格 4 环。" |

---

## 11. 文件清单总表

| # | 路径 | 类型 | 来自 | 改动量 |
|---|---|---|---|---|
| 1 | `Cargo.toml` | 配置 | 照搬 ddd-run | 改 name + description |
| 2 | `src/main.rs` | Rust 源 | 照搬 ddd-run | 改 CLI 名 |
| 3 | `src/commands/mod.rs` | Rust 源 | 照搬 ddd-run | 0 改动 |
| 4 | `src/commands/init.rs` | Rust 源 | 照搬 ddd-run | 改模板常量 + 新增 shared 安装步骤 |
| 5 | `src/commands/status.rs` | Rust 源 | 照搬 ddd-run | 改校验项(9+ vs ddd-run 8) |
| 6 | `src/templates/skills/bob-identify.md` | 模板 | **全新** | §3 全文 |
| 7 | `src/templates/skills/bob-onion.md` | 模板 | **全新** | §4 全文 |
| 8 | `src/templates/skills/bob-spec.md` | 模板 | **全新** | §5 全文 |
| 9 | `src/templates/root/CLAUDE.md` | 模板 | 改自 ddd-run | R0-R12,详见 §6 |
| 10 | `src/templates/root/ARCHITECTURE.md` | 模板 | **全新** | §4.4 模板 |
| 11 | `src/templates/root/README-RUN-BOB.md` | 模板 | 改自 ddd-run | §10 |
| 12 | `src/templates/root/CleanArchitectureTest.java` | 模板 | 改自 ddd-run | 参数化黑名单,§7 |
| 13 | `src/templates/root/UseCase.java` | 模板 | **全新** | §8.1 |
| 14 | `src/templates/root/TransactionalUseCaseDecorator.java` | 模板 | **全新** | §8.2 |
| 15 | `tests/integration.rs` | 测试 | 改自 ddd-run | 9+ 项资产校验 + 2 个 shared 骨架测试 |
| 16 | `.claude/skills/install/SKILL.md` | skill | 照搬 ddd-run | 全文替换 |
| 17 | `README.md` | 文档 | 改自 ddd-run | run-bob 介绍 + Bob 4 环立场 |
| 18 | `LICENSE` | 文档 | 照搬 ddd-run | 0 改动(MIT) |
| 19 | `.gitignore` | 配置 | 照搬 ddd-run | 0 改动 |

统计(共 19 个):照搬(0 改或仅字符串替换)≈ 3 个(`mod.rs` / `LICENSE` / `.gitignore`);改自 ddd-run ≈ 10 个;全新 ≈ 6 个(3 个 skill + `ARCHITECTURE.md` + `UseCase.java` + `TransactionalUseCaseDecorator.java`)。

---

## 12. 验收标准

### 12.1 工程级(CLI 自身)

- [ ] **A1**:`cargo install --path .` → `~/.cargo/bin/run-bob`
- [ ] **A2**:`run-bob --version` 与 `Cargo.toml` 一致
- [ ] **A3**:`run-bob --help` 列出 `init` / `status`,文案不含 "ddd"
- [ ] **A4**:`cargo test` 全绿

### 12.2 init 产出物

在空目录跑 `run-bob init`,产出包含 9 个文件 + 2 个目录:

- [ ] **B1**:`.claude/skills/bob-identify/SKILL.md` 存在且非空
- [ ] **B2**:`.claude/skills/bob-onion/SKILL.md` 存在且非空
- [ ] **B3**:`.claude/skills/bob-spec/SKILL.md` 存在且非空
- [ ] **B4**:`CLAUDE.md` 含 R0 元规则段
- [ ] **B5**:`ARCHITECTURE.md` §4 / §6 为模板待填态
- [ ] **B6**:`README-RUN-BOB.md` 存在
- [ ] **B7**:`CleanArchitectureTest.java` 含 `FORBIDDEN_IN_INNER` + `layered_dependencies`
- [ ] **B8**:`UseCase.java` 含 `interface UseCase<C, R>`
- [ ] **B9**:`TransactionalUseCaseDecorator.java` 含 `@Transactional`
- [ ] **B10**:`docs/bob/` 与 `docs/specs/` 已建

### 12.3 status 校验

- [ ] **C1**:完整 init 后 `run-bob status` 输出 "harness is complete"
- [ ] **C2**:`--minimal` init 后输出 "some assets are missing"
- [ ] **C3**:缺任一资产时该项标红 `✗`

### 12.4 内容级

- [ ] **D1**:`bob-identify.md` 含 5 问决策树文字 + G/B1/B2 触发条件 + 三段式提问规约
- [ ] **D2**:`bob-onion.md` 含 ARCHITECTURE.md 完整模板 + ADR 节 + 回写黑名单步骤
- [ ] **D3**:`bob-spec.md` 含命令 / 查询 / 重构 三模板 + 交给 Superpowers 的开放问题段
- [ ] **D4**:三个 skill frontmatter `description` 写出三种触发条件 + 自然语言触发词

### 12.5 反例拒绝(关键守卫验证)

用户在生成的工程里**故意**写下面违规代码,ArchUnit 必须拦住:

- [ ] **E1**:`entity/Order.java` import `org.springframework.stereotype.Component` → `entity_pure_of_frameworks` fail
- [ ] **E2**:`usecase/PayOrderUseCase.java` 加 `@Service` → `usecase_pure_of_frameworks` fail
- [ ] **E3**:`usecase/SomeUseCase.java` 加 `@Transactional` → `transactional_methods_only_in_decorator` fail
- [ ] **E4**:`adapter/web/OrderController.java` import `entity.Order` → `web_controller_no_entity` fail
- [ ] **E5**:任意 inner 包出现 `@EventListener` → `no_event_listener_unless_decided` fail

### 12.6 棕地兼容性(B2)

- [ ] **F1**:`@AnalyzeClasses(packages = {"com.example.subscription", "com.example.shared"})` 改作用域后,legacy 违规不被报错;`com.example.shared` 必须在数组里,否则 `transactional_methods_only_in_decorator` 规则评估不到装饰器(关键陷阱)
- [ ] **F2**:`bob-identify.md` 在用户输入"我已有 Spring 项目要加新功能 X"时自动识别为 B2

### 12.7 跨工程衔接

- [ ] **G1**:`bob-spec.md` 产出的 spec 末尾含"交给 Superpowers 的开放问题"
- [ ] **G2**:`CLAUDE.md ## 技术栈约定` 模板含警告,防 brainstorming 跳步

---

## 13. 范围外

- ❌ 重构 ddd-run 自身(独立项目)
- ❌ 非 JVM 栈(Rust / Go / Node / Python)等价模板族
- ❌ IDE 插件 / lint 规则
- ❌ 自动从已有项目"扫码生成" ARCHITECTURE.md(`/bob-onion` 通过对话产出)
- ❌ Domain Event / Outbox / Saga 模式(纯 Bob,出现需求时 ADR 触发"升级到 DDD 战术级"分支,**默认不引入**)
- ❌ ddd-run `/install` skill 自动适配到 run-bob(直接复制并字符串替换)

---

## 14. 决策溯源(brainstorm 关键问题汇总)

| # | 问题 | 选项 | 决定 | 理由摘要 |
|---|---|---|---|---|
| 1 | 工程形态 | A 完全对偶 / B 平行调整 skill 切分 / C 你来定 | **B** | 用户决定,skill 切分要反映 Bob 方法 |
| 2 | 入口模式 | A 仅绿地 / B 仅棕地 / C 双入口 | **C** | Bob 招牌叙事是棕地,绿地也常见 |
| 3 | 棕地子模式 | B1 全量重构 / B2 增量新功能 | **两者都要** | 用户强调 B2 也必须严格 γ |
| 4 | 配件识别 | 闭包黑名单 / 开放决策树 | **决策树**(R0 元规则) | 技术栈不收敛,黑名单必更新 |
| 5 | skill 切分 | A 工作流三段式 / B 三动作 / C 四件套 | **A** | UX 一致 + 三动作下沉到 onion |
| 6 | 端口接口归属 | usecase/port / domain | **usecase/port**(纯 Bob) | atlas Stage 5 §4.1 动作 1 |
| 7 | UseCase 风格 | atlas 裸方法 / `UseCase<C,R>` 接口 + 装饰器 | **`UseCase<C,R>` + 装饰器** | 事务唯一性的工程闭合 |
| 8 | 时钟 / ID 生成 | inner 直接 import / 抽端口 | **抽端口**(ClockPort / IdGenerator) | Q2 副作用判定 |
| 9 | Domain Event | 默认引入 / 默认拒绝 | **默认拒绝**(R10-bob) | Bob 单 BC + 同步;升级到 DDD 才加 |
| 10 | shared 骨架 | 文档讲 / 直接 ship Java | **直接 ship** | ArchUnit 引用具体类全名 |
| 11 | 提问风格 | 开放问题 / 推测 + 推荐 | **推测 + 推荐** | 用户偏好,降低协作往返 |
| 12 | 默认栈 | 栈无关 / Java/Spring 预设 | **Java/Spring** | atlas 全为 Java,ArchUnit 仅 JVM |

---

## 15. 依赖与版本锁

照搬 ddd-run,确保 Rust 1.75+ 兼容:

```toml
[dependencies]
clap   = { version = "=4.5.4", features = ["derive"] }
anyhow = "1.0"
colored = "2.1"

[dev-dependencies]
tempfile = "3"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

---

*Generated 2026-05-08 by `superpowers:brainstorming` with amwtke. Next step: `superpowers:writing-plans`.*
