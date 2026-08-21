// Find and replace across the whole folder, at the top of the tree.
//
// It lives here rather than in an overlay because it is not a question you
// ask and dismiss — it is a list you work through, ticking files, while the
// tree beneath it stays where it was. The ranked search overlay (⌘⇧F) is the
// other thing, and stays: "where is the bit about invoices" and "rename this
// symbol in 40 files" are different jobs with different right answers.
//
// Two rules the UI exists to enforce:
//
//  - **A generated file is shown and refused.** Not hidden: its matches are
//    real and worth reading. But replace will not write it, because the next
//    weave would undo the edit — so the row says which document to change
//    instead, which is also the change that fixes every other copy.
//  - **Nothing is written until Replace is pressed.** The preview and the
//    write use the same matcher on the server (one pattern, built once), so
//    the list cannot describe a different edit from the one performed.

import { useCallback, useEffect, useRef, useState } from "react";

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
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [replacement, setReplacement] = useState("");
  const [options, setOptions] = useState<FindOptions>({});
  const [result, setResult] = useState<{ files: FindFile[]; truncated: boolean } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<ReplaceResponse | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) inputRef.current?.focus();
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
  }, [open, query, options]);

  const writable = (result?.files ?? []).filter((f) => !f.generated_by);
  const total = (result?.files ?? []).reduce(
    (n, f) => n + f.matches.reduce((m, line) => m + line.at.length, 0),
    0,
  );

  const runReplace = useCallback(() => {
    if (!query || writable.length === 0) return;
    setBusy(true);
    setReport(null);
    api
      .replaceAll(
        query,
        replacement,
        options,
        writable.map((f) => f.path),
      )
      .then(
        (done) => {
          setBusy(false);
          setReport(done);
          onReplaced(done);
          // Re-run the find: what is left is what did not change, which is
          // the honest thing to show next.
          setResult(null);
          setQuery((q) => q);
        },
        (e: unknown) => {
          setBusy(false);
          setError(e instanceof Error ? e.message : String(e));
        },
      );
  }, [query, replacement, options, writable, onReplaced]);

  if (!open) {
    return (
      <button
        type="button"
        className="tree-find__open"
        data-tip="Find and replace across every file in this folder"
        onClick={() => setOpen(true)}
      >
        Find in folder…
      </button>
    );
  }

  return (
    <section className="tree-find" aria-label="Find and replace in this folder">
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
            disabled={busy || writable.length === 0}
            onClick={runReplace}
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
    </section>
  );
}
