const fs = require('fs');
const path = require('path');
const MarkdownIt = require('markdown-it');

function renderToHtml(md) {
  const mdParser = new MarkdownIt({ html: false, breaks: false, linkify: true });
  const tokens = mdParser.parse(md, {});

  const counters = { h: 0, p: 0, code: 0, quote: 0, table: 0, list: 0, img: 0 };
  let widgetCounter = 0;
  const widget = (scope, kind, target, locator) => {
    widgetCounter++;
    const loc = locator ? ` data-vmd-locator="${escapeAttr(locator)}"` : '';
    return `<div data-vmd-widget="${widgetCounter}" data-vmd-scope="${scope}" data-vmd-kind="${kind}" data-vmd-target="${target}"${loc}></div>`;
  };

  const renderer = mdParser.renderer;
  const defaultRender = (name) => renderer.rules[name] || ((tokens, idx, opts, env, self) =>
    self.renderToken(tokens, idx, opts));

  // Heading: set id, inject widget AFTER closing tag
  const baseHeadingOpen = defaultRender('heading_open');
  const baseHeadingClose = defaultRender('heading_close');
  renderer.rules.heading_open = (toks, idx, opts, env, self) => {
    const id = `h-${counters.h++}`;
    toks[idx].attrSet('data-vmd-id', id);
    toks[idx].attrSet('data-vmd-kind', 'heading');
    return baseHeadingOpen(toks, idx, opts, env, self);
  };
  renderer.rules.heading_close = (toks, idx, opts, env, self) => {
    const id = `h-${counters.h - 1}`;
    const tag = toks[idx].tag;
    return baseHeadingClose(toks, idx, opts, env, self)
      + widget('heading', tag, id);
  };

  // Paragraph: detect image-only paragraph and tag accordingly
  const baseParaOpen = defaultRender('paragraph_open');
  const baseParaClose = defaultRender('paragraph_close');
  renderer.rules.paragraph_open = (toks, idx, opts, env, self) => {
    const isImageOnly = toks[idx + 1]?.type === 'inline'
      && toks[idx + 1].children?.length === 1
      && toks[idx + 1].children[0].type === 'image';
    if (isImageOnly) {
      const id = `img-${counters.img++}`;
      toks[idx].attrSet('data-vmd-id', id);
      toks[idx].attrSet('data-vmd-kind', 'image');
      toks[idx].__vmdKind = 'image';
      toks[idx].__vmdId = id;
    } else {
      const id = `p-${counters.p++}`;
      toks[idx].attrSet('data-vmd-id', id);
      toks[idx].attrSet('data-vmd-kind', 'paragraph');
      toks[idx].__vmdKind = 'paragraph';
      toks[idx].__vmdId = id;
    }
    return baseParaOpen(toks, idx, opts, env, self);
  };
  renderer.rules.paragraph_close = (toks, idx, opts, env, self) => {
    let openIdx = idx;
    while (openIdx >= 0 && toks[openIdx].type !== 'paragraph_open') openIdx--;
    const open = toks[openIdx];
    return baseParaClose(toks, idx, opts, env, self)
      + widget('block', open.__vmdKind, open.__vmdId);
  };

  // Code block (fenced) — markdown-it renders <pre> manually, so inject attrs via string replace
  const baseFence = defaultRender('fence');
  renderer.rules.fence = (toks, idx, opts, env, self) => {
    const id = `code-${counters.code++}`;
    const raw = baseFence(toks, idx, opts, env, self);
    // Inject data-vmd-id and data-vmd-kind into the opening <pre> tag
    const tagged = raw.replace(/^<pre/, `<pre data-vmd-id="${id}" data-vmd-kind="code"`);
    return tagged + widget('block', 'code', id);
  };

  // Indented code block — same issue, attrs go on <code> not <pre>
  const baseCodeBlock = defaultRender('code_block');
  renderer.rules.code_block = (toks, idx, opts, env, self) => {
    const id = `code-${counters.code++}`;
    const raw = baseCodeBlock(toks, idx, opts, env, self);
    const tagged = raw.replace(/^<pre/, `<pre data-vmd-id="${id}" data-vmd-kind="code"`);
    return tagged + widget('block', 'code', id);
  };

  // Blockquote
  const baseQuoteOpen = defaultRender('blockquote_open');
  const baseQuoteClose = defaultRender('blockquote_close');
  renderer.rules.blockquote_open = (toks, idx, opts, env, self) => {
    const id = `quote-${counters.quote++}`;
    toks[idx].attrSet('data-vmd-id', id);
    toks[idx].attrSet('data-vmd-kind', 'blockquote');
    toks[idx].__vmdId = id;
    return baseQuoteOpen(toks, idx, opts, env, self);
  };
  renderer.rules.blockquote_close = (toks, idx, opts, env, self) => {
    let openIdx = idx;
    while (openIdx >= 0 && toks[openIdx].type !== 'blockquote_open') openIdx--;
    return baseQuoteClose(toks, idx, opts, env, self)
      + widget('block', 'blockquote', toks[openIdx].__vmdId);
  };

  // Table — track current table id, row index, col index
  let tableId = null;
  let rowIdx = -1;
  let colIdx = -1;
  let inHeader = false;
  let currentCellTextBuf = null;

  const baseTableOpen = defaultRender('table_open');
  const baseTableClose = defaultRender('table_close');
  const baseTheadOpen = defaultRender('thead_open');
  const baseTbodyOpen = defaultRender('tbody_open');
  const baseTrOpen = defaultRender('tr_open');
  const baseThOpen = defaultRender('th_open');
  const baseThClose = defaultRender('th_close');
  const baseTdOpen = defaultRender('td_open');
  const baseTdClose = defaultRender('td_close');

  renderer.rules.table_open = (toks, idx, opts, env, self) => {
    tableId = `table-${counters.table++}`;
    rowIdx = -1;
    toks[idx].attrSet('data-vmd-id', tableId);
    toks[idx].attrSet('data-vmd-kind', 'table');
    return baseTableOpen(toks, idx, opts, env, self);
  };
  renderer.rules.table_close = (toks, idx, opts, env, self) => {
    const id = tableId;
    tableId = null;
    return baseTableClose(toks, idx, opts, env, self) + widget('block', 'table', id);
  };
  renderer.rules.thead_open = (toks, idx, opts, env, self) => {
    inHeader = true; return baseTheadOpen(toks, idx, opts, env, self);
  };
  renderer.rules.tbody_open = (toks, idx, opts, env, self) => {
    inHeader = false; return baseTbodyOpen(toks, idx, opts, env, self);
  };
  renderer.rules.tr_open = (toks, idx, opts, env, self) => {
    rowIdx++; colIdx = -1;
    return baseTrOpen(toks, idx, opts, env, self);
  };

  function openCell(baseFn, tag) {
    return (toks, idx, opts, env, self) => {
      colIdx++;
      // Lookahead to extract cell text for locator
      let text = '';
      for (let i = idx + 1; i < toks.length && toks[i].type !== `${tag}_close`; i++) {
        if (toks[i].type === 'inline') text += toks[i].content;
      }
      currentCellTextBuf = text.trim().slice(0, 40);
      toks[idx].attrSet('data-vmd-cell', `${tableId}/r-${rowIdx}/c-${colIdx}`);
      return baseFn(toks, idx, opts, env, self);
    };
  }
  function closeCell(baseFn) {
    return (toks, idx, opts, env, self) => {
      const target = `${tableId}/r-${rowIdx}/c-${colIdx}`;
      const locator = `Row ${rowIdx} · Col ${colIdx} · &quot;${escapeAttr(currentCellTextBuf || '')}&quot;`;
      const btn = `<button class="vmd-cell-widget" data-vmd-widget data-vmd-scope="sub-block" data-vmd-kind="cell" data-vmd-target="${target}" data-vmd-locator="${locator}">💬</button>`;
      return btn + baseFn(toks, idx, opts, env, self);
    };
  }
  renderer.rules.th_open = openCell(baseThOpen, 'th');
  renderer.rules.th_close = closeCell(baseThClose);
  renderer.rules.td_open = openCell(baseTdOpen, 'td');
  renderer.rules.td_close = closeCell(baseTdClose);

  // Lists (ul/ol) — stack to handle nesting
  const listStack = [];

  const baseUlOpen = defaultRender('bullet_list_open');
  const baseUlClose = defaultRender('bullet_list_close');
  const baseOlOpen = defaultRender('ordered_list_open');
  const baseOlClose = defaultRender('ordered_list_close');
  const baseLiOpen = defaultRender('list_item_open');
  const baseLiClose = defaultRender('list_item_close');

  function openList(baseFn) {
    return (toks, idx, opts, env, self) => {
      const id = `list-${counters.list++}`;
      toks[idx].attrSet('data-vmd-id', id);
      toks[idx].attrSet('data-vmd-kind', 'list');
      listStack.push({ id, itemIdx: -1 });
      return baseFn(toks, idx, opts, env, self);
    };
  }
  function closeList(baseFn) {
    return (toks, idx, opts, env, self) => {
      const ctx = listStack.pop();
      return baseFn(toks, idx, opts, env, self) + widget('block', 'list', ctx.id);
    };
  }
  renderer.rules.bullet_list_open = openList(baseUlOpen);
  renderer.rules.bullet_list_close = closeList(baseUlClose);
  renderer.rules.ordered_list_open = openList(baseOlOpen);
  renderer.rules.ordered_list_close = closeList(baseOlClose);

  renderer.rules.list_item_open = (toks, idx, opts, env, self) => {
    const ctx = listStack[listStack.length - 1];
    ctx.itemIdx++;
    const target = `${ctx.id}/i-${ctx.itemIdx}`;
    // Lookahead first inline content for locator
    let text = '';
    for (let i = idx + 1; i < toks.length && toks[i].type !== 'list_item_close'; i++) {
      if (toks[i].type === 'inline') { text = toks[i].content; break; }
    }
    const locator = `Item ${ctx.itemIdx} · &quot;${escapeAttr(text.trim().slice(0, 40))}&quot;`;
    const btn = `<button class="vmd-item-widget" data-vmd-widget data-vmd-scope="sub-block" data-vmd-kind="item" data-vmd-target="${target}" data-vmd-locator="${locator}">💬</button>`;
    return baseLiOpen(toks, idx, opts, env, self) + btn;
  };
  renderer.rules.list_item_close = baseLiClose;

  const body = renderer.render(tokens, mdParser.options, {});
  const docWidget = widget('doc', 'doc', 'doc-0');
  return `<article data-vmd-doc>\n${docWidget}\n${body}</article>`;
}

function escapeAttr(s) {
  return String(s).replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;');
}

if (require.main === module) {
  const [inFile, outFile] = process.argv.slice(2);
  if (!inFile || !outFile) {
    console.error('Usage: md2html.cjs <input.md> <output.html>');
    process.exit(1);
  }
  const md = fs.readFileSync(inFile, 'utf-8');
  fs.writeFileSync(outFile, renderToHtml(md));
  process.stdout.write(outFile);
}

module.exports = { renderToHtml };
