---
name: bob-spec
description: |
  触发条件:用户输入 /bob-spec <用例名>(默认命令型),
  或 /bob-spec --query <查询名>(查询型读模型),
  或 /bob-spec --refactor <类名>(B1 重构型 spec)。

  读取项目根目录的 ARCHITECTURE.md,为指定用例生成一份 Superpowers
  可直接消化的 spec 文档:严格使用 ARCHITECTURE.md §3 §4 §5 中的术语
  (Entity / 端口 / UseCase),包含用例描述、前置/后置条件、业务规则、
  Given-When-Then 测试场景、纯 POJO usecase + framework Config 接口约定、
  Guardrails(给 Superpowers 实现时遵守)、和"交给 Superpowers 的开放问题"
  (技术栈决策)。

  这个 skill 是 Bob 4 环建模阶段与 Superpowers 实现阶段的桥梁。
  当用户说"生成 spec"、"出 TDD 测试场景"、"准备给 Superpowers 的输入"、
  "把这个用例写清楚"时也应触发此技能。
---

# Bob 4-Ring → Superpowers Spec Bridge Skill

## 触发

```
/bob-spec <用例名>                  # 默认:命令型
/bob-spec --query <查询名>          # 查询型(读模型)
/bob-spec --refactor <类名>         # B1 重构型
```

或自然语言触发:"生成 spec"、"出 TDD 测试场景"、"准备给 Superpowers 的输入"、"把这个用例写清楚"。

## 前置条件

- 项目根目录存在 `ARCHITECTURE.md` 且 §4 端口清单 + §5 UseCase 清单已填(否则提示先跑 `/bob-onion`)
- 例外:`--refactor` 模式只要求 §8 重构条目存在

## 适用范围:命令 / 查询 / 重构 三类

skill 自动识别用例类型,走不同模板:

| 类型 | 触发词模式 | 模板 |
|---|---|---|
| **命令(Command)** | `Pay`/`Cancel`/`Apply`/`Submit`/`Refund` 等动词前缀 | 模板 A(状态变化 + 事务装饰器) |
| **查询(Query)** | `View`/`List`/`Find`/`Browse`/`Get` 等动词前缀 | 模板 B(读模型,可绕端口直接 SQL) |
| **重构(Refactor)** | `--refactor <类>`,B1 模式 | 模板 C(α/β 现状 + 3 动作改造路径 + 测试覆盖路径) |

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

## 目标

把一个用例 / 一个查询 / 一个重构动作转化为 Superpowers 可消化的 spec,满足三个硬约束:

1. **术语锚定**:所有命名引用 `ARCHITECTURE.md` §3 / §4 / §5
2. **测试友好**:Given-When-Then 场景,Superpowers 可直接转 JUnit
3. **4 环纪律**:接口约定段强制展示纯 POJO usecase + framework Config 装配,**不允许**出现 `@Service @Transactional` 在 usecase 类上

## 工作流

### Step S1:读 ARCHITECTURE.md,定位用例归属

> **assistant**(用户输入 `/bob-spec PayOrder`):
>
> **推测用例归属**:
> - **Entity**:`Order`(从 §3 状态机 `CREATED → PAID` 那条迁移)
> - **状态前置**:`CREATED`
> - **状态后置**:`PAID`
> - **用到的端口**:`OrderRepository` / `PaymentGateway` / `InventoryClient` / `SmsNotifier`(从 §5 UseCase 清单)
> - **本用例 spec 落点**:`docs/specs/spec-1-pay-order.md`
>
> **推测 spec 类型**:**命令型**(动词 `Pay` 触发状态变化)→ 走模板 A
>
> 同意吗?

### Step S2:跨 Entity 检测

跨 Entity 风险检测——若用例触发的副作用涉及多个 Entity,**主动追问**:

