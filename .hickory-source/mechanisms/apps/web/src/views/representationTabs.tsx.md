<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/representationTabs.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-15e807c41782"></a>
## `apps/web/src/views/representationTabs.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-15e807c41782-readingSpace"></a>
### `readingSpace`

<storycode:copy id="f-15e807c41782-readingSpace">
import { useCallback, useEffect, type Dispatch, type SetStateAction } from "react";
import { open, panes, split, tab, type Layout, type Node, type Tab } from "../shell/layout";
import { OPEN_READING } from "../lib/readingViews";
import { DocumentReading } from "./DocumentReading";
import { OPEN_REPRESENTATION } from "../lib/openRepresentation";
import { RepresentationLibrary, RepresentationPane } from "./RepresentationPane";
import { GitPane } from "./GitPane";
import { HistoryLens } from "./HistoryLens";
import { GIT_TAB, STORY_TAB } from "./workspaceState";

function readingSpace(node: Node, fresh: string): Node {
  if (node.type === "pane") return node;
  if (node.children.some(child => child.id === fresh)) return { ...node, sizes: [0.3, 0.7] };
  return { ...node, children: node.children.map(child => readingSpace(child, fresh)) };
}

</storycode:copy>
<a id="f-15e807c41782-useRepresentationTabs"></a>
### `useRepresentationTabs`

<storycode:copy id="f-15e807c41782-useRepresentationTabs">
export function useRepresentationTabs(setLayout: Dispatch<SetStateAction<Layout>>) {
  useEffect(() => {
    const receive = (event: Event) => {
      const { id, base, target } = (event as CustomEvent<{ id:string; base?:string; target?:string }>).detail;
      const query = new URLSearchParams({id, ...(base ? {base} : {}), ...(target ? {target} : {})});
      setLayout(current => open(current, tab("tool", `literate:${query}`, "Literate view")));
    };
    const reading = (event: Event) => {
      const { id, title } = (event as CustomEvent<{ id: string; title: string }>).detail;
      setLayout(current => {
        const all = panes(current.root);
        const available = all.find(pane => pane.tabs.length === 0 || pane.tabs[pane.active]?.target.startsWith("reading:"));
        // A reading needs the document's measure. Keep the agent beside it,
        // rather than splitting the narrow conversation pane underneath itself.
        const document = all.find(pane => ["document", "untitled", "file", "generated"].includes(pane.tabs[pane.active]?.kind));
        const beside = available ? current : split(current, document?.id ?? current.focus, "column");
        const roomy = available ? beside : { ...beside, root: readingSpace(beside.root, beside.focus) };
        return open(roomy, tab("tool", `reading:${id}`, title), available?.id ?? roomy.focus);
      });
    };
    window.addEventListener(OPEN_REPRESENTATION, receive);
    window.addEventListener(OPEN_READING, reading);
    return () => { window.removeEventListener(OPEN_REPRESENTATION, receive); window.removeEventListener(OPEN_READING, reading); };
  }, [setLayout]);
  return useCallback(() => setLayout(current => open(current, tab("tool", "literate", "Literate views"))), [setLayout]);
}
</storycode:copy>
<a id="f-15e807c41782-reviewTool"></a>
### `reviewTool`

<storycode:copy id="f-15e807c41782-reviewTool">
export function reviewTool(tab: Tab, onOpenFile: (path:string)=>void, onOpenStory:()=>void, onOpenLiterate:()=>void) {
  if (tab.kind !== "tool") return null;
  if (tab.target.startsWith("reading:")) return <DocumentReading key={tab.target} id={tab.target.slice(8)} />;
  if (tab.target === GIT_TAB) return <GitPane {...{onOpenFile,onOpenStory,onOpenLiterate}} />;
  if (tab.target === STORY_TAB) return <HistoryLens />;
  if (tab.target === "literate") return <RepresentationLibrary />;
  if (tab.target.startsWith("literate:")) {
    const query=new URLSearchParams(tab.target.slice(9));
    return <RepresentationPane key={query.get("id")} id={query.get("id")!} bufferId={tab.id} initialBase={query.get("base")??""} initialTarget={query.get("target")??""} />;
  }
  return null;
}
</storycode:copy>

### Assemble `apps/web/src/views/representationTabs.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/representationTabs.tsx"><storycode:paste select="#f-15e807c41782-readingSpace"/><storycode:paste select="#f-15e807c41782-useRepresentationTabs"/><storycode:paste select="#f-15e807c41782-reviewTool"/></storycode:file>

</storycode:doc>
