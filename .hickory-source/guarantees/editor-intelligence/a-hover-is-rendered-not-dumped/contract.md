<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Hover Is Rendered, Not Dumped

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a language server's answer to `textDocument/hover`, when the tooltip is
drawn, then its Markdown is **rendered**: a fenced block becomes a code block
without its backticks or its language word, `---` becomes a rule, a link
becomes its own text, and inline code and emphasis become `<code>`, `<strong>`
and `<em>`. Prose is drawn in the proportional UI font; the fenced signature
stays monospace.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-hover-is-rendered-not-dumped.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-0f1f50575dc9"></a>
## `apps/web/src/lsp/cmLsp.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Hover Is Rendered, Not Dumped](contract.md).

<a id="f-0f1f50575dc9-LspNavigationTarget"></a>
### `LspNavigationTarget`

<storycode:copy id="f-0f1f50575dc9-LspNavigationTarget">
import { editorLanguageActions } from "../editor/languageActions";
import type { UiOutcome } from "../lib/uiActions";
import { cmKeyOf } from "../lib/keymap";
// CodeMirror bindings for the LSP bridge: hover, go-to-definition, find
// references, and diagnostics.
//
// LAYOUT RULE (the one that matters here): diagnostics are drawn as MARK
// decorations only — a squiggly underline via background-image, never a border
// or a block widget. Anything that changes a line's height would have to be a
// StateField and would still perturb the ribbon geometry in Split view, so
// this deliberately stays background-only.
//
// Navigation is coordinate-agnostic: callers give `positionAt`, which converts
// a CodeMirror offset into a position in the DOCUMENT's coordinate space. The
// document editor passes an identity mapping; the output editor maps through
// provenance first, so Cmd-clicking a symbol in generated code asks the same
// question about the prose that produced it.

import { RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import type { Extension, Text } from "@codemirror/state";
import { Decoration, EditorView, hoverTooltip, keymap } from "@codemirror/view";
import { renderHoverMarkdown } from "./hoverMarkdown";
import type { DecorationSet, Tooltip } from "@codemirror/view";
import type { LspClient, LspDiagnostic, LspLocation } from "./client";
import type { LspPosition } from "./positions";

export interface LspNavigationTarget {
  uri: string;
  range: { start: LspPosition; end: LspPosition };
}

</storycode:copy>
<a id="f-0f1f50575dc9-CmLspOptions"></a>
### `CmLspOptions`

<storycode:copy id="f-0f1f50575dc9-CmLspOptions">
export interface CmLspOptions {
  client: LspClient | null;
  /** Document URI every request is asked against (`hick:///<doc-path>`). */
  uri: string;
  /**
   * Map a CodeMirror offset in THIS editor to a position in the document's
   * coordinate space, or null when this offset has no source origin (a
   * synthetic byte the weaver produced).
   */
  positionAt: (offset: number, view: EditorView) => LspPosition | null;
  /** Resolve a local construction byte into its selected complete endpoint. */
  targetAt?: (offset: number, view: EditorView) => { uri: string; position: LspPosition } | null;
  /** Open a definition/reference target. */
  onNavigate: (target: LspNavigationTarget) => void;
  /** Show a reference list (empty array means "none found"). */
  onReferences?: (locations: LspLocation[], from: LspNavigationTarget) => void;
  /**
   * What this identifier currently HOLDS, when a debugger is paused.
   *
   * Resolves to null when nothing is paused, which is the usual case. The
   * value goes above the type in the tooltip: pointing at a symbol while
   * stopped is usually a question about its value, and the type is the
   * context for it.
   */
  runtimeValue?: (identifier: string) => Promise<string | null>;
}

</storycode:copy>
<a id="f-0f1f50575dc9-DiagnosticSpan"></a>
### `DiagnosticSpan`

--- diagnostics ------------------------------------------------------------
One diagnostic, in editor offsets, with the text a person needs to read.

<storycode:copy id="f-0f1f50575dc9-DiagnosticSpan">
// --- diagnostics ------------------------------------------------------------

/** One diagnostic, in editor offsets, with the text a person needs to read. */
export interface DiagnosticSpan {
  from: number;
  to: number;
  severity: number;
  /** What the server said. Carried through so hovering the squiggle can
      show it — an underline with no message tells you where the problem is
      and nothing about what it is. */
  message?: string;
  /** Which server said it (`typescript`, `basedpyright`), so a document with
      several languages does not leave you guessing. */
  source?: string;
}

</storycode:copy>
<a id="f-0f1f50575dc9-setLspDiagnostics"></a>
### `setLspDiagnostics`

<storycode:copy id="f-0f1f50575dc9-setLspDiagnostics">
export const setLspDiagnostics = StateEffect.define<DiagnosticSpan[]>();

</storycode:copy>
<a id="f-0f1f50575dc9-lspDiagnosticField"></a>
### `lspDiagnosticField`

<storycode:copy id="f-0f1f50575dc9-lspDiagnosticField">
export const lspDiagnosticField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(setLspDiagnostics)) {
        const builder = new RangeSetBuilder<Decoration>();
        const sorted = [...e.value].sort((a, b) => a.from - b.from || a.to - b.to);
        const max = tr.state.doc.length;
        for (const d of sorted) {
          const from = Math.min(d.from, max);
          const to = Math.min(d.to, max);
          if (to <= from) continue;
          builder.add(
            from,
            to,
            // The message rides along on the decoration's spec, which is how
            // the hover below finds it without a second data structure to
            // keep in step with this one.
            Decoration.mark({
              // Information and hints are marked too — a refused tool, a
              // command that exited 1 — but quietly: they are findable,
              // not counted (lib/problems.ts).
              class:
                d.severity === 1
                  ? "cm-lsp-error"
                  : d.severity === 2
                    ? "cm-lsp-warn"
                    : "cm-lsp-info",
              message: d.message ?? "",
              source: d.source ?? "",
              severity: d.severity,
            }),
          );
        }
        deco = builder.finish();
      }
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

