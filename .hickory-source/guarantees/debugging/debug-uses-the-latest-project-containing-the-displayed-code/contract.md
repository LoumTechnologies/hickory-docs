<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Debug the latest atomic project that still contains the displayed bytes

For: Authors debugging generated code while navigating its document source.

A caller and library change belong to one atomic scope. Debugging must not combine the new caller with the old library simply because that intermediate state existed during editing.

## What must happen

A launch selects the latest supported complete project endpoint containing the exact displayed code. Candidate states honor atomic scopes; dependency-local editing states do not automatically become launch candidates. Changed source or unavailable exact mapping refuses selection. The launch profile chooses the executable within that selected tree, while breakpoint routing preserves the viewed source identity.

## Limits of the evidence

Source selection is separate from language-server endpoint choice and from a successful native launch. Exact-source checks must be repeated at the host boundary.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Use the atomic-prefix correction and exact-source launch-profile rule, replacing the earlier permissive cumulative-state interpretation.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-455afcfce7fd"></a>
## `apps/web/src/components/ComparisonTools.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](contract.md), [Patch layout exposes endpoint breakpoints](../../literate-projects/patch-layout-breakpoints/contract.md).

<a id="f-455afcfce7fd-decode"></a>
### `decode`

<storycode:copy id="f-455afcfce7fd-decode">
import { inlineEvaluationOwner } from "../debug/inlineEval";
import { CapturedReferences } from "./CapturedReferences";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { launchWithProfiles } from "../debug/launchProfiles";
import { revealPreferredDebugLocation } from "../debug/preferredDebugLocation";
import { preferCapturedLiterateSurface, registerCapturedComparison, ProjectDebugLinks } from "../debug/projectDebugSurface";
import { PromptPanel, usePrompt } from "./PromptPanel";
import { completionSource, completionSources } from "../lsp/completion";
import { useEffect, useId, useMemo, useRef, useState, type ReactNode } from "react";
import { Compartment, StateEffect } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import { api } from "../api/client";
import { latestDebugProject, projectPosition, type ProjectEntry } from "../editor/hickLang";
import type { ComparisonSource } from "../lib/projectComparison";
import { openCapturedCode } from "../lib/capturedCode";
import { acquireCapturedContext, retainCapturedContext } from "../lib/capturedContexts";
import { useFollowingDebugSession } from "../debug/plainDebugHosts";
import { IDLE_SESSION } from "../debug/useDebugger";
import { DebugLaunchAction } from "../debug/DebugRail";
import { revealLine } from "../debug/cmDebug";
import { isDebuggable } from "../debug/languages";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { lspSupport, setLspDiagnostics, diagnosticRanges } from "../lsp/cmLsp";
import { debugEditor, debugStateEffects } from "../debug/cmDebug";
import { DebugStrip } from "../debug/DebugStrip";
import { languageFromPath } from "../editor/languages";

const decode = (entry: ProjectEntry) => new TextDecoder("utf-8", {fatal:true}).decode(new Uint8Array(entry.bytes));
</storycode:copy>
<a id="f-455afcfce7fd-offset"></a>
### `offset`

<storycode:copy id="f-455afcfce7fd-offset">
function offset(text: string, line: number, column: number): number | null {
  const lines = text.split("\n");
  if (line >= lines.length || column > lines[line].length) return null;
  return lines.slice(0,line).reduce((sum,s) => sum+s.length+1,0)+column;
}
</storycode:copy>
<a id="f-455afcfce7fd-position"></a>
### `position`

<storycode:copy id="f-455afcfce7fd-position">
function position(bytes: number[], byte: number) {
  const prefix = new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(bytes.slice(0,byte)));
  const lines = prefix.split("\n"); return {line:lines.length-1,character:lines.at(-1)!.length};
}
</storycode:copy>
<a id="f-455afcfce7fd-export_statement"></a>
### `export_statement`

<storycode:copy id="f-455afcfce7fd-export_statement">
export { openCapturedCode } from "../lib/capturedCode";

</storycode:copy>
<a id="f-455afcfce7fd-ComparisonTools"></a>
### `ComparisonTools`

Opening the view registers its isolated snapshot; native tools use only that captured project.

