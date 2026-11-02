<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Read the public implementation through the shared editor

For engineers maintaining the public source browser.

The static host supplies committed Markdown to `DocumentEmbed` in read-only mode. Reuse the product parser, formatting and scaffold folds; never simulate execution or open an engine connection. Verify a fetched page against the index path, revision and hash before rendering it. Resolve navigation against its source document, not the viewer URL. Abort superseded requests so a slow page cannot replace the newly selected chapter.

## Boundaries

The browser receives a compact index before fetching a chapter. Keep document paths and artifact URLs separate: a Markdown path is navigation identity, while only selected hash-named artifacts may be fetched. The product's existing scaffold fold extension supplies the initial collapsed view.

<storycode:copy id="public-source-reader-boundaries">
import { useEffect, useMemo, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";
import { sourceLinks } from "./sourceLinks";
import { DocumentEmbed } from "./DocumentEmbed";
import { ingestedFolds } from "../editor/folding";
import { sha256 } from "../lib/reviews";
import { UiLink, requestLinkNavigation } from "../components/UiLink";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";

interface SourceDocument { path: string; title: string; sha256: string; url: string }
interface SourceIndex { version: number; revision: string; entry: string; documents: SourceDocument[] }
const folds = [ingestedFolds()];
const safePath = (path: string) => path === "project.md" || path.startsWith(".hickory-source/") && path.endsWith(".md") && !/[\\?#%\u0000-\u001f]/.test(path) && path.split("/").every(part => part !== "" && part !== "." && part !== "..");

</storycode:copy>

## Identities

A cached index can outlive a deployment. Refuse source whose path, commit or SHA-256 differs from that index so a reader never sees one revision under another revision's label. Duplicate paths and external artifact URLs cannot become ambiguous fetch choices.

<storycode:copy id="public-source-reader-identities">
/** Refuse arbitrary fetch targets, duplicate paths and mismatched source revisions. */
export function decodeSourceIndex(value: unknown): SourceIndex {
  const index = value as SourceIndex;
  if (!index || index.version !== 1 || !/^[a-f0-9]{40}$/.test(index.revision) || index.entry !== "project.md" || !Array.isArray(index.documents)) throw Error("Invalid source index");
  const paths = new Set<string>();
  for (const doc of index.documents) {
    if (!doc || typeof doc.path !== "string" || !safePath(doc.path) || paths.has(doc.path) || typeof doc.title !== "string" || !/^[a-f0-9]{64}$/.test(doc.sha256) || !/^documents\/[a-f0-9]{64}\.json$/.test(doc.url)) throw Error("Invalid source document");
    paths.add(doc.path);
  }
  if (!paths.has(index.entry)) throw Error("Source entry is missing");
  return index;
}

export async function decodeSource(value: unknown, index: SourceIndex, document: SourceDocument): Promise<string> {
  const data = value as { path: string; revision: string; source: string };
  if (!data || data.path !== document.path || data.revision !== index.revision || typeof data.source !== "string" || await sha256(data.source) !== document.sha256) throw Error("Published source does not match its revision and hash");
  return data.source;
}

</storycode:copy>

## Navigation

Build the left files pane from the committed inventory. Render directory children only when expanded, and open the ancestors of the current document. Breadcrumbs describe the current source path; they do not add a second introduction or document heading.

<storycode:copy id="public-source-reader-navigation">
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
  return <nav className="source-breadcrumbs" aria-label="Breadcrumbs"><ol><li><UiLink href="?path=project.md" title={`Published revision ${index.revision}`}>Hickory Docs</UiLink></li>{parts.map((part, i) => {
    const folder = parts.slice(0, i + 1).join("/");
    const owner = index.documents.find(doc => doc.path === `${folder}/index.md` || doc.path === `${folder}/catalog.md`);
    return <li key={folder}>{i === parts.length - 1 ? <span aria-current="page">{part}</span> : owner ? <UiLink href={`?path=${encodeURIComponent(owner.path)}`}>{part}</UiLink> : <span>{part}</span>}</li>;
  })}</ol></nav>;
}

</storycode:copy>

## Reader

Load and authenticate the selected document, then let its own thesis, headings and links carry the tutorial. The surrounding layout contains only files and breadcrumbs. Ordinary links provide deep-link and browser-history behavior; the scoped open command uses that same navigation boundary. Keep folded scaffold output and fragment anchors usable without changing source bytes.

<storycode:copy id="public-source-reader-reader">
export function SourcePage() {
  const [index, setIndex] = useState<SourceIndex | null>(null);
  const path = new URL(location.href).searchParams.get("path") ?? "project.md";
  const [source, setSource] = useState<string | null>(null);
  const [error, setError] = useState("");
  const view = useRef<EditorView | null>(null);
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
    const document = index.documents.find(doc => doc.path === path);
    if (!document) { setError("This page is not published. Choose a document in the files pane."); return; }
    const controller = new AbortController();
    setSource(null); setError("");
    void fetch(`./source/${document.url}`, { signal: controller.signal }).then(async response => {
      if (!response.ok) throw Error("This published source page is unavailable");
      const text = await decodeSource(await response.json(), index, document);
      if (!controller.signal.aborted) setSource(text);
    }).catch(failure => { if (!controller.signal.aborted) setError(String(failure)); });
    return () => controller.abort();
  }, [index, path]);
  const folders = useMemo(() => sourceFolders(index?.documents ?? []), [index]);
  const extensions = useMemo(() => index ? [...folds, sourceLinks(path, index)] : folds, [index, path]);
  function revealAnchor(editor: EditorView) {
    let anchor: string;
    try { anchor = decodeURIComponent(location.hash.slice(1)); } catch { return; }
    if (!anchor) return;
    const text = editor.state.doc.toString();
    let at = text.indexOf(`<a id="${anchor}">`);
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
    <aside className="source-files"><nav aria-label="Source files"><SourceFiles folder={folders} current={path} /></nav></aside>
    <main className="source-document">
      {index && <SourceBreadcrumbs path={path} index={index} />}
      {error && <p role="alert">{error}</p>}
      {!error && source === null && <p role="status">Loading document…</p>}
      {source !== null && index && <DocumentEmbed key={path} path={path} source={source} revision={index.revision} readOnly extensions={extensions} onViewReady={editor => { view.current = editor; if (editor) revealAnchor(editor); }} />}
    </main>
  </div>;
}
</storycode:copy>

<storycode:file path="apps/web/src/embed/SourcePage.tsx"><storycode:paste select="#public-source-reader-boundaries"/><storycode:paste select="#public-source-reader-identities"/><storycode:paste select="#public-source-reader-navigation"/><storycode:paste select="#public-source-reader-reader"/></storycode:file>
</storycode:doc>
