<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Find And Replace Is Exhaustive, And Refuses What It Would Undo

For: Authors finding and replacing project text without missing results.

Given a pattern, when the folder is searched for it, then **every** match is
reported, in path order, or the answer says it stopped and why; and when those
matches are replaced, then exactly the matches that were shown are rewritten,
except in files a document generates, which are refused by name.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/search/find-and-replace-is-exhaustive.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-3bdafe22f219"></a>
## `apps/web/src/lib/revealLine.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Find And Replace Is Exhaustive, And Refuses What It Would Undo](contract.md).

<a id="f-3bdafe22f219-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3bdafe22f219-expression_statement">
import { afterEach, describe, expect, it, vi } from "vitest";
import { claimReveal, onRevealLine, resetReveals, revealLine } from "./revealLine";

afterEach(() => {
  resetReveals();
});

</storycode:copy>
<a id="f-3bdafe22f219-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3bdafe22f219-expression_statement-2">
describe("asking for a file at a line", () => {
  it("remembers the request for a pane that does not exist yet", () => {
    // A find hit opens a file; the editor for it is built a tick or two
    // later. An event alone would be shouted into an empty room.
    revealLine("src/main.rs", 42);
    expect(claimReveal("src/main.rs")).toBe(42);
  });

  it("is consumed by the first claim, so it does not fire twice", () => {
    // Re-answering on the next re-render would fight the reader for the
    // cursor.
    revealLine("a.txt", 3);
    expect(claimReveal("a.txt")).toBe(3);
    expect(claimReveal("a.txt")).toBeNull();
  });

  it("keeps the newest request for a file, because that is the one clicked", () => {
    revealLine("a.txt", 3);
    revealLine("a.txt", 90);
    expect(claimReveal("a.txt")).toBe(90);
  });

  it("answers nothing for a file nobody asked about", () => {
    expect(claimReveal("never.txt")).toBeNull();
  });

  it("clamps a line number that could not be scrolled to", () => {
    revealLine("a.txt", 0);
    expect(claimReveal("a.txt")).toBe(1);
    revealLine("b.txt", -5);
    expect(claimReveal("b.txt")).toBe(1);
  });

  it("tells listeners which path was asked for", () => {
    const heard = vi.fn();
    const off = onRevealLine(heard);
    revealLine("src/lib.rs", 7);
    expect(heard).toHaveBeenCalledWith("src/lib.rs");
    off();
    revealLine("src/lib.rs", 8);
    expect(heard).toHaveBeenCalledTimes(1);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/revealLine.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/revealLine.test.ts"><storycode:paste select="#f-3bdafe22f219-expression_statement"/><storycode:paste select="#f-3bdafe22f219-expression_statement-2"/></storycode:file>

<a id="output-60a64147ca82"></a>
## `apps/web/src/shell/TreeFindReplace.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Find And Replace Is Exhaustive, And Refuses What It Would Undo](contract.md).

<a id="f-60a64147ca82-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-60a64147ca82-expression_statement">
// Guarantee: docs/guarantees/search/find-and-replace-is-exhaustive.md
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { TreeFindReplace } from "./TreeFindReplace";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-60a64147ca82-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-60a64147ca82-expression_statement-2">
it("discloses search, validates hit identity and replaces with observed hashes", async () => {
  const find = vi.spyOn(api, "find").mockResolvedValue({ files: [{ path: "source.md", source_hash: "a".repeat(64), matches: [{ line: 3, text: "old", at: [{ column: 0, length: 3 }] }] }], truncated: false });
  const replace = vi.spyOn(api, "replaceAll").mockResolvedValue({ changed: [], skipped: [{ path: "source.md", matches: 1, reason: "source_changed" }], replacements: 0 });
  const open = vi.fn(), replaced = vi.fn();
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><TreeFindReplace onOpenHit={open} onReplaced={replaced} /></UiActionContext.Provider>);
  expect(registry.groups().groups).toEqual(["folder-find-disclosures"]);
  let n = 0;
  const invoke = async (parent: string, label: string, arguments_ = {}) => {
    const action = registry.observe(parent).actions.find(action => action.label === label)!;
    let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: arguments_, request_id: String(++n) }); }); return result;
  };
  await invoke("folder-find-disclosures", "Set folder find visibility", { open: true });
  const parent = registry.groups().groups.find(group => group.startsWith("folder-find:"))!;
  await invoke(parent, "Set folder search query and options", { query: "old", case: false, whole_word: false, regex: false });
  await waitFor(() => expect(registry.observe(parent).actions.find(action => action.label === "Open observed folder search hit")?.enabled).toBe(true));
  expect(find).toHaveBeenCalledWith("old", { case: false, whole_word: false, regex: false });
  expect(await invoke(parent, "Open observed folder search hit", { path: "source.md", line: 4 })).toMatchObject({ outcome: { status: "unavailable" } });
  await invoke(parent, "Open observed folder search hit", { path: "source.md", line: 3 }); expect(open).toHaveBeenCalledWith("source.md", 3);
  await invoke(parent, "Set folder replacement draft", { text: "new" });
  expect(await invoke(parent, "Replace folder matches")).toMatchObject({ outcome: { value: { changed_files: 0, skipped_files: 1, replacements: 0 } } });
  expect(replace).toHaveBeenCalledWith("old", "new", { case: false, whole_word: false, regex: false }, ["source.md"], { "source.md": "a".repeat(64) });
  expect(replaced).toHaveBeenCalledWith(expect.objectContaining({ skipped: [expect.objectContaining({ reason: "source_changed" })] }));
});
</storycode:copy>

