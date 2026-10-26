<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/completion.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4123a128ce4a"></a>
## `apps/web/src/lsp/completion.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4123a128ce4a-completionSources"></a>
### `completionSources`

<storycode:copy id="f-4123a128ce4a-completionSources">
import { completionActions, completionReading } from "./completionActions";
// Two kinds of suggestion, in one list, saying which is which.
//
// The two sources answer genuinely different questions, and that is the whole
// reason for showing them together and the whole reason they must be told
// apart:
//
//  - The **language server** knows what is in scope here and what its type
//    is. It is authoritative and it is narrow: it has no opinion about
//    whether this codebase calls the thing `cfg`, `config` or `settings`.
//  - The **local model** knows exactly that. It reads the project's own text
//    and ranks by how often a name is used and — when the model is installed
//    — by how close its surroundings are to what is being typed. It knows
//    nothing about types, and it can suggest something that does not exist
//    here at all.
//
// Neither subsumes the other, so neither is hidden. But a reader has to be
// able to see, without thinking about it, whether a suggestion is a fact or a
// guess — a list that mixed them silently would make the guesses look like
// facts, which is the failure worth designing against.
//
// So every entry carries its origin, the popup renders a mark for it, and the
// language server's answers sort first: when something IS authoritative,
// burying it under a frequency count would be a poor trade.

import { autocompletion } from "@codemirror/autocomplete";
import type { Completion, CompletionContext, CompletionResult, CompletionSource } from "@codemirror/autocomplete";
import { Facet, type Extension } from "@codemirror/state";

/** File projections add sources to the editor’s single completion configuration. */
export const completionSources = Facet.define<CompletionSource>();

</storycode:copy>
<a id="f-4123a128ce4a-import_statement"></a>
### `import_statement`

<storycode:copy id="f-4123a128ce4a-import_statement">
import type { LspClient } from "./client";
</storycode:copy>
<a id="f-4123a128ce4a-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-4123a128ce4a-import_statement-2">
import type { LspPosition } from "./positions";

</storycode:copy>
<a id="f-4123a128ce4a-CompletionOrigin"></a>
### `CompletionOrigin`

<storycode:copy id="f-4123a128ce4a-CompletionOrigin">
/** Where a suggestion came from. */
export type CompletionOrigin = "lsp" | "project";

</storycode:copy>
<a id="f-4123a128ce4a-ProjectSuggestion"></a>
### `ProjectSuggestion`

One suggestion from the project's own text.

<storycode:copy id="f-4123a128ce4a-ProjectSuggestion">
/** One suggestion from the project's own text. */
export interface ProjectSuggestion {
  text: string;
  detail: string;
  score: number;
  /** Whether the embedding model contributed. False is frequency alone —
   * still useful, still offline, and said rather than implied. */
  semantic: boolean;
}

</storycode:copy>
<a id="f-4123a128ce4a-originMark"></a>
### `originMark`

The mark shown beside a suggestion. Text, not an icon font: this app ships
no icon set, and these read at any size and in any theme.

<storycode:copy id="f-4123a128ce4a-originMark">
/** The mark shown beside a suggestion. Text, not an icon font: this app ships
 * no icon set, and these read at any size and in any theme. */
export function originMark(origin: CompletionOrigin): string {
  return origin === "lsp" ? "⌁" : "◇";
}

</storycode:copy>
<a id="f-4123a128ce4a-originLabel"></a>
### `originLabel`

What the mark means, spelled out for the hover and for screen readers.

<storycode:copy id="f-4123a128ce4a-originLabel">
/** What the mark means, spelled out for the hover and for screen readers. */
export function originLabel(origin: CompletionOrigin, semantic = false): string {
  if (origin === "lsp") return "from the language server — in scope here";
  return semantic
    ? "from this project's own text, ranked by what you are writing"
    : "from this project's own text, ranked by how often it is used";
}

</storycode:copy>
<a id="f-4123a128ce4a-LSP_KINDS"></a>
### `LSP_KINDS`

LSP's completion-kind numbers, as names, for the popup's `type` slot.

<storycode:copy id="f-4123a128ce4a-LSP_KINDS">
/** LSP's completion-kind numbers, as names, for the popup's `type` slot. */
const LSP_KINDS: Record<number, string> = {
  1: "text", 2: "method", 3: "function", 4: "constructor", 5: "property",
  6: "variable", 7: "class", 8: "interface", 9: "module", 10: "property",
  11: "unit", 12: "value", 13: "enum", 14: "keyword", 15: "snippet",
  16: "text", 17: "text", 18: "text", 19: "text", 20: "enum", 21: "constant",
  22: "class", 23: "interface", 24: "operator", 25: "type",
};

