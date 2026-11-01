<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/site.html

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Resolve appearance before the bundle paints: explicit saved themes take precedence,
otherwise use the OS light/dark preference. The shared theme mechanism tests this
inline resolver alongside the runtime implementation.


Captured SHA-256: `53f32f322dde289c3305662d0eae43cd15b82b189c0e8183fd721dfdee016371`.

<book:entry path="apps/web/site.html">
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="color-scheme" content="dark" />
    <title>Hickory Docs — a downloadable notes IDE</title>
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
    <!-- The landing page is reached from ads, link shares, and community
         posts, all of which render this card rather than the page itself. -->
    <meta
      name="description"
      content="A downloadable notes IDE. Prose and code together in a Markdown file. Try a live debugger in your browser."
    />
    <meta property="og:type" content="website" />
    <meta
      property="og:title"
      content="Hickory Docs — a downloadable notes IDE"
    />
    <meta
      property="og:description"
      content="Runs on your machine, with your own files. Try a short document with a live, paused debugger."
    />
    <meta name="twitter:card" content="summary_large_image" />
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.site.tsx"></script>
  </body>
</html>
</book:entry>

</book:doc>
