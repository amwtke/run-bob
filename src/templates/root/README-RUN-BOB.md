# Bob 4 环 Clean Architecture + Superpowers Harness

> 本文档由 `run-bob init` 生成,介绍本项目如何使用 Bob 4 环 Clean Architecture + Superpowers 协作开发。

---

## Clean Architecture 速览(Bob 同心圆 4 环)

本 harness 强制遵守 4 环向内依赖。详见 CLAUDE.md R0-R12。

```
entity     Ring 1 — POJO 状态机             (零框架,纯 Java)
usecase    Ring 2 — Interactor + port/      (POJO,零 Spring,零 SLF4J)
adapter    Ring 3 — Controller / Repo impl  (允许 Spring/JPA/SDK)
framework  Ring 4 — 装配 + 事务装饰器 + main
```

### atlas Stage 5 §4 三个落地动作

1. **接口位置反转**:business interface 在 `usecase/port`,Adapter 来 implement
2. **框架边界外推**:UseCase 类零 Spring / 零 SLF4J / 零 Jakarta / 零 Lombok
3. **状态机上提**:业务规则在 Entity 内部,不在 Service

事务 / 装配 / 事件订阅都收敛在 ring 3-4。**唯一的 `@Transactional`** 出现在
`shared.framework.transaction.TransactionalUseCaseDecorator`(由 `run-bob init` 安装为 shared 骨架)。

### 范例:CreateOrder 的层级分布

```java
// usecase/PayOrderUseCase.java —— Ring 2,零 Spring import
public class PayOrderUseCase implements UseCase<PayOrderCommand, PayOrderResult> {
    private final OrderRepository repo;
    private final PaymentGateway paymentGateway;
    public PayOrderUseCase(OrderRepository r, PaymentGateway pg) {
        this.repo = r; this.paymentGateway = pg;
    }
    @Override
    public PayOrderResult execute(PayOrderCommand cmd) {
        // 翻译输入 + 调 Entity 方法 + 保存,业务规则在 Entity
        Order order = repo.findById(OrderId.of(cmd.orderId())).orElseThrow();
        order.payTo(paymentGateway, /* InventoryClient */);
        repo.save(order);
        return new PayOrderResult(order.id().value(), order.status().name(), null);
    }
}
```

```java
// framework/config/OrderUseCaseConfig.java —— Ring 4,装配点
@Configuration
class OrderUseCaseConfig {
    @Bean
    UseCase<PayOrderCommand, PayOrderResult> payOrderUseCase(
            OrderRepository repo, PaymentGateway pg) {
        return new TransactionalUseCaseDecorator<>(
            new PayOrderUseCase(repo, pg));
    }
}
```

### ArchUnit 守卫

`src/test/java/architecture/CleanArchitectureTest.java` 是 4 环规则的机械执法者
(由 `run-bob init` 生成)。**请勿删除其中已有规则**;可在末尾追加项目特定规则。
CI 必须将这个测试设为合并门槛。

引入新外部库时,跑 5 问决策树 → 若判为配件,把根包加到 `FORBIDDEN_IN_INNER` 数组。

## 一、这套 harness 是什么?

当你使用 Claude Code / Cursor 等 AI 工具开发时,最常见的问题是:
- AI 直接产出"能跑但架构不对"的代码(贫血模型、跨层调用)
- 术语不一致,同一个概念有三四种命名
- 一次性生成一大坨代码,难以迭代

本 harness 通过**三个 Claude Code skills + 两份锚点文档 + 两个 shared Java 骨架 + 一个 ArchUnit 守卫**,把 Bob 4 环架构原则
和 Superpowers 的 TDD 实现流程串起来,让 AI 产出**符合架构意图**的代码。

### 三种入口模式

| 模式 | 场景 |
|---|---|
| G(Greenfield) | 全新项目。从业务描述 → 4 环架构 → spec |
| B1(Brownfield 全量重构) | 已有 α/β 代码,要改成 γ |
| B2(Brownfield 增量新功能) | 已有项目 + 新需求。**新代码必须 γ,legacy 不动**;新功能在 legacy 中开"清洁孤岛" |

## 二、目录布局

```
your-project/
├── .claude/
│   └── skills/
│       ├── bob-identify/SKILL.md     # 🔍 身份测试 — 5 问决策树
│       ├── bob-onion/SKILL.md        # 🧅 4 环设计 — 维护 ARCHITECTURE.md
│       └── bob-spec/SKILL.md         # 📝 用例 spec → Superpowers 桥接
├── CLAUDE.md                          # 🛡 项目级硬约束(R0-R12)
├── ARCHITECTURE.md                    # 📘 4 环架构 SSoT
├── README-RUN-BOB.md                  # 📖 本文档
├── docs/
│   ├── bob/                           # identify/onion 中间产物
│   └── specs/                         # bob-spec 输出
└── src/
    ├── main/java/com/example/shared/
    │   ├── usecase/UseCase.java                        # 通用 UseCase<C, R>
    │   └── framework/transaction/
    │       └── TransactionalUseCaseDecorator.java      # 全工程唯一 @Transactional
    └── test/java/architecture/
        └── CleanArchitectureTest.java                  # ArchUnit 守卫
```