<storycode:copy id="f-455afcfce7fd-ComparisonTools">
/** Opening the view registers its isolated snapshot; native tools use only that captured project. */
export function ComparisonTools({path,source,view,children}: {path:string;source:ComparisonSource;view:EditorView|null;children?: (debug: typeof IDLE_SESSION | null) => ReactNode}) {
  const prompt = usePrompt();
  const [registration,setContext] = useState<{id:string;uri:string;project:ProjectEntry[]}|null>(null);
  const [references,setReferences] = useState<{ scope: string; locations: import("../lsp/client").LspLocation[] } | null>(null);
  const [error,setError] = useState("");
  const [busy,setBusy] = useState(false);
  const [retry,setRetry] = useState(0);
  const tools = useMemo(() => new Compartment(),[]);
  const local = source.local ?? source.project;
  const selection = useMemo(() => {
    if (!local?.some(e => e.path === path) || !source.project) return null;
    try {
      return { ...latestDebugProject(source.versions ?? [source.project], local, path), error: "" };
    } catch (e) {
      return { project: undefined, path, latest: false, error: e instanceof Error ? e.message : String(e) };
    }
  }, [source.project, source.versions, local, path]);
  const selected = selection?.project;
  const displayed = local?.find(e => e.path === path);
  const activePath = selection?.path;
  const active = selected?.find(e => e.path === activePath && e.kind === "file");
  const displayMatches = !!displayed && decode(displayed) === source.text;
  const context = registration?.project === selected && displayMatches ? registration : null;
  const uri = context && active ? `${context.uri}${active.path}` : "";
  const referenceScope = JSON.stringify([context?.id, uri, source.id, source.text]);
  const lsp = useWorkspaceLsp(uri,active ? decode(active) : "");
  const debug = useFollowingDebugSession(uri, source.debugOwner) ?? IDLE_SESSION;
  const live = useRef({source,local,selected,context,debug}); live.current={source,local,selected,context,debug};
  const exact = !!active && !!displayed && decode(active) === source.text;
  useEffect(() => {
    let live = true;
    let release: (() => void) | undefined;
    setContext(null); setError("");
    if (!selected || !active || !displayMatches) { setBusy(false); return; }
    if (source.codeUri) {
      const match = /^hick-project:\/\/\/([^/]+)\//.exec(source.codeUri);
      if (match) {
        release = retainCapturedContext(source.codeUri);
        setContext({id:match[1],uri:match[0],project:selected}); setBusy(false);
        return release;
      }
    }
    setBusy(true);
    const entries = Object.fromEntries(selected.map(e => [e.path, { bytes: e.bytes, kind: e.kind === "file" ? "File" : "Symlink", executable: e.executable, origins: e.origins ?? [] }]));
    void acquireCapturedContext(entries,source.debugSurface?.path?.split("/").slice(0,-1).join("/") ?? source.target ?? "").then(next => {
      const dispose = next.release;
      if (!live) { dispose(); return; }
      release = dispose;
      setContext({ ...next, project: selected });
    }, e => { if (live) setError(e instanceof Error ? e.message : String(e)); })
      .finally(() => { if (live) setBusy(false); });
    return () => { live = false; release?.(); };
  }, [selected, active, displayMatches, retry, source.codeUri]);
  useEffect(() => {
    if (!view || !context || !local || !selected || !displayMatches) return;
    const targetAt = (at: number, v: EditorView) => {
        const current=live.current;
        if (v.state.doc.toString() !== current.source.text || !current.local || !current.selected || !current.context) return null;
        try {
          const target=projectPosition(current.local,current.selected,path,new TextEncoder().encode(current.source.text.slice(0,at)).length);
          if (!target) return null;
          return {uri:`${current.context.uri}${target[0]}`,position:position(current.selected.find(e=>e.path===target[0])!.bytes,target[1])};
        } catch { return null; }
      };
    const extensions = [
      ...lspSupport({client:lsp.client,uri:lsp.uri,positionAt:() => null,targetAt,
      onNavigate:target => {
        if (target.uri === uri && active) {
          const at = offset(decode(active),target.range.start.line,target.range.start.character);
          if (at !== null) {
            try {
              const localTarget = projectPosition(selected,local,active.path,new TextEncoder().encode(decode(active).slice(0,at)).length);
              if (localTarget?.[0] === path) {
                const anchor = new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(displayed!.bytes.slice(0,localTarget[1]))).length;
                view.dispatch({selection:{anchor},scrollIntoView:true}); view.focus(); return;
              }
            } catch { /* Open the exact captured source when no local mapping exists. */ }
          }
        }
        void openCapturedCode(target, source.debugOwner ?? uri).catch(e=>setError(String(e)));
      },
      onReferences:locations => { if (live.current.context === context && view.state.doc.toString() === source.text) setReferences({ scope: referenceScope, locations }); },
      runtimeValue:word => exact ? live.current.debug.valueAt(word) : Promise.resolve(null),
    }),
      completionSources.of(async completion => {
        if (!lsp.client || completion.state !== view.state) return null;
        const target = targetAt(completion.pos, view);
        if (!target) return null;
        const captured = live.current.context;
        const result = await completionSource({ lsp: { client: lsp.client, uri: target.uri, positionAt: () => target.position } })(completion);
        return captured === live.current.context && completion.state === view.state ? result : null;
      }),
    ];
    if (exact) extensions.push(...debugEditor({language:languageFromPath(path),onToggleBreakpoint:line=>live.current.debug.toggleBreakpoint(line),onSelectFrame:id=>live.current.debug.selectFrame(id),evaluationOwner: () => inlineEvaluationOwner(live.current.debug),
      onEvaluate:expression=>live.current.debug.query(expression),onAddWatch:expression=>live.current.debug.addWatch(expression),onSetBreakpointCondition:(line,patch)=>live.current.debug.setBreakpointCondition(line,patch)}));
    view.dispatch({effects:tools.get(view.state) === undefined ? StateEffect.appendConfig.of(tools.of(extensions)) : tools.reconfigure(extensions)});
    return () => { if (view.dom.isConnected) view.dispatch({effects:tools.reconfigure([])}); };
  },[view,context,lsp.client,lsp.uri,path,exact,tools,local,selected,displayMatches,source.text,referenceScope]);
  useEffect(() => { if (view && context && exact) view.dispatch({effects:debugStateEffects(debug)}); },[view,context,exact,debug]);
  useEffect(()=> {
    if(view && context && exact && displayMatches)return registerCapturedComparison(uri,view,source.text,source.debugSurface?.view);
  },[view,context,exact,displayMatches,uri,source.text,source.debugSurface?.view]);
  useEffect(() => {
    if (!view || !active || !local || !selected || !context || !displayMatches) { view?.dispatch({effects:setLspDiagnostics.of([])}); return; }
    const spans=diagnosticRanges(lsp.diagnostics,p=> {
      const at=offset(decode(active),p.line,p.character); if (at === null) return null;
      try {
        const target=projectPosition(selected,local,active.path,new TextEncoder().encode(decode(active).slice(0,at)).length);
        return target?.[0] === path ? new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(displayed!.bytes.slice(0,target[1]))).length : null;
      } catch { return null; }
    });
    view.dispatch({effects:setLspDiagnostics.of(spans)});
  },[view,lsp.diagnostics,context,active,local,selected,path,displayed,displayMatches]);
  const launch = (choose = false) => {
    if (!context || !exact || !displayMatches || (view && view.state.doc.toString() !== source.text)) {
      setError("The displayed code changed. Wait for it to save or reload it before debugging.");
      return;
    }
    const surface = source.debugSurface;
    if (surface && selected && context) {
      preferCapturedLiterateSurface(uri, context.uri, selected, {...surface,codeView:view,codeUri:uri,codeText:source.text});
    }
    void launchWithProfiles({uri}, profile=>debug.start(undefined,{},undefined,profile),
      {choose,contextLabel:selection?.latest?"Completed project":"Earlier compatible atomic project",isCurrent:()=>!view || (view.dom.isConnected && view.state.doc.toString()===source.text)});
  };
  const canDebug = !!active && isDebuggable(languageFromPath(active.path));
  const selectedFrame = debug.frames.find(frame => frame.id === debug.selectedFrame);
  useSelectedFrame(view, exact ? (selectedFrame?.line ?? debug.pausedLine) : null);
  const setup = `comparison-tools:${useId()}`;
  const setupRevision = useUiRevision(() => [source.id, source.text, path, uri, exact, displayMatches, busy, error, retry]);
  const setupAction = useUiCommands(error ? [{ key: setup, command: { parent: setup, subject: setup, label: "Retry captured code tools", parameters: {}, prerequisites: [], read: () => ({ revision: setupRevision(), enabled: !busy }), execute: async () => { setRetry(n => n + 1); return { status: "completed", value: { registration_requested: true, ready: false } }; } } }] : []);
  useUiReading("captured code tools status", { path, label: source.label, uri: uri || null, exact, display_matches: displayMatches, registering: busy, error: error || selection?.error || null, scope: "Actual captured-code controller state; registration is not execution or verification" }, [source.id, source.text, uri]);
  return <>
    {children?.(context && exact && canDebug ? debug : null)}
    {canDebug && <DebugLaunchAction view={view} label={`Debug ${path} · ${source.label}`} disabled={!context || busy || !displayMatches || !exact} onStart={()=>launch()} onProfiles={()=>launch(true)} hint={selection?.latest ? "Debug this code · latest project" : "Debug this code · latest project containing this version of the file"} />}
    <DebugStrip {...debug} locationActions={<ProjectDebugLinks owner={source.debugOwner ?? uri} debug={debug}/>} editorView={view} pausedLine={exact ? debug.pausedLine : null} program={path}
      onSelectFrame={id=> {
        debug.selectFrame(id); const frame=debug.frames.find(f=>f.id===id);
        if (frame && revealPreferredDebugLocation(source.debugOwner ?? uri, frame)) return;
        if (frame?.source?.startsWith("hick-project:///") && frame.source!==uri)
          void openCapturedCode({uri:frame.source,range:{start:{line:frame.source_line ?? 0,character:0},end:{line:frame.source_line ?? 0,character:0}}},source.debugOwner ?? uri).catch(e=>setError(String(e)));
        else if (view && frame?.line !== null && frame?.line !== undefined) revealLine(view,frame.line);
      }} onStep={debug.step} onStart={()=>launch()} onStop={debug.stop}
      onJumpHere={()=> {if(view && exact) debug.jumpTo(view.state.doc.lineAt(view.state.selection.main.head).number-1);}}
      onAddWatch={()=>{void prompt.askText("Expression to watch:", "", { agent: true }).then(expression=>{if(expression)debug.addWatch(expression);});}}
      onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch}
      onInstall={async offer=> {await api.installTool(offer.kind,offer.language);}} />
    {selection?.error && <div className="comparison-tools"><p role="alert">{selection.error}</p></div>}
    {error && <div className="comparison-tools">
      <p role="alert">Could not start code tools: {error}</p>
      <button className="btn" onClick={() => void setupAction(setup).catch(() => undefined)}>Retry code tools</button>
    </div>}
    <PromptPanel prompt={prompt.prompt} onSettle={prompt.settle} />
    {references?.scope === referenceScope && <CapturedReferences targets={references.locations} scope={referenceScope} owner={source.debugOwner ?? uri}
      current={() => !!context && displayMatches && (!view || (view.dom.isConnected && view.state.doc.toString() === source.text))}
      onClose={() => setReferences(null)} onError={setError} />}

  </>;
}

