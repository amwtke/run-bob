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

  // ===== Persistent widget: inject textarea on first render =====
  document.querySelectorAll('div[data-vmd-widget]').forEach(w => {
    const ta = document.createElement('textarea');
    ta.placeholder = 'prompt for this scope (leave empty to skip)';
    ta.addEventListener('input', () => {
      if (ta.value.trim()) w.classList.add('vmd-filled');
      else w.classList.remove('vmd-filled');
      updatePendingCount();
    });
    w.appendChild(ta);
  });

  // ===== Floating widget (cell/item): popover on click =====
  let currentPopover = null;
  // Map of target_id -> stored prompt (so popover re-open shows previous value)
  const subBlockPrompts = new Map();

  document.querySelectorAll('button.vmd-cell-widget, button.vmd-item-widget').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      openPopover(btn);
    });
  });

  function openPopover(btn) {
    if (currentPopover) currentPopover.remove();
    const target = btn.dataset.vmdTarget;
    const locator = btn.dataset.vmdLocator || '';
    const existing = subBlockPrompts.get(target) || '';

    const pop = document.createElement('div');
    pop.className = 'vmd-popover';
    pop.innerHTML = `
      <div class="vmd-popover-locator">${locator}</div>
      <textarea>${escapeHtml(existing)}</textarea>
      <div class="vmd-popover-actions">
        <button data-act="cancel">Cancel</button>
        <button data-act="clear">Clear</button>
        <button class="vmd-primary" data-act="save">Save</button>
      </div>
    `;
    document.body.appendChild(pop);

    const rect = btn.getBoundingClientRect();
    pop.style.left = Math.min(rect.left, window.innerWidth - 340) + 'px';
    pop.style.top = (rect.bottom + 4) + 'px';

    const ta = pop.querySelector('textarea');
    ta.focus();

    pop.addEventListener('click', (e) => {
      const act = e.target.dataset?.act;
      if (act === 'save') {
        const v = ta.value.trim();
        if (v) { subBlockPrompts.set(target, v); btn.classList.add('vmd-filled'); }
        else { subBlockPrompts.delete(target); btn.classList.remove('vmd-filled'); }
        closePopover();
        updatePendingCount();
      } else if (act === 'clear') {
        subBlockPrompts.delete(target);
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
    document.querySelectorAll('div[data-vmd-widget]').forEach(w => {
      const ta = w.querySelector('textarea');
      const v = ta && ta.value.trim();
      if (!v) return;
      items.push({
        scope: w.dataset.vmdScope,
        kind: w.dataset.vmdKind,
        target_id: w.dataset.vmdTarget,
        locator: w.dataset.vmdLocator || null,
        prompt: v
      });
    });
    subBlockPrompts.forEach((prompt, target_id) => {
      const btn = document.querySelector(`button[data-vmd-target="${target_id}"]`);
      items.push({
        scope: 'sub-block',
        kind: btn?.dataset.vmdKind || 'cell',
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
    document.querySelectorAll('div[data-vmd-widget]').forEach(w => {
      const ta = w.querySelector('textarea');
      if (ta) ta.value = '';
      w.classList.remove('vmd-filled');
    });
    document.querySelectorAll('button.vmd-cell-widget.vmd-filled, button.vmd-item-widget.vmd-filled')
      .forEach(b => b.classList.remove('vmd-filled'));
    subBlockPrompts.clear();
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
