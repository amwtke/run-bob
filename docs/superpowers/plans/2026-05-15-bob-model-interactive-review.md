# bob-model Interactive Review · Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade `/bob-model` output from read-only md+html to an interactive review canvas: every term / Entity field / BR / diagram / open-question gets an inline comment widget; user submits a batch via WebSocket to the visual companion server; Claude reads events, applies changes, re-pushes html; md is generated only at Stage 4.

**Architecture:** Server polling (reuse `superpowers/brainstorming/scripts/start-server.sh` + WebSocket via auto-injected `window.brainstorm.send`). HTML is the source of truth during Stage 3.5 iteration; md dumps at Stage 4. Inline JS + CSS in every generated html (no separate static asset).

**Tech Stack:**
- `src/templates/skills/bob-model.md` — the skill template Claude renders from (Markdown)
- `superpowers/brainstorming/scripts/server.cjs` — Node.js WebSocket server (read-only dependency; verified)
- Browser-side: vanilla JS (no frameworks), localStorage, fetch (only for fallback debug — primary path is WebSocket via `window.brainstorm.send`)
- Mermaid 10 via CDN (existing dep)
- Demo target: `/Users/xiaojin/workshop/tmp/ycb/docs/bob/03-model-create-order-20260515.html`

**Design spec:** `docs/superpowers/specs/2026-05-15-bob-model-interactive-review-design.md`

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `docs/superpowers/specs/2026-05-15-bob-model-interactive-review-design.md` | **modify** | Patch §4 (POST → WebSocket) per Task 0 finding |
| `src/templates/skills/bob-model.md` | **heavy modify** | Rewrite Stage 2/3/3.5/4; add §评论协议 + §html widget 规范; update 不变量; tighten §产物报告规约 |
| `/Users/xiaojin/workshop/tmp/ycb/docs/bob/03-model-create-order-20260515.html` | **rewrite** | Demo retrofit — single artifact embodying the new widget design |
| `/Users/xiaojin/workshop/tmp/ycb/docs/bob/03-model-create-order-20260515.md` | **leave** | Stays as-is (was Stage 4 product from previous session); future `/bob-model` runs won't regenerate until Stage 4 推进 |

No new files in run-bob. The page-side JS is embedded in the html template (which lives inside the skill markdown as a code block / template).

---

## Task 0 · Validate WebSocket protocol(已完成,记录结论)

**Files:**
- Inspect: `/Users/xiaojin/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/server.cjs`
- Inspect: `.../scripts/helper.js`

- [x] **Step 0.1: Inspect server.cjs message recording logic**

Confirmed: server records to `$STATE_DIR/events` only when message has truthy `.choice` field (server.cjs lines 234-237).

- [x] **Step 0.2: Inspect helper.js WebSocket setup**

Confirmed: helper.js opens `ws://<location.host>` and exposes `window.brainstorm.send(event)` for arbitrary JSON. Auto-injected by server when serving fragments OR when serving full DOCTYPE pages (per visual-companion.md).

- [x] **Step 0.3: Lock decision**

**Decision**: Use WebSocket via `window.brainstorm.send({...envelope, choice: 'submit'})`. The `choice: 'submit'` field is the trigger for server-side recording; type field distinguishes from brainstorming click events.

---

## Task 1 · Patch spec §4: POST → WebSocket

**Files:**
- Modify: `docs/superpowers/specs/2026-05-15-bob-model-interactive-review-design.md`

- [ ] **Step 1.1: Read existing §4.1 envelope schema in spec**

Confirm wording around "POST /events" / "POST body" — needs to be reworded to "WebSocket message via `window.brainstorm.send()` with `choice: 'submit'` trigger field".

- [ ] **Step 1.2: Apply Edit**

```
Old: ### 4.1 Event envelope(POST body / events JSONL 行)
New: ### 4.1 Event envelope(WebSocket message via window.brainstorm.send / events JSONL 行)
```

Add `"choice": "submit",` line to the JSON example after `"type"` field, and add a line of explanation:

