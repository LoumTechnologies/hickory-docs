<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/index.html

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Resolve appearance before the bundle paints: explicit saved themes take precedence,
otherwise use the OS light/dark preference. The shared theme mechanism tests this
inline resolver alongside the runtime implementation.


Captured SHA-256: `2f848d9bfd237d64640cde5e5fa8ea4080532a71c8142827d7be6e92f1c93017`.

<book:entry path="apps/web/index.html">
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="color-scheme" content="dark" />
    <title>Hickory Docs</title>
    <style>
      /* Available before the application bundle and its stylesheet arrive. */
      body { margin: 0; }
      .boot-status { margin: 0; padding: 2rem; min-height: 100vh; box-sizing: border-box;
        background: #1e2227; color: #dde3ea; font: 16px system-ui, sans-serif; }
      :root[data-theme="light"] .boot-status { background: #faf9f7; color: #21201d; }
      :root[data-theme="warm-dark"] .boot-status { background: #161512; color: #e6e3dc; }
    </style>
    <!-- Resolve saved choice or OS appearance before first paint.
         Keep this in sync with src/lib/theme.ts. -->
    <script>
      (function () {
        var theme;
        try {
          theme = localStorage.getItem("hickory.theme");
        } catch (e) {
          /* storage disabled: keep the default */
        }
        if (theme !== "dark" && theme !== "warm-dark" && theme !== "light") {
          theme = window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
        }
        document.documentElement.dataset.theme = theme;
        var meta = document.querySelector('meta[name="color-scheme"]');
        if (meta) meta.setAttribute("content", theme === "light" ? "light" : "dark");
      })();
    </script>
  </head>
  <body>
    <div id="root"><p class="boot-status" role="status">Opening Hickory Docs…</p></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
</book:entry>

</book:doc>