### Assemble `apps/web/src/shell/TreeFindReplace.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/TreeFindReplace.test.tsx"><storycode:paste select="#f-60a64147ca82-expression_statement"/><storycode:paste select="#f-60a64147ca82-expression_statement-2"/></storycode:file>

<a id="output-a222850e0cd6"></a>
## `apps/web/src/shell/TreeFindReplace.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Find And Replace Is Exhaustive, And Refuses What It Would Undo](contract.md).

<a id="f-a222850e0cd6-TreeFindReplace"></a>
### `TreeFindReplace`

Find and replace across the whole folder, at the top of the tree.
It lives here rather than in an overlay because it is not a question you
ask and dismiss — it is a list you work through, ticking files, while the
tree beneath it stays where it was. The ranked search overlay (⌘⇧F) is the
other thing, and stays: "where is the bit about invoices" and "rename this
symbol in 40 files" are different jobs with different right answers.
Two rules the UI exists to enforce:
- **A generated file is shown and refused.** Not hidden: its matches are
real and worth reading. But replace will not write it, because the next
weave would undo the edit — so the row says which document to change
instead, which is also the change that fixes every other copy.
- **Nothing is written until Replace is pressed.** The preview and the
write use the same matcher on the server (one pattern, built once), so
the list cannot describe a different edit from the one performed.

<storycode:copy id="f-a222850e0cd6-TreeFindReplace">
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

import { useCallback, useEffect, useId, useRef, useState } from "react";
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
    </section>
  );
}
</storycode:copy>

### Assemble `apps/web/src/shell/TreeFindReplace.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/TreeFindReplace.tsx"><storycode:paste select="#f-a222850e0cd6-TreeFindReplace"/></storycode:file>

<a id="output-51c37beeedd8"></a>
## `crates/hickory-cli/src/serve/find.rs`

Related contracts: [Local History Is The Interval Below The Commit](../../authoring/local-history-is-the-interval-below-the-commit/contract.md), [Find And Replace Is Exhaustive, And Refuses What It Would Undo](contract.md).

<a id="f-51c37beeedd8-MAX_FILE_BYTES"></a>
### `MAX_FILE_BYTES`

Exhaustive find, and replace, across the folder.
Deliberately NOT `/api/search`. That one is ranked — BM25 plus embeddings,
top-k — which is the right shape for "where is the thing about invoices"
and the wrong shape entirely for replace: a ranked answer is a *sample*,
and replacing across a sample silently changes some of the occurrences.
This one is exhaustive, ordered by path, and reports when it stopped.
## What replace refuses
A **generated** file. Its bytes come from a document, and writing to it
either loses the edit at the next weave or fights the up-loop for it. The
honest answer is to say so and name the document, so the reader can make
the change where it will survive — which is also the change that fixes
every other copy of it. See `declared_outputs` in `api.rs`.
A **`.md` document** is fair game: it is source, and a rename inside one
is exactly the edit somebody means.

