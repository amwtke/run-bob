# 架构(Bob 4 环)· <项目/上下文名>

> 本文档是本项目 Bob 4 环架构的 Single Source of Truth。
> 所有 Superpowers spec、代码命名、测试描述必须使用本文档定义的端口名 / Entity 名 / 状态名。
> 禁止在代码中出现与本文档不一致的命名。
> 本文件由 /bob-onion 管理。不要手工编辑(除非你知道自己在做什么)。

---

## 📌 状态
- 模式:**<G | B1 | B2>**(三选一,由 /bob-identify / /bob-onion 决定)
- [ ] 已完成身份测试(`/bob-identify`)
- [ ] 已完成 4 环设计(`/bob-onion`)
- [ ] 已生成至少一个 spec(`/bob-spec`)
- [ ] 已有代码实现(Superpowers)

当前状态:**待初始化**。请先运行 `/bob-identify <你的业务描述>`。

---

## 1. 上下文(Context)
- **名称**:<如 "订阅返点上下文">
- **职责**:<一句话说清这个上下文负责什么>
- **不负责**:<明确列出与之相邻但不属于本上下文的职责>

## 2. 4 环包结构

```
com.example.<bizname>/
├── entity/        Ring 1 — 实体 + 业务规则        (零框架,纯 Java)
├── usecase/       Ring 2 — Interactor             (零框架,POJO)
│   ├── *UseCase.java                              orchestration 类(本层根)
│   ├── in/      入站 record:*Command / *Query
│   ├── out/     出站 record:*Result
│   └── port/    端口接口(Repository / Gateway / Clock / Logger)
├── adapter/       Ring 3 — Controller / Repo impl (允许 Spring/JPA/SDK)
└── framework/     Ring 4 — 装配 + 事务装饰器 + main
```

**依赖方向**:只能由外向内。`entity` 不 import 任何东西;`usecase` 只 import `entity`;
`adapter` 可 import `usecase` 与 `entity`;`framework` 可 import 一切。

## 3. 核心 Entity 与状态机

> 每个有状态字段的 Entity 独立一节,由 `/bob-onion` 填充。

### 3.x <Entity 名>(模板)
- **字段**:<列出 + 类型 + 是否可变>
- **状态机**:
  ```
  CREATED ──pay──> PAID ──ship──> SHIPPED ──complete──> COMPLETED
     │                │
     │                └──cancel──> CANCELLED
     └──cancel──> CANCELLED
  ```
- **核心方法**:
  - `<methodSignature>`:前置条件 / 后置条件 / 触发的状态迁移
  - ...
- **不变量**:
  - INV-1:非法状态迁移必须 throw `IllegalStateException`
  - INV-2:字段无 setter(record 组件除外)
  - INV-3:<其他业务不变量>

## 4. 端口清单(usecase/port/)

> 业务定义的接口。Adapter 层 implements,framework 层装配。

| 端口名 | 签名摘要 | Adapter 实现 | 落位包 |
|---|---|---|---|
| _待 /bob-onion 填充_ | | | |

**示例**(由 /bob-onion 替换):
- `OrderRepository` / `findById, save` / `JpaOrderRepository` / `adapter/persistence`
- `PaymentGateway` / `pay, refund` / `WeChatPaymentAdapter` / `adapter/acl`
- `ClockPort` / `now()` / `SystemClockAdapter` / `adapter/time`
- `LoggerPort` / `info, error` / `Slf4jLoggerAdapter` / `adapter/logging`

## 5. UseCase 清单

| UseCase | Command record | Result record | 用到的端口 |
|---|---|---|---|
| _待 /bob-onion 填充_ | | | |

## 6. 配件清单(项目特化)

> 跑过 5 问决策树后识别出的、本项目实际用到的配件。
> 任何在 inner 包(entity / usecase)出现这些 import 的代码都违规。
> 引入新外部库时,必须先跑决策树扩充本表 + 回写 `CleanArchitectureTest.java` 的 `FORBIDDEN_IN_INNER`。

| 配件 | 根包 | 端口抽象 | 落位环 |
|---|---|---|---|
| Spring | `org.springframework..` | (注解模式,不需 port) | adapter / framework |
| SLF4J | `org.slf4j..` | `LoggerPort` | adapter/logging |
| Jakarta EE | `jakarta..` | (按需 port) | adapter / framework |
| Lombok | `lombok..` | (避免使用) | adapter only(若必须) |
| 达梦驱动(信创) | `io.dameng..` | (复用 Repository) | adapter/persistence |
| _待 /bob-onion 填充_ | | | |

