<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/SearchPanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7dbf0adaf494"></a>
## `apps/web/src/components/SearchPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7dbf0adaf494-SearchPanel"></a>
### `SearchPanel`

Search open files and the folder, opened with Mod-Shift-F from anywhere in the shell.
Picking a hit uses the workspace's existing open-and-reveal routing.

<storycode:copy id="f-7dbf0adaf494-SearchPanel">
// Search open files and the folder, opened with Mod-Shift-F from anywhere in the shell.
//
// Picking a hit uses the workspace's existing open-and-reveal routing.

import { useEffect, useId, useRef, useState } from "react";

import { useUiListActions } from "../lib/uiListActions";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { SearchHit, SearchResponse } from "../api/types";

export function SearchPanel({
  search,
  folderOpen,
  onNavigate,
  onClose,
}: {
  search: (query: string, limit: number) => Promise<SearchResponse>;
  folderOpen: boolean;
  onNavigate: (hit: SearchHit) => void;
  onClose: () => void;
}) {
  const [query, setQuery] = useState("");
  const [result, setResult] = useState<SearchResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // A keystroke is not a question — a pause is. Debounced, and guarded
  // against a slow earlier answer landing after a faster later one.
  useEffect(() => {
    const q = query.trim();
    if (!q) {
      setResult(null);
      setError(null);
      return;
    }
    let live = true;
    const timer = window.setTimeout(() => {
      search(q, 20).then(
        (r) => {
          if (!live) return;
          setResult(r);
          setSelected(0);
          setError(null);
        },
        (e) => live && setError(e instanceof Error ? e.message : String(e)),
      );
    }, 250);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [query, search]);

  const hits = result?.hits ?? [];

  // Arrow keys move the selection while focus stays in the input, so the
  // query is still editable mid-scan.
  useEffect(() => {
    const item = listRef.current?.children[selected];
    item?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  const pick = (hit: SearchHit) => {
    onNavigate(hit);
    onClose();
  };

  const actions = useUiListActions("File search", hits, { query, error, semantic: result?.semantic ?? false, limit: 20, folder_open: folderOpen, coverage: "Current ranked search response; not exhaustive matches" }, pick, onClose, { index: selected, set: setSelected });
  const parent = `search-query:${useId()}`;
  const revision = useUiRevision(() => [query, folderOpen]);
  const invoke = useUiCommands([{ key: parent, command: {
    label: "Set ranked file search query", parent, subject: parent,
    parameters: { query: { type: "string", maxLength: 16384 } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }),
    execute: async args => { setQuery(args.query as string); setResult(null); setError(null); setSelected(0); return { status: "completed", value: { search_requested: true, completion: "Observe current search results or error after debounce" } }; },
  } }]);
  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      void actions.close();
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (hits.length) void actions.select(Math.min(selected + 1, hits.length - 1));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      if (hits.length) void actions.select(Math.max(selected - 1, 0));
      return;
    }
    if (event.key === "Enter" && hits[selected]) {
      event.preventDefault();
      void actions.pick(selected);
    }
  };

  return (
    <div className="search-panel" role="dialog" aria-label="File search" onKeyDown={onKeyDown}>
      <input
        ref={inputRef}
        className="search-panel__input"
        type="search"
        placeholder={folderOpen ? "Search open files and this folder" : "Search open files"}
        value={query}
        onChange={(event) => void invoke(parent, { query: event.target.value })}
        aria-label="Search query"
      />
      {folderOpen && result && !result.semantic && (
        <p className="search-panel__hint">
          Lexical matches only — run <code>hick search --install-model</code> in a terminal to add
          semantic ranking.
        </p>
      )}
      {error && (
        <p className="search-panel__status error" role="alert">
          {error}
        </p>
      )}
      {result && hits.length === 0 && !error && (
        <p className="search-panel__status">No files match.</p>
      )}
      {hits.length > 0 && (
        <ul ref={listRef} className="search-panel__results" role="listbox">
          {hits.map((hit, index) => {
            return (
              <li key={`${hit.path}:${hit.start_line}:${index}`}>
                <button
                  type="button"
                  className={`search-panel__hit${index === selected ? " on" : ""}`}
                  role="option"
                  aria-selected={index === selected}
                  onMouseEnter={() => void actions.select(index)}
                  onClick={() => void actions.pick(index)}
                >
                  <span className="search-panel__where">
                    <span className="search-panel__path mono">{hit.path}</span>
                    <span className="search-panel__lines">
                      {hit.start_line === hit.end_line
                        ? `line ${hit.start_line}`
                        : `lines ${hit.start_line}–${hit.end_line}`}
                    </span>
                  </span>
                  <span className="search-panel__snippet">{hit.snippet}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/SearchPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/SearchPanel.tsx"><storycode:paste select="#f-7dbf0adaf494-SearchPanel"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-e9f603b09f67"></a>
## `apps/web/src/components/SearchPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-e9f603b09f67-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e9f603b09f67-expression_statement">
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { SearchPanel } from "./SearchPanel";
import { searchWorkspace } from "../lib/workspaceSearch";

afterEach(cleanup);

</storycode:copy>
<a id="f-e9f603b09f67-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-e9f603b09f67-expression_statement-2">
it("finds and navigates an open file without a folder", async () => {
  const navigate = vi.fn();
  const close = vi.fn();
  const search = (query: string, limit: number) => searchWorkspace(query, limit, [
    { path: "/outside/open.txt", content: "heading\nunsaved invoice" },
  ]);
  render(<SearchPanel search={search} folderOpen={false} onNavigate={navigate} onClose={close} />);
  fireEvent.change(screen.getByRole("searchbox"), { target: { value: "invoice" } });
  const result = await screen.findByRole("option");
  expect(result.textContent).toContain("unsaved invoice");
  expect(screen.getByPlaceholderText("Search open files")).toBeTruthy();
  fireEvent.click(result);
  expect(navigate).toHaveBeenCalledWith(expect.objectContaining({ path: "/outside/open.txt", start_line: 2 }));
  expect(close).toHaveBeenCalledOnce();
});

</storycode:copy>
<a id="f-e9f603b09f67-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-e9f603b09f67-expression_statement-3">
it("includes both folder results and open files", async () => {
  const search = (query: string, limit: number) => searchWorkspace(query, limit, [
    { path: "/outside/open.txt", content: "invoice" },
  ], async () => ({ semantic: false, hits: [
    { path: "closed.md", start_line: 1, end_line: 1, score: 1, snippet: "folder invoice" },
  ] }));
  render(<SearchPanel search={search} folderOpen onNavigate={vi.fn()} onClose={vi.fn()} />);
  fireEvent.change(screen.getByRole("searchbox"), { target: { value: "invoice" } });
  await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(2));
  expect(screen.getByPlaceholderText("Search open files and this folder")).toBeTruthy();
});
</storycode:copy>

### Assemble `apps/web/src/components/SearchPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/SearchPanel.test.tsx"><storycode:paste select="#f-e9f603b09f67-expression_statement"/><storycode:paste select="#f-e9f603b09f67-expression_statement-2"/><storycode:paste select="#f-e9f603b09f67-expression_statement-3"/></storycode:file>


</storycode:doc>
