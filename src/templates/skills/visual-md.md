---
name: visual-md
description: |
  Use when the user asks “用 visual-md 改这份 md”“vmd 这个文档” or wants an interactive Markdown canvas. Claude Code invokes `/visual-md 参数`;
  Codex invokes `$visual-md 参数` with identical semantics, using a Markdown path for modify mode or a quoted prompt for generate mode.
  This standalone auxiliary skill iteratively edits Markdown through an HTML canvas with block-level widgets and a multi-round WebSocket feedback loop. It ends on `/export`, `导出`, `done`, or `ok 收口`, never modifies the original Markdown, and always emits a new file as its primary output.
---

# visual-md Skill

## Trigger

```
/visual-md <md-path>                 # modify mode
/visual-md "<prompt>"                # generate mode
/visual-md <arg> --out <new-path>    # explicit output path
```

Codex:

```
$visual-md [文档路径]                # modify mode
$visual-md "[prompt]"                # generate mode
$visual-md [参数] --out [new-path]   # explicit output path
```

## 双宿主调用约定

- Claude Code 使用 `/visual-md`；Codex 使用 `$visual-md`，参数语义完全相同。
- 本文保留 slash 形式以保护 Claude Code 兼容性。
- 向用户给出下一步命令时，使用当前宿主的调用形式。
- 不从一个宿主的 skill 根回退到另一个宿主。

## Mode detection

1. Parse args: first non-flag = `<arg>`, optional `--out <path>`
2. If `<arg>` is a path that **exists** AND ends in `.md` or `.markdown` → **MODIFY mode**
3. Otherwise → **GENERATE mode** (treat `<arg>` as prompt)
4. **Edge case**: `<arg>` looks like a path (contains `/` or ends in `.md`) but file doesn't exist →
   HARD ERROR. Reply:
   > Path `<arg>` not found. If you meant a prompt, wrap it in quotes:
   > `/visual-md "your prompt here"`. If you meant a file, check the path.

## Session setup

- **MODIFY mode**: `SESSION_DIR = <dirname-of-source>/.visual-md/<basename>-<date>/`
- **GENERATE mode**: `SESSION_DIR = <cwd>/.visual-md/<slug>-<date>/`
  where `<slug>` = `node <project-root>/.claude/skills/visual-md/scripts/slugify.cjs "<prompt>"`

**First-run bootstrap** (idempotent — costs ~10s once per project, no-op after):
```bash
SCRIPTS_DIR="<project-root>/.claude/skills/visual-md/scripts"
if [ ! -d "$SCRIPTS_DIR/node_modules/markdown-it" ]; then
  (cd "$SCRIPTS_DIR" && npm install --no-audit --no-fund)
fi
```
If `npm` is missing, abort with a three-segment Q&A telling the user to install Node.js ≥20.

Start server (background):
```bash
<project-root>/.claude/skills/visual-md/scripts/start-server.sh "$SESSION_DIR"
```
Capture the printed `server-info` JSON; remember `url`, `port`, `screen_dir`, `state_dir`.

## Initial draft

- **MODIFY mode**: `current_draft = fs.read(<source>)` (keep source file untouched)
- **GENERATE mode**: Claude composes a skeleton md from `<prompt>` (headings + placeholder
  paragraphs + necessary list/table scaffolds if prompt implies them)

## Round 1: render

```bash
node <project-root>/.claude/skills/visual-md/scripts/md2html.cjs <draft-tmp.md> $SESSION_DIR/<basename>-<date>-v1.html
```

Report (mandatory format, every round):
> 已渲染 round N。
>
> **产物**(浏览器打开):
> - url:`http://localhost:<port>`
> - html:`<absolute-path>/<basename>-<date>-vN.html`

## Round loop

