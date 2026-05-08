---
name: bob-onion
description: |
  触发条件:用户输入 /bob-onion(默认读最新 docs/bob/01-identity-*.md),
  或 /bob-onion --identity <path> 指定 identity 文档,
  或 /bob-onion --refresh 跳过 identity 直接基于现有 ARCHITECTURE.md 增补。

  基于 /bob-identify 的输出,产出正式的 Bob 4 环架构设计:划出 4 环包结构、
  列端口清单、提取 Entity 状态机、决定装饰器边界、回写 ArchUnit 黑名单,
  并自动更新项目根目录的 ARCHITECTURE.md(4 环架构 SSoT)。
  棕地模式额外产出 α→γ 重构计划(B1)或清洁孤岛布局 + Legacy ACL(B2)。
  不写实现代码,只做架构设计。产出会被 /bob-spec 引用以生成 Superpowers spec。

  当用户说"画 4 环架构"、"设计端口"、"出重构计划"、"画洋葱图"、
  "决定状态机怎么放"时也应触发此技能。
---

# Bob 4-Ring Architecture Design Skill

## 触发

```
/bob-onion                          # 默认:读最新 docs/bob/01-identity-*.md
/bob-onion --identity <path>        # 指定 identity 文档
/bob-onion --refresh                # 跳过 identity,基于现有 ARCHITECTURE.md 增补
```

或自然语言触发:"画 4 环架构"、"设计端口"、"出重构计划"、"画洋葱图"、"决定状态机怎么放"。

## 前置条件

- `docs/bob/01-identity-*.md` 至少存在一份(否则提示先跑 `/bob-identify`)
- 例外:`--refresh` 模式只增补 ARCHITECTURE.md,不需 identity 文档

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

把身份测试的分类结果**落地为可执行的架构设计**:

1. 划出本上下文的 4 环包结构(具体到包名)
2. 列出**端口清单**(每个端口名称、签名、归属包、impl 指向)
3. 提取 **Entity 状态机**(每个 CORE/Entity 标了状态字段的对象都画状态机)
4. 决定 **TransactionalUseCaseDecorator + Framework Config** 的装配形态
5. 标注 **3 个落地动作**的应用点(接口位置反转 / 框架边界外推 / 状态机上提)
6. 棕地模式额外产出 **α→γ 重构计划**(B1)或 **清洁孤岛布局 + Legacy ACL**(B2)
7. **更新 `ARCHITECTURE.md`**(SSoT)+ **回写 ArchUnit 黑名单**(`FORBIDDEN_IN_INNER` 数组)

## 工作流(三模式共用骨架,分支处标注)

### Step O1:读入 identity 表,核对配件清单

读最新一份 `docs/bob/01-identity-*.md`,展示:
- CORE 候选 N 个(将进 entity / usecase)
- ADAPTER 候选 N 个(将进 adapter)
- FRAMEWORK 候选 N 个(将进 framework)
- TOOL 候选 N 个(可被 inner 直接 import,不需端口)
- violation 候选 N 个(B1 重构入口)

输出一句话总结后**追问**用户是否补充/修正。

### Step O2:决定本上下文包结构(三模式分支)

> **assistant**:
>
> **推测包路径**——
>
> | 模式 | 推测 |
> |---|---|
> | **G** | `com.example.<bizname>` 单一上下文,4 环子包 |
> | **B1** | 沿用现有顶级包,4 环重构;legacy 包逐步淘汰 |
> | **B2** | `com.example.<feature>` 清洁孤岛,跟 legacy 同级二级包 |
>
> **推荐**:`com.example.<bizname>` 或 `com.example.<feature>`(B2)
> **理由**:身份测试主题就是新业务领域名;G/B2 两模式默认隔离 legacy
>
> 同意吗?

### Step O3:画 4 环 + 端口清单(对话式)

对每个 CORE/UseCase 候选,**逐个**问:**这个用例需要哪些副作用端口**?

> **assistant**:
> 用例 `cancelOrder()`——
>
> **推测端口需求**:
> 1. `OrderRepository`(查/存订单)— 基于身份测试的 Repository 需求
> 2. `PaymentGateway`(已支付订单需退款)— 基于身份测试的支付通道
> 3. `InventoryClient`(取消要回写库存)— 业务描述里有
> 4. `SmsNotifier`(取消通知)— 业务描述里有
>
> **推荐端口归属**:**`usecase/port/`**
> **理由**:Bob 严派(atlas Stage 5 §4.1)把业务定义接口放在 usecase 包,Adapter 来 implement;纯 Bob 路线的标准做法
>
> 同意吗?

→ 用户答"是",自动生成端口签名草案:

```java
package com.example.<bizname>.usecase.port;

import com.example.<bizname>.entity.OrderId;
import java.util.Optional;

public interface OrderRepository {
    Optional<Order> findById(OrderId id);
    void save(Order order);
}
```

### Step O4:Entity 状态机上提(动作 3)

对每个 CORE/Entity 且**身份测试标"有状态"**的候选,跑状态机推导:

