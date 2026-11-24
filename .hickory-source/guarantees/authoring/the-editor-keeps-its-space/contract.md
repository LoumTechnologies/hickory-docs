<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The editor keeps its space

For: Authors editing executable documents and the files those documents produce.

Given the desktop workspace, when editing a document or plain file, then view
identity is conveyed by the tab and content rather than an explanatory information
control. One search icon in the pane header opens a modal with replacement for
writable buffers; conversation cards share one search control. Empty audit and
unrecorded check summaries are hidden. Composer tools are icons with accessible
names, and the platform-specific send shortcut appears on hover. Repository setup
warnings live in the status bar. The tab-strip ellipsis lists open tabs; file
actions use a distinct menu, and file/tab context menus offer revision history
and relative/absolute path copying. Creating a document belongs after the file
list. Document-wide Run/Test and plain-file Debug live in its
header’s action menu. Cell actions remain on the right-hand action rail;
crowded gutter actions open a floating grid without displacing later rows. A document with no rail actions reserves
no empty action column; adding a cell restores its actions. Editor instances, source bytes and
selections are not replaced by moving their controls.

Focused verification on 2026-11-06: 107 selected frontend tests passed across
conversation editing, Files replacement, file history, menus, save races and
plain/generated editor guards. Frontend type checking passed. The disposable
native-host browser witness (`bash scripts/test-spatial-ui.sh --compact`) passed
modal replacement and undo, retained draft text, one search per surface,
status-bar warning placement, root recipe popover visibility and unchanged
editor/tree geometry. Its merge-driver status is a presentation-boundary stub;
it does not establish repair success. Actual initialization replies remain
covered by `MergeDriverNotice.test.tsx`. These checks do not certify the packaged
desktop build or every inherited UI surface. Broad regressions were not run.

## Recipe buttons and gutter overflow

Saved justfile and .justfile recipes appear on their containing folder’s gutter
row in Files, including the workspace root’s read-only header. Discovery uses
just’s parse-only JSON dump; it must not evaluate assignments or run recipes.
Public recipes from imports and modules are included. Clicking runs the selected
recipe in a named terminal with its folder as the working directory. Parameterized
recipes ask for parameter values; cancellation starts nothing. Tree drafts retire
recipe markers so renamed text cannot select a different execution directory.

Across editors, gutter icons share one vertical column at the gutter’s right edge.
A stack may extend down only until the next populated row. If it cannot fit, its
last available slot becomes a three-dot button opening a floating grid of the
remaining icons. Opening it does not move text or later gutter rows. Escape,
outside pointer presses, scrolling and editor updates dismiss the grid. This
supersedes downward collision-pushing for both CodeMirror gutter icons and the
right-hand cell action rail.

Evidence is scoped to the focused recipe unit tests, native metadata tests and
`scripts/test-just-gutters.py`. The Python witness checks actual native discovery,
recipe output, root controls and browser geometry. Existing marker semantics such
as breakpoint dragging need their own focused checks; this change does not claim
that all inherited markers have accessible names.

Verification on 2026-11-04: the native Python/browser witness passed recipe
non-evaluation, contained-path refusal, a real terminal’s output in the selected
folder, root-header controls, Files grid geometry, document-action overflow,
normal pointer hit-testing, and Escape/outside dismissal. The 27 targeted
frontend tests cover shared slot arithmetic, source retirement, literal parameter
arguments, document callbacks, debugger ownership and controller behavior; five
existing document rail witnesses and six native plain-file/metadata safety tests
also passed. Frontend type checking and the literate check/audit were run
separately. Node 26 needed `NODE_OPTIONS=--no-experimental-webstorage` for the
existing jsdom storage setup. Broad regressions were not run.

Follow the [gutter construction](../../../mechanisms/apps/web/src/editor/gutterActions.ts.md)
and [document action rail](../../../mechanisms/apps/web/src/editor/CardRail.tsx.md)
when maintaining this behavior. The native witness runs with
`bash scripts/test-spatial-ui.sh --just-gutters`; it starts and stops a disposable
host and browser. It uses uv’s Playwright environment by default, or the Python
executable named by `HICKORY_SYSTEM_PYTHON` when Playwright is already installed.


The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-editor-keeps-its-space.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-a4f9707ac652"></a>
## `apps/web/src/editor/chrome.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-a4f9707ac652-ChromeKind"></a>
### `ChromeKind`

