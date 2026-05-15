# run-bob · `/bob-model` 交互式 review 设计(html 升级为可评论 canvas)

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-15
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 实施目标:把 `/bob-model` 产物从只读 md/html 升级为带评论 / 跳转 / 反馈控件的"画布",模仿 superpowers brainstorming 的 visual companion 体验。
> 相关 spec:
> - `docs/superpowers/specs/2026-05-14-bob-model-design.md`(原 model 设计,本 spec 在其基础上升级 html + 调整 Stage 2/3/3.5/4)
> - `docs/superpowers/specs/2026-05-15-bob-stories-design.md`(下游,消费 model 产物)
> 已 lock 的相关规则:
> - commit `16b271e` 强制阶段(必跑 model)
> - commit `c8e3aaa` 命名表意强制
> - commit `6e21e53` 命名示例非教条 + Stage 3.5 多轮修改循环
> - commit `31dd597` 报告必含文件链接

---

## 0. 目的与一句话总结

`/bob-model` 是 Bob 4 环 Clean Architecture 工作流的基石——下游 stories / identify / onion / spec 都消费它的产物。当前产物是**只读 html + md SSoT**,用户多轮修改时只能在 Claude 终端逐条口述,效率低、易遗漏。

本次升级把 html 改造为**交互式 review canvas**:每个 leaf(术语 / Entity 字段 / BR / 图 / 开放问题)都可就地评论,批量提交后 Claude 读 events 文件、应用变更、重写 html,直到用户给推进信号才一次性 dump md 给下游。

设计参照 `superpowers:brainstorming` 的 visual companion 机制(server polling + screen_dir + events),复用其 `scripts/start-server.sh` 基础设施。

---

## 1. 背景与动机

### 1.1 为什么需要

ycb 实测 session 暴露的问题:

- model 阶段产出 14 BR / 17 术语 / 6 Entity / 13 开放问题,任何调整(命名 / 公式 / 不变量收紧 / Q 决议)都要在终端逐字口述,长度爆炸
- 多轮迭代是常态(本次 session 已经 5 轮),每轮 Claude 都要重新 grep / Edit md+html 两份文件,操作重复
- 用户在终端口述"把术语 X 改成 Y" 时容易漏说级联点(BR 引用 / Entity 字段 / Q 引用),Claude 也容易级联失败
- 跨引用(BR-NNN ↔ Entity ↔ Q)只在文档里以文本形式存在,review 时翻找成本高

### 1.2 已 lock 的规则约束

本设计**不动**以下已锁定的不变量:

- 命名规约(§命名规约,普适原则 + 跨域示例)
- 多轮修改循环(§Stage 3.5,默认进入,不主动追问 Stage 4)
- 报告必含文件链接(§产物报告规约)
- 强制阶段(`/bob-stories` 校验 md 存在)—— **本设计调整了 md 生成时机,需相应调整 stories 门禁见 §5.3**

### 1.3 选定方案

参考 §approaches.html 三档,选 **B(生产级)**:每个 leaf 加 widget + 4 态色标 + 跨引用锚点 + sticky 计数 + localStorage 草稿。V1 单用户单 tab,V2 留底多人协作。

---

## 2. 架构与生命周期

### 2.1 Stage 流程对照

| Stage | 旧行为(commit 31dd597) | 新行为(本设计) |
|---|---|---|
| Stage 0 | 入口体检 + 短路判定 | 不变 |
| Stage 1 | 抽取(术语 / Entity / BR / UC / Q) | 不变 —— 但产物只是 Claude 内部快照,**不写文件** |
| **Stage 2** | 写 md(SSoT) | **改为生成 interactive html 内容**(Claude 在内存里 compose html 字符串,本设计主要工作;此时还未写盘) |
| **Stage 3** | 写 html(只读视图) | **启 visual companion server**(获得 `screen_dir` 路径)→ **把 Stage 2 的 html 写入 `screen_dir/model-v1.html`** → **给用户 URL + 路径** |
| **Stage 3.5** | 多轮修改循环(对 md + html 都改) | 多轮修改循环(**只改 html**,server 驱动评论提交) |
| **Stage 4** | 三段式收口 + 通报下一步 | **用户给推进信号 → 从最终 html dump md** → 停 server → 三段式收口 |