> **推荐**:本 spec 只覆盖单一 Entity 的状态迁移 + 端口的同步调用,不引入 Domain Event(Bob 假设单 BC + 同步)。
>
> **理由**:atlas Stage 6 §2.5 显示——Bob 风格 = sync 调用;DDD 风格 = async 事件。我们走纯 Bob,事件驱动留给将来真有跨 BC 需求时升级到 DDD 战术级再加。
>
> 同意吗?(若你坚持要异步事件,告诉我,我加一段 `Outbox` 设计,但这会偏离纯 Bob 并触发 R10-bob)

### Step S3:渲染对应模板(A / B / C)

skill 直接渲染下面三个模板之一,**不让用户从空白开始**。用户可逐节确认。

### Step S4:回写检测(防漂移)

若 spec 暴露了 ARCHITECTURE.md §4 端口清单中**未列出**的端口需求,**先停下** spec 生成,让用户确认是否回到 `/bob-onion --refresh` 加这一条端口,再继续 spec。

> **推测**:这是漏标。
> **推荐**:**先停下** spec 生成,让你确认是否回到 `/bob-onion --refresh` 加这一条端口,再继续 spec。
>
> 同意吗?(回"是"我等你跑 `/bob-onion --refresh`;回"先继续 spec,端口我事后补"我标记 TODO 继续)

→ 这一步是关键纪律,防止 spec 引入 ARCHITECTURE.md 没登记的术语。

---

## 模板 A:命令型 spec(写入 `docs/specs/spec-<n>-<slug>.md`)

````markdown
# Spec: <用例名>

> 本 spec 由 /bob-spec 生成。所有术语锚定到 ARCHITECTURE.md。
> 实现交给 Superpowers TDD 流程驱动。
> 模式:<G | B1 | B2>(从 ARCHITECTURE.md §状态读取)

## 1. 归属
- **Entity**:`Order`(ARCHITECTURE.md §3.1)
- **状态迁移**:`CREATED` → `PAID`
- **分层**:UseCase 编排 + Entity 内部业务规则
- **包路径**:`com.example.<bizname>.usecase.PayOrderUseCase`

## 2. 用例描述
<一段自然语言,只用 ARCHITECTURE.md 的术语>

## 3. 参与者
- <Actor>:<角色>

## 4. 前置条件
- Order 存在且状态为 `CREATED`
- ...

## 5. 后置条件(成功路径)
- Order.status = `PAID`,paidAt 被赋值
- PaymentGateway.pay 已调用并返回成功
- InventoryClient.decrease 已调用
- SmsNotifier.notifyOrderPaid 已发送
- Repository.save 持久化 Order

## 6. 业务规则(Entity 不变量)
- **INV-1**(来自 ARCHITECTURE.md §3.1):非 `CREATED` 状态调用 `payTo` 必抛 `IllegalStateException`
- **RULE-本用例-1**:支付通道返回失败时,**不**改 Order 状态,直接抛业务异常

## 7. 测试场景(Given-When-Then)

### 场景 1:成功路径
- **Given** Order 处于 `CREATED`,金额 100 元,商品 X 数量 2
- **When** 调用 `PayOrderUseCase.execute(PayOrderCommand(orderId))`
- **Then**
  - PaymentGateway.pay 被调用 1 次,参数 (Money(100), userId)
  - InventoryClient.decrease 被调用 1 次,参数 (X, 2)
  - SmsNotifier.notifyOrderPaid 被调用 1 次
  - Order.status == `PAID`,Order.paidAt 非空
  - Repository.save 被调用 1 次

### 场景 2:状态非法
- **Given** Order 处于 `SHIPPED`
- **When** 调用 `PayOrderUseCase.execute`
- **Then** 抛 `IllegalStateException`,消息含 "已支付/已取消订单不能再支付"
  - PaymentGateway.pay **未被**调用
  - Repository.save **未被**调用

### 场景 3:端口失败
- **Given** Order 处于 `CREATED`,PaymentGateway mock 抛 `PaymentDeclinedException`
- **When** 调用
- **Then** 异常向上抛;Order 状态保持 `CREATED`;装饰器事务回滚

