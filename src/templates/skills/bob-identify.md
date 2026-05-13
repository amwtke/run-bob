---
name: bob-identify
description: |
  触发条件:用户输入 /bob-identify <业务描述>(模式 G:绿地新项目),
  或 /bob-identify --refactor [path](模式 B1:对已有 α/β 代码做全量身份测试),
  或 /bob-identify <新功能描述>(模式 B2:已有 src/main/java + 描述新功能 = auto-detect 棕地增量)。

  对给定的业务描述 / 已有代码 / 新功能,跑一遍 5 问决策树
  (Q1 业务意义会变? Q2 有副作用? Q3 翻译者还是编排者?
   Q4 出现在 inner 包? Q5 棕地 legacy 复用?),
  把每一个候选概念 / 类 / import / 注解分类为 CORE / ADAPTER /
  FRAMEWORK / TOOL / 违规,产出一份结构化分析文档作为
  /bob-onion 的输入。

  适用于 Bob 4 环 Clean Architecture 的第一阶段:从模糊业务描述
  / 已有代码 / 新功能描述里提取核心 vs 配件骨架。
  当用户说"做身份测试"、"区分核心和配件"、"这段代码哪些是核心
  哪些是框架"、"这个功能里什么是 Entity"时也应触发此技能。
---

# Bob Identity Test Skill

## 触发

```
/bob-identify <业务描述>           # 模式 G:绿地新项目
/bob-identify --refactor [path]    # 模式 B1:对已有 α/β 代码做全量身份测试
/bob-identify <新功能描述>         # 模式 B2:auto-detect(已有 src/main/java + 描述新功能)
```

或自然语言触发:"做身份测试"、"区分核心和配件"、"这段代码哪些是核心哪些是框架"、"这个功能里什么是 Entity"。

## 先检查 /bob-survey (soft 前置)

启动时若检测到下面任一条件,**三段式追问**用户是否先跑 `/bob-survey`:

- 项目内不存在 `docs/bob/00-survey-*.md`
- 最新一份 `00-survey-*.md` 距今 > 7 天

格式:

> **Q0:看起来你还没跑过 `/bob-survey`(或最近一次已超过 7 天),是否先体检一次?**
>
> **推测**:建议先跑。我可以直接做身份测试,但跳过体检直接接需求,γ 底子下问题不大,β/α 底子下大概率会拖垮新代码质量。
> **理由**:`/bob-survey` 会给出 6 维度评分 + 需求难度 + 三档建议,15 秒内告诉你"现在能不能接 / 要不要先做几个修补"。
> **推荐选择**:`先跑 /bob-survey <需求>`
>
> 是否同意?(回"是"→跳出 identify 让用户执行 survey;回"否"→继续做身份测试)

**不强制**。用户明确说"否"则照旧继续。这一节遵从 TL 风:提问而不阻塞。

## 目标

把**每一个候选概念 / 类 / import / 注解**都跑过 5 问决策树,产出一份分类表。
**不画架构、不写代码、不出 spec**——只回答一个问题:**这个东西到底是什么?**

## 提问规约(强制)

任何需要用户选择的问题,**必须**按下面三段式输出。**禁止**抛开放问题。

格式:

> **[问题序号] [问题]**
>
> **推测**:<你的判断,基于上下文的最优解>
> **理由**:<一句话,为什么这么推测——引用业务描述/代码事实/Bob 原则>
> **推荐选择**:`<具体一个选项>`
>
> 是否同意?(回"是"走推荐;回"否,理由是 ..."重判;回"否,我选 X"切到 X)

## 5 问决策树

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

## 副作用的精确含义

"副作用"是 Q2 的关键判定点。在 Bob 4 环判定语境里,它指——**让代码"行为不可预测"或"必须有外部条件才能运行"的任何东西**:

| 类别 | 含义 | 例子 |
|---|---|---|
| **读外部状态** | 行为依赖进程外世界 | 读数据库、读文件、读环境变量、读系统时钟 |
| **改外部状态** | 行为会让进程外世界变化 | 写库、发 HTTP、推消息、写日志 |
| **非确定性** | 同样输入不一定同样输出 | `LocalDateTime.now()`、`UUID.randomUUID()`、`Math.random()` |
| **依赖容器/框架生命周期** | 必须在某个 runtime 才能跑 | `@PostConstruct`、`ApplicationContextAware`、Servlet 容器 |

**反例**(无副作用,可直接 import 进 Entity/UseCase):

- `Objects.requireNonNull(x)` —— 同样输入永远同样行为,不读不写外部
- `Collectors.toList()` —— 纯函数式操作
- `Math.max(a, b)` —— 确定性计算
- `String.format(...)` —— 同样输入同样输出
- 你自己写的 `Order.canBeCancelled()` 这种**只看自身字段**做判断的纯方法