### 2.2 数据流(单轮 Stage 3.5)

```
1. User 在 browser 写评论(localStorage 自动 cache 草稿,4 态色标)
2. User 点 sticky 顶栏 [📋 提交本轮反馈 (N)]
3. Browser JS collectFeedback() 聚合所有 textarea → window.brainstorm.send(envelope)
4. WebSocket server (server.cjs) 收到 → 检测 event.choice 非空 → append JSONL 到 $STATE_DIR/events
5. User 在 Claude 终端发"继续" / "next" / "看反馈"
6. Claude 读 $STATE_DIR/events(取 timestamp > last_processed)
7. Claude parse → 按 kind 分派处理器 → 改内部模型快照 → 重写 html-vN → push screen_dir
8. Claude 报告改动落点 + URL + screen_dir 路径(per §产物报告规约)
9. User 浏览器刷新 → 新 html;已提交的 widget 切 ✓ submitted → ⊘ applied
10. Loop 回 step 1,直到用户给推进信号
```

### 2.3 基础设施复用

复用 `superpowers/brainstorming/scripts/start-server.sh`,不另起服务器:

- 每次 `/bob-model` 启动时 spawn 一个 server(独立 session-dir,通过 `--project-dir` 持久化)
- Stage 4 推进后,Claude 用 SIGTERM 优雅停止 server(避免 idle 30 min 浪费)
- idle timeout 期间用户不动 → server 自杀 → 下次想继续 review 由 Claude 检测后重启(罕见)

### 2.4 关键设计取舍

- **Stage 2 不写 md**:消除"半成品 md"的歧义,下游 `/bob-stories` Stage 0 门禁读 md 时永远是 final
- **server 自动启停**:用户不用手动管;`/bob-model` 即开即用
- **评论 schema 标准化**:每条评论带 `{id, kind, target, comment}`,Claude 能稳定解析(详见 §4)
- **html 是 source of truth during 迭代**:Claude 内部维持模型快照,html 是渲染;两者由 Claude 保证一致

---

## 3. HTML widget 结构与布局

### 3.1 整体布局

```
┌────────────────────────────────────────────────────────────────┐
│ Sticky Header                                                  │
│ ─ 模型标题  · 已修改 N 处 / 共 M widget                       │
│ ─ [↻ 清空草稿]  [📋 提交本轮反馈 (N)]                         │
├──────────┬─────────────────────────────────────────────────────┤
│ TOC      │ 主内容(可滚动)                                     │
│  - 术语表 (18) ●2 │ § 1. 术语表(表格 + 行内 widget)           │
│  - Entity (6)     │ § 2. Entity 草图(每 Entity 一节 + 字段表) │
│  - BR (14)  ●1    │ § 3. 业务规则(BR-NNN 卡片 + 卡底 widget)  │
│  - UseCase (1)    │ § 4. UseCase 初步清单                       │
│  - 开放问题 (14)  │ § 5. 开放问题(Q 卡 + 卡底 widget)          │
│ (sticky) │                                                     │
└──────────┴─────────────────────────────────────────────────────┘
                                              [↑ 返回顶部 floating]
```

TOC 上每节后**括号数字**是该节的 widget 总数(固定);**红色圆点数字** `●N` 是该节当前未提交 draft 的数量,N=0 时不显示(避免视觉噪音)。

### 3.2 Widget 4 态色标

