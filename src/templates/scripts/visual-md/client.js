(function() {
  const WS_URL = 'ws://' + window.location.host;
  let ws = null;
  let eventQueue = [];

  function connect() {
    ws = new WebSocket(WS_URL);
    ws.onopen = () => {
      document.getElementById('vmd-status').textContent = 'connected';
      eventQueue.forEach(e => ws.send(JSON.stringify(e)));
      eventQueue = [];
    };
    ws.onmessage = (msg) => {
      try {
        const data = JSON.parse(msg.data);
        if (data.type === 'reload') window.location.reload();
      } catch (e) { /* ignore */ }
    };
    ws.onclose = () => {
      document.getElementById('vmd-status').textContent = 'reconnecting...';
      setTimeout(connect, 1000);
    };
    ws.onerror = () => { /* onclose handles reconnect */ };
  }
  function sendEvent(event) {
    event.timestamp = Date.now();
    const json = JSON.stringify(event);
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(json);
    else eventQueue.push(event);
  }

  // ===== Build left TOC from headings =====
  function buildToc() {
    const list = document.getElementById('vmd-toc-list');
    const scroller = document.getElementById('vmd-content');
    if (!list || !scroller) return;
    const headings = scroller.querySelectorAll(
      'article[data-vmd-doc] h1, article[data-vmd-doc] h2, article[data-vmd-doc] h3, ' +
      'article[data-vmd-doc] h4, article[data-vmd-doc] h5, article[data-vmd-doc] h6'
    );
    if (headings.length === 0) {
      list.innerHTML = '<li class="vmd-toc-empty">(无标题)</li>';
      return;
    }
    const links = [];
    headings.forEach((h, i) => {
      // Give each heading a stable id for scroll/anchor purposes
      if (!h.id) h.id = h.dataset.vmdId || `vmd-h-${i}`;
      const li = document.createElement('li');
      const a = document.createElement('a');
      a.href = '#' + h.id;
      a.className = 'vmd-toc-' + h.tagName.toLowerCase();
      a.textContent = h.textContent.trim();
      a.title = a.textContent;
      a.addEventListener('click', (e) => {
        e.preventDefault();
        h.scrollIntoView({ behavior: 'smooth', block: 'start' });
        setActive(a);
      });
      li.appendChild(a);
      list.appendChild(li);
      links.push({ a, h });
    });

    function setActive(activeA) {
      links.forEach(({ a }) => a.classList.toggle('vmd-toc-active', a === activeA));
    }

    // Highlight TOC entry for the heading currently nearest the top of viewport.
    let ticking = false;
    scroller.addEventListener('scroll', () => {
      if (ticking) return;
      ticking = true;
      requestAnimationFrame(() => {
        const top = scroller.getBoundingClientRect().top;
        let current = links[0];
        for (const item of links) {
          const r = item.h.getBoundingClientRect();
          if (r.top - top <= 80) current = item;
          else break;
        }
        if (current) setActive(current.a);
        ticking = false;
      });
    });
  }
  buildToc();

  // ===== Unified popover handling for all widget scopes =====
  let currentPopover = null;
  // Map of "scope::target_id" -> stored prompt (so popover re-open shows previous value)
  const widgetPrompts = new Map();
  const keyFor = (btn) => `${btn.dataset.vmdScope}::${btn.dataset.vmdTarget}`;

  document.querySelectorAll(
    'button.vmd-block-widget, button.vmd-cell-widget, button.vmd-item-widget'
  ).forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      openPopover(btn);
    });
  });

  function openPopover(btn) {
    if (currentPopover) currentPopover.remove();
    const key = keyFor(btn);
    const locator = btn.dataset.vmdLocator || '';
    const existing = widgetPrompts.get(key) || '';

    const pop = document.createElement('div');
    pop.className = 'vmd-popover';
    pop.innerHTML = `
      <div class="vmd-popover-locator">${locator}</div>
      <textarea placeholder="prompt for this scope (leave empty to skip)">${escapeHtml(existing)}</textarea>
      <div class="vmd-popover-actions">
        <button data-act="cancel">Cancel</button>
        <button data-act="clear">Clear</button>
        <button class="vmd-primary" data-act="save">Save</button>
      </div>
    `;
    document.body.appendChild(pop);

    const rect = btn.getBoundingClientRect();
    pop.style.left = Math.min(Math.max(rect.left, 8), window.innerWidth - 340) + 'px';
    pop.style.top = (rect.bottom + 4) + 'px';

    const ta = pop.querySelector('textarea');
    ta.focus();
    ta.setSelectionRange(ta.value.length, ta.value.length);

    pop.addEventListener('click', (e) => {
      const act = e.target.dataset?.act;
      if (act === 'save') {
        const v = ta.value.trim();
        if (v) { widgetPrompts.set(key, v); btn.classList.add('vmd-filled'); }
        else { widgetPrompts.delete(key); btn.classList.remove('vmd-filled'); }
        closePopover();
        updatePendingCount();
      } else if (act === 'clear') {
        widgetPrompts.delete(key);
        btn.classList.remove('vmd-filled');
        closePopover();
        updatePendingCount();
      } else if (act === 'cancel') {
        closePopover();
      }
    });

    currentPopover = pop;
    setTimeout(() => {
      document.addEventListener('click', dismissOnOutside);
    }, 0);
  }
  function closePopover() {
    if (currentPopover) { currentPopover.remove(); currentPopover = null; }
    document.removeEventListener('click', dismissOnOutside);
  }
  function dismissOnOutside(e) {
    if (currentPopover && !currentPopover.contains(e.target)) closePopover();
  }
  function escapeHtml(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  // ===== Submit Round =====
  function collectPayload() {
    const items = [];
    widgetPrompts.forEach((prompt, key) => {
      const sepIdx = key.indexOf('::');
      const scope = key.slice(0, sepIdx);
      const target_id = key.slice(sepIdx + 2);
      const btn = document.querySelector(
        `button[data-vmd-scope="${scope}"][data-vmd-target="${target_id}"]`
      );
      items.push({
        scope,
        kind: btn?.dataset.vmdKind || null,
        target_id,
        locator: btn?.dataset.vmdLocator || null,
        prompt
      });
    });
    return items;
  }
  function updatePendingCount() {
    const count = collectPayload().length;
    const btn = document.getElementById('vmd-submit-btn');
    btn.textContent = `Submit Round (${count})`;
    btn.disabled = count === 0;
    document.getElementById('vmd-pending-count').textContent =
      count === 0 ? 'no changes' : `${count} change${count === 1 ? '' : 's'} queued`;
  }
  document.getElementById('vmd-submit-btn').addEventListener('click', () => {
    const items = collectPayload();
    if (items.length === 0) return;
    sendEvent({ type: 'submit-round', items });
    // Clear all widget state so the user can immediately stage more changes
    // for the SAME round (events file accumulates) or just be ready when the
    // server's reload arrives. Per spec: widgets are never locked.
    document.querySelectorAll('button[data-vmd-widget].vmd-filled')
      .forEach(b => b.classList.remove('vmd-filled'));
    widgetPrompts.clear();
    updatePendingCount();
    // Brief status flash so the user knows the submit landed
    const status = document.getElementById('vmd-status');
    const prev = status.textContent;
    status.textContent = `submitted ${items.length} → say 继续 in CC`;
    setTimeout(() => { if (status.textContent.startsWith('submitted')) status.textContent = prev; }, 4000);
  });

  updatePendingCount();
  connect();
})();
