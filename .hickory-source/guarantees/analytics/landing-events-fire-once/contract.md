<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Visit Is Reported Exactly Once Per Page Load

For: Maintainers evaluating landing-page behavior and visitor privacy.

Given the homepage, when it mounts, then exactly one
`landing_viewed` event fires for that page load, regardless of how many times
React re-renders or re-mounts the view. In particular, React's StrictMode
double-mount in development must not produce two events.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/analytics/landing-events-fire-once.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-b156a6a8bd0c"></a>
## `apps/web/src/main.site.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Visit Is Reported Exactly Once Per Page Load](contract.md).

<a id="f-b156a6a8bd0c-expression_statement"></a>
### `expression_statement`

Entry point for the **static marketing site** (`site.html`).
It renders one page and mounts no API client: the site has no server to talk
to, and the demos on it run entirely in the browser. The desktop app's entry
is `main.tsx`; the two never load each other's code.

<storycode:copy id="f-b156a6a8bd0c-expression_statement">
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

</storycode:copy>
<a id="f-b156a6a8bd0c-expression_statement-2"></a>
### `expression_statement`

The demos run the real parser in the browser — the same `hick-lang` the
product ships, as WebAssembly — so a document the site shows is drawn the
way the app draws it.

<storycode:copy id="f-b156a6a8bd0c-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/main.site.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/main.site.tsx"><storycode:paste select="#f-b156a6a8bd0c-expression_statement"/><storycode:paste select="#f-b156a6a8bd0c-expression_statement-2"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/main.site.tsx](contract.md#output-b156a6a8bd0c)
- [apps/web/src/views/LandingView.tsx](../../../mechanisms/apps/web/src/views/LandingView.tsx.md#output-e86ad48a0990)

### Tests and fixtures

- [apps/web/src/views/LandingView.test.tsx](../../../mechanisms/apps/web/src/views/LandingView.tsx.md#output-ca24d1670e8c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