> **assistant**:
> Entity `Order`——
>
> **推测状态机**(基于身份测试中提及的事件):
>
> ```
>   CREATED ──pay──> PAID ──ship──> SHIPPED ──complete──> COMPLETED
>      │                │
>      │                └──cancel(已退款)──> CANCELLED
>      └──cancel(无退款)─────────────────> CANCELLED
> ```
>
> **推测应有的 Entity 方法**:
> - `payTo(PaymentGateway, InventoryClient)` —— `CREATED → PAID`
> - `ship(String trackingNo)` —— `PAID → SHIPPED`
> - `cancel(PaymentGateway, InventoryClient)` —— 任意未终态 → `CANCELLED`
> - `complete()` —— `SHIPPED → COMPLETED`
>
> **不变量**:
> - 状态非法迁移必须 `throw IllegalStateException`(对应 atlas §4.1 的 `ensureStatus` 模式)
> - Entity 没有 setter,字段全 final 或私有写
>
> **推荐**:照搬这个状态机进 ARCHITECTURE.md。
>
> 同意吗?

### Step O5:决定装饰器边界(@Transactional 唯一位置)

> **assistant**:
>
> **推测装饰器形态**(Bob 严派 + CLAUDE.md R8):
>
> ```java
> // shared/framework/transaction/TransactionalUseCaseDecorator.java —— 全工程唯一 @Transactional
> public class TransactionalUseCaseDecorator<C, R> implements UseCase<C, R> {
>     private final UseCase<C, R> inner;
>     public TransactionalUseCaseDecorator(UseCase<C, R> inner) { this.inner = inner; }
>     @Override @Transactional
>     public R execute(C cmd) { return inner.execute(cmd); }
> }
> ```
>
> **推荐**:
> - 引入通用 `UseCase<C, R>` 接口(`shared/usecase/UseCase.java`)
> - 每个 usecase 类 `implements UseCase<XxxCommand, XxxResult>`
> - 在 `<feature>/framework/config/<Feature>UseCaseConfig.java` 用 `@Bean` 装配,**强制走 `TransactionalUseCaseDecorator` 包裹**
>
> 同意吗?(由 run-bob init 已经预置 shared 骨架,只需在 framework/config 写装配)

→ 这一步是**关键决策点**,记入 ADR-1。

### Step O6:回写 ArchUnit 黑名单 + ARCHITECTURE.md 配件清单

> **assistant**:
>
> 身份测试发现的"应禁止于 inner 包"的配件根包:
>
> | 配件 | 根包 | 来源 |
> |---|---|---|
> | Spring | `org.springframework..` | 已在模板默认黑名单 |
> | Jakarta EE | `jakarta..` | 已在模板默认黑名单 |
> | SLF4J | `org.slf4j..` | 已在模板默认黑名单 |
> | Lombok | `lombok..` | 已在模板默认黑名单 |
> | **达梦驱动** | `io.dameng..` | 本次新增(身份测试发现) |
> | **FastJSON** | `com.alibaba.fastjson..` | 本次新增 |
>
> **推荐**:把"达梦驱动"和"FastJSON"两条加到 `CleanArchitectureTest.java` 的 `FORBIDDEN_IN_INNER`,同时在 ARCHITECTURE.md §配件清单 增加这两行项目特化条目。
>
> 同意吗?

→ skill 直接 edit 两个文件:`src/test/java/architecture/CleanArchitectureTest.java`(数组追加)+ `ARCHITECTURE.md`(表格追加)。

### Step O7(仅 B1):产出 α→γ 重构计划

对身份测试标 α / β / violation 的每个类,产出**重构条目**。表格:

| # | 类 | 评级 | 适用动作 | 工作量 | 依赖项 | 优先级 |
|---|---|---|---|---|---|---|
| 1 | `OrderApplicationService` | α | 全部 3 动作 | ~120 行 | #2 #3 | P0 |
| 2 | `OrderEntity` | α | 状态机上提 | ~60 行 | — | P0 |
| 3 | `OrderRepositoryImpl` | β | 接口反转 | ~30 行 | — | P1 |
| ... | | | | | | |

**推荐执行顺序**:`P0 → P1`,且**每条都对应一个独立 spec**(交给 Superpowers TDD 一条一条做)。

### Step O8(仅 B2):产出清洁孤岛布局 + Legacy ACL