**实现方式枚举**:
- 注解模式 — Spring 注解只在 adapter / framework
- 端口抽象 — usecase/port 接口 + adapter/<env> 实现
- 装饰器收敛 — 如 `@Transactional` 仅在 `TransactionalUseCaseDecorator`

## 7. 装配点(framework/)

- **TransactionalUseCaseDecorator**(`shared.framework.transaction`):全工程**唯一** `@Transactional`,且必须 `rollbackFor = Exception.class`(R8)
- **<Feature>UseCaseConfig**(每个上下文一个):`@Bean` 装配 + 装饰器包裹

```java
@Configuration
class <Feature>UseCaseConfig {
    @Bean
    UseCase<<Cmd>, <Result>> <cmdName>UseCase(<Repo> repo, ...) {
        return new TransactionalUseCaseDecorator<>(
            new <Cmd>UseCase(repo, ...));
    }
}
```

**装饰器实现固定形态**(R8 第 2 点):

```java
@Override
@Transactional(rollbackFor = Exception.class)   // ← 不允许写成裸 @Transactional
public R execute(C cmd) { return inner.execute(cmd); }
```

> Spring 默认仅在 `RuntimeException`/`Error` 时回滚。checked Exception 不显式声明 `rollbackFor` 就会静默 commit 部分写入。装饰器是全工程事务唯一入口,这里必须覆盖所有异常。

**并发安全装配检查清单**:

- 共享可变计数端口(库存 / 余额 / 配额)是否暴露 `tryDecrease(...) -> boolean` / `restore(...)` 等**原子语义动作**?(read-modify-write 一律拒绝,具体反模式见 R9)
- 被多 UseCase 触发状态迁移的聚合(如 Order 经 pay / ship / cancel / complete)是否在 JPA 实体上加了 `@Version`?Entity 是否透传 `version` 字段?Mapper 是否双向往返?
- 是否补齐 3 类并发测试:checked 异常回滚 / 多线程并发条件 UPDATE 不超卖 / 旧 version 保存抛 `OptimisticLockingFailureException`?

## 8. α/β/γ 评级与重构计划(仅 B1/B2)

### 8.1 当前评级分布(B1)
- γ(合规):<列表 / 数量>
- β(包对了但还碰框架):<列表 / 数量>
- α(业务规则散落):<列表 / 数量>
- violation(硬违规):<列表 / 数量>

### 8.2 重构条目(B1)

| # | 类 | 评级 | 适用动作 | 优先级 | spec 编号 |
|---|---|---|---|---|---|
| _待 /bob-onion 填充_ | | | | | |

### 8.3 清洁孤岛 + Legacy ACL(B2)
- **新功能包**:`com.example.<feature>`
- **Legacy ACL 表**:见 §4 端口清单中标 ★ 的项

## 9. ArchUnit 作用域

```java
// G/B1 默认:整工程作用域(覆盖业务包 + shared)
@AnalyzeClasses(packages = "com.example", importOptions = DoNotIncludeTests.class)

// B2 清洁孤岛:必须把 shared 加进数组,否则装饰器规则评估不到
// @AnalyzeClasses(packages = {"com.example.<feature>", "com.example.shared"},
//                 importOptions = DoNotIncludeTests.class)
```

## 10. ADR(架构决策记录)

> 每次重要架构决策在此记录,后续评审与迭代时用于说明"为什么这样设计"。

### ADR-1:UseCase 用 `UseCase<C, R>` 接口 + 装饰器
- **决策**:每个 usecase implements `UseCase<C, R>`,事务由 `TransactionalUseCaseDecorator` 收敛
- **理由**:atlas Stage 5 §4.1 裸方法风格让事务无处安放;装饰器是 Bob "framework 装配"理念的工程闭合
- **替代方案**:Spring AOP `@Transactional` aspect 方式 — 拒绝,因为 usecase 必须看不见 Spring
- **日期**:<YYYY-MM-DD>

### ADR-2:端口接口归属 `usecase/port`,不放 `domain`
- **决策**:business interface 在 usecase 包;DDD 派会放 domain,本项目选纯 Bob
- **理由**:atlas Stage 5 §4.1 动作 1 明确接口在 usecase
- **日期**:<YYYY-MM-DD>

### ADR-N:<待填充>
- **决策**:
- **理由**:
- **替代方案**:
- **日期**:

## 11. 下一步
- [ ] 运行 `/bob-identify <业务描述>` 开始身份测试
- [ ] 运行 `/bob-onion` 完成 4 环设计并更新本文件
- [ ] 对每个 UseCase 运行 `/bob-spec <用例名>`
- [ ] 启动 Superpowers brainstorming 决定栈细节(若 CLAUDE.md `## 技术栈约定` 段未填)

---
*Managed by run-bob + /bob-onion skill.*
