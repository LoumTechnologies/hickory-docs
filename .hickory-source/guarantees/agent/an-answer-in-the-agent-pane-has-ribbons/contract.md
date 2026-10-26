<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Trace conversation passages without confusing context with support

For: Authors using an agent to change documents while retaining review control.

Follow a link from an answer to the unsaved editor snapshot supplied to that turn. The link must describe historical input, not claim that it proves the answer.

## What must happen

The live Agent pane uses the protected conversation editor over the ordinary document editor. Recorded and streaming content remains protected; the response region remains editable. Context links use the actual recorded branch and source snapshots, including identifiable untitled editor addresses. They can attach to answer paragraphs in the same user turn and remain visible through folded bookkeeping. A new user turn does not inherit the previous turn's input links. The pane follows normal workspace stacking so connections remain visible.

## Limits of the evidence

Context records what was supplied. It does not establish derivation, entailment or current source freshness.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the live-document, explicit-buffer-identity, per-turn context and pane-stacking amendments into the current reading contract. Keep the standalone read-only session-card view separate.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-4916fa68abf9"></a>
## `apps/web/src/lib/conversationLineage.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](contract.md).

<a id="f-4916fa68abf9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4916fa68abf9-expression_statement">
// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { afterEach, describe, expect, it } from "vitest";
import { conversationAnchor, loadCollapsedLineage, saveCollapsedLineage } from "./conversationLineage";
afterEach(() => { localStorage.clear(); });
</storycode:copy>
<a id="f-4916fa68abf9-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4916fa68abf9-expression_statement-2">
describe("conversation lineage", () => {
  it("defaults to expanded items, with a persisted option for collapsed items", () => {
    expect(loadCollapsedLineage()).toBe(false);
    saveCollapsedLineage(true);
    expect(loadCollapsedLineage()).toBe(true);
  });
  it("anchors an expanded answer but hides closed work and responds to opening it", () => {
    const host = document.createElement("div");
    host.innerHTML = '<div data-session-from="0" data-session-to="100"><p>answer</p><div data-session-from="20" data-session-to="40"><details><summary>Read file</summary><pre>evidence</pre></details></div></div>';
    const fold = host.querySelector("details")!;
    expect(conversationAnchor(host, 0, 100, false)?.textContent).toContain("answer");
    expect(conversationAnchor(host, 20, 40, false)).toBeNull();
    expect(conversationAnchor(host, 20, 40, true)).toBe(fold.querySelector("summary"));
    fold.open = true;
    expect(conversationAnchor(host, 20, 40, false)).toBe(fold.parentElement);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/conversationLineage.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/conversationLineage.test.ts"><storycode:paste select="#f-4916fa68abf9-expression_statement"/><storycode:paste select="#f-4916fa68abf9-expression_statement-2"/></storycode:file>

<a id="output-db697b55388e"></a>
## `apps/web/src/lib/conversationLineage.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](contract.md).

<a id="f-db697b55388e-COLLAPSED_LINEAGE_KEY"></a>
### `COLLAPSED_LINEAGE_KEY`

<storycode:copy id="f-db697b55388e-COLLAPSED_LINEAGE_KEY">
// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
export const COLLAPSED_LINEAGE_KEY = "hickory.conversationCollapsedLineage";
</storycode:copy>
<a id="f-db697b55388e-loadCollapsedLineage"></a>
### `loadCollapsedLineage`

<storycode:copy id="f-db697b55388e-loadCollapsedLineage">
export function loadCollapsedLineage(): boolean {
  return typeof localStorage !== "undefined" && localStorage.getItem(COLLAPSED_LINEAGE_KEY) === "true";
}
</storycode:copy>
<a id="f-db697b55388e-saveCollapsedLineage"></a>
### `saveCollapsedLineage`

<storycode:copy id="f-db697b55388e-saveCollapsedLineage">
export function saveCollapsedLineage(value: boolean): void {
  localStorage.setItem(COLLAPSED_LINEAGE_KEY, String(value));
}

</storycode:copy>
<a id="f-db697b55388e-conversationAnchor"></a>
### `conversationAnchor`

The innermost rendered element is the anchor, not its hidden source lines.

<storycode:copy id="f-db697b55388e-conversationAnchor">
/** The innermost rendered element is the anchor, not its hidden source lines. */
export function conversationAnchor(host: HTMLElement, from: number, to: number, includeCollapsed: boolean): HTMLElement | null {
  const matches = [...host.querySelectorAll<HTMLElement>("[data-session-from]")].filter(element =>
    Number(element.dataset.sessionFrom) <= from && Number(element.dataset.sessionTo) >= to);
  const item = matches.sort((a, b) => (Number(a.dataset.sessionTo) - Number(a.dataset.sessionFrom)) -
    (Number(b.dataset.sessionTo) - Number(b.dataset.sessionFrom)))[0];
  if (!item) return null;
  const ownFold = item.firstElementChild instanceof HTMLDetailsElement ? item.firstElementChild : null;
  let parent: HTMLElement | null = ownFold ?? item;
  while (parent && host.contains(parent)) {
    if (parent instanceof HTMLDetailsElement && !parent.open) {
      return includeCollapsed ? parent.querySelector<HTMLElement>("summary") : null;
    }
    parent = parent.parentElement;
  }
  return item;
}
</storycode:copy>

### Assemble `apps/web/src/lib/conversationLineage.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/conversationLineage.ts"><storycode:paste select="#f-db697b55388e-COLLAPSED_LINEAGE_KEY"/><storycode:paste select="#f-db697b55388e-loadCollapsedLineage"/><storycode:paste select="#f-db697b55388e-saveCollapsedLineage"/><storycode:paste select="#f-db697b55388e-conversationAnchor"/></storycode:file>

<a id="output-c1a7c85841b2"></a>
## `apps/web/src/lib/editorBufferTargets.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](contract.md).

<a id="f-c1a7c85841b2-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c1a7c85841b2-expression_statement">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { bufferTarget, openBufferTarget, registerBufferEditor } from "./editorBufferTargets";
import { workspaceRibbonSources } from "../views/workspaceRibbonSources";
import { SessionRegistry } from "../views/documentSession";
import { initialWorkspace, openUntitledTab } from "../views/workspaceState";
import { panes } from "../shell/layout";

it("registers an unsaved editor as a ribbon target and reveals its cited paragraph without opening a file", () => {
  const host = document.createElement("div"); document.body.appendChild(host);
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "Short.\n\nThe longest paragraph. 📝" }) });
  const layout = openUntitledTab(initialWorkspace());
  const tab = panes(layout.root).flatMap(p => p.tabs).find(t => t.kind === "untitled")!;
  const unregister = registerBufferEditor(tab.id, view);
  const notice = vi.fn(), setLayout = vi.fn();
  try {
    const sources = workspaceRibbonSources(null, new SessionRegistry(), path => ({ path }));
    expect(sources.find(s => s.docPath === bufferTarget(tab.id))?.view).toBe(view);
    expect(openBufferTarget(bufferTarget(tab.id), 3, layout, setLayout, notice)).toBe(true);
    expect(view.state.selection.main.head).toBe(view.state.doc.line(3).from);
    expect(setLayout).toHaveBeenCalledOnce();
    expect(notice).not.toHaveBeenCalled();
    expect(openBufferTarget(bufferTarget("closed"), 3, layout, setLayout, notice)).toBe(true);
    expect(notice).toHaveBeenCalledOnce();
    // A previous window can reuse the short tab id without reusing its evidence.
    expect(openBufferTarget(`hickory-buffer://prior-window/${tab.id}`, 3, layout, setLayout, notice)).toBe(true);
    expect(notice).toHaveBeenCalledTimes(2);
    expect(setLayout).toHaveBeenCalledOnce();
    expect(openBufferTarget("note.md", 3, layout, setLayout, notice)).toBe(false);
  } finally { unregister(); view.destroy(); host.remove(); }
});
</storycode:copy>

### Assemble `apps/web/src/lib/editorBufferTargets.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/editorBufferTargets.test.ts"><storycode:paste select="#f-c1a7c85841b2-expression_statement"/></storycode:file>

<a id="output-96157ff2afcc"></a>
## `apps/web/src/lib/lensSources.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](contract.md).

<a id="f-96157ff2afcc-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-96157ff2afcc-expression_statement">
// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { describe, expect, it } from "vitest";

import { lensSources, onLensChange, registerLens, ribbonLinksOf } from "./lensSources";
import type { EditorView } from "@codemirror/view";