| 态 | 触发 | UI | localStorage |
|---|---|---|---|
| **○ untouched** | 页面初始 | 灰边 textarea 收起,角标 `💬 0` | 无记录 |
| **● draft** | `oninput` debounce 500ms | 蓝边 textarea 展开,角标 `💬 1`,sticky 计数 +1 | `drafts[key] = text` |
| **✓ submitted** | window.brainstorm.send() 返 200 | 绿边 + textarea 收起 + ✓ 角标 | `drafts[key]` 删,加入 `submitted[]` |
| **⊘ applied** | Claude 重写 html 含 `data-applied="<comment-id>"` | 虚线灰边 + opacity 0.7 + "✓ Claude 已处理" | `submitted[]` 中对应 id 删 |

### 3.3 5 类元素的 widget 形态

#### A. 术语表行(inline expand)

每个 `<tr>` 末尾一列 `💬 <count>` 按钮,点击 toggle 展开下一行 textarea(占满全行):

```html
<tr data-comment-id="term-discountRate" data-kind="term" data-target="discountRate">
  <td>折扣率</td>
  <td><code>discountRate</code></td>
  <td>用户实付比例...</td>
  <td><button class="comment-toggle">💬 <span class="count">0</span></button></td>
</tr>
<tr class="comment-row hidden">
  <td colspan="4"><textarea class="comment-input" data-comment-id="term-discountRate"></textarea></td>
</tr>
```

#### B. Entity 字段行

与 A 同款 inline expand 风格,每个 Entity 的属性表 / 不变量列表都加 💬 按钮。Entity 名出现处自动锚点 `<a href="#entity-Order">Order</a>`。

#### C. BR 卡(卡底 details)

```html
<div class="br-card" id="br-010" data-kind="br" data-target="BR-010">
  <h4>BR-010 全场折扣</h4>
  <p>公式: <code>itemTotalFee × discountRate</code></p>
  ...
  <details class="comment-section">
    <summary>💬 评论 (<span class="count">0</span>)</summary>
    <textarea class="comment-input" data-comment-id="br-BR-010"></textarea>
  </details>
</div>
```

#### D. Mermaid 图(图下 details)

图块下加 details/textarea。`data-target` 是图 slug(如 `order-state-machine`)。

#### E. 开放问题卡(卡底 details)

与 C 同款。`data-target` 是 Q 编号(如 `Q14`)。

### 3.4 跨引用锚点(自动生成)

| 类型 | id 模式 | 锚点替换规则 |
|---|---|---|
| BR | `id="br-001"` | 文本中 `BR-001` → `<a href="#br-001">BR-001</a>` |
| Entity | `id="entity-Order"` | 文本中 Entity 名(术语表注册过的)→ `<a href="#entity-Order">Order</a>` |
| 不变量 | `id="inv-Order-3"` | 文本中 `INV-Order-3` → 锚点 |
| 开放问题 | `id="q-14"` | 文本中 `Q14` → 锚点 |

floating "↑ 返回顶部" 按钮固定右下角。

---

## 4. 提交协议与事件 schema

### 4.1 Event envelope(WebSocket message via window.brainstorm.send / events JSONL 行)

```json
{
  "type": "bob-model-feedback",
  "slug": "create-order",
  "round": 2,
  "timestamp": 1778820000000,
  "choice": "submit",
  "comments": [
    {
      "id": "c-1778820000000-001",
      "kind": "term",
      "target": "discountRate",
      "comment": "考虑改为 rateOfPayment, 更显式"
    },
    {
      "id": "c-1778820000000-002",
      "kind": "entity-field",
      "target": "Order.totalAmount",
      "comment": "应该叫 finalAmount, total 易混淆"
    },
    {
      "id": "c-1778820000000-003",
      "kind": "br",
      "target": "BR-010",
      "comment": "rate=1 也应该允许(等价无折扣), 把不变量改回 (0,1]"
    },
    {
      "id": "c-1778820000000-006",
      "kind": "general",
      "target": null,
      "comment": "BR 数太多了, 看能否合并"
    }
  ]
}
```

字段语义补充:`choice` 是 brainstorming server 的记录触发字段(server.cjs:234 只记录 `event.choice` 非空的消息),固定为 `"submit"`;Claude 读 events 时按 `type === "bob-model-feedback"` 过滤,忽略 choice。

