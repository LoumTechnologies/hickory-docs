<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Sample Shows Generated Lines Without Storing Them

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:sample path="…" from="…" to="…">` inside the `hick:exec` that
generates that file, when the document is woven, then those lines of the file
appear in the markdown as a fenced block tagged with the file's language,
captioned, and footed with the path and the exact line range shown — and the
`.md` document contains none of those bytes, only the one self-closing
element that names them.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-867fcd1186a7"></a>
## `apps/web/src/shell/SamplePicker.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Sample Shows Generated Lines Without Storing Them](contract.md).

<a id="f-867fcd1186a7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-867fcd1186a7-expression_statement">
// Guarantee: docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { SamplePicker } from "./SamplePicker";
import { api } from "../api/client";
import * as reviews from "../lib/reviews";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-867fcd1186a7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-867fcd1186a7-expression_statement-2">
it("shares caption and source-bound dispatch, refuses a retired hash capture and leaves new selection status untouched", async () => {
  const create = vi.spyOn(api,"createSample").mockResolvedValue({doc_id:"doc",doc_path:"doc.md",path:"out.txt",from:1,to:2});
  const hash = vi.spyOn(reviews,"sha256").mockResolvedValue("a".repeat(64));
  const registry = new UiActions({workspace:"work",window:"sample"},()=>true,async()=>{});
  const surface = (contents:string) => <UiActionContext.Provider value={registry}><SamplePicker path="out.txt" range={{from:1,to:2}} contents={contents} /></UiActionContext.Provider>;
  const view=render(surface("selected bytes"));
  const actions=()=>registry.groups().groups.flatMap(group=>registry.observe(group,64).actions);
  const caption=actions().find(action=>action.label === "Set caption for this sample")!;
  await act(async()=>{await registry.act(registry.scope,{kind:"agent",id:"sampler"},{handle:caption.handle,arguments:{caption:"Illustration"},request_id:"caption"});});
  expect(screen.getByLabelText("Caption for this sample")).toHaveProperty("value","Illustration");
  const add=actions().find(action=>action.label === "Show selected generated lines in document")!;
  await act(async()=>{await registry.act(registry.scope,{kind:"agent",id:"sampler"},{handle:add.handle,arguments:{},request_id:"sample"});});
  expect(create).toHaveBeenCalledWith("out.txt",1,2,"Illustration","a".repeat(64));
  expect(screen.getByRole("status").textContent).toContain("doc.md");
  let finishHash!: (value:string)=>void;
  hash.mockImplementationOnce(()=>new Promise(resolve=>{finishHash=resolve;}));
  fireEvent.click(screen.getByRole("button",{name:"Show in document"}));
  view.rerender(surface("replacement bytes"));
  await act(async()=>{finishHash("b".repeat(64));});
  expect(create).toHaveBeenCalledTimes(1);
  expect(screen.queryByRole("status")).toBeNull();
  expect(await registry.act(registry.scope,{kind:"agent",id:"sampler"},{handle:add.handle,arguments:{},request_id:"retired"})).toMatchObject({status:"stale"});
});
</storycode:copy>

### Assemble `apps/web/src/shell/SamplePicker.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/SamplePicker.controls.test.tsx"><storycode:paste select="#f-867fcd1186a7-expression_statement"/><storycode:paste select="#f-867fcd1186a7-expression_statement-2"/></storycode:file>

<a id="output-3ca3810ecbe5"></a>
## `apps/web/src/shell/SamplePicker.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Sample Shows Generated Lines Without Storing Them](contract.md).

<a id="f-3ca3810ecbe5-DOC"></a>
### `DOC`

Which lines a selection means — the arithmetic behind "Lines 11–18".
Protects docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md

<storycode:copy id="f-3ca3810ecbe5-DOC">
// Which lines a selection means — the arithmetic behind "Lines 11–18".
//
// Protects docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md

import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";

import { selectedLines } from "./SamplePicker";

const DOC = "one\ntwo\nthree\nfour\n";

</storycode:copy>
<a id="f-3ca3810ecbe5-at"></a>
### `at`

<storycode:copy id="f-3ca3810ecbe5-at">
function at(anchor: number, head: number) {
  return EditorState.create({ doc: DOC, selection: { anchor, head } });
}

