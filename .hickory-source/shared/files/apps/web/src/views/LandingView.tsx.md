<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/LandingView.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Visit Is Reported Exactly Once Per Page Load](../../../../../../guarantees/analytics/landing-events-fire-once/contract.md)
- [Every Hover Hint Is Drawn In The App Theme](../../../../../../guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme/contract.md)
- [The engineer tour distinguishes replay from execution](../../../../../../guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution/contract.md)
- [Every Claim On The Home Page Is True Of The Downloaded Binary](../../../../../../guarantees/landing/home-page-claims-are-true-of-the-binary/contract.md)
- [Browser-local prediction practice uses the shared gate](../../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md)

Captured SHA-256: `863ae5ff99fb129f66ba01271709e201a30b708e47af4405aa34f1dd28d5091c`.

<book:entry path="apps/web/src/views/LandingView.tsx">
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
export function resetViewedForTest() { viewedFired = false; }

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
      {config.sourceRevision && <p className="muted">Build source: <UiLink href={`https://github.com/LoumTechnologies/hickory-docs/tree/${config.sourceRevision}`}>{config.sourceRevision.slice(0, 7)}</UiLink></p>}
    </footer>
    <TooltipLayer />
  </main>;
}
</book:entry>

</book:doc>