> 字段语义补充:`choice` 是 brainstorming server 的记录触发字段(server.cjs:234 只记录 `event.choice` 非空的消息),固定为 `"submit"`;Claude 读 events 时按 `type === "bob-model-feedback"` 过滤,忽略 choice。

- [ ] **Step 1.3: Update §2.2 data flow step 3-4**

```
Old (step 3-4):
3. Browser JS collectFeedback() 聚合所有 textarea → POST /events
4. Server append JSONL 到 $STATE_DIR/events

New:
3. Browser JS collectFeedback() 聚合所有 textarea → window.brainstorm.send(envelope)
4. WebSocket server (server.cjs) 收到 → 检测 event.choice 非空 → append JSONL 到 $STATE_DIR/events
```

- [ ] **Step 1.4: Update §10 risk row "visual companion server 不支持自定义 POST body"**

Change probability from "中" to "无(已验证)";recovery 字段填 "已验证通过 WebSocket + choice 触发字段实现"。

- [ ] **Step 1.5: grep verify**

Run: `grep -nc "POST /events" docs/superpowers/specs/2026-05-15-bob-model-interactive-review-design.md`
Expected: 0(应 0 处残留 POST 措辞)

- [ ] **Step 1.6: Commit**

```bash
git add docs/superpowers/specs/2026-05-15-bob-model-interactive-review-design.md
git commit -m "docs(specs): correct bob-model review feedback transport: WebSocket not POST"
```

---

## Task 2 · 起草 page-helper JS(草稿成本最高的部分,先确定它再嵌)

**Files:**
- Will-be-embedded in: `src/templates/skills/bob-model.md` (later in Task 7);先写到一个临时 scratch 文件方便迭代和review
- Create scratch: `/tmp/bob-model-page-helper.js`

- [ ] **Step 2.1: Write scratch JS file**

Create `/tmp/bob-model-page-helper.js` with the full inline helper:

```javascript
// bob-model interactive review · page helper (inline, no module system)
// Globals expected: window.BOB_MODEL_SLUG (str), window.BOB_MODEL_ROUND (int)
(function() {
  if (!window.BOB_MODEL_SLUG) { console.error('BOB_MODEL_SLUG missing'); return; }
  const SLUG = window.BOB_MODEL_SLUG;
  const DRAFTS_KEY = `bob-model:${SLUG}:drafts`;
  const SUBMITTED_KEY = `bob-model:${SLUG}:submitted`;

  // -- localStorage helpers --
  const loadDrafts = () => JSON.parse(localStorage.getItem(DRAFTS_KEY) || '{}');
  const saveDrafts = (d) => localStorage.setItem(DRAFTS_KEY, JSON.stringify(d));
  const loadSubmitted = () => JSON.parse(localStorage.getItem(SUBMITTED_KEY) || '[]');
  const saveSubmitted = (s) => localStorage.setItem(SUBMITTED_KEY, JSON.stringify(s));

  // -- widget key (= comment-id) helpers --
  function widgetKey(widget) {
    // data-comment-id is the stable identifier; equal to {kind}:{target} or 'general'
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

      // 1. Applied? (Claude rewrote html with data-applied attr on widget)
      if (widget.hasAttribute('data-applied')) {
        setState(widget, 'applied');
        cleanedSubmitted.delete(key);
        return;
      }

      // 2. Submitted but not yet applied?
      if (submitted.has(key)) {
        setState(widget, 'submitted');
        return;
      }

      // 3. Draft?
      if (drafts[key]) {
        if (textarea) textarea.value = drafts[key];
        setState(widget, 'draft');
        return;
      }

      // 4. Default
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

  // -- sticky counter + submit button enable --
  function updateCounter() {
    const draftCount = document.querySelectorAll('.state-draft').length;
    const counterEl = document.getElementById('draft-counter');
    const submitBtn = document.getElementById('submit-button');
    if (counterEl) counterEl.textContent = draftCount;
    if (submitBtn) submitBtn.disabled = draftCount === 0;

    // Per-section counts in TOC
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
      const kind = widget.dataset.kind;
      const target = widget.dataset.target || null;
      return {
        id: `c-${ts}-${String(i + 1).padStart(3, '0')}`,
        kind,
        target,
        comment
      };
    });
    return {
      type: 'bob-model-feedback',
      choice: 'submit',
      slug: SLUG,
      round: parseInt(window.BOB_MODEL_ROUND || '1', 10),
      timestamp: ts,
      comments
    };
  }

  // -- submit via window.brainstorm.send --
  function submitFeedback() {
    const envelope = collectFeedback();
    if (envelope.comments.length === 0) return;

    if (!window.brainstorm || !window.brainstorm.send) {
      showToast('WebSocket helper 未加载 (页面可能未通过 server 访问)');
      return;
    }
    try {
      window.brainstorm.send(envelope);
      // Move drafts → submitted
      const submitted = loadSubmitted();
      envelope.comments.forEach(c => {
        const key = c.kind === 'general' ? 'general' : `${c.kind}:${c.target}`;
        if (!submitted.includes(key)) submitted.push(key);
      });
      saveSubmitted(submitted);
      saveDrafts({});

      // UI transition
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

  // -- toast --
  function showToast(msg) {
    const t = document.createElement('div');
    t.className = 'bob-toast';
    t.textContent = msg;
    document.body.appendChild(t);
    setTimeout(() => t.classList.add('show'), 10);
    setTimeout(() => { t.classList.remove('show'); setTimeout(() => t.remove(), 300); }, 3000);
  }

  // -- comment toggle (show/hide textarea) --
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
```

