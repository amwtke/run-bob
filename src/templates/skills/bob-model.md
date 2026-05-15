---
name: bob-model
description: |
  触发条件:用户输入 /bob-model <doc-path>(主入口:把一份需求文档建模成结构化领域快照),
  或 /bob-model --story <story-path>(退路:已有 stories 时反向补建模),
  或 /bob-model --refresh(强制重写已有模型,即使源文档未变化)。

  在 /bob-survey 之后、/bob-stories 之前运行。**/bob-model 是 bob-* 链路的强制阶段**:
  与 /bob-survey(可跳过)不同,不论需求难度 Easy / Medium / Hard,只要决定接需求,
  就必须跑 /bob-model;"极小需求"可走 Stage 0 短路分支,但仍产出占位 md。
  不存在"AC 看起来清晰所以跳过 model"或"端口数少于 N 就不跑 model"之类的阈值短路。

  读 PM 风格的散文需求文档(.md / .pdf / .docx / .txt),抽取出:1) 术语表,
  2) Entity 草图(属性 + 状态机种子 + 不变量),3) 业务规则清单(BR-NNN,跨 story 共享),
  4) UseCase 初步清单,5) 开放问题。
  产出交互式 docs/bob/03-model-<slug>-<date>.html(Stage 2-3.5 review canvas,带 widget + WebSocket 反馈)
  + Stage 4 推进时从最终 html 状态 dump 出 docs/bob/03-model-<slug>-<date>.md(SSoT)。

  适用于 Bob 4 环 Clean Architecture 工作流的领域建模 phase。结构上对称
  phase 2 (/bob-nfr) 和 phase 3 (/bob-compliance),都用 5 stage + 三段式。

  当用户说"建个模"、"做下领域建模"、"统一下术语"、"抽取业务规则"时也应触发此技能。
---

# Bob Model Skill

## 触发

```
/bob-model <doc-path>          # 主入口:对源需求文档建模
/bob-model --story <path>      # 退路:已有 stories 时反向建模
/bob-model --refresh           # 强制重写,即使源文档未变化
```

或自然语言触发:"建个模"、"做下领域建模"、"统一下术语"、"抽取业务规则"。

## 前置条件

- 项目位于 git 仓库内
- 建议:`/bob-survey` 已完成 + 难度评估 Medium/Hard
- 源需求文档可读(PDF/docx 需要 Read 工具的 `pages` 参数;md/txt 直读)

## 提问规约(强制三段式)

任何需要用户选择的问题,**必须**按下面三段式输出。**禁止**抛开放问题。

格式:

> **[问题序号] [问题]**
>
> **推测**:<你的判断>
> **理由**:<一句话>
> **推荐选择**:`<具体一个选项>`
>
> 是否同意?(回"是"走推荐;回"否,理由是 ..."重判;回"否,我选 X"切到 X)

## 命名规约(强制表意,禁止光秃名词)

**理由**:术语表的英文列 = 下游 `/bob-spec` → TDD 阶段代码的**变量名 / 类型名直接候选**。光秃名词(`total` / `rate` / `time` / `discount` / `info`)会让人读到字段时无法判断它是钱、是数量、是时间、还是 boolean,在 spec 阶段需要重新命名,造成术语漂移和返工。**消除歧义优先于简洁**。

**通用心法**:**动词 + 名词 / 形容词 + 名词 / 修饰语 + 被动式** 组合,让命名自描述;不要让一个英文裸单词承担"判断它是什么类型"的认知负担。

### 核心原则(强制 · 普适)

每个名字必须让读者**脱离上下文**就能判断:**(a) 它是什么 type**(钱 / 数量 / 时间 / bool / 状态 / 结果 / 事件 / ...)和 **(b) 它的语义角色**(active / applied / computed / pending / ...)。光秃名词在两项上都失败。

通过 **POS 组合**(动词 + 名词 / 形容词 + 名词 / 修饰语 + 被动式)同时编码 (a) 和 (b)。**具体的后缀 / 前缀是领域约定**,**原则是普适的**——下面表里的具体词(`Fee` / `Rate` / `At` / `Quantity`)是电商 / 订单域示例,**换到其它域必须按本域自然约定灵活调整**(见后文「跨领域适用」)。

### 参考模式(电商/订单域示例,非穷举,按域灵活调整)

| 类别 | 反模式(禁) | 推荐模式 | 说明 |
|---|---|---|---|
| **金额** | `total` / `price` / `discount` / `cost`(裸) | `itemTotalFee` / `discountAmount` / `unitPrice` / `packingFee` | 用 `Fee` / `Amount` / `Price` / `Cost` 后缀,让"是钱"显式 |
| **比率** | `rate` / `ratio` | `discountRate` / `taxRate` / `commissionRate` | 用语义前缀 + `Rate`,说明"哪种比率" |
| **数量** | `num` / `cnt` / `count`(裸) | `itemQuantity` / `bonusQuantity` / `attemptCount` | 用限定词 + `Quantity` / `Count`,说明"几个什么" |
| **时间** | `time` / `date` | `createdAt` / `expiredAt` / `paidAt` | 用过去分词被动式 + `At`,说明"什么事件的时间" |
| **类型(归属型)** | `MerchantDiscount`(只说归属) | `ActiveDiscountPlan` —— 形容词 + 抽象名词 | 类型名应表达"状态 / 角色 / 结果",归属由 repository 携带(`ActiveDiscountPlanRepository.findByMerchantId`) |
| **类型(动作型)** | `Discount` / `Apply`(光秃) | `DiscountResult` / `ApplyDiscountCommand` / `AppliedDiscountSnapshot` | 用后缀(`Result` / `Command` / `Snapshot` / `Event`)消除"动词 vs 名词"歧义 |
| **Boolean** | `valid` / `discount` / `active` | `isValid` / `hasActiveDiscount` / `wasApplied` / `canCancel` | 用 `has` / `is` / `was` / `can` 前缀 + 形容词或被动式,让"是 bool"显式 |

### 跨领域适用(原则普适,具体词按域改)

上表是电商 / 订单域示例。换到其它域,**原则不变**——名字必须同时编码 type + role,禁止光秃名词——**但具体词必须换成本域自然量纲与角色**:

| 域 | 金额族 | 数量 / 度量族 | 时间族 | 类型族 |
|---|---|---|---|---|
| **物流 / 配送** | `shippingFeeAmount` / `codAmount` | `weightGram` / `weightKg` / `distanceKm` / `parcelCount` | `shippedAt` / `deliveredAt` / `signedAt` | `DeliveryStatusSnapshot` / `ActiveRouteAssignment` |
| **IoT / 设备** | (少金额) | `temperatureCelsius` / `batteryPercentage` / `signalRssi` | `lastPingedAt` / `lastChargedAt` | `DeviceHealthSnapshot` / `ActiveFirmwareImage` |
| **金融 / 账务** | `ledgerAmount` / `txnFeeAmount` | `shareCount` / `lotSize` | `settledAt` / `postedAt` / `accruedAt` | `PostedLedgerEntry` / `ActiveCounterparty` |
| **医疗 / 处方** | `prescriptionCharge` | `dosageMg` / `dosageMl` / `vialCount` | `prescribedAt` / `administeredAt` | `FilledPrescription` / `ActivePatientChart` |
| **教育 / LMS** | `tuitionFeeAmount` / `scholarshipAmount` | `enrolledCount` / `attemptCount` | `enrolledAt` / `submittedAt` / `gradedAt` | `SubmittedAssignment` / `ActiveEnrollmentPlan` |

**心法**:**不要照搬表里的具体词**——要复制表的**结构**:找到本域自然量纲(Kg / Km / Mg / Ml / Celsius / RSSI / ...),用 `xxxAt` 表达事件时间,用形容词 + 抽象名词表达类型角色。**普适规则只有一条:每个名字 = type + role 的显式编码。**

### Stage 1 抽取过程内自检(强制)

在写 §1 术语表之前,Claude 对每一个候选名做三项自检:

1. **金额 / 比率 / 数量 / 时间字段**:有没有光秃名词?有就**直接重命名**,不抛 Q——这类歧义本规约已解决。
2. **类型名**:只表达"归属"或裸"概念"?如是,问自己"它是什么**状态 / 角色 / 结果**?",加形容词或语义后缀。
3. **脱离上下文测试**:任何字段名从术语表里抽出来单看,能否猜到 ≥ 80% 含义?不能就重命名。

自检发现的歧义**直接在 §1 给出表意名**——不要写出光秃名再用 Q 兜底。Q 留给"两种表意名都合理需用户拍板"的真歧义(三段式),不留给本规约可消除的。

### 违反 = 产出无效

本规约是 model 阶段的**硬约束**(配合"强制阶段不可跳过"不变量)。下游(stories / spec / TDD / code review)若收到含光秃名词的 model md,等价于 model 阶段没跑完——必须 `/bob-model --refresh` 重抽。Reviewer 接到 model md 时,**优先扫名词族**(grep 金额族 / 比率族 / 时间族 / Boolean 前缀)做形式校验。

## 产物报告规约(强制列出文件链接)

**每次** `/bob-model` 产出或更新文件后的报告中,**必须显式列出文件的绝对路径**(md + html),让用户能直接复制 / 浏览器打开 review。

### 必含项(任何"产物落盘"或"改动落点"的报告)

- url(浏览器打开):`http://localhost:<port>`(Stage 3.5 期间提供;Stage 4 后 server 已停,可忽略)
- screen_dir(html 本体):`/Users/.../03-model-<slug>-<date>-vN.html`(Stage 3.5 期间)
- md(SSoT):`/Users/.../docs/bob/03-model-<slug>-<date>.md`(Stage 4 起才存在)
- html(团队视图):`/Users/.../docs/bob/03-model-<slug>-<date>.html`(Stage 4 起从 screen_dir 复制)

### 格式示例

**改动报告(Stage 3.5 每一轮)**:

> 已应用本轮 N 条改动:
> - <kind:target>: <一行总结>
>
> **产物**(可直接打开 review):
> - url(浏览器打开):`http://localhost:<port>`
> - screen_dir(html 本体):`/Users/.../03-model-create-order-20260515-v2.html`

**Stage 4 收口三段式**:

> **Q: 建模完成。N 条术语 / ...**
>
> **产物**(review 直链):
> - md(SSoT):`/Users/.../docs/bob/03-model-create-order-20260515.md`
> - html(团队视图,浏览器打开):`/Users/.../docs/bob/03-model-create-order-20260515.html`
>
> **推测**:...
> **推荐选择**:...

### 明令禁止

- ❌ 只说"html 已落盘 / 已更新"而不给路径 —— 用户没法直接打开
- ❌ 只给相对路径(`docs/bob/...`)—— 用户在不同 cwd 时无法点击,IDE / 终端解析失败
- ❌ 把路径埋在散文里 —— 必须用 list / table 醒目展示
- ❌ 第二轮起省略路径 —— **每轮都要列**(即使路径没变),让用户随时定位
- ❌ 只列其中一个文件 —— md + html 两个都要(就算用户只用 html,md 仍是 SSoT 应保持可见)

### 为什么强制

用户在多轮修改循环(§Stage 3.5)中频繁需要打开 html 验证 Mermaid 图 / 长表格 / 跨域命名;每次都要翻历史消息找路径是摩擦。把路径作为"每轮报告标配"消除噪音。

## html widget 规范(Stage 2 compose 时必照)

每次 `/bob-model` Stage 2 生成 html 时,必须遵循以下结构 + 内嵌 CSS + 内嵌 JS。