One set of editor chrome, for every editor in the app.
This exists because of a bug that was really a structural mistake. The
gutters, the cursor, the selection, and the find panel were styled by rules
scoped to `.document-editor` — so a `.md` document had a dark gutter with
light numbers, and every OTHER editor (a `.py` file, a `.md` file, a woven
output) silently fell through to CodeMirror's built-in theme, which is
light. A white gutter with dark numbers, in a dark window.
Scoping the chrome to one editor was the mistake; the white gutter was only
how it showed up. So the chrome is an EXTENSION now, not a stylesheet
selector: every editor loads it, and an editor that forgets to has no
chrome at all rather than quietly inheriting somebody else's.
## Why a CodeMirror theme rather than more CSS
CodeMirror's base theme is injected into the document with its own
specificity, and a plain rule in styles.css loses to it in the places that
matter most (`.cm-panels`, `.cm-tooltip`, the selection layer). A theme
extension is emitted through the same mechanism at the same specificity, so
it wins where a rule would not — and it travels with the extension list,
which is the property that makes "every editor looks the same" true by
construction instead of by discipline.
## The two kinds
A `.md` document is prose first: proportional type, generous leading,
because most of what is in it is sentences. Everything else is code, and
it is drawn the way the SAME code looks when it is inside a document — the
sunken ground and the accent rule of a `hick:file` body (see `.cm-file-line`
in styles.css). That is the point: opening `orders.py` and reading the
`<hick:file path="orders.py">` block that writes it should not feel like
two different programs.

<storycode:copy id="f-a4f9707ac652-ChromeKind">
// One set of editor chrome, for every editor in the app.
//
// This exists because of a bug that was really a structural mistake. The
// gutters, the cursor, the selection, and the find panel were styled by rules
// scoped to `.document-editor` — so a `.md` document had a dark gutter with
// light numbers, and every OTHER editor (a `.py` file, a `.md` file, a woven
// output) silently fell through to CodeMirror's built-in theme, which is
// light. A white gutter with dark numbers, in a dark window.
//
// Scoping the chrome to one editor was the mistake; the white gutter was only
// how it showed up. So the chrome is an EXTENSION now, not a stylesheet
// selector: every editor loads it, and an editor that forgets to has no
// chrome at all rather than quietly inheriting somebody else's.
//
// ## Why a CodeMirror theme rather than more CSS
//
// CodeMirror's base theme is injected into the document with its own
// specificity, and a plain rule in styles.css loses to it in the places that
// matter most (`.cm-panels`, `.cm-tooltip`, the selection layer). A theme
// extension is emitted through the same mechanism at the same specificity, so
// it wins where a rule would not — and it travels with the extension list,
// which is the property that makes "every editor looks the same" true by
// construction instead of by discipline.
//
// ## The two kinds
//
// A `.md` document is prose first: proportional type, generous leading,
// because most of what is in it is sentences. Everything else is code, and
// it is drawn the way the SAME code looks when it is inside a document — the
// sunken ground and the accent rule of a `hick:file` body (see `.cm-file-line`
// in styles.css). That is the point: opening `orders.py` and reading the
// `<hick:file path="orders.py">` block that writes it should not feel like
// two different programs.

import { Decoration, EditorView, ViewPlugin, keymap } from "@codemirror/view";
import { Prec, type Extension } from "@codemirror/state";
import { foldedRanges, unfoldEffect } from "@codemirror/language";
import { SearchQuery, search, setSearchQuery, replaceAll } from "@codemirror/search";
import { gutterIconLayout } from "./gutterIconLayout";

/** Which of the two looks an editor wears. */
export type ChromeKind = "document" | "code";

</storycode:copy>
<a id="f-a4f9707ac652-editorChrome"></a>
### `editorChrome`

Local search uses one modal for each pane. Conversation cards register their separate editors under the conversation host, preserving independent undo and read-only boundaries. Match case, whole-word and regular-expression queries use CodeMirror’s matcher and replacement command. Replacement is offered only for writable editors; the editor’s transaction filters remain authoritative. Search controls join the pane header when available and never overlay each recorded turn.

The shared chrome.
Everything here reads a token, never a literal colour, so all three themes
follow without a second definition.

