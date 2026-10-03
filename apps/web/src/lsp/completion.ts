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

import type { LspClient } from "./client";
import type { LspPosition } from "./positions";

/** Where a suggestion came from. */
export type CompletionOrigin = "lsp" | "project";

/** One suggestion from the project's own text. */
export interface ProjectSuggestion {
  text: string;
  detail: string;
  score: number;
  /** Whether the embedding model contributed. False is frequency alone —
   * still useful, still offline, and said rather than implied. */
  semantic: boolean;
}

/** The mark shown beside a suggestion. Text, not an icon font: this app ships
 * no icon set, and these read at any size and in any theme. */
export function originMark(origin: CompletionOrigin): string {
  return origin === "lsp" ? "⌁" : "◇";
}

/** What the mark means, spelled out for the hover and for screen readers. */
export function originLabel(origin: CompletionOrigin, semantic = false): string {
  if (origin === "lsp") return "from the language server — in scope here";
  return semantic
    ? "from this project's own text, ranked by what you are writing"
    : "from this project's own text, ranked by how often it is used";
}

/** LSP's completion-kind numbers, as names, for the popup's `type` slot. */
const LSP_KINDS: Record<number, string> = {
  1: "text", 2: "method", 3: "function", 4: "constructor", 5: "property",
  6: "variable", 7: "class", 8: "interface", 9: "module", 10: "property",
  11: "unit", 12: "value", 13: "enum", 14: "keyword", 15: "snippet",
  16: "text", 17: "text", 18: "text", 19: "text", 20: "enum", 21: "constant",
  22: "class", 23: "interface", 24: "operator", 25: "type",
};

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

/** Turn one project suggestion into a CodeMirror completion. */
export function projectCompletion(suggestion: ProjectSuggestion): Completion {
  return {
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
  };
}

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
  return autocompletion({
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
  });
}
