// Search open files and the folder, opened with Mod-Shift-F from anywhere in the shell.
//
// Picking a hit uses the workspace's existing open-and-reveal routing.

import { useEffect, useRef, useState } from "react";

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

  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setSelected((current) => Math.min(current + 1, Math.max(0, hits.length - 1)));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setSelected((current) => Math.max(current - 1, 0));
      return;
    }
    if (event.key === "Enter" && hits[selected]) {
      event.preventDefault();
      pick(hits[selected]);
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
        onChange={(event) => setQuery(event.target.value)}
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
                  onMouseEnter={() => setSelected(index)}
                  onClick={() => pick(hit)}
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
