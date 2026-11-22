#!/usr/bin/env node
// Renderiza blocos mermaid em SVG (offline depois do 1º download). Entrada: JSON [{id, code}] em stdin. Saída: JSON {id: svg}.
// Usa o Chromium via CDP. mermaid@11.4.1 fica em tools/.cache (não versionado).
import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync, existsSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
const here = resolve(fileURLToPath(import.meta.url), '..');
const lib = join(here, '.cache', 'mermaid.min.js');
if (!existsSync(lib)) { console.error('falta tools/.cache/mermaid.min.js (curl -o ... https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.min.js)'); process.exit(2); }
const input = JSON.parse(readFileSync(0, 'utf8'));
const dir = mkdtempSync(join(tmpdir(), 'kzm-'));
const page = join(dir, 'm.html');
writeFileSync(page, `<!doctype html><meta charset=utf-8><body><div id=o></div><script src="file://${lib}"></script>`);
const port = 9800 + Math.floor(Math.random() * 400);
const chrome = spawn('chromium', ['--headless=new', '--no-sandbox', '--disable-gpu', '--allow-file-access-from-files', `--remote-debugging-port=${port}`, `--user-data-dir=${dir}/p`, 'about:blank'], { stdio: 'ignore' });
const sleep = ms => new Promise(r => setTimeout(r, ms));
let wsUrl; for (let i = 0; i < 60 && !wsUrl; i++) { try { const l = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json(); wsUrl = l.find(t => t.type === 'page')?.webSocketDebuggerUrl; } catch {} if (!wsUrl) await sleep(150); }
const sock = new WebSocket(wsUrl); await new Promise(r => sock.addEventListener('open', r));
let id = 0; const pend = new Map();
sock.addEventListener('message', e => { const m = JSON.parse(e.data); if (m.id && pend.has(m.id)) { pend.get(m.id)(m); pend.delete(m.id); } });
const send = (method, params = {}) => new Promise(r => { const i = ++id; pend.set(i, r); sock.send(JSON.stringify({ id: i, method, params })); });
await send('Page.enable'); await send('Page.navigate', { url: 'file://' + page }); await sleep(1200);
const theme = { theme: 'base', securityLevel: 'loose', flowchart: { htmlLabels: true, curve: 'basis', useMaxWidth: true, padding: 14, nodeSpacing: 36, rankSpacing: 44 },
  themeVariables: { fontFamily: 'Hanken Grotesk, system-ui, sans-serif', fontSize: '15px', primaryColor: '#fff1e6', primaryBorderColor: '#c2570c', primaryTextColor: '#2a1608', lineColor: '#5c4a3d', secondaryColor: '#e8f3f0', tertiaryColor: '#f6f2ec', edgeLabelBackground: '#faf6f1', clusterBkg: '#f6f2ec', nodeBorder: '#c2570c' } };
const expr = `(async()=>{mermaid.initialize(${JSON.stringify(theme)});const items=${JSON.stringify(input)};const out={};for(const it of items){try{const r=await mermaid.render('g'+it.id,it.code);out[it.id]=r.svg;}catch(e){out[it.id]='ERRO '+e.message;}}return JSON.stringify(out);})()`;
const r = await send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true });
import('node:fs').then(fs => { fs.writeFileSync(process.argv[2], r.result.result.value || JSON.stringify({ erro: JSON.stringify(r.result) })); sock.close(); chrome.kill('SIGKILL'); process.exit(0); });
