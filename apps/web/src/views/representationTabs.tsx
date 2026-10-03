import { useCallback, useEffect, type Dispatch, type SetStateAction } from "react";
import { open, tab, type Layout, type Tab } from "../shell/layout";
import { OPEN_REPRESENTATION } from "../lib/openRepresentation";
import { RepresentationLibrary, RepresentationPane } from "./RepresentationPane";
import { GitPane } from "./GitPane";
import { HistoryLens } from "./HistoryLens";
import { GIT_TAB, STORY_TAB } from "./workspaceState";

export function useRepresentationTabs(setLayout: Dispatch<SetStateAction<Layout>>) {
  useEffect(() => {
    const receive = (event: Event) => {
      const { id, base, target } = (event as CustomEvent<{ id:string; base?:string; target?:string }>).detail;
      const query = new URLSearchParams({id, ...(base ? {base} : {}), ...(target ? {target} : {})});
      setLayout(current => open(current, tab("tool", `literate:${query}`, "Literate view")));
    };
    window.addEventListener(OPEN_REPRESENTATION, receive);
    return () => window.removeEventListener(OPEN_REPRESENTATION, receive);
  }, [setLayout]);
  return useCallback(() => setLayout(current => open(current, tab("tool", "literate", "Literate views"))), [setLayout]);
}
export function reviewTool(tab: Tab, onOpenFile: (path:string)=>void, onOpenStory:()=>void, onOpenLiterate:()=>void) {
  if (tab.kind !== "tool") return null;
  if (tab.target === GIT_TAB) return <GitPane {...{onOpenFile,onOpenStory,onOpenLiterate}} />;
  if (tab.target === STORY_TAB) return <HistoryLens />;
  if (tab.target === "literate") return <RepresentationLibrary />;
  if (tab.target.startsWith("literate:")) {
    const query=new URLSearchParams(tab.target.slice(9));
    return <RepresentationPane key={query.get("id")} id={query.get("id")!} initialBase={query.get("base")??""} initialTarget={query.get("target")??""} />;
  }
  return null;
}
