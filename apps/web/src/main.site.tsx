// Entry point for the **static marketing site** (`site.html`).
//
// It renders one page and mounts no API client: the site has no server to talk
// to, and the demos on it run entirely in the browser. The desktop app's entry
// is `main.tsx`; the two never load each other's code.
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { LandingView } from "./views/LandingView";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <LandingView />
  </StrictMode>,
);