describe("a lens's links, as the overlay draws them", () => {
  it("start from the session's lines and end at the path each element named", () => {
    const links = ribbonLinksOf("sessions/s.md", [
      { family: "context", span: [10, 40], to: { path: "data/x.csv", lines: [1, 8] }, title: "ctx", lines: [4, 4] },
      { family: "lineage", span: [50, 90], to: { path: "notes/today.md", lines: [72, 75] }, title: "lin", lines: [9, 9] },
      { family: "declared", span: [50, 90], to: { path: "meetings/sync.md" }, title: "dec", lines: [9, 12] },
    ]);
    expect(links.map((l) => [l.family, l.from.lines, l.to.path, l.to.kind])).toEqual([
      ["context", [4, 4], "data/x.csv", "file"],
      ["lineage", [9, 9], "notes/today.md", "document"],
      ["declared", [9, 12], "meetings/sync.md", "document"],
    ]);
    expect(links.every((l) => l.from.path === "sessions/s.md")).toBe(true);
    expect(new Set(links.map((l) => l.key)).size).toBe(3);
  });

  it("registers and unregisters a lens, telling listeners each time", () => {
    let told = 0;
    const off = onLensChange(() => told++);
    const view = {} as EditorView;
    const unregister = registerLens({ path: "sessions/a.md", view, source: "", links: [] });
    expect(lensSources().map((l) => l.path)).toContain("sessions/a.md");
    unregister();
    expect(lensSources().map((l) => l.path)).not.toContain("sessions/a.md");
    expect(told).toBe(2);
    off();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/lensSources.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/lensSources.test.ts"><storycode:paste select="#f-96157ff2afcc-expression_statement"/></storycode:file>

<a id="output-f20ae3d80724"></a>
## `apps/web/src/lib/lensSources.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](contract.md).

<a id="f-f20ae3d80724-LensSource"></a>
### `LensSource`

The lenses on screen, as ribbon sources.
A lens is a document view over something the workspace does not hold as a
focused document — the agent pane's session, today. The ribbon overlay
draws from lines of any editor it is handed, so a lens registers its
editor, its text and the links its elements declared, and the workspace
hands them on with everything else it draws. See

<storycode:copy id="f-f20ae3d80724-LensSource">
// The lenses on screen, as ribbon sources.
//
// A lens is a document view over something the workspace does not hold as a
// focused document — the agent pane's session, today. The ribbon overlay
// draws from lines of any editor it is handed, so a lens registers its
// editor, its text and the links its elements declared, and the workspace
// hands them on with everything else it draws. See
// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.

import type { EditorView } from "@codemirror/view";

import type { SessionLink } from "../api/types";
import type { RibbonLink } from "../shell/Ribbons";

export interface LensSource {
  /** The file the lens shows, folder-relative. */
  path: string;
  view: EditorView;
  source: string;
  links: RibbonLink[];
}

</storycode:copy>
<a id="f-f20ae3d80724-ribbonLinksOf"></a>
### `ribbonLinksOf`

A session's declared links as the overlay draws them: from lines of the
session file to the path each element named.

<storycode:copy id="f-f20ae3d80724-ribbonLinksOf">
/** A session's declared links as the overlay draws them: from lines of the
 * session file to the path each element named. */
export function ribbonLinksOf(path: string, links: readonly SessionLink[]): RibbonLink[] {
  return links.map((link, i) => ({
    key: `lens:${path}:${i}`,
    family: link.family,
    from: { path, lines: link.lines },
    to: {
      path: link.to.path,
      lines: link.to.lines,
      kind: link.to.path.endsWith(".md") ? "document" : "file",
    },
    title: link.title,
  }));
}

</storycode:copy>
<a id="f-f20ae3d80724-lenses"></a>
### `lenses`

<storycode:copy id="f-f20ae3d80724-lenses">
const lenses = new Map<string, LensSource>();
</storycode:copy>
<a id="f-f20ae3d80724-listeners"></a>
### `listeners`

<storycode:copy id="f-f20ae3d80724-listeners">
const listeners = new Set<() => void>();
</storycode:copy>
<a id="f-f20ae3d80724-version"></a>
### `version`

<storycode:copy id="f-f20ae3d80724-version">
let version = 0;

</storycode:copy>
<a id="f-f20ae3d80724-changed"></a>
### `changed`

<storycode:copy id="f-f20ae3d80724-changed">
function changed() {
  version += 1;
  for (const listener of listeners) listener();
}

</storycode:copy>
<a id="f-f20ae3d80724-registerLens"></a>
### `registerLens`

<storycode:copy id="f-f20ae3d80724-registerLens">
export function registerLens(lens: LensSource): () => void {
  lenses.set(lens.path, lens);
  changed();
  return () => {
    if (lenses.get(lens.path) === lens) {
      lenses.delete(lens.path);
      changed();
    }
  };
}

</storycode:copy>
<a id="f-f20ae3d80724-lensSources"></a>
### `lensSources`

<storycode:copy id="f-f20ae3d80724-lensSources">
export function lensSources(): LensSource[] {
  return [...lenses.values()];
}

</storycode:copy>
<a id="f-f20ae3d80724-lensVersion"></a>
### `lensVersion`

<storycode:copy id="f-f20ae3d80724-lensVersion">
export function lensVersion(): number {
  return version;
}

</storycode:copy>
<a id="f-f20ae3d80724-onLensChange"></a>
### `onLensChange`

<storycode:copy id="f-f20ae3d80724-onLensChange">
export function onLensChange(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
</storycode:copy>

### Assemble `apps/web/src/lib/lensSources.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/lensSources.ts"><storycode:paste select="#f-f20ae3d80724-LensSource"/><storycode:paste select="#f-f20ae3d80724-ribbonLinksOf"/><storycode:paste select="#f-f20ae3d80724-lenses"/><storycode:paste select="#f-f20ae3d80724-listeners"/><storycode:paste select="#f-f20ae3d80724-version"/><storycode:paste select="#f-f20ae3d80724-changed"/><storycode:paste select="#f-f20ae3d80724-registerLens"/><storycode:paste select="#f-f20ae3d80724-lensSources"/><storycode:paste select="#f-f20ae3d80724-lensVersion"/><storycode:paste select="#f-f20ae3d80724-onLensChange"/></storycode:file>

<a id="output-77f4467cff5b"></a>
## `apps/web/src/views/workspaceRibbonSources.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Trace conversation passages without confusing context with support](contract.md).

<a id="f-77f4467cff5b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-77f4467cff5b-expression_statement">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
import { expect, it } from "vitest";
import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { registerLens } from "../lib/lensSources";
import { registerSearchEditor } from "../lib/workspaceSearch";
import { workspaceRibbonSources } from "./workspaceRibbonSources";
import { SessionRegistry } from "./documentSession";

it("keeps conversation and open buffer sources when no saved document has focus", () => {
  const host = document.createElement("div"); document.body.appendChild(host);
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "unsaved paragraph" }) });
  const unregisterFile = registerSearchEditor("note.md", view);
  const unregisterLens = registerLens({ path: "sessions/s.md", view, source: "conversation", links: [] });
  try {
    const sources = workspaceRibbonSources(undefined, new SessionRegistry(), path => ({ path }));
    expect(sources.map(s => s.docPath)).toEqual(["note.md", "sessions/s.md"]);
    expect(sources[0].docSource).toBe("unsaved paragraph");
    expect(sources[0].view).toBe(view);
  } finally {
    unregisterFile(); unregisterLens(); view.destroy(); host.remove();
  }
});
</storycode:copy>

### Assemble `apps/web/src/views/workspaceRibbonSources.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceRibbonSources.test.ts"><storycode:paste select="#f-77f4467cff5b-expression_statement"/></storycode:file>

<a id="output-463c594a8afa"></a>
## `crates/hick-literate/src/session_elements.rs`

Related contracts: [Trace conversation passages without confusing context with support](contract.md).

<a id="f-463c594a8afa-SessionFacts"></a>
### `SessionFacts`

The session vocabulary as elements: what a conversation's record looks
like as blocks, and what each block knows about where it came from.
A session file is a document (`a-session-is-the-conversation.md`), so
the agent pane is a lens over it rather than a renderer of its own. Each
element renders to a block the frontend draws as a card, and the ones
that touch files declare their provenance beside that:
- `read` (harness-written after a read tool) and a read tool call with a
file argument declare a **context** link — this file, these lines,
were in front of the model.
- `wrote` (harness-written after an edit tool) and an edit tool call
declare a **lineage** link — this turn's output landed there.
- `assistant` prose declares a **declared** link for every markdown link
or `path:line` mention in it — what the model pointed at, an assertion.
Derived from the record, never from the model's opinion of itself
(`docs/specs/freeform/three-provenances.md`). Claude Code transcripts
ingested as sessions (`hick ingest --from claude-code`) carry the same
elements with the tool names Claude Code uses, so they get the same
links. See docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.

<storycode:copy id="f-463c594a8afa-SessionFacts">
//! The session vocabulary as elements: what a conversation's record looks
//! like as blocks, and what each block knows about where it came from.
//!
//! A session file is a document (`a-session-is-the-conversation.md`), so
//! the agent pane is a lens over it rather than a renderer of its own. Each
//! element renders to a block the frontend draws as a card, and the ones
//! that touch files declare their provenance beside that:
//!
//! - `read` (harness-written after a read tool) and a read tool call with a
//!   file argument declare a **context** link — this file, these lines,
//!   were in front of the model.
//! - `wrote` (harness-written after an edit tool) and an edit tool call
//!   declare a **lineage** link — this turn's output landed there.
//! - `assistant` prose declares a **declared** link for every markdown link
//!   or `path:line` mention in it — what the model pointed at, an assertion.
//!
//! Derived from the record, never from the model's opinion of itself
//! (`docs/specs/freeform/three-provenances.md`). Claude Code transcripts
//! ingested as sessions (`hick ingest --from claude-code`) carry the same
//! elements with the tool names Claude Code uses, so they get the same
//! links. See docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.

use std::path::Path;

use hick_blocks::{
    AttrSpec, Block, Descend, Element, Family, Link, LinkTarget, Registry, attr, span_of,
};
use hick_lang::HickTag;

/// What the session elements render from: where the open folder is, so a
/// transcript's absolute paths fold to the folder's spelling.
#[derive(Debug, Default, Clone)]
pub struct SessionFacts {
    pub root: Option<std::path::PathBuf>,
}

</storycode:copy>
<a id="f-463c594a8afa-impl-SessionFacts"></a>
### `impl-SessionFacts`

<storycode:copy id="f-463c594a8afa-impl-SessionFacts">
impl SessionFacts {
    /// A path as the folder knows it.
    pub fn relative(&self, path: &str) -> String {
        let clean = path.replace('\\', "/");
        if let Some(root) = &self.root {
            let root = root.to_string_lossy().replace('\\', "/");
            let root = root.trim_end_matches('/');
            if let Some(rest) = clean.strip_prefix(&format!("{root}/")) {
                return rest.to_string();
            }
        }
        clean.trim_start_matches("./").to_string()
    }
}

</storycode:copy>
<a id="f-463c594a8afa-session_registry"></a>
### `session_registry`

The session elements over the folder's facts.

<storycode:copy id="f-463c594a8afa-session_registry">
/// The session elements over the folder's facts.
pub fn session_registry() -> Registry<SessionFacts> {
    let mut registry = Registry::new();
    registry
        .register(SessionRoot)
        .register(UserElement)
        .register(AssistantElement)
        .register(ToolElement)
        .register(ToolResultElement)
        .register(ReadElement)
        .register(WroteElement)
        .register(ContextElement)
        .register(ObservationElement)
        .register(ActionElement)
        .register(ReasoningElement)
        .register(InputElement)
        .register(MetaElement("usage"))
        .register(MetaElement("next"));
    registry
}

</storycode:copy>
<a id="f-463c594a8afa-parse_lines"></a>
### `parse_lines`

<storycode:copy id="f-463c594a8afa-parse_lines">
/// `a-b` or `a` as lines.
pub fn parse_lines(text: &str) -> Option<(usize, usize)> {
    let mut parts = text.trim().splitn(2, '-');
    let a: usize = parts.next()?.trim().parse().ok()?;
    let b: usize = match parts.next() {
        Some(b) => b.trim().parse().ok()?,
        None => a,
    };
    (a > 0).then_some((a, b.max(a)))
}

</storycode:copy>
<a id="f-463c594a8afa-target"></a>
### `target`

<storycode:copy id="f-463c594a8afa-target">
fn target(facts: &SessionFacts, path: &str, lines: Option<(usize, usize)>) -> LinkTarget {
    LinkTarget {
        path: facts.relative(path),
        lines,
    }
}

</storycode:copy>
<a id="f-463c594a8afa-lines_text"></a>
### `lines_text`

<storycode:copy id="f-463c594a8afa-lines_text">
fn lines_text(lines: Option<(usize, usize)>) -> String {
    match lines {
        Some((a, b)) if a == b => format!(" line {a}"),
        Some((a, b)) => format!(" lines {a}–{b}"),
        None => String::new(),
    }
}

</storycode:copy>
<a id="f-463c594a8afa-SessionRoot"></a>
### `SessionRoot`

`<hick:session>`: the conversation's root when the file is wrapped. Draws
nothing; what it holds is drawn.

<storycode:copy id="f-463c594a8afa-SessionRoot">
// ---------------------------------------------------------------------------

/// `<hick:session>`: the conversation's root when the file is wrapped. Draws
/// nothing; what it holds is drawn.
struct SessionRoot;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-SessionRoot"></a>
### `impl-Element<SessionFacts> for SessionRoot`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-SessionRoot">
impl Element<SessionFacts> for SessionRoot {
    fn name(&self) -> &'static str {
        "session"
    }
    fn descend(&self) -> Descend {
        Descend::All
    }
    fn render(&self, _: &HickTag, _: &SessionFacts) -> Option<Block> {
        None
    }
}