Files and literate documents share a search icon and native modal. Typing marks
all literal, case-insensitive matches in the live buffer, and opens folds containing
matches so results can be seen. Files searches row names rather than file contents.
Closing the modal retains highlights; clearing its field removes them. Native modal
focus containment and Escape dismissal keep keyboard interaction consistent.

<storycode:copy id="f-a4f9707ac652-editorChrome">
/**
 * The shared chrome.
 *
 * Everything here reads a token, never a literal colour, so all three themes
 * follow without a second definition.
 */
export function editorChrome(kind: ChromeKind): Extension {
  const code = kind === "code";
  return [gutterIconLayout, EditorView.theme({
    "&": {
      color: "var(--fg)",
      backgroundColor: "transparent",
      // The height belongs to the pane; an editor that sized itself would
      // fight the shell's grid.
      height: "100%",
    },
    "&.cm-focused": { outline: "none" },
    ".cm-scroller": {
      fontFamily: code
        ? 'ui-monospace, "SF Mono", Menlo, Consolas, monospace'
        : 'system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
      lineHeight: code ? "var(--editor-code-leading, 1.4)" : "var(--editor-document-leading, 1.45)",
    },
    ".cm-content": { "--cm-pad-x": "var(--editor-pad-x, .75rem)", caretColor: "var(--fg)", padding: "var(--editor-pad-top, .5rem) var(--editor-pad-x, .75rem) var(--editor-pad-bottom, .75rem)" },

    // -- the gutters, which is where this started ---------------------------
    ".cm-gutters": {
      backgroundColor: "transparent",
      color: "var(--fg-muted)",
      border: "none",
      // The numbers are always monospace, even beside proportional prose:
      // a column of digits that does not line up is not a column.
      fontFamily: 'ui-monospace, "SF Mono", Menlo, Consolas, monospace',
      fontSize: "0.78em",
    },
    ".cm-lineNumbers .cm-gutterElement": { padding: "0 0.5em 0 var(--editor-number-pad, .5em)" },
    ".cm-activeLineGutter": {
      backgroundColor: "transparent",
      color: "var(--fg)",
    },
    ".cm-foldGutter .cm-gutterElement": {
      minWidth: "1.5rem",
      padding: "0 0.25rem 0 0.6rem",
    },

    // -- caret, selection, active line --------------------------------------
    ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--fg)" },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": {
      backgroundColor: "var(--select-bg, rgba(110, 160, 255, 0.28))",
    },
    ".cm-activeLine": { backgroundColor: "transparent" },
    ".cm-selectionMatch": {
      backgroundColor: "var(--select-bg, rgba(110, 160, 255, 0.18))",
    },

    // -- the find panel, which was the other place the light theme leaked ---
    ".cm-panels": {
      backgroundColor: "var(--bg-raised)",
      color: "var(--fg)",
      borderBottom: "1px solid var(--border)",
      fontFamily: 'system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
      fontSize: "0.8rem",
    },
    ".cm-panels.cm-panels-bottom": {
      borderTop: "1px solid var(--border)",
      borderBottom: "none",
    },
    ".cm-panel input, .cm-panel button, .cm-panel select": {
      backgroundColor: "var(--bg)",
      color: "var(--fg)",
      border: "1px solid var(--border)",
      borderRadius: "4px",
      padding: "0.1rem 0.35rem",
      font: "inherit",
    },
    ".cm-panel button:hover": { borderColor: "var(--accent)" },
    ".cm-panel label": { color: "var(--fg-muted)" },
    ".cm-searchMatch": {
      backgroundColor: "var(--search-hit, rgba(217, 161, 63, 0.32))",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "var(--search-hit-on, rgba(217, 161, 63, 0.6))",
    },

    // -- tooltips and completion --------------------------------------------
    ".cm-tooltip": {
      backgroundColor: "var(--bg-raised)",
      color: "var(--fg)",
      border: "1px solid var(--border)",
      borderRadius: "6px",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]": {
      backgroundColor: "var(--accent)",
      color: "var(--bg)",
    },
  })];
}

