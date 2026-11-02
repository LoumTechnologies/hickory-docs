<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Inspect a symbol with touch or a keyboard

A hover vanishes when a finger lifts. Build an explicit inspection gesture that asks the same client for documentation and navigation targets, then leaves presentation to the host. Both a live language client and the frozen static client implement this interface. Keep source-position mapping with the caller so generated code can use provenance too. A newer selection or a destroyed view must invalidate pending replies; a failed provider answer remains distinguishable from an empty result.

<static:copy id="static-apps-web-src-lsp-symbolInspection-ts-0">
import { EditorView, ViewPlugin } from "@codemirror/view";
import type { Extension } from "@codemirror/state";
import type { LspClient, LspLocation } from "./client";
import type { LspPosition } from "./positions";
import { hoverText } from "./cmLsp";

export interface SymbolInspection {
  label: string;
  documentation: string;
  definitions: LspLocation[];
  references: LspLocation[];
  unavailable: string[];
}

export function symbolInspection(options: {
  client: Pick<LspClient, "hover" | "definition" | "references">;
  targetAt: (offset: number, view: EditorView) => { uri: string; position: LspPosition } | null;
  onInspect: (value: SymbolInspection) => void;
}): Extension {
  return ViewPlugin.fromClass(class {
    private generation = 0;
    start: { id: number; x: number; y: number; time: number } | null = null;
    constructor(readonly view: EditorView) {}
    inspect(at: number) {
      const generation = ++this.generation;
      const target = options.targetAt(at, this.view);
      if (!target) return;
      const word = this.view.state.wordAt(at);
      const label = word ? this.view.state.sliceDoc(word.from, word.to) : "Symbol";
      void Promise.allSettled([
        options.client.hover(target.uri, target.position),
        options.client.definition(target.uri, target.position),
        options.client.references(target.uri, target.position),
      ]).then(([hover, definitions, references]) => {
        if (generation !== this.generation) return;
        options.onInspect({ label,
          documentation: hover.status === "fulfilled" ? hoverText(hover.value?.contents) : "",
          definitions: definitions.status === "fulfilled" ? definitions.value : [],
          references: references.status === "fulfilled" ? references.value : [],
          unavailable: [hover, definitions, references].flatMap((answer, i) => answer.status === "rejected" ? [["Documentation", "Definitions", "References"][i]] : []),
        });
      });
    }
    destroy() { ++this.generation; }
  }, {
    eventHandlers: {
      pointerdown(event) {
        this.start = event.pointerType === "touch" || event.pointerType === "pen"
          ? { id: event.pointerId, x: event.clientX, y: event.clientY, time: event.timeStamp } : null;
        return false;
      },
      pointermove(event) {
        if (this.start && Math.hypot(event.clientX - this.start.x, event.clientY - this.start.y) > 10) this.start = null;
        return false;
      },
      pointercancel() { this.start = null; return false; },
      pointerup(event) {
        const start = this.start; this.start = null;
        if (!start || start.id !== event.pointerId || event.timeStamp - start.time > 600
          || Math.hypot(event.clientX - start.x, event.clientY - start.y) > 10
          || !(event.target instanceof Element) || !event.target.closest(".cm-content")
          || event.target.closest("a, button, .source-prose-link")) return false;
        const at = this.view.posAtCoords({ x: event.clientX, y: event.clientY });
        if (at !== null) this.inspect(at);
        return false;
      },
      keydown(event) {
        if (event.altKey && event.key === "Enter") {
          this.inspect(this.view.state.selection.main.head);
          event.preventDefault();
          return true;
        }
        return false;
      },
    },
  });
}
</static:copy>

<static:file path="apps/web/src/lsp/symbolInspection.ts"><static:paste select="#static-apps-web-src-lsp-symbolInspection-ts-0"/></static:file>
</static:doc>
