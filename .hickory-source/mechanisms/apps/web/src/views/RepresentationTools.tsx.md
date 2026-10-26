<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/RepresentationTools.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f5ea4f306ebf"></a>
## `apps/web/src/views/RepresentationTools.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A comparison keeps current code editable](../../../../../guarantees/editing/a-comparison-keeps-current-code-editable/contract.md).

<a id="f-f5ea4f306ebf-RepresentationTools"></a>
### `RepresentationTools`

<storycode:copy id="f-f5ea4f306ebf-RepresentationTools">
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

export function RepresentationTools({ file, source, view, onStart, onDiagnostics, onDebug, onMessage, askText, askChoice }: {
  file: RepresentedFile; source: string; view: EditorView | null; saved: boolean; onStart: () => void;
  onDiagnostics: (path: string, spans: DiagnosticSpan[]) => void;
  onDebug: (path: string, session: DebugSession) => void;
  onMessage: (message: string) => void;
  askText: ReturnType<typeof usePrompt>["askText"];
  askChoice: ReturnType<typeof usePrompt>["askChoice"];
}) {
  const lsp = useWorkspaceLsp(file.path, file.content);
  const debug = usePlainDebugSession(file.path) ?? IDLE_SESSION;
  const live = useRef({ source, file, debug }); live.current = { source, file, debug };
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
        onRename: current => askText(`Rename ${current} to:`, current, { agent: true, currentRevision: () => view.state.doc }),
        onCodeActions: actions => askChoice("Code actions", actions.map(action=>({label:action.title,value:action})), { agent: true, currentRevision: () => view.state.doc }),
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
  const pausedOffset = debug.pausedLine === null ? null : representationOffset(source, file, { line: debug.pausedLine, character: 0 });
  const pausedLine = pausedOffset === null || !view ? null : view.state.doc.lineAt(pausedOffset).number - 1;
  return <>
    <DebugStrip reading={debug.reading} editorView={view} pausedLine={pausedLine} status={debug.status} program={file.path} message={debug.message} capabilities={debug.capabilities}
      offerInstall={debug.offerInstall} onInstall={async offer => { await api.installTool(offer.kind, offer.language); }}
      frames={debug.frames} selectedFrame={debug.selectedFrame} watches={debug.watches} exitCode={debug.exitCode} buildOutput={debug.buildOutput}
      onSelectFrame={debug.selectFrame} onStep={debug.step} onStart={onStart} onStop={debug.stop}
      onJumpHere={() => { if (!view) return; const p = filePosition(source, file, view.state.selection.main.head); if (p) debug.jumpTo(p.line); }}
      onAddWatch={() => { void askText("Expression to watch:", "", { agent: true }).then(expression=> { if (expression) debug.addWatch(expression); }); }}
      exceptionFilters={debug.exceptionFilters} onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch} />
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/views/RepresentationTools.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/RepresentationTools.tsx"><storycode:paste select="#f-f5ea4f306ebf-RepresentationTools"/></storycode:file>

</storycode:doc>
