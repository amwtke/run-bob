# CLAUDE.md

> 本文件是 Claude Code 在本项目的**最高优先级约束**。
> 所有生成的代码、文档、测试必须符合本文件的规则。
> 本项目使用 `run-bob` 搭建的 **Bob 4 环 Clean Architecture + Superpowers harness**。
> 严守 atlas Stage 5 §4 的 3 个落地动作:接口位置反转 / 框架边界外推 / 状态机上提。

## 项目定位
<请在此处填写项目一句话描述,例如:"会员积分管理系统的领域服务">

## 模式
- [ ] G(绿地新项目)
- [ ] B1(棕地全量重构)
- [ ] B2(棕地增量新功能 — 清洁孤岛)

## 技术栈约定

> ⚠️ **本段未填充前,任何 skill(包括 Superpowers)不得产出实现代码。**
> 栈决策必须由 `superpowers:brainstorming` 驱动,在回答完 `/bob-spec` 末尾"交给 Superpowers 的开放问题"后,由用户确认并**写回本段**,再进入 `writing-plans`。

决策必须覆盖:
- 语言 / 运行时
- 应用框架(Web / Service / CLI)
- 持久化方案(关系型 / 文档型 / 事件存储 / 内存)
- **范围:仅后端,还是前后端全栈?**若含前端,用什么框架
- 对外交互形态(REST / gRPC / GraphQL / CLI / 消息)
- 测试框架 / 构建工具

<!-- 填充示例(完成后删除本注释,替换为上面的条目):
- 语言:Java 17
- 框架:Spring Boot 3.x
- 持久化:MyBatis(仓储实现)/ JPA(可选)
- 辅助:Lombok(仅 DTO/VO 在 adapter 层),MapStruct(DTO ↔ Entity)
- 范围:仅后端服务
- 交互:REST
- 测试:JUnit 5 + AssertJ + Mockito
- 构建:Maven
-->

_待填充(由 `superpowers:brainstorming` 产出后写回本段)_

## 分层架构(Bob 同心圆 4 环,详见 R7)

```
┌──────────────────────────────────────────┐
│  framework (Ring 4)                       │  ← Spring 装配 + 事务装饰器 + main
├──────────────────────────────────────────┤
│  adapter (Ring 3)                         │  ← REST Controller / Saga / Repository 实现 / ACL
├──────────────────────────────────────────┤
│  usecase (Ring 2)                         │  ← Interactor (POJO,零 Spring,零 SLF4J)
├──────────────────────────────────────────┤
│  entity (Ring 1)                          │  ← Entity / 状态机 / 值对象(纯 Java)
└──────────────────────────────────────────┘
```

**依赖方向**:只能由外向内。`entity` 不 import 任何东西;`usecase` 只 import `entity`;`adapter` 可 import `usecase` 与 `entity`;`framework` 可 import 一切。

**关键铁律**(完整规则见 R7-R12):
- usecase 层**禁止**任何 Spring / Jakarta / SLF4J / Lombok import 或注解
- 全工程**唯一**的 `@Transactional` 在 `shared.framework.transaction.TransactionalUseCaseDecorator`,且**必须** `rollbackFor = Exception.class`(防 checked 异常静默 commit)
- 共享可变计数(库存 / 余额 / 配额)**禁止** read-modify-write,只能在 adapter 层用原子条件 UPDATE,端口暴露语义动作(`tryDecrease` / `restore`)而非 `findById/save`
- 被多个 UseCase 触发状态迁移的聚合,**必须**带 `@Version` 乐观锁(防并发 last-writer-wins)
- Entity 不得 `LocalDateTime.now()` / `UUID.randomUUID()` (用 ClockPort / IdGenerator)
- 跨上下文 / 异步 = 升级触发器,默认拒绝 Domain Event(R10-bob)

## 强制规则(Hard Rules)

### R0. 通用判定优先于具体清单

本文件 R7-R12 列出的"禁止 import"清单(Spring / SLF4J / Jakarta / Lombok)只是
2026 年 Java/Spring 生态的典型样本,不构成穷举。