→ 这些可以**直接 import** 进 Entity/UseCase,无需端口。

**Q2 真正威力**:它**预先识别隐性外部依赖**,在用户写代码之前就把它揪出来抽端口。

## 工作流

### 模式 G(绿地)— 从业务描述抽核心

**Step G1**:复述业务描述,确保理解一致。

**Step G2**:用业务语言列出所有名词(订单、商品、库存、支付、短信、配送)+ 所有动词("用户下单"、"扣减库存"、"调用支付"、"发短信通知")。

**Step G3**:对每个名词 / 动词,**对话式**(三段式提问)跑 5 问决策树。

**Step G4**:对每个"动作 / 副作用"(发短信、调支付、写库),**主动追问外部对接形态**:
- 调用支付——具体是哪个支付通道?微信支付 SDK / 支付宝 / 银联?
- 发短信——阿里云 SMS / 腾讯云 / 第三方?
- 写库——MySQL / PostgreSQL / 信创(达梦 / 人大金仓)?

**Step G5**:产出 `docs/bob/01-identity-<topic-slug>.md`(模板见下)。

### 模式 B1(棕地全量重构)

**Step B1.1**:跑代码扫描:
```bash
find src/main/java -name "*.java" -not -path "*/test/*"
```
分析:
- 包结构(是否已有 entity/usecase/adapter/framework 命名?有 → β 嫌疑;无 → α 嫌疑)
- 每个类的 import(命中 R0 配件清单的扔进表)
- 每个类的注解(`@Service` / `@Repository` / `@Component` / `@Transactional` / `@Slf4j` 等)
- 关键方法签名(如 `public void cancel(Long id)` 直接 SQL → 状态机散落 α 信号)

**Step B1.2**:对每个类跑决策树,标 4 个评级之一:

| 评级 | 含义 | 后续动作 |
|---|---|---|
| **γ** | 已合规,什么都不用动 | 跳过 |
| **β** | 包结构对了但 usecase 还碰框架 | 列入"框架边界外推"重构清单 |
| **α** | 业务规则散落 Service / 状态机在 Service / interface 在外层 | 列入"接口位置反转 + 状态机上提"重构清单 |
| **violation** | 严重违反(Entity 上有 `@Entity`、`@Slf4j`) | 列入"硬违规"段,优先重构 |

**Step B1.3**:产出 `docs/bob/01-identity-<topic-slug>.md`,**额外含**:
- "α/β/γ 评级分布"(文字描述,如 "45 类:α 12 / β 28 / γ 3 / violation 2")
- "重构优先级"(violations → α → β,按业务重要性二级排序)

### 模式 B2(棕地增量新功能)

**Step B2.1**:**重要追问**——是否有 legacy 模块需要复用?

**Step B2.2**:对**新功能**的描述(业务语言)跑模式 G 的流程,产出 CORE / ADAPTER / FRAMEWORK 分类。

**Step B2.3**:对每个"复用 legacy"的需求,执行**清洁孤岛规则**:
- legacy 不动
- 在新功能 usecase/port 里定义业务领域接口(用新业务语言,不要照抄 legacy 的方法签名)
- 在新功能 adapter/acl 里写 adapter 包装 legacy

**Step B2.4**:产出 `docs/bob/01-identity-<新功能名>.md`,**额外含**:
- "清洁孤岛包路径"(确认新代码落在哪个包,如 `com.example.subscription/{entity,usecase,adapter,framework}/`)
- "Legacy 隔离接口清单"(每条:legacy 类 → 新业务语言端口 → adapter 名)
- "ArchUnit 作用域提示"(给出修改 `@AnalyzeClasses` 的具体语法行,**必须**含 `com.example.shared`)

## 产出文档模板

