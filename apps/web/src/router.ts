import { useEffect, useState } from "react";

// Tiny hash router — deliberately no routing dependency.

export type Route =
  // The front door ("/"): decides which document to land in. Never a chooser —
  // the app opens like an editor, in a document you can type into.
  | { name: "landing" }
  // File → New Window: a deliberate window with no folder or document open.
  | { name: "blank" }
  // A document that does not exist yet: an unsaved draft until Save names it.
  | { name: "new" }
  // The whole pipeline at once, one column per stage. Project-scoped because
  // a chain crosses documents, and a doc-scoped route could only ever show
  // one link of it.
  | { name: "lineage"; id: string }
  // LLM API keys for the agent — the only configuration this app has.
  | { name: "settings" }
  // A place to type a thought before it has a name or a file.
  | { name: "scratchpad" }
  | { name: "doc"; id: string };

export function parseRoute(hash: string): Route {
  const path = hash.replace(/^#/, "") || "/";
  let m: RegExpMatchArray | null;
  if ((m = path.match(/^\/projects\/([^/]+)\/lineage$/)))
    return { name: "lineage", id: decodeURIComponent(m[1]) };
  if ((m = path.match(/^\/docs\/([^/]+)$/))) return { name: "doc", id: m[1] };
  if (path === "/new") return { name: "new" };
  if (path === "/blank") return { name: "blank" };
  if (path === "/settings") return { name: "settings" };
  if (path === "/scratchpad") return { name: "scratchpad" };
  // "/" and anything unrecognised — including a retired route like the old
  // "#/documents" list still living in a bookmark — lands the way the app
  // always lands: in a document. The folder's files are a pane, not a page.
  return { name: "landing" };
}

export function navigate(path: string) {
  location.hash = path;
}

/** Fired on `window` whenever somebody asks for a new document. */
export const NEW_DOCUMENT_EVENT = "hickory-new-document";

/**
 * Ask for a new document: File → New, the tree's +, the welcome verb.
 *
 * Not just `navigate("/new")`. The hash is a place, and setting it to where
 * we already are is a no-op — so after the first Untitled tab was opened and
 * closed, every further New Document did nothing at all, because the address
 * still said `#/new`. The event is the act; the route stays so a deep link
 * and a restart still land on an untitled buffer.
 */
export function newDocument() {
  window.dispatchEvent(new CustomEvent(NEW_DOCUMENT_EVENT));
  navigate("/new");
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