</storycode:copy>
<a id="f-455afcfce7fd-useSelectedFrame"></a>
### `useSelectedFrame`

<storycode:copy id="f-455afcfce7fd-useSelectedFrame">
function useSelectedFrame(view: EditorView | null, line: number | null | undefined) {
  useEffect(()=> { if (view && line !== null && line !== undefined) revealLine(view,line); },[view,line]);
}
</storycode:copy>

### Assemble `apps/web/src/components/ComparisonTools.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ComparisonTools.tsx"><storycode:paste select="#f-455afcfce7fd-decode"/><storycode:paste select="#f-455afcfce7fd-offset"/><storycode:paste select="#f-455afcfce7fd-position"/><storycode:paste select="#f-455afcfce7fd-export_statement"/><storycode:paste select="#f-455afcfce7fd-ComparisonTools"/><storycode:paste select="#f-455afcfce7fd-useSelectedFrame"/></storycode:file>

<a id="output-16d8467a71c2"></a>
## `apps/web/src/debug/GeneratedDebugTools.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](contract.md).

<a id="f-16d8467a71c2-generatedLineMap"></a>
### `generatedLineMap`

<storycode:copy id="f-16d8467a71c2-generatedLineMap">
import { inlineEvaluationOwner } from "./inlineEval";
import { observationSourceChanged } from "./observations";
import { launchWithProfiles } from "./launchProfiles";
import { useEffect, useMemo, useRef } from "react";
import { Compartment, StateEffect } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import { parseHickDoc } from "../editor/hickDoc";
import type { OutputFile } from "../api/types";
import { api } from "../api/client";
import { provToChars } from "../components/OutputEditorPane";
import { sourcePositionAt } from "../lsp/outputMapping";
import { DebugLaunchAction } from "./DebugRail";
import { DebugStrip } from "./DebugStrip";
import { debugEditor, debugStateEffects, revealLine } from "./cmDebug";
import { isDebuggable } from "./languages";
import { languageFromPath } from "../editor/languages";
import type { DebugSession } from "./useDebugger";

