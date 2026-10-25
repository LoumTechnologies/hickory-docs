<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/index.html

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


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
    <!-- Theme, before the bundle: styles.css defaults to the dark theme on
         :root, so only a stored non-default choice needs applying — but it
         must land before first paint or a light-theme user gets a dark
         flash. Mirrors src/lib/theme.ts (key, values, default). -->
    <script>
      (function () {
        var theme;
        try {
          theme = localStorage.getItem("hickory.theme");
        } catch (e) {
          /* storage disabled: keep the default */
        }
        if (theme !== "dark" && theme !== "warm-dark" && theme !== "light") theme = "dark";
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