</storycode:copy>
<a id="f-463c594a8afa-UserElement"></a>
### `UserElement`

<storycode:copy id="f-463c594a8afa-UserElement">
struct UserElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-UserElement"></a>
### `impl-Element<SessionFacts> for UserElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-UserElement">
impl Element<SessionFacts> for UserElement {
    fn name(&self) -> &'static str {
        "user"
    }
    fn kind(&self) -> &'static str {
        "session-user"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const A: &[AttrSpec] = &[AttrSpec::optional("turn", "which turn this prompt opens")];
        A
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-user", span_of(tag))
                .with("turn", attr(tag, "turn").or_else(|| attr(tag, "id")))
                .with("body", tag.text_content().trim()),
        )
    }
}

</storycode:copy>
<a id="f-463c594a8afa-assistant_prose"></a>
### `assistant_prose`

The recorder's unnamed input wrapper captures literal prose containing
hick tags. It is speech, not a tool input, and retains its citations.

<storycode:copy id="f-463c594a8afa-assistant_prose">
/// The recorder's unnamed input wrapper captures literal prose containing
/// hick tags. It is speech, not a tool input, and retains its citations.
fn assistant_prose(tag: &HickTag) -> String {
    tag.children
        .iter()
        .filter_map(|node| match node {
            hick_lang::HickNode::Text(text, _) => Some(text.clone()),
            hick_lang::HickNode::Tag(child)
                if child.name == "input" && child.get_attribute("name").is_none() =>
            {
                Some(child.text_content())
            }
            _ => None,
        })
        .collect()
}

</storycode:copy>
<a id="f-463c594a8afa-AssistantElement"></a>
### `AssistantElement`

<storycode:copy id="f-463c594a8afa-AssistantElement">
struct AssistantElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-AssistantElement"></a>
### `impl-Element<SessionFacts> for AssistantElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-AssistantElement">
impl Element<SessionFacts> for AssistantElement {
    fn name(&self) -> &'static str {
        "assistant"
    }
    fn kind(&self) -> &'static str {
        "session-assistant"
    }
    fn descend(&self) -> Descend {
        Descend::All
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        // The prose alone: tool calls and actions nested in the answer are
        // blocks of their own, drawn after it.
        let prose = assistant_prose(tag);
        Some(Block::new("session-assistant", span_of(tag)).with("body", prose.trim()))
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        crate::session_receipts::citations(tag, facts)
    }
}

</storycode:copy>
<a id="f-463c594a8afa-READ_TOOLS"></a>
### `READ_TOOLS`

Tools that show the model a file, by the names hick's agent and Claude
Code use, and the ones that change one.

<storycode:copy id="f-463c594a8afa-READ_TOOLS">
/// Tools that show the model a file, by the names hick's agent and Claude
/// Code use, and the ones that change one.
const READ_TOOLS: &[&str] = &["read_doc", "read_output", "read_file", "read", "Read"];
</storycode:copy>
<a id="f-463c594a8afa-WRITE_TOOLS"></a>
### `WRITE_TOOLS`

<storycode:copy id="f-463c594a8afa-WRITE_TOOLS">
const WRITE_TOOLS: &[&str] = &[
    "edit_doc",
    "write_doc",
    "edit_file",
    "write_file",
    "replace_lines",
    "insert_lines",
    "Edit",
    "Write",
    "MultiEdit",
    "NotebookEdit",
];

</storycode:copy>
<a id="f-463c594a8afa-tool_arg"></a>
### `tool_arg`

<storycode:copy id="f-463c594a8afa-tool_arg">
fn tool_arg(tag: &HickTag, names: &[&str]) -> Option<String> {
    // hick's agent writes `<hick:arg name="doc">…</hick:arg>` children;
    // an ingested transcript may carry attributes instead.
    for child in tag.child_tags() {
        if child.name == "arg"
            && let Some(name) = child.get_attribute("name")
            && names.contains(&name)
        {
            return Some(child.text_content().trim().to_string());
        }
    }
    names.iter().find_map(|n| attr(tag, n))
}

</storycode:copy>
<a id="f-463c594a8afa-ToolElement"></a>
### `ToolElement`

<storycode:copy id="f-463c594a8afa-ToolElement">
struct ToolElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-ToolElement"></a>
### `impl-Element<SessionFacts> for ToolElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-ToolElement">
impl Element<SessionFacts> for ToolElement {
    fn name(&self) -> &'static str {
        "tool"
    }
    fn kind(&self) -> &'static str {
        "session-tool"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const A: &[AttrSpec] = &[
            AttrSpec::required("name", "the tool called"),
            AttrSpec::optional("call", "the provider's id for this call"),
        ];
        A
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        let args: Vec<(String, String)> = tag
            .child_tags()
            .filter(|c| c.name == "arg")
            .map(|c| {
                (
                    c.get_attribute("name").unwrap_or_default().to_string(),
                    c.text_content().trim().to_string(),
                )
            })
            .collect();
        Some(
            Block::new("session-tool", span_of(tag))
                .with("name", attr(tag, "name"))
                .with("call", attr(tag, "call"))
                .with("args", args),
        )
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        let Some(name) = attr(tag, "name") else {
            return vec![];
        };
        let Some(file) = tool_arg(
            tag,
            &[
                "file_path",
                "path",
                "file",
                "doc",
                "output",
                "notebook_path",
            ],
        ) else {
            return vec![];
        };
        // `read_doc` with no argument reads the conversation's own document
        // and records "." — a place, not a file; the `read` the harness
        // writes after it names the file.
        if file.is_empty() || file == "." {
            return vec![];
        }
        if READ_TOOLS.contains(&name.as_str()) {
            // Claude Code's Read takes an offset and a limit; hick's names lines.
            let lines = tool_arg(tag, &["lines"])
                .and_then(|l| parse_lines(&l))
                .or_else(|| {
                    let first: usize = tool_arg(tag, &["from"])?.parse().ok()?;
                    let last: usize = tool_arg(tag, &["to"])?.parse().ok()?;
                    (first > 0 && last >= first).then_some((first, last))
                })
                .or_else(|| {
                    let offset: usize = tool_arg(tag, &["offset"])?.parse().ok()?;
                    let limit: usize = tool_arg(tag, &["limit"])?.parse().ok()?;
                    (offset > 0 && limit > 0)
                        .then(|| offset.checked_add(limit - 1).map(|last| (offset, last)))
                        .flatten()
                });
            vec![Link {
                family: Family::Context,
                span: span_of(tag),
                title: format!(
                    "Read request — the {name} invocation names {}{}. This records requested access; a completed read receipt establishes which bytes were returned.",
                    facts.relative(&file),
                    lines_text(lines)
                ),
                to: target(facts, &file, lines),
            }]
        } else if WRITE_TOOLS.contains(&name.as_str()) {
            vec![Link {
                family: Family::Lineage,
                span: span_of(tag),
                title: format!(
                    "Lineage — the {name} tool wrote {} during this turn: the answer is the source, the file is its output.",
                    facts.relative(&file)
                ),
                to: target(facts, &file, None),
            }]
        } else {
            vec![]
        }
    }
}

