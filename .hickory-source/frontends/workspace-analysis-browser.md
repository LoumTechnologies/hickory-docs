<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Bound workspace knowledge acquisition

For maintainers changing workspace services independently of the editor renderer.

Workspace analysis is a bounded acquisition of source-bound knowledge, not a verification verdict. Two workers load available source and then await one outline read before taking another file. Cancellation stops admission while in-flight reads drain, so restarting cannot multiply concurrency. A source changed during a read produces an explicit stale result; earlier knowledge is used only with its exact source. UI renderers consume this ordinary workspace model without owning its scheduler. Failures belong to individual files and never hide readable text or stop independent workers.

<ui:copy id="simple-workspace-analysis-browser">
import type { FileNode } from "../api/types";
import { workspaceAnalysisScope, type SemanticLocation } from "./Intelligence";
export interface AnalysisKnowledge { path: string; source?: string; locations: SemanticLocation[]; state: "ready" | "stale" | "unavailable"; notice: string }
export class WorkspaceAnalysis {
  running = false; stopping = false; completed = 0; scope = workspaceAnalysisScope([]);
  knowledge = new Map<string, AnalysisKnowledge>();
  private listeners = new Set<() => void>(); private version = 0;
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => this.listeners.delete(listener); };
  snapshot = () => this.version;
  private changed() { this.version++; for (const listener of this.listeners) listener(); }
  stop() { if (this.running) { this.stopping = true; this.changed(); } }
  async run(files: readonly FileNode[], load: (file: FileNode) => Promise<{ text: string }>, outline: (file: FileNode, source: string) => Promise<SemanticLocation[]>) {
    if (this.running) return false;
    this.scope = workspaceAnalysisScope(files); this.running = true; this.stopping = false; this.completed = 0; this.knowledge.clear(); this.changed();
    let cursor = 0;
    const worker = async () => {
      while (!this.stopping && cursor < this.scope.files.length) {
        const file = this.scope.files[cursor++]; let source: string | undefined;
        try {
          const buffer = await load(file); source = buffer.text;
          const locations = await outline(file, source);
          this.knowledge.set(file.path, { path: file.path, source, locations: buffer.text === source ? locations : [], state: buffer.text === source ? "ready" : "stale", notice: buffer.text === source ? "Provider outline returned; diagnostics arrive independently." : "Source changed during analysis. Run again." });
        } catch (error) {
          this.knowledge.set(file.path, { path: file.path, source, locations: [], state: "unavailable", notice: error instanceof Error ? error.message : String(error) });
        }
        this.completed++; this.changed();
      }
    };
    try { await Promise.all([worker(), worker()]); } finally { this.running = false; this.stopping = false; this.changed(); }
    return true;
  }
  current(path: string, source: string) { const item = this.knowledge.get(path); return item?.state === "ready" && item.source === source ? item : null; }
}
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceAnalysis.ts"><ui:paste select="#simple-workspace-analysis-browser"/></ui:file>
</ui:doc>