</storycode:copy>
<a id="f-3ca3810ecbe5-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3ca3810ecbe5-expression_statement">
describe("selectedLines", () => {
  it("offers nothing for a bare caret — clicking around is not picking", () => {
    expect(selectedLines(at(5, 5))).toBeNull();
  });

  it("counts the lines a selection covers", () => {
    // From inside line 1 to inside line 3.
    expect(selectedLines(at(1, 10))).toEqual({ from: 1, to: 3 });
  });

  it("does not count a line the selection only touched the start of", () => {
    // "one\ntwo\n" — ends exactly at the first character of line 3, which is
    // two lines highlighted, not three.
    expect(selectedLines(at(0, 8))).toEqual({ from: 1, to: 2 });
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/SamplePicker.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/SamplePicker.test.tsx"><storycode:paste select="#f-3ca3810ecbe5-DOC"/><storycode:paste select="#f-3ca3810ecbe5-at"/><storycode:paste select="#f-3ca3810ecbe5-expression_statement"/></storycode:file>

<a id="output-dea4cc6e7f42"></a>
## `apps/web/src/shell/SamplePicker.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Sample Shows Generated Lines Without Storing Them](contract.md).

<a id="f-dea4cc6e7f42-LineRange"></a>
### `LineRange`

Picking a sample: the gesture that puts a few lines of generated code under
the cell that generated them.
The offer appears only when lines are selected, because that is the moment
it means something — and it says how many, because "sample these 8 lines"
is a different promise from "sample this file". What lands in the document
is one element naming a path and a range; the lines themselves are never
copied in, so nothing here can drift out of date.
See crates/hickory-cli/src/serve/sample.rs, and the `hick:sample` arm in
crates/hick-literate/src/weave.rs.

<storycode:copy id="f-dea4cc6e7f42-LineRange">
// Picking a sample: the gesture that puts a few lines of generated code under
// the cell that generated them.
//
// The offer appears only when lines are selected, because that is the moment
// it means something — and it says how many, because "sample these 8 lines"
// is a different promise from "sample this file". What lands in the document
// is one element naming a path and a range; the lines themselves are never
// copied in, so nothing here can drift out of date.
//
// See crates/hickory-cli/src/serve/sample.rs, and the `hick:sample` arm in
// crates/hick-literate/src/weave.rs.

import { useEffect, useId, useRef, useState } from "react";
import type { EditorState } from "@codemirror/state";

import { api } from "../api/client";
import { sha256 } from "../lib/reviews";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";

export interface LineRange {
  from: number;
  to: number;
}

</storycode:copy>
<a id="f-dea4cc6e7f42-selectedLines"></a>
### `selectedLines`

The 1-based line range the selection covers, or `null` when there is no
selection to speak of.
A caret is not a sample: a one-line range is offered only when the user
actually selected something, so simply clicking around in a generated file
never makes a button appear under the cursor.

<storycode:copy id="f-dea4cc6e7f42-selectedLines">
/**
 * The 1-based line range the selection covers, or `null` when there is no
 * selection to speak of.
 *
 * A caret is not a sample: a one-line range is offered only when the user
 * actually selected something, so simply clicking around in a generated file
 * never makes a button appear under the cursor.
 */
export function selectedLines(state: EditorState): LineRange | null {
  const sel = state.selection.main;
  if (sel.empty) return null;
  const from = state.doc.lineAt(sel.from).number;
  // A selection that ends at the very start of a line has not selected that
  // line; counting it would show one more line than was highlighted.
  const endPos = sel.to > sel.from && state.doc.lineAt(sel.to).from === sel.to ? sel.to - 1 : sel.to;
  return { from, to: state.doc.lineAt(endPos).number };
}

</storycode:copy>
<a id="f-dea4cc6e7f42-SamplePicker"></a>
### `SamplePicker`

<storycode:copy id="f-dea4cc6e7f42-SamplePicker">
export function SamplePicker({ path, range, contents }: { path: string; range: LineRange; contents: string }) {
  const parent = `sample:${useId()}`;
  const selection = JSON.stringify([path, range.from, range.to, contents]);
  const current = useRef(selection); current.current = selection;
  const [caption, setCaption] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [landed, setLanded] = useState<string | null>(null);

  // A new selection is a new sample: whatever the last one said about itself
  // must not linger over it.
  useEffect(() => {
    setError(null);
    setLanded(null);
  }, [selection]);

  const count = range.to - range.from + 1;

  const pick = async (): Promise<UiOutcome> => {
    setBusy(true); setError(null);
    try {
      const hash = await sha256(contents);
      if (current.current !== selection) return { status: "unavailable", reason: "Selected output bytes changed before sample dispatch" };
      const created = await api.createSample(path, range.from, range.to, caption.trim(), hash);
      if (current.current === selection) { setLanded(created.doc_path); setCaption(""); }
      return { status: "completed", value: { doc_path: created.doc_path, output_hash: hash, from: range.from, to: range.to, selection_still_current: current.current === selection, execution_certified: false } };
    } catch (e) {
      const reason = e instanceof Error ? e.message : String(e);
      if (current.current === selection) setError(reason);
      return { status: "unavailable", reason };
    } finally { setBusy(false); }
  };
  const revision = useUiRevision(() => [selection, caption, busy, error, landed]);
  useUiReading("generated sample selection", { path, from: range.from, to: range.to, caption, busy, error, added_to: landed });
  useUiCommands([
    { key: `${parent}:caption`, command: { label: "Set caption for this sample", parent, subject: parent, parameters: { caption: { type: "string", maxLength: 8192 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy, value: caption }), execute: async args => { setCaption(args.caption as string); return { status: "completed", value: args.caption }; } } },
    { key: `${parent}:pick`, command: { label: "Show selected generated lines in document", parent, subject: parent, parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy }), execute: pick } },
  ]);

  return (
    <div className="sample-picker" data-testid="sample-picker">
      <span className="sample-picker__range">
        {count === 1 ? "Line" : "Lines"} {range.from}
        {count > 1 && `–${range.to}`}
      </span>
      <input
        className="sample-picker__caption"
        value={caption}
        disabled={busy}
        placeholder="What does this show? (optional)"
        onChange={(e) => setCaption(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !busy) void pick();
        }}
        aria-label="Caption for this sample"
      />
      <button type="button" onClick={() => void pick()} disabled={busy}>
        {busy ? "Adding…" : "Show in document"}
      </button>
      {landed && (
        <span className="sample-picker__landed" role="status">
          Added to {landed}. It appears in the weave, not in the document.
        </span>
      )}
      {error && (
        <span className="sample-picker__error" role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/shell/SamplePicker.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/SamplePicker.tsx"><storycode:paste select="#f-dea4cc6e7f42-LineRange"/><storycode:paste select="#f-dea4cc6e7f42-selectedLines"/><storycode:paste select="#f-dea4cc6e7f42-SamplePicker"/></storycode:file>

<a id="output-15b9d730d9ff"></a>
## `crates/hickory-cli/src/serve/sample.rs`

Related contracts: [A Sample Shows Generated Lines Without Storing Them](contract.md).

<a id="f-15b9d730d9ff-SampleRequest"></a>
### `SampleRequest`

Picking a sample: `POST /api/samples`.
A generator exists so nobody reads its output, and a document that pasted
the output back in would undo the thing it exists for. But a reader still
has to see WHAT the generator makes, once, before they will believe the
rules — so a sample is a window: a few lines of a generated file, shown
under the cell that generated it.
**The bytes are never written into the document.** What lands in the
`.md` is one self-closing element naming a path and a line range; the
lines themselves appear only in the weave. That is what makes a sample
free to keep — it costs the document one line, it cannot go stale without
the drift check saying so, and it cannot be edited into a lie, because
there is nothing there to edit.
This endpoint is the pick gesture. You select lines in the generated pane
and the app writes the element into the right cell, which is the part a
person should not have to do by hand: finding which of seven documents
and which of its cells wrote a file two directories away is exactly the
bookkeeping the file tree already does.

<storycode:copy id="f-15b9d730d9ff-SampleRequest">
//! Picking a sample: `POST /api/samples`.
//!
//! A generator exists so nobody reads its output, and a document that pasted
//! the output back in would undo the thing it exists for. But a reader still
//! has to see WHAT the generator makes, once, before they will believe the
//! rules — so a sample is a window: a few lines of a generated file, shown
//! under the cell that generated it.
//!
//! **The bytes are never written into the document.** What lands in the
//! `.md` is one self-closing element naming a path and a line range; the
//! lines themselves appear only in the weave. That is what makes a sample
//! free to keep — it costs the document one line, it cannot go stale without
//! the drift check saying so, and it cannot be edited into a lie, because
//! there is nothing there to edit.
//!
//! This endpoint is the pick gesture. You select lines in the generated pane
//! and the app writes the element into the right cell, which is the part a
//! person should not have to do by hand: finding which of seven documents
//! and which of its cells wrote a file two directories away is exactly the
//! bookkeeping the file tree already does.

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult, generated_by};

#[derive(Deserialize)]
pub struct SampleRequest {
    /// Root-relative path of the generated file to sample.
    pub path: String,
    /// First line to show, 1-based and inclusive.
    pub from: usize,
    /// Last line to show, 1-based and inclusive.
    pub to: usize,
    /// The one line of prose above the sample saying what it illustrates.
    #[serde(default)]
    pub caption: Option<String>,
    /// Exact selected generated bytes, supplied by the shared UI controller.
    #[serde(default)]
    pub expected_output_hash: Option<String>,
}

</storycode:copy>
<a id="f-15b9d730d9ff-create"></a>
### `create`

`POST /api/samples` — put a `<hick:sample>` under the cell that generated
this file.

<storycode:copy id="f-15b9d730d9ff-create">
/// `POST /api/samples` — put a `<hick:sample>` under the cell that generated
/// this file.
pub async fn create(
    State(state): State<LocalState>,
    Json(body): Json<SampleRequest>,
) -> ApiResult<Json<Value>> {
    if body.from == 0 || body.to < body.from {
        return Err(ApiError::bad_request(format!(
            "a sample runs from one line to a later one — got {}–{}",
            body.from, body.to
        )));
    }
    let root = state.index.root().to_path_buf();
    let generated = super::api::generated_outputs(&root, &state.index);
    let doc_id = generated_by(&generated, &body.path).ok_or_else(|| {
        ApiError::unprocessable(format!(
            "no document in this folder generates {} — a sample can only show \
             generated bytes, since hand-written ones belong in the document itself",
            body.path
        ))
    })?;
    let doc_rel = state.index.path_of(&doc_id).ok_or_else(|| {
        ApiError::internal(format!("the document generating {} has no path", body.path))
    })?;
    crate::engine::writes::during(&state.writes, async {
        let source = state.read_source_by_doc_path(&doc_rel)?;
        if let Some(expected) = &body.expected_output_hash {
            use sha2::{Digest, Sha256};
            let path = root
                .join(&body.path)
                .canonicalize()
                .map_err(|_| ApiError::conflict("Selected generated output is unavailable"))?;
            let confined = root
                .canonicalize()
                .map_err(|_| ApiError::conflict("Project root is unavailable"))?;
            if !path.starts_with(&confined) || !path.is_file() {
                return Err(ApiError::bad_request(
                    "Selected generated output is outside the project",
                ));
            }
            let bytes = std::fs::read(path)
                .map_err(|_| ApiError::conflict("Selected generated output is unavailable"))?;
            if expected != &format!("{:x}", Sha256::digest(&bytes)) {
                return Err(ApiError::conflict(
                    "Selected generated output changed or is not saved; reselect current saved bytes",
                ));
            }
        }

        // Where the document says it writes this is what tells us which cell to
        // put the sample under — but the two ways it can say so lead there
        // differently. A `hick:file` sits INSIDE its cell; a volume is declared at
        // document level and reaches a cell by being mounted, so the search runs
        // forward from the declaration to the mount rather than outward from it.
        let claim = declaring_claim(&doc_rel, &source, &body.path).ok_or_else(|| {
            ApiError::unprocessable(format!(
                "{doc_rel} generates {} but does not name it in a way this can find \
                 (a path built from a variable, most likely) — add the \
                 `<hick:sample>` by hand",
                body.path
            ))
        })?;
        let anchor = match &claim {
            Claim::File(offset) => *offset,
            Claim::Volume(name) => mounting_cell(&source, name).ok_or_else(|| {
                ApiError::unprocessable(format!(
                    "{doc_rel} declares the volume {name:?} that holds {} but no cell \
                     in it mounts that volume, so there is no run for a sample to sit \
                     under",
                    body.path
                ))
            })?,
        };
        let (insert_at, indent) = insertion_point(&source, anchor).ok_or_else(|| {
            ApiError::unprocessable(format!(
                "{doc_rel} writes {} outside any cell, so there is no run for a \
                 sample to sit under",
                body.path
            ))
        })?;

        // `path=` is relative to the document, the same as everywhere else in the
        // language — the reader of the `.md` should see the path they would
        // type, not one rebased on the folder root.
        let rel_to_doc = relative_to_doc(&doc_rel, &body.path);
        let mut element = format!(
            "{indent}<hick:sample path=\"{rel_to_doc}\" from=\"{}\" to=\"{}\"",
            body.from, body.to
        );
        if let Some(caption) = body
            .caption
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
        {
            if caption.contains('"') {
                return Err(ApiError::bad_request(
                    "a caption cannot contain a double quote — it is an attribute value".to_string(),
                ));
            }
            element.push_str(&format!("\n{indent}             caption=\"{caption}\""));
        }
        element.push_str(" />\n");

        let mut updated = source.clone();
        updated.insert_str(insert_at, &element);
        hick_lang::parse(&updated).map_err(|e| {
            ApiError::unprocessable(format!("the sample would make {doc_rel} unparseable: {e}"))
        })?;

        state.write_source_by_doc_path(&doc_rel, &updated)?;
        state.rooms.apply_external_source(&doc_id, &updated).await;

        Ok(Json(json!({
            "doc_id": doc_id,
            "doc_path": doc_rel,
            "path": rel_to_doc,
            "from": body.from,
            "to": body.to,
        })))
    })
    .await
}

</storycode:copy>
<a id="f-15b9d730d9ff-Claim"></a>
### `Claim`

<storycode:copy id="f-15b9d730d9ff-Claim">
/// How a document claims a generated path.
#[derive(Debug, PartialEq)]
enum Claim {
    /// A `hick:file` at this byte offset. The cell, if any, encloses it.
    File(usize),
    /// A volume by this name. The cell is whichever one mounts it.
    Volume(String),
}

</storycode:copy>
<a id="f-15b9d730d9ff-declaring_claim"></a>
### `declaring_claim`

How `doc_rel` claims `path`, if it does.
Uses the same reading of the source the file tree uses, so a file the tree
marks generated is a file this can find — and one it cannot is one the
tree could not have marked either.

<storycode:copy id="f-15b9d730d9ff-declaring_claim">
/// How `doc_rel` claims `path`, if it does.
///
/// Uses the same reading of the source the file tree uses, so a file the tree
/// marks generated is a file this can find — and one it cannot is one the
/// tree could not have marked either.
fn declaring_claim(doc_rel: &str, source: &str, path: &str) -> Option<Claim> {
    let dir = doc_rel.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let mut best: Option<(usize, Claim)> = None; // (specificity, where)
    for (claim, value) in declared_attribute_values(source) {
        let Some(claimed) = super::api::join_under(dir, &value) else {
            continue;
        };
        // Either this file exactly, or a directory it lies under.
        let claims = claimed == path
            || (path.starts_with(&claimed) && path.as_bytes().get(claimed.len()) == Some(&b'/'));
        let hit = claims.then_some(claimed.len());
        // Longest claim wins, matching `generated_by`: a narrow volume nested
        // inside a wider one is the cell that actually wrote the file.
        if let Some(len) = hit
            && best.as_ref().is_none_or(|(best_len, _)| len > *best_len)
        {
            best = Some((len, claim));
        }
    }
    best.map(|(_, claim)| claim)
}

</storycode:copy>
<a id="f-15b9d730d9ff-mounting_cell"></a>
### `mounting_cell`

The `<hick:exec>` that mounts `volume`, as a byte offset inside it.
`mount="domain:Domain,apiout:out"` — the volume's own name is the part
before the colon, and it is matched on the whole word so `api` does not
find `apiout`.

<storycode:copy id="f-15b9d730d9ff-mounting_cell">
/// The `<hick:exec>` that mounts `volume`, as a byte offset inside it.
///
/// `mount="domain:Domain,apiout:out"` — the volume's own name is the part
/// before the colon, and it is matched on the whole word so `api` does not
/// find `apiout`.
fn mounting_cell(source: &str, volume: &str) -> Option<usize> {
    let re = regex::Regex::new(r#"<hick:exec\b[^>]*?\bmount\s*=\s*"([^"]*)""#)
        .expect("the mount-attribute pattern compiles");
    for capture in re.captures_iter(source) {
        let mounts = &capture[1];
        let names = mounts
            .split(',')
            .map(|m| m.split(':').next().unwrap_or("").trim());
        if names.into_iter().any(|n| n == volume) {
            // Past the tag name, so `insertion_point` walking BACK to the
            // nearest `<hick:exec` finds this one and not an earlier cell.
            return Some(capture.get(0).expect("a match has a whole").end());
        }
    }
    None
}

</storycode:copy>
<a id="f-15b9d730d9ff-declared_attribute_values"></a>
### `declared_attribute_values`

Every `path=`/`output=` value in the source, with how it is claimed.

<storycode:copy id="f-15b9d730d9ff-declared_attribute_values">
/// Every `path=`/`output=` value in the source, with how it is claimed.
fn declared_attribute_values(source: &str) -> Vec<(Claim, String)> {
    let mut out = Vec::new();
    let files = regex::Regex::new(r#"<hick:file\b[^>]*?\bpath\s*=\s*"([^"]*)""#)
        .expect("the file-path pattern compiles");
    for capture in files.captures_iter(source) {
        let start = capture.get(0).expect("a match has a whole").start();
        out.push((Claim::File(start), capture[1].to_string()));
    }
    let volumes = regex::Regex::new(
        r#"<hick:volume\b[^>]*?\bname\s*=\s*"([^"]*)"[^>]*?\boutput\s*=\s*"([^"]*)""#,
    )
    .expect("the volume-output pattern compiles");
    for capture in volumes.captures_iter(source) {
        out.push((
            Claim::Volume(capture[1].to_string()),
            capture[2].to_string(),
        ));
    }
    out
}

</storycode:copy>
<a id="f-15b9d730d9ff-insertion_point"></a>
### `insertion_point`

Where inside the enclosing cell the sample goes, and how far it is
indented: the last line before `</hick:exec>`.
Last, because a sample is an illustration of what the run produced and
belongs after the expectation that checks it, the same way the reader
meets them: the rule, then one example of the rule.

<storycode:copy id="f-15b9d730d9ff-insertion_point">
/// Where inside the enclosing cell the sample goes, and how far it is
/// indented: the last line before `</hick:exec>`.
///
/// Last, because a sample is an illustration of what the run produced and
/// belongs after the expectation that checks it, the same way the reader
/// meets them: the rule, then one example of the rule.
fn insertion_point(source: &str, anchor: usize) -> Option<(usize, String)> {
    let open = source[..anchor].rfind("<hick:exec")?;
    let close = source[anchor..].find("</hick:exec>")? + anchor;
    // A `hick:file` outside any cell can still sit before a later cell's
    // opening tag; only an anchor genuinely inside this one counts.
    if source[open..close].contains("</hick:exec>") {
        return None;
    }
    let line_start = source[..close].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let indent: String = source[line_start..close]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    Some((line_start, indent))
}

</storycode:copy>
<a id="f-15b9d730d9ff-relative_to_doc"></a>
### `relative_to_doc`

A root-relative path, rewritten relative to the document that will name
it. Only the easy direction is handled — a document at the root, or an
output beneath the document's own directory — because those are the ones
the language can express without `..`.

<storycode:copy id="f-15b9d730d9ff-relative_to_doc">
/// A root-relative path, rewritten relative to the document that will name
/// it. Only the easy direction is handled — a document at the root, or an
/// output beneath the document's own directory — because those are the ones
/// the language can express without `..`.
fn relative_to_doc(doc_rel: &str, path: &str) -> String {
    let Some(dir) = doc_rel.rsplit_once('/').map(|(d, _)| d) else {
        return path.to_string();
    };
    path.strip_prefix(&format!("{dir}/"))
        .unwrap_or(path)
        .to_string()
}

</storycode:copy>
<a id="f-15b9d730d9ff-tests"></a>
### `tests`

<storycode:copy id="f-15b9d730d9ff-tests">
#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"# The API

<hick:volume name="domain" input="src/Domain" />
<hick:volume name="apiout" output="src/Api/Generated" />

<hick:exec container="gen" mount="domain:Domain,apiout:out">
python3 gen.py
<hick:expect>
8 endpoints
</hick:expect>
</hick:exec>
"#;

    // Guarantee: docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md
    #[tokio::test]
    async fn selected_output_hash_refuses_stale_bytes_before_editing_the_source() {
        use crate::{
            ExecutorChoice,
            serve::{ServeOptions, prepare},
        };
        use sha2::{Digest, Sha256};
        let root = tempfile::tempdir().unwrap();
        let settings = tempfile::tempdir().unwrap();
        let source = "<hick:container name=\"c\" image=\"alpine\" />\n<hick:exec container=\"c\"><hick:file path=\"out.txt\">Generated</hick:file></hick:exec>";
        std::fs::write(root.path().join("doc.md"), source).unwrap();
        std::fs::write(root.path().join("out.txt"), "current generated bytes").unwrap();
        let prepared = prepare(ServeOptions {
            target: root.path().to_path_buf(),
            port: 0,
            params: vec![],
            executor: ExecutorChoice::Local,
            key_store_path: Some(settings.path().join("keys.json")),
            ui_settings_path: Some(settings.path().join("ui.json")),
        })
        .await
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, prepared.router).await.unwrap();
        });
        let client = reqwest::Client::new();
        let url = format!("http://{address}/api/samples");
        let stale = client
            .post(&url)
            .json(&json!({"path":"out.txt","from":1,"to":1,"expected_output_hash":"a".repeat(64)}))
            .send()
            .await
            .unwrap();
        assert_eq!(stale.status().as_u16(), 409);
        assert_eq!(
            std::fs::read_to_string(root.path().join("doc.md")).unwrap(),
            source
        );
        let hash = format!("{:x}", Sha256::digest(b"current generated bytes"));
        let accepted = client
            .post(url)
            .json(&json!({"path":"out.txt","from":1,"to":1,"expected_output_hash":hash}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            accepted.status().as_u16(),
            200,
            "{}",
            accepted.text().await.unwrap()
        );
        let changed = std::fs::read_to_string(root.path().join("doc.md")).unwrap();
        assert!(changed.contains("<hick:sample") && !changed.contains("current generated bytes"));
        server.abort();
    }

    #[test]
    fn a_volume_output_is_traced_back_to_the_cell_that_mounts_it() {
        // The declaration sits at document level, OUTSIDE the cell — which is
        // the ordinary shape and the one that made the first version of this
        // report "outside any cell" for every generated file in the demo.
        let claim = declaring_claim("30-api.md", DOC, "src/Api/Generated/Endpoints.g.cs")
            .expect("the volume claims it");
        assert_eq!(claim, Claim::Volume("apiout".into()));
        let anchor = mounting_cell(DOC, "apiout").expect("a cell mounts it");
        let (at, indent) = insertion_point(DOC, anchor).expect("the mount is a cell");
        assert_eq!(indent, "");
        // Immediately before the closing tag, after the expectation.
        assert!(DOC[at..].starts_with("</hick:exec>"));
    }

    #[test]
    fn a_mount_name_is_matched_whole() {
        // `api` must not find `apiout`, or a sample lands under the wrong run.
        assert!(mounting_cell(DOC, "api").is_none());
        assert!(mounting_cell(DOC, "apiout").is_some());
    }

    #[test]
    fn a_file_written_outside_a_cell_has_nowhere_to_hang_a_sample() {
        let doc = "<hick:file path=\"a.rs\">\nfn main() {}\n</hick:file>\n";
        let claim = declaring_claim("d.md", doc, "a.rs").expect("the file claims it");
        let Claim::File(offset) = claim else {
            panic!("a hick:file is claimed as a file");
        };
        assert!(insertion_point(doc, offset).is_none());
    }

    #[test]
    fn the_path_is_written_the_way_the_document_would_name_it() {
        assert_eq!(
            relative_to_doc("docs/30-api.md", "docs/src/x.cs"),
            "src/x.cs"
        );
        assert_eq!(relative_to_doc("30-api.md", "src/x.cs"), "src/x.cs");
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/sample.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/sample.rs"><storycode:paste select="#f-15b9d730d9ff-SampleRequest"/><storycode:paste select="#f-15b9d730d9ff-create"/><storycode:paste select="#f-15b9d730d9ff-Claim"/><storycode:paste select="#f-15b9d730d9ff-declaring_claim"/><storycode:paste select="#f-15b9d730d9ff-mounting_cell"/><storycode:paste select="#f-15b9d730d9ff-declared_attribute_values"/><storycode:paste select="#f-15b9d730d9ff-insertion_point"/><storycode:paste select="#f-15b9d730d9ff-relative_to_doc"/><storycode:paste select="#f-15b9d730d9ff-tests"/></storycode:file>

<a id="output-8668aa7756c2"></a>
## `crates/hickory-cli/tests/samples.rs`

Related contracts: [A Sample Shows Generated Lines Without Storing Them](contract.md).

<a id="f-8668aa7756c2-hick"></a>
### `hick`

Samples: a window onto generated bytes, shown and never stored.
Protects docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md
The thing worth testing is the whole reason the element exists: the lines
reach the reader, the document does not gain them, and every way the
window can stop pointing at anything says so out loud instead of showing
an empty box.

<storycode:copy id="f-8668aa7756c2-hick">
//! Samples: a window onto generated bytes, shown and never stored.
//!
//! Protects docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md
//!
//! The thing worth testing is the whole reason the element exists: the lines
//! reach the reader, the document does not gain them, and every way the
//! window can stop pointing at anything says so out loud instead of showing
//! an empty box.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-8668aa7756c2-doc"></a>
### `doc`

A document whose cell writes three lines into a volume, and shows two of
them. Deliberately a VOLUME rather than a `hick:file`: a program wrote
these bytes, which is the case a sample is for.

<storycode:copy id="f-8668aa7756c2-doc">
/// A document whose cell writes three lines into a volume, and shows two of
/// them. Deliberately a VOLUME rather than a `hick:file`: a program wrote
/// these bytes, which is the case a sample is for.
fn doc(from: &str, to: &str) -> String {
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="api.md">
# Generated

<hick:volume name="gen" output="gen" />
<hick:exec container="sh" mount="gen:out">
mkdir -p out
printf 'alpha\nbeta\ngamma\n' > out/thing.txt
<hick:sample path="gen/thing.txt" from="{from}" to="{to}"
             caption="Two of the three" />
</hick:exec>
</hick:doc>
"##
    )
}

</storycode:copy>
<a id="f-8668aa7756c2-run"></a>
### `run`

<storycode:copy id="f-8668aa7756c2-run">
fn run(source: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("api.md");
    std::fs::write(&path, source).unwrap();
    let out = hick().arg("run").arg(&path).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("api.md")).unwrap();
    (dir, woven)
}

</storycode:copy>
<a id="f-8668aa7756c2-the_lines_reach_the_reader_and_the_document_never_gains_them"></a>
### `the_lines_reach_the_reader_and_the_document_never_gains_them`

<storycode:copy id="f-8668aa7756c2-the_lines_reach_the_reader_and_the_document_never_gains_them">
#[test]
fn the_lines_reach_the_reader_and_the_document_never_gains_them() {
    let source = doc("2", "3");
    let (dir, woven) = run(&source);

    assert!(woven.contains("Two of the three"), "no caption:\n{woven}");
    assert!(woven.contains("beta\ngamma"), "no sampled lines:\n{woven}");
    // "alpha" is in the transcript (it is in the command); the sample block
    // itself must hold only the two lines asked for.
    let block = woven.split("```").nth(1).expect("a fenced sample");
    assert_eq!(
        block.trim(),
        "beta\ngamma",
        "wrong lines in the block:\n{woven}"
    );
    assert!(
        woven.contains("gen/thing.txt lines 2–3"),
        "the reader is not told where this came from:\n{woven}"
    );
    // The document is the point: it holds the element, not the bytes.
    let after = std::fs::read_to_string(dir.path().join("api.md")).unwrap();
    assert_eq!(after, source, "the run wrote into the document");
    assert!(
        after.contains("hick:sample"),
        "the element is what the document holds"
    );
}

</storycode:copy>
<a id="f-8668aa7756c2-a_sample_is_neither_the_command_nor_its_stdin"></a>
### `a_sample_is_neither_the_command_nor_its_stdin`

<storycode:copy id="f-8668aa7756c2-a_sample_is_neither_the_command_nor_its_stdin">
#[test]
fn a_sample_is_neither_the_command_nor_its_stdin() {
    // A cell fed its own output is the opposite of what a sample is for, and
    // a sample swept into the command text would be run as a shell line.
    let (_dir, woven) = run(&doc("1", "1"));
    assert!(
        !woven.contains("hick:sample"),
        "the element leaked into the transcript:\n{woven}"
    );
    assert!(
        !woven.contains("caption="),
        "the attributes reached the shell:\n{woven}"
    );
}

</storycode:copy>
<a id="f-8668aa7756c2-a_range_past_the_end_of_the_file_says_so_rather_than_showing_nothing"></a>
### `a_range_past_the_end_of_the_file_says_so_rather_than_showing_nothing`

<storycode:copy id="f-8668aa7756c2-a_range_past_the_end_of_the_file_says_so_rather_than_showing_nothing">
#[test]
fn a_range_past_the_end_of_the_file_says_so_rather_than_showing_nothing() {
    let (_dir, woven) = run(&doc("9", "12"));
    assert!(
        woven.contains("has 3 line(s)") && woven.contains("Re-pick"),
        "a slid-off range showed silently:\n{woven}"
    );
}

</storycode:copy>
<a id="f-8668aa7756c2-a_file_that_was_never_generated_says_to_run_the_document"></a>
### `a_file_that_was_never_generated_says_to_run_the_document`

<storycode:copy id="f-8668aa7756c2-a_file_that_was_never_generated_says_to_run_the_document">
#[test]
fn a_file_that_was_never_generated_says_to_run_the_document() {
    let source = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="api.md">
<hick:exec container="sh">
echo hello
<hick:sample path="nowhere/thing.txt" from="1" to="2" />
</hick:exec>
</hick:doc>
"##;
    let (_dir, woven) = run(source);
    assert!(
        woven.contains("has not been generated yet"),
        "a missing file showed as an empty block:\n{woven}"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/samples.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/samples.rs"><storycode:paste select="#f-8668aa7756c2-hick"/><storycode:paste select="#f-8668aa7756c2-doc"/><storycode:paste select="#f-8668aa7756c2-run"/><storycode:paste select="#f-8668aa7756c2-the_lines_reach_the_reader_and_the_document_never_gains_them"/><storycode:paste select="#f-8668aa7756c2-a_sample_is_neither_the_command_nor_its_stdin"/><storycode:paste select="#f-8668aa7756c2-a_range_past_the_end_of_the_file_says_so_rather_than_showing_nothing"/><storycode:paste select="#f-8668aa7756c2-a_file_that_was_never_generated_says_to_run_the_document"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/shell/SamplePicker.tsx](contract.md#output-dea4cc6e7f42)
- [apps/web/src/shell/views.tsx](../../../mechanisms/apps/web/src/shell/views.tsx.md#output-68a8fba10658)
- [crates/hick-exec/src/dag.rs](../../../mechanisms/crates/hick-exec/src/dag.rs.md#output-e6f480388088)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-cli/src/serve/sample.rs](contract.md#output-15b9d730d9ff)

### Tests and fixtures

- [apps/web/src/shell/SamplePicker.controls.test.tsx](contract.md#output-867fcd1186a7)
- [apps/web/src/shell/SamplePicker.test.tsx](contract.md#output-3ca3810ecbe5)
- [crates/hickory-cli/tests/samples.rs](contract.md#output-8668aa7756c2)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