/** Exact line correspondence; duplicate source occurrences remain ambiguous. */
export function generatedLineMap(file: OutputFile, docPath: string, source: string): Map<number, number> {
  const result = new Map<number, number>(), provenance = provToChars(file);
  let at = 0;
  file.content.split("\n").forEach((text,line) => {
    const position = sourcePositionAt(at, provenance, docPath, source);
    if (position) result.set(line, position.line);
    at += text.length + 1;
  });
  return result;
}
</storycode:copy>
<a id="f-16d8467a71c2-GeneratedDebugTools"></a>
### `GeneratedDebugTools`

<storycode:copy id="f-16d8467a71c2-GeneratedDebugTools">
export function GeneratedDebugTools({view,file,debug,docPath,source,askText}: {
  view: EditorView | null; file: OutputFile; debug: DebugSession; docPath: string; source: string;
  askText: (title:string,initial:string)=>Promise<string|null>;
}) {
  const tools = useMemo(()=>new Compartment(),[]);
  const mapping = useMemo(()=>generatedLineMap(file,docPath,source),[file,docPath,source]);
  const outputLine = (line:number|null) => {
    const candidates = [...mapping].filter(([,sourceLine])=>sourceLine===line);
    return candidates.length===1 ? candidates[0][0] : null;
  };
  const current = useRef({debug,mapping,file}); current.current={debug,mapping,file};
  useEffect(()=> {
    if (!view || !isDebuggable(languageFromPath(file.path))) return;
    const sourceLine = (line:number) => view.state.doc.toString()===current.current.file.content ? current.current.mapping.get(line) : undefined;
    const extensions = debugEditor({language:languageFromPath(file.path),lineNumbers:false,
      onToggleBreakpoint:line=>{const at=sourceLine(line);if(at!==undefined)current.current.debug.toggleBreakpoint(at);},
      onSetBreakpointCondition:(line,patch)=>{const at=sourceLine(line);if(at!==undefined)current.current.debug.setBreakpointCondition(at,patch);},
      onSelectFrame:id=>current.current.debug.selectFrame(id),
      evaluationOwner: () => inlineEvaluationOwner(current.current.debug),
      onEvaluate:expression=>current.current.debug.query(expression),onAddWatch:expression=>current.current.debug.addWatch(expression),
    });
    view.dispatch({effects:tools.get(view.state)===undefined?StateEffect.appendConfig.of(tools.of(extensions)):tools.reconfigure(extensions)});
    return ()=>{if(view.dom.isConnected)view.dispatch({effects:tools.reconfigure([])});};
  },[view,file.path,tools]);
  const activeHere = debug.program === file.path;
  const pausedLine = activeHere ? outputLine(debug.frames.find(frame=>frame.id===debug.selectedFrame)?.line ?? debug.pausedLine) : null;
  useEffect(()=> {
    if (!view || !isDebuggable(languageFromPath(file.path))) return;
    const exact = view.state.doc.toString()===file.content;
    view.dispatch({effects:debugStateEffects({...debug,pausedLine:exact?pausedLine:null,
      breakpoints:debug.breakpoints.flatMap(bp=>{const line=outputLine(bp.line);return line===null?[]:[{...bp,line}];}),
      frames:activeHere ? debug.frames.map(frame=>({...frame,line:outputLine(frame.line)})) : [],
      variables:activeHere?debug.variables:[], watches:activeHere?debug.watches:[],
    })});
    if(exact && pausedLine!==null)revealLine(view,pausedLine);
  },[view,file,debug,pausedLine,mapping,activeHere]);
  if(!isDebuggable(languageFromPath(file.path)))return null;
  const start=(choose=false)=>{
    if(view?.state.doc.toString()!==file.content)return;
    const project=parseHickDoc(source).blocks.some(b=>b.name==="project")?`latest-code:${JSON.stringify([file.path,file.content,source])}`:"new";
    void launchWithProfiles({uri:`hick:///${docPath.replace(/^\/+/,"")}`,program:file.path,project,expected_document:source,expected_source:file.content},
      profile=>debug.start(file.path,{},project,profile),{choose,isCurrent:()=>view.state.doc.toString()===file.content});
  };
  return <>
    <DebugLaunchAction view={view} label={`Debug ${file.path}`} onStart={()=>start()} onProfiles={()=>start(true)} disabled={!view || view.state.doc.toString()!==file.content} hint={view?.state.doc.toString()!==file.content?"Waiting for your edits to save before debugging":undefined}/>
    <DebugStrip {...debug} readingChanged={observationSourceChanged(debug.reading, source)} status={activeHere?debug.status:"idle"} editorView={view} pausedLine={pausedLine} program={file.path}
      onStep={debug.step} onStart={()=>start()} onStop={debug.stop} onSelectFrame={debug.selectFrame}
      onJumpHere={()=>{if(view){const line=mapping.get(view.state.doc.lineAt(view.state.selection.main.head).number-1);if(line!==undefined)debug.jumpTo(line);}}}
      onAddWatch={()=>{void askText("Expression to watch:","").then(expression=>{if(expression)debug.addWatch(expression);});}}
      onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch}
      onInstall={offer=>api.installTool(offer.kind,offer.language).then(()=>undefined)}/>
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/debug/GeneratedDebugTools.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/GeneratedDebugTools.tsx"><storycode:paste select="#f-16d8467a71c2-generatedLineMap"/><storycode:paste select="#f-16d8467a71c2-GeneratedDebugTools"/></storycode:file>

