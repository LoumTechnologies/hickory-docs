<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep language knowledge with workspace buffers

For maintainers changing workspace services independently of the editor renderer.

The workspace retains language sessions when panes hide. A small per-buffer subscription publishes provider state; a workspace subscription indexes all reported messages. Text changes retire positional results immediately, independently of provider response time. Editors and Problems borrow this state instead of owning transport lifetimes. Standalone conversation/document surfaces retain their existing session behavior.

Cache each buffer's problem projection by its available text and diagnostic snapshot. A report arriving for one source must not re-split all other open sources. Digest completion publishes a fresh diagnostic array, so newly verified per-report evidence invalidates exactly that projection.

Before reading an outline, the engine must acknowledge the exact UTF-8 snapshot. Session readiness alone can precede its debounced source synchronization. The acknowledgement rejects stale revisions and source effects; a plain source need not parse as a literate document. This preparation is independent of editor selection.

<ui:copy id="simple-workspace-language-owners">
import { createContext, useContext, useEffect, useState, useSyncExternalStore } from "react";
import { useWorkspaceLsp, type LspSession } from "../lsp/useLsp";
import { documentUri } from "../lsp/documentUri";
import type { Buffer, Buffers } from "./buffers";
import {sourceDigest} from "../lsp/diagnosticRevision";
import { diagnosticBoundSource } from "../lsp/diagnosticRevision";
import { reportedProblems, semanticLocations } from "./Intelligence";
// Buffer lifetime owns language knowledge; a pane only borrows its session.
class LanguageEntry {
  constructor(private changed: () => void = () => {}) {}
  value: LspSession = { client: null, uri: "", ready: false, diagnostics: [] };
  private problemCache?: { source: string; diagnostics: LspSession["diagnostics"]; diagnosticSource?: string; reports: ReturnType<typeof reportedProblems> };
  problems(buffer: Buffer) {
    const session = this.value, cached = this.problemCache;
    if (cached && cached.source === buffer.text && cached.diagnostics === session.diagnostics && cached.diagnosticSource === session.diagnosticSource) return cached.reports;
    const reports = reportedProblems([{ uri: session.uri || documentUri(buffer.file.path), currentSource: buffer.text, source: session.diagnosticSource,
      diagnostics: session.diagnostics.map(diagnostic => ({ ...diagnostic, sourceSnapshot: diagnosticBoundSource(diagnostic) })) }]);
    this.problemCache = { source: buffer.text, diagnostics: session.diagnostics, diagnosticSource: session.diagnosticSource, reports };
    return reports;
  }
  private source = "";
  readyFor(source: string): Promise<LspSession> {
    return new Promise((resolve,reject) => {
      const check=()=>{if(this.value.ready && this.value.client && this.source===source) {clearTimeout(timeout);unsubscribe();resolve(this.value);}};
      const timeout=setTimeout(()=>{unsubscribe();reject(new Error("Language session unavailable. Retry analysis."));},12000);
      const unsubscribe=this.subscribe(check);check();
    });
  }
  private listeners = new Set<() => void>();
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => this.listeners.delete(listener); };
  read = () => this.value;
  publish(value: LspSession, source = "") { this.source = source; this.value = value; for (const listener of this.listeners) listener(); this.changed(); }
}
class WorkspaceLanguages {
  private version = 0;
  private listeners = new Set<() => void>();
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => this.listeners.delete(listener); };
  snapshot = () => this.version;
  private changed = () => { this.version++; for (const listener of this.listeners) listener(); };
  private entries = new WeakMap<Buffer, LanguageEntry>();
  entry(buffer: Buffer) { let entry = this.entries.get(buffer); if (!entry) { entry = new LanguageEntry(this.changed); this.entries.set(buffer, entry); } return entry; }
}
const languageContext = createContext<WorkspaceLanguages | null>(null);
const emptyLanguageEntry = new LanguageEntry();
function LanguageOwner({ buffer, registry }: { buffer: Buffer; registry: WorkspaceLanguages }) {
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const session = useWorkspaceLsp(buffer.transient ? "" : buffer.file.path, buffer.text);
  useEffect(() => {
    registry.entry(buffer).publish(session,buffer.text);
  }, [buffer, registry, session.client, session.uri, session.ready, session.diagnostics, session.diagnosticSource, buffer.text]);
  useEffect(() => () => registry.entry(buffer).publish({ client: null, uri: "", ready: false, diagnostics: [] }), [buffer, registry]);
  return null;
}
export function SimpleLanguages({ buffers, children }: { buffers: Buffers; children: React.ReactNode }) {
  const [registry] = useState(() => new WorkspaceLanguages());
  return <languageContext.Provider value={registry}>
    {[...buffers.values.values()].map(buffer => <LanguageOwner key={buffer.file.path} buffer={buffer} registry={registry} />)}
    {children}
  </languageContext.Provider>;
}
export function useSimpleLanguage(buffer: Buffer): LspSession {
  const registry = useContext(languageContext);
  // Standalone document surfaces (including conversation drafts) keep their
  // existing session. Workspace panes borrow the retained owner instead.
  const standalone = useWorkspaceLsp(registry || buffer.transient ? "" : buffer.file.path, buffer.text);
  const entry = registry?.entry(buffer) ?? emptyLanguageEntry;
  const shared = useSyncExternalStore(entry.subscribe, entry.read);
  return registry ? shared : standalone;
}
export function useWorkspaceLanguageLookup() {
  const registry = useContext(languageContext) ?? emptyWorkspace;
  useSyncExternalStore(registry.subscribe, registry.snapshot);
  return (buffer: Buffer) => registry.entry(buffer).read();
}
export function validateSourcePreparation(ack:unknown,digest:string,requireParsed:boolean):void {
  const value=ack as Record<string,unknown>|null;
  if(!value||value.processed!==true||value.current!==true||value.source_files_modified!==false||value.source_hash!==digest||typeof value.source_revision!=="string"||!(/^[a-f0-9]{64}$/).test(value.source_revision)||(requireParsed&&value.parsed!==true))throw new Error("Provider did not confirm this exact source snapshot. Retry analysis.");
}
export function useWorkspaceSourcePreparation() {
  const registry=useContext(languageContext) ?? emptyWorkspace;
  return async (buffer: Buffer,source: string) => {
    const session=await registry.entry(buffer).readyFor(source);
    const path=buffer.file.path,namespace=buffer.namespace;
    if(buffer.text!==source||session.uri!==documentUri(path)) throw new Error("Source address changed before analysis. Run again.");
    const revision=buffer.sourceRevision,client=session.client!,uri=session.uri;
    const current=()=>buffer.text===source&&buffer.file.path===path&&buffer.namespace===namespace&&buffer.sourceRevision===revision&&registry.entry(buffer).read().client===client&&registry.entry(buffer).read().uri===uri;
    const digest=await sourceDigest(source);if(!current())throw new Error("Source or provider changed during preparation. Run again.");
    const ack=await client.request("hick/captureDocument",{textDocument:{uri,languageId:"hick",version:1,text:source}}) as Record<string,unknown>;
    if(!current())throw new Error("Source or provider changed during preparation. Run again.");
    validateSourcePreparation(ack,digest,!!buffer.file.doc_id);
    return {client,uri,current,expected:{source_hash:digest,source_revision:ack.source_revision as string}};
  };
}
export function useWorkspaceOutlineReader() {
  const prepare=useWorkspaceSourcePreparation();
  return async(buffer:Buffer,source:string)=>{
    const {client,uri,current}=await prepare(buffer,source);
    const result=await client.request("textDocument/documentSymbol",{textDocument:{uri}});
    if(!current()) throw new Error("Source or provider changed during analysis. Run again.");
    return semanticLocations(result,uri).filter(location=>location.uri===uri);
  };
}
export function useWorkspaceAnalysisRunner(model: import("./WorkspaceAnalysis").WorkspaceAnalysis,buffers: Buffers,files: import("../api/types").FileNode[]) {
  const read=useWorkspaceOutlineReader();
  return ()=>model.run(files,file=>buffers.open(file),async(file,source)=>{const buffer=buffers.values.get(file.path);if(!buffer)throw new Error("Source no longer retained. Retry analysis.");return read(buffer,source);});
}
export function useHasWorkspaceLanguages() { return useContext(languageContext) !== null; }
const emptyWorkspace = new WorkspaceLanguages();
export function useWorkspaceProblems(buffers: Buffers) {
  const registry = useContext(languageContext) ?? emptyWorkspace;
  useSyncExternalStore(registry.subscribe, registry.snapshot);
  return [...buffers.values.values()].flatMap(buffer => registry.entry(buffer).problems(buffer));
}
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceLanguages.tsx"><ui:paste select="#simple-workspace-language-owners"/></ui:file>
## Refuse an acknowledgement that does not bind this source

