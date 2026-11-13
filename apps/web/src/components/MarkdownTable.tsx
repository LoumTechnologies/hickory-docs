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