### 场景 4:事务回滚(checked Exception 也必须回滚)
- **Given** Order 处于 `CREATED`,InventoryClient.decrease 抛 `InsufficientStockException`(假设 checked)
- **When** 调用
- **Then** 异常向上抛;Order 不被持久化为 PAID(事务回滚);装饰器 `@Transactional(rollbackFor = Exception.class)` 保证 checked 异常同样回滚
- **额外断言**:`TransactionalUseCaseDecorator` 的单测里必须有一对孪生 case——一条抛 `RuntimeException`、一条抛 checked `Exception`,两条都断言事务回滚。少一条说明装饰器有静默 commit 漏洞

### 场景 5:并发不超卖(共享计数端口的硬场景)
> 仅当用例触达**共享可变计数端口**(Inventory / Wallet / QuotaCounter…)时必填。
- **Given** 商品 X 库存 1,50 个线程并发请求 `PayOrderUseCase.execute(PayOrderCommand(orderId-i))`
- **When** 全部并发执行
- **Then** 恰好 1 个请求成功(Order PAID + 库存 = 0),其余 49 个抛"库存不足"业务异常;**不允许超卖**
- **实现要求**:`InventoryRepository.tryDecrease` 必须是 adapter 层一条 `UPDATE ... SET qty = qty - :q WHERE qty >= :q` 的原子 SQL,UseCase 仅判 boolean——**禁止** `findByX → mutate → save` 的 read-modify-write

### 场景 6:并发状态迁移(@Version 乐观锁)
> 仅当 Entity 被**≥2 个 UseCase**触发状态迁移时必填(如 Order 经 pay/ship/cancel/complete)。
- **Given** Order 已加载到内存(version=N),期间另一线程已经把数据库行更新到 version=N+1
- **When** 当前线程 `repo.save(order)` 写回(仍带 version=N)
- **Then** 抛 `OptimisticLockingFailureException`(由 JPA `@Version` 触发);上层装饰器回滚
- **实现要求**:JPA 实体加 `@Version long version`,Entity 透传 `version` 字段,mapper 双向往返

## 8. 接口约定

> 严格遵守 4 环 Clean Architecture(CLAUDE.md R7-R12)。

### Command(usecase 层 record,纯 Java)— 落 `usecase/in/`

```java
package com.example.<bizname>.usecase.in;

public record PayOrderCommand(
    String orderId
) {}
```

> 包结构纪律:`*Command` / `*Query` 一律放 `usecase/in/`;`*Result` 一律放 `usecase/out/`;
> `usecase/` 根目录只放 `*UseCase.java` 编排类与 `port/` 子包。混在根目录视为违规(CLAUDE.md R7/R9)。

### Result(usecase 层 record,纯 Java)— 落 `usecase/out/`

```java
package com.example.<bizname>.usecase.out;

public record PayOrderResult(
    String orderId,
    String status,
    String paidAt
) {}
```

### UseCase 实现(usecase 层 POJO,**零 Spring,零 SLF4J**)

```java
package com.example.<bizname>.usecase;

import com.example.<bizname>.entity.*;
import com.example.<bizname>.usecase.in.PayOrderCommand;
import com.example.<bizname>.usecase.out.PayOrderResult;
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
    this.paidAt = clock.now();
}
```

### 装配(framework 层,**唯一**事务点)

```java
package com.example.<bizname>.framework.config;

import com.example.shared.framework.transaction.TransactionalUseCaseDecorator;
import com.example.<bizname>.usecase.*;
import com.example.<bizname>.usecase.port.*;
import com.example.shared.usecase.UseCase;
import org.springframework.context.annotation.*;

@Configuration
class OrderUseCaseConfig {
    @Bean
    UseCase<PayOrderCommand, PayOrderResult> payOrderUseCase(
            OrderRepository repo,
            PaymentGateway pg,
            InventoryClient ic,
            SmsNotifier sn) {
        return new TransactionalUseCaseDecorator<>(
            new PayOrderUseCase(repo, pg, ic, sn));
    }
}
```

