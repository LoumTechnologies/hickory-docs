<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MergeView.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4c1db87e0a96"></a>
## `apps/web/src/components/MergeView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4c1db87e0a96-MergeViewProps"></a>
### `MergeViewProps`

<storycode:copy id="f-4c1db87e0a96-MergeViewProps">
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";
import { LocalRealtime } from "../api/realtime";
import type { EditorView } from "@codemirror/view";
const LiterateEditor = lazy(() => import("../editor/DocumentEditor").then(module => ({ default: module.DocumentEditor })));
// The merge, as something to read and answer.
//
// It opens whenever two versions of one file both have changes worth keeping:
// a draft restored against a file that moved on while the app was closed, and
// — once anything else needs it — any other three-way situation, because the
// shape is the same one every time.
//
// The design rule here is that the reader should be able to see how much of
// this is actually their problem before they read a word of it. Most of a
// merge is not: regions only one side changed are already decided, and saying
// so plainly ("11 changes merged, 2 need you") is the difference between a
// merge someone answers and a merge someone abandons.
//
// So:
//  - Agreed text is shown, quietly, in place, because context is what makes a
//    conflict answerable.
//  - Auto-resolved regions are shown and LABELLED with who they came from —
//    not hidden. A merge tool that silently took a side is a merge tool
//    nobody trusts twice.
//  - Conflicts are the only thing that looks like a question.
//
// The merged text is live: it updates as answers are given, and it is what
// lands in the buffer when the reader accepts. Until then the unanswered
// conflicts read as OUR text (see `mergedText`), so the reader's own unsaved
// work never disappears off the screen while they think.

import { lazy, Suspense, useEffect, useId, useMemo, useRef, useState } from "react";

import {
  conflictCount,
  mergeThreeWay,
  mergeTwoWay,
  mergedText,
  type MergeRegion,
  type Resolution,
} from "../lib/merge";

export interface MergeViewProps {
  literate?: boolean;
  /** What is being merged, for the heading. */
  path: string;
  /** The common ancestor. Empty string means there is none, and the merge
   * falls back to a two-way comparison that has to ask about everything. */
  base: string;
  /** Our side — in the restore case, the reader's own unsaved buffer. */
  ours: string;
  /** Their side — in the restore case, the file as it is on disk now. */
  theirs: string;
  /** What each side is called, in this reader's terms. */
  oursLabel?: string;
  theirsLabel?: string;
  /** Take the merged text. */
  onAccept: (text: string) => void | boolean | Record<string, unknown> | Promise<void | boolean | Record<string, unknown>>;
  sourceCurrent?: () => boolean;
  /** Walk away and keep `ours` untouched. */
  onCancel: () => void;
}

</storycode:copy>
<a id="f-4c1db87e0a96-CHOICES"></a>
### `CHOICES`

<storycode:copy id="f-4c1db87e0a96-CHOICES">
const CHOICES: { value: Resolution; label: string; hint: string }[] = [
  { value: "ours", label: "Keep mine", hint: "Take this side and drop theirs" },
  { value: "theirs", label: "Keep theirs", hint: "Take their side and drop mine" },
  { value: "both", label: "Keep both", hint: "Mine first, then theirs" },
  { value: "base", label: "Keep neither", hint: "Go back to what was there before" },
];

</storycode:copy>
<a id="f-4c1db87e0a96-MergeView"></a>
### `MergeView`

<storycode:copy id="f-4c1db87e0a96-MergeView">
export function MergeView(props: MergeViewProps) {
  return <MergeController key={JSON.stringify([props.path, props.base, props.ours, props.theirs])} {...props} />;
}

</storycode:copy>
<a id="f-4c1db87e0a96-MergeController"></a>
### `MergeController`

