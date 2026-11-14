import { createContext, useContext, useEffect, useState, useSyncExternalStore } from "react";
import { useWorkspaceLsp, type LspSession } from "../lsp/useLsp";
import { documentUri } from "../lsp/documentUri";
import type { Buffer, Buffers } from "./buffers";
import { reportedProblems } from "./Intelligence";
// Buffer lifetime owns language knowledge; a pane only borrows its session.
class LanguageEntry {
  constructor(private changed: () => void = () => {}) {}
  value: LspSession = { client: null, uri: "", ready: false, diagnostics: [] };
  private listeners = new Set<() => void>();
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => this.listeners.delete(listener); };
  read = () => this.value;
  publish(value: LspSession) { this.value = value; for (const listener of this.listeners) listener(); this.changed(); }
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
    registry.entry(buffer).publish(session);
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
export function useHasWorkspaceLanguages() { return useContext(languageContext) !== null; }
const emptyWorkspace = new WorkspaceLanguages();
export function useWorkspaceProblems(buffers: Buffers) {
  const registry = useContext(languageContext) ?? emptyWorkspace;
  useSyncExternalStore(registry.subscribe, registry.snapshot);
  return reportedProblems([...buffers.values.values()].map(buffer => {
    const session = registry.entry(buffer).read();
    return { uri: session.uri || documentUri(buffer.file.path), currentSource: buffer.text, source: session.diagnosticSource, diagnostics: session.diagnostics };
  }));
}
