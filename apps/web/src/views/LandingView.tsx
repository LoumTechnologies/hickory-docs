import { useEffect } from "react";
import { emit } from "../analytics/events";
import { TooltipLayer } from "../components/TooltipLayer";
import { BrowserDebugDemo } from "../landing/demos/BrowserDebugDemo";
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
      <p className="landing-sub">A notes IDE. Prose and code, together in a Markdown file.</p>
      <a className="btn" href="#download">Get the desktop app</a>
    </header>
    <BrowserDebugDemo />
    <footer className="landing-download" id="download">
      <h2>Download Hickory Docs</h2>
      <p>The desktop app works with your own files and tools. No account required.</p>
      <p className="muted">Public downloads coming soon.</p>
    </footer>
    <TooltipLayer />
  </main>;
}