### Controller(adapter/web,**只 import usecase 包**)

```java
package com.example.<bizname>.adapter.web;

import com.example.<bizname>.usecase.*;
import com.example.shared.usecase.UseCase;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/orders")
class OrderController {
    private final UseCase<PayOrderCommand, PayOrderResult> payOrder;

    OrderController(UseCase<PayOrderCommand, PayOrderResult> payOrder) {
        this.payOrder = payOrder;
    }

    @PostMapping("/{id}/pay")
    PayOrderResult pay(@PathVariable String id) {
        return payOrder.execute(new PayOrderCommand(id));
    }
}
```

> 注意:Controller 严禁 `import com.example.<bizname>.entity.*`(R9)。

## 9. Guardrails(给 Superpowers 实现时遵守)

参考 CLAUDE.md R0 / R7-R12:

- ❌ 业务规则不得写进 Controller / UseCase 方法体(必须在 Entity)
- ❌ UseCase 类不得 import `org.springframework.*` / `jakarta.*` / `org.slf4j.*` / `lombok.*`
- ❌ UseCase 类不得加任何注解(包括 `@Service` `@Transactional`)
- ❌ UseCase 不得调 `LoggerFactory.getLogger`(用 `LoggerPort`)
- ❌ Entity 不得 `LocalDateTime.now()`(用注入的 `ClockPort`)
- ❌ `@Transactional` 必须仅在 `TransactionalUseCaseDecorator`,且形态固定为 `@Transactional(rollbackFor = Exception.class)`(裸 `@Transactional` 不接受 checked 异常回滚)
- ❌ 共享可变计数端口(Inventory / Wallet / 配额)在 UseCase 里 `findByX → mutate → save` —— read-modify-write 在并发下超卖。改用 adapter 层原子 `UPDATE ... WHERE qty >= :q`,端口暴露 `tryDecrease(...) -> boolean` / `restore(...)`
- ❌ 被多 UseCase 触发状态迁移的聚合(Order pay/ship/cancel/complete 类)缺失 `@Version` 乐观锁(并发 last-writer-wins 静默丢更)
- ❌ `*Command` / `*Query` record 落在 `usecase/` 根目录(必须 `usecase/in/`);`*Result` 同理(必须 `usecase/out/`)
- ❌ Entity 字段无 setter
- ❌ Repository 实现不得放在 `framework/`(应在 `adapter/persistence/`)
- ✅ TDD 节奏:每个测试场景写一个测试,先红再绿再重构
- ✅ 实现完一个 UseCase 后必须自检:
  ```bash
  grep -rE "org\.springframework|jakarta\.|org\.slf4j|lombok\." \
    src/main/java/com/example/<bizname>/usecase/   # 期望零命中
  grep -nE "@Transactional[^(]" \
    src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java
    # 期望 0 命中(必须是 @Transactional(rollbackFor = Exception.class))
  ```
- ✅ 跑 ArchUnit:`mvn test -Dtest=CleanArchitectureTest`(或 Gradle 等同) 期望全绿
- ✅ **遇到任何新 import**(spec 没列出的)→ **停下**,跑 5 问决策树,违反 R0 必须先抽端口

## 10. 交给 Superpowers 的开放问题(技术实施层面)

本 spec **不回答**以下问题,留给 `superpowers:brainstorming` 在进入 `writing-plans` 之前回答,并写回 `CLAUDE.md ## 技术栈约定`:

- 语言/运行时(Java 17 / Kotlin / Go / ...)
- 应用框架(Spring Boot / Ktor / ...)
- 持久化(JPA / MyBatis / JOOQ / 信创数据库?)
- 范围(仅后端 / 前后端全栈?)
- 对外交互(REST / gRPC / GraphQL / CLI / 消息?)
- 测试框架 / 构建工具
- 部署形态(单体 / modulith / 微服务)
- 非功能(并发量级 / 延迟预算 / 数据规模)

