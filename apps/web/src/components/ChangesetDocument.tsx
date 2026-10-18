import { useEffect, useMemo } from "react";
import { LocalRealtime } from "../api/realtime";
import { DocumentEditor } from "../editor/DocumentEditor";
import { materializeProject, type ProjectEntry } from "../editor/hickLang";

const noCells = new Set<string>();
/** Accepted artifacts are immutable records. Their patches use ordinary note rendering. */
export function ChangesetDocument({ entry, sources, input, tools = true, filesystemOnly = false }: {
  entry: string; sources: Record<string,string>; input: ProjectEntry[]; tools?: boolean; filesystemOnly?: boolean;
}) {
  const realtime = useMemo(() => new LocalRealtime(), []);
  const documents = useMemo(() => [entry,...Object.keys(sources).filter(path => path !== entry)].map(document => ({
    document, project: {entry,document,sources,input,tools,filesystemOnly},
  })),[entry,sources,input,tools,filesystemOnly]);
  useEffect(() => () => realtime.close(),[realtime]);
  // Check the closure locally before exposing native tools. Never substitute disk bytes.
  const error = useMemo(() => {
    try { materializeProject(entry,sources,input); return ""; }
    catch (e) { return e instanceof Error ? e.message : String(e); }
  },[entry,sources,input]);
  if (error) return <p role="alert">Could not render final changeset: {error}</p>;
  return <section aria-label="Final literate changeset">
    {documents.map(({document,project}) => <div key={document}>
      {documents.length > 1 && <h3>{document}</h3>}
      <DocumentEditor docId={`final:${document}`} initialSource={sources[document]} realtime={realtime}
        capturedProject={project} readOnly preserveBytes execBlocks={[]}
        runningCells={noCells} onRunCell={() => {}} />
    </div>)}
  </section>;
}
