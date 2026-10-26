<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/LandingView.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e86ad48a0990"></a>
## `apps/web/src/views/LandingView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Visit Is Reported Exactly Once Per Page Load](../../../../../guarantees/analytics/landing-events-fire-once/contract.md), [Every Hover Hint Is Drawn In The App Theme](../../../../../guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme/contract.md), [The engineer tour distinguishes replay from execution](../../../../../guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md), [Every Claim On The Home Page Is True Of The Downloaded Binary](../../../../../guarantees/landing/home-page-claims-are-true-of-the-binary/contract.md), [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md).

<a id="f-e86ad48a0990-viewedFired"></a>
### `viewedFired`

<storycode:copy id="f-e86ad48a0990-viewedFired">
import { UiLink } from "../components/UiLink";
import { config } from "../config";
import { WorkflowTour } from "../components/WorkflowTour";
import { useEffect, useState } from "react";
import { emit } from "../analytics/events";
import { TooltipLayer } from "../components/TooltipLayer";
import { BrowserDebugDemo } from "../landing/demos/BrowserDebugDemo";
import { WorkflowRecording } from "../landing/WorkflowRecording";
import "../landing/homepage.css";

let viewedFired = false;
</storycode:copy>
<a id="f-e86ad48a0990-resetViewedForTest"></a>
### `resetViewedForTest`

<storycode:copy id="f-e86ad48a0990-resetViewedForTest">
export function resetViewedForTest() { viewedFired = false; }

</storycode:copy>
<a id="f-e86ad48a0990-LandingView"></a>
### `LandingView`

A first visit from a developer: see a document, try its debugger, download.

<storycode:copy id="f-e86ad48a0990-LandingView">
/** A first visit from a developer: see a document, try its debugger, download. */
export function LandingView() {
  const [predictionOnly, setPredictionOnly] = useState(() => new URL(location.href).searchParams.get("predictionPractice") === "1");
  useEffect(() => {
    if (viewedFired) return;
    viewedFired = true;
    emit({ name: "landing_viewed", path: location.hash || "#/" });
  }, []);
  if (predictionOnly) return <main className="landing landing-simple"><WorkflowTour surface="site" predictionOnly /></main>;
  return <main className="landing landing-simple">
    <header className="landing-hero">
      <p className="landing-eyebrow">Downloadable software · Runs on your machine</p>
      <h1>Hickory Docs</h1>
      <p className="landing-sub">Delegate a bug investigation, inspect the agent's source-linked evidence, and review a focused, verified change. Hickory keeps the requirement, code, debugger and execution receipts together in Markdown.</p>
      <UiLink className="btn btn-primary" href="#watch-workflow" onClick={() => emit({ name: "cta_clicked", cta_id: "inspect-agent-workflow" })}>Inspect the agent workflow</UiLink>{" "}
      <UiLink className="btn" href="#download">Get the desktop app</UiLink>
      <p><UiLink href="https://github.com/LoumTechnologies/hickory-docs">View source on GitHub</UiLink></p>
    </header>
    <WorkflowRecording />
    <p><UiLink href={`https://github.com/LoumTechnologies/hickory-docs/blob/${config.sourceRevision ?? "master"}/docs/developers/agent-first-demo.md`} onClick={() => emit({ name: "interest_clicked", interest_id: "agent-engineering", link_id: "review-brief" })}>Read the customer problem, delegation, decision and evidence scope</UiLink> · <UiLink href="#try-workflow" onClick={() => emit({ name: "cta_clicked", cta_id: "try-retry-budget" })}>Try the matching browser example</UiLink></p>
    <div id="try-workflow"><WorkflowTour surface="site" onPredictionPractice={() => setPredictionOnly(true)} /></div>
    <BrowserDebugDemo />
    <footer className="landing-download" id="download">
      <h2>Download Hickory Docs</h2>
      <p>The desktop app works with your own files and tools. No account required.</p>
      <p><UiLink href="https://github.com/LoumTechnologies/hickory-docs/releases/tag/unstable" onClick={() => emit({ name: "cta_clicked", cta_id: "desktop-prerelease" })}>Current desktop prerelease downloads</UiLink> · <UiLink href="https://github.com/LoumTechnologies/hickory-docs/blob/master/apps/desktop/README.md">Build and setup instructions</UiLink></p>
      <p className="muted">The unstable channel has no compatibility promise. Check each artifact's build revision: published packages may predate this development workflow. Source/build instructions and asset availability do not establish a witnessed native/provider run.</p>
      <p><UiLink href="./snapshot.html?index=publications/index.json">Published executable-spec results</UiLink></p>
      <p><UiLink href="./snapshot.html">Open a document snapshot</UiLink></p>
      <p><UiLink href="./ci.html">Explore a local CI run with its literate source</UiLink></p>
      {config.sourceRevision && <p className="muted">Build source: <UiLink href={`https://github.com/LoumTechnologies/hickory-docs/tree/${config.sourceRevision}`}>{config.sourceRevision.slice(0, 7)}</UiLink></p>}
    </footer>
    <TooltipLayer />
  </main>;
}
</storycode:copy>

