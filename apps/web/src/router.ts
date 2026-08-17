import { useEffect, useState } from "react";

// Tiny hash router — deliberately no routing dependency.

export type Route =
  // The front door ("/"): decides which document to land in. Never a chooser —
  // the app opens like an editor, in a document you can type into.
  | { name: "landing" }
  // A document that does not exist yet: held in memory, created on first edit.
  | { name: "new" }
  // The whole pipeline at once, one column per stage. Project-scoped because
  // a chain crosses documents, and a doc-scoped route could only ever show
  // one link of it.
  | { name: "lineage"; id: string }
  // LLM API keys for the agent — the only configuration this app has.
  | { name: "settings" }
  | { name: "doc"; id: string };

export function parseRoute(hash: string): Route {
  const path = hash.replace(/^#/, "") || "/";
  let m: RegExpMatchArray | null;
  if ((m = path.match(/^\/projects\/([^/]+)\/lineage$/)))
    return { name: "lineage", id: decodeURIComponent(m[1]) };
  if ((m = path.match(/^\/docs\/([^/]+)$/))) return { name: "doc", id: m[1] };
  if (path === "/new") return { name: "new" };
  if (path === "/settings") return { name: "settings" };
  // "/" and anything unrecognised — including a retired route like the old
  // "#/documents" list still living in a bookmark — lands the way the app
  // always lands: in a document. The folder's files are a pane, not a page.
  return { name: "landing" };
}

export function navigate(path: string) {
  location.hash = path;
}

/** Navigate without a history entry — for hops the Back button must not
 * revisit (the landing decision, the untitled buffer becoming a real doc). */
export function redirect(path: string) {
  location.replace(`#${path}`);
}

export function useRoute(): Route {
  const [route, setRoute] = useState<Route>(() => parseRoute(location.hash));
  useEffect(() => {
    const onChange = () => setRoute(parseRoute(location.hash));
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return route;
}