</storycode:copy>
<a id="f-0f1f50575dc9-diagnosticRanges"></a>
### `diagnosticRanges`

Convert LSP diagnostics into editor offsets using a position mapper.

<storycode:copy id="f-0f1f50575dc9-diagnosticRanges">
/** Convert LSP diagnostics into editor offsets using a position mapper. */
export function diagnosticRanges(
  diagnostics: LspDiagnostic[],
  offsetAt: (pos: LspPosition) => number | null,
): DiagnosticSpan[] {
  const out: DiagnosticSpan[] = [];
  for (const d of diagnostics) {
    const from = offsetAt(d.range.start);
    const to = offsetAt(d.range.end);
    if (from === null || to === null) continue;
    out.push({
      from,
      // A zero-width diagnostic (an insertion point) still has to be
      // hoverable, so it is widened to one character.
      to: Math.max(to, from + 1),
      severity: d.severity ?? 1,
      message: d.message,
      source: d.source,
    });
  }
  return out;
}

</storycode:copy>
<a id="f-0f1f50575dc9-diagnosticsAt"></a>
### `diagnosticsAt`

The diagnostics under `pos`, outermost first, for the hover.

<storycode:copy id="f-0f1f50575dc9-diagnosticsAt">
/** The diagnostics under `pos`, outermost first, for the hover. */
export function diagnosticsAt(
  state: { field: (f: typeof lspDiagnosticField) => DecorationSet },
  pos: number,
): { message: string; source: string; severity: number }[] {
  const found: { message: string; source: string; severity: number }[] = [];
  state.field(lspDiagnosticField).between(pos, pos, (_from, _to, value) => {
    const spec = value.spec as { message?: string; source?: string; severity?: number };
    if (spec.message) {
      found.push({
        message: spec.message,
        source: spec.source ?? "",
        severity: spec.severity ?? 1,
      });
    }
  });
  return found;
}

</storycode:copy>
<a id="f-0f1f50575dc9-hoverText"></a>
### `hoverText`

<storycode:copy id="f-0f1f50575dc9-hoverText">
// --- hover ------------------------------------------------------------------

function hoverText(contents: unknown): string {
  if (contents == null) return "";
  if (typeof contents === "string") return contents;
  if (Array.isArray(contents)) return contents.map(hoverText).filter(Boolean).join("\n\n");
  const o = contents as { value?: unknown; language?: unknown };
  if (typeof o.value === "string") return o.value;
  return "";
}

</storycode:copy>
<a id="f-0f1f50575dc9-identifierAt"></a>
### `identifierAt`

--- the extension bundle ---------------------------------------------------
The identifier under an offset, for asking a debugger about it.

<storycode:copy id="f-0f1f50575dc9-identifierAt">
// --- the extension bundle ---------------------------------------------------