<storycode:copy id="f-4c1db87e0a96-MergeController">
function MergeController({
  path,
  base,
  ours,
  theirs,
  oursLabel = "Your unsaved changes",
  theirsLabel = "The file on disk",
  onAccept,
  onCancel,
  sourceCurrent,
  literate = false,
}: MergeViewProps) {
  // A base of "" is genuinely "no common ancestor", not "the ancestor was an
  // empty file" — a draft with no file behind it stores it that way. The
  // difference matters: three-way over an empty base would call every line of
  // both files an addition and merge them into one another.
  const threeWay = base.length > 0;
  const regions = useMemo(
    () => (threeWay ? mergeThreeWay(base, ours, theirs) : mergeTwoWay(ours, theirs)),
    [threeWay, base, ours, theirs],
  );
  const [answers, setAnswers] = useState<ReadonlyMap<number, Resolution>>(new Map());

  const total = conflictCount(regions);
  const auto = regions.filter((r) => r.kind === "resolved").length;
  const answered = [...answers.keys()].length;
  const [edited, setEdited] = useState<string | null>(null);
  const text = edited ?? mergedText(regions, answers);

  const answer = (index: number, choice: Resolution) => {
    setEdited(null);
    setAnswers((current) => {
      const next = new Map(current);
      // Choosing the same answer again clears it, which is the only way back
      // to "I have not decided" without reopening the whole merge.
      if (next.get(index) === choice) next.delete(index);
      else next.set(index, choice);
      return next;
    });
  };

  const parent = `merge:${useId()}`;
  const [busy, setBusy] = useState(false), [error, setError] = useState("");
  const inFlight = useRef(false);
  const current = () => sourceCurrent?.() ?? true;
  const revision = useUiRevision(() => [path, base, ours, theirs, [...answers], busy, error, current()]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: !busy && current(), reason: busy ? "Merge acceptance pending" : !current() ? "Merge owner changed or closed" : undefined }), execute } });
  add("read", "Read merge regions and preview bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ path, three_way: threeWay, ours_label: oursLabel, theirs_label: theirsLabel, regions, answers: [...answers], preview: text, conflicts: total, answered, auto, error: error || null, scope: "Current merge preview; undecided conflicts keep ours. Acceptance is not independent verification or named proposal approval." }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    return offset > bytes.length ? { status: "unavailable", reason: "Offset outside current merge" } : { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  add("answer", "Choose merge conflict resolution", { index: { type: "number", minimum: 0, maximum: 1000000, integer: true }, choice: { type: "string", maxLength: 16, choices: threeWay ? ["ours", "theirs", "both", "base"] : ["ours", "theirs", "both"] } }, async args => {
    if ((args.index as number) >= total) return { status: "unavailable", reason: "Conflict outside current merge" };
    answer(args.index as number, args.choice as Resolution); return { status: "completed", value: { preview_changed: true, source_written: false } };
  });
  add("cancel", "Cancel merge", {}, async () => { onCancel(); return { status: "completed", value: { cancelled: true, source_written: false } }; });
  add("accept", "Accept merge preview", {}, async () => {
    if (inFlight.current || !current()) return { status: "unavailable", reason: "Merge owner changed or an acceptance is pending" };
    inFlight.current = true; setBusy(true); setError("");
    try { const accepted = await onAccept(text); if (accepted === false) throw new Error("Merge owner refused stale input; inspect the current merge or save error"); return { status: "completed", value: { handler_returned: true, owner_reply: JSON.stringify(accepted ?? null), saved: "Not established by this merge controller", approved: false, verified: false } }; }
    catch (e) { const reason = e instanceof Error ? e.message : String(e); setError(reason); return { status: "unavailable", reason }; }
    finally { inFlight.current = false; setBusy(false); }
  });
  const invoke = useUiCommands(commands);
  const human = (key: string, args: Parameters<typeof invoke>[1] = {}) => void invoke(`${parent}:${key}`, args).catch(() => undefined);
  let conflictIndex = -1;

  return (
    <section className="merge-view" data-testid="merge-view" aria-label={`Merge ${path}`}>
      <header className="merge-view__head">
        <h2 className="merge-view__title">
          <span className="mono">{path}</span> changed while you were away
        </h2>
        {/* The first thing the reader needs is how much of this is their
            problem. */}
        <p className="merge-view__summary" role="status">
          {total === 0
            ? `Merged cleanly — ${auto} ${auto === 1 ? "change" : "changes"} from both sides, nothing to decide.`
            : `${auto} ${auto === 1 ? "change" : "changes"} merged on their own. ${total - answered} of ${total} still ${total - answered === 1 ? "needs" : "need"} you.`}
        </p>
        {!threeWay && (
          <p className="merge-view__note">
            There is no common ancestor for these two, so every difference is a
            question — nothing can tell an addition on one side from a deletion
            on the other.
          </p>
        )}
      </header>

      <div className="merge-view__regions">
        {regions.map((region, i) => {
          if (region.kind === "conflict") conflictIndex++;
          return (
            <MergeRegionRow
              key={i}
              region={region}
              conflictIndex={region.kind === "conflict" ? conflictIndex : -1}
              chosen={region.kind === "conflict" ? answers.get(conflictIndex) : undefined}
              threeWay={threeWay}
              oursLabel={oursLabel}
              theirsLabel={theirsLabel}
              onAnswer={(index, choice) => human("answer", { index, choice })}
              disabled={busy || !current()}
            />
          );
        })}
      </div>

      {literate && <Suspense fallback={<p role="status">Opening literate merge editor…</p>}>
        <LiterateMergePreview path={path} source={text} onChange={setEdited} readOnly={busy || !current()} />
      </Suspense>}
      {error && <p role="alert">{error}</p>}
      <footer className="merge-view__actions">
        <button type="button" className="btn" disabled={busy} onClick={() => human("cancel")}>
          Cancel
        </button>
        <button
          type="button"
          className="btn btn-primary"
          disabled={busy || !current()}
          onClick={() => human("accept")}
          // Deliberately NOT disabled while conflicts are unanswered. An
          // unanswered conflict already reads as our text, which is what the
          // buffer holds right now — accepting is a real, meaningful choice
          // ("everything I have not answered stays mine") and blocking it
          // would trap someone in a dialog over a region they do not care
          // about.
          data-tip={
            answered < total
              ? "Unanswered conflicts keep your side"
              : "Put the merged text in the editor"
          }
        >
          {answered < total ? "Accept, keeping mine where undecided" : "Accept merge"}
        </button>
      </footer>
    </section>
  );
}

