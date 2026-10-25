<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/main.site.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Visit Is Reported Exactly Once Per Page Load](../../../../contract.md)

Captured SHA-256: `c01bde9d9208a3790ee40119956cdfc191798d55ae1f006c036ba37bd30ffcb8`.

<book:entry path="apps/web/src/main.site.tsx">
// Entry point for the **static marketing site** (`site.html`).
//
// It renders one page and mounts no API client: the site has no server to talk
// to, and the demos on it run entirely in the browser. The desktop app's entry
// is `main.tsx`; the two never load each other's code.
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { LandingView } from "./views/LandingView";
import { StaticUiSurface } from "./embed/StaticUiSurface";
import { applyStoredTheme } from "./lib/theme";
import { loadHickLang } from "./editor/hickLang";
import "./styles.css";

applyStoredTheme();

// The demos run the real parser in the browser — the same `hick-lang` the
// product ships, as WebAssembly — so a document the site shows is drawn the
// way the app draws it.
void loadHickLang().then(() => {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <StaticUiSurface><LandingView /></StaticUiSurface>
    </StrictMode>,
  );
});
</book:entry>

</book:doc>