/** The identifier under an offset, for asking a debugger about it. */
export function identifierAt(text: string, offset: number): string | null {
  const isWord = (ch: string) => /[A-Za-z0-9_$.]/.test(ch);
  if (offset >= text.length || !isWord(text[offset] ?? "")) return null;
  let start = offset;
  while (start > 0 && isWord(text[start - 1])) start--;
  let end = offset;
  while (end < text.length && isWord(text[end])) end++;
  const word = text.slice(start, end);
  // A bare number is not an identifier, and asking a debugger to evaluate
  // `42` wastes a round trip to be told it is 42.
  return /^[A-Za-z_$]/.test(word) ? word : null;
}

</storycode:copy>
<a id="f-0f1f50575dc9-lspSupport"></a>
### `lspSupport`

<storycode:copy id="f-0f1f50575dc9-lspSupport">
export function lspSupport(opts: CmLspOptions): Extension[] {
  const { client, uri, positionAt, targetAt, onNavigate, onReferences, runtimeValue } = opts;
  const target = (offset: number, view: EditorView) => targetAt ? targetAt(offset, view) : (() => { const position = positionAt(offset, view); return position ? {uri,position} : null; })();
  if (!client) return [lspDiagnosticField];

  const requestAt = async (
    offset: number,
    view: EditorView,
    what: "definition" | "references",
    signal?: AbortSignal,
  ): Promise<UiOutcome> => {
    const original = view.state.doc;
    const location = target(offset, view);
    if (!location) return { status: "unavailable", reason: "Position has no mapped language source" };
    const {uri,position} = location;
    const from: LspNavigationTarget = { uri, range: { start: position, end: position } };
    try {
      const locations = await (what === "definition" ? client.definition(uri, position) : client.references(uri, position));
      if (signal?.aborted || view.state.doc !== original || !view.dom.isConnected) return { status: "unavailable", reason: "Navigation source changed, editor closed or request cancelled" };
      if (what === "definition" && locations.length) onNavigate(locations[0]);
      if (what === "references") onReferences?.(locations, from);
      return { status: "completed", value: { kind: what, total: locations.length, navigation_requested: what === "definition" && locations.length > 0, references_requested: what === "references" && !!onReferences, scope: "Actual language-provider response and shared navigation callback; navigation is not an execution witness" } };
    } catch (e) { return { status: "unavailable", reason: e instanceof Error ? e.message : String(e) }; }
  };
  let hoverSnapshot: { source: Text; id: string; bytes: Uint8Array } | undefined;
  const hoverAt = async (view: EditorView, pos: number) => {
    const original = view.state.doc;
    const problems = diagnosticsAt(view.state, pos);
    const location = target(pos, view);
    const identifier = identifierAt(original.toString(), pos);
    const [result, value] = await Promise.all([
      location ? client.hover(location.uri, location.position).catch(() => null) : Promise.resolve(null),
      identifier && runtimeValue ? runtimeValue(identifier).catch(() => null) : Promise.resolve(null),
    ]);
    if (view.state.doc !== original || !view.dom.isConnected) return null;
    const found = { problems, identifier, value, text: hoverText(result?.contents) };
    hoverSnapshot = { source: original, id: crypto.randomUUID(), bytes: new TextEncoder().encode(JSON.stringify(found)) };
    return { ...found, snapshot: hoverSnapshot };
  };

  return [
    lspDiagnosticField,
    ...(["definition", "references"] as const).map(what => editorLanguageActions.of({
      id: `${uri}:${what}`, label: `${what === "definition" ? "Go to definition" : "Find references"} at editor cursor (${uri.slice(0, 200)})`, parameters: {},
      enabled: view => !!target(view.state.selection.main.head, view),
      execute: (view, _args, signal) => requestAt(view.state.selection.main.head, view, what, signal),
    })),
    editorLanguageActions.of({
      id: `${uri}:hover`, label: `Inspect editor hover (${uri.slice(0, 200)})`,
      parameters: { position: { type: "number", minimum: 0, maximum: 100000000, integer: true } },
      enabled: () => true,
      execute: async (view, args, signal) => {
        const pos = args.position as number;
        if (pos > view.state.doc.length) return { status: "unavailable", reason: "Hover position outside current editor source" };
        const found = await hoverAt(view, pos);
        if (!found || signal.aborted || found.snapshot.source !== view.state.doc) return { status: "unavailable", reason: "Hover source changed, editor closed or request cancelled" };
        return { status: "completed", value: { snapshot: found.snapshot.id, total_bytes: found.snapshot.bytes.length, scope: "Read captured diagnostic, language hover and debugger identifier value through Read editor hover snapshot bytes" } };
      },
    }),
    editorLanguageActions.of({
      id: `${uri}:hover-read`, label: `Read editor hover snapshot bytes (${uri.slice(0, 200)})`,
      parameters: { snapshot: { type: "string", maxLength: 128 }, offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } },
      revision: () => hoverSnapshot?.id,
      enabled: view => !!hoverSnapshot && hoverSnapshot.source === view.state.doc,
      execute: async (view, args) => {
        const snapshot = hoverSnapshot;
        if (!snapshot || snapshot.id !== args.snapshot || snapshot.source !== view.state.doc) return { status: "unavailable", reason: "Hover snapshot changed; inspect the current hover again" };
        const { bytes } = snapshot;
        const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
        if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside captured hover response" };
        return { status: "completed", value: { snapshot: snapshot.id, bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null, scope: "Immutable human-handler hover capture; runtime value is captured evidence, not a current execution verdict" } };
      },
    }),
    hoverTooltip(async (view, pos): Promise<Tooltip | null> => {
      const found = await hoverAt(view, pos);
      if (!found) return null;
      const { problems, identifier, value, text } = found;
      if (problems.length === 0 && !text.trim() && !value) return null;

      return {
        pos,
        above: true,
        create: () => {
          const dom = document.createElement("div");
          dom.className = "cm-lsp-hover";
          for (const problem of problems) {
            const line = document.createElement("div");
            line.className =
              problem.severity === 1
                ? "cm-lsp-hover-error"
                : problem.severity === 2
                  ? "cm-lsp-hover-warn"
                  : "cm-lsp-hover-note";
            line.textContent = problem.source
              ? `${problem.message}  (${problem.source})`
              : problem.message;
            dom.appendChild(line);
          }
          if (value && identifier) {
            // The runtime value, above the type: while stopped, "what does
            // this hold" is the question being asked.
            const runtime = document.createElement("div");
            runtime.className = "cm-lsp-hover-value";
            runtime.textContent = `${identifier} = ${value}`;
            dom.appendChild(runtime);
          }
          if (text.trim()) {
            const info = document.createElement("div");
            // Separated from the problem above it, so two different kinds of
            // statement do not read as one paragraph.
            info.className = problems.length > 0 || value ? "cm-lsp-hover-info" : "";
            // Rendered, not dumped: a server answers in Markdown, and setting
            // it as text showed the backticks, the word `rust`, and a row of
            // hyphens. Never `innerHTML` — the string is another program's.
            info.appendChild(renderHoverMarkdown(text));
            dom.appendChild(info);
          }
          return { dom };
        },
      };
    }),
    keymap.of([
      ...([ ["editor.definition", "definition"], ["editor.references", "references"] ] as const).flatMap(([action,what]) => {
        const key = cmKeyOf(action);
        return key ? [{key,run:(view:EditorView)=>{void requestAt(view.state.selection.main.head,view,what);return true;}}] : [];
      }),
    ]),
    EditorView.domEventHandlers({
      mousedown(event, view) {
        if (!(event.metaKey || event.ctrlKey) || event.button !== 0) return false;
        const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
        if (pos === null) return false;
        event.preventDefault();
        void requestAt(pos, view, event.shiftKey ? "references" : "definition");
        return true;
      },
    }),
    // Ctrl/Cmd held: show the symbol under the pointer as a link, so the
    // affordance is discoverable instead of being folklore.
    EditorView.theme({
      "&.cm-lsp-linking .cm-content": { cursor: "pointer" },
    }),
  ];
}