- [ ] **Step 2.2: Open in browser manually + sanity check syntax**

Run: `node -c /tmp/bob-model-page-helper.js` (syntax-only parse)
Expected: no output, exit 0

- [ ] **Step 2.3: Note testability**

No automated tests for browser JS (run-bob is Rust; adding Node test infra is overkill). All JS testing is **manual browser** in Task 11.

- [ ] **Step 2.4: Stage the JS for embedding(暂不提交,Task 7 嵌入 skill 后再 commit)**

This JS will be embedded inside the skill template's §html widget 规范 section in Task 7. Keep the scratch file for now.

---

## Task 3 · skill: rewrite Stage 2(改为生成 html 内容)

**Files:**
- Modify: `src/templates/skills/bob-model.md`

- [ ] **Step 3.1: Read current Stage 2 section**

Run: `grep -n "^## Stage 2" src/templates/skills/bob-model.md`
Confirm location.

- [ ] **Step 3.2: Apply Edit — replace Stage 2 body**

Find old text:
```markdown
## Stage 2. 写 md(SSoT,下游消费)
[entire section through `--- ` before Stage 3]
```

Replace with:
```markdown
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
- 顶部 `<script>window.BOB_MODEL_SLUG='create-order'; window.BOB_MODEL_ROUND=1;</script>`(Claude 按当次会话填)

### 2.2 文件名计算(与旧版相同)

- `<slug>` = 源文档名去后缀 + 业务化("ycb需求.md" → "create-order")
- `<date>` = `YYYYMMDD` UTC
- 输出文件名(Stage 3 写盘时用):`screen_dir/03-model-<slug>-<date>.html`

### 2.3 不在此阶段做的事

- ❌ 写 md(由 Stage 4 dump)
- ❌ 启 server(由 Stage 3 启)
- ❌ 把 html 写到 `docs/bob/`(因为最终的 path 是 `screen_dir`,且 Stage 4 才把 html 复制 / 移到 `docs/bob/`)
```

- [ ] **Step 3.3: grep verify Stage 2 rewrite intact**

Run: `grep -A2 "^## Stage 2" src/templates/skills/bob-model.md | head -4`
Expected: 第一行 "## Stage 2. 生成 interactive html 内容(Claude 在内存里 compose)"

- [ ] **Step 3.4: Commit**(暂缓 — 与 Task 4-9 合并一个 "Stages 2-4 rewrite + new sections" commit)

(no commit yet)

---

## Task 4 · skill: rewrite Stage 3(启 server + 写 html + 给 URL)

**Files:**
- Modify: `src/templates/skills/bob-model.md`

- [ ] **Step 4.1: Apply Edit — replace Stage 3 body**

Find old Stage 3 section, replace with:

```markdown
## Stage 3. 启 visual companion server + push html + 给用户 URL

### 3.1 启 server

```bash
SCRIPT=/Users/xiaojin/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/start-server.sh
"$SCRIPT" --project-dir <repo-root>
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
```

- [ ] **Step 4.2: grep verify**

Run: `grep -n "screen_dir" src/templates/skills/bob-model.md | wc -l`
Expected: ≥ 4 occurrences in Stage 3 area

---

## Task 5 · skill: rewrite Stage 3.5(event-driven loop)

**Files:**
- Modify: `src/templates/skills/bob-model.md`

- [ ] **Step 5.1: Apply Edit — replace Stage 3.5 body**

Find old Stage 3.5 section, replace with:

```markdown
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
```

- [ ] **Step 5.2: grep verify**

Run: `grep -n "event-driven\|last_processed" src/templates/skills/bob-model.md | wc -l`
Expected: ≥ 3

---

## Task 6 · skill: rewrite Stage 4(dump md + 停 server)

**Files:**
- Modify: `src/templates/skills/bob-model.md`

- [ ] **Step 6.1: Apply Edit — replace Stage 4 body**

Find old Stage 4 section, replace with:

```markdown
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
```

- [ ] **Step 6.2: grep verify**

Run: `grep -n "Stage 4 \|dump md\|SIGTERM" src/templates/skills/bob-model.md | head -5`
Expected: 多处出现

---

## Task 7 · skill: 新增 §html widget 规范(嵌入 CSS + JS)

**Files:**
- Modify: `src/templates/skills/bob-model.md`

- [ ] **Step 7.1: 找插入位置**

`§产物报告规约` 之后、`## 目标` 之前(与 §命名规约 同层)。

- [ ] **Step 7.2: 插入新章节**

```markdown
## html widget 规范(Stage 2 compose 时必照)

每次 `/bob-model` Stage 2 生成 html 时,必须遵循以下结构 + 内嵌 CSS + 内嵌 JS。

### Page-level 骨架

```html
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
    <nav class="bob-toc">...</nav>
    <main class="bob-main">
      <!-- §1 术语表 / §2 Entity / §3 BR / §4 UC / §5 Q -->
    </main>
  </div>
  <script>/* 见 §JS */</script>
</body>
</html>
```

### Widget DOM 模板

每个 widget 必须有 3 个 data-* 属性:`data-comment-id`(唯一,= `<kind>:<target>` 或 'general')、`data-kind`、`data-target`,以及一个 `.comment-input` textarea。

5 种形态详见 spec §3.3。统一格式:

```html
<div class="bob-widget" data-comment-id="term:discountRate" data-kind="term" data-target="discountRate" data-section="terms">
  <button class="comment-toggle">💬 <span class="count">0</span></button>
  <textarea class="comment-input" placeholder="对此 term 的修改意见..."></textarea>
</div>
```

### CSS(嵌 head 内 <style>)

```css
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
```

### JS(嵌 body 末 <script>)

[完整 JS 见 Task 2 起草版本,210 行;Stage 2 compose html 时整段拷入 <script> 标签]

### 不变量

- 每个 widget 必有 `data-comment-id` / `data-kind` / `data-target` / `data-section`
- 所有 cross-ref 必须是 `<a href="#br-001">BR-001</a>` 格式(自动生成,Claude compose 时正则替换)
- Stage 4 dump md 时,**自动 strip** 所有 widget DOM(只保留语义内容)→ md 干净
```

(JS 太长,Task 7 这里只引用 "见 Task 2",真正落地时整段拷)

- [ ] **Step 7.3: 把 Task 2 的 JS 嵌入 §html widget 规范 末尾**

把 `/tmp/bob-model-page-helper.js` 的全部内容,作为一个 ```js 代码块,贴到 §html widget 规范 的 "### JS" 子节之下,替代占位文本。

- [ ] **Step 7.4: grep verify**

Run: `grep -c "BOB_MODEL_SLUG" src/templates/skills/bob-model.md`
Expected: ≥ 3(被 page-template 引用、JS 自检、不变量提及)

---

## Task 8 · skill: 新增 §评论协议与 schema

**Files:**
- Modify: `src/templates/skills/bob-model.md`

- [ ] **Step 8.1: 插入新章节(在 §html widget 规范 之后)**

```markdown
## 评论协议与 schema(Stage 3.5 必照)