</storycode:copy>
<a id="f-4c1db87e0a96-MergeRegionRow"></a>
### `MergeRegionRow`

<storycode:copy id="f-4c1db87e0a96-MergeRegionRow">
function MergeRegionRow({
  region,
  conflictIndex,
  chosen,
  threeWay,
  oursLabel,
  theirsLabel,
  onAnswer,
  disabled,
}: {
  region: MergeRegion;
  conflictIndex: number;
  chosen: Resolution | undefined;
  threeWay: boolean;
  oursLabel: string;
  theirsLabel: string;
  onAnswer: (index: number, choice: Resolution) => void;
  disabled: boolean;
}) {
  if (region.kind === "stable") {
    if (!region.text) return null;
    return (
      <pre className="merge-region merge-region--stable">
        <code>{region.text}</code>
      </pre>
    );
  }

  if (region.kind === "resolved") {
    // Shown, not hidden. A merge tool that silently took a side is a merge
    // tool nobody trusts twice.
    const who =
      region.from === "both"
        ? "both sides made this same change"
        : region.from === "ours"
          ? `only ${oursLabel.toLowerCase()} touched this`
          : `only ${theirsLabel.toLowerCase()} changed this`;
    return (
      <div className="merge-region merge-region--resolved">
        <p className="merge-region__label">Merged — {who}</p>
        <pre>
          <code>{region.text}</code>
        </pre>
      </div>
    );
  }

  const choices = threeWay ? CHOICES : CHOICES.filter((c) => c.value !== "base");
  return (
    <div
      className={`merge-region merge-region--conflict${chosen ? " merge-region--answered" : ""}`}
      role="group"
      aria-label={`Conflict ${conflictIndex + 1}`}
    >
      <div className="merge-region__sides">
        <div
          className={`merge-side merge-side--ours${chosen === "ours" || chosen === "both" ? " merge-side--taken" : ""}`}
        >
          <p className="merge-region__label">{oursLabel}</p>
          <pre>
            <code>{region.ours || <em className="muted">(nothing)</em>}</code>
          </pre>
        </div>
        <div
          className={`merge-side merge-side--theirs${chosen === "theirs" || chosen === "both" ? " merge-side--taken" : ""}`}
        >
          <p className="merge-region__label">{theirsLabel}</p>
          <pre>
            <code>{region.theirs || <em className="muted">(nothing)</em>}</code>
          </pre>
        </div>
      </div>
      <div className="merge-region__choices" role="toolbar">
        {choices.map((choice) => (
          <button
            key={choice.value}
            type="button"
            className={`btn btn-small${chosen === choice.value ? " btn-primary" : ""}`}
            disabled={disabled}
            aria-pressed={chosen === choice.value}
            data-tip={choice.hint}
            onClick={() => onAnswer(conflictIndex, choice.value)}
          >
            {choice.label}
          </button>
        ))}
      </div>
    </div>
  );
}
</storycode:copy>