</storycode:copy>
<a id="f-463c594a8afa-ToolResultElement"></a>
### `ToolResultElement`

<storycode:copy id="f-463c594a8afa-ToolResultElement">
struct ToolResultElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-ToolResultElement"></a>
### `impl-Element<SessionFacts> for ToolResultElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-ToolResultElement">
impl Element<SessionFacts> for ToolResultElement {
    fn name(&self) -> &'static str {
        "tool-result"
    }
    fn kind(&self) -> &'static str {
        "session-tool-result"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-tool-result", span_of(tag))
                .with("name", attr(tag, "name"))
                .with("ok", attr(tag, "ok").map(|v| v == "true"))
                .with("body", tag.text_content().trim()),
        )
    }
}

</storycode:copy>
<a id="f-463c594a8afa-ReadElement"></a>
### `ReadElement`

<storycode:copy id="f-463c594a8afa-ReadElement">
struct ReadElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-ReadElement"></a>
### `impl-Element<SessionFacts> for ReadElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-ReadElement">
impl Element<SessionFacts> for ReadElement {
    fn name(&self) -> &'static str {
        "read"
    }
    fn kind(&self) -> &'static str {
        "session-read"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const A: &[AttrSpec] = &[
            AttrSpec::required("file", "the file shown to the model"),
            AttrSpec::optional("lines", "which lines, `a-b`"),
            AttrSpec::optional("sha256", "the content hash it was read at"),
            AttrSpec::optional("commit", "the commit it was read at"),
        ];
        A
    }
    fn render(&self, tag: &HickTag, facts: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-read", span_of(tag))
                .with("file", attr(tag, "file").map(|f| facts.relative(&f)))
                .with("lines", attr(tag, "lines"))
                .with("sha256", attr(tag, "sha256"))
                .with("commit", attr(tag, "commit")),
        )
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        let Some(file) = attr(tag, "file") else {
            return vec![];
        };
        let lines = attr(tag, "lines").and_then(|l| parse_lines(&l));
        vec![Link {
            family: Family::Context,
            span: span_of(tag),
            title: format!(
                "Recorded read — {}{} was returned to the model. Snapshot SHA-256: {}. Commit: {}. This proves the recorded read, not a conclusion drawn from it; the current file may have changed.",
                facts.relative(&file),
                lines_text(lines),
                attr(tag, "sha256").unwrap_or_else(|| "not recorded".into()),
                attr(tag, "commit").unwrap_or_else(|| "not recorded".into())
            ),
            to: target(facts, &file, lines),
        }]
    }
}

</storycode:copy>
<a id="f-463c594a8afa-WroteElement"></a>
### `WroteElement`

<storycode:copy id="f-463c594a8afa-WroteElement">
struct WroteElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-WroteElement"></a>
### `impl-Element<SessionFacts> for WroteElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-WroteElement">
impl Element<SessionFacts> for WroteElement {
    fn name(&self) -> &'static str {
        "wrote"
    }
    fn kind(&self) -> &'static str {
        "session-wrote"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const A: &[AttrSpec] = &[
            AttrSpec::required("file", "the file the edit landed in"),
            AttrSpec::optional("lines", "the lines it left, `a-b`"),
            AttrSpec::optional("hashes", "the hashline hashes of those lines"),
        ];
        A
    }
    fn render(&self, tag: &HickTag, facts: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-wrote", span_of(tag))
                .with("file", attr(tag, "file").map(|f| facts.relative(&f)))
                .with("lines", attr(tag, "lines")),
        )
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        let Some(file) = attr(tag, "file") else {
            return vec![];
        };
        let lines = attr(tag, "lines").and_then(|l| parse_lines(&l));
        vec![Link {
            family: Family::Lineage,
            span: span_of(tag),
            title: format!(
                "Lineage — this turn wrote {}{}: the answer is the source, the file is its output.",
                facts.relative(&file),
                lines_text(lines)
            ),
            to: target(facts, &file, lines),
        }]
    }
}

</storycode:copy>
<a id="f-463c594a8afa-ContextElement"></a>
### `ContextElement`

<storycode:copy id="f-463c594a8afa-ContextElement">
struct ContextElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-ContextElement"></a>
### `impl-Element<SessionFacts> for ContextElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-ContextElement">
impl Element<SessionFacts> for ContextElement {
    fn name(&self) -> &'static str {
        "context"
    }
    fn kind(&self) -> &'static str {
        "session-context"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-context", span_of(tag))
                .with("context_kind", attr(tag, "kind"))
                .with("body", tag.text_content().trim()),
        )
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        if tag.get_attribute("kind") == Some("editor-buffers") {
            return crate::session_receipts::editor_context(tag, facts);
        }
        if tag.get_attribute("kind") != Some("acp-activity") {
            return vec![];
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&tag.text_content()) else {
            return vec![];
        };
        let reported_read = matches!(value["kind"].as_str(), Some("read" | "search"));
        let reported_edit = value["kind"] == "edit"
            || value["content"]
                .as_array()
                .is_some_and(|items| items.iter().any(|v| v["type"] == "diff"));
        if !reported_read && !reported_edit {
            return vec![];
        }
        let family = Family::Declared;
        let mut links = Vec::new();
        for location in value["locations"].as_array().into_iter().flatten().chain(
            value["content"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|v| v["type"] == "diff"),
        ) {
            let Some(path) = location["path"].as_str() else {
                continue;
            };
            let lines = location["line"]
                .as_u64()
                .map(|line| (line as usize + 1, line as usize + 1));
            let to = target(facts, path, lines);
            if links.iter().any(|link: &Link| link.to == to) {
                continue;
            }
            links.push(Link {
                family,
                span: span_of(tag),
                to,
                title: format!(
                    "ACP-reported {} — {}{}. Reported by the adapter; no byte hash was supplied.",
                    if reported_read {
                        "file access"
                    } else {
                        "file edit"
                    },
                    facts.relative(path),
                    lines_text(lines)
                ),
            });
        }
        links
    }
}

</storycode:copy>
<a id="f-463c594a8afa-ObservationElement"></a>
### `ObservationElement`

<storycode:copy id="f-463c594a8afa-ObservationElement">
struct ObservationElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-ObservationElement"></a>
### `impl-Element<SessionFacts> for ObservationElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-ObservationElement">
impl Element<SessionFacts> for ObservationElement {
    fn name(&self) -> &'static str {
        "observation"
    }
    fn kind(&self) -> &'static str {
        "session-observation"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-observation", span_of(tag))
                .with("source", attr(tag, "source"))
                .with("exit", attr(tag, "exit"))
                .with("body", tag.text_content().trim()),
        )
    }
}

</storycode:copy>
<a id="f-463c594a8afa-ActionElement"></a>
### `ActionElement`

<storycode:copy id="f-463c594a8afa-ActionElement">
struct ActionElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-ActionElement"></a>
### `impl-Element<SessionFacts> for ActionElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-ActionElement">
impl Element<SessionFacts> for ActionElement {
    fn name(&self) -> &'static str {
        "action"
    }
    fn kind(&self) -> &'static str {
        "session-action"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-action", span_of(tag))
                .with("lang", attr(tag, "lang").or_else(|| attr(tag, "language")))
                .with("body", tag.text_content().trim()),
        )
    }
}

</storycode:copy>
<a id="f-463c594a8afa-ReasoningElement"></a>
### `ReasoningElement`

`<hick:reasoning>`: the model's reasoning, kept apart from its answer and
drawn folded (`reasoning-is-shown-apart-from-the-answer.md`).

<storycode:copy id="f-463c594a8afa-ReasoningElement">
/// `<hick:reasoning>`: the model's reasoning, kept apart from its answer and
/// drawn folded (`reasoning-is-shown-apart-from-the-answer.md`).
struct ReasoningElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-ReasoningElement"></a>
### `impl-Element<SessionFacts> for ReasoningElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-ReasoningElement">
impl Element<SessionFacts> for ReasoningElement {
    fn name(&self) -> &'static str {
        "reasoning"
    }
    fn kind(&self) -> &'static str {
        "session-reasoning"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(Block::new("session-reasoning", span_of(tag)).with("body", tag.text_content().trim()))
    }
}

</storycode:copy>
<a id="f-463c594a8afa-InputElement"></a>
### `InputElement`

<storycode:copy id="f-463c594a8afa-InputElement">
/// `<hick:input>`: a tool's multi-line payload, raw.
struct InputElement;
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-InputElement"></a>
### `impl-Element<SessionFacts> for InputElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-InputElement">
impl Element<SessionFacts> for InputElement {
    fn name(&self) -> &'static str {
        "input"
    }
    fn kind(&self) -> &'static str {
        "session-input"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-input", span_of(tag))
                .with("name", attr(tag, "name"))
                .with("body", tag.text_content().trim()),
        )
    }
}