<storycode:copy id="f-51c37beeedd8-MAX_FILE_BYTES">
//! Exhaustive find, and replace, across the folder.
//!
//! Deliberately NOT `/api/search`. That one is ranked — BM25 plus embeddings,
//! top-k — which is the right shape for "where is the thing about invoices"
//! and the wrong shape entirely for replace: a ranked answer is a *sample*,
//! and replacing across a sample silently changes some of the occurrences.
//! This one is exhaustive, ordered by path, and reports when it stopped.
//!
//! ## What replace refuses
//!
//! A **generated** file. Its bytes come from a document, and writing to it
//! either loses the edit at the next weave or fights the up-loop for it. The
//! honest answer is to say so and name the document, so the reader can make
//! the change where it will survive — which is also the change that fixes
//! every other copy of it. See `declared_outputs` in `api.rs`.
//!
//! A **`.md` document** is fair game: it is source, and a rename inside one
//! is exactly the edit somebody means.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// Files bigger than this are skipped. A find that has to read a 200MB build
/// artefact to tell you it is not there has already wasted more of your time
/// than the answer is worth.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

</storycode:copy>
<a id="f-51c37beeedd8-MAX_MATCHES"></a>
### `MAX_MATCHES`

Stop after this many matches. A pattern like `e` matches a million times
in a real repository, and a listing nobody can read is not an answer.

<storycode:copy id="f-51c37beeedd8-MAX_MATCHES">
/// Stop after this many matches. A pattern like `e` matches a million times
/// in a real repository, and a listing nobody can read is not an answer.
const MAX_MATCHES: usize = 5_000;

</storycode:copy>
<a id="f-51c37beeedd8-FindParams"></a>
### `FindParams`

<storycode:copy id="f-51c37beeedd8-FindParams">
#[derive(Deserialize)]
pub struct FindParams {
    /// The pattern. Literal unless `regex` is set.
    pub q: String,
    #[serde(default)]
    pub regex: bool,
    /// Case-sensitive when set; insensitive is the default, because that is
    /// what someone typing a word into a box means.
    #[serde(default)]
    pub case: bool,
    #[serde(default)]
    pub whole_word: bool,
}

</storycode:copy>
<a id="f-51c37beeedd8-matcher"></a>
### `matcher`

Build the matcher these parameters describe.
Every flag is a change to the PATTERN rather than a branch in the loop, so
find and replace cannot end up matching differently — which would be the
worst possible bug in a tool like this: a preview that does not describe
the write.

<storycode:copy id="f-51c37beeedd8-matcher">
/// Build the matcher these parameters describe.
///
/// Every flag is a change to the PATTERN rather than a branch in the loop, so
/// find and replace cannot end up matching differently — which would be the
/// worst possible bug in a tool like this: a preview that does not describe
/// the write.
fn matcher(params: &FindParams) -> Result<regex::Regex, ApiError> {
    if params.q.is_empty() {
        return Err(ApiError::bad_request("the pattern q= must not be empty"));
    }
    let body = if params.regex {
        params.q.clone()
    } else {
        regex::escape(&params.q)
    };
    let bounded = if params.whole_word {
        format!(r"\b(?:{body})\b")
    } else {
        body
    };
    let cased = if params.case {
        bounded
    } else {
        format!("(?i){bounded}")
    };
    regex::Regex::new(&cased).map_err(|e| {
        ApiError::bad_request(format!(
            "that is not a valid regular expression: {e}\n  \
             Turn the regex option off to search for it literally."
        ))
    })
}

</storycode:copy>
<a id="f-51c37beeedd8-line_hits"></a>
### `line_hits`

One matched line, with enough to render a row and to jump to it.

