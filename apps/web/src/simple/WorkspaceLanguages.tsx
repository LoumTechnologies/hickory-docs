import { createContext, useContext, useEffect, useState, useSyncExternalStore } from "react";
import { useWorkspaceLsp, type LspSession } from "../lsp/useLsp";
import { documentUri } from "../lsp/documentUri";
import type { Buffer, Buffers } from "./buffers";
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
export function useWorkspaceOutlineReader() {
  const registry=useContext(languageContext) ?? emptyWorkspace;
  return async (buffer: Buffer,source: string) => {
    const session=await registry.entry(buffer).readyFor(source);
    if(buffer.text!==source) throw new Error("Source changed before analysis. Run again.");
    const result=await session.client!.request("textDocument/documentSymbol",{textDocument:{uri:session.uri}});
    if(buffer.text!==source || registry.entry(buffer).read().client!==session.client) throw new Error("Source or provider changed during analysis. Run again.");
    return semanticLocations(result,session.uri).filter(location=>location.uri===session.uri);
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