<a id="output-c0a4d45aa387"></a>
## `apps/web/src/debug/documentDebugContext.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](contract.md).

<a id="f-c0a4d45aa387-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c0a4d45aa387-expression_statement">
import { expect, it } from "vitest";
import { documentDebugContext } from "./documentDebugContext";
import { parseHickDoc } from "../editor/hickDoc";
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
it("identifies the clicked construction and fences the exact reading, including Unicode and BOM", () => {
  const source = '\uFEFF<hick:project/>\n🦀\n<hick:file path="app.py">print(1)\n</hick:file>';
  const at = source.indexOf('<hick:file');
  const context = documentDebugContext("folder/p.md", source, at);
  const [id, reading] = JSON.parse(context.slice("latest-file:".length));
  expect(id).toBe(`p.md:${new TextEncoder().encode(source.slice(1, at)).length}`);
  expect(reading).toBe(source);
});
</storycode:copy>
<a id="f-c0a4d45aa387-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c0a4d45aa387-expression_statement-2">
it("keeps repeated filenames distinct through construction identities", () => {
  const source = '<hick:project/><hick:file id="first" path="app.py">a</hick:file><hick:file id="second" path="app.py">b</hick:file>';
  const blocks = parseHickDoc(source).blocks.filter(b => b.name === "file");
  expect(JSON.parse(documentDebugContext("p.md", source, blocks[1].from).slice(12))[0]).toBe("second");
});
</storycode:copy>
<a id="f-c0a4d45aa387-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c0a4d45aa387-expression_statement-3">
it("keeps ordinary non-project script debugging", () => {
  expect(documentDebugContext("p.md", '<hick:file path="app.py">print(1)</hick:file>', 0)).toBe("new");
});
</storycode:copy>

### Assemble `apps/web/src/debug/documentDebugContext.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/documentDebugContext.test.ts"><storycode:paste select="#f-c0a4d45aa387-expression_statement"/><storycode:paste select="#f-c0a4d45aa387-expression_statement-2"/><storycode:paste select="#f-c0a4d45aa387-expression_statement-3"/></storycode:file>

