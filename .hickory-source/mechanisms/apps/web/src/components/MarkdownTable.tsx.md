<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MarkdownTable.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-04adf2e0e8b7"></a>
## `apps/web/src/components/MarkdownTable.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Cell formatting keeps numbers numeric](../../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md), [Markdown tables use the table editor](../../../../../guarantees/authoring/markdown-tables-use-the-table-editor/contract.md).

<a id="f-04adf2e0e8b7-MarkdownTable"></a>
### `MarkdownTable`

The native bundle mounts this same Markdown/CSV adapter, rather than maintaining another spreadsheet implementation. Each edit names an exact host revision and waits for a new authoritative snapshot before another write. Layout remains local. Formula RPC is limited to language discovery, evaluation and tracing; its bounded requests never grant the local page general network access. Host source changes update the existing React grid so selection and size survive accepted edits. A native rejection refreshes the snapshot and never reports a save.

Initially fill the native editor’s already reserved rectangle instead of shrinking the document when the richer capability arrives. Once the user explicitly changes viewport height, let that chosen scroller height determine the native anchor. Row insertion and column sizing do not silently contract the original viewport.

<storycode:copy id="f-04adf2e0e8b7-MarkdownTable">
import { Component, useState, type ReactNode } from "react";
import { createRoot } from "react-dom/client";
import { TablePanel, type TableLayout, type TablePanelProps } from "./TablePanel";
import { parseCsv, writeCsv } from "../lib/csv";
import { splitFormatComment, withFormatComment } from "../lib/tableFormats";
import { markdownRows, writeMarkdownTable } from "../lib/markdownTable";

/** The existing spreadsheet surface, backed by Markdown rather than CSV. */
export function MarkdownTable({ source, onChange, ...props }: TablePanelProps) {
  const { body, formats } = splitFormatComment(source);
  const csv = writeCsv({ rows: markdownRows(body), delimiter: ",", newline: "\n", trailingNewline: false });
  return <TablePanel fitProseDefault {...props} source={csv} formats={formats} delimiter="," header
    onChange={onChange ? (next, formats = "", actor) => { const text = withFormatComment(writeMarkdownTable(body, parseCsv(next, ",").rows), formats); if (actor) onChange(text, undefined, actor); else onChange(text); } : undefined} />;
}

export interface NativeTableSnapshot {
  revision: string; source: string; format: "csv" | "markdown"; readonly: boolean;
  delimiter?: string; language?: string; header?: boolean;
}
type NativeTableMessage = { kind: string; revision?: string; [key: string]: unknown };
/** The native page has no network capability. Only formula RPC and guarded edits leave it. */
export function mountNativeTable(host: HTMLElement, post: (message: NativeTableMessage) => void) {
  host.classList.add("native-table-fill");
  let current: NativeTableSnapshot | null = null, writing = false, serial = 0;
  const pending = new Map<number, { resolve: (value: unknown) => void; reject: (error: Error) => void; timer: ReturnType<typeof setTimeout> }>();
  const rpc = <T,>(operation: string, language?: string, rows?: string[][]): Promise<T> => new Promise((resolve, reject) => {
    const id = ++serial;
    const timer = setTimeout(() => { pending.delete(id); reject(Error("Formula request timed out. Check the workspace connection and retry.")); }, 30000);
    pending.set(id, { resolve: value => resolve(value as T), reject, timer });
    post({ kind: "formula", id, operation, language, rows });
  });
  const formulaApi: NonNullable<TablePanelProps["formulaApi"]> = {
    formulaLanguages: () => rpc("languages"), evaluateFormulas: (language, rows) => rpc("evaluate", language, rows), traceFormulas: (language, rows) => rpc("trace", language, rows),
  };
  function Surface({ snapshot }: { snapshot: NativeTableSnapshot }) {
    const [layout, setLayout] = useState<TableLayout>({});
    const Panel = snapshot.format === "markdown" ? MarkdownTable : TablePanel;
    return <Panel source={snapshot.source} delimiter={snapshot.delimiter} language={snapshot.language} header={snapshot.header} chooseFormulaLanguage formulaApi={formulaApi}
      sourceIdentity="native-inline-table" sourceCurrent={() => current === snapshot && !writing && !current.readonly}
      layout={layout} onLayout={next => { if (next.height !== undefined) host.classList.remove("native-table-fill"); setLayout(next); }}
      onChange={snapshot.readonly ? undefined : source => {
        if (current !== snapshot || writing) return;
        writing = true;
        post({ kind: "change", revision: snapshot.revision, source });
      }} />;
  }
  class Boundary extends Component<{ children: ReactNode }, { failed: boolean }> {
    state = { failed: false };
    static getDerivedStateFromError() { return { failed: true }; }
    componentDidCatch() { post({ kind: "unavailable" }); }
    render() { return this.state.failed ? null : this.props.children; }
  }
  const root = createRoot(host);
  let lastHeight = 0;
  const observer = new ResizeObserver(() => {
    const height = Math.max(180, Math.min(1200, Math.ceil(host.getBoundingClientRect().height) + 28));
    if (current && height !== lastHeight) { lastHeight = height; post({ kind: "height", revision: current.revision, height }); }
  });
  observer.observe(host);
  const bridge = {
    update(snapshot: NativeTableSnapshot) {
      current = snapshot; writing = false;
      root.render(<Boundary><Surface snapshot={snapshot} /></Boundary>);
    },
    reply(id: number, value: unknown, error?: string) {
      const request = pending.get(id); if (!request) return;
      pending.delete(id); clearTimeout(request.timer);
      if (error) request.reject(Error(error)); else request.resolve(value);
    },
    dispose() { observer.disconnect(); root.unmount(); for (const request of pending.values()) { clearTimeout(request.timer); request.reject(Error("Table closed")); } pending.clear(); },
  };
  post({ kind: "ready" });
  return bridge;
}
</storycode:copy>

### Assemble `apps/web/src/components/MarkdownTable.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MarkdownTable.tsx"><storycode:paste select="#f-04adf2e0e8b7-MarkdownTable"/></storycode:file>

</storycode:doc>
