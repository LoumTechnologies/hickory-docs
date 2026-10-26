<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/MergedView.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0e4f75bd1f77"></a>
## `apps/web/src/views/MergedView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Merged View Is A Lens Over Several Worktrees, Never A Document](../../../../../guarantees/collaboration/the-merged-view-is-a-lens/contract.md), [A view declares where edits land](../../../../../guarantees/lenses/a-view-declares-where-edits-land/contract.md).

<a id="f-0e4f75bd1f77-MergedView"></a>
### `MergedView`

<storycode:copy id="f-0e4f75bd1f77-MergedView">
import { SurfaceInfo } from "../components/SurfaceInfo";
// One tab, several worktrees, and the merge already done.
//
// The file as it exists in several places at once: regions every source
// agrees on appear ONCE, regions that differ appear as variants in the shape
// `<hick:when>` already has. None of that text exists on disk — there is no
// file containing those conditionals, nothing to commit, nothing to merge.
//
// **It is a lens, not a document.** No save path, no `.md` extension, no
// place in the folder tree. The same category as a diff view, and nobody
// mistakes `git diff` output for a source artifact. Getting this wrong
// reverses the product's central claim: a document is the source of its
// files, and a synthetic document assembled FROM files points the other way.
//
// The sources are PEERS (decided 2026-08-23): shared means agreed by ALL of
// them, and there is no order, because the view removes the question.
//
// Region edits require explicit writable targets and captured source bytes.
// Each target reports its own outcome; the lens itself has no save path.
//
// docs/specs/freeform/the-merged-view.md