</storycode:copy>
<a id="f-0f1f50575dc9-offsetToPosition"></a>
### `offsetToPosition`

--- coordinate helpers -----------------------------------------------------
CodeMirror offset → LSP position. Both count UTF-16 code units within a
line, so this is a line lookup and a subtraction — no re-encoding.

<storycode:copy id="f-0f1f50575dc9-offsetToPosition">
// --- coordinate helpers -----------------------------------------------------

/**
 * CodeMirror offset → LSP position. Both count UTF-16 code units within a
 * line, so this is a line lookup and a subtraction — no re-encoding.
 */
export function offsetToPosition(doc: Text, offset: number): LspPosition {
  const clamped = Math.max(0, Math.min(offset, doc.length));
  const line = doc.lineAt(clamped);
  return { line: line.number - 1, character: clamped - line.from };
}

</storycode:copy>
<a id="f-0f1f50575dc9-positionToOffset"></a>
### `positionToOffset`

LSP position → CodeMirror offset, clamped into the document.

<storycode:copy id="f-0f1f50575dc9-positionToOffset">
/** LSP position → CodeMirror offset, clamped into the document. */
export function positionToOffset(doc: Text, pos: LspPosition): number {
  const lineNo = Math.max(1, Math.min(pos.line + 1, doc.lines));
  const line = doc.line(lineNo);
  return Math.min(line.from + Math.max(0, pos.character), line.to);
}
</storycode:copy>

