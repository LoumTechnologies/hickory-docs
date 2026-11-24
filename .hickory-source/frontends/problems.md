<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read provider reports without shrinking code

For maintainers changing workspace services independently of the editor renderer.

The existing status bar and Edit menu open a floating, paged Problems view. Messages remain readable when their positions are stale; only matching source snapshots enable precise navigation. Next/previous consume the same model as the list. An explicit source expectation survives asynchronous file opening and is checked again by the workspace.

The workspace total counts received messages. Show the active file's language
and report state separately. An absent session and a missing source-bound
report explain different gaps. Only a version-bound empty publication can say
that the provider reported zero problems for the current file text.

<ui:copy id="simple-browser-workspace-problems">
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { Buffer, Buffers } from "./buffers";
import { nextWorkspaceProblem, type SemanticLocation, type WorkspaceProblem } from "./Intelligence";
import { useWorkspaceLanguageLookup, useWorkspaceProblems } from "./WorkspaceLanguages";
import { openLocation } from "../lib/revealLine";
const NEXT_PROBLEM = "hickory:simple:next-problem";
export function requestProblemNavigation(direction: -1 | 1) { window.dispatchEvent(new CustomEvent(NEXT_PROBLEM, { detail: direction })); }
export function WorkspaceProblems({ buffers, source, visible, setVisible, current, focusCurrent }: { buffers: Buffers; source?: Buffer; visible: boolean; setVisible: (value: boolean) => void; current: () => SemanticLocation | null; focusCurrent: () => void }) {
  const close = () => { setVisible(false); focusCurrent(); };
  const problems = useWorkspaceProblems(buffers), panel = useRef<HTMLDivElement>(null);
  const language = useWorkspaceLanguageLookup();
  const session = source ? language(source) : null;
  const [query, setQuery] = useState(""), [page, setPage] = useState(0), [notice, setNotice] = useState("");
  const matching = problems.filter(problem => `${problem.path} ${problem.message}`.toLowerCase().includes(query.toLowerCase()));
  const last = Math.max(0, Math.ceil(matching.length / 100) - 1), selectedPage = Math.min(page, last);
  const open = (problem: WorkspaceProblem) => {
    const location = problem.location;
    if (!problem.current || !location || problem.source === null || buffers.values.get(problem.path)?.text !== problem.source) { setNotice("Problem source changed or its location is unversioned. Refresh diagnostics."); return; }
    openLocation(problem.path, location.range.start.line + 1, location.range.start.character, true, problem.source); setVisible(false);
  };
  useEffect(() => { if (visible) panel.current?.focus(); }, [visible]);
  useEffect(() => {
    const move = (event: Event) => {
      const direction = (event as CustomEvent).detail;
      if (direction !== -1 && direction !== 1) return;
      const target = nextWorkspaceProblem(problems, current(), direction);
      if (target) open(target); else { setNotice("No current source locations. Provider messages remain available below."); setVisible(true); }
    };
    window.addEventListener(NEXT_PROBLEM, move); return () => window.removeEventListener(NEXT_PROBLEM, move);
  }, [problems, current]);
  return <><button className="simple-problems-status" type="button" onClick={() => { if (visible) close(); else setVisible(true); }} aria-label={`Problems: ${problems.length} reported`}>Problems {problems.length}</button>{visible && createPortal(
    <div ref={panel} tabIndex={-1} className="simple-workspace-problems" role="dialog" aria-label="Workspace Problems" onKeyDown={event => { if (event.key === "Escape") { event.preventDefault(); close(); } }}>
      <button type="button" aria-label="Close Problems" onClick={close}>×</button>
      <strong>Problems</strong><p>Language-provider reports · {problems.filter(problem => problem.current).length} current locations</p>
      {source ? <p>
        <strong>{source.file.name} · {source.file.language ?? "Unclassified source"}</strong><br />
        {!session?.client || !session.ready ? "No language session is attached to this file."
          : problems.some(problem => problem.path === source.file.path && problem.current) ? "Provider locations match the current file text."
          : session.diagnostics.length === 0 && session.diagnosticSource === source.text ? "The provider reported zero problems for the current file text."
          : "No source-bound diagnostic report for the current file text."}
      </p> : <p>Open a source file to see its diagnostic status.</p>}
      <input aria-label="Filter problems" value={query} onChange={event => { setQuery(event.target.value); setPage(0); }} />
      {notice && <p role="status">{notice}</p>}
      {!matching.length && <p>{query ? "No received reports match the filter." : "No workspace problems have been reported."}</p>}
      {matching.slice(selectedPage * 100, (selectedPage + 1) * 100).map(problem => <div key={problem.id}>
        <button type="button" disabled={!problem.current} title={!problem.current ? "The diagnostic is stale. Refresh workspace problems before navigating." : undefined} onClick={() => open(problem)}>{problem.path}{problem.location ? `:${problem.location.range.start.line + 1}` : ""} · {problem.message}</button>
        {!problem.current && <small>Source location stale or unversioned. <button type="button" onClick={() => { openLocation(problem.path, 1); setVisible(false); }}>Open file</button></small>}
      </div>)}
      {last > 0 && <div><button type="button" disabled={selectedPage === 0} title={selectedPage === 0 ? "You’re on the first page." : undefined} onClick={() => setPage(selectedPage - 1)}>Previous results</button><span>{selectedPage + 1}/{last + 1}</span><button type="button" disabled={selectedPage === last} title={selectedPage === last ? "You’re on the last page." : undefined} onClick={() => setPage(selectedPage + 1)}>Next results</button></div>}
    </div>, document.body)}</>;
}
</ui:copy>

<ui:file path="apps/web/src/simple/Problems.tsx"><ui:paste select="#simple-browser-workspace-problems"/></ui:file>
</ui:doc>