</storycode:copy>
<a id="f-463c594a8afa-MetaElement"></a>
### `MetaElement`

The harness's bookkeeping — `<hick:usage>` totals and the `<hick:next>`
protocol marker. Facts of the record, not of the conversation: drawn as
nothing, so the lens folds them away rather than showing raw markup
between the cards.

<storycode:copy id="f-463c594a8afa-MetaElement">
/// The harness's bookkeeping — `<hick:usage>` totals and the `<hick:next>`
/// protocol marker. Facts of the record, not of the conversation: drawn as
/// nothing, so the lens folds them away rather than showing raw markup
/// between the cards.
struct MetaElement(&'static str);
</storycode:copy>
<a id="f-463c594a8afa-impl-Element-SessionFacts-for-MetaElement"></a>
### `impl-Element<SessionFacts> for MetaElement`

<storycode:copy id="f-463c594a8afa-impl-Element-SessionFacts-for-MetaElement">
impl Element<SessionFacts> for MetaElement {
    fn name(&self) -> &'static str {
        self.0
    }
    fn kind(&self) -> &'static str {
        "session-meta"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(Block::new("session-meta", span_of(tag)).with("element", self.0))
    }
}

</storycode:copy>
<a id="f-463c594a8afa-mentions"></a>
### `mentions`

What prose points at
Files an answer names: markdown links to local targets (with a
`#L10-L20` fragment as lines), and bare `path/to/file.ext:12` mentions.
Needs a dot in the last segment, so `at 3:15` in prose is not a file,
and no scheme, so a URL is not one either.

<storycode:copy id="f-463c594a8afa-mentions">
// ---------------------------------------------------------------------------
// What prose points at
// ---------------------------------------------------------------------------

/// Files an answer names: markdown links to local targets (with a
/// `#L10-L20` fragment as lines), and bare `path/to/file.ext:12` mentions.
/// Needs a dot in the last segment, so `at 3:15` in prose is not a file,
/// and no scheme, so a URL is not one either.
pub fn mentions(text: &str) -> Vec<(String, Option<(usize, usize)>)> {
    let mut out: Vec<(String, Option<(usize, usize)>)> = Vec::new();
    let mut add = |path: String, lines: Option<(usize, usize)>| {
        if path.is_empty()
            || path.ends_with('/')
            || out.iter().any(|(p, l)| *p == path && *l == lines)
        {
            return;
        }
        out.push((path, lines));
    };
    // [label](target "title")
    let mut rest = text;
    while let Some(open) = rest.find("](") {
        let after = &rest[open + 2..];
        let Some(close) = after.find(')') else { break };
        let target = after[..close].split_whitespace().next().unwrap_or("");
        if !target.is_empty()
            && (!target.contains("://") || target.starts_with("hickory-buffer://"))
            && !target.starts_with("mailto:")
            && !target.starts_with('#')
        {
            let (mut path, fragment) = target.split_once('#').unwrap_or((target, ""));
            let mut lines = fragment.strip_prefix('L').and_then(|f| {
                let f = f.replace('L', "");
                parse_lines(&f)
            });
            if let Some((file, range)) = path.rsplit_once(':')
                && let Some(parsed) = parse_lines(range)
            {
                path = file;
                lines = Some(parsed);
            }
            if path.starts_with("hickory-buffer://")
                || path
                    .rsplit('/')
                    .next()
                    .is_some_and(|last| last.contains('.'))
            {
                add(path.to_string(), lines);
            }
        }
        rest = &after[close + 1..];
    }
    // path:12 or path:12-20
    for token in text.split(|c: char| c.is_whitespace() || "()`'\"<>,;".contains(c)) {
        let Some((path, lines)) = token.rsplit_once(':') else {
            continue;
        };
        let Some(lines) = parse_lines(lines.trim_end_matches(|c: char| ".?!".contains(c))) else {
            continue;
        };
        let last = path.rsplit('/').next().unwrap_or(path);
        if !last.contains('.') || path.contains("://") || path.starts_with('#') {
            continue;
        }
        let path = path.trim_end_matches(|c: char| ".?!".contains(c));
        add(path.to_string(), Some(lines));
    }
    out
}

</storycode:copy>
<a id="f-463c594a8afa-session_blocks"></a>
### `session_blocks`

<storycode:copy id="f-463c594a8afa-session_blocks">
/// The blocks of a session document.
pub fn session_blocks(source: &str, root: Option<&Path>) -> Vec<Block> {
    let facts = SessionFacts {
        root: root.map(Path::to_path_buf),
    };
    let (doc, _) = hick_lang::parse_lenient(source);
    let prose_inputs: Vec<_> = doc
        .all_tags()
        .into_iter()
        .filter(|tag| tag.name == "assistant")
        .flat_map(|tag| {
            tag.child_tags()
                .filter(|child| child.name == "input" && child.get_attribute("name").is_none())
                .map(span_of)
        })
        .collect();
    session_registry()
        .blocks(&doc, &facts)
        .into_iter()
        .filter(|block| block.kind != "session-input" || !prose_inputs.contains(&block.span))
        .collect()
}

</storycode:copy>
<a id="f-463c594a8afa-session_links"></a>
### `session_links`

The links of a session document, with the lines each span covers.

<storycode:copy id="f-463c594a8afa-session_links">
/// The links of a session document, with the lines each span covers.
pub fn session_links(source: &str, root: Option<&Path>) -> Vec<LinkWithLines> {
    let facts = SessionFacts {
        root: root.map(Path::to_path_buf),
    };
    let (doc, _) = hick_lang::parse_lenient(source);
    let mut links = session_registry().links(&doc, &facts);
    links.extend(crate::session_receipts::answer_context(&doc, &facts));
    links
        .into_iter()
        .map(|link| {
            let lines = (
                line_of(source, link.span.0),
                line_of(source, link.span.1.saturating_sub(1).max(link.span.0)),
            );
            LinkWithLines { link, lines }
        })
        .collect()
}

</storycode:copy>
<a id="f-463c594a8afa-LinkWithLines"></a>
### `LinkWithLines`

A link with the 1-based lines its span starts and ends on — what the
overlay anchors on.

<storycode:copy id="f-463c594a8afa-LinkWithLines">
/// A link with the 1-based lines its span starts and ends on — what the
/// overlay anchors on.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkWithLines {
    #[serde(flatten)]
    pub link: Link,
    pub lines: (usize, usize),
}

</storycode:copy>
<a id="f-463c594a8afa-line_of"></a>
### `line_of`

<storycode:copy id="f-463c594a8afa-line_of">
fn line_of(source: &str, byte: usize) -> usize {
    source.as_bytes()[..byte.min(source.len())]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

</storycode:copy>
<a id="f-463c594a8afa-tests"></a>
### `tests`

<storycode:copy id="f-463c594a8afa-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_line_lookup_accepts_offsets_inside_unicode() {
        let source = "first\n📝\nlast";
        assert_eq!(line_of(source, 8), 2);
        assert_eq!(line_of(source, usize::MAX), 3);
    }

    type Summary = (String, String, Option<(usize, usize)>, usize);
    type Where = (String, String, Option<(usize, usize)>);

    // Guarantees: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
    #[test]
    fn literal_assistant_speech_stays_visible_and_keeps_its_citation() {
        let source = r#"<hick:session><hick:user>Which file?</hick:user>
<hick:assistant><hick:input>
[test2.md](/work/test2.md:8) produces `test2.py` through `<hick:file path="test2.py">`.
</hick:input></hick:assistant></hick:session>"#;
        let blocks = session_blocks(source, Some(Path::new("/work")));
        assert_eq!(blocks.len(), 2);
        assert!(
            blocks[1].props["body"]
                .as_str()
                .unwrap()
                .contains("<hick:file")
        );
        let links = session_links(source, Some(Path::new("/work")));
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link.to.path, "test2.md");
        assert_eq!(links[0].link.to.lines, Some((8, 8)));
    }

    // Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
    #[test]
    fn requested_access_and_completed_read_evidence_have_distinct_explanations() {
        let source = r#"<hick:session>
<hick:tool name="read_file" path="note.md" from="2" to="3"/>
<hick:tool-result name="read_file" ok="false">No such file</hick:tool-result>
<hick:read file="other.md" lines="4-5" sha256="snapshot-hash" commit="abc123"/>
</hick:session>"#;
        let links = session_links(source, None);
        assert_eq!(links.len(), 2);
        assert!(links[0].link.title.starts_with("Read request"));
        assert!(links[0].link.title.contains("completed read receipt"));
        assert_eq!(links[0].link.to.lines, Some((2, 3)));
        assert!(links[1].link.title.starts_with("Recorded read"));
        assert!(links[1].link.title.contains("snapshot-hash"));
        assert!(links[1].link.title.contains("abc123"));
        assert_eq!(links[1].link.to.lines, Some((4, 5)));
    }

    #[test]
    fn acp_locations_and_diffs_link_but_shell_text_never_becomes_lineage() {
        let source = r#"<hick:session>
<hick:context kind="acp-activity">{"kind":"read","locations":[{"path":"/work/input.py","line":0}]}</hick:context>
<hick:context kind="acp-activity">{"kind":"edit","content":[{"type":"diff","path":"/work/output.py"}]}</hick:context>
<hick:context kind="acp-activity">{"kind":"execute","title":"cat output.py","rawOutput":"input.md produces output.py"}</hick:context>
</hick:session>"#;
        let links = session_links(source, Some(Path::new("/work")));
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].link.family, Family::Declared);
        assert_eq!(links[0].link.to.lines, Some((1, 1)));
        assert_eq!(links[1].link.family, Family::Declared);
        assert!(
            links
                .iter()
                .all(|link| link.link.title.contains("ACP-reported"))
        );
    }

    const SESSION: &str = r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-08-22T12:56:34Z">
