#!/usr/bin/env node
// Captura um protótipo com o Chromium (CDP, viewport exato) e, opcionalmente, mede os textos.
// v4: texto que rolou para baixo do cabeçalho de vidro conta como coberto (não é para ser lido ali).
// Uso: node tools/capture.mjs <tela|caminho.html> <largura> <altura> <saida.png> [hash] [--probe] [--dpr=N]
// Com --probe imprime em stdout um JSON com os textos visíveis (cor, caixa) e grava o PNG com os textos escondidos.
import { spawn } from 'node:child_process';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const args = process.argv.slice(2);
const flags = args.filter(a => a.startsWith('--'));
const pos = args.filter(a => !a.startsWith('--'));
const [screen, W, H, out, hash = ''] = pos;
const probe = flags.includes('--probe');
const dpr = Number((flags.find(f => f.startsWith('--dpr=')) || '--dpr=1').slice(6));
const here = resolve(fileURLToPath(import.meta.url), '..', '..');
const url = screen.endsWith('.html') ? 'file://' + resolve(screen) : `file://${here}/prototypes/${screen}.html`;
const port = 20000 + Math.floor(Math.random() * 20000);
const dir = mkdtempSync(join(tmpdir(), 'kz-'));
const chrome = spawn('chromium', ['--headless=new', '--no-sandbox', '--disable-gpu', '--hide-scrollbars', `--remote-debugging-port=${port}`, `--user-data-dir=${dir}`, 'about:blank'], { stdio: 'ignore' });
const sleep = ms => new Promise(r => setTimeout(r, ms));
setTimeout(() => { chrome.kill('SIGKILL'); try { rmSync(dir, { recursive: true, force: true }); } catch {} console.error('timeout'); process.exit(3); }, 45000);
async function ws() {
  for (let i = 0; i < 60; i++) {
    try { const r = await fetch(`http://127.0.0.1:${port}/json/list`); const l = await r.json(); const p = l.find(t => t.type === 'page'); if (p) return p.webSocketDebuggerUrl; } catch {}
    await sleep(150);
  }
  throw new Error('chromium não respondeu');
}
const sock = new WebSocket(await ws());
await new Promise(r => sock.addEventListener('open', r));
let id = 0; const pend = new Map();
sock.addEventListener('message', e => { const m = JSON.parse(e.data); if (m.id && pend.has(m.id)) { pend.get(m.id)(m); pend.delete(m.id); } });
const send = (method, params = {}) => new Promise(r => { const i = ++id; pend.set(i, r); sock.send(JSON.stringify({ id: i, method, params })); });
await send('Emulation.setDeviceMetricsOverride', { width: +W, height: +H, deviceScaleFactor: dpr, mobile: +W < 600 });
await send('Page.enable');
await send('Page.navigate', { url: url + '#' + hash });
await sleep(900);
await send('Runtime.evaluate', { expression: 'document.fonts.ready.then(()=>1)', awaitPromise: true });
await send('Runtime.evaluate', { expression: 'new Promise(r=>{var t=0;(function f(){var i=[...document.images];if(i.every(x=>x.complete)||t++>40)r(1);else setTimeout(f,50)})()})', awaitPromise: true });
await sleep(700);
if (hash.includes('scroll=')) await sleep(300);
let items = null;
if (probe) {
  const js = `(function(){if(document.body.classList.contains('scrolled'))document.querySelectorAll('.topbar').forEach(function(e){e.style.pointerEvents='auto'});var out=[],w=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT),n;
  while((n=w.nextNode())){var t=n.nodeValue.trim();if(!t)continue;var el=n.parentElement;if(!el||el.closest('script,style,[hidden],.sr-only'))continue;
  var cs=getComputedStyle(el);if(cs.visibility==='hidden'||cs.display==='none')continue;
  var r=document.createRange();r.selectNodeContents(n);var b=r.getBoundingClientRect();if(b.width<2||b.height<2)continue;
  if(b.bottom<0||b.top>innerHeight||b.right<0||b.left>innerWidth)continue;
  var pts=[[.5,.5],[.1,.15],[.9,.85],[.1,.85],[.9,.15]],bad=false;
  for(var k=0;k<pts.length;k++){var hit=document.elementFromPoint(Math.min(innerWidth-1,Math.max(0,b.left+b.width*pts[k][0])),Math.min(innerHeight-1,Math.max(0,b.top+b.height*pts[k][1])));
  if(!hit||!(el.contains(hit)||hit.contains(el))){bad=true;break;}}
  if(bad)continue;
  var o=1,p=el;while(p&&p!==document.documentElement){o*=parseFloat(getComputedStyle(p).opacity);p=p.parentElement;}
  out.push({t:t.slice(0,48),c:cs.color,o:o,x:b.left,y:b.top,w:b.width,h:b.height,fs:parseFloat(cs.fontSize),fw:+cs.fontWeight||400});}
  return JSON.stringify({w:innerWidth,h:innerHeight,items:out});})()`;
  const r = await send('Runtime.evaluate', { expression: js, returnByValue: true });
  items = r.result.result.value;
  await send('Runtime.evaluate', { expression: `(function(){var s=document.createElement('style');s.textContent='*{color:transparent!important;-webkit-text-fill-color:transparent!important;text-shadow:none!important} svg,.nowbars{visibility:hidden!important} input::placeholder{color:transparent!important}';document.head.appendChild(s)})()` });
  await sleep(250);
}
const shot = await send('Page.captureScreenshot', { format: 'png' });
writeFileSync(out, Buffer.from(shot.result.data, 'base64'));
if (items) process.stdout.write(items);
sock.close(); chrome.kill('SIGKILL'); await sleep(200); try { rmSync(dir, { recursive: true, force: true }); } catch {}
process.exit(0);