## 三、两份锚点文档

### 3.1 `CLAUDE.md` — 项目级硬约束

这份文档是 Claude Code 的**最高优先级规则**。Claude Code 在本项目的每一次操作
都会读这份文档。它定义:

- 技术栈(由 `superpowers:brainstorming` 决定后写回)
- 分层架构(Bob 4 环:entity / usecase / adapter / framework)
- **强制规则 R0-R12**:
  - R0:通用判定优先于具体清单(5 问决策树)
  - R1-R6:战略层先行 + 富 Entity + Repository + TDD
  - R7-R9:包结构 + 装配 + 反模式硬清单
  - R10-bob:跨上下文 / 异步 = 升级触发器(默认拒绝 Domain Event)
  - R11:ArchUnit 守卫
  - R12:B2 清洁孤岛

**作用**:让 AI 不用每次都告诉它"别写贫血模型"、"@Transactional 别乱放",而是一次性在 CLAUDE.md 立规矩。

### 3.2 `ARCHITECTURE.md` — 4 环架构的 SSoT

这份文档是本项目 4 环架构的**唯一真实源**,包含:

- 限界上下文定义(单 BC,Bob 假设)
- **核心 Entity 与状态机**(每个 Entity 自封状态迁移)
- **端口清单**(usecase/port/ 接口名 + 签名 + adapter 实现)
- **UseCase 清单**(Command record + Result record + 用到的端口)
- **配件清单**(项目特化,跑过 5 问决策树后识别出的配件根包)
- **ADR**(架构决策记录)

**作用**:
- **代码命名锚定**:所有 class 名、方法名、DTO 字段都必须引用 ARCHITECTURE.md 中的术语
- **跨会话一致性**:即使多次会话、多人协作,术语都保持一致
- **评审/协作沟通**:这份文档就是团队共享的"架构可视化成果"

**修改权限**:`ARCHITECTURE.md` 由 `/bob-onion` 管理,其他 skill 和 Superpowers **不得擅自修改**。
如果实现过程中发现 ARCHITECTURE.md 有缺失,要**停下来回到 `/bob-onion` 修正**。

## 四、三个 Skill 的使用

### 4.1 `/bob-identify` — 身份测试(第一步)

**何时用**:拿到一段业务需求 / 已有代码 / 新功能描述,还没开始设计。

**用法**(三种模式):
```
/bob-identify <业务描述>           # 模式 G:绿地新项目
/bob-identify --refactor [path]    # 模式 B1:对已有代码做全量身份测试
/bob-identify <新功能描述>         # 模式 B2:auto-detect(已有 src/main/java)
```

**产出**:
- `docs/bob/01-identity-<topic>.md`
- 包含:5 问决策树分类表(每个候选概念 / import / 注解 → CORE / ADAPTER / FRAMEWORK / TOOL / 违规)
- B1 额外:α/β/γ 评级 + 重构优先级
- B2 额外:清洁孤岛包路径 + Legacy 隔离接口清单

**关键**:这一步**不画架构,只识别核心 vs 配件**。

### 4.2 `/bob-onion` — 4 环架构设计(第二步)

**何时用**:身份测试完成后,开始画 4 环。

**用法**:
```
/bob-onion                # 默认:读最新 docs/bob/01-identity-*.md
/bob-onion --refresh      # 增补已有 ARCHITECTURE.md
```

**产出**:
- `docs/bob/02-onion-<topic>.md`(设计过程记录)
- **更新根目录 `ARCHITECTURE.md`**(SSoT)
- **回写** `CleanArchitectureTest.java` 的 `FORBIDDEN_IN_INNER` 黑名单

**关键**:
- 划出 4 环包结构 + 端口清单 + Entity 状态机 + 装配点
- 严格遵守接口位置反转(端口在 usecase/port,不在 entity)
- 决定 TransactionalUseCaseDecorator 装配方式
- 对话式设计,每个端口 / Entity 成型后确认

### 4.3 `/bob-spec` — Spec 生成(桥接 Superpowers)

**何时用**:ARCHITECTURE.md 设计完毕,要开始实现某个用例。

**用法**(三种模板):
```
/bob-spec <用例名>                  # 命令型(改状态)
/bob-spec --query <查询名>          # 查询型(读模型)
/bob-spec --refactor <类名>         # B1 重构型
```

**产出**:
- `docs/specs/spec-<n>-<slug>.md`
- 包含:用例描述、前置/后置条件、Entity 不变量、**Given-When-Then 测试场景**、纯 POJO usecase + framework Config 接口约定、Guardrails、交给 Superpowers 的开放问题

**关键**:产出的 spec **严格使用 ARCHITECTURE.md 中的术语**,可直接喂给 Superpowers。

## 五、完整工作流