<hick:user turn="t1">Why is checkout slow?</hick:user>
<hick:assistant>
<hick:tool name="read_doc">
<hick:arg name="doc">meetings/sync.md</hick:arg>
</hick:tool>
</hick:assistant>
<hick:tool-result name="read_doc" ok="true">
lines…
</hick:tool-result>
<hick:read file="/home/me/notes/data/latency.csv" sha256="ff" lines="1-8"/>
<hick:assistant>
Checkout slowed after the pool change; see [the sync](meetings/sync.md#L12-L20) and data/latency.csv:3.
</hick:assistant>
<hick:wrote file="notes/today.md" lines="72-75" hashes="a b c d"/>
</hick:session>
"#;

    #[test]
    fn a_session_is_blocks_of_its_own_kinds() {
        let blocks = session_blocks(SESSION, Some(Path::new("/home/me/notes")));
        let kinds: Vec<&str> = blocks.iter().map(|b| b.kind.as_str()).collect();
        assert_eq!(
            kinds,
            vec![
                "session-user",
                "session-assistant",
                "session-tool",
                "session-tool-result",
                "session-read",
                "session-assistant",
                "session-wrote"
            ]
        );
        assert_eq!(blocks[0].str_prop("turn"), Some("t1"));
        assert_eq!(blocks[4].str_prop("file"), Some("data/latency.csv"));
        assert!(
            blocks[5]
                .str_prop("body")
                .unwrap()
                .starts_with("Checkout slowed")
        );
    }

    #[test]
    fn the_three_families_come_from_the_record_and_the_prose() {
        let links = session_links(SESSION, Some(Path::new("/home/me/notes")));
        let summary: Vec<Summary> = links
            .iter()
            .map(|l| {
                (
                    format!("{:?}", l.link.family).to_lowercase(),
                    l.link.to.path.clone(),
                    l.link.to.lines,
                    l.lines.0,
                )
            })
            .collect();
        assert_eq!(
            summary,
            vec![
                ("context".into(), "meetings/sync.md".into(), None, 4),
                (
                    "context".into(),
                    "data/latency.csv".into(),
                    Some((1, 8)),
                    11
                ),
                (
                    "declared".into(),
                    "meetings/sync.md".into(),
                    Some((12, 20)),
                    13
                ),
                (
                    "declared".into(),
                    "data/latency.csv".into(),
                    Some((3, 3)),
                    13
                ),
                (
                    "lineage".into(),
                    "notes/today.md".into(),
                    Some((72, 75)),
                    15
                ),
            ]
        );
    }

    #[test]
    fn a_claude_code_transcript_reads_the_same_way() {
        let src = r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:user>fix it</hick:user>
<hick:assistant>
<hick:tool name="Read" call="t1"><hick:arg name="file_path">/home/me/notes/src/lib.rs</hick:arg><hick:arg name="offset">10</hick:arg><hick:arg name="limit">5</hick:arg></hick:tool>
<hick:tool name="Edit" call="t2"><hick:arg name="file_path">/home/me/notes/src/lib.rs</hick:arg></hick:tool>
<hick:tool name="Bash" call="t3"><hick:arg name="command">ls</hick:arg></hick:tool>
</hick:assistant>
</hick:session>
"#;
        let links = session_links(src, Some(Path::new("/home/me/notes")));
        let summary: Vec<Where> = links
            .iter()
            .map(|l| {
                (
                    format!("{:?}", l.link.family).to_lowercase(),
                    l.link.to.path.clone(),
                    l.link.to.lines,
                )
            })
            .collect();
        assert_eq!(
            summary,
            vec![
                ("context".into(), "src/lib.rs".into(), Some((10, 14))),
                ("lineage".into(), "src/lib.rs".into(), None),
            ]
        );
    }

    #[test]
    fn mentions_find_links_and_path_lines_and_nothing_else() {
        let found = mentions(
            "See [x](../a/b.md#L3-L4), src/lib.rs:42, `data/x.csv:1-8`; not https://e.com/x.md:3, not at 3:15, not README.",
        );
        assert_eq!(
            found,
            vec![
                ("../a/b.md".into(), Some((3, 4))),
                ("src/lib.rs".into(), Some((42, 42))),
                ("data/x.csv".into(), Some((1, 8))),
            ]
        );
    }
}
</storycode:copy>

### Assemble `crates/hick-literate/src/session_elements.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/session_elements.rs"><storycode:paste select="#f-463c594a8afa-SessionFacts"/><storycode:paste select="#f-463c594a8afa-impl-SessionFacts"/><storycode:paste select="#f-463c594a8afa-session_registry"/><storycode:paste select="#f-463c594a8afa-parse_lines"/><storycode:paste select="#f-463c594a8afa-target"/><storycode:paste select="#f-463c594a8afa-lines_text"/><storycode:paste select="#f-463c594a8afa-SessionRoot"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-SessionRoot"/><storycode:paste select="#f-463c594a8afa-UserElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-UserElement"/><storycode:paste select="#f-463c594a8afa-assistant_prose"/><storycode:paste select="#f-463c594a8afa-AssistantElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-AssistantElement"/><storycode:paste select="#f-463c594a8afa-READ_TOOLS"/><storycode:paste select="#f-463c594a8afa-WRITE_TOOLS"/><storycode:paste select="#f-463c594a8afa-tool_arg"/><storycode:paste select="#f-463c594a8afa-ToolElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-ToolElement"/><storycode:paste select="#f-463c594a8afa-ToolResultElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-ToolResultElement"/><storycode:paste select="#f-463c594a8afa-ReadElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-ReadElement"/><storycode:paste select="#f-463c594a8afa-WroteElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-WroteElement"/><storycode:paste select="#f-463c594a8afa-ContextElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-ContextElement"/><storycode:paste select="#f-463c594a8afa-ObservationElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-ObservationElement"/><storycode:paste select="#f-463c594a8afa-ActionElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-ActionElement"/><storycode:paste select="#f-463c594a8afa-ReasoningElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-ReasoningElement"/><storycode:paste select="#f-463c594a8afa-InputElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-InputElement"/><storycode:paste select="#f-463c594a8afa-MetaElement"/><storycode:paste select="#f-463c594a8afa-impl-Element-SessionFacts-for-MetaElement"/><storycode:paste select="#f-463c594a8afa-mentions"/><storycode:paste select="#f-463c594a8afa-session_blocks"/><storycode:paste select="#f-463c594a8afa-session_links"/><storycode:paste select="#f-463c594a8afa-LinkWithLines"/><storycode:paste select="#f-463c594a8afa-line_of"/><storycode:paste select="#f-463c594a8afa-tests"/></storycode:file>

<a id="output-af680583ad64"></a>
## `crates/hickory-cli/src/serve/sessions.rs`

Related contracts: [Trace conversation passages without confusing context with support](contract.md), [Assumptions remain auditable after the work](../assumptions-remain-auditable-after-the-work/contract.md).

<a id="f-af680583ad64-SessionQuery"></a>
### `SessionQuery`

The conversation lens and current, independently checked output evidence.

<storycode:copy id="f-af680583ad64-SessionQuery">
//! The conversation lens and current, independently checked output evidence.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

#[derive(Deserialize)]
pub struct SessionQuery {
    pub path: String,
}

</storycode:copy>
<a id="f-af680583ad64-session_view"></a>
### `session_view`

`GET /api/sessions/view?path=…` — a session file read back as the
conversation it records: turns (with their parent turn, for the tree),
and within each turn the steps — reasoning, prose, scripts and their
output, tool calls and results, files shown, lines written. The same
shape the chat dock renders, so a session opened as a document looks like
the chat it was.

