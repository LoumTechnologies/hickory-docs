<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Discovery Landing Page Is Organized By Interest, Never By Audience Label

For: Prospective users checking what the homepage demonstrates and promises.

The original document explicitly retires this homepage design. Its current successor is landing/homepage-opens-paused.md. Retaining old implementation bytes does not reinstate that page contract.

This is historical product intent. Preserved source files below remain part of byte-exact construction and may serve other current behaviors.

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/landing/discovery-page-is-interest-organized.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: The original document explicitly retires this homepage design. Its current successor is landing/homepage-opens-paused.md. Retaining old implementation bytes does not reinstate that page contract.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-eec2f8a321cc"></a>
## `apps/web/src/router.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-eec2f8a321cc-Route"></a>
### `Route`

<storycode:copy id="f-eec2f8a321cc-Route">
import { useEffect, useState } from "react";

// Tiny hash router — deliberately no routing dependency.

export type Route =
  // The front door ("/"): opens the unsaved introduction. Never a chooser —
  // the app opens like an editor, in a document you can type into.
  | { name: "landing" }
  // File → New Window: a deliberate window with no folder or document open.
  | { name: "blank" }
  // A document that does not exist yet: an unsaved draft until Save names it.
  | { name: "new"; introduction?: boolean; file?: string }
  // The whole pipeline at once, one column per stage. Project-scoped because
  // a chain crosses documents, and a doc-scoped route could only ever show
  // one link of it.
  | { name: "lineage"; id: string }
  // LLM API keys for the agent — the only configuration this app has.
  | { name: "settings" }
  // A place to type a thought before it has a name or a file.
  | { name: "scratchpad" }
  | { name: "doc"; id: string };

</storycode:copy>
<a id="f-eec2f8a321cc-parseRoute"></a>
### `parseRoute`

<storycode:copy id="f-eec2f8a321cc-parseRoute">
export function parseRoute(hash: string): Route {
  const path = hash.replace(/^#/, "") || "/";
  let m: RegExpMatchArray | null;
  if ((m = path.match(/^\/projects\/([^/]+)\/lineage$/)))
    return { name: "lineage", id: decodeURIComponent(m[1]) };
  if ((m = path.match(/^\/docs\/([^/]+)$/))) return { name: "doc", id: m[1] };
  if (path.startsWith("/file/")) return { name: "new", file: decodeURIComponent(path.slice(6)) };
  if (path === "/new") return { name: "new" };
  if (path === "/blank") return { name: "blank" };
  if (path === "/settings") return { name: "settings" };
  if (path === "/scratchpad") return { name: "scratchpad" };
  // "/" and anything unrecognised — including a retired route like the old
  // "#/documents" list still living in a bookmark — lands the way the app
  // always lands: in a document. The folder's files are a pane, not a page.
  return { name: "landing" };
}

</storycode:copy>
<a id="f-eec2f8a321cc-navigate"></a>
### `navigate`

<storycode:copy id="f-eec2f8a321cc-navigate">
export function navigate(path: string) {
  location.hash = path;
}

</storycode:copy>
<a id="f-eec2f8a321cc-NEW_DOCUMENT_EVENT"></a>
### `NEW_DOCUMENT_EVENT`

Fired on `window` whenever somebody asks for a new document.

<storycode:copy id="f-eec2f8a321cc-NEW_DOCUMENT_EVENT">
/** Fired on `window` whenever somebody asks for a new document. */
export const NEW_DOCUMENT_EVENT = "hickory-new-document";

</storycode:copy>
<a id="f-eec2f8a321cc-newDocument"></a>
### `newDocument`

Ask for a new document: File → New, the tree's +, the welcome verb.
Not just `navigate("/new")`. The hash is a place, and setting it to where
we already are is a no-op — so after the first Untitled tab was opened and
closed, every further New Document did nothing at all, because the address
still said `#/new`. The event is the act; the route stays so a deep link
and a restart still land on an untitled buffer.

<storycode:copy id="f-eec2f8a321cc-newDocument">
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
  if (parseRoute(location.hash).name !== "new") redirect("/new");
}

</storycode:copy>
<a id="f-eec2f8a321cc-redirect"></a>
### `redirect`

Navigate without a history entry — for hops the Back button must not
revisit (the landing decision, the untitled buffer becoming a real doc).

<storycode:copy id="f-eec2f8a321cc-redirect">
/** Navigate without a history entry — for hops the Back button must not
 * revisit (the landing decision, the untitled buffer becoming a real doc). */
export function redirect(path: string) {
  location.replace(`#${path}`);
}

</storycode:copy>
<a id="f-eec2f8a321cc-useRoute"></a>
### `useRoute`

<storycode:copy id="f-eec2f8a321cc-useRoute">
export function useRoute(): Route {
  const [route, setRoute] = useState<Route>(() => parseRoute(location.hash));
  useEffect(() => {
    const onChange = () => setRoute(parseRoute(location.hash));
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return route;
}
</storycode:copy>

### Assemble `apps/web/src/router.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/router.ts"><storycode:paste select="#f-eec2f8a321cc-Route"/><storycode:paste select="#f-eec2f8a321cc-parseRoute"/><storycode:paste select="#f-eec2f8a321cc-navigate"/><storycode:paste select="#f-eec2f8a321cc-NEW_DOCUMENT_EVENT"/><storycode:paste select="#f-eec2f8a321cc-newDocument"/><storycode:paste select="#f-eec2f8a321cc-redirect"/><storycode:paste select="#f-eec2f8a321cc-useRoute"/></storycode:file>

## Implementation review

Historical contract. No separate implementation review was recorded.

</storycode:doc>