遇到任何新外部库 / 新注解 / 新框架 / 新信创组件,必须先跑 `ARCHITECTURE.md §配件清单`
扩充,且在 inner 包(entity / usecase)使用前先跑 5 问决策树
(详见 `/bob-identify` skill 与 ARCHITECTURE.md §配件清单 章节)。

判定为配件的所有外部依赖,无论是否在 R7-R12 显式列出,都不允许出现在 entity/** 与 usecase/** 包。
**R0 是 R7-R12 的母规则——R7-R12 是 R0 的当前可执行实例。**

### R1. 战略层先行 + 技术栈先决策

任何新特性的实现顺序必须是:
```
/bob-identify → /bob-onion(更新 ARCHITECTURE.md) → /bob-spec
              → superpowers:brainstorming(若 ## 技术栈约定 未填)
              → superpowers:writing-plans
              → superpowers:executing-plans(TDD)
              → superpowers:finishing-a-development-branch
```
**禁止跳过前面任一步直接写代码**。尤其:
- 进入 `writing-plans` 之前,本文件"## 技术栈约定"段必须已填(由 `brainstorming` 写回)。
- 如果用户要求跳步,请指出这会违反本项目的 harness 约定。

### R2. 术语一致性

所有代码命名必须引用 `ARCHITECTURE.md` 中的:
- **§3 核心 Entity 与状态机**(Entity 名 + 状态名 + 方法名)
- **§4 端口清单**(端口接口名 + 方法签名)
- **§5 UseCase 清单**(UseCase 类名 + Command/Result record 名)

如发现代码中的命名与 ARCHITECTURE.md 不一致,**停下来**,不要自作主张修改,而是询问用户:
> 代码中的 `X` 与 ARCHITECTURE.md 中的 `Y` 不一致,是代码要改还是 ARCHITECTURE.md 要改?

### R3. 富 Entity 模型(禁止贫血)

Entity 必须封装业务行为:
- ✅ `order.payTo(paymentGateway, inventoryClient)` — Entity 校验状态 + 调端口 + 改自身
- ❌ `orderUseCase.payOrder(order, pg, ic)` — 规则写在 UseCase,Entity 变成数据袋

UseCase 只能做这几件事:
1. 接收 Command(纯数据 record)
2. 从 Repository 取出 Entity
3. 调用 Entity 的业务方法
4. 持久化(通过 Repository)
5. 返回 Result(纯数据 record)

**任何 `if/else`、`for` 循环里包含业务判断的代码都必须在 Entity 层,不在 UseCase**。

**R3 补充(Clean Architecture)**:用例层(`usecase/**`)严禁出现任何注解或框架 import。
usecase 是纯 Java POJO,通过构造器注入端口接口,业务规则委托给 Entity。
日志需求通过 `LoggerPort` 抽象,不允许直接 `import org.slf4j.*`。

### R4-bob. Entity 状态迁移自封

Bob 不强调 DDD 聚合边界(那是 DDD 战术级);相反,本规则要求每个 Entity 自封
所有状态迁移规则。

- ✅ `order.payTo(paymentGateway, inventoryClient)` — Entity 内 `ensureStatus(...)` 守
- ❌ `if (order.getStatus() == CREATED) order.setStatus(PAID)` — 外部修改(贫血)

参考 atlas Stage 5 §4.1 动作 3:**状态机上提到 Entity**。

### R5. Repository 端口归属 usecase/port

- ✅ `MemberRepository`、`OrderRepository`(端口接口在 `usecase/port/`,纯 Java)
- ✅ `JpaOrderRepository implements OrderRepository`(实现在 `adapter/persistence/`)
- ❌ Repository 接口直接 extends `org.springframework.data.jpa.repository.JpaRepository`(违反接口位置反转)
- ❌ Repository 实现放在 `framework/`(必须在 `adapter/persistence/`)

端口名格式:`<EntityName>Repository`。**不要**叫"聚合根 Repository"——本项目是纯 Bob,不用 DDD 术语。

### R6. TDD 节奏(由 Superpowers 执行)

进入实现阶段后,严格遵循 Superpowers 的 spec → test → code 节奏:
1. 一次只处理一个 spec
2. 先写测试,让它失败
3. 最小改动让测试通过
4. 重构
5. 进入下一个 spec

**禁止"一次性生成整套代码"**。如用户要求一次性生成,请指出这违反 harness 约定。

### R7. 包结构(4 环 Clean Architecture)

> 下方默认是 Java / Spring 风格。若 `superpowers:brainstorming` 选定了其他栈,目录命名按该栈调整,但**4 环依赖语义不变**。

```
com.example.<bizname>/
├── entity/                                 Ring 1 — POJO 状态机(零框架)
│   ├── <Entity>.java                       状态机 + 业务规则
│   ├── <ValueObject>.java                  record / 不可变值对象
│   └── <EntityName>Id.java                 强类型 ID(record)
├── usecase/                                Ring 2 — Interactor (POJO,零框架)
│   ├── <Command>UseCase.java               implements UseCase<C, R>(orchestration 类放本层根)
│   ├── in/
│   │   ├── <Command>Command.java           入站 record(命令,usecase 边界)
│   │   └── <Query>Query.java               入站 record(查询)
│   ├── out/
│   │   └── <Command>Result.java            出站 record(避免泄露 Entity)
│   └── port/
│       ├── <EntityName>Repository.java     端口接口(纯 Java)
│       ├── <Gateway>.java                  出站端口(外部系统/ACL)
│       ├── ClockPort.java                  时钟端口(若 Entity 需 now())
│       └── LoggerPort.java                 日志端口(若 usecase 需日志)
├── adapter/                                Ring 3 — 允许 Spring/JPA/SDK
│   ├── web/<Aggregate>Controller.java      入站 REST
│   ├── messaging/<X>EventHandler.java      入站 MQ(如启用)
│   ├── persistence/
│   │   ├── <Entity>JpaEntity.java          JPA 映射
│   │   ├── SpringData<Entity>Repo.java     Spring Data 接口
│   │   └── Jpa<Entity>Repository.java      实现 usecase 端口
│   ├── acl/<Gateway>HttpAcl.java           出站 ACL
│   ├── time/SystemClockAdapter.java        ClockPort 实现
│   └── logging/Slf4jLoggerAdapter.java     LoggerPort 实现
└── framework/                              Ring 4 — 允许全 Spring
    └── config/
        └── <Feature>UseCaseConfig.java     @Bean 装配 + 装饰器包裹
```

工程级共享包:

```
com.example.shared/
├── usecase/UseCase.java                    通用 UseCase<C, R> 接口
└── framework/
    └── transaction/
        └── TransactionalUseCaseDecorator.java   全工程唯一 @Transactional
```

**依赖方向**:只能由外向内。`entity` 不 import 任何东西;`usecase` 只 import `entity`;
`adapter` 可 import `usecase` 与 `entity`;`framework` 可 import 一切。

### R8. 装配规则(Spring 注解的位置)

1. `@Transactional` 在全工程**有且仅有一处**:`shared.framework.transaction.TransactionalUseCaseDecorator.execute()` 方法
2. 该 `@Transactional` **必须**写成 `@Transactional(rollbackFor = Exception.class)`。Spring 默认仅在 `RuntimeException` / `Error` 时回滚——任何 checked Exception 抛出都会静默 commit 部分写入。装饰器是全工程事务唯一入口,不允许这种漏洞
3. Spring 注解(`@Service` / `@Component` / `@Repository` / `@Configuration` / `@Bean` / `@RestController` / `@EventListener` / `@Autowired` 等)**只允许**出现在 `adapter/**` 或 `framework/**`
4. 任何 usecase 都通过 `<feature>.framework.config.<Feature>UseCaseConfig` 中的 `@Bean` 注册,**必须**经 `TransactionalUseCaseDecorator` 包装(命令、查询统一,无例外)
5. **测试义务**:`TransactionalUseCaseDecorator` 必须有两条测试——抛 `RuntimeException` 回滚 / 抛 checked `Exception` 同样回滚。少一条说明 R8 第 2 点失守

### R9. 反模式硬清单(代码不得通过 review)

- ❌ `usecase/**/*.java` 出现任何 `import org.springframework.*` / `jakarta.persistence.*` / `jakarta.inject.*` / `org.slf4j.*` / `lombok.*`
- ❌ `usecase/**/*.java` 出现任何注解
- ❌ `entity/**/*.java` 出现任何框架 / Lombok / SLF4J import
- ❌ `@Transactional` 出现在 `shared.framework.transaction.TransactionalUseCaseDecorator` 之外
- ❌ `adapter/**` 之间横向 import(只能向内)
- ❌ `adapter/web/Controller` 直接 import `entity.*`(应通过 usecase 的 Command / Result 类型)
- ❌ Repository 实现放在 `framework/`(应在 `adapter/persistence/`)
- ❌ Saga handler 内部出现 `if/else` 业务分支(只能事件 → Command 翻译 + 调 usecase)
- ❌ usecase 跳过装饰器,Config 直接 `@Bean` 返回裸 POJO
- ❌ usecase 内出现 `Logger log = LoggerFactory.getLogger(...)`(必须用 `LoggerPort`)
- ❌ Entity 不得 `LocalDateTime.now()` / `UUID.randomUUID()` / `System.currentTimeMillis()`(用 ClockPort / IdGenerator)
- ❌ 决策树判为配件的库一律不得在 inner 包(entity / usecase),即使本文件未显式列出
- ❌ B2 模式:新功能不得 import legacy 的 `@Service` 类(必须通过 usecase/port 端口 + adapter/acl ACL 隔离)
- ❌ 装饰器写成裸 `@Transactional` 而非 `@Transactional(rollbackFor = Exception.class)`(checked 异常会静默 commit)
- ❌ 共享可变计数(库存 / 余额 / 配额 / 名额)在 UseCase 里 `findById → mutate → save`(read-modify-write 在并发下产生 lost-update,直接超卖)。正解:端口暴露语义动作 `tryDecrease(id, qty) -> boolean` / `restore(id, qty)`,adapter 用一条 `UPDATE ... WHERE quantity >= :qty` 原子 SQL,UseCase 只判 boolean
- ❌ 被多个 UseCase 触发状态迁移的聚合(如 Order 的 pay/ship/cancel/complete)缺失 `@Version` 乐观锁(并发下 last-writer-wins 静默丢更)。要求 JPA 实体加 `@Version`、Entity 透传 `version` 字段、mapper 双向往返
- ❌ `*Command` / `*Query` record 与 `*UseCase` 混在 `usecase/` 根包(必须 `usecase/in/`);`*Result` 同理(必须 `usecase/out/`)。`usecase/` 根目录只放 `*UseCase.java` orchestration 类

### R10-bob. 跨上下文 / 异步 = 升级触发器

Bob 默认假设单 Bounded Context + 同步业务。

- 出现"跨聚合协作"、"事件驱动"、"Saga 补偿"、"最终一致性"需求时:
  1. 停下,不要直接引入 `@EventListener` / `Outbox` / `ApplicationEventPublisher`
  2. 在 `ARCHITECTURE.md §10 ADR` 记录"升级到 DDD 战术级"的决定
  3. 修改 `CleanArchitectureTest.java` 的 `no_event_listener_unless_decided` 规则到 messaging 限定
  4. 考虑切换到 ddd-run harness(它支持 Domain Event 一等公民)

参考 atlas Stage 6 §2.5:Bob `order.payTo(pg, ic)` Sync vs DDD `order.pay()` + Event Async 的
差异。Bob 的 Sync 风格在跨 BC 场景下退化,这是触发升级的信号。

### R11. ArchUnit 守卫

`src/test/java/architecture/CleanArchitectureTest.java` 是 R7-R10 的**机械执法者**。
不要删除已有规则;可在文件末尾追加项目特定规则。CI 必须运行该测试并对失败阻塞合并。

引入新外部库时,跑 5 问决策树 → 若判为配件,把根包加到 `FORBIDDEN_IN_INNER` 数组。

### R12. 增量新功能(B2)必须 γ,即使周围是 α/β legacy

棕地增量新功能场景下,新代码必须建立"**清洁孤岛**"——即使周围是 α/β legacy:

- 新功能落在独立包(如 `com.example.<feature>`),与 legacy 同级或子级,**不混在 legacy 包内**
- 新功能不允许跨包"复用" legacy `@Service`;若需调用,通过 usecase/port 端口 + adapter/acl ACL 包装
- 新包内禁止"为兼容 legacy 风格"的妥协(如新 usecase 加 `@Service` "保持一致")
- ArchUnit `@AnalyzeClasses` 改为多包数组,**必须**包含 `com.example.shared`,否则
  `transactional_methods_only_in_decorator` 规则评估不到装饰器

  ```java
  @AnalyzeClasses(
      packages = {"com.example.<feature>", "com.example.shared"},
      importOptions = DoNotIncludeTests.class
  )
  ```

legacy 是另一个"外部世界",和 MySQL / 人大金仓 / 微信支付 SDK 没区别,统一用端口 + ACL 隔离。

### R13. 合规规则前置(项目级)

实现代码之前,**必须**检查 `docs/compliance/*.md`:

- 若不存在或目录为空 → 跳过(无合规要求)
- 若存在 → 读取与当前文件 / 模块相关的章节,**严格遵守所有【强制】条款**
- 在代码注释里引用规则 ID(例:`// 遵守 [ALI-1.1.2] 命名规约`),便于后续 `/bob-compliance` 校验时复核
- 不得擅自违反【强制】条款 —— 如确需违反,必须在 spec 的"交给 Superpowers 的开放问题"段写明**豁免**理由,
  否则 `/bob-compliance` Stage 3 会标记为违反

R13 是 `/bob-compliance` 工作流的**模式 A** 载体(写代码时的"指导"环节)。
Claude 在 TDD 时本来就会读 CLAUDE.md,R13 是最廉价的注入路径:合规知识 = 自然语言规约,
Claude 直接消费 markdown,无需任何额外工具。

## 工作流总览

```
┌──────────────────────────────────────────────────────────────┐
│                      战略层(身份测试 + 4 环设计)            │
│  业务需求 / 已有代码 / 新功能                                 │
│       ↓                                                       │
│  /bob-identify  ─→ docs/bob/01-identity-*.md(5 问决策树)    │
│       ↓                                                       │
│  /bob-onion     ─→ ARCHITECTURE.md(SSoT)                    │
└──────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌──────────────────────────────────────────────────────────────┐
│                  桥接层(spec + 栈决策)                      │
│  ARCHITECTURE.md + 用例 ─→ /bob-spec ─→ docs/specs/spec-*.md │
│                          ↓                                    │
│  superpowers:brainstorming ─→ 技术栈 / FE-BE 范围 / 交互     │
│                              形态(写回本文件 "## 技术栈")   │
│                          ↓                                    │
│  superpowers:writing-plans ─→ docs/superpowers/plans/*.md    │
└──────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌──────────────────────────────────────────────────────────────┐
│                      战术层(实现)                           │
│  executing-plans ─→ TDD ─→ 测试 ─→ 实现 ─→ finishing-branch  │
└──────────────────────────────────────────────────────────────┘
```

## 修改 ARCHITECTURE.md 的流程

`ARCHITECTURE.md` 是 Bob 4 环架构的 Single Source of Truth,**不得随意修改**。

允许的修改路径:
1. 通过 `/bob-onion` 重新设计(推荐)
2. 在 `/bob-spec` 过程中发现缺失,**停下来先修 ARCHITECTURE.md 再继续**

禁止的修改路径:
- ❌ Superpowers 实现过程中擅自修改 ARCHITECTURE.md
- ❌ 为了让代码通过测试而改 ARCHITECTURE.md 的术语

## 代码质量底线

- 每个 Entity 必须有单元测试(覆盖所有状态迁移 + 不变量)
- 每个 UseCase 必须有集成测试(用 mock 端口)
- 测试命名使用业务语言:`shouldRedeemPointsWhenBalanceIsSufficient`
- 禁止魔法数字(用 `Points.of(100)` 而非 `100`)
- 禁止 `public` 字段(除 `record` 组件)
- 禁止 setter(除 `record` 组件)
- 禁止 `LocalDateTime.now()` / `UUID.randomUUID()` 在 inner 包(用 ClockPort / IdGenerator)

---
*Generated by run-bob. 本文件可根据项目实际情况调整,但不要删除"强制规则"部分。*