```
┌────────────────────────────────────────────────────────────────────┐
│  Step 1:业务需求 / 已有代码 / 新功能(口头/文档)                   │
│         ↓                                                           │
│  Step 2:/bob-identify <需求>      ──→ docs/bob/01-identity-*.md   │
│         ↓                                                           │
│  Step 3:/bob-onion                ──→ ARCHITECTURE.md(SSoT)      │
│         ↓                                                           │
│  Step 4:/bob-spec <用例>          ──→ docs/specs/spec-*.md        │
│         ↓                                                           │
│  Step 5:superpowers:brainstorming(**首次必做**)                   │
│         │   决定:技术栈 / 前后端范围 / 交互形态 / 测试栈           │
│         │   产出:写回 CLAUDE.md 的 "## 技术栈约定" 段              │
│         ↓                                                           │
│  Step 6:superpowers:writing-plans ──→ docs/superpowers/plans/*.md │
│         ↓                                                           │
│  Step 7:superpowers:executing-plans + TDD → 测试 → 实现           │
│         ↓                                                           │
│  Step 8:superpowers:finishing-a-development-branch                │
│         ↓                                                           │
│  Step 9:回到 Step 4 处理下一个用例                                │
│         (技术栈已定,可跳过 Step 5)                              │
└────────────────────────────────────────────────────────────────────┘
```

run-bob 负责战略层(身份测试 → 4 环设计 → 用例 spec);
Superpowers 负责战术层(技术栈决策 → 计划 → TDD 实施 → 收尾)。

## 六、一个完整示例

假设你要做"订单系统 PayOrder 用例",从零开始(模式 G):

```bash
# 1. 项目初始化(已做过一次)
run-bob init

# 2. 打开 Claude Code,开始身份测试
> /bob-identify 订单系统:用户支付订单,扣库存,发短信通知。已支付订单不能再支付。

# 3. Claude 跑 5 问决策树,把 Order/PaymentGateway/SLF4J 等分类
#    产出 docs/bob/01-identity-order.md

# 4. 开始 4 环设计
> /bob-onion

# 5. 对话式确认端口清单 + Order 状态机,Claude 更新 ARCHITECTURE.md

# 6. 对第一个用例生成 spec
> /bob-spec PayOrder

# 7. Claude 产出 docs/specs/spec-1-pay-order.md

# 8. 首次进入实现阶段:Superpowers brainstorming 决定栈
> 请启动 superpowers:brainstorming,基于 spec-1 末尾的"交给 Superpowers 的开放问题"
> 确定:语言/框架/持久化、是否含前端、交互形态、测试栈。
> 决策结果写回 CLAUDE.md 的 "## 技术栈约定" 段。

# 9. Superpowers writing-plans 产出分步实施计划

# 10. Superpowers executing-plans 按 TDD 逐步实施,finishing-branch 收尾

# 11. 回到 Step 6 处理下一个用例(栈已定,跳过 Step 8)
```

## 七、常见问题

**Q: 可以跳过 `/bob-identify` 直接 `/bob-onion` 吗?**
A: 简化版可以(需求很清晰时)。但建议至少跑一次 5 问决策树,防止漏掉副作用追问 / legacy 复用点。

**Q: ARCHITECTURE.md 什么时候会被修改?**
A: 只有 `/bob-onion` 会修改它。其他阶段发现问题要停下来回到 `/bob-onion`。

**Q: 我已有 Spring 项目要加新功能,要全部重构吗?**
A: 不必。用 **B2 模式**开"清洁孤岛":新功能落在独立包(如 `com.example.<feature>`),严格 4 环;legacy 不动,通过 usecase/port 端口 + adapter/acl ACL 隔离访问 legacy。

**Q: 和 Superpowers 的全流程怎么衔接?**
A: `/bob-spec` 产出的 spec **末尾列出了技术栈 / 前后端范围 / 交互形态等 how 问题**。
   首次进入实现阶段时,先用 `superpowers:brainstorming` 回答这些问题,把决策写回 `CLAUDE.md` 的 "## 技术栈约定" 段;
   之后依次:`writing-plans` → `executing-plans` (TDD) → `finishing-a-development-branch`。
   后续用例如果技术栈不变,从 spec 直接进 `writing-plans`。

**Q: 我用什么语言?**
A: 由 Superpowers brainstorming 跟你对话决定,不由本 harness 预设。
   决策写回 `CLAUDE.md`,其中的 "## 技术栈约定" 段默认留空,R7 包结构只是 Java/Spring 的示例。
   skills 本身(bob-identify / bob-onion / bob-spec)完全与语言无关,但 ArchUnit 仅 JVM。

**Q: 我只需要做个原型/小脚本,真的要走完这么多步吗?**
A: 不必。本 harness 的目标是长期迭代的领域系统。一次性小工具直接用 Superpowers 即可,不需要 run-bob。

**Q: run-bob 跟 ddd-run 是什么关系?**
A: 互补。`ddd-run` 走 DDD 战术级(聚合根 / 限界上下文 / Domain Event),`run-bob` 走纯 Bob 4 环(单 BC + 同步 + 无 Domain Event)。中等复杂度选 run-bob,跨多个事业部 / 异步业务选 ddd-run。

---
*Generated by run-bob. 如需更新 harness,重新运行 `run-bob init --force`。*