<storycode:copy id="f-af680583ad64-session_view">
/// `GET /api/sessions/view?path=…` — a session file read back as the
/// conversation it records: turns (with their parent turn, for the tree),
/// and within each turn the steps — reasoning, prose, scripts and their
/// output, tool calls and results, files shown, lines written. The same
/// shape the chat dock renders, so a session opened as a document looks like
/// the chat it was.
pub async fn session_view(
    State(state): State<LocalState>,
    Query(q): Query<SessionQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let rel = std::path::Path::new(&q.path);
    let abs = if rel.is_absolute() {
        rel.to_path_buf()
    } else {
        root.join(rel)
    };
    let canon = abs
        .canonicalize()
        .map_err(|e| ApiError::not_found(format!("no session at {}: {e}", q.path)))?;
    let root_canon = root.canonicalize().unwrap_or(root.clone());
    let local_sessions = hickory_workspace::WorkspaceStore::for_project(&root)
        .ok()
        .map(|store| store.dir().join("literate-agents/sessions"));
    if !canon.starts_with(&root_canon)
        && !local_sessions
            .is_some_and(|dir| dir.canonicalize().is_ok_and(|dir| canon.starts_with(dir)))
    {
        return Err(ApiError::not_found(format!(
            "{} is outside this folder",
            q.path
        )));
    }
    let rel_path = canon
        .strip_prefix(&root_canon)
        .map(|p| p.display().to_string())
        .unwrap_or(q.path.clone());
    // The session as the lens draws it: its blocks, and the provenance each
    // element declares — context for what the model was shown, lineage for
    // what a turn wrote, declared for what the prose points at. See
    // docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.
    // A growing provider session can contain megabytes of recorded context.
    // Parsing it on the request executor starves token-bound editor capture
    // acknowledgements and makes independent freshness checks time out.
    let parse_root = root_canon.clone();
    let (source, view, blocks, mut links) = tokio::task::spawn_blocking(move || {
        let source = std::fs::read_to_string(&canon)
            .map_err(|e| ApiError::not_found(format!("cannot read {}: {e}", q.path)))?;
        if !hick_lang::is_session_source(&source) {
            return Err(ApiError::bad_request(format!(
                "{} is not a hick:session document",
                q.path
            )));
        }
        let view = hickory_agent::session_view::session_view(&source);
        let blocks = hick_literate::session_elements::session_blocks(&source, Some(&parse_root));
        let links = hick_literate::session_elements::session_links(&source, Some(&parse_root));
        Ok((
            source,
            view,
            blocks,
            serde_json::to_value(links).unwrap_or_default(),
        ))
    })
    .await
    .map_err(|_| ApiError::internal("Session parsing task failed"))??;
    let evidence = output_evidence(&state, &root_canon, &source, &links).await;
    if let Some(links) = links.as_array_mut() {
        links.extend(evidence);
    }
    let payload = tokio::task::spawn_blocking(move || {
        let mut audit = hickory_agent::audit::timeline(&source);
        super::audit_evidence::enrich(&root_canon, &mut audit, &links);
        super::audit_evidence::attach_passages(&root_canon, &source, &audit, &mut links);
        json!({ "path": rel_path, "source": source, "view": view, "blocks": blocks, "links": links, "audit": audit })
    })
    .await
    .map_err(|_| ApiError::internal("Session evidence task failed"))?;
    Ok(Json(payload))
}

</storycode:copy>
<a id="f-af680583ad64-AuditAnnotation"></a>
### `AuditAnnotation`

<storycode:copy id="f-af680583ad64-AuditAnnotation">
#[derive(Deserialize)]
pub struct AuditAnnotation {
    pub path: String,
    pub entry: serde_json::Value,
}

</storycode:copy>
<a id="f-af680583ad64-annotate"></a>
### `annotate`

<storycode:copy id="f-af680583ad64-annotate">
pub async fn annotate(
    State(state): State<LocalState>,
    Json(body): Json<AuditAnnotation>,
) -> ApiResult<Json<Value>> {
    let _annotation_gate = state.agent.annotations_gate.try_write().map_err(|_| {
        ApiError::conflict("Wait for active agent turns to finish before annotating a session.")
    })?;
    // Validate the same confinement as the read endpoint before creating a writer.
    let _ = session_view(
        State(state.clone()),
        Query(SessionQuery {
            path: body.path.clone(),
        }),
    )
    .await?;
    if state
        .agent
        .turns
        .lock()
        .unwrap()
        .values()
        .flatten()
        .any(|turn| turn.status == "running")
    {
        return Err(ApiError::conflict(
            "Wait for active agent turns to finish before annotating a session.",
        ));
    }
    let path = state.index.root().join(&body.path);
    let result = hickory_agent::audit::annotate_at(&path, body.entry, state.index.root())
        .map_err(|error| ApiError::unprocessable(error.to_string()))?;
    Ok(Json(json!({"recorded":result})))
}

</storycode:copy>
<a id="f-af680583ad64-output_evidence"></a>
### `output_evidence`

Citation is the trigger, never the evidence. Weave without executing and
accept only an output whose disk bytes still match and whose provenance
names a source inside this workspace. Old sessions are not rewritten.

<storycode:copy id="f-af680583ad64-output_evidence">
/// Citation is the trigger, never the evidence. Weave without executing and
/// accept only an output whose disk bytes still match and whose provenance
/// names a source inside this workspace. Old sessions are not rewritten.
async fn output_evidence(
    state: &LocalState,
    root: &Path,
    session_source: &str,
    links: &Value,
) -> Vec<Value> {
    let mut evidence = Vec::new();
    let docs: std::collections::BTreeSet<_> = links
        .as_array()
        .into_iter()
        .flatten()
        .filter(|link| link["family"] == "declared")
        .filter_map(|link| link["to"]["path"].as_str())
        .filter(|path| path.ends_with(".md") || path.ends_with(".md"))
        .collect();
    for doc in docs {
        let cited: Vec<_> = links
            .as_array()
            .into_iter()
            .flatten()
            .filter(|link| link["family"] == "declared" && link["to"]["path"] == doc)
            .collect();
        if cited.is_empty() {
            continue;
        }
        let Ok(path) = root.join(doc).canonicalize() else {
            continue;
        };
        if !path.starts_with(root) {
            continue;
        }
        let id = state.index.id_for_path(doc);
        let run = if state.index.absolute(&id).is_some() {
            state.weave(&id).await.ok()
        } else {
            // Older .md files can be cited even when startup's .md scan
            // did not index them. This is the same non-executing weave.
            crate::run_doc(
                &path,
                &state.params,
                crate::RunMode::Weave,
                crate::ExecutorChoice::Local,
            )
            .await
            .ok()
        };
        let Some(run) = run else {
            continue;
        };
        if std::fs::read_to_string(&path).ok().as_deref() != Some(run.source.as_str()) {
            continue;
        }
        let dir = Path::new(doc).parent().unwrap_or(Path::new(""));
        for (key, content) in &run.result.files {
            let Some(text) = content.as_text() else {
                continue;
            };
            let output = dir.join(key);
            let Ok(canon) = root.join(&output).canonicalize() else {
                continue;
            };
            if !canon.starts_with(root)
                || std::fs::read(&canon).ok().as_deref() != Some(text.as_bytes())
            {
                continue;
            }
            let Ok(provenance) = crate::output_lineage(&run, key) else {
                continue;
            };
            for citation in &cited {
                let Some(start) = citation["span"][0].as_u64() else {
                    continue;
                };
                let Some(end) = citation["span"][1].as_u64() else {
                    continue;
                };
                let Some(body) = session_source.get(start as usize..end as usize) else {
                    continue;
                };
                let output_name = output.to_string_lossy();
                let named = body
                    .split(|c: char| c.is_whitespace() || "`\"'()<>[];,".contains(c))
                    .any(|token| {
                        token.trim_end_matches(['.', '!', '?']) == output_name || token == key
                    });
                if !named {
                    continue;
                }
                for entry in &provenance {
                    let Some((origin_path, from, to)) = entry.origin.location() else {
                        continue;
                    };
                    let Ok(origin) = root.join(origin_path).canonicalize() else {
                        continue;
                    };
                    let Ok(relative) = origin.strip_prefix(root) else {
                        continue;
                    };
                    // Cross-document origins need their own captured source;
                    // this check currently establishes this producer only.
                    if origin != path {
                        continue;
                    }
                    let source = &run.source;
                    if entry.end <= entry.start || to <= from {
                        continue;
                    }
                    let source_lines =
                        (line_at(source, from), line_at(source, to.saturating_sub(1)));
                    let output_lines = (
                        line_at(text, entry.start),
                        line_at(text, entry.end.saturating_sub(1)),
                    );
                    let proof = json!({
                        "family":"lineage", "span":citation["span"], "lines":citation["lines"],
                        "to":{"path":output_name,"lines":output_lines},
                        "evidence":{"path":relative.to_string_lossy(),"lines":source_lines},
                        "title":format!("Checked current output lineage — {}:{}–{} produces {}:{}–{}. Woven bytes match the file on disk; this is document evidence, not a claim that the conversation wrote it.", relative.display(), source_lines.0, source_lines.1, output_name, output_lines.0, output_lines.1)
                    });
                    if !evidence.contains(&proof) {
                        evidence.push(proof);
                    }
                }
            }
        }
    }
    evidence
}

</storycode:copy>
<a id="f-af680583ad64-line_at"></a>
### `line_at`

