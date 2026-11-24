/* Kazzu protótipos v3 — interações mínimas (tema, transparência, saturação, capa, estados, folhas, gesto de arrastar para fechar).
   Sem rede, sem telemetria, sem armazenamento além da preferência de tema. */
(function () {
  var root = document.documentElement;
  var immersive = document.body.getAttribute("data-immersive") === "1";
  root.classList.add("kz-init");
  requestAnimationFrame(function () { requestAnimationFrame(function () { root.classList.remove("kz-init"); }); });
  function params() { return new URLSearchParams(location.hash.replace(/^#/, "")); }
  function setTheme(t) { if (immersive) return; if (t === "light" || t === "dark") root.setAttribute("data-theme", t); else root.removeAttribute("data-theme"); }
  function setRT(on) { root.classList.toggle("reduce-transparency", !!on); }
  function cover(id) {
    var d = (window.KAZZU_TRACKS || {})[id];
    document.body.className = document.body.className.replace(/\bcv-\d+\b/, "cv-" + id);
    document.body.setAttribute("data-cover", id);
    var src = "covers/cover-" + id + ".svg";
    document.querySelectorAll("[data-art],[data-stage]").forEach(function (i) { i.src = src; });
    if (!d) return;
    var q = function (s) { return document.querySelector(s); };
    if (q("[data-song]")) q("[data-song]").textContent = d.song;
    if (q("[data-artist]")) q("[data-artist]").textContent = d.artist;
    if (q("[data-fmt]")) q("[data-fmt]").textContent = d.fmt;
    if (q("[data-bits]")) q("[data-bits]").textContent = d.bits;
    var b = q("[data-fmt-badge]"); if (b) b.classList.toggle("hires", !!d.hires);
    var a = q("[data-art]"); if (a) a.alt = "Capa de " + d.album;
  }
  function state(s) {
    var els = document.querySelectorAll("[data-state]"); if (!els.length || !s) return;
    var known = Array.prototype.some.call(els, function (e) { return e.getAttribute("data-state") === s; });
    if (!known) return;
    els.forEach(function (el) { el.hidden = el.getAttribute("data-state") !== s; });
    document.querySelectorAll("[data-pick]").forEach(function (c) { c.setAttribute("aria-current", c.getAttribute("data-pick") === s ? "true" : "false"); });
  }
  function apply() {
    var p = params();
    if (p.has("theme")) setTheme(p.get("theme") === "auto" ? null : p.get("theme"));
    else { try { var t = localStorage.getItem("kz-theme"); if (t) setTheme(t === "auto" ? null : t); } catch (e) {} }
    setRT(p.get("rt") === "1");
    if (p.get("sat") === "0") root.setAttribute("data-sat", "0"); else root.removeAttribute("data-sat");
    if (p.get("c")) cover(p.get("c"));
    state(p.get("s"));
    if (p.get("motion") === "0") root.classList.add("reduce-motion");
  }
  window.addEventListener("hashchange", apply);
  window.KAZZU_TRACKS = {
    "01": { song: "Maré Alta", artist: "Aurora Vento", album: "Cidade Submersa", fmt: "OPUS", bits: "160 kbps" },
    "02": { song: "Hora do Café", artist: "Doma", album: "Nove Horas", fmt: "AAC", bits: "256 kbps" },
    "03": { song: "Mapa de Neblina", artist: "Lia Cantos", album: "Bruma", fmt: "OPUS", bits: "160 kbps" },
    "04": { song: "Canal Sete", artist: "Os Faróis", album: "Quadra 7", fmt: "OPUS", bits: "160 kbps" },
    "05": { song: "Escada de Sal", artist: "Tomás Reis", album: "Sal e Cinza", fmt: "FLAC", bits: "24-bit/192 kHz", hires: true },
    "06": { song: "Linha de Fuga", artist: "Meridiano", album: "Linha de Fuga", fmt: "OPUS", bits: "160 kbps" },
    "07": { song: "Quase Verão", artist: "Nina Sabiá", album: "Calor de Março", fmt: "OPUS", bits: "160 kbps" },
    "08": { song: "Respiração de Vidro", artist: "Kaito e Vê", album: "Vidro Líquido", fmt: "AAC", bits: "256 kbps" },
    "09": { song: "Seda e Sal", artist: "Ilha Branca", album: "Papel de Seda", fmt: "OPUS", bits: "160 kbps" },
    "10": { song: "Fita sem Fim", artist: "Rádio Cobre", album: "Fita Magnética", fmt: "FLAC", bits: "24-bit/96 kHz", hires: true }
  };
  apply();

  // links internos preservam as opções do hub (tema, transparência, saturação, capa)
  document.addEventListener("click", function (e) {
    var a = e.target.closest("a[href]");
    if (a && location.hash && /^[a-z]+\.html(#.*)?$/.test(a.getAttribute("href"))) {
      var base = a.getAttribute("href").split("#")[0], own = a.getAttribute("href").split("#")[1] || "";
      var p = params(), keep = new URLSearchParams(own);
      ["theme", "rt", "sat", "c"].forEach(function (k) { if (p.get(k) && !keep.has(k)) keep.set(k, p.get(k)); });
      if (String(keep)) a.href = base + "#" + keep;
    }
    var d = e.target.closest("[data-dismiss]"); if (d) { var t = document.getElementById(d.getAttribute("data-dismiss")); if (t) t.hidden = true; }
    var g = e.target.closest("[data-goto]"); if (g) { state(g.getAttribute("data-goto")); }
  });

  document.querySelectorAll('[role="switch"]').forEach(function (el) {
    el.addEventListener("click", function () {
      if (el.getAttribute("aria-disabled") === "true") return;
      var on = el.getAttribute("aria-checked") !== "true"; el.setAttribute("aria-checked", on);
      if (el.dataset.setting === "reduce-transparency") setRT(on);
      if (el.dataset.setting === "reduce-motion") root.classList.toggle("reduce-motion", on);
    });
  });
  document.querySelectorAll('[role="radiogroup"]').forEach(function (g) {
    g.querySelectorAll('[role="radio"]').forEach(function (b) {
      b.addEventListener("click", function () {
        g.querySelectorAll('[role="radio"]').forEach(function (x) { x.setAttribute("aria-checked", x === b); });
        if (b.dataset.theme) { var t = b.dataset.theme; setTheme(t === "auto" ? null : t); try { localStorage.setItem("kz-theme", t); } catch (e) {} }
        if (b.dataset.accent) document.body.classList.toggle("dyn", b.dataset.accent === "dyn");
      });
    });
  });
  document.querySelectorAll(".chip[aria-pressed]").forEach(function (c) {
    c.addEventListener("click", function () {
      if (c.hasAttribute("data-hub")) return;
      var grp = c.parentElement; grp.querySelectorAll(".chip[aria-pressed]").forEach(function (x) { x.setAttribute("aria-pressed", x === c); });
    });
  });
  document.querySelectorAll("[data-toggle]").forEach(function (b) {
    b.addEventListener("click", function () {
      var on = b.getAttribute("aria-pressed") !== "true"; b.setAttribute("aria-pressed", on);
      var u = b.querySelector("use"); if (u && /heart/.test(u.getAttribute("href"))) u.setAttribute("href", on ? "#i-heart-fill" : "#i-heart");
      if (b.getAttribute("aria-label") === "Curtir" || b.getAttribute("aria-label") === "Descurtir") b.setAttribute("aria-label", on ? "Descurtir" : "Curtir");
    });
  });
  document.querySelectorAll("[data-playtoggle]").forEach(function (pt) {
    pt.addEventListener("click", function () {
      var playing = pt.getAttribute("aria-label") === "Pausar";
      pt.setAttribute("aria-label", playing ? "Tocar" : "Pausar");
      pt.querySelector("use").setAttribute("href", playing ? "#i-play" : "#i-pause");
    });
  });
  var q = document.querySelector("[data-q]");
  if (q) q.addEventListener("input", function () { if (q.value.length) state("results"); else state("idle"); });

  // sliders: arrastar e teclado (setas ±5%)
  document.querySelectorAll(".slider").forEach(function (s) {
    function setP(clientX) { var r = s.getBoundingClientRect(); var p = Math.max(0, Math.min(1, (clientX - r.left) / r.width)); s.style.setProperty("--p", (p * 100).toFixed(1) + "%"); s.setAttribute("aria-valuenow", Math.round(p * (+s.getAttribute("aria-valuemax") || 100))); }
    s.addEventListener("pointerdown", function (e) { s.classList.add("drag"); s.setPointerCapture(e.pointerId); setP(e.clientX); });
    s.addEventListener("pointermove", function (e) { if (s.classList.contains("drag")) setP(e.clientX); });
    s.addEventListener("pointerup", function () { s.classList.remove("drag"); });
    s.addEventListener("keydown", function (e) {
      var cur = parseFloat(getComputedStyle(s).getPropertyValue("--p")) || 0, d = e.key === "ArrowRight" || e.key === "ArrowUp" ? 5 : e.key === "ArrowLeft" || e.key === "ArrowDown" ? -5 : 0;
      if (d) { e.preventDefault(); s.style.setProperty("--p", Math.max(0, Math.min(100, cur + d)) + "%"); }
    });
  });

  // gesto: arrastar a folha do player para baixo fecha (alternativa por toque: botão "Fechar o player")
  var sheet = document.querySelector("[data-gesture=sheet]");
  if (sheet) {
    var y0 = null, dy = 0;
    var handle = sheet.querySelector(".grabber"), head = sheet.querySelector(".player-top");
    [handle, head].forEach(function (h) {
      if (!h) return;
      h.style.touchAction = "none";
      h.addEventListener("pointerdown", function (e) { if (e.target.closest("a,button")) return; y0 = e.clientY; dy = 0; h.setPointerCapture(e.pointerId); });
      h.addEventListener("pointermove", function (e) { if (y0 === null) return; dy = Math.max(0, e.clientY - y0); sheet.style.transform = "translateY(" + dy + "px)"; });
      h.addEventListener("pointerup", function () { if (y0 === null) return; if (dy > 140) location.href = "home.html" + (location.hash || ""); else sheet.style.transform = ""; y0 = null; });
    });
  }

  // cabeçalho de rolagem (Apple): vira vidro ao rolar; o título pequeno entra quando o título grande sai
  var hdr = document.querySelector("[data-hdr]");
  if (hdr) {
    var lt = document.querySelector(".lt");
    var upd = function () {
      var y = window.scrollY || 0;
      hdr.classList.toggle("scrolled", y > 4);
      hdr.classList.toggle("title-in", lt ? lt.getBoundingClientRect().bottom < 64 : y > 40);
    };
    window.addEventListener("scroll", upd, { passive: true }); upd();
  }

  // ---- ferramentas de verificação (tools/check_render.py): #scroll=N rola a página; #probe=1 lista os textos; #probe=2 esconde os textos
  var pp = params();
  if (pp.get("scroll")) window.addEventListener("load", function () { window.scrollTo(0, +pp.get("scroll")); });
  if (pp.get("probe") === "2") { var st2 = document.createElement("style"); st2.textContent = "*{color:transparent!important;text-shadow:none!important;-webkit-text-fill-color:transparent!important} svg,.nowbars{visibility:hidden!important} input::placeholder{color:transparent!important}"; document.head.appendChild(st2); }
  if (pp.get("probe") === "1") window.addEventListener("load", function () {
    (document.fonts ? Promise.all(["400 16px Inter","500 16px Inter","600 16px Inter","700 16px Inter","700 16px Inter Display","600 16px Inter Display","400 12px Geist Mono","500 12px Geist Mono"].map(function (f) { return document.fonts.load(f); })) : Promise.resolve()).then(function () { setTimeout(function () {
      if (pp.get("probe") === "1") {
        var out = [], w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT), n;
        while ((n = w.nextNode())) {
          var t = n.nodeValue.trim(); if (!t) continue; var el = n.parentElement; if (!el || el.closest("script,style,[hidden],.sr-only")) continue;
          var cs = getComputedStyle(el); if (cs.visibility === "hidden" || cs.display === "none") continue;
          var eo = 1, an = el; while (an && an !== document.documentElement) { eo *= parseFloat(getComputedStyle(an).opacity); an = an.parentElement; } if (eo < 0.05) continue;
          var r = document.createRange(); r.selectNodeContents(n); var b = r.getBoundingClientRect(); if (b.width < 2 || b.height < 2) continue;
          if (b.bottom < 0 || b.top > innerHeight || b.right < 0 || b.left > innerWidth) continue;
          var hit = document.elementFromPoint(Math.min(innerWidth - 1, Math.max(0, b.left + b.width / 2)), Math.min(innerHeight - 1, Math.max(0, b.top + b.height / 2)));
          if (!hit || !(el.contains(hit) || hit.contains(el))) continue;
          out.push({ t: t.slice(0, 48), c: cs.color, x: b.left, y: b.top, w: b.width, h: b.height, fs: parseFloat(cs.fontSize), fw: +cs.fontWeight || 400 });
        }
        console.log("KZPROBE" + JSON.stringify(out));
        var st3 = document.createElement("style"); st3.textContent = "*{color:transparent!important;text-shadow:none!important;-webkit-text-fill-color:transparent!important} svg,.nowbars{visibility:hidden!important} input::placeholder{color:transparent!important}"; document.head.appendChild(st3);
      } else {
        var st = document.createElement("style"); st.textContent = "*{color:transparent!important;text-shadow:none!important} svg,.nowbars{visibility:hidden!important} input::placeholder{color:transparent!important}"; document.head.appendChild(st);
      }
    }, 1500); });
  });
})();