### Envelope schema(WebSocket message via window.brainstorm.send / events JSONL 行)

```json
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
```

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

意图不清(`kind=general / "BR 数太多了"` 或 含 "看能否 / 我觉得 / 也许")→ **先三段式确认**(给 1 个推测 + 影响范围),用户回 OK 才动手。
```

- [ ] **Step 8.2: grep verify**

Run: `grep -c "bob-model-feedback\|last_processed" src/templates/skills/bob-model.md`
Expected: ≥ 4

---

## Task 9 · skill: 更新不变量(+3 行)+ §产物报告规约(微调)

**Files:**
- Modify: `src/templates/skills/bob-model.md`

- [ ] **Step 9.1: Apply Edit — 不变量列表插入 3 行**

在 "强制阶段 / 命名表意 / 多轮修改 / 报告必含文件链接 / md 是 SSoT" 之间插入:

```
- **html 是 review canvas(非只读)** —— Stage 2 起 html 含 widget / 状态机 / localStorage / WebSocket 提交;不再是只读视图。详见 §html widget 规范。
- **md 仅 Stage 4 生成** —— Stage 2 不写 md;Stage 3.5 只动 html;md 在用户给 Stage 4 推进信号后从最终 html 状态 dump。下游 `/bob-stories` 通过"是否有 md"自然判断 model 是否 final。
- **server 自启自停** —— Stage 3 自动 spawn visual companion server;Stage 4 自动 SIGTERM。idle 30 min 自杀 由 server 自管;Claude 检测到 server-stopped 后自动重启。
```

并修改 "**md 是 SSoT**" 改为 "**md 是 SSoT(Stage 4 起)**":

```
- **md 是 SSoT(Stage 4 起)** —— Stage 4 dump 后,md 即为下游消费的 final;html 仅是视图,每次 `/bob-model` 运行重写
```

- [ ] **Step 9.2: Apply Edit — §产物报告规约**

把现有 Stage 4 收口模板的"必含"列表,补充 url + state_dir:

```
- url(浏览器打开):http://localhost:<port>(Stage 3.5 期间提供;Stage 4 后 server 已停,可忽略)
- screen_dir(html 本体):/Users/.../03-model-<slug>-<date>-vN.html(Stage 3.5 期间;Stage 4 后由 final html 替代)
- md(SSoT):/Users/.../docs/bob/03-model-<slug>-<date>.md(Stage 4 起才存在)
- html(团队视图):/Users/.../docs/bob/03-model-<slug>-<date>.html(Stage 4 起从 screen_dir 复制)
```

- [ ] **Step 9.3: grep verify**

Run: `grep -c "review canvas\|仅 Stage 4 生成" src/templates/skills/bob-model.md`
Expected: ≥ 2(每个不变量 1 处 + 引用)

- [ ] **Step 9.4: Commit Tasks 3-9 as one big skill rewrite commit**

```bash
git add src/templates/skills/bob-model.md
git commit -m "$(cat <<'EOF'
feat(model): bob-model interactive review — Stages 2-4 rewrite + §html widget 规范 + §评论协议

Implements design from docs/superpowers/specs/2026-05-15-bob-model-interactive-review-design.md
(commit 045a858).

- Stage 2: write md → compose interactive html (in-memory)
- Stage 3: write html (read-only) → start visual companion server + write html to screen_dir + give URL
- Stage 3.5: edit md/html sync → event-driven loop (read $STATE_DIR/events on user signal, apply per-kind handlers, rewrite html-vN)
- Stage 4: 三段式 → dump md from final state, copy html to docs/bob/, SIGTERM server, 三段式
- New §html widget 规范: page skeleton + widget DOM template + CSS for 4 states + inline JS (~210 lines)
- New §评论协议: envelope schema, 6 kinds, idempotency, freeform fallback
- Invariants +3: html is review canvas / md only Stage 4 / server self-managed
- §产物报告规约 微调: include URL + screen_dir during Stage 3.5

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10 · ycb retrofit(demo)