<storycode:copy id="f-51c37beeedd8-line_hits">
/// One matched line, with enough to render a row and to jump to it.
fn line_hits(regex: &regex::Regex, text: &str, budget: &mut usize) -> Vec<Value> {
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if *budget == 0 {
            break;
        }
        let mut columns = Vec::new();
        for m in regex.find_iter(line) {
            // A zero-width match (`^`, `\b`) would otherwise report every
            // position on the line and replace nothing.
            if m.is_empty() {
                continue;
            }
            columns.push(json!({ "column": m.start(), "length": m.len() }));
            *budget -= 1;
            if *budget == 0 {
                break;
            }
        }
        if !columns.is_empty() {
            out.push(json!({
                "line": index + 1,
                // The whole line, so a result row shows the match in context
                // without a second request per hit.
                "text": line,
                "at": columns,
            }));
        }
    }
    out
}

</storycode:copy>
<a id="f-51c37beeedd8-walk_text_files"></a>
### `walk_text_files`

Every text file under the root, in path order, with its contents.

<storycode:copy id="f-51c37beeedd8-walk_text_files">
/// Every text file under the root, in path order, with its contents.
fn walk_text_files(root: &Path) -> Vec<(String, PathBuf, String)> {
    let mut files = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .require_git(false)
        .git_global(false)
        .filter_entry(|e| e.file_name() != ".hick-cache" && e.file_name() != "node_modules")
        .build();
    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        if entry.metadata().map(|m| m.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(root) else {
            continue;
        };
        // Not UTF-8 is not an error: it is a binary file, and a find over the
        // folder should not stop at the first PNG.
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        files.push((
            rel.to_string_lossy().replace('\\', "/"),
            entry.path().to_path_buf(),
            text,
        ));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

</storycode:copy>
<a id="f-51c37beeedd8-find"></a>
### `find`

`GET /api/find?q=…` — every match in the folder, in path order.

<storycode:copy id="f-51c37beeedd8-find">
/// `GET /api/find?q=…` — every match in the folder, in path order.
pub async fn find(
    State(state): State<LocalState>,
    Query(params): Query<FindParams>,
) -> ApiResult<Json<Value>> {
    let regex = matcher(&params)?;
    let root = state.index.root().to_path_buf();
    let generated = generated_map(&state);

    let (files, truncated) = tokio::task::spawn_blocking(move || {
        let mut budget = MAX_MATCHES;
        let mut out = Vec::new();
        for (rel, _, text) in walk_text_files(&root) {
            if budget == 0 {
                return (out, true);
            }
            let hits = line_hits(&regex, &text, &mut budget);
            if hits.is_empty() {
                continue;
            }
            out.push(json!({
                "path": rel,
                "source_hash": hick_context_protocol::digest(text.as_bytes()),
                "matches": hits,
                // Named here so the UI can grey out what replace will refuse
                // BEFORE anyone presses the button.
                "generated_by": super::api::generated_by(&generated, &rel),
            }));
        }
        (out, budget == 0)
    })
    .await
    .map_err(|e| ApiError::internal(format!("the find task failed: {e}")))?;

    Ok(Json(json!({ "files": files, "truncated": truncated })))
}

</storycode:copy>
<a id="f-51c37beeedd8-ReplaceBody"></a>
### `ReplaceBody`

<storycode:copy id="f-51c37beeedd8-ReplaceBody">
#[derive(Deserialize)]
pub struct ReplaceBody {
    #[serde(flatten)]
    pub find: FindParams,
    /// What each match becomes. `$1` and friends work when `regex` is set.
    pub replacement: String,
    /// Only these paths, when given — how "replace in the files I ticked"
    /// is expressed. Absent means every file that matches.
    #[serde(default)]
    pub paths: Option<Vec<String>>,
    /// Guard the exact files returned by a previous find response.
    #[serde(default)]
    pub expected_sources: Option<HashMap<String, String>>,
}

</storycode:copy>
<a id="f-51c37beeedd8-replace"></a>
### `replace`

`POST /api/find/replace` — rewrite every match, and say what was skipped.

<storycode:copy id="f-51c37beeedd8-replace">
/// `POST /api/find/replace` — rewrite every match, and say what was skipped.
pub async fn replace(
    State(state): State<LocalState>,
    Json(body): Json<ReplaceBody>,
) -> ApiResult<Json<Value>> {
    if let Some(expected) = &body.expected_sources {
        let paths: std::collections::HashSet<_> = body
            .paths
            .as_ref()
            .ok_or_else(|| {
                ApiError::bad_request("Source-guarded replacement requires explicit paths")
            })?
            .iter()
            .collect();
        if paths.len() != expected.len()
            || expected.iter().any(|(path, hash)| {
                !paths.contains(path)
                    || hash.trim_start_matches("sha256:").len() != 64
                    || !hash
                        .trim_start_matches("sha256:")
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit())
            })
        {
            return Err(ApiError::bad_request(
                "Each selected path needs its observed source SHA-256",
            ));
        }
    }
    let regex = matcher(&body.find)?;
    let root = state.index.root().to_path_buf();
    let generated = generated_map(&state);
    let only: Option<std::collections::HashSet<String>> =
        body.paths.map(|list| list.into_iter().collect());
    let replacement = body.replacement.clone();
    let expected = body.expected_sources;

    let find = body.find.q.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let mut changed = Vec::new();
        let mut skipped = Vec::new();
        let mut total = 0usize;
        // Collected first, written second. A replace is the batch writer this
        // whole design is shaped around: forty files in one act, so undoing
        // it is one act too rather than forty reverts done in the right order
        // by hand. Recording per file as we went would produce forty acts and
        // make the undo useless.
        let mut writes: Vec<(std::path::PathBuf, Vec<u8>)> = Vec::new();
        let mut planned = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (rel, absolute, text) in walk_text_files(&root) {
            if only.as_ref().is_some_and(|set| !set.contains(&rel)) {
                continue;
            }
            seen.insert(rel.clone());
            let source_hash = hick_context_protocol::digest(text.as_bytes());
            if expected.as_ref().is_some_and(|expected| !expected.get(&rel).is_some_and(|hash| hash.trim_start_matches("sha256:").eq_ignore_ascii_case(source_hash.trim_start_matches("sha256:")))) {
                skipped.push(json!({"path":rel,"matches":0,"reason":"source_changed","observed_source_hash":source_hash}));
                continue;
            }
            let count = regex.find_iter(&text).filter(|m| !m.is_empty()).count();
            if count == 0 {
                continue;
            }
            // The refusal, and the reason. Writing here either loses the edit
            // at the next weave or fights the up-loop for it.
            if let Some(doc) = generated.get(&rel) {
                skipped.push(json!({
                    "path": rel,
                    "matches": count,
                    "reason": "generated",
                    "document": doc,
                }));
                continue;
            }
            let next = regex.replace_all(&text, replacement.as_str()).into_owned();
            if next == text {
                continue;
            }
            writes.push((absolute.clone(), next.into_bytes()));
            planned.push((rel, absolute, source_hash, count));
        }
        if let Some(expected) = &expected {
            for path in expected.keys().filter(|path| !seen.contains(*path)) {
                skipped.push(json!({"path":path,"matches":0,"reason":"source_unavailable"}));
            }
        }

        // One act for the whole replace, recorded before any of it lands.
        // This is what closes the sentence at the end of
        // `docs/guarantees/search/find-and-replace-is-exhaustive.md`.
        crate::history::record(
            &root,
            hickory_workspace::history::ActKind::Replace,
            Some(format!("{find} → {replacement}")),
            &writes,
        );
        for ((absolute, bytes), (relative, _, source_hash, count)) in writes.iter().zip(planned) {
            if expected.is_some() && !std::fs::read(absolute).is_ok_and(|current| hick_context_protocol::digest(&current) == source_hash) {
                skipped.push(json!({"path":relative,"matches":count,"reason":"source_changed_before_write"}));
                continue;
            }
            super::store::write_atomic(absolute, bytes)
                .map_err(|e| format!("could not write {}: {e:#}", absolute.display()))?;
            total += count;
            changed.push(json!({"path":relative,"matches":count,"before_source_hash":source_hash,"after_source_hash":hick_context_protocol::digest(bytes)}));
        }
        Ok(json!({ "changed": changed, "skipped": skipped, "replacements": total }))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the replace task failed: {e}")))?
    .map_err(ApiError::internal)?;

    Ok(Json(result))
}

</storycode:copy>
<a id="f-51c37beeedd8-generated_map"></a>
### `generated_map`

Output path → the id of the document that writes it.

<storycode:copy id="f-51c37beeedd8-generated_map">
/// Output path → the id of the document that writes it.
fn generated_map(state: &LocalState) -> HashMap<String, String> {
    super::api::generated_outputs(state.index.root(), &state.index)
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/find.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/find.rs"><storycode:paste select="#f-51c37beeedd8-MAX_FILE_BYTES"/><storycode:paste select="#f-51c37beeedd8-MAX_MATCHES"/><storycode:paste select="#f-51c37beeedd8-FindParams"/><storycode:paste select="#f-51c37beeedd8-matcher"/><storycode:paste select="#f-51c37beeedd8-line_hits"/><storycode:paste select="#f-51c37beeedd8-walk_text_files"/><storycode:paste select="#f-51c37beeedd8-find"/><storycode:paste select="#f-51c37beeedd8-ReplaceBody"/><storycode:paste select="#f-51c37beeedd8-replace"/><storycode:paste select="#f-51c37beeedd8-generated_map"/></storycode:file>

## Test the behavior: `source_guarded_replace_refuses_changed_bytes_and_accepts_a_fresh_observation`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_find.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-18392dfca977-source_guarded_replace_refuses_changed_bytes_and_accepts_a_fresh_observation"></a>
### `source_guarded_replace_refuses_changed_bytes_and_accepts_a_fresh_observation`

<storycode:copy id="f-18392dfca977-source_guarded_replace_refuses_changed_bytes_and_accepts_a_fresh_observation">
// Guarantee: docs/guarantees/search/find-and-replace-is-exhaustive.md
#[tokio::test(flavor = "multi_thread")]
async fn source_guarded_replace_refuses_changed_bytes_and_accepts_a_fresh_observation() {
    let session = start(&[("source.txt", "old value\n")]).await;
    let (_, observed) = get(&session, "/api/find?q=old").await;
    let old_hash = observed["files"][0]["source_hash"].as_str().unwrap();
    std::fs::write(
        session.dir.path().join("source.txt"),
        "old changed independently\n",
    )
    .unwrap();
    let (status, refused) = post(&session, "/api/find/replace", json!({"q":"old","replacement":"new","paths":["source.txt"],"expected_sources":{"source.txt":old_hash}})).await;
    assert_eq!(status, 200, "{refused}");
    assert_eq!(refused["changed"], json!([]));
    assert_eq!(refused["skipped"][0]["reason"], "source_changed");
    assert_eq!(
        std::fs::read_to_string(session.dir.path().join("source.txt")).unwrap(),
        "old changed independently\n"
    );
    let (_, fresh) = get(&session, "/api/find?q=old").await;
    let (status, changed) = post(&session, "/api/find/replace", json!({"q":"old","replacement":"new","paths":["source.txt"],"expected_sources":{"source.txt":fresh["files"][0]["source_hash"]}})).await;
    assert_eq!(status, 200, "{changed}");
    assert_eq!(changed["replacements"], 1);
    assert_eq!(
        std::fs::read_to_string(session.dir.path().join("source.txt")).unwrap(),
        "new changed independently\n"
    );
    let (status, _) = post(
        &session,
        "/api/find/replace",
        json!({"q":"new","replacement":"other","paths":["source.txt"],"expected_sources":{}}),
    )
    .await;
    assert_eq!(status, 400);
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/lib/revealLine.ts](../../../mechanisms/apps/web/src/lib/revealLine.ts.md#output-bca672428db0)
- [apps/web/src/shell/TreeFindReplace.tsx](contract.md#output-a222850e0cd6)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/find.rs](contract.md#output-51c37beeedd8)

### Tests and fixtures

- [apps/web/src/lib/revealLine.test.ts](contract.md#output-3bdafe22f219)
- [apps/web/src/shell/TreeFindReplace.test.tsx](contract.md#output-60a64147ca82)
- [crates/hickory-cli/tests/serve_find.rs](../../../mechanisms/crates/hickory-cli/tests/serve_find.rs.md#output-18392dfca977)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