### Page-level 骨架

    <!DOCTYPE html>
    <html lang="zh-CN">
    <head>
      <meta charset="UTF-8">
      <title>领域模型 · {{title}} · {{date}}</title>
      <script>
        window.BOB_MODEL_SLUG = '{{slug}}';
        window.BOB_MODEL_ROUND = {{round}};
      </script>
      <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
      <script>mermaid.initialize({ startOnLoad: true });</script>
      <style>/* 见 §CSS */</style>
    </head>
    <body>
      <header class="bob-sticky">
        <div class="title">{{title}}</div>
        <div class="meta">已修改 <span id="draft-counter">0</span> 处 / 共 {{totalWidgets}} widget</div>
        <div class="actions">
          <button id="clear-button">↻ 清空草稿</button>
          <button id="submit-button" disabled>📋 提交本轮反馈</button>
        </div>
      </header>
      <div class="bob-layout">
        <nav class="bob-toc">
          <a href="#sec-terms">术语表 (18) <span class="section-badge" data-section-counter="terms"></span></a>
          <a href="#sec-entities">Entity (6) <span class="section-badge" data-section-counter="entities"></span></a>
          <a href="#sec-br">BR (14) <span class="section-badge" data-section-counter="br"></span></a>
          <a href="#sec-uc">UseCase (1) <span class="section-badge" data-section-counter="uc"></span></a>
          <a href="#sec-q">开放问题 (14) <span class="section-badge" data-section-counter="q"></span></a>
        </nav>
        <main class="bob-main">
          <!-- §1 术语表 / §2 Entity / §3 BR / §4 UC / §5 Q -->
        </main>
      </div>
      <button class="bob-back-top" onclick="window.scrollTo({top:0, behavior:'smooth'})" aria-label="返回顶部">↑</button>
      <script>/* 见 §JS */</script>
    </body>
    </html>

### Widget DOM 模板

每个 widget 必须有 3 个 data-* 属性:`data-comment-id`(唯一,= `<kind>:<target>` 或 'general')、`data-kind`、`data-target`,以及一个 `.comment-input` textarea。

5 种 widget 形态:

1. **术语表行**(inline expand):每个 `<tr>` 末尾 💬 按钮 toggle 展开下方 textarea。
2. **Entity 字段行**:同上 inline expand;每个属性 / 不变量行都加 💬。
3. **BR 卡**(卡底 details):每张 BR 卡底加 `<details>` 包 textarea。
4. **Mermaid 图**(图下 details):图块下 details/textarea,`data-target` 用图 slug。
5. **开放问题卡**(卡底 details):同 BR 卡;`data-target` 用 Q 编号。

### 布局与导航(粘性 TOC)

- 顶部 `header.bob-sticky`(sticky 定位)固定可见,含计数 + 提交按钮
- 左侧 `nav.bob-toc` **粘性 TOC**(`position: sticky; top: 56px`),5 节锚点 + 每节 draft 红角标
- 主内容区 `main.bob-main` 滚动区
- 右下浮动 `.bob-back-top` 返顶按钮

### Mermaid 图清单(每节用什么类型)

| 出现位置 | Mermaid 类型 | 用途 |
|---|---|---|
| Entity 草图 §2.x | `classDiagram` | 每个 Entity 的属性 class 块 + `<<value object>>` / `<<entity>>` 注解;sum type 子类用 `<|--` 继承箭头 |
| Order 状态机种子 | `stateDiagram-v2` | 已出现状态用实线 `-->`,未确认状态用虚线 `..>` |
| UseCase 折扣应用 §4.x | `flowchart TD` 或 `flowchart LR` | 流程图,展示 UseCase 内部 step 串联 / 分支逻辑 |

每张图都包在 `<pre><code class="language-mermaid">...</code></pre>` 内便于离线降级,同时下方 `<div class="mermaid">...</div>` 让 CDN Mermaid 渲染。

统一格式(以 term 为例):

    <div class="bob-widget" data-comment-id="term:discountRate" data-kind="term" data-target="discountRate" data-section="terms">
      <button class="comment-toggle">💬 <span class="count">0</span></button>
      <textarea class="comment-input" placeholder="对此 term 的修改意见..."></textarea>
    </div>