### Assemble `apps/web/src/lsp/cmLsp.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/cmLsp.ts"><storycode:paste select="#f-0f1f50575dc9-LspNavigationTarget"/><storycode:paste select="#f-0f1f50575dc9-CmLspOptions"/><storycode:paste select="#f-0f1f50575dc9-DiagnosticSpan"/><storycode:paste select="#f-0f1f50575dc9-setLspDiagnostics"/><storycode:paste select="#f-0f1f50575dc9-lspDiagnosticField"/><storycode:paste select="#f-0f1f50575dc9-diagnosticRanges"/><storycode:paste select="#f-0f1f50575dc9-diagnosticsAt"/><storycode:paste select="#f-0f1f50575dc9-hoverText"/><storycode:paste select="#f-0f1f50575dc9-identifierAt"/><storycode:paste select="#f-0f1f50575dc9-lspSupport"/><storycode:paste select="#f-0f1f50575dc9-offsetToPosition"/><storycode:paste select="#f-0f1f50575dc9-positionToOffset"/></storycode:file>

<a id="output-b887a5a62495"></a>
## `apps/web/src/lsp/hoverMarkdown.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Hover Is Rendered, Not Dumped](contract.md).

<a id="f-b887a5a62495-render"></a>
### `render`

Protects docs/guarantees/editor-intelligence/a-hover-is-rendered-not-dumped.md

<storycode:copy id="f-b887a5a62495-render">
// Protects docs/guarantees/editor-intelligence/a-hover-is-rendered-not-dumped.md

import { describe, expect, it } from "vitest";
import { renderHoverMarkdown } from "./hoverMarkdown";

function render(text: string): HTMLElement {
  const host = document.createElement("div");
  host.appendChild(renderHoverMarkdown(text));
  return host;
}

</storycode:copy>
<a id="f-b887a5a62495-RUST_ANALYZER"></a>
### `RUST_ANALYZER`

What rust-analyzer actually answers for a `Vec` binding.

<storycode:copy id="f-b887a5a62495-RUST_ANALYZER">
/** What rust-analyzer actually answers for a `Vec` binding. */
const RUST_ANALYZER = [
  "```rust",
  "let names: Vec<String>",
  "```",
  "",
  "---",
  "",
  "A contiguous growable array type, written as `Vec<T>`, short for *vector*.",
  "",
  "See [Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html) for more.",
].join("\n");