</storycode:copy>
<a id="f-4123a128ce4a-wordBefore"></a>
### `wordBefore`

The word being typed, if any. `null` when the caret is not in one.

<storycode:copy id="f-4123a128ce4a-wordBefore">
/** The word being typed, if any. `null` when the caret is not in one. */
export function wordBefore(
  context: Pick<CompletionContext, "matchBefore" | "explicit">,
): { from: number; text: string } | null {
  const word = context.matchBefore(/[A-Za-z_][A-Za-z0-9_]*/);
  if (!word) return null;
  // An explicit request (Ctrl-Space) completes from nothing; an implicit one
  // waits for a couple of characters, or every keystroke opens a list of the
  // whole project.
  if (word.from === word.to && !context.explicit) return null;
  return { from: word.from, text: word.text };
}

</storycode:copy>
<a id="f-4123a128ce4a-projectCompletion"></a>
### `projectCompletion`

Turn one project suggestion into a CodeMirror completion.

<storycode:copy id="f-4123a128ce4a-projectCompletion">
/** Turn one project suggestion into a CodeMirror completion. */
export function projectCompletion(suggestion: ProjectSuggestion): Completion {
  return completionReading({
    label: suggestion.text,
    detail: suggestion.detail,
    type: "text",
    // Below every LSP answer. When something IS authoritative, burying it
    // under a frequency count would be a poor trade.
    boost: -50,
    section: "This project",
    info: () => {
      const el = document.createElement("div");
      el.className = "cm-completion-info";
      el.textContent = originLabel("project", suggestion.semantic);
      return el;
    },
  }, originLabel("project", suggestion.semantic));
}

</storycode:copy>
<a id="f-4123a128ce4a-lspCompletion"></a>
### `lspCompletion`

Turn one LSP item into a CodeMirror completion.

<storycode:copy id="f-4123a128ce4a-lspCompletion">
/** Turn one LSP item into a CodeMirror completion. */
export function lspCompletion(item: {
  label: string;
  kind?: number;
  detail?: string;
  insertText?: string;
}): Completion {
  return {
    label: item.label,
    apply: item.insertText ?? item.label,
    detail: item.detail,
    type: item.kind ? (LSP_KINDS[item.kind] ?? "text") : "text",
    boost: 0,
    section: "In scope",
  };
}

</storycode:copy>
<a id="f-4123a128ce4a-completionSource"></a>
### `completionSource`

The combined source.
Both are asked at once and neither waits for the other: a language server
mid-index must not stop the project's own answers appearing, and a slow
project index must not delay the authoritative ones. Whichever fails
contributes nothing, silently — a completion popup is not the place to
report that a subsystem is warming up.

<storycode:copy id="f-4123a128ce4a-completionSource">
/**
 * The combined source.
 *
 * Both are asked at once and neither waits for the other: a language server
 * mid-index must not stop the project's own answers appearing, and a slow
 * project index must not delay the authoritative ones. Whichever fails
 * contributes nothing, silently — a completion popup is not the place to
 * report that a subsystem is warming up.
 */
export function completionSource(options: {
  lsp?: {
    client: LspClient;
    uri: string;
    positionAt: (offset: number, view: CompletionContext["state"]) => LspPosition | null;
  };
  project?: (prefix: string, context: string) => Promise<ProjectSuggestion[]>;
}): (context: CompletionContext) => Promise<CompletionResult | null> {
  return async (context: CompletionContext) => {
    const word = wordBefore(context);
    if (!word) return null;

    const around = context.state.doc.sliceString(
      Math.max(0, word.from - 400),
      Math.min(context.state.doc.length, word.from + 200),
    );

    const position = options.lsp?.positionAt(context.pos, context.state);
    const [lspItems, projectItems] = await Promise.all([
      options.lsp && position
        ? options.lsp.client
            .completion(options.lsp.uri, position)
            .catch(() => [])
        : Promise.resolve([]),
      options.project
        ? options.project(word.text, around).catch(() => [])
        : Promise.resolve([]),
    ]);

    const seen = new Set<string>();
    const completions: Completion[] = [];
    for (const item of lspItems) {
      if (seen.has(item.label)) continue;
      seen.add(item.label);
      completions.push(lspCompletion(item));
    }
    for (const suggestion of projectItems) {
      // A name the language server already offered is the same name. Showing
      // it twice would say the two sources disagree, when they agree.
      if (seen.has(suggestion.text)) continue;
      seen.add(suggestion.text);
      completions.push(projectCompletion(suggestion));
    }
    if (completions.length === 0) return null;
    return { from: word.from, options: completions, validFor: /^[A-Za-z0-9_]*$/ };
  };
}

