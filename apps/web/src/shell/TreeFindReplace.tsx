// Folder find and replace lives in a modal: opening options and results
// must not push the file list away from the row the reader was using.
import { useCallback, useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField } from "../lib/uiActions";

import { api } from "../api/client";
import type { FindFile, FindOptions, ReplaceResponse } from "../api/types";

export function TreeFindReplace({
  onOpenHit,
  onReplaced,
}: {
  /** Jump to a match: path, and the 1-based line it is on. */
  onOpenHit: (path: string, line: number) => void;
  /** A write happened — the tree and any open buffer should refresh. */
  onReplaced: (result: ReplaceResponse) => void;
}) {
  const instance = useId(); const parent = `folder-find:${instance}`;
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [replacement, setReplacement] = useState("");
  const [options, setOptions] = useState<FindOptions>({});
  const [result, setResult] = useState<{ files: FindFile[]; truncated: boolean } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<ReplaceResponse | null>(null);
  const [reload, setReload] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLDialogElement>(null);
  const searchButton = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (open) { if (!dialogRef.current?.open) dialogRef.current?.showModal(); inputRef.current?.focus(); }
    else if (dialogRef.current?.open) dialogRef.current.close();
  }, [open]);

  // A keystroke is not a question — a pause is. Guarded against a slow
  // earlier answer landing after a faster later one.
  useEffect(() => {
    if (!open) return;
    const q = query;
    if (!q) {
      setResult(null);
      setError(null);
      return;
    }
    let live = true;
    setResult(null);
    const timer = window.setTimeout(() => {
      api.find(q, options).then(
        (found) => {
          if (!live) return;
          setResult(found);
          setError(null);
        },
        (e: unknown) => {
          if (!live) return;
          setResult(null);
          setError(e instanceof Error ? e.message : String(e));
        },
      );
    }, 250);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [open, query, options, reload]);

  const writable = (result?.files ?? []).filter((f) => !f.generated_by);
  const total = (result?.files ?? []).reduce(
    (n, f) => n + f.matches.reduce((m, line) => m + line.at.length, 0),
    0,
  );

  const sourceGuardAvailable = writable.length > 0 && writable.every(file => typeof file.source_hash === "string" && /^(sha256:)?[a-f0-9]{64}$/.test(file.source_hash));
  const runReplace = useCallback(async (requireSource = false) => {
    if (!query || writable.length === 0 || busy) return { error: "Find writable matches and wait for the current operation" };
    if (result?.truncated) return { error: "Narrow the query until the search response is complete before replacing" };
    if (requireSource && !sourceGuardAvailable) return { error: "The current host did not supply source identities. Refresh search with an updated host before replacing" };
    setBusy(true); setReport(null);
    try {
      const done = await api.replaceAll(query, replacement, options, writable.map(file => file.path), sourceGuardAvailable ? Object.fromEntries(writable.map(file => [file.path, file.source_hash!])) : undefined);
      setReport(done); onReplaced(done); setResult(null); setReload(value => value + 1);
      return { result: done };
    } catch (e) { const error = e instanceof Error ? e.message : String(e); setError(error); return { error }; }
    finally { setBusy(false); }
  }, [query, replacement, options, writable, onReplaced, busy, sourceGuardAvailable, result?.truncated]);
  const revision = useUiRevision(() => [open, query, replacement, options, result, error, busy, report]);
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], enabled = true, group = parent, reason?: string) {
    commands.push({ key: `${parent}/${id}`, command: { label, parent: group, subject: parent, parameters, prerequisites: ["Search matches are retained observations, not complete edit baselines"], read: () => ({ revision: revision(), enabled, reason }), execute } });
  }
  add("open", "Set folder find visibility", { open: { type: "boolean" } }, async args => { setOpen(Boolean(args.open)); return { status: "completed", value: { open: Boolean(args.open) } }; }, true, "folder-find-disclosures");
  if (open) {
    add("query", "Set folder search query and options", { query: { type: "string", maxLength: 8192 }, case: { type: "boolean" }, whole_word: { type: "boolean" }, regex: { type: "boolean" } }, async args => { setQuery(String(args.query)); setOptions({ case: Boolean(args.case), whole_word: Boolean(args.whole_word), regex: Boolean(args.regex) }); return { status: "completed", value: { dispatch: "search requested", result: "Read the actual asynchronous search response or error" } }; }, !busy);
    add("replacement", "Set folder replacement draft", { text: { type: "string", maxLength: 32768 } }, async args => { setReplacement(String(args.text)); return { status: "completed", value: { applied: false } }; }, !busy);
    add("read", "Read folder search state bytes", { offset: { type: "number", minimum: 0, maximum: 100000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ query, replacement, options, result, error, busy, report, edit_baseline: false, interpretation: "A null result has no current search response; inspect errors and truncation" }));
      const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current search state" };
      return { status: "completed", value: { encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], total_bytes: bytes.length, next: end < bytes.length ? end : null } };
    });
    add("hit", "Open observed folder search hit", { path: { type: "string", maxLength: 8192 }, line: { type: "number", minimum: 1, maximum: 100000000, integer: true } }, async args => {
      const file = result?.files.find(file => file.path === args.path);
      if (!file?.matches.some(match => match.line === args.line)) return { status: "unavailable", reason: "Hit does not belong to this observed search response" };
      onOpenHit(file.path, Number(args.line)); return { status: "completed", value: { dispatch: "navigation requested", path: file.path, line: args.line } };
    }, !!result);
    add("replace", "Replace folder matches", {}, async () => {
      const outcome = await runReplace(true);
      return outcome.result ? { status: "completed", value: { replacements: outcome.result.replacements, changed_files: outcome.result.changed.length, skipped_files: outcome.result.skipped.length, result: "Read the actual per-file report; skipped source changes and generated files remain unmodified" } } : { status: "unavailable", reason: outcome.error ?? "Replacement unavailable" };
    }, !busy && !!query && sourceGuardAvailable && !result?.truncated, parent, "Find complete fresh writable matches with source identities; narrow truncated results");
  }
  useUiCommands(commands);

  return <>
    <div style={{ display: "flex", justifyContent: "flex-end", height: 28 }}><button ref={searchButton} type="button" aria-label="Search files" title="Search files" onClick={() => setOpen(true)} style={{ background: "transparent", border: 0, color: "var(--fg)", cursor: "pointer" }}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" aria-hidden="true"><circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/></svg>
    </button></div>
    {open && createPortal(<dialog ref={dialogRef} className="tree-find" aria-label="Search files" onCancel={() => setOpen(false)} onClose={() => { setOpen(false); searchButton.current?.focus(); }} style={{ width: "min(680px, 85vw)", maxHeight: "80vh", overflow: "auto", background: "var(--bg-raised)", color: "var(--fg)", border: "1px solid var(--border)", borderRadius: 8 }}>
      <div className="tree-find__fields">
        <input
          ref={inputRef}
          className="tree-find__input"
          type="search"
          placeholder="Find"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") setOpen(false);
          }}
        />
        <input
          className="tree-find__input"
          type="text"
          placeholder="Replace with"
          value={replacement}
          onChange={(e) => setReplacement(e.target.value)}
        />
        <button
          type="button"
          className="tree-find__close"
          aria-label="Close find"
          data-tip="Close"
          onClick={() => setOpen(false)}
        >
          ×
        </button>
      </div>

      <div className="tree-find__options" role="group" aria-label="Match options">
        {(
          [
            ["case", "Aa", "Match case"],
            ["whole_word", "ab|", "Whole word"],
            ["regex", ".*", "Regular expression"],
          ] as const
        ).map(([key, glyph, label]) => (
          <button
            key={key}
            type="button"
            className={`tree-find__toggle${options[key] ? " tree-find__toggle--on" : ""}`}
            aria-pressed={options[key] === true}
            aria-label={label}
            data-tip={label}
            onClick={() => setOptions((current) => ({ ...current, [key]: !current[key] }))}
          >
            {glyph}
          </button>
        ))}
      </div>

      {error && (
        <p className="error tree-find__error" role="alert">
          {error}
        </p>
      )}

      {result && (
        <p className="tree-find__summary" role="status">
          {total === 0
            ? "No matches."
            : `${total} match${total === 1 ? "" : "es"} in ${result.files.length} file${
                result.files.length === 1 ? "" : "s"
              }${result.truncated ? " — and more; narrow the pattern." : ""}`}
        </p>
      )}

      {result && total > 0 && (
        <div className="tree-find__actions">
          <button
            type="button"
            className="btn btn-small btn-primary"
            disabled={busy || writable.length === 0 || result?.truncated}
            onClick={() => void runReplace()}
            data-tip={
              writable.length === 0
                ? "Every match is in a generated file — change the document instead"
                : "Rewrite every match in the files that can be written"
            }
          >
            {busy ? "Replacing…" : `Replace in ${writable.length} file${writable.length === 1 ? "" : "s"}`}
          </button>
        </div>
      )}

      {report && (
        <p className="tree-find__summary" role="status">
          Replaced {report.replacements} in {report.changed.length} file
          {report.changed.length === 1 ? "" : "s"}
          {report.skipped.length > 0 &&
            `; skipped ${report.skipped.length} generated file${report.skipped.length === 1 ? "" : "s"}`}
          .
        </p>
      )}

      <ul className="tree-find__files">
        {(result?.files ?? []).map((file) => (
          <li key={file.path} className={file.generated_by ? "tree-find__file--generated" : undefined}>
            <p className="tree-find__path mono">
              {file.path}
              {file.generated_by && (
                <span
                  className="tree-find__generated"
                  data-tip="Written by a document — replace would be undone by the next weave. Change the document instead."
                >
                  generated
                </span>
              )}
            </p>
            <ul>
              {file.matches.map((match) => (
                <li key={match.line}>
                  <button
                    type="button"
                    className="tree-find__hit mono"
                    onClick={() => onOpenHit(file.path, match.line)}
                  >
                    <span className="tree-find__line">{match.line}</span>
                    <span className="tree-find__text">{match.text.trim()}</span>
                  </button>
                </li>
              ))}
            </ul>
          </li>
        ))}
      </ul>
    </dialog>, document.body)}
  </>;
}