<a id="output-1317c5ba0132"></a>
## `apps/web/src/debug/documentDebugContext.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](contract.md).

<a id="f-1317c5ba0132-documentDebugContext"></a>
### `documentDebugContext`

<storycode:copy id="f-1317c5ba0132-documentDebugContext">
import { parseHickDoc } from "../editor/hickDoc";

/** The clicked construction, never a global project-version preference. */
export function documentDebugContext(path: string, source: string, at?: number): string {
  const doc = parseHickDoc(source);
  if (at === undefined || !doc.blocks.some(b => b.name === "project")) return "new";
  const block = doc.blocks.find(b => b.name === "file" && b.from === at);
  if (!block) throw new Error("This file block changed. Press its Debug icon again.");
  // Parser spans exclude the optional BOM; editor positions include it.
  const byte = new TextEncoder().encode(source.slice(source.startsWith("\uFEFF") ? 1 : 0, at)).length;
  const id = block.attrs.id ?? `${path.split("/").pop()}:${byte}`;
  return `latest-file:${JSON.stringify([id, source])}`;
}
</storycode:copy>

### Assemble `apps/web/src/debug/documentDebugContext.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/documentDebugContext.ts"><storycode:paste select="#f-1317c5ba0132-documentDebugContext"/></storycode:file>

<a id="output-0ca80b3bce9b"></a>
## `crates/hick-lang/src/project/debug.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](contract.md).

<a id="f-0ca80b3bce9b-latest_debug_project"></a>
### `latest_debug_project`

Debug actions select the newest evaluated tree containing the displayed bytes.

<storycode:copy id="f-0ca80b3bce9b-latest_debug_project">
//! Debug actions select the newest evaluated tree containing the displayed bytes.
use super::{EntryKind, ProjectTree, project_byte_location};

/// Versions are complete project prefixes in evaluation order, oldest first.
/// Never synthesize a tree by mixing files from different versions. A rename is
/// followed only through unique byte provenance, never by searching equal text.
pub fn latest_debug_project(
    versions: &[ProjectTree],
    displayed: &ProjectTree,
    path: &str,
) -> Result<(usize, String), String> {
    let file = displayed
        .get(path)
        .filter(|entry| entry.kind == EntryKind::File)
        .ok_or_else(|| {
            format!("No source file is displayed for {path}. Open its code before debugging.")
        })?;
    for (index, tree) in versions.iter().enumerate().rev() {
        let renamed = if tree.get(path).is_none() && !file.bytes.is_empty() {
            project_byte_location(displayed, tree, path, 0)
                .ok()
                .flatten()
                .filter(|(_, offset)| *offset == 0)
                .map(|(path, _)| path)
        } else {
            None
        };
        let target = renamed.as_deref().unwrap_or(path);
        if tree
            .get(target)
            .is_some_and(|entry| entry.kind == EntryKind::File && entry.bytes == file.bytes)
        {
            return Ok((index, target.to_owned()));
        }
    }
    Err(format!(
        "No evaluated project contains the exact displayed code for {path}. Save or reload this code before debugging."
    ))
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/debug.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/debug.rs"><storycode:paste select="#f-0ca80b3bce9b-latest_debug_project"/></storycode:file>

## Test the behavior: `latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code`

This is the maintained test fragment from `crates/hick-lang/tests/project_debug.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3d2aa0516f40-latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code"></a>
### `latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code`

<storycode:copy id="f-3d2aa0516f40-latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn latest_compatible_version_preserves_later_other_files_without_replacing_displayed_code() {
    let project = fixture();
    let displayed = &project.local_states["zbase"].1;
    let (index, path) = latest_debug_project(&project.versions, displayed, "app.py").unwrap();
    assert_eq!(path, "app.py");
    let selected = &project.versions[index];
    assert_eq!(
        selected.get("app.py").unwrap().bytes.as_ref(),
        b"print(1)\n"
    );
    assert_eq!(
        selected.get("lib.py").unwrap().bytes.as_ref(),
        b"value = 2\n"
    );
    assert!(selected.get("config.json").is_none()); // No synthetic mixed-version tree.
    let (index, _) = latest_debug_project(
        &project.versions,
        &project.local_states["replacement"].1,
        "app.py",
    )
    .unwrap();
    assert_eq!(index, project.versions.len() - 1);
    assert!(project.versions[index].get("config.json").is_some());
}
</storycode:copy>

## Test the behavior: `unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes`

This is the maintained test fragment from `crates/hick-lang/tests/project_debug.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3d2aa0516f40-unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes"></a>
### `unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes`

<storycode:copy id="f-3d2aa0516f40-unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn unsaved_or_absent_code_is_refused_instead_of_debugging_different_bytes() {
    let project = fixture();
    let mut draft = project.new.clone();
    draft
        .insert(
            "app.py",
            hick_lang::ProjectEntry::file(b"unsaved".as_slice()),
        )
        .unwrap();
    assert!(
        latest_debug_project(&project.versions, &draft, "app.py")
            .unwrap_err()
            .contains("exact displayed code")
    );
    assert!(latest_debug_project(&project.versions, &ProjectTree::default(), "app.py").is_err());
}
</storycode:copy>

