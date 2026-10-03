import { useEffect, useMemo, useRef } from "react";
import { Compartment, StateEffect } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { completionSource, completionSources } from "../lsp/completion";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { diagnosticRanges, lspSupport, type DiagnosticSpan } from "../lsp/cmLsp";
import { lspFeatures } from "../lsp/cmLspFeatures";
import { openLocation, pathOfDocUri } from "../lib/revealLine";
import { filePosition, representationOffset, representationRange, type RepresentedFile } from "../lsp/representationMapping";
import { usePlainDebugSession } from "../debug/plainDebugHosts";
import { IDLE_SESSION, type DebugSession } from "../debug/useDebugger";
import { DebugStrip } from "../debug/DebugStrip";
import type { usePrompt } from "../components/PromptPanel";
import { api } from "../api/client";

export function RepresentationTools({ file, source, view, saved, onDiagnostics, onDebug, onMessage, askText, askChoice }: {
  file: RepresentedFile; source: string; view: EditorView | null; saved: boolean;
  onDiagnostics: (path: string, spans: DiagnosticSpan[]) => void;
  onDebug: (path: string, session: DebugSession) => void;
  onMessage: (message: string) => void;
  askText: ReturnType<typeof usePrompt>["askText"];
  askChoice: ReturnType<typeof usePrompt>["askChoice"];
}) {
  const lsp = useWorkspaceLsp(file.path, file.content);
  const debug = usePlainDebugSession(file.path) ?? IDLE_SESSION;
  const live = useRef({ source, file, debug }); live.current = { source, file, debug };
  const language = file.path.split(".").pop();
  const compartment = useMemo(() => new Compartment(), []);
  useEffect(() => {
    if (!view) return;
    const positionAt = (offset: number) => view.state.doc.toString() === live.current.source ? filePosition(live.current.source, live.current.file, offset) : null;
    const offsetAt = (position: { line: number; character: number }) => view.state.doc.toString() === live.current.source ? representationOffset(live.current.source, live.current.file, position) : null;
    const extensions = [
      ...lspSupport({ client: lsp.client, uri: lsp.uri, positionAt, onNavigate: target => {
        if (target.uri === lsp.uri) {
          const offset = offsetAt(target.range.start);
          if (offset !== null) view.dispatch({ selection: { anchor: offset }, scrollIntoView: true });
        } else { const path = pathOfDocUri(target.uri); if (path) openLocation(path, target.range.start.line + 1); }
      }, runtimeValue: word => live.current.debug.valueAt(word) }),
      ...lspFeatures({ client: lsp.client, uri: lsp.uri, positionAt, offsetAt,
        rangeAt: range => view.state.doc.toString() === live.current.source ? representationRange(live.current.source,live.current.file,range):null,
        onRename: current => askText(`Rename ${current} to:`, current),
        onCodeActions: actions => askChoice("Code actions", actions.map(action=>({label:action.title,value:action}))),
        onMessage,
      }),
      completionSources.of(completionSource({ lsp: lsp.client ? { client: lsp.client, uri: lsp.uri, positionAt } : undefined })),
    ];
    view.dispatch({ effects: compartment.get(view.state) === undefined ? StateEffect.appendConfig.of(compartment.of(extensions)) : compartment.reconfigure(extensions) });
    return () => { if (view.dom.isConnected) view.dispatch({ effects: compartment.reconfigure([]) }); };
  }, [view, lsp.client, lsp.uri, compartment, onMessage, askText, askChoice]);
  useEffect(() => {
    onDiagnostics(file.path, diagnosticRanges(lsp.diagnostics, p => representationOffset(source, file, p)));
  }, [lsp.diagnostics, file, source, onDiagnostics]);
  useEffect(() => () => { onDiagnostics(file.path, []); onDebug(file.path, IDLE_SESSION); }, [file.path, onDiagnostics, onDebug]);
  useEffect(() => { onDebug(file.path, debug); }, [file.path, debug, onDebug]);
  return <div className="representation-file-tools">
    <span className="mono">{file.path}</span>
    {["py", "js", "ts", "go", "cs"].includes(language ?? "") && <button className="btn btn-small" disabled={!saved} onClick={() => debug.start()}>Debug</button>}
    <DebugStrip status={debug.status} program={file.path} message={debug.message} capabilities={debug.capabilities}
      offerInstall={debug.offerInstall} onInstall={async offer => { await api.installTool(offer.kind, offer.language); debug.start(); }}
      frames={debug.frames} selectedFrame={debug.selectedFrame} watches={debug.watches} exitCode={debug.exitCode} buildOutput={debug.buildOutput}
      onSelectFrame={debug.selectFrame} onStep={debug.step} onStart={() => debug.start()} onStop={debug.stop}
      onJumpHere={() => { if (!view) return; const p = filePosition(source, file, view.state.selection.main.head); if (p) debug.jumpTo(p.line); }}
      onAddWatch={() => { void askText("Expression to watch:", "").then(expression=> { if (expression) debug.addWatch(expression); }); }}
      exceptionFilters={debug.exceptionFilters} onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch} />
  </div>;
}
