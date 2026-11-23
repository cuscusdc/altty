// Comportamento mínimo dos protótipos da v4. Nada aqui é necessário para ler as telas: sem script elas aparecem inteiras.
(function () {
  var d = document.documentElement, b = document.body;
  var h = new URLSearchParams(location.hash.slice(1));
  // leva tema e acessibilidade de uma tela para a outra
  var keep = ['theme', 'rt', 'rm'].filter(function (k) { return h.get(k); }).map(function (k) { return k + '=' + h.get(k); }).join('&');
  if (keep) document.querySelectorAll('a[href]').forEach(function (a) {
    var u = a.getAttribute('href');
    if (/\.html(#|$)/.test(u)) a.setAttribute('href', u + (u.indexOf('#') < 0 ? '#' : '&') + keep);
  });
  // cabeçalho de vidro quando o título sai da tela
  // nas telas de detalhe o cabeçalho só aparece depois que a placa do título saiu de vista
  var hero = document.querySelector('.detail .hero');
  function onScroll() { b.classList.toggle('scrolled', window.scrollY > (hero ? hero.offsetHeight - 110 : 56)); }
  window.addEventListener('scroll', onScroll, { passive: true });
  if (h.get('scroll')) window.scrollTo(0, +h.get('scroll'));
  onScroll();
  // capa do player ou do álbum pela URL (#c=NN)
  var c = h.get('c'), K = window.KZ_COVERS || {};
  if (c && K[c]) {
    document.querySelectorAll('.player .art, .hero > img').forEach(function (i) { i.src = '../assets/covers/' + c + '.jpg'; });
    document.querySelectorAll('.player .backdrop').forEach(function (i) { i.src = '../assets/covers/' + c + (i.classList.contains('backdrop--wide') ? '-blur.jpg' : '-back.jpg'); });
    document.querySelectorAll('.console').forEach(function (e) { e.style.setProperty('--dim', K[c].d); });
    document.querySelectorAll('[data-c-title], [data-c-album]').forEach(function (e) { e.textContent = K[c].t; });
    document.querySelectorAll('[data-c-artist]').forEach(function (e) { e.textContent = K[c].a; });
    document.querySelectorAll('[data-c-sub]').forEach(function (e) { e.textContent = K[c].k + ' de ' + K[c].a; });
  }
  // alternâncias
  document.addEventListener('click', function (ev) {
    var t = ev.target.closest('button, [role="tab"]');
    if (!t) return;
    if (t.matches('.pp')) { b.classList.toggle('paused'); document.querySelectorAll('.pp').forEach(function (p) { p.setAttribute('aria-label', b.classList.contains('paused') ? 'Tocar' : 'Pausar'); }); }
    else if (t.matches('[role="switch"]')) {
      var on = t.getAttribute('aria-checked') !== 'true'; t.setAttribute('aria-checked', on);
      if (t.dataset.switch === 'rt') d.toggleAttribute('data-rt', on);
      if (t.dataset.switch === 'rm') d.toggleAttribute('data-rm', on);
    }
    else if (t.matches('.seg [aria-pressed]')) { t.parentNode.querySelectorAll('[aria-pressed]').forEach(function (x) { x.setAttribute('aria-pressed', x === t); }); var m = { 'Claro': 'light', 'Escuro': 'dark' }[t.textContent]; if (t.parentNode.getAttribute('aria-labelledby') === 'st-tema') { if (m) d.dataset.mode = m; else delete d.dataset.mode; } }
    else if (t.matches('[role="tab"]')) t.parentNode.querySelectorAll('[role="tab"]').forEach(function (x) { x.setAttribute('aria-selected', x === t); });
    else if (t.matches('[aria-pressed]')) t.setAttribute('aria-pressed', t.getAttribute('aria-pressed') !== 'true');
    else if (t.matches('[data-dismiss]')) t.closest('.banner').hidden = true;
    else if (t.matches('[data-clear]')) { var q = document.getElementById('q'); q.value = ''; q.focus(); }
  });
  // busca e nova playlist: nada é enviado a lugar nenhum
  document.querySelectorAll('form').forEach(function (f) {
    f.addEventListener('submit', function (ev) {
      ev.preventDefault();
      if (f.matches('[data-search]')) { location.href = 'search-results.html' + (keep ? '#' + keep : ''); return; }
      var i = f.querySelector('input'), help = f.querySelector('[aria-live]');
      if (!i.value.trim()) { i.setAttribute('aria-invalid', 'true'); i.setAttribute('aria-describedby', help.id); help.textContent = 'Dê um nome para a playlist.'; i.focus(); return; }
      i.removeAttribute('aria-invalid'); help.textContent = 'Protótipo: a playlist não é gravada aqui.';
    });
  });
  if (h.get('focus') === '1') { var q = document.getElementById('q'); if (q) q.focus(); }
})();