</storycode:copy>
<a id="f-4123a128ce4a-completions"></a>
### `completions`

The extension
Autocompletion, from both sources, marked.
`addToOptions` is where the marks are drawn: CodeMirror's own renderer has
a slot for a type icon and none for "who said this", and the difference
between an authoritative answer and a frequency guess is exactly the thing
a reader must be able to see without thinking about it.

<storycode:copy id="f-4123a128ce4a-completions">
// ---------------------------------------------------------------------------
// The extension
// ---------------------------------------------------------------------------

/**
 * Autocompletion, from both sources, marked.
 *
 * `addToOptions` is where the marks are drawn: CodeMirror's own renderer has
 * a slot for a type icon and none for "who said this", and the difference
 * between an authoritative answer and a frequency guess is exactly the thing
 * a reader must be able to see without thinking about it.
 */
export function completions(options: {
  lsp?: {
    client: LspClient;
    uri: string;
    positionAt: (offset: number, state: CompletionContext["state"]) => LspPosition | null;
  };
  project?: (prefix: string, context: string) => Promise<ProjectSuggestion[]>;
}): Extension {
  return [completionActions(), autocompletion({
    override: [async context => {
      const results = (await Promise.all([completionSource(options)(context), ...context.state.facet(completionSources).map(source => source(context))])).filter((r): r is CompletionResult => r !== null);
      if (!results.length) return null;
      const first=results[0];
      const ranked=results.filter(r=>r.from===first.from).flatMap(r=>r.options).sort((a,b)=>(b.boost??0)-(a.boost??0));
      const seen=new Set<string>();
      return {...first, options:ranked.filter(item=> { if(seen.has(item.label)) return false; seen.add(item.label); return true; })};
    }],
    // Not while nothing has been typed: a popup that opens on its own in the
    // middle of prose is a popup people turn off.
    activateOnTyping: true,
    closeOnBlur: true,
    icons: true,
    addToOptions: [
      {
        render: (completion) => {
          const origin: CompletionOrigin =
            completion.section === "This project" ? "project" : "lsp";
          const mark = document.createElement("span");
          mark.className = `cm-completion-origin cm-completion-origin--${origin}`;
          mark.textContent = originMark(origin);
          // `data-tip`, never `title`: every hover hint in this app is drawn
          // by the themed layer.
          mark.dataset.tip = originLabel(origin);
          mark.setAttribute("aria-label", originLabel(origin));
          return mark;
        },
        // Before the label, where an icon would be — the origin is the first
        // thing to read, not a footnote.
        position: 15,
      },
    ],
  })];
}
</storycode:copy>

### Assemble `apps/web/src/lsp/completion.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/completion.ts"><storycode:paste select="#f-4123a128ce4a-completionSources"/><storycode:paste select="#f-4123a128ce4a-import_statement"/><storycode:paste select="#f-4123a128ce4a-import_statement-2"/><storycode:paste select="#f-4123a128ce4a-CompletionOrigin"/><storycode:paste select="#f-4123a128ce4a-ProjectSuggestion"/><storycode:paste select="#f-4123a128ce4a-originMark"/><storycode:paste select="#f-4123a128ce4a-originLabel"/><storycode:paste select="#f-4123a128ce4a-LSP_KINDS"/><storycode:paste select="#f-4123a128ce4a-wordBefore"/><storycode:paste select="#f-4123a128ce4a-projectCompletion"/><storycode:paste select="#f-4123a128ce4a-lspCompletion"/><storycode:paste select="#f-4123a128ce4a-completionSource"/><storycode:paste select="#f-4123a128ce4a-completions"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-f2da457dd964"></a>
## `apps/web/src/lsp/completion.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f2da457dd964-contextAt"></a>
### `contextAt`

<storycode:copy id="f-f2da457dd964-contextAt">
import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { CompletionContext } from "@codemirror/autocomplete";

import {
  completionSource,
  lspCompletion,
  originLabel,
  originMark,
  projectCompletion,
  wordBefore,
} from "./completion";

const contextAt = (doc: string, pos: number, explicit = false) =>
  new CompletionContext(EditorState.create({ doc }), pos, explicit);

