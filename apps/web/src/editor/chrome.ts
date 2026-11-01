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
import { Prec, StateEffect, type Extension } from "@codemirror/state";
import { foldedRanges, unfoldEffect } from "@codemirror/language";
import { SearchCursor } from "@codemirror/search";

/** Which of the two looks an editor wears. */
export type ChromeKind = "document" | "code";

/**
 * The shared chrome.
 *
 * Everything here reads a token, never a literal colour, so all three themes
 * follow without a second definition.
 */
export function editorChrome(kind: ChromeKind): Extension {
  const code = kind === "code";
  return EditorView.theme({
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
  });
}

// The same local find interaction serves document text and Files row names.
// Decorations leave the buffer unchanged; revealing a match only opens folds.
export function paneSearch(label: string): Extension {
  const refresh = StateEffect.define<null>();
  const plugin = ViewPlugin.fromClass(class {
    query = "";
    decorations = Decoration.none;
    button = document.createElement("button");
    dialog = document.createElement("dialog");
    input = document.createElement("input");
    status = document.createElement("span");
    constructor(readonly view: EditorView) {
      this.button.type = "button";
      this.button.setAttribute("aria-label", `Search ${label}`);
      this.button.title = `Search ${label}`;
      this.button.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/></svg>';
      this.button.style.cssText = "position:absolute;right:8px;top:4px;z-index:5;background:var(--bg-raised);color:var(--fg);border:1px solid var(--border);border-radius:4px;cursor:pointer";
      this.dialog.setAttribute("aria-label", `Search ${label}`);
      this.dialog.style.cssText = "background:var(--bg-raised);color:var(--fg);border:1px solid var(--border);border-radius:8px;padding:16px";
      this.input.type = "search";
      this.input.placeholder = `Search ${label}`;
      this.input.setAttribute("aria-label", `Search ${label} text`);
      this.input.oninput = () => {
        this.query = this.input.value;
        const effects: ReturnType<typeof unfoldEffect.of>[] = [];
        if (this.query) foldedRanges(view.state).between(0, view.state.doc.length, (from, to) => {
          if (view.state.sliceDoc(from, to).toLowerCase().includes(this.query.toLowerCase()))
            effects.push(unfoldEffect.of({ from, to }));
        });
        view.dispatch({ effects: [...effects, refresh.of(null)] });
      };
      const close = document.createElement("button");
      close.type = "button"; close.textContent = "Close";
      close.onclick = () => this.dialog.close();
      this.dialog.onclose = () => this.button.focus();
      this.dialog.append(this.input, this.status, close);
      this.button.onclick = () => this.open();
      view.dom.append(this.button, this.dialog);
    }
    open() { if (!this.dialog.open) this.dialog.showModal(); this.input.focus(); }
    update() {
      const marks = [];
      if (this.query) {
        const cursor = new SearchCursor(this.view.state.doc, this.query, 0, this.view.state.doc.length, text => text.toLowerCase());
        while (!cursor.next().done)
          marks.push(Decoration.mark({ class: "cm-searchMatch" }).range(cursor.value.from, cursor.value.to));
      }
      this.decorations = Decoration.set(marks);
      this.status.textContent = ` ${marks.length} matches `;
    }
    destroy() { this.button.remove(); this.dialog.remove(); }
  }, { decorations: value => value.decorations });
  return [plugin, Prec.highest(keymap.of([{ key: "Mod-f", run: view => {
    view.plugin(plugin)?.open(); return true;
  } }]))];
}
