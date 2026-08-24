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

import { api } from "../api/client";
import { LineageColumns } from "../lineage/LineageColumns";
import { TimeSlider } from "../lineage/TimeSlider";
import { buildModel, type SourceDocument } from "../lineage/build";
import { withStructure } from "../lineage/structure";
import type {
  OutputFile,
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
  const [docs, setDocs] = useState<SourceDocument[] | null>(null);
  const [structure, setStructure] = useState<StructureResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [partial, setPartial] = useState<string[]>([]);
  // Replay: which commit this picture is being shown at, `null` for the
  // working tree. See src/lineage/TimeSlider.tsx.
  const [history, setHistory] = useState<ReplayCommit[]>([]);
  const [at, setAt] = useState<string | null>(null);
  const [boundary, setBoundary] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setDocs(null);
    setError(null);
    setPartial([]);
    setBoundary(null);
    setBusy(at !== null);

    (async () => {
      try {
        const summaries = await api.projectDocs(projectId);
        const loaded: SourceDocument[] = [];
        const skipped: string[] = [];

        for (const summary of summaries) {
          // Replaying: the document as it stood, woven without executing
          // anything. A document that did not exist then is simply absent
          // from the picture, which is the truth about that commit.
          if (at !== null) {
            try {
              const replayed = await api.docReplay(summary.id, at);
              if (replayed.grammar_boundary) {
                if (!cancelled) setBoundary(replayed.message);
                continue;
              }
              const outputs: OutputFile[] = [];
              for (const path of replayed.outputs) {
                const one = await api.docReplay(summary.id, at, path);
                if (!one.grammar_boundary && one.output) {
                  outputs.push({
                    path: one.output.path,
                    language: "",
                    content: one.output.content,
                    provenance: one.output.provenance,
                  });
                }
              }
              loaded.push({ path: summary.path, source: replayed.source, outputs });
            } catch {
              skipped.push(summary.path);
            }
            continue;
          }

          const doc = await api.doc(summary.id);
          const outputs: OutputFile[] = [];
          try {
            const listed = await api.outputs(summary.id);
            for (const meta of listed.files) {
              // One request per generated file: provenance is what makes the
              // links real, and the list endpoint does not carry it.
              outputs.push(await api.outputFile(summary.id, meta.path));
            }
          } catch {
            // A document whose outputs cannot be woven right now (a parse
            // error, a missing upstream) still belongs in the picture as a
            // stage — with no links, which is the truth about it until it
            // weaves again.
            skipped.push(summary.path);
          }
          loaded.push({ path: doc.path, source: doc.source, outputs });
        }

        if (cancelled) return;
        setDocs(loaded);
        setPartial(skipped);
        setBusy(false);

        // The stops the slider can take: every commit that touched any of
        // these documents. Fetched once per project, not per replay.
        try {
          const histories = await Promise.all(
            summaries.map((s) =>
              api.docHistory(s.id).then(
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
          const found = await api.structure();
          if (!cancelled) setStructure(found);
        } catch {
          if (!cancelled) setStructure(null);
        }
      } catch (e) {
        if (!cancelled) {
          setError(String((e as Error).message ?? e));
          setBusy(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [projectId, at]);

  const { model, ambiguous } = useMemo(() => {
    if (!docs) return { model: null, ambiguous: new Map<string, number>() };
    const built = buildModel(docs);
    if (!structure) return { model: built, ambiguous: new Map<string, number>() };
    return withStructure(built, structure);
  }, [docs, structure]);

  const slider = (
    <TimeSlider
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
        <div className="lineage-empty muted">Reading the pipeline…</div>
      </div>
    );

  if (model.stages.length === 0) {
    return (
      <div className="lineage-view">
        {slider}
        <div className="lineage-empty">
          <p>
            {at === null
              ? "No documents in this project yet."
              : "No document in this project existed at that commit."}
          </p>
          <p className="muted">A stage is a `.hick` document plus the files it generates.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="lineage-view">
      {slider}
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
          {partial.length} document{partial.length === 1 ? "" : "s"} could not be woven just now
          ({partial.join(", ")}), so {partial.length === 1 ? "its" : "their"} links are missing
          rather than wrong.
        </p>
      )}
      <LineageColumns model={model} />
    </div>
  );
}