## Test the behavior: `byte_identical_revert_uses_latest_state_even_when_provenance_changed`

This is the maintained test fragment from `crates/hick-lang/tests/project_debug.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3d2aa0516f40-byte_identical_revert_uses_latest_state_even_when_provenance_changed"></a>
### `byte_identical_revert_uses_latest_state_even_when_provenance_changed`

<storycode:copy id="f-3d2aa0516f40-byte_identical_revert_uses_latest_state_even_when_provenance_changed">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn byte_identical_revert_uses_latest_state_even_when_provenance_changed() {
    let source = format!(
        "<hick:project/><hick:file id=\"base\" path=\"app.py\">print(1)\n</hick:file>{}{}",
        patch("change", "base", "print(1)\n", "print(2)\n"),
        patch("revert", "change", "print(2)\n", "print(1)\n")
    );
    let project = evaluate_project(
        "p.md",
        &BTreeMap::from([("p.md".into(), source)]),
        &ProjectTree::default(),
    )
    .unwrap();
    let (index, _) =
        latest_debug_project(&project.versions, &project.local_states["base"].1, "app.py").unwrap();
    assert_eq!(index, project.versions.len() - 1);
}
</storycode:copy>

## Test the behavior: `unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not`

This is the maintained test fragment from `crates/hick-lang/tests/project_debug.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3d2aa0516f40-unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not"></a>
### `unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not`

<storycode:copy id="f-3d2aa0516f40-unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn unchanged_rename_follows_provenance_but_equal_unrelated_files_do_not() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"app.py\">print(1)\n</hick:file><hick:patch id=\"rename\" after=\"base\">\n--- a/app.py\n+++ b/main.py\n@@ -1 +1 @@\n print(1)\n</hick:patch>";
    let project = evaluate_project(
        "p.md",
        &BTreeMap::from([("p.md".into(), source.into())]),
        &ProjectTree::default(),
    )
    .unwrap();
    let (index, path) =
        latest_debug_project(&project.versions, &project.local_states["base"].1, "app.py").unwrap();
    assert_eq!(index, project.versions.len() - 1);
    assert_eq!(path, "main.py");
}

</storycode:copy>

## Test the behavior: `a_mode_change_does_not_make_identical_code_older`

This is the maintained test fragment from `crates/hick-lang/tests/project_debug.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3d2aa0516f40-a_mode_change_does_not_make_identical_code_older"></a>
### `a_mode_change_does_not_make_identical_code_older`

<storycode:copy id="f-3d2aa0516f40-a_mode_change_does_not_make_identical_code_older">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn a_mode_change_does_not_make_identical_code_older() {
    let mut old = ProjectTree::default();
    old.insert(
        "app.py",
        hick_lang::ProjectEntry::file(b"print(1)\n".as_slice()),
    )
    .unwrap();
    let mut executable = old.get("app.py").unwrap().as_ref().clone();
    executable.executable = true;
    let mut new = old.clone();
    new.insert("app.py", executable).unwrap();
    assert_eq!(
        latest_debug_project(&[old.clone(), new], &old, "app.py")
            .unwrap()
            .0,
        1
    );
}

</storycode:copy>

## Test the behavior: `equal_text_in_an_unrelated_file_is_not_used_as_a_rename`

This is the maintained test fragment from `crates/hick-lang/tests/project_debug.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3d2aa0516f40-equal_text_in_an_unrelated_file_is_not_used_as_a_rename"></a>
### `equal_text_in_an_unrelated_file_is_not_used_as_a_rename`

<storycode:copy id="f-3d2aa0516f40-equal_text_in_an_unrelated_file_is_not_used_as_a_rename">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn equal_text_in_an_unrelated_file_is_not_used_as_a_rename() {
    let mut old = ProjectTree::default();
    old.insert(
        "app.py",
        hick_lang::ProjectEntry::file(b"print(1)\n".as_slice()),
    )
    .unwrap();
    let mut unrelated = ProjectTree::default();
    unrelated
        .insert(
            "other.py",
            hick_lang::ProjectEntry::file(b"print(1)\n".as_slice()),
        )
        .unwrap();
    assert_eq!(
        latest_debug_project(&[old.clone(), unrelated], &old, "app.py").unwrap(),
        (0, "app.py".into())
    );
}

</storycode:copy>

## Test the behavior: `clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines`