字段语义:

- `id`:Claude 标记"已处理"用的幂等键,格式 `c-<timestamp>-<seq>`,page JS 生成
- `kind`:6 种之一(`term / entity-field / br / diagram / open-question / general`),分派到对应处理器
- `target`:稳定标识符,格式见下表;`general` 时为 null
- `comment`:用户写的 markdown 文本

### 4.2 6 种 kind 与 Claude 处理动作

| kind | target 格式 | Claude 处理 |
|---|---|---|
| `term` | 英文名 e.g. `discountRate` | 改术语行(改名 / 改定义 / 加同义词);如改名 → 自动级联所有 BR / Entity / Q 引用 |
| `entity-field` | `Entity.field` e.g. `Order.totalAmount` | 改字段(改名 / 改类型 / 改必填 / 加不变量)。改名 → 级联 |
| `br` | `BR-NNN` | 改公式 / 改约束 / 改来源 / 删 BR / 合并 BR |
| `diagram` | 图 slug e.g. `order-state-machine` | 重画 Mermaid(加状态 / 加转移 / 改类型);用户用 PR 风格描述 |
| `open-question` | `QN` | 3 种选择:决议落 BR / 改暂定假设 / 拆为多个 Q |
| `general` | `null` | 自由评论;Claude 用三段式确认意图后再动手(避免误改) |

### 4.3 幂等与增量读取

- **Claude 内部维护** `last_processed_event_timestamp`(在内部 model 快照状态里)
- 每次读 `$STATE_DIR/events` 时只取 `timestamp > last_processed_event_timestamp` 的行
- 处理完 → 更新 last_processed 为本批最大 timestamp
- **Comment id 兜底**:即使 timestamp 比较错位,处理完的 `comment.id` 记入 `processed_comment_ids` set,避免重复应用

### 4.4 自由文本兜底

意图清晰(`kind=br / target=BR-010 / comment="rate=1 也应该允许"`)→ 直接执行。

意图不清(`kind=general / "BR 数太多了"`)或含建议性措辞("看能否" / "我觉得" / "也许")→ **先三段式确认**(给出合并方案 + 影响范围),用户回 OK 才动手。

---

## 5. 草稿持久化与状态转移

### 5.1 localStorage 设计

单 key 单 JSON(每 slug 一份),避免 key 爆炸:

```javascript
// key: bob-model:create-order:drafts
{
  "term:discountRate":           "考虑改为 rateOfPayment, 更显式",
  "entity-field:Order.totalAmount": "应该叫 finalAmount, total 易混淆",
  "br:BR-010":                   "rate=1 也应该允许...",
  "general":                     "BR 数太多了, 看能否合并"
}

// 另一个 key: bob-model:create-order:submitted
// 已提交但 Claude 还没回应的 widget id list(防止刷新后丢"submitted"态)
["term:discountRate", "br:BR-010"]
```

### 5.2 关键时序

1. 用户输入 → 500ms debounce → `localStorage.drafts` 写入 → UI 切 draft 态
2. 用户点 Submit → 聚合所有 draft → window.brainstorm.send() → 等响应
3. window.brainstorm.send() 返 200 → 清掉对应 drafts 项 → 加入 submitted set → 切 submitted 态
4. 用户在终端发"继续" → Claude 读 events → 应用 → 重写 html(含 `data-applied`)
5. 用户刷新浏览器 → 新 html 加载 → JS 启动 → 扫 `data-applied` → 把 submitted set 中对应 id 删 → 切 applied 态
6. 剩余 untouched / draft widget → JS 读 localStorage 恢复界面状态

### 5.3 Edge cases