> 首次进入实现阶段必答;若 `CLAUDE.md ## 技术栈约定` 已填,跳过直接进 `writing-plans`。

## 11. 下一步

```
1. superpowers:brainstorming(首次):栈决策写回 CLAUDE.md
2. superpowers:writing-plans:基于本 spec 出分步计划
3. superpowers:executing-plans + TDD:红 → 绿 → 重构
4. superpowers:finishing-a-development-branch:验全绿 → 合并 / PR
```

实现自检两步(每个 UseCase 写完后必做):
1. `grep -rE "org\.springframework|jakarta\.|org\.slf4j|lombok\." src/main/java/.../usecase/` → 零命中
2. `mvn test -Dtest=CleanArchitectureTest` → 全绿
````

---

## 模板 B:查询型 spec(简化版)

只在跨上下文联合 / 投影 / 分页 / 独立 Read Model 表 时使用。**简单单 Entity findById + DTO 映射不需要 spec**。结构:

````markdown
# Query Spec: <查询名>

> 本 spec 由 /bob-spec 生成(查询模板),用于复杂查询。
> 简单单 Entity findById 查询不需要 spec,直接参照 ARCHITECTURE.md §4 实现即可。

## 归属
- **类型**:**读模型查询**(不改状态,无不变量保护)
- **实现方式**:<单 Entity 直查 / 跨上下文联合 / 绕端口 SQL / 独立 Read Model 表>
- **涉及 Entity / 表**:<...>

## 用例描述

## 输入(Query Params)

## 输出(DTO)
```
<DTO 名称> {
  <字段>: <类型>
  ...
}
```

## 约束
- 权限 / 分页 / 一致性 / 性能期望

## 测试场景

### 场景 1:基本查询
### 场景 2:空结果
### 场景 3:权限 / 分页边界

## 接口约定

### REST 端点
```
GET /<resource>[/{id}][?param=...]
Response 200: <DTO>
Response 404: { error, message }
```

### Application 入口签名
```java
<QueryService>.<method>(<QueryParams>) -> <DTO>
```

## 禁止项
- ❌ 在查询路径里改状态(无副作用)
- ❌ 查询 DTO 泄露 Entity 内部可变对象(返回 record / 不可变 VO)
- ❌ 为优化绕过 Entity 时仍拼装业务规则(规则归写侧)
- ✅ 查询可直接 SQL,但 DTO 字段必须在 ARCHITECTURE.md §6 配件清单可追溯

## 下一步
查询实现通常短小(Application Service + DTO + Controller GET 端点),
不一定需要完整 Superpowers 全流程;可直接实现 + 集成测试验证。
````

---

## 模板 C:重构型 spec(B1 模式专用)

针对一个 α/β 类,产出"现状 → 目标 → 三动作步骤 → TDD 覆盖"。

````markdown
# Refactor Spec: <类名>

## 1. 现状(α/β)
- **类**:`com.example.legacy.OrderApplicationService`
- **评级**:α(身份测试第 N 行)
- **关键违规**:
  - `@Service @Transactional` 在 Service 上
  - SQL `WHERE status != 'SHIPPED'` 隐含状态机
  - Lombok `@Slf4j` 注入 Logger
  - 直接 import `JdbcTemplate`

## 2. 目标(γ)
- **包重定向**:`com.example.<bizname>.usecase.CancelOrderUseCase`
- **3 动作落点**:
  1. **接口位置反转**:抽 `OrderRepository`(usecase/port) — 替代 JdbcTemplate
  2. **框架边界外推**:删除 `@Service` `@Transactional` `@Slf4j`,用装饰器 + LoggerPort
  3. **状态机上提**:把 `WHERE status != 'SHIPPED'` 移到 `Order.cancel(...)`,Entity 内 `ensureStatus` 守