> **assistant**:
>
> **推测清洁孤岛布局**:
>
> ```
> com.example.<feature>/
> ├── entity/
> │   ├── <Feature>.java                            # POJO,状态机自封
> │   ├── <Feature>Status.java                      # enum 值对象
> │   └── <FeatureId>.java                          # 强类型 ID
> ├── usecase/
> │   ├── Apply<Feature>Command.java                # record
> │   ├── Apply<Feature>UseCase.java                # POJO
> │   ├── Apply<Feature>Result.java                 # record
> │   └── port/
> │       ├── <Feature>Repository.java              # 端口
> │       ├── <LegacyAccess>.java                   # ★ legacy 隔离端口
> │       └── ClockPort.java                        # 时钟端口
> ├── adapter/
> │   ├── web/<Feature>Controller.java
> │   ├── persistence/Jpa<Feature>Repository.java
> │   ├── acl/<Legacy>Adapter.java                  # ★ 包装 legacy
> │   └── time/SystemClockAdapter.java
> └── framework/
>     └── config/<Feature>UseCaseConfig.java
> ```
>
> **Legacy ACL 表**:
>
> | Legacy 类 | 业务领域端口 | Adapter |
> |---|---|---|
> | `legacy.OrderApplicationService.fetchByCustomer` | `CustomerOrderHistory.findRecentBy(CustomerId, Period)` | `LegacyCustomerOrderAdapter` |
>
> **关键纪律**:legacy 方法签名是 `Page<OrderDTO> fetchByCustomer(Long, Date, Date)`——**绝不**让新 usecase 看到这个签名。新端口用业务领域语言(`Period` 替代 `Date,Date`,`OrderSnapshot` 替代 `OrderDTO`)。
>
> **ArchUnit 作用域**:
> ```java
> @AnalyzeClasses(
>     packages = {"com.example.<feature>", "com.example.shared"},
>     importOptions = DoNotIncludeTests.class
> )
> ```
> ——只对新功能强制 4 环;legacy 不波及。**关键陷阱**:必须把 `com.example.shared` 加进数组,否则 `transactional_methods_only_in_decorator` 规则评估不到装饰器。
>
> 同意整套布局吗?

### Step O9:更新 ARCHITECTURE.md

把 Step O2-O8 的所有结论写入 / 合并到 `ARCHITECTURE.md`。**不擅自修改** identity 表;**追加**新章节,不覆盖旧 ADR。

## ARCHITECTURE.md 模板(SSoT)

> 注意:`run-bob init` 已经在项目根目录创建了一个空的 ARCHITECTURE.md 模板。本 skill 的工作是**填充**它,而不是从头创建。
> 模板结构(11 节):📌 状态 / 1 上下文 / 2 4 环包结构 / 3 核心 Entity 与状态机 / 4 端口清单 / 5 UseCase 清单 / 6 配件清单 / 7 装配点 / 8 α/β/γ 评级 / 9 ArchUnit 作用域 / 10 ADR / 11 下一步。

逐节填充建议:

- §📌 状态:勾选已完成的步骤,在"模式"行三选一
- §1 上下文:从 identify 第 1 节抄业务描述提炼一句话职责
- §2 4 环包结构:替换 `<bizname>` 为实际包名(G/B2 模式独立包,B1 现有顶级包)
- §3 Entity 与状态机:每个 CORE/Entity 一节,Step O4 推导的状态机图 + 方法签名 + 不变量
- §4 端口清单:Step O3 列出的所有端口
- §5 UseCase 清单:每个 CORE/UseCase 一行,Command/Result/用到的端口
- §6 配件清单:identify 表格里所有 ADAPTER/FRAMEWORK 候选 + 默认 4 项(Spring/SLF4J/Jakarta/Lombok)+ 信创预设
- §7 装配点:照模板说明,引用 shared 骨架
- §8 α/β/γ 评级与重构计划:仅 B1/B2 填,B1 有重构表,B2 有清洁孤岛布局
- §9 ArchUnit 作用域:G/B1 默认 `com.example`;B2 用多包数组
- §10 ADR:**至少**记 ADR-1(UseCase + 装饰器)+ ADR-2(端口归属 usecase/port);棕地额外记 ADR-3(α→γ 路径)或 ADR-3(legacy ACL 决策)
- §11 下一步:列出待跑的 `/bob-spec` 用例清单

## 反模式(skill 必须拒绝)

- ❌ 跳过 `/bob-identify` 直接画 4 环(空中楼阁)
- ❌ 把端口接口放 entity 包(纯 Bob 派应放 usecase/port;若用户坚持 DDD 派,必须记 ADR)
- ❌ 装饰器之外允许 `@Transactional`(违反 R8)
- ❌ 在 ARCHITECTURE.md 里出现 framework 类型(`HttpServletRequest` / `JpaEntityManager` 等)
- ❌ B2 模式不做 Legacy ACL,让新 usecase 直接 import legacy `@Service`

## 与其他 skill 衔接

- **上游**:`docs/bob/01-identity-*.md`
- **下游**:`/bob-spec` 读 `ARCHITECTURE.md` 的端口清单 + UseCase 清单生成 spec
- **回写**:`CleanArchitectureTest.java` 的 `FORBIDDEN_IN_INNER` 数组(根据 §6 配件清单)

## 文件落位

- 设计过程记录:`docs/bob/02-onion-<topic>.md`
- 最终产出:**更新项目根目录 `ARCHITECTURE.md`**(SSoT)
- 副作用:**追加** `src/test/java/architecture/CleanArchitectureTest.java` 的黑名单数组