This is the maintained test fragment from `crates/hick-lsp/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d2279aae5a8a-clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines"></a>
### `clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines`

<storycode:copy id="f-d2279aae5a8a-clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn clicked_file_uses_exact_old_bytes_and_original_document_breakpoint_lines() {
    let source = "<hick:project/>\n<hick:file id=\"base\" path=\"app.py\">\nprint(1)\n</hick:file>\n<hick:file id=\"lib\" after=\"base\" path=\"lib.py\">value = 2\n</hick:file>\n<hick:patch id=\"replace\" after=\"lib\">\n--- a/app.py\n+++ b/app.py\n@@ -1 +1 @@\n-print(1)\n+print(2)\n</hick:patch>";
    let sources = BTreeMap::from([("p.md".into(), source.into())]);
    let selection = ProjectEndpoint::LatestFile {
        id: "base".into(),
        source: source.into(),
    };
    let state = hick_lsp::document::HickDocumentState::from_project_at(
        "p.md",
        &sources,
        &ProjectTree::default(),
        &selection,
    )
    .unwrap();
    let tree = state.selected_project.as_ref().unwrap();
    assert_eq!(tree.get("app.py").unwrap().bytes.as_ref(), b"print(1)\n");
    assert!(tree.get("lib.py").is_some());
    let map = &state.project_maps["app.py"];
    assert_eq!(map.to_virtual(2, 0), Some((0, 0)));
    let changed = BTreeMap::from([("p.md".into(), source.replace("print(1)", "print(3)"))]);
    let error = hick_lsp::document::HickDocumentState::from_project_at(
        "p.md",
        &changed,
        &ProjectTree::default(),
        &selection,
    )
    .unwrap_err();
    assert!(error.to_string().contains("document changed"));
}

</storycode:copy>

## Test the behavior: `generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context`

This is the maintained test fragment from `crates/hick-lsp/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d2279aae5a8a-generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context"></a>
### `generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context`

<storycode:copy id="f-d2279aae5a8a-generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context">
// Guarantee: docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md
#[test]
fn generated_code_selection_uses_displayed_bytes_instead_of_the_previous_file_context() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"app.py\">print(1)\n</hick:file><hick:patch id=\"change\" after=\"base\">\n--- a/app.py\n+++ b/app.py\n@@ -1 +1 @@\n-print(1)\n+print(2)\n</hick:patch>";
    let sources = BTreeMap::from([("p.md".into(), source.into())]);
    for code in ["print(1)\n", "print(2)\n"] {
        let value = format!(
            "latest-code:{}",
            serde_json::json!(["app.py", code, source])
        );
        let selection = ProjectEndpoint::parse(&value).unwrap();
        let state = hick_lsp::document::HickDocumentState::from_project_at(
            "p.md",
            &sources,
            &ProjectTree::default(),
            &selection,
        )
        .unwrap();
        assert_eq!(
            state
                .selected_project
                .unwrap()
                .get("app.py")
                .unwrap()
                .bytes
                .as_ref(),
            code.as_bytes()
        );
    }
    let selection = ProjectEndpoint::LatestCode {
        path: "app.py".into(),
        code: "not evaluated".into(),
        source: source.into(),
    };
    assert!(
        hick_lsp::document::HickDocumentState::from_project_at(
            "p.md",
            &sources,
            &ProjectTree::default(),
            &selection
        )
        .unwrap_err()
        .to_string()
        .contains("exact displayed code")
    );
}
</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/ComparisonTools.tsx](contract.md#output-455afcfce7fd)
- [apps/web/src/debug/GeneratedDebugTools.tsx](contract.md#output-16d8467a71c2)
- [apps/web/src/debug/documentDebugContext.ts](contract.md#output-1317c5ba0132)
- [apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm](../../../shared/files/apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.md)
- [apps/web/src/elements/patch/Editor.tsx](../../../mechanisms/apps/web/src/elements/patch/Editor.tsx.md#output-dc5641c063e3)
- [apps/web/src/views/documentSession.tsx](../../../mechanisms/apps/web/src/views/documentSession.tsx.md#output-bfb59577f48b)
- [crates/hick-lang/src/project/debug.rs](contract.md#output-0ca80b3bce9b)
- [crates/hick-lang/src/project/evaluate.rs](../../../mechanisms/crates/hick-lang/src/project/evaluate.rs.md#output-4957ad4b2f9a)
- [crates/hick-lsp/src/document.rs](../../../mechanisms/crates/hick-lsp/src/document.rs.md#output-7017743f2a99)
- [crates/hick-lsp/src/project.rs](../../../mechanisms/crates/hick-lsp/src/project.rs.md#output-144d2ea0027d)
- [crates/hickory-cli/src/debug_sessions.rs](../../../mechanisms/crates/hickory-cli/src/debug_sessions.rs.md#output-24a9413a429b)
- [crates/hickory-cli/src/project_documents.rs](../../../mechanisms/crates/hickory-cli/src/project_documents.rs.md#output-7cd7736e0d32)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/components/ComparisonTools.test.tsx](../../../mechanisms/apps/web/src/components/ComparisonTools.test.tsx.md#output-d1cfc2ed25eb)
- [apps/web/src/debug/documentDebugContext.test.ts](contract.md#output-c0a4d45aa387)
- [apps/web/src/debug/useDebugger.test.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-101aa3d58173)
- [crates/hick-lang/tests/project_debug.rs](../../../mechanisms/crates/hick-lang/tests/project_debug.rs.md#output-3d2aa0516f40)
- [crates/hick-lsp/tests/project.rs](../../../mechanisms/crates/hick-lsp/tests/project.rs.md#output-d2279aae5a8a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