// One modal per working surface. Conversation cards share it without sharing
// their editor state, so searching never reconstructs the draft or its history.
const paneSearchGroups = new WeakMap<HTMLElement, {
  views: Set<EditorView>; query: SearchQuery; open: () => void; refresh: () => void; destroy: () => void;
}>();
export function paneSearch(label: string): Extension {
  const plugin = ViewPlugin.fromClass(class {
    decorations = Decoration.none;
    group: NonNullable<ReturnType<typeof paneSearchGroups.get>>;
    host: HTMLElement;
    constructor(readonly view: EditorView) {
      this.host = view.dom.closest<HTMLElement>(".conversation-document") ?? view.dom;
      let group = paneSearchGroups.get(this.host);
      if (!group) {
        const views = new Set<EditorView>();
        const button = document.createElement("button");
        button.type = "button";
        const name = this.host === view.dom ? label : "conversation";
        button.setAttribute("aria-label", `Search ${name}`);
        button.title = `Search ${name}`;
        button.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/></svg>';
        button.style.cssText = "background:transparent;color:var(--fg);border:0;border-radius:4px;padding:5px;cursor:pointer";
        const toolbar = document.createElement("div");
        toolbar.className = "pane-search-tools";
        toolbar.style.cssText = "display:flex;justify-content:flex-end;position:sticky;top:0;z-index:5;height:28px;flex-shrink:0";
        toolbar.append(button);
        const header = this.host.closest(".shell-pane")?.querySelector(".pane-header-actions");
        if (header) header.append(toolbar); else this.host.prepend(toolbar);
        const dialog = document.createElement("dialog");
        dialog.setAttribute("aria-label", `Search ${name}`);
        dialog.style.cssText = "background:var(--bg-raised);color:var(--fg);border:1px solid var(--border);border-radius:8px;padding:16px;width:min(480px,80vw)";
        const input = document.createElement("input"), replacement = document.createElement("input");
        input.type = "search"; input.placeholder = "Find"; input.setAttribute("aria-label", `Search ${name} text`);
        replacement.placeholder = "Replace with"; replacement.setAttribute("aria-label", "Replace with");
        const status = document.createElement("p"), options = document.createElement("div"), controls = document.createElement("div");
        status.setAttribute("role", "status");
        options.style.cssText = controls.style.cssText = "display:flex;gap:8px;margin-top:8px;align-items:center";
        const toggles: HTMLInputElement[] = [];
        for (const name of ["Match case", "Whole word", "Regular expression"]) {
          const label = document.createElement("label"), toggle = document.createElement("input");
          toggle.type = "checkbox"; toggle.setAttribute("aria-label", name); toggles.push(toggle); label.append(toggle, name); options.append(label);
        }
        const action = (name: string, run: () => void) => { const control = document.createElement("button"); control.type = "button"; control.textContent = name; control.onclick = run; controls.append(control); return control; };
        let current: { view: EditorView; from: number; to: number } | null = null;
        const matches = () => {
          const found: { view: EditorView; from: number; to: number }[] = [];
          if (!group!.query.valid) return found;
          for (const view of views) {
            const cursor = group!.query.getCursor(view.state);
            for (let match = cursor.next(); !match.done; match = cursor.next()) found.push({ view, ...match.value });
          }
          return found;
        };
        const refresh = () => {
          const found = matches();
          status.textContent = input.value && !group!.query.valid ? "Invalid expression" : `${found.length} matches`;
          const writable = [...views].some(view => !view.state.readOnly);
          replacement.hidden = replace.hidden = !writable;
          replace.disabled = !group!.query.valid;
          replace.title = replace.disabled ? "Fix the search expression before replacing matches." : "";
        };
        const navigate = (direction: number) => {
          const found = matches(); if (!found.length) return;
          const index = current ? found.findIndex(match => match.view === current!.view && match.from === current!.from && match.to === current!.to) : direction > 0 ? -1 : 0;
          current = found[(index + direction + found.length) % found.length];
          const { view, from, to } = current;
          const effects = [];
          foldedRanges(view.state).between(0, view.state.doc.length, (start, end) => { if (start < to && end > from) effects.push(unfoldEffect.of({ from: start, to: end })); });
          effects.push(EditorView.scrollIntoView(from, { y: "center" }));
          view.dispatch({ selection: { anchor: from, head: to }, effects });
        };
        action("Previous", () => navigate(-1)); action("Next", () => navigate(1));
        const replace = action("Replace all", () => {
          for (const view of views) if (!view.state.readOnly) { view.dispatch({ effects: setSearchQuery.of(group!.query) }); replaceAll(view); }
          current = null; refresh();
        });
        action("Close", () => dialog.close());
        const update = () => {
          group!.query = new SearchQuery({ search: input.value, replace: replacement.value, caseSensitive: toggles[0].checked, wholeWord: toggles[1].checked, regexp: toggles[2].checked });
          current = null;
          for (const view of views) view.dispatch({ effects: setSearchQuery.of(group!.query) });
          refresh();
        };
        input.oninput = replacement.oninput = update; toggles.forEach(toggle => { toggle.onchange = update; });
        input.onkeydown = event => { if (event.key === "Enter") { event.preventDefault(); navigate(event.shiftKey ? -1 : 1); } };
        dialog.onclose = () => button.focus();
        dialog.append(input, replacement, options, status, controls); document.body.append(dialog);
        group = { views, query: new SearchQuery({ search: "" }), refresh, open: () => { refresh(); if (!dialog.open) dialog.showModal(); input.focus(); }, destroy: () => { toolbar.remove(); dialog.remove(); } };
        paneSearchGroups.set(this.host, group); button.onclick = group.open;
      }
      this.group = group; group.views.add(view);
    }
    open() { this.group.open(); }
    update() {
      const marks = [], query = this.group.query;
      if (query.valid) {
        const cursor = query.getCursor(this.view.state);
        for (let match = cursor.next(); !match.done; match = cursor.next()) {
          if (match.value.to > match.value.from) marks.push(Decoration.mark({ class: "cm-searchMatch" }).range(match.value.from, match.value.to));
        }
      }
      this.decorations = Decoration.set(marks); this.group.refresh();
    }
    destroy() { this.group.views.delete(this.view); if (!this.group.views.size) { this.group.destroy(); paneSearchGroups.delete(this.host); } }
  }, { decorations: value => value.decorations });
  return [search(), plugin, Prec.highest(keymap.of([{ key: "Mod-f", run: view => { view.plugin(plugin)?.open(); return true; } }]))];
}
</storycode:copy>

