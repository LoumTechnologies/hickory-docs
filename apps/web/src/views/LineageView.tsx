// The lineage browser over a whole project: one column per stage.
//
// This is the view the two-pane editor could not be. A document explains
// itself, but "why does this say what it says?" is usually answered two
// documents upstream, and the answer has to be reachable without losing the
// place you were reading. See src/lineage/LineageColumns.tsx.
//
// Everything drawn here is computed: the links come from the provenance the
// local server returns for each generated file, so a line between two blocks
// means hick actually carried those bytes. Nothing is asserted, because the
// language has no way to declare an influence yet — when it does, those
// arrive as a different kind and are drawn differently.

import { useEffect, useMemo, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { api } from "../api/client";
import { boundedLineageRead, readLineageStage } from "./readLineageStage";
import { LineageColumns } from "../lineage/LineageColumns";
import { TimeSlider } from "../lineage/TimeSlider";
import { buildModel, LINEAGE_MODEL_TEXT_LIMIT, type SourceDocument } from "../lineage/build";
import { withStructure } from "../lineage/structure";
import type {
  ReplayCommit,
  StructureResponse,
} from "../api/types";

/** The union of every shown document's history, newest first.
 *
 * A project-wide picture wants a project-wide slider, and a commit that
 * touched any of these documents is a stop worth having: it is a moment when
 * this picture could have looked different. */
function mergedHistory(histories: ReplayCommit[][]): ReplayCommit[] {
  const bySha = new Map<string, ReplayCommit>();
  for (const commits of histories) {
    for (const commit of commits) {
      if (!bySha.has(commit.sha)) bySha.set(commit.sha, commit);
    }
  }
  return [...bySha.values()].sort((a, b) => b.time - a.time);
}

export function LineageView({ projectId }: { projectId: string }) {
  return <ProjectLineage key={projectId} projectId={projectId} />;
}

function ProjectLineage({ projectId }: { projectId: string }) {
  const [retry, setRetry] = useState(0);
  const [docs, setDocs] = useState<SourceDocument[] | null>(null);
  const [structure, setStructure] = useState<StructureResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [partial, setPartial] = useState<string[]>([]);
  const [progress, setProgress] = useState<{ completed: number; total: number; path: string } | null>(null);
  // Replay: which commit this picture is being shown at, `null` for the
  // working tree. See src/lineage/TimeSlider.tsx.
  const [history, setHistory] = useState<ReplayCommit[]>([]);
  const [at, setAt] = useState<string | null>(null);
  const [boundary, setBoundary] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setDocs(null);
    setProgress(null);
    setStructure(null);
    setError(null);
    setPartial([]);
    setBoundary(null);
    setBusy(at !== null);

    (async () => {
      try {
        const summaries = await boundedLineageRead(api.projectDocs(projectId));
        const loaded: SourceDocument[] = [];
        const skipped: string[] = [];

        for (const [index, summary] of summaries.entries()) {
          if (cancelled) return;
          setProgress({ completed: index, total: summaries.length, path: summary.path });
          const stage = await readLineageStage(summary, at, api, () => !cancelled);
          if (cancelled) return;
          if (stage.document) loaded.push(stage.document);
          if (stage.error) skipped.push(`${summary.path}: ${stage.error}`);
          if (stage.boundary) setBoundary(stage.boundary);
          // Publish only completed/explicitly unavailable stages. A slow output
          // must not hide provenance already read from the other documents.
          setDocs([...loaded]);
          setPartial([...skipped]);
        }

        if (cancelled) return;
        setDocs(loaded);
        setPartial(skipped);
        setBusy(false);
        setProgress(null);

        // The stops the slider can take: every commit that touched any of
        // these documents. Fetched once per project, not per replay.
        try {
          const histories = await Promise.all(
            summaries.map((s) =>
              boundedLineageRead(api.docHistory(s.id)).then(
                (h) => h.commits,
                () => [] as ReplayCommit[],
              ),
            ),
          );
          if (!cancelled) setHistory(mergedHistory(histories));
        } catch {
          if (!cancelled) setHistory([]);
        }

        // Structural navigation is a separate, optional request: it comes
        // from tree-sitter rather than the weave, and the lineage picture is
        // complete without it. A failure here must not cost the reader the
        // provenance they came for.
        try {
          if (at !== null || cancelled) return;
          const found = await boundedLineageRead(api.structure());
          if (!cancelled) setStructure(found);
        } catch {
          if (!cancelled) setStructure(null);
        }
      } catch (e) {
        if (!cancelled) {
          setProgress(null);
          setError(String((e as Error).message ?? e));
          setBusy(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [projectId, at, retry]);

  const limitedPaths = [...new Set((docs ?? []).flatMap(doc => [
    ...(doc.source.length > LINEAGE_MODEL_TEXT_LIMIT ? [doc.path] : []),
    ...doc.outputs.filter(file => file.content.length > LINEAGE_MODEL_TEXT_LIMIT).map(file => file.path),
  ]))];
  const revision = useUiRevision(() => [projectId, at, retry, error, docs, partial, boundary, busy, progress]);
  useUiReading("project lineage status", { projectId, at, error, partial, boundary, loading: (docs === null || progress !== null) && error === null, progress, complete: docs !== null && progress === null && !error && partial.length === 0 && boundary === null && limitedPaths.length === 0, preview_limited_files: limitedPaths,
    structural_navigation: at === null ? "Optional current working-tree structure" : "Unavailable for historical replay; current structure is not historical evidence" });
  useUiCommands(error || partial.length ? [{ key: `lineage-retry:${projectId}`, command: {
    label: "Retry reading project lineage", parent: `lineage:${projectId}`, subject: `lineage:${projectId}`,
    parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true }),
    execute: async () => { setRetry(value => value + 1); return { status: "completed", value: { read_requested: true, read_completed: false } }; },
  } }] : []);

  const { model, ambiguous } = useMemo(() => {
    if (!docs) return { model: null, ambiguous: new Map<string, number>() };
    const built = buildModel(docs);
    if (!structure) return { model: built, ambiguous: new Map<string, number>() };
    return withStructure(built, structure);
  }, [docs, structure]);

  const reading = progress && <p role="status">Reading the pipeline: {progress.completed} of {progress.total} documents finished; reading {progress.path}. Shown links cover only the finished reads.</p>;
  const slider = (
    <TimeSlider
      owner={projectId}
      commits={history}
      at={at}
      onChange={setAt}
      boundary={boundary}
      busy={busy}
    />
  );

  if (error) {
    return (
      <div className="lineage-empty">
        {slider}
        <button type="button" onClick={() => setRetry(value => value + 1)}>Retry reading project lineage</button>
        <p>Could not read this project: {error}</p>
        <p className="muted">
          The lineage browser reads the same documents and generated files the editor does; if
          those load, this will too.
        </p>
      </div>
    );
  }

  if (!model)
    return (
      <div className="lineage-view">
        {slider}
        {reading ?? <div className="lineage-empty muted">Reading the pipeline…</div>}
      </div>
    );

  if (model.stages.length === 0) {
    return (
      <div className="lineage-view">
        {slider}
        <div className="lineage-empty">
          {reading}
          {partial.length > 0 && <><p role="status">No readable stages yet; this graph is incomplete. {partial.join(", ")}</p><button type="button" onClick={() => setRetry(value => value + 1)}>Retry reading project lineage</button></>}
          <p>
            {partial.length > 0 || progress ? "Waiting for readable document stages." : at === null
              ? "No documents in this project yet."
              : "No document in this project existed at that commit."}
          </p>
          <p className="muted">A stage is a `.md` document plus the files it generates.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="lineage-view">
      {slider}
      {reading}
      {partial.length > 0 && <button type="button" onClick={() => setRetry(value => value + 1)}>Retry reading project lineage</button>}
      {limitedPaths.length > 0 && <p className="lineage-warning" role="status">This graph has limited previews for {limitedPaths.join(", ")}. Links involving those large files are omitted; open a file's full or owning document to read the complete source.</p>}
      {ambiguous.size > 0 && (
        <p className="lineage-note" role="status">
          {ambiguous.size} name{ambiguous.size === 1 ? "" : "s"} match more than one definition
          ({[...ambiguous.entries()].slice(0, 3).map(([name, n]) => `${name} ×${n}`).join(", ")}
          {ambiguous.size > 3 ? ", …" : ""}). Structural links are matched by name, so those are
          offered as candidates rather than answers.
        </p>
      )}
      {partial.length > 0 && (
        <p className="lineage-warning" role="status">
          {partial.length} document{partial.length === 1 ? "" : "s"} could not be fully read just now
          ({partial.join(", ")}), so some links are unavailable. Only successfully read provenance is shown.
        </p>
      )}
      <LineageColumns model={model} />
    </div>
  );
}