**Files:**
- Rewrite: `/Users/xiaojin/workshop/tmp/ycb/docs/bob/03-model-create-order-20260515.html`

- [ ] **Step 10.1: Read current ycb html**

Run: `wc -l /Users/xiaojin/workshop/tmp/ycb/docs/bob/03-model-create-order-20260515.html`
Get line count for comparison.

- [ ] **Step 10.2: Compose new html with full widget treatment**

Apply §html widget 规范 (Task 7):
- DOCTYPE + window.BOB_MODEL_SLUG / ROUND
- Sticky header
- TOC with section badges
- 5 sections preserved (内容同 Stage 1 抽取产物)
- 每个 term / Entity field / BR / Mermaid / Q 包成 `.bob-widget` 含 textarea
- 所有 cross-ref 转成 anchor
- 内嵌完整 CSS + JS

(从 ycb 现有 html 的内容里抽数据,塞进新 widget 框架)

- [ ] **Step 10.3: Push to brainstorming server's screen_dir**

注意:演示阶段,直接写到 brainstorming server 的 screen_dir(Task 0 拿到的路径)以便测试。production /bob-model 时,会写到自己 spawn 的 server 的 screen_dir。

```bash
DEST=<screen_dir from server-info>
cp /Users/xiaojin/workshop/tmp/ycb/docs/bob/03-model-create-order-20260515.html "$DEST/ycb-retrofit-demo.html"
```

(或:用 Write 工具直接生成在那个 dest)

- [ ] **Step 10.4: 手动打开 + 视觉检查**

打开浏览器到 server URL → 应见新版交互式 html(sticky 顶栏 + widget + 跨引用)

- [ ] **Step 10.5: Commit ycb 改动(可选 — ycb 不在 run-bob repo 内,所以这步 N/A)**

ycb 在 `/Users/xiaojin/workshop/tmp/ycb` 下,**不属于 run-bob 仓库**。不 commit 到 run-bob;若 ycb 自己是 git repo,可在那里单独 commit;否则文件落盘即可。

---

## Task 11 · 端到端手动测试(per spec §7.1)

**Files:** N/A(纯测试)

- [ ] **Step 11.1: 跑 `/bob-model` 全流程**

(注:本步骤需要更新过的 bob-model.md skill 部署到目标项目;ycb 项目当前 .claude/skills/ 仍是旧版,需手动同步或重跑 install)

在一个新会话里 `/bob-model /Users/xiaojin/workshop/ycb需求.md`,验证:
- ✓ Stage 0 / 1 完成
- ✓ Stage 2 compose html(终端可见 "html 已 compose,准备启 server")
- ✓ Stage 3 启 server + 写 html + 给 URL
- ✓ 浏览器打开见 widget

- [ ] **Step 11.2: 每种 kind 各加 1 条评论 + 提交**

在浏览器:
- ✓ term widget(任选一个术语):写"考虑改名为 X"
- ✓ entity-field widget(任选一个字段):写"改类型为 Y"
- ✓ br widget(任选一个 BR):写"改公式为 Z"
- ✓ diagram widget(状态机图):写"加一个 PAID 转移"
- ✓ open-question widget(Q14):写"决议:按 PRC 惯例"
- ✓ general(可选):写"整体看 BR 太多"

点 sticky 顶栏「📋 提交本轮反馈 (N)」

- [ ] **Step 11.3: 验证 events 写入**

```bash
cat "$STATE_DIR/events" | jq 'select(.type == "bob-model-feedback")'
```

Expected: 见 1 条 envelope,含 6 个 comments,每个 comment 有 `id / kind / target / comment`

- [ ] **Step 11.4: 终端发"继续",验证 Claude 应用**

期望 Claude:
- 报告"已应用本轮 N 条改动" + 每条简短描述
- 列新 URL + new screen_dir path(per §产物报告规约)
- 不追问"是否进入 stories"

刷新浏览器 → 见提交过的 widget 切 ⊘ applied 态(灰虚线)

