import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { Buffers } from "./buffers";
import { nextWorkspaceProblem, type SemanticLocation, type WorkspaceProblem } from "./Intelligence";
import { useWorkspaceProblems } from "./WorkspaceLanguages";
import { openLocation } from "../lib/revealLine";
const NEXT_PROBLEM = "hickory:simple:next-problem";
export function requestProblemNavigation(direction: -1 | 1) { window.dispatchEvent(new CustomEvent(NEXT_PROBLEM, { detail: direction })); }
export function WorkspaceProblems({ buffers, visible, setVisible, current, focusCurrent }: { buffers: Buffers; visible: boolean; setVisible: (value: boolean) => void; current: () => SemanticLocation | null; focusCurrent: () => void }) {
  const close = () => { setVisible(false); focusCurrent(); };
  const problems = useWorkspaceProblems(buffers), panel = useRef<HTMLDivElement>(null);
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
      <input aria-label="Filter problems" value={query} onChange={event => { setQuery(event.target.value); setPage(0); }} />
      {notice && <p role="status">{notice}</p>}
      {!matching.length && <p>No reported problems match. This is provider information, not a verification result.</p>}
      {matching.slice(selectedPage * 100, (selectedPage + 1) * 100).map(problem => <div key={problem.id}>
        <button type="button" disabled={!problem.current} onClick={() => open(problem)}>{problem.path}{problem.location ? `:${problem.location.range.start.line + 1}` : ""} · {problem.message}</button>
        {!problem.current && <small>Source location stale or unversioned. <button type="button" onClick={() => { openLocation(problem.path, 1); setVisible(false); }}>Open file</button></small>}
      </div>)}
      {last > 0 && <div><button type="button" disabled={selectedPage === 0} onClick={() => setPage(selectedPage - 1)}>Previous results</button><span>{selectedPage + 1}/{last + 1}</span><button type="button" disabled={selectedPage === last} onClick={() => setPage(selectedPage + 1)}>Next results</button></div>}
    </div>, document.body)}</>;
}
