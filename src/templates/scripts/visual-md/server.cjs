// Self-contained HTTP + WebSocket server for visual-md.
// WebSocket RFC 6455 implementation adapted from bob-model/server.cjs (read-only reference).
const crypto = require('crypto');
const http = require('http');
const fs = require('fs');
const path = require('path');

const OPCODES = { TEXT: 0x01, CLOSE: 0x08, PING: 0x09, PONG: 0x0A };
const WS_MAGIC = '258EAFA5-E914-47DA-95CA-C5AB0DC85B11';

function computeAcceptKey(clientKey) {
  return crypto.createHash('sha1').update(clientKey + WS_MAGIC).digest('base64');
}
function encodeFrame(opcode, payload) {
  const fin = 0x80;
  const len = payload.length;
  let header;
  if (len < 126) { header = Buffer.alloc(2); header[0] = fin | opcode; header[1] = len; }
  else if (len < 65536) { header = Buffer.alloc(4); header[0] = fin | opcode; header[1] = 126; header.writeUInt16BE(len, 2); }
  else { header = Buffer.alloc(10); header[0] = fin | opcode; header[1] = 127; header.writeBigUInt64BE(BigInt(len), 2); }
  return Buffer.concat([header, payload]);
}
function decodeFrame(buffer) {
  if (buffer.length < 2) return null;
  const secondByte = buffer[1];
  const opcode = buffer[0] & 0x0F;
  const masked = (secondByte & 0x80) !== 0;
  let payloadLen = secondByte & 0x7F;
  let offset = 2;
  if (!masked) throw new Error('Client frames must be masked');
  if (payloadLen === 126) { if (buffer.length < 4) return null; payloadLen = buffer.readUInt16BE(2); offset = 4; }
  else if (payloadLen === 127) { if (buffer.length < 10) return null; payloadLen = Number(buffer.readBigUInt64BE(2)); offset = 10; }
  const maskOffset = offset; const dataOffset = offset + 4;
  const totalLen = dataOffset + payloadLen;
  if (buffer.length < totalLen) return null;
  const mask = buffer.slice(maskOffset, dataOffset);
  const data = Buffer.alloc(payloadLen);
  for (let i = 0; i < payloadLen; i++) data[i] = buffer[dataOffset + i] ^ mask[i % 4];
  return { opcode, payload: data, bytesConsumed: totalLen };
}

const PORT = Number(process.env.VISUAL_MD_PORT) || (49152 + Math.floor(Math.random() * 16383));
const HOST = process.env.VISUAL_MD_HOST || '127.0.0.1';
const URL_HOST = process.env.VISUAL_MD_URL_HOST || (HOST === '127.0.0.1' ? 'localhost' : HOST);
const SESSION_DIR = process.env.VISUAL_MD_DIR || path.join(process.cwd(), '.visual-md');
const CONTENT_DIR = path.join(SESSION_DIR);
const STATE_DIR = path.join(SESSION_DIR, 'state');
let ownerPid = process.env.VISUAL_MD_OWNER_PID ? Number(process.env.VISUAL_MD_OWNER_PID) : null;
const IDLE_TIMEOUT_MS = 30 * 60 * 1000;

const WAITING_PAGE = `<!DOCTYPE html><html><body><h1>visual-md</h1>
<p>Waiting for the canvas to render...</p></body></html>`;

const frameTemplate = fs.readFileSync(path.join(__dirname, 'frame-template.html'), 'utf-8');
const clientScript = fs.readFileSync(path.join(__dirname, 'client.js'), 'utf-8');
const clientInjection = '<script>\n' + clientScript + '\n</script>';

function getNewestHtml() {
  if (!fs.existsSync(CONTENT_DIR)) return null;
  const files = fs.readdirSync(CONTENT_DIR)
    .filter(f => f.endsWith('.html'))
    .map(f => ({ path: path.join(CONTENT_DIR, f), mtime: fs.statSync(path.join(CONTENT_DIR, f)).mtime.getTime() }))
    .sort((a, b) => b.mtime - a.mtime);
  return files.length > 0 ? files[0].path : null;
}
function wrapInFrame(bodyHtml) {
  return frameTemplate.replace('<!-- CONTENT -->', bodyHtml);
}

let lastActivity = Date.now();
const touch = () => { lastActivity = Date.now(); };

function handleRequest(req, res) {
  touch();
  if (req.method === 'GET' && req.url === '/') {
    const newest = getNewestHtml();
    let html;
    if (!newest) html = WAITING_PAGE;
    else {
      const raw = fs.readFileSync(newest, 'utf-8');
      html = raw.trimStart().toLowerCase().startsWith('<!doctype') || raw.trimStart().toLowerCase().startsWith('<html')
        ? raw : wrapInFrame(raw);
    }
    if (html.includes('</body>')) html = html.replace('</body>', clientInjection + '\n</body>');
    else html += clientInjection;
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    res.end(html);
  } else {
    res.writeHead(404); res.end('Not found');
  }
}

