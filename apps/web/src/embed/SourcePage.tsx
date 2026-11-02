import { useEffect, useMemo, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";
import { decodeStaticLanguage, staticLanguageExtensions, staticSymbols } from "./staticLanguage";
import type { StaticLanguage } from "./staticLanguage";
import type { LspLocation } from "../lsp/client";
import type { SymbolInspection } from "../lsp/symbolInspection";
import { renderHoverMarkdown } from "../lsp/hoverMarkdown";
import { artifactText } from "./staticArtifacts";
import { sourceLinks } from "./sourceLinks";
import { DocumentEmbed } from "./DocumentEmbed";
import { ingestedFolds } from "../editor/folding";
import { sha256 } from "../lib/reviews";
import { UiLink, requestLinkNavigation } from "../components/UiLink";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";

interface SourceDocument { path: string; title: string; sha256: string; url: string; intelligence_sha256?: string }
interface SourceModel { language: string; sha256: string; url: string }
interface SourceIndex { version: number; revision: string; entry: string; documents: SourceDocument[]; models?: SourceModel[]; source_url?: string }
const folds = [ingestedFolds()];
const safePath = (path: string) => path.endsWith(".md") && !path.startsWith("/") && !/[\\:?#%\u0000-\u001f]/.test(path) && path.split("/").every(part => part !== "" && part !== "." && part !== "..");

/** Refuse arbitrary fetch targets, duplicate paths and mismatched source revisions. */
export function decodeSourceIndex(value: unknown): SourceIndex {
  const index = value as SourceIndex;
  if (!index || index.version !== 1 || !/^[a-f0-9]{40}$/.test(index.revision) || typeof index.entry !== "string" || !safePath(index.entry) || !Array.isArray(index.documents)) throw Error("Invalid source index");
  if (index.source_url !== undefined) {
    const url = new URL(index.source_url);
    if (!["http:", "https:"].includes(url.protocol) || url.username || url.password || url.search || url.hash) throw Error("Invalid public source URL");
  }
  const paths = new Set<string>();
  for (const doc of index.documents) {
    if (!doc || typeof doc.path !== "string" || !safePath(doc.path) || paths.has(doc.path) || typeof doc.title !== "string" || !/^[a-f0-9]{64}$/.test(doc.sha256) || !/^documents\/[a-f0-9]{64}\.json(?:\.gz)?$/.test(doc.url)) throw Error("Invalid source document");
    if (doc.intelligence_sha256 !== undefined && !/^[a-f0-9]{64}$/.test(doc.intelligence_sha256)) throw Error("Invalid language artifact hash");
    paths.add(doc.path);
  }
  if (index.models !== undefined && (!Array.isArray(index.models) || index.models.some(model => !model || !/^[a-z][a-z0-9_-]*$/.test(model.language) || !/^[a-f0-9]{64}$/.test(model.sha256) || ![`models/${model.sha256}.json`, `models/${model.sha256}.json.gz`].includes(model.url)))) throw Error("Invalid code model artifact");
  if (!paths.has(index.entry)) throw Error("Source entry is missing");
  return index;
}

export async function decodeSource(value: unknown, index: SourceIndex, document: SourceDocument): Promise<string> {
  const data = value as { path: string; revision: string; source: string };
  if (!data || data.path !== document.path || data.revision !== index.revision || typeof data.source !== "string" || await sha256(data.source) !== document.sha256) throw Error("Published source does not match its revision and hash");
  return data.source;
}

interface SourceFolder { name: string; path: string; folders: SourceFolder[]; files: SourceDocument[] }

function sourceFolders(documents: SourceDocument[]): SourceFolder {
  const root: SourceFolder = { name: "", path: "", folders: [], files: [] };
  for (const doc of documents) {
    const parts = doc.path.split("/"); parts.pop();
    let folder = root;
    for (const name of parts) {
      const path = folder.path ? `${folder.path}/${name}` : name;
      let child = folder.folders.find(item => item.path === path);
      if (!child) { child = { name, path, folders: [], files: [] }; folder.folders.push(child); }
      folder = child;
    }
    folder.files.push(doc);
  }
  return root;
}

function SourceFiles({ folder, current }: { folder: SourceFolder; current: string }) {
  return <ul>{folder.files.map(doc => <li key={doc.path}><UiLink href={`?path=${encodeURIComponent(doc.path)}`} aria-current={current === doc.path ? "page" : undefined} title={doc.title}>{doc.path.split("/").at(-1)!}</UiLink></li>)}
    {folder.folders.map(child => <SourceDirectory key={child.path} folder={child} current={current} />)}</ul>;
}

function SourceDirectory({ folder, current }: { folder: SourceFolder; current: string }) {
  const selected = current.startsWith(`${folder.path}/`);
  const [open, setOpen] = useState(selected || folder.path === ".hickory-source");
  useEffect(() => { if (selected) setOpen(true); }, [current, selected]);
  return <li><details open={open} onToggle={event => setOpen(event.currentTarget.open)}><summary>{folder.name}</summary>{open && <SourceFiles folder={folder} current={current} />}</details></li>;
}

function SourceBreadcrumbs({ path, index }: { path: string; index: SourceIndex }) {
  const parts = path.split("/");
  return <nav className="source-breadcrumbs" aria-label="Breadcrumbs"><ol><li><UiLink href={`?path=${encodeURIComponent(index.entry)}`} title={`Published revision ${index.revision}`}>{index.documents.find(doc => doc.path === index.entry)?.title ?? index.entry}</UiLink></li>{parts.map((part, i) => {
    const folder = parts.slice(0, i + 1).join("/");
    const owner = index.documents.find(doc => doc.path === `${folder}/index.md` || doc.path === `${folder}/catalog.md`);
    return <li key={folder}>{i === parts.length - 1 ? <span aria-current="page">{part}</span> : owner ? <UiLink href={`?path=${encodeURIComponent(owner.path)}`}>{part}</UiLink> : <span>{part}</span>}</li>;
  })}</ol></nav>;
}

function CapturedModels({ index }: { index: SourceIndex }) {
  const [open, setOpen] = useState(false);
  const [models, setModels] = useState<unknown[] | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    if (!open) return;
    const controller = new AbortController();
    setModels(null); setError("");
    void Promise.all((index.models ?? []).map(async model => {
      const response = await fetch(`./source/${model.url}`, { signal: controller.signal });
      if (!response.ok) throw Error("Captured code model is unavailable");
      const text = await artifactText(response);
      if (await sha256(text) !== model.sha256) throw Error("Captured code model hash differs");
      const value = JSON.parse(text);
      if (value.revision !== index.revision || value.language !== model.language) throw Error("Captured code model revision differs");
      return value;
    })).then(values => { if (!controller.signal.aborted) setModels(values); })
      .catch(failure => { if (!controller.signal.aborted) setError(String(failure)); });
    return () => controller.abort();
  }, [open, index]);
  return <details open={open} onToggle={event => setOpen(event.currentTarget.open)}>
    <summary>Captured code models</summary>
    <p>Schema and finite type graph at this published revision. Unresolved bindings remain in the capture. New parameterized queries require a new build.</p>
    {error ? <p role="alert">{error}</p> : models ? models.map((model, i) => <pre key={i}>{JSON.stringify(model, null, 2)}</pre>) : open && <p role="status">Loading captured code models…</p>}
  </details>;
}

