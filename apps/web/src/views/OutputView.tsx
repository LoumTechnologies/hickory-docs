// The Output view: the document's generated files, editable on arrival. Edits
// resolve backwards through provenance into the source document, so the next
// run reproduces them exactly. The lineage strip traces whatever is under the
// cursor back to the prose that produced it.
//
// There is no "edit mode" — see components/OutputEditorPane.tsx.

import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "../api/client";
import type { OutputFile, OutputFileMeta, Provenance } from "../api/types";
import { OutputEditorPane, provToChars, type ProvChar } from "../components/OutputEditorPane";
import { useOutputRealtime } from "../api/useOutputRealtime";
import type { OutputProvenance } from "../lsp/outputMapping";
import type { Extension } from "@codemirror/state";

function originLabel(p: Provenance): string {
  if (p.origin.kind === "synthetic") return "synthetic (weaver-generated, not editable)";
  return `from ${p.origin.doc_path} bytes ${p.origin.span[0]}..${p.origin.span[1]} (${p.origin.kind})`;
}

export interface OutputViewProps {
  docId: string;
  /** Jump to a source span in the Document view (lineage click-through). */
  onSelectSpan: (span: [number, number]) => void;
  /** Build LSP bindings for an output buffer, given its position mapper. */
  makeOutputLsp?: (provenance: OutputProvenance[]) => Extension[];
  /** Open this generated file, positioned on the given line span. */
  outputTarget?: { path: string; span: [number, number] } | null;
}

export function OutputView({
  docId,
  onSelectSpan,
  makeOutputLsp,
  outputTarget,
}: OutputViewProps) {
  const [files, setFiles] = useState<OutputFileMeta[] | null>(null);
  const [activePath, setActivePath] = useState<string | null>(null);
  const [file, setFile] = useState<OutputFile | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [activeProv, setActiveProv] = useState<ProvChar[]>([]);
  const outputRealtime = useOutputRealtime(docId, activePath);

  useEffect(() => {
    api.outputs(docId).then(
      (r) => {
        setFiles(r.files);
        setActivePath((p) =>
          p && r.files.some((f) => f.path === p) ? p : (r.files[0]?.path ?? null),
        );
      },
      (e) => setError(e instanceof Error ? e.message : String(e)),
    );
  }, [docId]);

  const loadFile = useCallback(() => {
    if (!activePath) return;
    api.outputFile(docId, activePath).then(
      (f) => {
        setFile(f);
        setError(null);
      },
      (e) => setError(e instanceof Error ? e.message : String(e)),
    );
  }, [docId, activePath]);

  useEffect(loadFile, [loadFile]);

  // An LSP target landed in a generated file: open that file's tab.
  useEffect(() => {
    if (outputTarget) setActivePath(outputTarget.path);
  }, [outputTarget]);

  // Language bindings for this buffer: positions travel back through the
  // file's provenance before any question is asked about them.
  const outputLsp = useMemo(
    () => (makeOutputLsp && file ? makeOutputLsp(provToChars(file)) : []),
    [makeOutputLsp, file],
  );

  if (error) {
    return (
      <div className="output-view">
        <p className="error">{error}</p>
      </div>
    );
  }
  if (files === null) {
    return (
      <div className="output-view">
        <p className="muted">Loading outputs…</p>
      </div>
    );
  }
  if (files.length === 0) {
    return (
      <div className="output-view">
        <p className="muted">
          No generated outputs yet — this document has no woven files, or it has not
          completed a successful run.
        </p>
      </div>
    );
  }

  return (
    <div className="output-view">
      <div className="output-tabs" role="tablist">
        {files.map((f) => (
          <button
            key={f.path}
            role="tab"
            aria-selected={f.path === activePath}
            className={`output-tab mono${f.path === activePath ? " on" : ""}`}
            onClick={() => {
              setActivePath(f.path);
              setActiveProv([]);
            }}
          >
            {f.path}
          </button>
        ))}
      </div>

      <div className="output-body">
        {file && outputRealtime ? (
          <OutputEditorPane
            key={file.path}
            file={file}
            realtime={outputRealtime}
            extensions={outputLsp}
            onLineage={setActiveProv}
          />
        ) : (
          <p className="muted">Loading output…</p>
        )}
        <aside className="lineage-strip" data-testid="lineage-strip">
          <h3>Lineage</h3>
          {activeProv.length === 0 ? (
            <p className="muted">Put the cursor in the output to trace its origin.</p>
          ) : (
            <ul>
              {activeProv.map((p, i) => (
                <li key={i} className={p.origin.kind === "synthetic" ? "prov-synthetic" : ""}>
                  {p.origin.kind === "synthetic" ? (
                    <span>{originLabel(p)}</span>
                  ) : (
                    <button
                      className="btn btn-link"
                      onClick={() => {
                        const o = p.origin;
                        if (o.kind !== "synthetic") onSelectSpan(o.span);
                      }}
                    >
                      {originLabel(p)}
                    </button>
                  )}
                </li>
              ))}
            </ul>
          )}
        </aside>
      </div>
    </div>
  );
}