</storycode:copy>
<a id="f-f2da457dd964-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f2da457dd964-expression_statement">
describe("telling the two sources apart", () => {
  it("gives each origin its own mark", () => {
    // A list that mixed them silently would make the guesses look like facts.
    expect(originMark("lsp")).not.toBe(originMark("project"));
  });

  it("says what each one actually knows", () => {
    expect(originLabel("lsp")).toMatch(/in scope/i);
    expect(originLabel("project", false)).toMatch(/how often/i);
    expect(originLabel("project", true)).toMatch(/what you are writing/i);
  });

  it("does not claim semantics that did not happen", () => {
    // With no model installed the ranking is frequency, and the label says
    // frequency.
    expect(originLabel("project", false)).not.toMatch(/what you are writing/i);
  });

  it("sorts the authoritative answers first", () => {
    // When something IS in scope, burying it under a frequency count would
    // be a poor trade.
    const lsp = lspCompletion({ label: "x" });
    const project = projectCompletion({ text: "y", detail: "a.py:1", score: 9, semantic: false });
    expect((lsp.boost ?? 0) > (project.boost ?? 0)).toBe(true);
  });

  it("puts them in named sections", () => {
    expect(lspCompletion({ label: "x" }).section).toBe("In scope");
    expect(
      projectCompletion({ text: "y", detail: "", score: 1, semantic: false }).section,
    ).toBe("This project");
  });
});

</storycode:copy>
<a id="f-f2da457dd964-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f2da457dd964-expression_statement-2">
describe("when the popup opens at all", () => {
  it("waits for a word rather than firing on every keystroke", () => {
    expect(wordBefore(contextAt("x = ", 4))).toBeNull();
  });

  it("completes from nothing when it was asked for explicitly", () => {
    // Ctrl-Space means "show me now", which is a different question.
    expect(wordBefore(contextAt("x = ", 4, true))).toBeNull();
    expect(wordBefore(contextAt("x = tot", 7, true))?.text).toBe("tot");
  });

  it("finds the word the caret is inside", () => {
    const word = wordBefore(contextAt("total_uni", 9));
    expect(word).toEqual({ from: 0, text: "total_uni" });
  });
});

</storycode:copy>
<a id="f-f2da457dd964-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-f2da457dd964-expression_statement-3">
describe("the combined list", () => {
  const suggest = (text: string) => ({ text, detail: "a.py:1", score: 1, semantic: false });

  it("shows both sources", async () => {
    const source = completionSource({
      project: async () => [suggest("total_units"), suggest("total_revenue")],
    });
    const result = await source(contextAt("tot", 3));
    expect(result?.options.map((o) => o.label)).toEqual(["total_units", "total_revenue"]);
  });

  it("does not show one name twice when both sources have it", async () => {
    // Showing it twice would say the two disagree, when they agree.
    const source = completionSource({
      lsp: {
        client: { completion: async () => [{ label: "total_units" }] } as never,
        uri: "file:///x",
        positionAt: () => ({ line: 0, character: 0 }),
      },
      project: async () => [suggest("total_units"), suggest("total_revenue")],
    });
    const result = await source(contextAt("tot", 3));
    expect(result?.options.map((o) => o.label)).toEqual(["total_units", "total_revenue"]);
  });

  it("keeps the project's answers when the language server fails", async () => {
    // A server mid-index must not stop the project's own answers appearing.
    const source = completionSource({
      lsp: {
        client: { completion: async () => { throw new Error("not ready"); } } as never,
        uri: "file:///x",
        positionAt: () => ({ line: 0, character: 0 }),
      },
      project: async () => [suggest("total_units")],
    });
    const result = await source(contextAt("tot", 3));
    expect(result?.options.map((o) => o.label)).toEqual(["total_units"]);
  });

  it("keeps the language server's answers when the project index fails", async () => {
    const source = completionSource({
      lsp: {
        client: { completion: async () => [{ label: "typed_thing" }] } as never,
        uri: "file:///x",
        positionAt: () => ({ line: 0, character: 0 }),
      },
      project: async () => {
        throw new Error("indexing");
      },
    });
    const result = await source(contextAt("ty", 2));
    expect(result?.options.map((o) => o.label)).toEqual(["typed_thing"]);
  });

  it("answers nothing rather than an empty popup", async () => {
    const source = completionSource({ project: async () => [] });
    expect(await source(contextAt("tot", 3))).toBeNull();
  });

  it("stays open while the word is still being typed", async () => {
    const source = completionSource({ project: async () => [suggest("total_units")] });
    const result = await source(contextAt("tot", 3));
    expect("total".match(result!.validFor as RegExp)).toBeTruthy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lsp/completion.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/completion.test.ts"><storycode:paste select="#f-f2da457dd964-contextAt"/><storycode:paste select="#f-f2da457dd964-expression_statement"/><storycode:paste select="#f-f2da457dd964-expression_statement-2"/><storycode:paste select="#f-f2da457dd964-expression_statement-3"/></storycode:file>


</storycode:doc>