1. **Wait for user input**. User options:
   - Fill widgets in browser, click "Submit Round (N)" → payload appended to
     `$SESSION_DIR/state/events`. After submit, widgets clear so the user can
     immediately stage MORE changes for the same round if they want.
   - Type in terminal `继续` / `go` / `process` / `apply` → trigger Step 2 (read + report + apply)
   - Type in terminal an export trigger (`/export`, `导出`, `done`, `ok 收口`) → go to "Export"
   - Type in terminal any other prompt → treat as an implicit doc-scope prompt, append to
     events, then proceed as if `继续` was typed

2. **Read `$SESSION_DIR/state/events`** (do NOT busy-loop; the user's next turn IS the signal).
   The server resets this file each time a NEW canvas html is added, so the file accumulates
   ONLY items submitted since the last canvas push. Parse each line as JSON; for
   `type: 'submit-round'`, flatten all `items` arrays into one list.

3. **Report what was received** BEFORE applying. Mandatory format:

   > 收到 round N 的 K 个修改点(events 文件 J 行,K 个 items):
   >
   > 1. `<scope>` · `<kind>` · `<target_id>` (`<locator>`):「<prompt>」
   > 2. ...
   >
   > 计数校验:K == 浏览器最后显示的 "Submit Round (M)" 中的 M? 若不一致,
   >   提示用户(可能 race-condition,或多批次累计)。
   >
   > 应用中...

   **Always** print this report so the user can see exactly what was captured.
   If `K` differs from what the user expected, surface it as a three-segment Q&A.

4. **Apply changes** to `current_draft`:
   - Apply in scope order: `doc` → `heading` → `block` → `sub-block`
   - For each item, locate the target (by `target_id` + `kind`), apply the prompt as a text edit
   - If two prompts conflict (e.g. doc says "make formal" + paragraph says "make funnier"),
     ask via **three-segment Q&A** before proceeding

5. **Render** the new draft to `$SESSION_DIR/<basename>-<date>-v(N+1).html`. The server's
   file-watcher will push `reload` to the browser automatically, AND will reset the events
   file so the next round starts clean.

6. **Report** with the mandatory product paths block (url + html absolute path).

7. Loop back to 1.

## Three-segment Q&A (mandatory for any user decision)

> **Q: [question]**
>
> **推测**: <Claude's guess>
> **理由**: <one-line reason>
> **推荐选择**: `<one concrete option>`
>
> 是否同意?(回"是"走推荐;回"否,理由是 ..." 重判;回"否,我选 X" 切到 X)

Use for: conflict resolution between scope prompts, export confirmation, mode-detection
edge cases, naming choices.

## Export

When user types `/export`, `导出`, `done`, or `ok 收口`:

1. **Confirm via three-segment Q&A**:
   > **Q: 收口导出?**
   > **推测**: 当前 round N 是最终版,导出到 `<out-path>`
   > **理由**: 用户已触发 `<trigger-word>`
   > **推荐选择**: 是,导出并关闭 server
   > 是否同意?

2. On confirmation:
   - Write `current_draft` to the output path:
     - MODIFY mode default: `<source-dir>/<basename>-revised-<date>.md`
     - GENERATE mode default: `<cwd>/visual-md-<slug>-<date>.md`
     - `--out <path>` overrides
   - Stop the server: `<project-root>/.claude/skills/visual-md/scripts/stop-server.sh "$SESSION_DIR"`
   - Final report:
     > 已导出。
     >
     > **产物**:
     > - md(新):`<absolute-path-to-new-md>`
     > - 源 md(未改):`<absolute-path-to-source>`(仅 MODIFY 模式)
     > - 中间 html 全版本:`<session-dir>/`(可保留也可手动清理)

3. **NEVER auto-export**. Only the user pulls the trigger.

## Reporting rules (per DESIGN.md §6)

Every round MUST print absolute paths in a list/table:
- url(browser)
- html(absolute)
- After export: md(new, absolute) + source(absolute, MODIFY only)

NO bury-in-prose, NO relative paths, every round both `url` + `html`.