</storycode:copy>
<a id="f-b887a5a62495-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b887a5a62495-expression_statement">
describe("a hover's markdown", () => {
  it("renders a fence as a code block, without its backticks", () => {
    const host = render(RUST_ANALYZER);
    const code = host.querySelector("pre.cm-lsp-hover-code code");
    expect(code?.textContent).toBe("let names: Vec<String>");
    // The three things a person used to see and should not.
    expect(host.textContent).not.toContain("```");
    expect(host.textContent).not.toContain("---");
    expect(host.querySelector("pre")?.getAttribute("data-language")).toBe("rust");
  });

  it("draws a rule as a rule", () => {
    expect(render(RUST_ANALYZER).querySelectorAll("hr")).toHaveLength(1);
  });

  it("shows a link as its text, since a tooltip is not clickable", () => {
    const host = render(RUST_ANALYZER);
    expect(host.textContent).toContain("See Vec for more.");
    expect(host.textContent).not.toContain("https://doc.rust-lang.org");
    expect(host.querySelector("a")).toBeNull();
  });

  it("renders inline code and emphasis", () => {
    const host = render("A `Vec<T>`, short for *vector*, and **not** a slice.");
    expect(host.querySelector("code")?.textContent).toBe("Vec<T>");
    expect(host.querySelector("em")?.textContent).toBe("vector");
    expect(host.querySelector("strong")?.textContent).toBe("not");
  });

  it("does not read bold as two emphases", () => {
    const host = render("**bold**");
    expect(host.querySelector("strong")?.textContent).toBe("bold");
    expect(host.querySelector("em")).toBeNull();
  });

  it("never interprets the server's text as HTML", () => {
    // The string comes from another program. `innerHTML` on it would be an
    // injection with extra steps.
    const host = render("<img src=x onerror=alert(1)> and `<b>literal</b>`");
    expect(host.querySelector("img")).toBeNull();
    expect(host.querySelector("b")).toBeNull();
    expect(host.textContent).toContain("<img src=x onerror=alert(1)>");
    expect(host.querySelector("code")?.textContent).toBe("<b>literal</b>");
  });

  it("keeps a paragraph's own line breaks", () => {
    const host = render("first line\nsecond line");
    expect(host.querySelectorAll("p")).toHaveLength(1);
    expect(host.textContent).toBe("first line\nsecond line");
  });

  it("splits paragraphs on a blank line", () => {
    expect(render("one\n\ntwo").querySelectorAll("p")).toHaveLength(2);
  });

  it("renders a fence nobody closed rather than swallowing it", () => {
    const host = render("```rust\nfn main() {}");
    expect(host.querySelector("code")?.textContent).toBe("fn main() {}");
  });

  it("produces nothing for nothing", () => {
    expect(render("").childNodes).toHaveLength(0);
    expect(render("\n\n  \n").childNodes).toHaveLength(0);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lsp/hoverMarkdown.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/hoverMarkdown.test.ts"><storycode:paste select="#f-b887a5a62495-render"/><storycode:paste select="#f-b887a5a62495-RUST_ANALYZER"/><storycode:paste select="#f-b887a5a62495-expression_statement"/></storycode:file>

<a id="output-60b2f1a87d40"></a>
## `apps/web/src/lsp/hoverMarkdown.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Hover Is Rendered, Not Dumped](contract.md).

<a id="f-60b2f1a87d40-LINK"></a>
### `LINK`

The little bit of Markdown a hover actually contains.
A language server answers `textDocument/hover` with `MarkupContent`, and
every real one sends Markdown: rust-analyzer wraps the signature in a
\```rust fence, rules off the sections with `---`, and links types to
docs.rs. The tooltip used to set that string as `textContent`, so what a
person saw was the backticks, the word `rust`, and a row of hyphens —
noted while dogfooding on 2026-09-02.
This is deliberately not a Markdown library. A hover is a paragraph, a
fenced block, a rule, and some inline emphasis, and the failure mode of a
full parser here is a tooltip that renders a table. What is NOT optional is
that everything is built as DOM and set as text: the string comes from
another program, and `innerHTML` on it would be an injection with extra
steps.
A link is shown as its text: a tooltip is not somewhere you can click.

<storycode:copy id="f-60b2f1a87d40-LINK">
// The little bit of Markdown a hover actually contains.
//
// A language server answers `textDocument/hover` with `MarkupContent`, and
// every real one sends Markdown: rust-analyzer wraps the signature in a
// ```rust fence, rules off the sections with `---`, and links types to
// docs.rs. The tooltip used to set that string as `textContent`, so what a
// person saw was the backticks, the word `rust`, and a row of hyphens —
// noted while dogfooding on 2026-09-02.
//
// This is deliberately not a Markdown library. A hover is a paragraph, a
// fenced block, a rule, and some inline emphasis, and the failure mode of a
// full parser here is a tooltip that renders a table. What is NOT optional is
// that everything is built as DOM and set as text: the string comes from
// another program, and `innerHTML` on it would be an injection with extra
// steps.

/** A link is shown as its text: a tooltip is not somewhere you can click. */
const LINK = /\[([^\]]+)\]\([^)]*\)/g;

</storycode:copy>
<a id="f-60b2f1a87d40-inline"></a>
### `inline`

Inline spans, in the order they are taken off.

<storycode:copy id="f-60b2f1a87d40-inline">
/** Inline spans, in the order they are taken off. */
function inline(target: HTMLElement, text: string): void {
  const withoutLinks = text.replace(LINK, "$1");
  // One pass, longest markers first, so `**bold**` is not read as two `*em*`.
  const pattern = /(`[^`]+`)|(\*\*[^*]+\*\*)|(__[^_]+__)|(\*[^*]+\*)|(_[^_]+_)/g;
  let last = 0;
  for (const match of withoutLinks.matchAll(pattern)) {
    const at = match.index ?? 0;
    if (at > last) target.appendChild(document.createTextNode(withoutLinks.slice(last, at)));
    const token = match[0];
    if (token.startsWith("`")) {
      const code = document.createElement("code");
      code.textContent = token.slice(1, -1);
      target.appendChild(code);
    } else if (token.startsWith("**") || token.startsWith("__")) {
      const strong = document.createElement("strong");
      strong.textContent = token.slice(2, -2);
      target.appendChild(strong);
    } else {
      const em = document.createElement("em");
      em.textContent = token.slice(1, -1);
      target.appendChild(em);
    }
    last = at + token.length;
  }
  if (last < withoutLinks.length) {
    target.appendChild(document.createTextNode(withoutLinks.slice(last)));
  }
}