import { WorktreeRegionEditor } from "../components/WorktreeRegionEditor";
import { ComparisonEditor } from "../components/ComparisonEditor";
import { EvidenceDisclosure } from "../components/EvidenceDisclosure";
import { useMergedViewControls } from "./useMergedViewControls";
export function MergedView({ path }: { path: string }) {
  const controls = useMergedViewControls(path);
  const { worktrees, view, chosen, writable, editing, comparison, result, error, busy, needsRefresh, shown, human, apply } = controls;
  if (!worktrees || !view) return <><p className="muted">Reading the worktrees…</p>{error && <p role="alert">{error}</p>}
    <button type="button" disabled={busy} onClick={() => human("refresh")}>Refresh worktree comparison</button></>;

  if (!view.repository || worktrees.length === 0) {
    return (
      <div className="merged-view merged-view--none">
        <button type="button" disabled={busy} onClick={() => human("refresh")}>Refresh worktree comparison</button>
        <p className="muted">
          This folder is not a git repository with worktrees, so there is
          nothing to merge a view over.
        </p>
        <p className="muted">
          A merged view shows one file as it exists in several worktrees at
          once. “Across branches” means across worktrees — a branch that is
          not checked out cannot be written to without going behind the
          working tree.
        </p>
      </div>
    );
  }

  if (worktrees.length < 2) {
    return (
      <div className="merged-view merged-view--none">
        <button type="button" disabled={busy} onClick={() => human("refresh")}>Refresh worktree comparison</button>
        <p className="muted">
          This repository has one worktree, so a merged view over it would just
          be the file.
        </p>
        <p className="muted">
          Add one with <span className="mono">git worktree add</span>, and this
          shows what the two agree on and where they differ.
        </p>
      </div>
    );
  }

  return (
    <div className="merged-view">
      <SurfaceInfo name={`Worktree comparison · ${path}`} editable={writable.size>0 && !needsRefresh} destination="A lens with no save path. Region edits write only targets explicitly opened for writing, against the captured bytes." />
      <header className="merged-view__head">
        <span className="merged-view__path mono">{path}</span>
        {/* Said at the top rather than left to be discovered. */}
        <span className="merged-view__lens" data-tip="A lens, not a document: this text exists on disk nowhere, there is nothing to save, and nothing to commit. Region edits name their writable targets and check captured bytes before writing.">
          {writable.size && !needsRefresh ? "lens · selected targets editable" : "lens · read-only"}
        </span>
      </header>

      {error && <p role="alert">{error}</p>}
      {controls.latestRequest?.state === "failed_or_uncertain" && <p role="status">Latest {controls.latestRequest.kind} request failed. Inspect its error and target state before retrying.</p>}
      {needsRefresh && <p role="status">A write was requested. Refresh the captured comparison before editing another region; inspect the per-target result first.</p>}
      <button type="button" disabled={busy} onClick={() => human("refresh")}>Refresh worktree comparison</button>
      <button className="btn" disabled={busy || needsRefresh || !view.snapshot || shown.length < 2} onClick={() => human("inspect")}>Compare complete worktree projects</button>
      {comparison && <ComparisonEditor path={path} model={comparison} onChange={busy ? undefined : controls.setComparison} />}
      {result && <div aria-label="Per-target write status"><p>Last received {controls.lastReplyKind} reply{controls.resultIsLatest ? "" : " · from an earlier request"}</p>{result.targets.map(t=><p key={t.source} role={t.ok?"status":"alert"}>{t.source} · {t.ok?"applied":t.error}</p>)}
        {result.undo?.map(item=><button key={item.source} className="btn" disabled={busy} onClick={() => human(`undo:${item.source}`)}>Undo {item.source}</button>)}
      <EvidenceDisclosure label="Worktree write reply" value={result}><pre>{JSON.stringify(result, null, 2)}</pre></EvidenceDisclosure></div>}
      <div className="merged-view__sources">
        {worktrees.map((tree) => {
          const on = chosen ? chosen.has(tree.name) : true;
          return (
            <label key={tree.path} className="merged-view__source">
              <input
                type="checkbox"
                checked={on}
                disabled={busy || !controls.canSelect(tree.name)}
                onChange={event => human(`select:${tree.name}`, { on: event.target.checked })}
              />
              <span className="mono">{tree.name}</span>
              {tree.branch && <span className="muted"> · {tree.branch}</span>}
              {tree.current && <span className="muted"> · here</span>}
              <input type="checkbox" aria-label={`Open ${tree.name} for writing`} checked={writable.has(tree.name)} disabled={busy || !controls.canWrite(tree.name)} onChange={event => human(`write:${tree.name}`, { on: event.target.checked })} />write
            </label>
          );
        })}
      </div>

      {view.missing.length > 0 && (
        <p className="lineage-note" role="status">
          {view.missing.join(", ")} {view.missing.length === 1 ? "does" : "do"} not
          have this file at all — which is an answer, not an omission.
        </p>
      )}

      <p className="merged-view__summary muted">
        {view.shared_lines ?? 0} line{(view.shared_lines ?? 0) === 1 ? "" : "s"} agreed by
        all {shown.length} source{shown.length === 1 ? "" : "s"}
        {(view.variants ?? 0) > 0
          ? `, ${view.variants} place${view.variants === 1 ? "" : "s"} where they differ.`
          : ". They agree everywhere."}{" "}
        A shared region is agreed by construction for this capture. Later writes
        still recheck each target’s captured bytes.
      </p>

      <div className="merged-view__body">
        {view.regions.map((region, i) =>
          region.kind === "shared" ? (
            <section key={i} className="merged-region merged-region--shared"><pre>{region.text}</pre>
              <button className="btn" disabled={busy || needsRefresh || !view.snapshot} onClick={() => human(`region:${i}`)}>Edit shared region</button>
              {editing===i && <WorktreeRegionEditor disabled={busy || needsRefresh} sourceEpoch={view.snapshot ? JSON.stringify([path, view.snapshot, i]) : null} path={path} values={Object.fromEntries(shown.map(name=>[name,region.text]))} shared writable={writable} onApply={(text,target,route)=>apply(i,text,target,route)} />}
            </section>
          ) : (
            <div key={i} className="merged-region merged-region--variant">
              <button className="btn" disabled={busy || needsRefresh || !view.snapshot} onClick={() => human(`region:${i}`)}>Edit variant region</button>
              {editing===i && <WorktreeRegionEditor disabled={busy || needsRefresh} sourceEpoch={view.snapshot ? JSON.stringify([path, view.snapshot, i]) : null} path={path} values={region.by_source} shared={false} writable={writable} onApply={(text,target,route)=>apply(i,text,target,route)} />}
              {Object.entries(region.by_source).map(([name, text]) => (
                <div key={name} className="merged-variant">
                  <span className="merged-variant__label mono">{name}</span>
                  {text ? (
                    <pre className="merged-variant__text">{text}</pre>
                  ) : (
                    <p className="muted merged-variant__empty">nothing here</p>
                  )}
                </div>
              ))}
            </div>
          ),
        )}
      </div>
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/views/MergedView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/MergedView.tsx"><storycode:paste select="#f-0e4f75bd1f77-MergedView"/></storycode:file>

</storycode:doc>
