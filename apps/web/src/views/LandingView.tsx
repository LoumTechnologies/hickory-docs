import { UiLink } from "../components/UiLink";
import { config } from "../config";
import { WorkflowTour } from "../components/WorkflowTour";
import { useEffect } from "react";
import { emit } from "../analytics/events";
import { TooltipLayer } from "../components/TooltipLayer";
import { BrowserDebugDemo } from "../landing/demos/BrowserDebugDemo";
import { WorkflowRecording } from "../landing/WorkflowRecording";
import "../landing/homepage.css";

let viewedFired = false;
export function resetViewedForTest() { viewedFired = false; }

/** A first visit from a developer: see a document, try its debugger, download. */
export function LandingView() {
  useEffect(() => {
    if (viewedFired) return;
    viewedFired = true;
    emit({ name: "landing_viewed", path: location.hash || "#/" });
  }, []);
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
    <div id="try-workflow"><WorkflowTour surface="site" /></div>
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
