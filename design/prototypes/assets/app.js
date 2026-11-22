/* Kazzu protótipos — interações mínimas (tema, transparência, sliders, interruptores, troca de capa). Sem rede, sem telemetria. */
(function () {
  var root = document.documentElement;
  root.classList.add("kz-init"); // sem transições durante a aplicação inicial do tema
  requestAnimationFrame(function () { requestAnimationFrame(function () { root.classList.remove("kz-init"); }); });
  function params() { return new URLSearchParams(location.hash.replace(/^#/, "")); }
  function setTheme(t) { if (t === "light" || t === "dark") root.setAttribute("data-theme", t); else root.removeAttribute("data-theme"); }
  function setRT(on) { root.classList.toggle("reduce-transparency", !!on); }
  function cover(id) {
    var d = (window.KAZZU_TRACKS || {})[id]; if (!d) return;
    document.body.className = document.body.className.replace(/\bcv-\d+\b/, "cv-" + id);
    document.body.setAttribute("data-cover", id);
    var src = "covers/cover-" + id + ".svg";
    document.querySelectorAll("[data-art],[data-blur],[data-ambient]").forEach(function (i) { i.src = src; });
    var q = function (s) { return document.querySelector(s); };
    if (q("[data-song]")) q("[data-song]").textContent = d.song;
    if (q("[data-artist]")) q("[data-artist]").textContent = d.artist;
    if (q("[data-fmt]")) q("[data-fmt]").textContent = d.fmt;
    if (q("[data-bits]")) q("[data-bits]").textContent = d.bits;
    var b = q("[data-fmt-badge]"); if (b) b.classList.toggle("hires", d.hires);
    var a = q("[data-art]"); if (a) a.alt = "Capa de " + d.song;
  }
  function apply() {
    var p = params();
    if (p.has("theme")) setTheme(p.get("theme") === "auto" ? null : p.get("theme"));
    else { try { var t = localStorage.getItem("kz-theme"); if (t) setTheme(t === "auto" ? null : t); } catch (e) {} }
    setRT(p.get("rt") === "1");
    if (p.get("c")) cover(p.get("c"));
    var s = p.get("s");
    if (s && document.querySelector("[data-state]")) {
      document.querySelectorAll("[data-state]").forEach(function (el) { el.hidden = el.getAttribute("data-state") !== s; });
    }
  }
  window.addEventListener("hashchange", apply); apply();

  // links internos preservam tema/transparência quando a página está num iframe do hub
  document.addEventListener("click", function (e) {
    var st = e.target.closest("[data-stop]"); if (st) { e.preventDefault(); e.stopPropagation(); }
    var a = e.target.closest("a[href]");
    if (a && !st && location.hash && /^[a-z]+\.html$/.test(a.getAttribute("href"))) {
      var p = params(); var keep = new URLSearchParams(); ["theme", "rt"].forEach(function (k) { if (p.get(k)) keep.set(k, p.get(k)); });
      if (String(keep)) a.href = a.getAttribute("href") + "#" + keep;
    }
  });

  // interruptores
  document.querySelectorAll('[role="switch"]').forEach(function (el) {
    el.addEventListener("click", function () {
      if (el.getAttribute("aria-disabled") === "true") return;
      var on = el.getAttribute("aria-checked") !== "true"; el.setAttribute("aria-checked", on);
      if (el.dataset.setting === "reduce-transparency") setRT(on);
      if (el.dataset.setting === "reduce-motion") root.classList.toggle("reduce-motion", on);
    });
  });
  // grupos segmentados
  document.querySelectorAll('[role="radiogroup"]').forEach(function (g) {
    g.querySelectorAll('[role="radio"]').forEach(function (b) {
      b.addEventListener("click", function () {
        g.querySelectorAll('[role="radio"]').forEach(function (x) { x.setAttribute("aria-checked", x === b); });
        if (b.dataset.theme) { var t = b.dataset.theme; setTheme(t === "auto" ? null : t); try { localStorage.setItem("kz-theme", t); } catch (e) {} }
        if (b.dataset.accent) { document.body.classList.toggle("dyn", b.dataset.accent === "dyn"); }
      });
    });
  });
  // chips (alternância)
  document.querySelectorAll(".chip[aria-pressed]").forEach(function (c) {
    c.addEventListener("click", function () {
      var grp = c.parentElement; var single = grp.getAttribute("aria-label") === "Filtrar por tipo" || !grp.getAttribute("aria-label");
      if (single) grp.querySelectorAll(".chip").forEach(function (x) { x.setAttribute("aria-pressed", x === c); });
      else c.setAttribute("aria-pressed", c.getAttribute("aria-pressed") !== "true");
    });
  });
  // botões de alternância (curtir/aleatório/repetir)
  document.querySelectorAll("[data-toggle]").forEach(function (b) { b.addEventListener("click", function () { b.setAttribute("aria-pressed", b.getAttribute("aria-pressed") !== "true"); }); });
  var pt = document.querySelector("[data-playtoggle]");
  if (pt) pt.addEventListener("click", function () {
    var playing = pt.getAttribute("aria-label") === "Pausar";
    pt.setAttribute("aria-label", playing ? "Tocar" : "Pausar");
    pt.querySelector("use").setAttribute("href", playing ? "#i-play" : "#i-pause");
  });
  // sliders
  function fmtT(s) { return Math.floor(s / 60) + ":" + ("0" + Math.floor(s % 60)).slice(-2); }
  document.querySelectorAll(".slider[role=slider]").forEach(function (s) {
    if (s.getAttribute("aria-disabled") === "true") return;
    var max = +s.getAttribute("aria-valuemax"), min = +s.getAttribute("aria-valuemin") || 0;
    function set(v) {
      v = Math.max(min, Math.min(max, v)); s.setAttribute("aria-valuenow", Math.round(v));
      s.style.setProperty("--p", ((v - min) / (max - min) * 100) + "%");
      if (s.dataset.kind === "seek") {
        s.setAttribute("aria-valuetext", fmtT(v) + " de " + fmtT(max));
        var cur = document.querySelector("[data-cur]"), left = document.querySelector("[data-left]");
        if (cur) cur.textContent = fmtT(v); if (left) left.textContent = "−" + fmtT(max - v);
      } else s.setAttribute("aria-valuetext", Math.round(v) + "%");
    }
    function fromEvent(e) { var r = s.getBoundingClientRect(); set(min + (e.clientX - r.left) / r.width * (max - min)); }
    s.addEventListener("pointerdown", function (e) { s.setPointerCapture(e.pointerId); s.classList.add("drag"); fromEvent(e); });
    s.addEventListener("pointermove", function (e) { if (s.classList.contains("drag")) fromEvent(e); });
    s.addEventListener("pointerup", function () { s.classList.remove("drag"); });
    s.addEventListener("keydown", function (e) {
      var v = +s.getAttribute("aria-valuenow"), step = s.dataset.kind === "seek" ? 5 : 5;
      if (e.key === "ArrowRight" || e.key === "ArrowUp") { set(v + step); e.preventDefault(); }
      if (e.key === "ArrowLeft" || e.key === "ArrowDown") { set(v - step); e.preventDefault(); }
      if (e.key === "Home") { set(min); e.preventDefault(); } if (e.key === "End") { set(max); e.preventDefault(); }
    });
  });
  // carrossel
  document.querySelectorAll("[data-rail]").forEach(function (b) {
    b.addEventListener("click", function () { var r = document.getElementById(b.dataset.target); r.scrollBy({ left: +b.dataset.rail * 400, behavior: "smooth" }); });
  });
  // protótipo: trocar capa
  document.querySelectorAll("[data-cover-pick]").forEach(function (b) { b.addEventListener("click", function () { cover(b.dataset.coverPick); }); });
  // busca: limpar
  var inp = document.querySelector("[data-search-input]"), clr = document.querySelector("[data-clear]");
  if (inp && clr) clr.addEventListener("click", function () { inp.value = ""; inp.focus(); document.querySelectorAll("[data-state]").forEach(function (el) { el.hidden = el.getAttribute("data-state") !== "idle"; }); });
  if (inp) inp.addEventListener("input", function () { var st = inp.value ? "results" : "idle"; document.querySelectorAll("[data-state]").forEach(function (el) { el.hidden = el.getAttribute("data-state") !== st; }); });
})();