const clients = new Set();
function handleUpgrade(req, socket) {
  const key = req.headers['sec-websocket-key'];
  if (!key) { socket.destroy(); return; }
  const accept = computeAcceptKey(key);
  socket.write(
    'HTTP/1.1 101 Switching Protocols\r\n' +
    'Upgrade: websocket\r\n' +
    'Connection: Upgrade\r\n' +
    'Sec-WebSocket-Accept: ' + accept + '\r\n\r\n'
  );
  let buffer = Buffer.alloc(0);
  clients.add(socket);
  socket.on('data', (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    while (buffer.length > 0) {
      let r;
      try { r = decodeFrame(buffer); }
      catch (e) { socket.end(encodeFrame(OPCODES.CLOSE, Buffer.alloc(0))); clients.delete(socket); return; }
      if (!r) break;
      buffer = buffer.slice(r.bytesConsumed);
      switch (r.opcode) {
        case OPCODES.TEXT: handleMessage(r.payload.toString()); break;
        case OPCODES.CLOSE: socket.end(encodeFrame(OPCODES.CLOSE, Buffer.alloc(0))); clients.delete(socket); return;
        case OPCODES.PING: socket.write(encodeFrame(OPCODES.PONG, r.payload)); break;
        case OPCODES.PONG: break;
        default: socket.end(encodeFrame(OPCODES.CLOSE, Buffer.alloc(0))); clients.delete(socket); return;
      }
    }
  });
  socket.on('close', () => clients.delete(socket));
  socket.on('error', () => clients.delete(socket));
}

function handleMessage(text) {
  let event;
  try { event = JSON.parse(text); } catch (e) { return; }
  touch();
  console.log(JSON.stringify({ source: 'user-event', ...event }));
  if (event.type === 'submit-round') {
    const eventsFile = path.join(STATE_DIR, 'events');
    fs.appendFileSync(eventsFile, JSON.stringify(event) + '\n');
  }
}
function broadcast(msg) {
  const frame = encodeFrame(OPCODES.TEXT, Buffer.from(JSON.stringify(msg)));
  for (const s of clients) { try { s.write(frame); } catch (e) { clients.delete(s); } }
}

function startServer() {
  if (!fs.existsSync(CONTENT_DIR)) fs.mkdirSync(CONTENT_DIR, { recursive: true });
  if (!fs.existsSync(STATE_DIR)) fs.mkdirSync(STATE_DIR, { recursive: true });

  const knownFiles = new Set(fs.readdirSync(CONTENT_DIR).filter(f => f.endsWith('.html')));
  const server = http.createServer(handleRequest);
  server.on('upgrade', handleUpgrade);

  const debounce = new Map();
  const watcher = fs.watch(CONTENT_DIR, (eventType, filename) => {
    if (!filename || !filename.endsWith('.html')) return;
    if (debounce.has(filename)) clearTimeout(debounce.get(filename));
    debounce.set(filename, setTimeout(() => {
      debounce.delete(filename);
      const fp = path.join(CONTENT_DIR, filename);
      if (!fs.existsSync(fp)) return;
      touch();
      if (!knownFiles.has(filename)) {
        knownFiles.add(filename);
        // Reset events file: each new screen = new round, prior events already consumed
        const eventsFile = path.join(STATE_DIR, 'events');
        if (fs.existsSync(eventsFile)) fs.unlinkSync(eventsFile);
        console.log(JSON.stringify({ type: 'screen-added', file: fp }));
      } else {
        console.log(JSON.stringify({ type: 'screen-updated', file: fp }));
      }
      broadcast({ type: 'reload' });
    }, 100));
  });

  function shutdown(reason) {
    console.log(JSON.stringify({ type: 'server-stopped', reason }));
    const infoFile = path.join(STATE_DIR, 'server-info');
    if (fs.existsSync(infoFile)) fs.unlinkSync(infoFile);
    fs.writeFileSync(path.join(STATE_DIR, 'server-stopped'),
      JSON.stringify({ reason, timestamp: Date.now() }) + '\n');
    watcher.close();
    clearInterval(lifecycle);
    server.close(() => process.exit(0));
  }
  function ownerAlive() {
    if (!ownerPid) return true;
    try { process.kill(ownerPid, 0); return true; } catch (e) { return e.code === 'EPERM'; }
  }
  const lifecycle = setInterval(() => {
    if (!ownerAlive()) shutdown('owner process exited');
    else if (Date.now() - lastActivity > IDLE_TIMEOUT_MS) shutdown('idle timeout');
  }, 60 * 1000);
  lifecycle.unref();

  server.listen(PORT, HOST, () => {
    const info = JSON.stringify({
      type: 'server-started', port: PORT, host: HOST, url_host: URL_HOST,
      url: 'http://' + URL_HOST + ':' + PORT,
      screen_dir: CONTENT_DIR, state_dir: STATE_DIR
    });
    console.log(info);
    fs.writeFileSync(path.join(STATE_DIR, 'server-info'), info + '\n');
  });
}

if (require.main === module) startServer();
module.exports = { computeAcceptKey, encodeFrame, decodeFrame, OPCODES };