```markdown
# 身份测试:<主题>

> 模式:G / B1 / B2(三选一)
> 由 /bob-identify 生成。所有"配件"必须在后续 /bob-onion 阶段被分配到 adapter 或 framework 环;
> 所有"违规"必须在 /bob-onion 给出重构方案。

## 1. 业务描述复述
<重述业务描述,模式 G;模式 B1 复述代码扫描发现;模式 B2 复述新功能并标识需要复用的 legacy>

## 2. 5 问决策树分类表

| # | 候选元素 | Q1 业务意义会变? | Q2 有副作用? | Q3 翻译者还是编排者? | Q4 出现在 inner 包? | 分类 | 建议端口名 | 备注 |
|---|---|---|---|---|---|---|---|---|
| 1 | Order | 会变 | — | — | — | **CORE/Entity** | — | 状态机:Created → Paid → Shipped → Completed/Cancelled |
| 2 | OrderStatus | 会变 | — | — | — | **CORE/Entity** | — | enum,值对象 |
| 3 | payOrder() 流程 | 会变 | — | — | — | **CORE/UseCase** | — | UseCase 编排,不放业务规则 |
| 4 | 微信支付 SDK (`WxPayClient`) | 不会变 | 有(网络) | 翻译者 | 否 | **ADAPTER/acl** | `PaymentGateway` | 业务接口在 usecase |
| 5 | JdbcTemplate | 不会变 | 有(磁盘) | 翻译者 | 否 | **ADAPTER/persistence** | `OrderRepository` | 接口在 usecase |
| 6 | SLF4J `Logger` | 不会变 | 有(磁盘) | 翻译者 | **是 ⚠️** | **违规** | `LoggerPort` | OrderUseCase 里有 `private final Logger log = ...` 必须删 |
| 7 | `@Transactional` | 不会变 | 有(容器) | 编排者 | **是 ⚠️** | **违规** | TransactionalUseCaseDecorator | 当前在 OrderApplicationService 上 → 必须移到 framework 装饰器 |
| 8 | `LocalDateTime.now()` | 不会变 | 有(时钟) | 翻译者 | **是 ⚠️** | **违规** | `ClockPort` | Entity 字段 `paidAt = LocalDateTime.now()` → 接收 Instant 参数,now() 在 framework 注入 |
| 9 | `Objects.requireNonNull` | 不会变 | 无 | — | — | **TOOL** | — | JDK util,可直接 import |
| 10 | 达梦驱动 `DmDriver` | 不会变 | 有(网络) | 翻译者 | 否 | **ADAPTER/persistence** | 同 OrderRepository | 信创栈,只在 adapter |
| ... | | | | | | | | |

## 3. α/β/γ 评级(仅模式 B1)

- **γ(合规)**:`<列表>`
- **β(包对了但还碰框架)**:`<列表>`,共 N 类
- **α(业务规则散落)**:`<列表>`,共 N 类
- **violation(硬违规)**:`<列表>`,共 N 类

## 4. 清洁孤岛 / Legacy 隔离(仅模式 B2)

- **新功能包路径**:`com.example.<feature>/{entity,usecase,adapter,framework}/`
- **Legacy 隔离接口**:
  | Legacy 类 | 业务领域接口(新建) | Adapter 实现(新建) |
  |---|---|---|
  | `LegacyOrderService.fetchByCustomer` | `usecase/port/CustomerOrderHistory` | `adapter/acl/LegacyCustomerOrderAdapter` |
- **ArchUnit 作用域行**:
  ```java
  @AnalyzeClasses(
      packages = {"com.example.<feature>", "com.example.shared"},
      importOptions = DoNotIncludeTests.class
  )
  ```
  **关键陷阱**:必须包含 `com.example.shared`,否则 `transactional_methods_only_in_decorator` 规则评估不到装饰器。

## 5. 配件清单回写建议(给 /bob-onion 用)

本次新识别出的"未在 R7-R12 列出但应禁止于 inner 包"的配件包名:
- `io.dameng..`(达梦驱动)
- `com.alibaba.fastjson..`
- ...
→ /bob-onion 阶段把这些写回 `CleanArchitectureTest.java` 的 `FORBIDDEN_IN_INNER` 数组,并在 ARCHITECTURE.md §配件清单 增加项目特化条目。

## 6. 开放问题

主动列 2-5 个对业务/技术的疑问,不要假设:
- Q1: ...
- Q2: ...

## 7. 下一步

> 运行 `/bob-onion` 基于本表画 4 环架构。
> 模式 B1 还要附上 α→γ 重构计划。模式 B2 要确认清洁孤岛包路径已就绪。
```

## 反模式(skill 必须拒绝)

- ❌ 直接给"分类结果"而不跑决策树(无 Q1-Q4 推导,用户没法 review)
- ❌ 闭目使用 R7-R12 黑名单做分类(R0 优先,要跑通用决策树)
- ❌ 在 identify 阶段就画 4 环 / 写代码(那是 `/bob-onion` 的事)
- ❌ 模式 G 假设业务里没有副作用(任何非 trivial 业务都有副作用,主动追问并入表)
- ❌ 模式 B2 不主动追问 legacy 复用(漏问 = 后续 spec 阶段再返工 100x)

## 与其他 skill 衔接

- **上游**:用户业务描述 / 已有代码 / 新需求
- **下游**:产出表喂给 `/bob-onion` 画 4 环
- **不做**:不画架构、不写代码、不出 spec

## 文件落位

`docs/bob/01-identity-<topic-slug>.md`(目录不存在自动建)
