// kfk: a live terminal on a node. One per element with data-term="<seat>".
(function () {
  'use strict';
  var FONT = 'ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace';

  function start(box) {
    var seat = box.dataset.term;
    var path = '/ws/node/' + encodeURIComponent(seat);
    var screen = box.querySelector('.term-screen');
    var status = box.querySelector('.term-status');
    var again = box.querySelector('[data-term-connect]');
    var full = box.querySelector('[data-term-full]');
    var term = new Terminal({
      cursorBlink: true, scrollback: 0, fontFamily: FONT, fontSize: 13, macOptionClickForcesSelection: true,
      theme: { background: '#14151a', foreground: '#e8e6df', cursor: '#e8e6df' }
    });
    var fit = new FitAddon.FitAddon();
    var enc = new TextEncoder();
    var ws = null;
    term.loadAddon(fit);
    term.open(screen);

    function say(text, offerRetry) { status.textContent = text; again.hidden = !offerRetry; }
    function refit() { try { fit.fit(); } catch (e) { /* not laid out yet */ } }
    function send(data) { if (ws && ws.readyState === 1) ws.send(data); }
    function sendSize() { send(JSON.stringify({ cols: term.cols, rows: term.rows })); }

    function connect() {
      refit();
      say('Connecting…', false);
      var scheme = location.protocol === 'https:' ? 'wss://' : 'ws://';
      var sock = new WebSocket(scheme + location.host + path + '?cols=' + term.cols + '&rows=' + term.rows);
      var opened = false;
      sock.binaryType = 'arraybuffer';
      sock.onopen = function () { opened = true; ws = sock; say('Connected: you are typing into ' + seat, false); sendSize(); term.focus(); };
      sock.onmessage = function (e) { term.write(typeof e.data === 'string' ? e.data : new Uint8Array(e.data)); };
      sock.onclose = function () {
        ws = null;
        if (opened) { say('Disconnected.', true); return; }
        // the server says why in plain text when asked without a WebSocket
        fetch(path).then(function (r) { return r.text(); }).then(function (t) { say('Not connected. ' + t, true); }, function () { say('Not connected: the web view is not answering.', true); });
      };
    }

    term.onData(function (d) { send(enc.encode(d)); });
    term.onBinary(function (d) { var b = new Uint8Array(d.length); for (var i = 0; i < d.length; i++) b[i] = d.charCodeAt(i) & 255; send(b); });
    term.onResize(sendSize);
    new ResizeObserver(refit).observe(screen);
    again.addEventListener('click', connect);

    // Full screen fills the window. Where the browser can leave Escape to the page (keyboard lock), it also
    // goes to real full screen; elsewhere Escape would leave full screen instead of reaching the node.
    var native = !!(navigator.keyboard && navigator.keyboard.lock && box.requestFullscreen);
    function setFull(on) {
      box.classList.toggle('term-max', on);
      document.documentElement.classList.toggle('term-open', on);
      full.setAttribute('aria-pressed', on ? 'true' : 'false');
      full.textContent = on ? 'Exit full screen' : 'Full screen';
      refit();
      term.focus();
    }
    if (full) {
      full.addEventListener('click', function () {
        var on = !box.classList.contains('term-max');
        setFull(on);
        if (!native) return;
        if (on) box.requestFullscreen().then(function () { return navigator.keyboard.lock(['Escape']); }).catch(function () {});
        else if (document.fullscreenElement) document.exitFullscreen();
      });
      document.addEventListener('fullscreenchange', function () {
        if (native && !document.fullscreenElement && box.classList.contains('term-max')) setFull(false);
        else refit();
      });
    }

    if (box.dataset.auto === '1') connect();
    else say('The node is not up.', true);
  }

  document.querySelectorAll('[data-term]').forEach(start);
})();