## 3. 改造步骤(原子化,每步一次提交)

### Step 1:加测试覆盖现状(防止改坏)
- 写集成测试覆盖现 `OrderApplicationService.cancel`,记录现行为基线
- **禁止**这一步删任何代码

### Step 2:抽 `OrderRepository` 端口
- 新建 `usecase/port/OrderRepository.java`
- 新建 `adapter/persistence/JdbcOrderRepository.java` implements,内部仍用 JdbcTemplate
- 让 `OrderApplicationService` 依赖端口,不直接依赖 JdbcTemplate
- 跑测试:基线绿 → 通过

### Step 3:状态机上提到 Order
- 在 `entity/Order.java` 新增 `cancel(PaymentGateway, InventoryClient)` 方法
- 把 SQL `WHERE status != 'SHIPPED'` 翻译成 `ensureStatus(...)` Java 表达
- Service 改调 `order.cancel(...)`,SQL 改为单纯 `UPDATE order SET ...`
- 跑测试:基线绿 → 通过

### Step 4:框架边界外推
- 新建 `CancelOrderUseCase` POJO,迁移 Service 编排逻辑
- 新建 `framework/config/OrderUseCaseConfig.java`,@Bean + 装饰器装配
- 删除 `@Service` `@Transactional` `@Slf4j`,引入 `LoggerPort`
- 跑测试 + ArchUnit:全绿 → 通过

### Step 5:删除 legacy Service
- 现 Controller 改注入 `UseCase<CancelOrderCommand, ...>`
- 删除 `OrderApplicationService.java`
- 最终验证:`grep` + ArchUnit + 集成测试

## 4. 测试场景(Given-When-Then)

### 场景 1:成功路径
### 场景 2:状态非法
### 场景 3:端口失败
### 场景 4:事务回滚

### 场景 5:回归基线
- **Given** Step 1 录制的所有基线行为
- **When** 重构完成
- **Then** 所有基线 case 行为不变(refactor = 不改业务的代码迁移)

## 5. Guardrails / 下一步
- 同模板 A §9 的反模式硬清单
- **B1 模式重构每完成一步,必须 git commit;严禁批量合并**
````

---

## 反模式(skill 必须拒绝)

- ❌ spec 里出现 ARCHITECTURE.md 之外的端口名 / Entity 名(必须先 `/bob-onion --refresh`)
- ❌ 命令 spec 里同时引入 Domain Event(那是 DDD,不是 Bob)
- ❌ 只有 happy path,没有状态非法 / 端口失败 / 事务回滚场景
- ❌ 命令 spec 触达共享可变计数端口却**没有**"并发不超卖"场景(场景 5)
- ❌ 被多 UseCase 触发状态迁移的聚合的 spec **没有**"并发版本冲突"场景(场景 6)
- ❌ 装饰器事务回滚场景只测 `RuntimeException`,漏掉 checked `Exception`(R8 第 5 点要求孪生 case)
- ❌ Command / Query / Result 的 package 写成 `com.example.<bizname>.usecase` 根包(必须 `usecase.in` / `usecase.out`)
- ❌ Given-When-Then 用技术语言("调用 API"、"写入 MySQL")替代业务语言("Order 状态变 PAID")
- ❌ 简单 GET 接口也强行跑 spec(ARCHITECTURE.md §4 有条件:跨上下文 / 分页 / 投影 才需 spec)
- ❌ 重构 spec 把多个动作合并一步("3 个动作一次性改完" — 必须分原子步骤,每步可回滚)

## 与其他 skill 衔接

- **上游**:`ARCHITECTURE.md`(SSoT)
- **下游**:`superpowers:brainstorming` → `writing-plans` → `executing-plans` → `finishing-a-development-branch`
- **副作用**:可能触发回写 `/bob-onion --refresh` 补端口

## 文件落位

`docs/specs/spec-<自增序号>-<slug>.md`(目录不存在自动建)