</storycode:copy>
<a id="f-60b2f1a87d40-isRule"></a>
### `isRule`

<storycode:copy id="f-60b2f1a87d40-isRule">
/** Whether a line is a horizontal rule. */
function isRule(line: string): boolean {
  const trimmed = line.trim();
  return /^(-{3,}|\*{3,}|_{3,})$/.test(trimmed);
}

</storycode:copy>
<a id="f-60b2f1a87d40-renderHoverMarkdown"></a>
### `renderHoverMarkdown`

Render a hover's Markdown into a fragment.
Everything is `textContent`; nothing is ever parsed as HTML.

<storycode:copy id="f-60b2f1a87d40-renderHoverMarkdown">
/**
 * Render a hover's Markdown into a fragment.
 *
 * Everything is `textContent`; nothing is ever parsed as HTML.
 */
export function renderHoverMarkdown(text: string): DocumentFragment {
  const fragment = document.createDocumentFragment();
  const lines = text.replace(/\r\n/g, "\n").split("\n");

  let i = 0;
  let paragraph: string[] = [];
  const flush = () => {
    // Trailing blank lines are not a paragraph.
    while (paragraph.length > 0 && paragraph[paragraph.length - 1].trim() === "") paragraph.pop();
    if (paragraph.length === 0) return;
    const p = document.createElement("p");
    p.className = "cm-lsp-hover-text";
    inline(p, paragraph.join("\n"));
    fragment.appendChild(p);
    paragraph = [];
  };

  while (i < lines.length) {
    const line = lines[i];
    const fence = /^\s*```+\s*([\w+#-]*)\s*$/.exec(line);
    if (fence) {
      flush();
      const language = fence[1] ?? "";
      const body: string[] = [];
      i += 1;
      while (i < lines.length && !/^\s*```+\s*$/.test(lines[i])) {
        body.push(lines[i]);
        i += 1;
      }
      // A fence that is never closed still renders: the server sent it, and
      // swallowing the signature because a backtick is missing is worse than
      // showing it.
      i += 1;
      const pre = document.createElement("pre");
      pre.className = "cm-lsp-hover-code";
      if (language) pre.dataset.language = language;
      const code = document.createElement("code");
      code.textContent = body.join("\n");
      pre.appendChild(code);
      fragment.appendChild(pre);
      continue;
    }
    if (isRule(line)) {
      flush();
      fragment.appendChild(document.createElement("hr"));
      i += 1;
      continue;
    }
    if (line.trim() === "") {
      flush();
      i += 1;
      continue;
    }
    paragraph.push(line);
    i += 1;
  }
  flush();
  return fragment;
}
</storycode:copy>

### Assemble `apps/web/src/lsp/hoverMarkdown.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/hoverMarkdown.ts"><storycode:paste select="#f-60b2f1a87d40-LINK"/><storycode:paste select="#f-60b2f1a87d40-inline"/><storycode:paste select="#f-60b2f1a87d40-isRule"/><storycode:paste select="#f-60b2f1a87d40-renderHoverMarkdown"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lsp/cmLsp.ts](contract.md#output-0f1f50575dc9)
- [apps/web/src/lsp/hoverMarkdown.ts](contract.md#output-60b2f1a87d40)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)

### Tests and fixtures

- [apps/web/src/lsp/hoverMarkdown.test.ts](contract.md#output-b887a5a62495)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