### Assemble `apps/web/src/views/LandingView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/LandingView.tsx"><storycode:paste select="#f-e86ad48a0990-viewedFired"/><storycode:paste select="#f-e86ad48a0990-resetViewedForTest"/><storycode:paste select="#f-e86ad48a0990-LandingView"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-ca24d1670e8c"></a>
## `apps/web/src/views/LandingView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Visit Is Reported Exactly Once Per Page Load](../../../../../guarantees/analytics/landing-events-fire-once/contract.md), [The homepage opens with a live, paused debugger](../../../../../guarantees/landing/homepage-opens-paused/contract.md).

<a id="f-ca24d1670e8c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ca24d1670e8c-expression_statement">
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setDeliveryForTest } from "../analytics/sink";
import { LandingView, resetViewedForTest } from "./LandingView";

// Browser execution is exercised in all three real engines by Playwright.
vi.mock("../landing/demos/BrowserDebugDemo", () => ({ BrowserDebugDemo: () => <section aria-label="Live document demo" /> }));
</storycode:copy>
<a id="f-ca24d1670e8c-captured"></a>
### `captured`

<storycode:copy id="f-ca24d1670e8c-captured">
let captured: string[] = [];
</storycode:copy>
<a id="f-ca24d1670e8c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ca24d1670e8c-expression_statement-2">
beforeEach(() => {
  localStorage.clear(); captured = []; resetViewedForTest();
  setDeliveryForTest((event) => { captured.push(event); });
});
</storycode:copy>
<a id="f-ca24d1670e8c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ca24d1670e8c-expression_statement-3">
afterEach(cleanup);
</storycode:copy>
<a id="f-ca24d1670e8c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-ca24d1670e8c-expression_statement-4">
describe("the short homepage", () => {
  it("identifies downloadable software before the live example", () => {
    const { container } = render(<LandingView />);
    const header = container.querySelector("header")!;
    expect(header.textContent).toContain("Downloadable software");
    expect(header.textContent).toContain("Runs on your machine");
    expect(screen.getByRole("link", { name: "Get the desktop app" }).getAttribute("href")).toBe("#download");
    expect(container.querySelectorAll("section")).toHaveLength(2);
    expect(screen.getByRole("link", { name: "Inspect the agent workflow" }).getAttribute("href")).toBe("#watch-workflow");
    expect(screen.getByRole("link", { name: "Current desktop prerelease downloads" }).getAttribute("href")).toContain("/releases/tag/unstable");
    expect(screen.queryByText("Public downloads coming soon.")).toBeNull();
    expect(screen.queryByText("Three things that should have been one")).toBeNull();
  });
  it("reports the visit once per load", () => {
    const { rerender } = render(<LandingView />); rerender(<LandingView />);
    expect(captured.filter((event) => event === "landing_viewed")).toHaveLength(1);
  });
});
</storycode:copy>

### Assemble `apps/web/src/views/LandingView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/LandingView.test.tsx"><storycode:paste select="#f-ca24d1670e8c-expression_statement"/><storycode:paste select="#f-ca24d1670e8c-captured"/><storycode:paste select="#f-ca24d1670e8c-expression_statement-2"/><storycode:paste select="#f-ca24d1670e8c-expression_statement-3"/><storycode:paste select="#f-ca24d1670e8c-expression_statement-4"/></storycode:file>


</storycode:doc>
