import { useEffect, useMemo, useState } from "react";
import { DocumentEmbed } from "./DocumentEmbed";
import { ingestedFolds } from "../editor/folding";
import { sha256 } from "../lib/reviews";
import { UiLink } from "../components/UiLink";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";

interface SourceDocument { path: string; title: string; sha256: string; url: string }
interface SourceIndex { version: number; revision: string; entry: string; documents: SourceDocument[] }
const folds = [ingestedFolds()];
const safePath = (path: string) => path === "project.md" || path.startsWith(".hickory-source/") && path.endsWith(".md") && !/[\\?#%\u0000-\u001f]/.test(path) && path.split("/").every(part => part !== "" && part !== "." && part !== "..");

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

/** Resolve source links against the document, never against this viewer's URL. */
export function relatedSources(source: string, path: string, index: SourceIndex): SourceDocument[] {
  const targets = new Set<string>();
  const base = new URL(path, "https://source.invalid/");
  const links = [...source.matchAll(/\[[^\]\n]*\]\(([^)\s]+)\)/g)].map(m => m[1]);
  links.push(...[...source.matchAll(/<[^\s:>]+:include\s+file="([^"]+)"/g)].map(m => m[1]));
  for (const target of links) {
    try {
      const url = new URL(target, base);
      if (url.origin === base.origin) targets.add(decodeURIComponent(url.pathname.slice(1)));
    } catch { /* A malformed prose link is not a source navigation target. */ }
  }
  return index.documents.filter(doc => doc.path !== path && targets.has(doc.path));
}

export function SourcePage() {
  const [index, setIndex] = useState<SourceIndex | null>(null);
  const [path, setPath] = useState(() => new URL(location.href).searchParams.get("path") ?? "project.md");
  const [source, setSource] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
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
    const back = () => { setSource(null); setError(""); setPath(new URL(location.href).searchParams.get("path") ?? "project.md"); };
    addEventListener("popstate", back); return () => removeEventListener("popstate", back);
  }, []);
  useEffect(() => {
    if (!index) return;
    const document = index.documents.find(doc => doc.path === path);
    if (!document) { setError("This page is not in the published source. Start at project.md or choose a source below."); return; }
    const controller = new AbortController();
    setSource(null); setError("");
    void fetch(`./source/${document.url}`, { signal: controller.signal }).then(async response => {
      if (!response.ok) throw Error("This published source page is unavailable");
      const text = await decodeSource(await response.json(), index, document);
      if (!controller.signal.aborted) setSource(text);
    }).catch(failure => { if (!controller.signal.aborted) setError(String(failure)); });
    return () => controller.abort();
  }, [index, path]);
  const related = useMemo(() => source !== null && index ? relatedSources(source, path, index) : [], [source, path, index]);
  const choices = index?.documents.filter(doc => `${doc.title} ${doc.path}`.toLowerCase().includes(query.toLowerCase())) ?? [];
  function navigate(next: string) {
    if (next === path) { window.scrollTo(0, 0); return; }
    const url = new URL(location.href); url.searchParams.set("path", next);
    history.pushState(null, "", url); setSource(null); setError(""); setPath(next); window.scrollTo(0, 0);
  }
  const stateRevision = useUiRevision(() => [index?.revision, path, source !== null, query, error]);
  useUiReading("public source browser", { revision: index?.revision, path, loaded: source !== null, error, filter: query });
  useUiCommands([
    { key: "source-browser:open", command: { label: "Open a published source page", parent: "source-browser", subject: "source-browser", parameters: { path: { type: "string", maxLength: 4096 } }, prerequisites: ["The path must be in this published source index."], read: () => ({ revision: stateRevision(), enabled: index !== null }), execute: async args => {
      if (!index?.documents.some(doc => doc.path === args.path)) return { status: "unavailable", reason: "Source page is not published" };
      navigate(args.path as string); return { status: "completed", value: { requested_path: args.path, revision: index.revision } };
    } } },
    { key: "source-browser:filter", command: { label: "Filter published source pages", parent: "source-browser", subject: "source-browser", parameters: { query: { type: "string", maxLength: 4096 } }, prerequisites: [], read: () => ({ revision: stateRevision(), enabled: index !== null }), execute: async args => { setQuery(args.query as string); return { status: "completed", value: { filter_requested: args.query } }; } } },
  ]);
  const link = (doc: SourceDocument) => <UiLink key={doc.path} href={`?path=${encodeURIComponent(doc.path)}`} onClick={event => { event.preventDefault(); navigate(doc.path); }}>{doc.title}</UiLink>;
  return <main className="source-browser">
    <header><UiLink href="./">Hickory Docs</UiLink><h1>Read the code that builds Hickory</h1>
      <p>Start with the workspace, then follow the implementation, tests and requirements. This is a read-only view of the maintained Markdown.</p>
      {index && <p>Published revision <UiLink href={`https://github.com/LoumTechnologies/hickory-docs/tree/${index.revision}`}>{index.revision.slice(0, 8)}</UiLink> · {index.documents.length} source pages</p>}
      <nav aria-label="Source navigation"><UiLink href="?path=project.md" onClick={event => { event.preventDefault(); navigate("project.md"); }}>Start at project.md</UiLink></nav>
      <details><summary>Find a source page</summary><label>Filter source pages <input value={query} onChange={event => setQuery(event.target.value)} /></label><label>Open source <select value={choices.some(doc => doc.path === path) ? path : ""} onChange={event => navigate(event.target.value)}><option value="" disabled>Choose a source page</option>{choices.map(doc => <option key={doc.path} value={doc.path}>{doc.title} · {doc.path}</option>)}</select></label></details>
    </header>
    {error && <p role="alert">{error}</p>}
    {!error && source === null && <p role="status">Loading source…</p>}
    {source !== null && index && <><h2>{path}</h2>{related.length > 0 && <nav aria-label="Related source pages"><details><summary>Continue reading · {related.length} related source pages</summary><ul>{related.map(doc => <li key={doc.path}>{link(doc)}</li>)}</ul></details></nav>}<DocumentEmbed key={path} path={path} source={source} revision={index.revision} readOnly extensions={folds} /></>}
  </main>;
}