### CSS(嵌 head 内 <style>)

    :root {
      --c-untouched: #d0d7de;
      --c-draft: #0969da;
      --c-submitted: #1a7f37;
      --c-applied: #57606a;
      --c-bg: #fafbfc;
      --c-fg: #1f2328;
    }
    * { box-sizing: border-box; }
    body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", sans-serif; background: var(--c-bg); color: var(--c-fg); }
    .bob-sticky { position: sticky; top: 0; z-index: 100; background: var(--c-draft); color: white; padding: 12px 16px; display: flex; align-items: center; gap: 16px; }
    .bob-sticky .title { font-weight: 700; font-size: 16px; }
    .bob-sticky .meta { opacity: 0.85; font-size: 13px; }
    .bob-sticky .actions { margin-left: auto; display: flex; gap: 8px; }
    .bob-sticky button { padding: 4px 12px; border-radius: 4px; border: 1px solid rgba(255,255,255,0.4); background: rgba(255,255,255,0.2); color: white; cursor: pointer; font-size: 13px; }
    .bob-sticky #submit-button:not(:disabled) { background: white; color: var(--c-draft); font-weight: 600; }
    .bob-sticky #submit-button:disabled { opacity: 0.5; cursor: not-allowed; }
    .bob-layout { display: flex; max-width: 1280px; margin: 0 auto; }
    .bob-toc { width: 220px; padding: 16px; font-size: 13px; position: sticky; top: 56px; align-self: flex-start; max-height: calc(100vh - 56px); overflow-y: auto; border-right: 1px solid var(--c-untouched); }
    .bob-toc a { display: block; padding: 4px 8px; color: var(--c-fg); text-decoration: none; border-radius: 4px; }
    .bob-toc a:hover { background: #ddf4ff; color: var(--c-draft); }
    .bob-toc .section-badge { display: inline-block; background: var(--c-draft); color: white; font-size: 11px; padding: 0 6px; border-radius: 8px; margin-left: 4px; }
    .bob-toc .section-badge:empty { display: none; }
    .bob-main { flex: 1; padding: 16px 40px 80px; }
    .bob-widget { position: relative; }
    .bob-widget .comment-toggle { background: transparent; border: 1px solid var(--c-untouched); padding: 2px 8px; border-radius: 12px; font-size: 11px; cursor: pointer; color: var(--c-fg); }
    .bob-widget.expanded .comment-toggle { background: var(--c-draft); color: white; border-color: var(--c-draft); }
    .bob-widget .comment-input { width: 100%; min-height: 60px; margin-top: 8px; padding: 8px; border: 2px solid var(--c-untouched); border-radius: 4px; font-size: 13px; resize: vertical; display: none; }
    .bob-widget.expanded .comment-input { display: block; }
    .bob-widget.state-draft .comment-input { border-color: var(--c-draft); background: #ddf4ff; }
    .bob-widget.state-submitted .comment-input { border-color: var(--c-submitted); background: #dafbe1; display: none; }
    .bob-widget.state-submitted .comment-toggle { background: var(--c-submitted); color: white; border-color: var(--c-submitted); }
    .bob-widget.state-submitted .comment-toggle::after { content: ' ✓'; }
    .bob-widget.state-applied { opacity: 0.6; }
    .bob-widget.state-applied .comment-toggle { background: transparent; color: var(--c-applied); border: 2px dashed var(--c-applied); }
    .bob-widget.state-applied .comment-input { border: 2px dashed var(--c-applied); background: #f6f8fa; color: var(--c-applied); display: none; }
    .bob-toast { position: fixed; bottom: 24px; left: 50%; transform: translateX(-50%) translateY(80px); background: #1f2328; color: white; padding: 10px 18px; border-radius: 6px; font-size: 13px; box-shadow: 0 4px 12px rgba(0,0,0,0.15); transition: transform 0.3s ease; z-index: 200; }
    .bob-toast.show { transform: translateX(-50%) translateY(0); }
    .bob-back-top { position: fixed; right: 24px; bottom: 24px; width: 40px; height: 40px; border-radius: 20px; background: var(--c-draft); color: white; border: none; cursor: pointer; box-shadow: 0 4px 12px rgba(0,0,0,0.15); }

### JS(嵌 body 末 <script>)

    // bob-model interactive review · page helper (inline, no module system)
    // Globals expected: window.BOB_MODEL_SLUG (str), window.BOB_MODEL_ROUND (int)
    (function() {
      if (!window.BOB_MODEL_SLUG) { console.error('BOB_MODEL_SLUG missing'); return; }
      const SLUG = window.BOB_MODEL_SLUG;
      const DRAFTS_KEY = `bob-model:${SLUG}:drafts`;
      const SUBMITTED_KEY = `bob-model:${SLUG}:submitted`;
    
      // -- localStorage helpers --
      const loadDrafts = () => {
        try { return JSON.parse(localStorage.getItem(DRAFTS_KEY) || '{}'); }
        catch (e) { console.warn('drafts localStorage corrupted, resetting:', e); return {}; }
      };
      const saveDrafts = (d) => localStorage.setItem(DRAFTS_KEY, JSON.stringify(d));
      const loadSubmitted = () => {
        try { return JSON.parse(localStorage.getItem(SUBMITTED_KEY) || '[]'); }
        catch (e) { console.warn('submitted localStorage corrupted, resetting:', e); return []; }
      };
      const saveSubmitted = (s) => localStorage.setItem(SUBMITTED_KEY, JSON.stringify(s));
    
      // -- widget key (= comment-id) helpers --
      function widgetKey(widget) {
        return widget.dataset.commentId;
      }
      function setState(widget, state) {
        widget.classList.remove('state-untouched', 'state-draft', 'state-submitted', 'state-applied');
        widget.classList.add('state-' + state);
      }
    
      // -- hydrate UI from localStorage on page load --
      function hydrate() {
        const drafts = loadDrafts();
        const submitted = new Set(loadSubmitted());
        const cleanedSubmitted = new Set(submitted);
    
        document.querySelectorAll('[data-comment-id]').forEach(widget => {
          const key = widgetKey(widget);
          const textarea = widget.querySelector('.comment-input');
    
          if (widget.hasAttribute('data-applied')) {
            setState(widget, 'applied');
            cleanedSubmitted.delete(key);
            return;
          }
    
          if (submitted.has(key)) {
            setState(widget, 'submitted');
            return;
          }
    
          if (drafts[key]) {
            if (textarea) textarea.value = drafts[key];
            setState(widget, 'draft');
            return;
          }
    
          setState(widget, 'untouched');
        });
    
        saveSubmitted([...cleanedSubmitted]);
        updateCounter();
      }
    
      // -- debounced auto-save on input --
      const saveTimers = new Map();
      function setupAutoSave() {
        document.querySelectorAll('.comment-input').forEach(textarea => {
          textarea.addEventListener('input', () => {
            const widget = textarea.closest('[data-comment-id]');
            const key = widgetKey(widget);
            if (saveTimers.has(key)) clearTimeout(saveTimers.get(key));
            saveTimers.set(key, setTimeout(() => {
              const drafts = loadDrafts();
              const val = textarea.value.trim();
              if (val) {
                drafts[key] = textarea.value;
                setState(widget, 'draft');
              } else {
                delete drafts[key];
                setState(widget, 'untouched');
              }
              saveDrafts(drafts);
              updateCounter();
            }, 500));
          });
        });
      }
    
      // -- sticky counter + submit button enable + per-section counters --
      function updateCounter() {
        const draftCount = document.querySelectorAll('.state-draft').length;
        const counterEl = document.getElementById('draft-counter');
        const submitBtn = document.getElementById('submit-button');
        if (counterEl) counterEl.textContent = draftCount;
        if (submitBtn) submitBtn.disabled = draftCount === 0;
    
        // Per-widget count badge: 1 if widget has content (draft or submitted), 0 if untouched/applied
        document.querySelectorAll('[data-comment-id]').forEach(widget => {
          const countEl = widget.querySelector('.comment-toggle .count');
          if (countEl) {
            const hasContent = widget.classList.contains('state-draft') || widget.classList.contains('state-submitted');
            countEl.textContent = hasContent ? '1' : '0';
          }
        });
    
        const sectionCounts = {};
        document.querySelectorAll('.state-draft').forEach(w => {
          const section = w.dataset.section;
          if (section) sectionCounts[section] = (sectionCounts[section] || 0) + 1;
        });
        document.querySelectorAll('[data-section-counter]').forEach(el => {
          const sec = el.dataset.sectionCounter;
          const c = sectionCounts[sec] || 0;
          el.textContent = c > 0 ? '●' + c : '';
        });
      }
    
      // -- collect feedback envelope --
      function collectFeedback() {
        const drafts = loadDrafts();
        const ts = Date.now();
        const comments = Object.entries(drafts).map(([key, comment], i) => {
          const widget = document.querySelector(`[data-comment-id="${CSS.escape(key)}"]`);
          if (!widget) {
            console.warn('orphaned draft key has no widget; skipping:', key);
            return null;
          }
          const kind = widget.dataset.kind;
          const target = widget.dataset.target || null;
          return {
            id: `c-${ts}-${String(i + 1).padStart(3, '0')}`,
            kind,
            target,
            comment
          };
        }).filter(Boolean);
        return {
          type: 'bob-model-feedback',
          choice: 'submit',
          slug: SLUG,
          round: parseInt(window.BOB_MODEL_ROUND || '1', 10),
          timestamp: ts,
          comments
        };
      }
    
      // -- submit via window.bobReview.send --
      function submitFeedback() {
        const envelope = collectFeedback();
        if (envelope.comments.length === 0) return;
    
        if (!window.bobReview || !window.bobReview.send) {
          showToast('WebSocket helper 未加载 (页面可能未通过 server 访问)');
          return;
        }
        try {
          window.bobReview.send(envelope);
          const submitted = loadSubmitted();
          envelope.comments.forEach(c => {
            const key = c.kind === 'general' ? 'general' : `${c.kind}:${c.target}`;
            if (!submitted.includes(key)) submitted.push(key);
          });
          saveSubmitted(submitted);
          saveDrafts({});
    
          document.querySelectorAll('.state-draft').forEach(w => setState(w, 'submitted'));
          updateCounter();
          showToast(`已提交 ${envelope.comments.length} 条反馈,等 Claude 处理`);
        } catch (e) {
          showToast(`提交失败: ${e.message}`);
        }
      }
    
      function clearDrafts() {
        if (!confirm('清空所有未提交草稿?(已提交不受影响)')) return;
        saveDrafts({});
        document.querySelectorAll('.state-draft').forEach(w => {
          setState(w, 'untouched');
          const ta = w.querySelector('.comment-input');
          if (ta) ta.value = '';
        });
        updateCounter();
      }
    
      function showToast(msg) {
        const t = document.createElement('div');
        t.className = 'bob-toast';
        t.textContent = msg;
        document.body.appendChild(t);
        setTimeout(() => t.classList.add('show'), 10);
        setTimeout(() => { t.classList.remove('show'); setTimeout(() => t.remove(), 300); }, 3000);
      }
    
      function setupToggles() {
        document.querySelectorAll('.comment-toggle').forEach(btn => {
          btn.addEventListener('click', () => {
            const widget = btn.closest('[data-comment-id]');
            widget.classList.toggle('expanded');
          });
        });
      }
    
      document.addEventListener('DOMContentLoaded', () => {
        hydrate();
        setupAutoSave();
        setupToggles();
        const sb = document.getElementById('submit-button');
        const cb = document.getElementById('clear-button');
        if (sb) sb.addEventListener('click', submitFeedback);
        if (cb) cb.addEventListener('click', clearDrafts);
      });
    })();

### 不变量

- 每个 widget 必有 `data-comment-id` / `data-kind` / `data-target` / `data-section`
- 所有 cross-ref 必须是 `<a href="#br-001">BR-001</a>` 格式(自动生成,Claude compose 时正则替换)
- Stage 4 dump md 时,**自动 strip** 所有 widget DOM(只保留语义内容)→ md 干净

## 评论协议与 schema(Stage 3.5 必照)

### Envelope schema(WebSocket message via window.bobReview.send / events JSONL 行)

    {
      "type": "bob-model-feedback",
      "choice": "submit",
      "slug": "create-order",
      "round": 2,
      "timestamp": 1778820000000,
      "comments": [
        {"id": "c-1778820000000-001", "kind": "term", "target": "discountRate", "comment": "..."},
        {"id": "c-1778820000000-002", "kind": "entity-field", "target": "Order.totalAmount", "comment": "..."},
        {"id": "c-1778820000000-003", "kind": "br", "target": "BR-010", "comment": "..."},
        {"id": "c-1778820000000-004", "kind": "diagram", "target": "order-state-machine", "comment": "..."},
        {"id": "c-1778820000000-005", "kind": "open-question", "target": "Q14", "comment": "..."},
        {"id": "c-1778820000000-006", "kind": "general", "target": null, "comment": "..."}
      ]
    }

### 6 种 kind × Claude 处理

| kind | target 格式 | Claude 处理 |
|---|---|---|
| `term` | 英文名 e.g. `discountRate` | 改术语行(改名/改定义/加同义词);改名 → 自动级联所有 BR / Entity / Q 引用 |
| `entity-field` | `Entity.field` e.g. `Order.totalAmount` | 改字段(改名/改类型/改必填/加不变量);改名 → 级联 |
| `br` | `BR-NNN` | 改公式/约束/来源/删/合并 |
| `diagram` | 图 slug e.g. `order-state-machine` | 重画 Mermaid |
| `open-question` | `QN` | 决议落 BR / 改暂定 / 拆分 |
| `general` | `null` | **先三段式**确认意图再动手 |

### 幂等与增量

- `last_processed_event_timestamp`(Claude 内部 model snapshot 状态)
- `processed_comment_ids` set(兜底防 timestamp 错位)
- 每轮处理完更新两者

### 自由文本兜底

意图清晰(`kind=br / target=BR-010 / comment="rate=1 也应该允许"`)→ **直接执行**。

意图不清(`kind=general / "BR 数太多了"` 或含 "看能否 / 我觉得 / 也许")→ **先三段式确认**(给 1 个推测 + 影响范围),用户回 OK 才动手。

## 目标

**翻译**散文需求文档为下游可消费的领域模型快照。只回答两个问题:

1. **这份需求里的术语 / Entity / 业务规则 / UseCase 各是什么?**
2. **PM 没说清楚、需要交给 `/bob-spec` 进一步消化的开放问题有哪些?**

**不写代码、不切 story、不画架构**。产出一份**交互式 html**(Stage 2-3.5,review canvas)+ 一份 md(Stage 4 dump,SSoT)。

## 工作流(6 个 Stage)

```
Stage 0. 入口体检 + 短路判定(可跳过建模)
Stage 1. 抽取(术语 / Entity / 规则 / UseCase / 开放问题)— 三段式追问填空(in-memory snapshot,不写文件)
Stage 2. 生成 interactive html 内容(Claude 在内存里 compose)
Stage 3. 启 visual companion server + push html + 给用户 URL
Stage 3.5. 多轮修改循环(event-driven,默认进入)
Stage 4. 三段式收口(用户给推进信号 → dump md → 停 server)
```

---

## Stage 0. 入口体检 + 短路判定

读取并通报:

| 状态 | 判定条件 | 后续行为 |
|---|---|---|
| **缺输入** | 无 `<doc-path>` 参数 + 无 `--story` + 找不到 survey 的源文档引用 | 拒绝运行,提示用户提供文档路径 |
| **极小需求** | 文档体量 < 50 行 + 概念 ≤ 3 + 无跨 story 共享规则 | 三段式询问"短路:写占位 md 后继续"。用户同意则写**一份占位 md** 留痕(空段 + 短路理由)。**model 阶段视为已完成**,下游 stories 门禁通过 |
| **常规** | 其他 | → Stage 1 |

向用户三段式通报:

> **Q0: 这份需求按哪种模式建模?**
>
> **推测**:常规 / 极小需求 / 缺输入
> **理由**:`<具体证据:行数 / 概念数 / 跨 story 共享规则数>`
> **推荐选择**:`继续建模(常规)` / `短路(写占位 md 后继续)` / `补充输入`
>
> 是否同意?

> **不变量**:**短路**分支 → Stage 0 立刻写一份**占位 md**(`docs/bob/03-model-<slug>-<date>.md`)+ 退出;**常规**分支 → md 不在此处写,会在 **Stage 4 推进**时从最终 html 状态 dump。无论哪条路径,最终都会落 md;下游 `/bob-stories` 在 Stage 0 硬校验本文件存在。"短路"≠"跳过 model" —— 它只是把内容压缩到占位 md。

输出文件名计算:
- `<slug>` = Claude 从源文档名 + 业务语义生成的 3-5 字符 kebab-case 标识符(如 "ycb需求.md" + 业务"创建订单" → "create-order");**同一 slug 贯穿 Stage 0/2/3/4 所有阶段**(短路占位 md / 交互式 html / Stage 4 dump 都用此值)
- `<date>` = `YYYYMMDD`(UTC)
- 输出:`docs/bob/03-model-<slug>-<date>.md` + `docs/bob/03-model-<slug>-<date>.html`
- **同一天再跑** → 覆盖同名文件;**跨天** → 新文件,旧文件保留(团队可手动清理)

`--refresh` flag 显式触发 Stage 1,即使 Stage 0 判定为"极小需求"。

---

## Stage 1. 抽取(领域核心工作)

仅在 Stage 0 判定为 **常规** 或用户主动选择 `--refresh` 时执行。

### 1.1 按格式读源文档

| 后缀 | 读取方式 |
|---|---|
| `.pdf` | Read 工具的 `pages` 参数分批读(每次 5-10 页),逐批抽取 |
| `.docx` | 同 PDF,Read 工具直接处理(注:抽取质量取决于文档结构) |
| `.md`、`.txt`、`.markdown` | 一次性 Read 全文 |
| 其他 | 跳过,在 md/html 报告里标注"格式不支持" |

### 1.2 抽取 5 段内容

Claude 按下列顺序识别,**每段三段式追问填空**(不抛开放问题)。

#### A. 术语表(Glossary)

| 字段 | 说明 |
|---|---|
| 中文 | 例:订单 |
| 英文 | 例:Order |
| 定义 | 一句话 |
| 来源 | story / AC 编号 |
| 同义词 | 散文里出现的其他叫法,统一指向此条 |

#### B. Entity 草图

每个 Entity 一段:

- **属性清单** —— 名字 + 类型 + 必填(从 AC 推断)
- **状态机种子** —— 已出现的状态名 + 已知的迁移箭头;**未确认的状态用虚线**,标注"待 /bob-spec 确认"
- **不变量** —— 用 AC 措辞反推的 invariant,例:"一个 Order 只能包含一个商家的餐品"(YCB-001 AC#5)
- **生命周期事件**(可选) —— "创建时""提交时""完结时"等关键 hook

#### C. 业务规则清单(Business Rules)

| 规则 ID | 类型 | 公式/约束 | 来源 |
|---|---|---|---|
| BR-001 | 计算 | `Σ(item.price × item.qty) + 1 + 3` | YCB-001 价格计算 |
| BR-002 | 格式 | orderNumber = `yyyyMMddHHmmss + 6 位随机数` | YCB-001 AC#2 |
| BR-003 | 约束 | 一个 Order 只能一个商家 | YCB-001 AC#5 |

规则 ID 命名:`BR-<NNN>`,顺序递增,跨 story 共享。下游 spec 在 GWT 里直接 `// 验证 BR-001`,TDD 测试方法名可以是 `shouldCalculatePriceAccordingToBR001`。

#### D. UseCase 初步清单

| UseCase | 涉及 Entity | 涉及规则 | 来源 story | 备注 |
|---|---|---|---|---|
| CreateOrder | Order | BR-001 / BR-002 / BR-003 | YCB-001 + 1.1 + 1.3 | 命令型 |
| CalculateOrderPrice | Order | BR-001 | YCB-001 + 1.2 | 命令型 |
| ApplyDiscount | Order, DiscountScheme | (待定) | YCB-003 | 命令型 |

这只是**初步切**。`/bob-stories` 拿到后会按 4 因子复评再 1:1 切到 UseCase 粒度。

#### E. 开放问题清单

**显式列出**散文需求没说清楚的:

| 编号 | 问题 | 影响哪个下游 | 暂定假设 |
|---|---|---|---|
| Q1 | YCB-003 多种折扣是否可以叠加?顺序? | spec / TDD | 暂定不可叠加,以最高优惠为准 |
| Q2 | 除 PENDING_PAYMENT 外完整 Order 状态机? | onion / spec | 暂只建模 PENDING_PAYMENT;其他用虚线 |
| Q3 | 订单超时未支付是否自动取消? | onion / spec | 暂不实现 |
| Q4 | 配送地址校验?(手机号格式) | spec | 暂不校验 |

下游 `/bob-spec` 的"交给 Superpowers 的开放问题"段会进一步消化这些 Q。

### 1.3 三段式确认抽取结果

> **Q1: 抽取完成。N 条术语 / M 个 Entity / K 条业务规则 / U 个 UseCase / W 个开放问题。**
>
> **推测**:看起来覆盖完整 / 还有遗漏(列出我看到但未确认的概念)
> **理由**:基于源文档的 grep + AC 反推
> **推荐选择**:`确认无误,进入 Stage 2 写 md` / `补充遗漏后再写`
>
> 是否同意?

---

## Stage 2. 生成 interactive html 内容(Claude 在内存里 compose)

**重要变化**:本阶段**不再写 md**(md 推迟到 Stage 4);本阶段 Claude **在内存里 compose 完整的 html 字符串**,Stage 3 才把它写盘 + 启 server。

### 2.1 html 必含组件(详见 §html widget 规范)

- DOCTYPE + 完整 `<html><head><body>` 结构(本 html 由 server 直传不经 frame wrapper)
- Sticky 顶栏:模型标题 / sticky 计数 / [↻ 清空草稿] / [📋 提交本轮反馈 (N)] 按钮
- 左侧 TOC(sticky)+ 主内容区
- 5 节内容(术语 / Entity / BR / UC / 开放问题),每节内 widget 见 §html widget 规范
- 跨引用锚点:BR-NNN / Entity / INV / Q 全自动锚点
- 嵌入 inline CSS(状态色标 / 布局)
- 嵌入 inline JS(localStorage / 提交 / 自动保存,详见 §html widget 规范的 page-helper)
- 顶部 `<script>window.BOB_MODEL_SLUG='<slug>'; window.BOB_MODEL_ROUND=1;</script>`(Claude 按当次会话填)

### 2.2 文件名计算(与旧版相同)

- `<slug>` = 源文档名去后缀 + 业务化("ycb需求.md" → "create-order")
- `<date>` = `YYYYMMDD` UTC
- 输出文件名(Stage 3 写盘时用):`screen_dir/03-model-<slug>-<date>.html`

### 2.3 不在此阶段做的事

- ❌ 写 md(由 Stage 4 dump)
- ❌ 启 server(由 Stage 3 启)
- ❌ 把 html 写到 `docs/bob/`(因为最终的 path 是 `screen_dir`,且 Stage 4 才把 html 复制 / 移到 `docs/bob/`)

---

## Stage 3. 启 visual companion server + push html + 给用户 URL

### 3.1 启 server

```bash
SCRIPT="<project-root>/.claude/skills/bob-model/scripts/start-server.sh"
"$SCRIPT" --project-dir <project-root>
```

返回 JSON 包含 `port` / `url` / `screen_dir` / `state_dir`。Claude 在内部记下 `screen_dir` 与 `state_dir`。

> **失败兜底**:若 start-server.sh 返回非 0,Claude 终端报错并降级 — 把 Stage 2 的 html 直接写到 `docs/bob/03-model-<slug>-<date>.html`(只读 fallback),并告知用户"interactive review 不可用,可作为只读 review"。

### 3.2 写 html 到 screen_dir

把 Stage 2 compose 的 html 字符串写到:
```
<screen_dir>/03-model-<slug>-<date>.html
```

server 会自动检测新文件并 broadcast `{type: 'reload'}` 到所有连接的 browser(server.cjs:296)。

### 3.3 通报用户(per §产物报告规约)

```
**产物**(review 直链):
- url(浏览器打开):http://localhost:<port>
- screen_dir(html 本体):/Users/.../03-model-<slug>-<date>.html
- state_dir(events 文件):/Users/.../state/events(本轮还没,首次提交后才生成)

请打开 URL,在 widget 里写评论 + 点 sticky 顶栏「📋 提交本轮反馈」。
完成后在本终端发 "继续" 让我读 events。
```

### 3.4 进入 Stage 3.5 等待

Stage 3 至此结束。Claude **不主动追问**,等用户在终端发推进信号。

---

## Stage 3.5 · 多轮修改循环(event-driven,默认进入)

**仍是默认进入,Claude 不主动追问 Stage 4**(per commit `6e21e53` 不变量);区别于旧版,本阶段改为 event-driven:用户在 browser 提交 → server 写 events → Claude 在终端被叫"继续" → 读 events → 应用 → 重写 html。

### 3.5.1 触发(用户在终端给出推进/继续/查反馈信号)

监听以下信号(任一即进入处理):
- "继续" / "next" / "看反馈" / "看下反馈" / "处理一下" / 类似
- 默认假设用户已在 browser 点过 sticky 顶栏「📋 提交本轮反馈」

> 若用户在终端发的不是上述信号(而是直接命令 / 别的问题),仍走 §Stage 3.5 旧版 protocol(直接 Edit md/html);**新版 event-driven 只在用户给"继续"类信号时启动**。

### 3.5.2 处理流程

1. **读 events**:
   ```bash
   tail -n+1 "$STATE_DIR/events"
   ```
   Claude 内部 filter 出 `type === 'bob-model-feedback'` 且 `timestamp > last_processed_event_timestamp` 的行。

2. **Parse + 分派**(按 §评论协议与 schema 的 6 种 kind):
   - `term` → 改术语行 + 级联引用
   - `entity-field` → 改字段 + 级联引用
   - `br` → 改 BR 卡
   - `diagram` → 重画 Mermaid
   - `open-question` → 决议 / 改暂定 / 拆分
   - `general` → **先三段式**确认意图再动手

3. **更新内部模型快照** + 更新 `last_processed_event_timestamp` + `processed_comment_ids`

4. **重写 html**:重新 compose html 字符串(per Stage 2 规范),关键变化:
   - 对每个**已应用的 comment.id**,在对应 widget 上加 `data-applied="<comment-id>"` 属性
   - 重新生成跨引用锚点(因为可能 rename 了 Entity / BR)
   - BOB_MODEL_ROUND++

5. **写盘新 html**:**新文件名**(不覆盖旧版)避免 server 把它当 update 而 broadcast reload 之外的副作用。例如 `screen_dir/03-model-<slug>-<date>-v2.html`、`v3.html` 等。

> server.cjs:287 新文件触发 `{type: 'screen-added'}` + **清空 events 文件**(server.cjs:290)。Claude 必须**先读完本批 events** 再写新文件;否则新提交事件会丢。

6. **报告改动 + 列 URL**(per §产物报告规约):
   ```
   已应用本轮 N 条改动:
   - <kind:target>: <一行总结>
   - ...
   
   **产物**(review 直链):
   - url: http://localhost:<port>(刷新看新版)
   - screen_dir: /Users/.../03-model-<slug>-<date>-vN.html
   ```

### 3.5.3 自由文本兜底(per spec §4.4)

`kind === 'general'` 或 comment 含"也许 / 看能否 / 我觉得"等不确定措辞 → **先三段式**确认意图(给出 1 个推测方案 + 影响范围),用户回 OK 才进 step 4。

### 3.5.4 server 中途死

读 events 前先 `ls "$STATE_DIR/server-info" "$STATE_DIR/server-stopped"`:
- 只有 server-info:正常,继续读 events
- server-stopped 存在 → 重启 server(复用同 screen_dir 不丢历史 html)+ 告知用户

### 3.5.5 不要做的事(per commit `6e21e53` 多轮修改协议)

- ❌ 改动后追问"是否进入 /bob-stories" —— 那是 Stage 4 的专责
- ❌ 把单点改动当 Stage 1.3 全量重审
- ❌ 在改动报告里夹带 Stage 4 三段式

---

## Stage 4. 三段式收口(用户给推进信号 → dump md → 停 server)

### 4.1 触发(用户显式给出退循环信号)

- "OK 推进" / "可以了" / "继续 stories" / "/bob-stories" / "done with model" / 类似
- 若用户多轮未动,Claude **可一次**主动问 "还有改动吗?",收到 "没了" 再进 Stage 4

### 4.2 dump md(从最终内部模型快照)

写 `<repo>/docs/bob/03-model-<slug>-<date>.md`(per 旧版 schema),frontmatter 含:
- `name: bob-model`
- `source_doc: <绝对路径>`
- `source_doc_sha256: <shasum>`
- `generated_at: <UTC ISO8601>`
- `target_phase: pre-stories`
- `slug: <slug>`
- **新增**:`interactive_review: { rounds: <N>, comments_applied: <total>, final_html: <path> }`

### 4.3 保留 final html 到 docs/bob/

把 `screen_dir/03-model-<slug>-<date>-vN.html`(最终版)**复制**到 `<repo>/docs/bob/03-model-<slug>-<date>.html`(去掉 vN 后缀,作为 final;入 git)。

### 4.4 停 server(优雅 SIGTERM)

```bash
kill -TERM $(cat "$STATE_DIR/server.pid")
```

(或调用 stop-server.sh 如存在)

### 4.5 三段式收口(per §产物报告规约的 Stage 4 模板)

```
> **Q: 建模完成。N 条术语 / M 个 Entity / K 条业务规则 / W 个开放问题 + <rounds> 轮交互评审 / <applied> 条评论已应用。**
>
> **产物**(review 直链):
> - md(SSoT):/Users/.../docs/bob/03-model-<slug>-<date>.md
> - html(团队视图,浏览器打开):/Users/.../docs/bob/03-model-<slug>-<date>.html
>
> **推测**:难度 `<Medium/Hard>` → 建议 `/bob-stories`;`<Easy>` → 直接 `/bob-identify`
> **理由**:从 /bob-survey + 本模型规模综合
> **推荐选择**:`继续 /bob-stories` / `直接 /bob-identify` / `先 review 后再决定`
>
> 是否同意?(回"是"按推荐;回"先 review"暂停;回"回头再说"也可)
```

### 4.6 失败兜底

- md dump 失败(磁盘 / 权限)→ 不停 server,允许用户继续 review
- final html 复制失败 → 用户从 screen_dir 手动复制,Claude 给出明确路径
- server 已 idle 自杀 → SIGTERM no-op,跳过

---

## 不变量

- **强制阶段(不可跳过)** —— Medium/Hard 链路上 `/bob-model` 必跑。`/bob-stories` 在 Stage 0 硬校验 `docs/bob/03-model-*.md` 存在,缺失立即拒绝。"极小需求"可走短路,但仍必须产出占位 md。
- **命名表意强制(原则普适,具体词按域调整)** —— 术语 / Entity 字段 / 类型名必须显式编码 type + role,禁止光秃名词。**具体后缀按域换**:电商 `Fee` / 物流 `Gram` / IoT `Celsius` / 医疗 `Mg` ...,**不要把电商示例当唯一标准**。详见 §命名规约「核心原则」+「跨领域适用」。违反 = 本阶段未完成,需 `--refresh` 重抽。
- **多轮修改是默认** —— Stage 3(html 落盘)与 Stage 4(收口)之间**默认进入修改循环**,3-8 轮迭代是常态。Claude **不主动**追问"是否进入下一步";**只在用户显式给推进信号**("OK 推进" / "继续 stories" / 等)时才发 Stage 4 三段式。详见 §Stage 3.5。
- **报告必含文件链接** —— 每次产物落盘 / 改动报告 / Stage 4 收口都必须**显式列出 md 与 html 绝对路径**,方便用户直接打开 review。**每轮都要列**(即使路径没变),不要省略,不要藏在散文里。详见 §产物报告规约。
- **html 是 review canvas(非只读)** —— Stage 2 起 html 含 widget / 状态机 / localStorage / WebSocket 提交;不再是只读视图。详见 §html widget 规范。
- **md 仅 Stage 4 生成** —— Stage 2 不写 md;Stage 3.5 只动 html;md 在用户给 Stage 4 推进信号后从最终 html 状态 dump。下游 `/bob-stories` 通过"是否有 md"自然判断 model 是否 final。
- **server 自启自停** —— Stage 3 自动 spawn visual companion server;Stage 4 自动 SIGTERM。idle 30 min 自杀由 server 自管;Claude 检测到 server-stopped 后自动重启。
- **md 是 SSoT(Stage 4 起)** —— Stage 4 dump 后,md 即为下游消费的 final;html 仅是视图,每次 `/bob-model` 运行重写
- **html 入 git** —— 团队 PR review 用,差异可见
- **Mermaid via CDN** —— 单 `<script>` 标签;离线时优雅降级为代码块
- **`docs/bob/03-` 槽位独占** —— model 用这个槽,其他 skill 不占
- **不内置业务术语** —— 完全 runtime 抽取,无 schema fixture
- **不引入新 R 规则** —— Model 是翻译层,不构造硬约束
- **覆盖 vs 累积** —— 同一天 + 同源文档 → 覆盖;跨天 → 新文件(自动保留历史)
