(function () {
  // Keep this script readable by WebViews that cannot load the application's module.
  if (!Object.hasOwn) {
    Object.defineProperty(Object, 'hasOwn', {
      configurable: true,
      writable: true,
      value: function (object, key) {
        return Object.prototype.hasOwnProperty.call(object, key);
      }
    });
  }
  if (!Array.prototype.at) {
    Object.defineProperty(Array.prototype, 'at', {
      configurable: true,
      writable: true,
      value: function (index) {
        index = +index || 0;
        index = index < 0 ? Math.ceil(index) : Math.floor(index);
        if (index < 0) index += this.length;
        return index >= 0 && index < this.length ? this[index] : undefined;
      }
    });
  }

  var status = document.getElementById('startup-status');
  var message = document.getElementById('startup-message');
  var details = document.getElementById('startup-details');
  var retry = document.getElementById('startup-retry');
  var android = /Android/i.test(navigator.userAgent);
  var engine = /Chrome\/([\d.]+)/.exec(navigator.userAgent);
  var timer = window.setTimeout(function () {
    showFailure('启动超时 / Startup timed out');
  }, 15000);

  function showFailure(reason) {
    if (!status.parentNode) return;
    window.clearTimeout(timer);
    message.textContent = android
      ? '应用未能启动。请更新 Android System WebView 或 Chrome，再重试。 / Update Android System WebView or Chrome, then retry.'
      : '应用未能启动。请重试。 / The app failed to start. Please retry.';
    details.textContent = [engine ? 'Chromium ' + engine[1] : '', reason].filter(Boolean).join(' · ');
    details.hidden = false;
    retry.hidden = false;
  }

  window.addEventListener('error', function (event) {
    if (event.message) {
      var file = event.filename && event.filename.split('/').pop();
      showFailure(event.message + (file && event.lineno ? ' (' + file + ':' + event.lineno + ')' : ''));
    }
    else if (event.target && event.target.tagName === 'SCRIPT') {
      showFailure('无法加载界面脚本 / Could not load the app script');
    }
  }, true);
  window.addEventListener('hoi4reader:startup-error', function (event) {
    showFailure(event.detail);
  });
  retry.addEventListener('click', function () { window.location.reload(); });
}());
