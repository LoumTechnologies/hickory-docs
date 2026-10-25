<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/site.html

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `53f32f322dde289c3305662d0eae43cd15b82b189c0e8183fd721dfdee016371`.

<book:entry path="apps/web/site.html">
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="color-scheme" content="dark" />
    <title>Hickory Docs — a downloadable notes IDE</title>
    <!-- Theme, before the bundle — same contract as index.html and
         src/lib/theme.ts (key, values, default "dark"). -->
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