| 场景 | 行为 |
|---|---|
| 浏览器刷新 / 关闭重开 | localStorage 持久;draft / submitted 都还在 |
| 误点 Submit | 无 undo(KISS)。可在 Claude 处理前发"撤回 c-xxx"指令(走 general kind) |
| window.brainstorm.send() 失败(server 死了) | drafts 不动,toast 提示。用户重启 server 后再 submit |
| Claude 部分应用失败 | html 中对应 `data-applied` 不发出 → 该 widget 仍 submitted 态 → 用户重新提交或终端补刀 |
| 多 tab 打开同 URL | ⚠ V1 不支持(双写 localStorage 冲突);文档明示单 tab。BroadcastChannel 留 V2 |
| server idle 超时(30 min) | 已提交事件已写盘,无丢失;新提交会失败 → 用户告诉 Claude 重启 server |

---

## 6. skill 模板集成与下游影响

### 6.1 bob-model.md 改动

| Section | 动作 | 改动要点 |
|---|---|---|
| Stage 0 / 1 | 不变 | 入口体检 / 抽取逻辑不变(Stage 1 产出仅内部快照,不写盘) |
| **Stage 2** | 重写 | 原"写 md (SSoT)" → 生成 interactive html:含 widget / 状态机 JS / localStorage / collectFeedback / 跨引用锚点 / sticky 顶栏 |
| **Stage 3** | 重写 | 原"写 html (视图)" → 启 visual companion server(复用 `scripts/start-server.sh --project-dir`)+ push html 到 `screen_dir` + 给用户 URL + 列路径 |
| **Stage 3.5** | 重写 | 改为 event-driven 循环:用户终端发"继续" → 读 events → parse 6 种 kind → 应用 → 重写 html → push;只动 html 不动 md |
| **Stage 4** | 重写 | 推进信号 → 从最终 html 状态 dump md(`docs/bob/03-model-*.md`)→ 停 server(优雅 SIGTERM)→ 三段式收口(产物链接含 md / html 两份) |
| §命名规约 | 不变 | 原则普适规则保留 |
| §产物报告规约 | 微调 | 报告必含 URL + screen_dir 路径(原有 md/html 绝对路径);md 仅 Stage 4 起出现 |
| **新增 §** 评论协议与 schema | 新增 | §4 落成 skill 节 |
| **新增 §** html widget 规范 | 新增 | §3 落成 skill 节 |
| 不变量 | +3 条 | 「html 是 review canvas(非只读)」/「md 仅 Stage 4 生成」/「server 自启自停」 |

### 6.2 下游 `/bob-stories` 门禁影响

当前 stories Stage 0 Step 1 校验 `docs/bob/03-model-*.md` 存在 + 创建时间 < 7 天(commit `16b271e`)。本设计下,md 仅在 Stage 4 生成,所以:

- 用户在 model 阶段没推进就跑 stories → md 不存在 → stories 拒绝运行(行为不变,符合预期)
- 用户推进 model 后 md 生成 → stories 校验通过(行为不变)
- **不需要改 stories skill**

### 6.3 错误处理矩阵

| 错 | 检测 | 恢复 |
|---|---|---|
| server 启动失败(端口占用 / 脚本缺失) | start-server.sh 退出非 0 | Claude 终端报错 + 降级到旧版只读 html(不带 widget) |
| 浏览器 window.brainstorm.send() 失败 | page JS 收到非 200 | drafts 不清,toast 提示;用户重启 server |
| events JSON 解析失败 | Claude 终端 try/parse | 该行 skip + 报告 + 询问用户是否手动提供 comment |
| target 找不到(typo / 用户编了不存在的 id) | apply 时 lookup miss | 三段式:列候选最相似 3 个,问用户哪个 |
| server 中途死(idle timeout) | Claude 读 events 时检查 server-info / server-stopped | 复用之前的 screen_dir 重启(或新建);保留 events 历史 |
| html 写盘失败(磁盘满 / 权限) | Write tool 返错 | 直接抛给用户,中止本轮 |
| Stage 4 md 生成失败 | Claude 检查产物 | 回退到 Stage 3.5,允许用户继续 review;不假装成功 |

---

## 7. 测试策略