### Assemble `apps/web/src/editor/chrome.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/chrome.ts"><storycode:paste select="#f-a4f9707ac652-ChromeKind"/><storycode:paste select="#f-a4f9707ac652-editorChrome"/></storycode:file>

<a id="output-abce6d0421a3"></a>
## `apps/web/src/shell/PaneHeader.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-abce6d0421a3-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-abce6d0421a3-expression_statement">
import { afterEach, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { PaneHeaderProvider, PaneHeaderSlots, PaneHeaderContent } from "./PaneHeader";
import { SurfaceInfo } from "../components/SurfaceInfo";

afterEach(cleanup);
</storycode:copy>
<a id="f-abce6d0421a3-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-abce6d0421a3-expression_statement-2">
// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("keeps info and actions in the header without recreating the editing surface", () => {
  function Pane() {
    const [label, setLabel] = useState("old");
    return <PaneHeaderProvider><header data-testid="header"><PaneHeaderSlots /></header><main>
      <SurfaceInfo name="Note" editable destination="Saves to note.md" />
      <PaneHeaderContent part="actions"><button onClick={() => setLabel("new")}>{label}</button></PaneHeaderContent>
      <textarea aria-label="Source" defaultValue="unchanged" />
    </main></PaneHeaderProvider>;
  }
  render(<Pane />);
  const source = screen.getByRole("textbox") as HTMLTextAreaElement;
  source.setSelectionRange(1, 4);
  expect(screen.getByTestId("header").contains(screen.getByRole("button", { name: "old" }))).toBe(true);
  expect(screen.getByTestId("header").querySelector(".surface-info")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "old" }));
  expect(screen.getByRole("textbox")).toBe(source);
  expect(source.value).toBe("unchanged");
  expect([source.selectionStart, source.selectionEnd]).toEqual([1, 4]);
});
</storycode:copy>

### Assemble `apps/web/src/shell/PaneHeader.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/PaneHeader.test.tsx"><storycode:paste select="#f-abce6d0421a3-expression_statement"/><storycode:paste select="#f-abce6d0421a3-expression_statement-2"/></storycode:file>

<a id="output-376a93f02516"></a>
## `apps/web/src/shell/PaneHeader.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-376a93f02516-Part"></a>
### `Part`