### Offer a choice before replacing a buffer

Poll the saved source independently of the editor's live text. A disk version is
only an observation: ignoring it records that version locally and writes nothing.
Reload and merge both re-read disk and compare the live buffer before applying;
a preference may automate this only while the buffer still equals its saved base.
The timer also catches edits while the window stays focused. A failed read keeps
the buffer available. The host owns persistence and supplies the saved snapshot.

For disk conflicts, mount the real literate editor over a local-only room for the
merged result. Region choices supply its initial result; free edits remain a
preview until Accept. Changing a region choice rebuilds that result, so choices
should precede free editing. No exec action runs from this preview.

<storycode:copy id="disk-change-choice">
function LiterateMergePreview({ path, source, onChange, readOnly }: {
  path: string; source: string; onChange: (text: string) => void; readOnly: boolean;
}) {
  const realtime = useMemo(() => new LocalRealtime(), []);
  const [view, setView] = useState<EditorView | null>(null);
  useEffect(() => {
    if (view && view.state.doc.toString() !== source) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: source } });
  }, [view, source]);
  useEffect(() => () => realtime.close(), [realtime]);
  return <section aria-label="Literate merge result"><h3>Edit merged result</h3>
    <p>Choose regions first, then edit the result. Changing a region choice resets manual edits in this preview.</p>
    <LiterateEditor docId={`disk-merge:${path}`} initialSource={source} realtime={realtime}
      execBlocks={[]} runningCells={new Set()} onRunCell={() => {}} readOnly={readOnly}
      onViewReady={setView} onChange={onChange} />
  </section>;
}