The wire receipt must contain typed booleans and an opaque revision, not merely
truthy fields. Plain text may report no parsed document; literate source requires
its parsed graph. These assertions establish receipt admission, while the live
source-history and provider flows establish independent editor integration.

<ui:copy id="source-preparation-browser-witness">
import {expect,it} from 'vitest';
import {validateSourcePreparation} from './WorkspaceLanguages';
const hash='sha256:'+'a'.repeat(64);
const ack={processed:true,current:true,source_files_modified:false,source_hash:hash,source_revision:'b'.repeat(64),parsed:false};
it('admits confirmed plain text and requires a parsed literate graph',()=>{expect(()=>validateSourcePreparation(ack,hash,false)).not.toThrow();expect(()=>validateSourcePreparation(ack,hash,true)).toThrow('exact source');expect(()=>validateSourcePreparation({...ack,parsed:true},hash,true)).not.toThrow();});
it('refuses absent, forged, stale and effectful preparation receipts',()=>{for(const value of [null,{},...['processed','current'].flatMap(key=>[false,1,'true'].map(value=>({...ack,[key]:value}))),...['source_files_modified'].flatMap(key=>[true,0,'false'].map(value=>({...ack,[key]:value}))),{...ack,source_hash:'sha256:'+'c'.repeat(64)},{...ack,source_revision:''},{...ack,source_revision:'B'.repeat(64)}])expect(()=>validateSourcePreparation(value,hash,false)).toThrow('exact source');});
</ui:copy>
<ui:file path="apps/web/src/simple/SourcePreparation.test.ts"><ui:paste select="#source-preparation-browser-witness"/></ui:file>

</ui:doc>