<storycode:copy id="f-af680583ad64-line_at">
fn line_at(source: &str, byte: usize) -> usize {
    source.as_bytes()[..byte.min(source.len())]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/sessions.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/sessions.rs"><storycode:paste select="#f-af680583ad64-SessionQuery"/><storycode:paste select="#f-af680583ad64-session_view"/><storycode:paste select="#f-af680583ad64-AuditAnnotation"/><storycode:paste select="#f-af680583ad64-annotate"/><storycode:paste select="#f-af680583ad64-output_evidence"/><storycode:paste select="#f-af680583ad64-line_at"/></storycode:file>

<a id="output-21c9526a7692"></a>
## `crates/hickory-cli/tests/session_lens.rs`

Related contracts: [Trace conversation passages without confusing context with support](contract.md).

<a id="f-21c9526a7692-ENV_LOCK"></a>
### `ENV_LOCK`

The session view carries what the lens draws — the conversation as
blocks — and what each element declared about where it came from.
Protects docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.

<storycode:copy id="f-21c9526a7692-ENV_LOCK">
//! The session view carries what the lens draws — the conversation as
//! blocks — and what each element declared about where it came from.
//!
//! Protects docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;
use std::sync::Arc;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

</storycode:copy>
<a id="f-21c9526a7692-SESSION"></a>
### `SESSION`

<storycode:copy id="f-21c9526a7692-SESSION">
const SESSION: &str = r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-09-05T00:00:00Z" doc="notes/today.md">
<hick:user turn="t1">Why is checkout slow?</hick:user>
<hick:assistant>
<hick:tool name="read_doc">
<hick:arg name="doc">meetings/sync.md</hick:arg>
</hick:tool>
</hick:assistant>
<hick:tool-result name="read_doc" ok="true">
…
</hick:tool-result>
<hick:read file="data/latency.csv" sha256="ff" lines="1-8"/>
<hick:assistant>
It slowed after the pool change; see [the sync](meetings/sync.md#L12-L20).
</hick:assistant>
<hick:wrote file="notes/today.md" lines="72-75" hashes="a b c d"/>
</hick:session>
"#;

</storycode:copy>
<a id="f-21c9526a7692-the_session_view_carries_blocks_and_the_three_families_of_links"></a>
### `the_session_view_carries_blocks_and_the_three_families_of_links`

<storycode:copy id="f-21c9526a7692-the_session_view_carries_blocks_and_the_three_families_of_links">
#[tokio::test]
async fn the_session_view_carries_blocks_and_the_three_families_of_links() {
    let _env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("notes")).unwrap();
    std::fs::create_dir_all(root.join("sessions")).unwrap();
    std::fs::write(root.join("notes/today.md"), "# today\n").unwrap();
    std::fs::write(root.join("sessions/s.md"), SESSION).unwrap();
    unsafe { std::env::set_var("HICKORY_STATE_DIR", state_dir.path()) };
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    let res = reqwest::get(format!(
        "http://127.0.0.1:{port}/api/sessions/view?path=sessions/s.md"
    ))
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["path"], "sessions/s.md");
    assert!(body["source"].as_str().unwrap().contains("<hick:user"));

    let kinds: Vec<&str> = body["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        vec![
            "session-user",
            "session-assistant",
            "session-tool",
            "session-tool-result",
            "session-read",
            "session-assistant",
            "session-wrote"
        ]
    );

    let links: Vec<(String, String, Value, u64)> = body["links"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["family"].as_str().unwrap().to_string(),
                l["to"]["path"].as_str().unwrap().to_string(),
                l["to"]["lines"].clone(),
                l["lines"][0].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        links,
        vec![
            ("context".into(), "meetings/sync.md".into(), Value::Null, 4),
            (
                "context".into(),
                "data/latency.csv".into(),
                serde_json::json!([1, 8]),
                11
            ),
            (
                "declared".into(),
                "meetings/sync.md".into(),
                serde_json::json!([12, 20]),
                13
            ),
            (
                "lineage".into(),
                "notes/today.md".into(),
                serde_json::json!([72, 75]),
                15
            ),
        ]
    );
    // Retrospective corrections survive reopening and never erase the original.
    let client = reqwest::Client::new();
    let endpoint = format!("http://127.0.0.1:{port}/api/sessions/audit");
    let response = client.post(&endpoint).json(&serde_json::json!({
        "path":"sessions/s.md", "entry":{
            "statement":"Checkout delay was caused by the pool change",
            "reason":"This was the explanation originally used", "dependencies":["notes/today.md"]
        }
    })).send().await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let view_url = format!("http://127.0.0.1:{port}/api/sessions/view?path=sessions/s.md");
    let before: Value = client
        .get(&view_url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let original = before["audit"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["record_kind"] == "audit")
        .unwrap();
    assert_eq!(original["actor"], "user-declared");
    let id = original["id"].as_str().unwrap();
    std::fs::write(root.join("notes/today.md"), "# corrected context\n").unwrap();
    let response = client.post(&endpoint).json(&serde_json::json!({
        "path":"sessions/s.md", "entry":{
            "statement":"The cause is still unresolved", "reason":"The pool explanation lacks a controlled witness", "supersedes":id
        }
    })).send().await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let after: Value = client
        .get(&view_url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let declarations: Vec<_> = after["audit"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["record_kind"] == "audit")
        .collect();
    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0]["id"], id);
    assert_eq!(
        declarations[0]["dependency_snapshots"][0]["current_state"],
        "changed"
    );
    assert_eq!(declarations[1]["entry"]["supersedes"], id);
    assert!(
        after["source"]
            .as_str()
            .unwrap()
            .contains("Why is checkout slow?")
    );
    let invalid = client.post(&endpoint).json(&serde_json::json!({
        "path":"sessions/s.md", "entry":{"statement":"Correction", "reason":"Check", "supersedes":"nonexistent"}
    })).send().await.unwrap();
    assert_eq!(invalid.status().as_u16(), 422);
}

</storycode:copy>
<a id="f-21c9526a7692-a_literal_acp_answer_gets_current_output_evidence_only_when_disk_bytes_match"></a>
### `a_literal_acp_answer_gets_current_output_evidence_only_when_disk_bytes_match`

<storycode:copy id="f-21c9526a7692-a_literal_acp_answer_gets_current_output_evidence_only_when_disk_bytes_match">
// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
#[tokio::test]
async fn a_literal_acp_answer_gets_current_output_evidence_only_when_disk_bytes_match() {
    let _env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::create_dir(root.join("sessions")).unwrap();
    let document = "# Producer\n<hick:file path=\"test2.py\">\nprint('hello')\n</hick:file>\n";
    std::fs::write(root.join("test2.md"), document).unwrap();
    std::fs::write(root.join("test2.py"), "print('hello')\n").unwrap();
    let source = format!(
        r#"<hick:session><hick:user turn="one" provider="acp:codex">Which document?</hick:user>
<hick:assistant><hick:input>
[test2.md]({}:2) writes `test2.py` through `<hick:file path="test2.py">`.
</hick:input></hick:assistant></hick:session>"#,
        root.join("test2.md").display()
    );
    std::fs::write(root.join("sessions/s.md"), &source).unwrap();
    unsafe { std::env::set_var("HICKORY_STATE_DIR", state_dir.path()) };
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let url = format!("http://127.0.0.1:{port}/api/sessions/view?path=sessions/s.md");
    let body: Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
    assert_eq!(body["blocks"][1]["kind"], "session-assistant");
    assert!(
        body["blocks"][1]["body"]
            .as_str()
            .unwrap()
            .contains("<hick:file")
    );
    assert_eq!(body["links"][0]["family"], "declared");
    assert_eq!(body["links"][0]["to"]["path"], "test2.md");
    let proof = body["links"]
        .as_array()
        .unwrap()
        .iter()
        .find(|link| link.get("evidence").is_some())
        .unwrap_or_else(|| panic!("current output lineage: {body:#}"));
    assert_eq!(proof["family"], "lineage");
    assert_eq!(proof["to"]["path"], "test2.py");
    assert_eq!(proof["evidence"]["path"], "test2.md");
    assert_eq!(proof["evidence"]["lines"], serde_json::json!([3, 3]));
    std::fs::write(root.join("test2.py"), "changed outside document\n").unwrap();
    let changed: Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
    assert!(
        changed["links"]
            .as_array()
            .unwrap()
            .iter()
            .all(|link| link.get("evidence").is_none())
    );
    assert_eq!(
        std::fs::read_to_string(root.join("sessions/s.md")).unwrap(),
        source
    );
    server.abort();
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/session_lens.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/session_lens.rs"><storycode:paste select="#f-21c9526a7692-ENV_LOCK"/><storycode:paste select="#f-21c9526a7692-SESSION"/><storycode:paste select="#f-21c9526a7692-the_session_view_carries_blocks_and_the_three_families_of_links"/><storycode:paste select="#f-21c9526a7692-a_literal_acp_answer_gets_current_output_evidence_only_when_disk_bytes_match"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [apps/web/src/elements/session/view.tsx](../../../mechanisms/apps/web/src/elements/session/view.tsx.md#output-d096a8a10df6)
- [apps/web/src/lib/conversationLineage.ts](contract.md#output-db697b55388e)
- [apps/web/src/lib/lensSources.ts](contract.md#output-f20ae3d80724)
- [apps/web/src/views/SessionLens.tsx](../../../mechanisms/apps/web/src/views/SessionLens.tsx.md#output-11b8d4e005a0)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [crates/hick-blocks/src/lib.rs](../../../mechanisms/crates/hick-blocks/src/lib.rs.md#output-5260198b0a7b)
- [crates/hick-literate/src/session_elements.rs](contract.md#output-463c594a8afa)
- [crates/hick-literate/src/session_receipts.rs](../../../mechanisms/crates/hick-literate/src/session_receipts.rs.md#output-4e6c982edf60)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/sessions.rs](contract.md#output-af680583ad64)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/elements/index.test.ts](../../language/an-element-is-drawn-by-its-view/contract.md#output-e188c4134b98)
- [apps/web/src/lib/conversationLineage.test.ts](contract.md#output-4916fa68abf9)
- [apps/web/src/lib/editorBufferTargets.test.ts](contract.md#output-c1a7c85841b2)
- [apps/web/src/lib/lensSources.test.ts](contract.md#output-96157ff2afcc)
- [apps/web/src/lib/offsets.test.ts](../../../mechanisms/apps/web/src/lib/offsets.test.ts.md#output-dea322d4979a)
- [apps/web/src/shell/linkShapes.test.ts](../../../mechanisms/apps/web/src/shell/linkShapes.ts.md#output-04fb400a1dba)
- [apps/web/src/views/ConversationEditor.test.tsx](../../../mechanisms/apps/web/src/views/ConversationEditor.tsx.md#output-edc92cd336d7)
- [apps/web/src/views/SessionLens.test.tsx](../../../mechanisms/apps/web/src/views/SessionLens.tsx.md#output-3a59c565f3d4)
- [apps/web/src/views/workspaceRibbonSources.test.ts](contract.md#output-77f4467cff5b)
- [crates/hickory-cli/tests/session_lens.rs](contract.md#output-21c9526a7692)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