export function DiskChangeNotice<T extends { source: string }>({ path, base, source, read, apply }: {
  path: string; base: string; source: string;
  read: () => Promise<T>;
  apply: (disk: T, text: string) => void | Promise<void>;
}) {
  const current = useRef({ base, source, read, apply });
  current.current = { base, source, read, apply };
  const [pending, setPending] = useState<T | null>(null);
  const [merge, setMerge] = useState<{ base: string; ours: string; disk: T } | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [automatic, setAutomatic] = useState(() => {
    try { return localStorage.getItem("hickory.reload-unmodified") === "true"; } catch { return false; }
  });
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const ignored = useRef<string | null>(null);
  const applying = useRef(false);
  const accept = async (disk: T, text: string, expected: string) => {
    if (applying.current) return false;
    applying.current = true; setBusy(true); setError("");
    try {
      const fresh = await current.current.read();
      if (!mounted.current) return false;
      if (fresh.source !== disk.source) { setPending(fresh); setMerge(null); throw new Error("The file changed again. Review its latest version."); }
      if (current.current.source !== expected) throw new Error("Your buffer changed. Its newer edits were kept; choose again.");
      await current.current.apply(fresh, text);
      ignored.current = null; setPending(null); setMerge(null);
      return true;
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); return false; }
    finally { applying.current = false; setBusy(false); }
  };
  const acceptRef = useRef(accept); acceptRef.current = accept;
  useEffect(() => {
    let live = true, reading = false;
    const check = async () => {
      if (reading || applying.current) return;
      reading = true;
      try {
        const disk = await current.current.read();
        if (!live) return;
        const now = current.current;
        if (disk.source === now.base || disk.source === now.source) { setPending(null); return; }
        if (disk.source === ignored.current) return;
        setPending(disk);
        let auto = automatic;
        try { auto = localStorage.getItem("hickory.reload-unmodified") === "true"; } catch { /* Per-view fallback. */ }
        if (auto && now.source === now.base) void acceptRef.current(disk, disk.source, now.source);
      } catch { /* Unreadable files never replace a surviving buffer. */ }
      finally { reading = false; }
    };
    const run = () => { void check(); };
    const timer = setInterval(run, 2000);
    window.addEventListener("focus", run);
    window.addEventListener("hickory:files-changed", run);
    run();
    return () => { live = false; clearInterval(timer); window.removeEventListener("focus", run); window.removeEventListener("hickory:files-changed", run); };
  }, [path, automatic]);
  const parent = `disk-change:${useId()}`;
  const revision = useUiRevision(() => [path, current.current.base, current.current.source, pending?.source, ignored.current, busy, automatic, !!merge]);
  const invoke = useUiCommands((["reload", "ignore", "merge", "automatic", "review"] as const).map((operation): { key: string; command: UiCommand } => ({ key: `${parent}:${operation}`, command: {
    parent, subject: path,
    label: ({ reload: "Reload disk version and discard unsaved buffer", ignore: "Ignore disk changes without writing", merge: "Open literate merge editor", automatic: "Set safe automatic reload preference", review: "Review ignored disk changes" })[operation],
    parameters: operation === "automatic" ? { enabled: { type: "boolean" as const } } : {},
    prerequisites: ["Current disk observation and buffer; reload discards unsaved edits; merge acceptance uses a fresh disk read"],
    read: () => ({ revision: revision(), enabled: !applying.current && (operation === "automatic" ? true : operation === "review" ? ignored.current !== null : !!pending && !merge), reason: "Disk choice changed or application pending" }),
    execute: async args => {
      if (operation === "automatic") {
        const enabled = args.enabled === true; setAutomatic(enabled);
        try { localStorage.setItem("hickory.reload-unmodified", String(enabled)); } catch { /* Per-view preference. */ }
      } else if (operation === "review") {
        const disk = await current.current.read();
        if (!mounted.current) return { status: "unavailable", reason: "Disk view closed" };
        ignored.current = null; setPending(disk);
      } else if (!pending) return { status: "unavailable", reason: "Disk observation changed" };
      else if (operation === "reload") {
        if (!await accept(pending, pending.source, current.current.source)) return { status: "unavailable", reason: "Reload refused stale disk or buffer; inspect the notice" };
      } else if (operation === "ignore") { ignored.current = pending.source; setPending(null); setError(""); }
      else if (operation === "merge") setMerge({ base: current.current.base, ours: current.current.source, disk: pending });
      return { status: "completed", value: { operation, scope: "Disk choice handler completed; ignore writes nothing; save and verification depend on the owning editor" } };
    },
  } })));
  const human = (operation: string, args: Parameters<typeof invoke>[1] = {}) => { void invoke(`${parent}:${operation}`, args).catch(e => setError(e instanceof Error ? e.message : String(e))); };
  const preference = <label><input type="checkbox" checked={automatic} onChange={event => human("automatic", { enabled: event.target.checked })} />Always reload when no unsaved changes would be lost</label>;
  if (!pending) return ignored.current === null ? (automatic ? preference : null) : <p role="status">Disk changes ignored. Your buffer was kept. <button type="button" className="btn" onClick={() => human("review")}>Review disk changes</button></p>;
  const dirty = source !== base;
  return <section className="banner" aria-label={`Disk changes for ${path}`}>
    <p role="status">{path} changed on disk.{dirty ? " Reloading will discard your unsaved changes." : " You can reload without losing changes."}</p>
    {error && <p role="alert">{error}</p>}
    {merge ? <MergeView literate path={path} base={merge.base} ours={merge.ours} theirs={merge.disk.source}
      sourceCurrent={() => current.current.source === merge.ours && !applying.current}
      onAccept={text => accept(merge.disk, text, merge.ours)} onCancel={() => setMerge(null)} /> : <>
      <button type="button" className="btn" disabled={busy} onClick={() => human("ignore")}>Ignore disk changes</button>
      <button type="button" className="btn" disabled={busy} onClick={() => human("reload")}>{dirty ? "Reload and discard unsaved changes" : "Reload from disk"}</button>
      {dirty ? <button type="button" className="btn" disabled={busy} onClick={() => human("merge")}>Merge changes</button> :
        preference}
    </>}
  </section>;
}
</storycode:copy>

### Assemble `apps/web/src/components/MergeView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MergeView.tsx"><storycode:paste select="#f-4c1db87e0a96-MergeViewProps"/><storycode:paste select="#f-4c1db87e0a96-CHOICES"/><storycode:paste select="#f-4c1db87e0a96-MergeView"/><storycode:paste select="#f-4c1db87e0a96-MergeController"/><storycode:paste select="#f-4c1db87e0a96-MergeRegionRow"/><storycode:paste select="#disk-change-choice"/></storycode:file>

</storycode:doc>
