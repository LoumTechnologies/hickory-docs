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