<storycode:copy id="f-376a93f02516-Part">
import { createContext, useContext, useMemo, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import "./paneHeader.css";

type Part = "info" | "actions";
</storycode:copy>
<a id="f-376a93f02516-HeaderSlots"></a>
### `HeaderSlots`

<storycode:copy id="f-376a93f02516-HeaderSlots">
interface HeaderSlots {
  info: HTMLElement | null;
  actions: HTMLElement | null;
  setInfo: (node: HTMLDivElement | null) => void;
  setActions: (node: HTMLDivElement | null) => void;
}
</storycode:copy>
<a id="f-376a93f02516-HeaderContext"></a>
### `HeaderContext`

<storycode:copy id="f-376a93f02516-HeaderContext">
const HeaderContext = createContext<HeaderSlots | null>(null);

</storycode:copy>
<a id="f-376a93f02516-PaneHeaderProvider"></a>
### `PaneHeaderProvider`

Move only chrome into a pane's existing header. Its editor stays mounted.

<storycode:copy id="f-376a93f02516-PaneHeaderProvider">
/** Move only chrome into a pane's existing header. Its editor stays mounted. */
export function PaneHeaderProvider({ children }: { children: ReactNode }) {
  const [info, setInfo] = useState<HTMLDivElement | null>(null);
  const [actions, setActions] = useState<HTMLDivElement | null>(null);
  const slots = useMemo(() => ({ info, actions, setInfo, setActions }), [info, actions]);
  return <HeaderContext.Provider value={slots}>{children}</HeaderContext.Provider>;
}

</storycode:copy>
<a id="f-376a93f02516-PaneHeaderSlots"></a>
### `PaneHeaderSlots`

<storycode:copy id="f-376a93f02516-PaneHeaderSlots">
export function PaneHeaderSlots() {
  const header = useContext(HeaderContext);
  return <div className="pane-header-slots" onPointerDown={event => event.stopPropagation()}>
    <div className="pane-header-info" ref={header?.setInfo} />
    <div className="pane-header-actions" ref={header?.setActions} />
  </div>;
}

</storycode:copy>
<a id="f-376a93f02516-PaneHeaderContent"></a>
### `PaneHeaderContent`

Standalone/embedded surfaces retain their own chrome if there is no shell.

<storycode:copy id="f-376a93f02516-PaneHeaderContent">
/** Standalone/embedded surfaces retain their own chrome if there is no shell. */
export function PaneHeaderContent({ part, children }: { part: Part; children: ReactNode }) {
  const header = useContext(HeaderContext);
  return header?.[part] ? createPortal(children, header[part]!) : <>{children}</>;
}
</storycode:copy>

### Assemble `apps/web/src/shell/PaneHeader.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/PaneHeader.tsx"><storycode:paste select="#f-376a93f02516-Part"/><storycode:paste select="#f-376a93f02516-HeaderSlots"/><storycode:paste select="#f-376a93f02516-HeaderContext"/><storycode:paste select="#f-376a93f02516-PaneHeaderProvider"/><storycode:paste select="#f-376a93f02516-PaneHeaderSlots"/><storycode:paste select="#f-376a93f02516-PaneHeaderContent"/></storycode:file>

<a id="output-c09b7ea67dd0"></a>
## `apps/web/src/shell/paneHeader.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/shell/paneHeader.css">
.pane-header-slots { display: flex; align-items: center; flex: none; gap: 4px; }
.pane-header-info, .pane-header-actions { display: flex; align-items: center; min-width: 0; }
.spatial-view-header:has(.pane-header-slots) { overflow: visible; }
.pane-header-slots .doc-tab-toolbar { padding: 0; background: transparent; border: 0; font-size: 12px; }
.pane-header-slots .doc-tab-toolbar .btn { padding: 2px 5px; font-size: 12px; }
.pane-header-slots .surface-info { position: relative; padding: 0; border: 0; font-size: 12px; }
.pane-header-slots .surface-info > summary { padding: 4px; }
.pane-header-slots .surface-info > summary::before { content: "ⓘ"; }
.pane-header-slots .surface-info[open] > summary::before { transform: none; }
.pane-header-slots .surface-info > summary > strong,
.pane-header-slots .surface-info > summary > span { display: none; }
.pane-header-slots .surface-info > p { position: absolute; z-index: 30; right: 0; top: 24px; width: min(320px, 75vw); box-sizing: border-box; margin: 0; padding: 10px; background: var(--bg-raised); color: var(--fg); border: 1px solid var(--border); border-radius: 4px; box-shadow: var(--shadow); white-space: normal; }
.pane-header-slots .save-state { max-width: 8rem; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
</storycode:entry>

</storycode:doc>
