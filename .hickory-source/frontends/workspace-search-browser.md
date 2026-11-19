<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Find saved text across the workspace

For maintainers changing workspace services independently of the editor renderer.

Search results are snapshots of saved workspace text. Keep every occurrence and its byte column, including generated-file ownership. Convert a byte column only at a Unicode scalar boundary; editors use UTF-16, so a character before the match may occupy two units. A result may reveal a precise position only after the open buffer matches the reported source hash. No search edits or saves source. Submit a query through the existing exhaustive find endpoint, with literal/regex, case and whole-word options. Superseded requests cannot publish results or navigate. Hand the match to the tab owner before reading its bytes, so a later tab selection can retire the read, hash check, and reveal together. Pagination bounds rendered rows without silently treating a capped answer as complete. The overlay is fixed; its dismissal restores editor focus.

<ui:copy id="simple-browser-workspace-search">
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { api } from "../api/client";
import type { FileNode, FindOptions, FindResponse } from "../api/types";
import type { Buffer, Buffers } from "./buffers";
import { sha256 } from "../lib/reviews";
import { searchCharacter } from "./Intelligence";
export function WorkspaceSearch({visible, close, buffers, files, navigate}: {
  visible: boolean; close: (restoreFocus: boolean) => void; buffers: Buffers; files: FileNode[];
  navigate: (file: FileNode, line: number, character: number, current: () => boolean, verify: (buffer: Buffer) => Promise<void>) => Promise<boolean>;
}) {
  const [query, setQuery] = useState(""), [options, setOptions] = useState<FindOptions>({});
  const [result, setResult] = useState<FindResponse | null>(null), [notice, setNotice] = useState(""), [page, setPage] = useState(0);
  const ticket = useRef(0), input = useRef<HTMLInputElement>(null);
  const invalidate = () => { ticket.current++; setResult(null); setNotice(""); setPage(0); };
  useEffect(() => { if (visible) { ticket.current++; input.current?.focus(); } }, [visible]);
  useEffect(() => () => { ticket.current++; }, []);
  const hits = result?.files.flatMap(file => file.matches.flatMap(match => match.at.map(at => ({ file, match, at })))) ?? [];
  async function run() {
    const current = ++ticket.current; setResult(null); setNotice("Searching saved workspace text…"); setPage(0);
    try { const answer = await api.find(query, options); if (current === ticket.current) { setResult(answer); setNotice(""); } }
    catch (error) { if (current === ticket.current) setNotice(`${String(error)} Check the query and retry.`); }
  }
  async function reveal(hit: typeof hits[number]) {
    const current = ++ticket.current;
    try {
      function find(nodes: FileNode[]): FileNode | undefined { for (const node of nodes) { if (node.path === hit.file.path) return node; const child = find(node.children ?? []); if (child) return child; } }
      const file = buffers.values.get(hit.file.path)?.file ?? find(files);
      if (!file || file.dir) throw new Error("File no longer listed. Refresh files and search again.");
      const character = searchCharacter(hit.match.text, hit.at.column);
      const stale = () => new Error("Search source changed or its revision is unknown. Save edits and search again.");
      if (character === null) throw stale();
      if (await navigate(file, hit.match.line, character, () => current === ticket.current, async buffer => {
        const source = buffer.text, hash = await sha256(source);
        if (!hit.file.source_hash || hash !== hit.file.source_hash.replace(/^sha256:/, "").toLowerCase()
          || buffer.text !== source || source.split("\n")[hit.match.line - 1]?.replace(/\r$/, "") !== hit.match.text.replace(/\r$/, "")) throw stale();
      })) close(false);
    } catch (error) { if (current === ticket.current) setNotice(String(error)); }
  }
  if (!visible) return null;
  const dismiss = () => { ticket.current++; close(true); };
  const last = Math.max(0, Math.ceil(hits.length / 100) - 1), selected = Math.min(page, last);
  return createPortal(<div className="simple-workspace-problems" role="dialog" aria-label="Find in workspace" onKeyDown={event => { if (event.key === "Escape") { event.preventDefault(); dismiss(); } }}>
    <button aria-label="Close workspace search" onClick={dismiss}>×</button><strong>Find in workspace</strong>
    <p>Saved workspace text · unsaved edits are preserved</p>
    <form onSubmit={event => { event.preventDefault(); void run(); }}><input ref={input} aria-label="Workspace search query" value={query} onChange={event => { invalidate(); setQuery(event.target.value); }} />
      {([['regex', 'Regular expression'], ['case', 'Match case'], ['whole_word', 'Whole word']] as const).map(([key, label]) => <label key={key}><input type="checkbox" checked={!!options[key]} onChange={event => { invalidate(); setOptions({ ...options, [key]: event.target.checked }); }} />{label}</label>)}
      <button disabled={!query}>Search</button></form>
    {notice && <p role="status">{notice}</p>}{result && <p>{hits.length} occurrences{result.truncated ? " · Result cap reached; narrow the query" : ""}</p>}
    {hits.slice(selected * 100, (selected + 1) * 100).map((hit, index) => <div key={index}><button onClick={() => { void reveal(hit); }}>{hit.file.path}:{hit.match.line} · {hit.match.text}</button>{hit.file.generated_by && <small>Generated by {hit.file.generated_by} · read-only</small>}</div>)}
    {last > 0 && <div><button disabled={!selected} onClick={() => setPage(selected - 1)}>Previous results</button><span>{selected + 1}/{last + 1}</span><button disabled={selected === last} onClick={() => setPage(selected + 1)}>Next results</button></div>}
  </div>, document.body);
}
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceSearch.tsx"><ui:paste select="#simple-browser-workspace-search"/></ui:file>
</ui:doc>