function InspectionDocumentation({ text }: { text: string }) {
  const content = useRef<HTMLDivElement>(null);
  useEffect(() => { content.current?.replaceChildren(renderHoverMarkdown(text)); }, [text]);
  return <div ref={content} className="symbol-documentation" />;
}

export function SourcePage() {
  const [index, setIndex] = useState<SourceIndex | null>(null);
  const path = new URL(location.href).searchParams.get("path") ?? index?.entry ?? "project.md";
  const [source, setSource] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [intelligence, setIntelligence] = useState<StaticLanguage | null>(null);
  const [references, setReferences] = useState<LspLocation[]>([]);
  const [inspection, setInspection] = useState<SymbolInspection | null>(null);
  const [filesOpen, setFilesOpen] = useState(false);
  const inspector = useRef<HTMLElement>(null);
  useEffect(() => { if (inspection) inspector.current?.focus({ preventScroll: true }); }, [inspection]);
  const view = useRef<EditorView | null>(null);
  useEffect(() => {
    const close = (event: KeyboardEvent) => { if (event.key === "Escape") { if (inspector.current?.contains(document.activeElement)) view.current?.focus(); setInspection(null); setFilesOpen(false); } };
    addEventListener("keydown", close); return () => removeEventListener("keydown", close);
  }, []);
  useEffect(() => {
    const controller = new AbortController();
    void fetch("./source/index.json", { signal: controller.signal }).then(async response => {
      if (!response.ok) throw Error("The published source index is unavailable");
      const next = decodeSourceIndex(await response.json());
      if (!controller.signal.aborted) setIndex(next);
    }).catch(failure => { if (!controller.signal.aborted) setError(String(failure)); });
    return () => controller.abort();
  }, []);
  useEffect(() => {
    if (!index) return;
    setSource(null); setIntelligence(null); setReferences([]); setInspection(null); setFilesOpen(false);
    const document = index.documents.find(doc => doc.path === path);
    if (!document) { setError("This page is not published. Choose a document in the files pane."); return; }
    const controller = new AbortController();
    setError("");
    void fetch(`./source/${document.url}`, { signal: controller.signal }).then(async response => {
      if (!response.ok) throw Error("This published source page is unavailable");
      const payload = JSON.parse(await artifactText(response));
      const text = await decodeSource(payload, index, document);
      const analysis = await decodeStaticLanguage(payload.intelligence, text, document.intelligence_sha256);
      if (!controller.signal.aborted) { setIntelligence(analysis); setSource(text); }
    }).catch(failure => { if (!controller.signal.aborted) setError(String(failure)); });
    return () => controller.abort();
  }, [index, path]);
  const folders = useMemo(() => sourceFolders(index?.documents ?? []), [index]);
  function targetHref(target: LspLocation) {
    if (!target.uri.startsWith("hick:///")) return null;
    let targetPath: string;
    try { targetPath = decodeURIComponent(target.uri.slice(8).split(/[?#]/)[0]); } catch { return null; }
    if (!index?.documents.some(doc => doc.path === targetPath)) return null;
    const { line, character } = target.range.start;
    if (!Number.isInteger(line) || line < 0 || !Number.isInteger(character) || character < 0) return null;
    return `?path=${encodeURIComponent(targetPath)}#L${line + 1}:${character}`;
  }
  const extensions = useMemo(() => index ? [...folds, sourceLinks(path, index),
    ...(intelligence && source !== null ? staticLanguageExtensions(intelligence, source, path,
      target => { const href = targetHref(target); if (href) void requestLinkNavigation(href); }, setReferences, setInspection) : []),
  ] : folds, [index, path, intelligence, source]);
  function revealAnchor(editor: EditorView) {
    let anchor: string;
    try { anchor = decodeURIComponent(location.hash.slice(1)); } catch { return; }
    if (!anchor) return;
    const text = editor.state.doc.toString();
    const position = /^L([1-9][0-9]*):([0-9]+)$/.exec(anchor);
    let at = -1;
    if (position && Number(position[1]) <= editor.state.doc.lines) {
      const line = editor.state.doc.line(Number(position[1]));
      at = Math.min(line.to, line.from + Number(position[2]));
      editor.dispatch({ selection: { anchor: at } });
    }
    if (at < 0) at = text.indexOf(`<a id="${anchor}">`);
    if (at < 0) {
      const heading = [...text.matchAll(/^#{1,6}\s+(.+)$/gm)].find(match => match[1].toLowerCase().replace(/[^\p{L}\p{N}\s-]/gu, "").trim().replace(/\s+/g, "-") === anchor);
      at = heading?.index ?? -1;
    }
    if (at >= 0) requestAnimationFrame(() => { if (view.current === editor) editor.dispatch({ effects: EditorView.scrollIntoView(at, { y: "start" }) }); });
  }
  useEffect(() => {
    const reveal = () => { if (view.current) revealAnchor(view.current); };
    addEventListener("hashchange", reveal); return () => removeEventListener("hashchange", reveal);
  }, []);
  const stateRevision = useUiRevision(() => [index?.revision, path, source !== null, error]);
  useUiReading("public source document", { revision: index?.revision, path, loaded: source !== null, error });
  useUiCommands([{ key: "source-browser:open", command: { label: "Open a published source document", parent: "source-browser", subject: "source-browser", parameters: { path: { type: "string", maxLength: 4096 } }, prerequisites: ["The path must be in this published source index."], read: () => ({ revision: stateRevision(), enabled: index !== null }), execute: async args => {
    if (!index?.documents.some(doc => doc.path === args.path)) return { status: "unavailable", reason: "Source document is not published" };
    return requestLinkNavigation(`?path=${encodeURIComponent(args.path as string)}`);
  } } }]);
  return <div className="source-browser" data-revision={index?.revision}>
    <button className="source-files-toggle" aria-expanded={filesOpen} aria-controls="source-files" onClick={() => setFilesOpen(open => !open)}>Files</button>
    <aside id="source-files" className={`source-files${filesOpen ? " is-open" : ""}`}><nav aria-label="Source files"><SourceFiles folder={folders} current={path} /></nav></aside>
    <main className={`source-document${inspection ? " has-inspector" : ""}`}>
      {index && <SourceBreadcrumbs path={path} index={index} />}
      {index?.models?.length ? <CapturedModels index={index} /> : null}
      {intelligence && <details><summary>Published language information</summary>
        <p>Tap a code symbol to inspect it. With a keyboard, place the cursor on a symbol and press Alt+Enter. Ctrl/Cmd-click opens its definition; add Shift for references.</p>
        <p>{intelligence.state === "captured" ? "Language answers captured at this revision. Missing provider answers are unavailable." : (intelligence.reason ?? "Syntax highlighting compiled; semantic analysis is unavailable for this page.")}</p>
        <ul>{staticSymbols(intelligence).map((symbol, i) => {
          const target = symbol.location ?? { uri: `hick:///${path}`, range: symbol.selectionRange ?? symbol.range };
          const href = targetHref(target);
          return <li key={i}>{href ? <UiLink href={href}>{symbol.name}</UiLink> : symbol.name}</li>;
        })}</ul>
      </details>}
      {references.length > 0 && <aside aria-label="Captured references"><ul>{references.map((target, i) => {
        const href = targetHref(target);
        return <li key={i}>{href ? <UiLink href={href}>{target.uri.slice(8)}:{target.range.start.line + 1}</UiLink> : <span>Target outside published source</span>}</li>;
      })}</ul></aside>}
      {inspection && <aside ref={inspector} tabIndex={-1} className="source-inspector" aria-label="Symbol inspector">
        <header><strong>{inspection.label}</strong><button onClick={() => { setInspection(null); view.current?.focus(); }} aria-label="Close symbol inspector">Close</button></header>
        {inspection.documentation ? <InspectionDocumentation text={inspection.documentation} /> : <p>No documentation available for this symbol.</p>}
        {inspection.unavailable.length > 0 && <p>{inspection.unavailable.join(", ")} unavailable at this revision.</p>}
        <details><summary>Definitions ({inspection.definitions.length})</summary><ul>{inspection.definitions.map((target, i) => {
          const href = targetHref(target);
          return <li key={i}>{href ? <UiLink href={href} onClick={() => setInspection(null)}>{target.uri.slice(8)}:{target.range.start.line + 1}</UiLink> : "Target outside published source"}</li>;
        })}</ul></details>
        <details><summary>References ({inspection.references.length})</summary><ul>{inspection.references.map((target, i) => {
          const href = targetHref(target);
          return <li key={i}>{href ? <UiLink href={href} onClick={() => setInspection(null)}>{target.uri.slice(8)}:{target.range.start.line + 1}</UiLink> : "Target outside published source"}</li>;
        })}</ul></details>
      </aside>}
      {error && <p role="alert">{error}</p>}
      {!error && source === null && <p role="status">Loading document…</p>}
      {source !== null && index && <DocumentEmbed key={path} path={path} source={source} revision={index.revision} readOnly extensions={extensions} onViewReady={editor => { view.current = editor; if (editor) revealAnchor(editor); }} />}
    </main>
  </div>;
}