### 7.1 手动验证

- ✓ 跑 `/bob-model` on 实际项目(ycb 重做)→ 服务器启 → 浏览器开 → 看见 widget
- ✓ 每种 kind(term / entity-field / br / diagram / open-question / general)各加 1 条评论 → 提交 → 终端"继续" → Claude 应用 → 浏览器刷新看 applied 态
- ✓ 4 态色标转换:未碰 → 输入(灰 → 蓝)→ 提交(蓝 → 绿)→ Claude 处理(绿 → 灰虚)
- ✓ 跨引用:点 BR 卡里的 "RateDiscount" → 跳到 Entity 2.4 章节
- ✓ localStorage 持久:写半条 → 刷新 → 草稿还在
- ✓ 多轮迭代:连续 3 轮提交,events 历史和 last_processed_timestamp 正确
- ✓ Stage 4 推进:用户回"OK 推进" → md 文件生成,frontmatter sha256 / generated_at 正确

### 7.2 自动化(简版)

- events JSON parse:Bash + jq 脚本验证 envelope schema
- html 结构 grep:每个 widget 必含 `data-comment-id` / `data-kind` / `data-target` 属性
- 不写完整 e2e(LLM 输出难稳定,人工 vibes 验证够用 V1)

---

## 8. V1 范围 / V2 留底

| V1(本设计) | V2 留底(本次不做) |
|---|---|
| 单用户单 tab | 多人 review(导入导出 JSON) |
| 批量提交 | 评论 thread / resolve |
| 4 态色标 | 评论 owner 字段 |
| 跨引用锚点 | 多 tab BroadcastChannel |
| localStorage 草稿 | diff view(this round vs last round) |
| server idle 30min 自杀 | 评论历史回放 |

---

## 9. 实施计划(供 `superpowers:writing-plans` 接力)

1. **更新 `src/templates/skills/bob-model.md`**(Stage 2 / 3 / 3.5 / 4 全部重写 + 2 个新章节 + 3 条不变量)
2. **设计 html 模板**(参考 ycb 现有 html 重写为 widget 版,作为模板范例)
3. **重做 ycb session 的 html**(作为 V1 demo + 验证)
4. **端到端手动测试**(7.1 清单)
5. **commit + push 全部改动**
6. **运行 `install` skill**(可选):把更新过的 skill 模板同步到目标项目

`writing-plans` 应把以上拆成 6-10 个可独立验证的子任务,带回滚点。

---

## 10. 风险与对策

| 风险 | 概率 | 影响 | 对策 |
|---|---|---|---|
| visual companion server 不支持自定义 POST body | 无(已验证) | 高(机制不成立) | 已验证通过 WebSocket + choice 触发字段实现 |
| html JS 在不同浏览器表现不一致 | 低 | 中 | 只测 Chrome / Safari(macOS 主力);其它浏览器 V2 处理 |
| Claude 解析自由文本的"应该是哪种 kind"判断错 | 中 | 中 | `data-kind` / `data-target` 由 page JS 写好,Claude 不靠猜 |
| 多人误用同一 server URL | 低 | 中(comments 串了) | V1 文档明示单用户;sticky 顶栏显示 session-id 让用户察觉 |
| Stage 4 md dump 漏字段 | 中 | 高 | 写一个 md schema 检查(必含 frontmatter / 5 节),Stage 4 自检 |

---

## 11. 决策日志(brainstorming 记录)

- **Q1 反馈传输**:服务器轮询(模仿 visual companion),用户拒绝了剪贴板和文件导出
- **Q2 md/html 同步**:迭代只动 html,md 仅 Stage 4 生成(用户原意)
- **Q3 提交 UX**:批量提交(用户点 sticky 顶栏一个按钮)
- **方案档**:选 B(生产级,含 localStorage 草稿 + 4 态色标 + 跨引用锚点)

---

*Generated by `superpowers:brainstorming` · 2026-05-15 · 待 `superpowers:writing-plans` 接力*