- [ ] **Step 11.5: 验证状态色标转换全程**

- ○ untouched → 输入 → ● draft(蓝色)
- ● draft → 提交 → ✓ submitted(绿色)
- ✓ submitted → Claude 应用 → 刷新 → ⊘ applied(灰虚线)

- [ ] **Step 11.6: localStorage 持久测试**

写半条评论 → 刷新浏览器 → 草稿 textarea 还在(蓝边)

- [ ] **Step 11.7: 多轮迭代**

连续 3 轮提交 + 应用,确认:
- 每轮 `screen_dir` 新增 vN html(v2 / v3 / v4)
- events 文件每轮被清(server.cjs:290 行为)
- Claude 内部 `last_processed_event_timestamp` 正确

- [ ] **Step 11.8: Stage 4 推进**

终端发 "OK 推进":
- ✓ md 文件生成到 `docs/bob/03-model-<slug>-<date>.md`
- ✓ html 复制到 `docs/bob/03-model-<slug>-<date>.html`(去掉 vN 后缀)
- ✓ server SIGTERM,$STATE_DIR/server-stopped 出现
- ✓ Claude 发 Stage 4 三段式

- [ ] **Step 11.9: 跨引用链接**

在 html 任意位置点 `BR-010` 文本链接 → 跳到 BR-010 卡;点 `Order` → 跳到 Entity Order 章节

- [ ] **Step 11.10: 记录测试结果**

写一份测试报告:`docs/superpowers/tests/2026-05-XX-bob-model-interactive-review-manual.md`(可选,不强制提交)

---

## Task 12 · 最终 commit + push

**Files:** N/A(所有改动应在前面 Task 中已 commit;本任务汇总验证)

- [ ] **Step 12.1: git status 检查无遗漏**

Run: `git status`
Expected: working tree clean

- [ ] **Step 12.2: git log 验证 commits**

Run: `git log --oneline -8`
Expected: 至少 3 个新 commits:
- `docs(specs): correct ... WebSocket not POST` (Task 1)
- `feat(model): bob-model interactive review — Stages 2-4 rewrite + ...` (Task 9)
- 可能 1 个 follow-up fix commit

- [ ] **Step 12.3: git push**

Run: `git push origin master`
Expected: 成功推送

- [ ] **Step 12.4: 验证 GitHub**

Run: `gh pr list --limit 5` 或浏览器看 commits

- [ ] **Step 12.5: 用户通报**

```
✓ bob-model 交互式 review V1 已落地并推送。

下次任何用 /bob-model 的项目(含 ycb),只要装的是新版 bob-model.md,就会自动进入交互式 review 模式。

ycb 演示文件:/Users/xiaojin/workshop/tmp/ycb/docs/bob/03-model-create-order-20260515.html

V2 留底(本次没做):多人 review / 评论 thread / diff view — 见 spec §8。
```

---

## 风险 & 回滚

| 风险 | 触发 | 回滚 |
|---|---|---|
| 用户机器上 brainstorming server 启不来 | start-server.sh 异常 | Stage 3 降级路径已设计(只读 html 写 docs/bob/);保留旧 skill 行为可走 |
| html 渲染问题(JS bug / CSS 冲突) | 浏览器看不到 widget | 改回旧版只读 html 模板;keep skill but mark §html widget 规范 为 "WIP" |
| Claude apply 错(rename 级联漏) | 测试时见 BR 卡引用没改 | 三段式让用户指出,补改;长期靠覆盖性的 grep 验证(待加入 §不变量) |
| 用户嫌交互模式累 | 反馈说"想要简单只读" | V2 加 `--no-interactive` flag;V1 不提供逃生口(per 设计) |

## V2 留底(本次不做,见 spec §8)

- 多人 review(导出/导入 JSON)
- 评论 thread + resolve
- 评论 owner 字段
- 多 tab BroadcastChannel
- diff view (round N vs N-1)
- 评论历史回放
- `--no-interactive` flag

---

*Plan generated by superpowers:writing-plans · 2026-05-15 · 待 superpowers:subagent-driven-development 或 superpowers:executing-plans 接力*
