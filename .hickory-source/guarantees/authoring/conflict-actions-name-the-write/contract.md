<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Conflict actions name the write

For: Authors editing executable documents and the files those documents produce.

Given a diverged file, when its choices are shown, then each button names
its consequence and the visible explanation says which bytes will be written
or discarded. Leaving disk unchanged dismisses a warning without claiming that
divergence is resolved. Regeneration is offered only when an output exists;
otherwise the surface explains how to obtain one.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/conflict-actions-name-the-write.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-a0cc7ee401d2"></a>
## `apps/web/src/components/DivergedBanner.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Conflict actions name the write](contract.md).

<a id="f-a0cc7ee401d2-DivergedBannerProps"></a>
### `DivergedBannerProps`

<storycode:copy id="f-a0cc7ee401d2-DivergedBannerProps">
import { useDivergenceActions } from "./useDivergenceActions";
// The one surface for "the disk does not hold what this document produces".
//
// A produced file the loop is holding, a produced file the document cannot
// reproduce yet, and a plain file that changed on disk under an unsaved
// buffer are one state — diverged — and used to be three banners with three
// vocabularies. This is the one banner, with the same three ways out
// everywhere: keep mine, take theirs, merge. Axis 3 of
// docs/specs/freeform/three-axes.md.

export interface DivergedBannerProps {
  /** What is diverged, for the sentence. */
  what: string;
  /** Why, in the words the loop or the save gave. */
  reason: string;
  /** What "mine" is — "the file on disk", "your unsaved text". */
  mine: string;
  /** What "theirs" is — "the document's version", "the file on disk". */
  theirs: string;
  /** Keep mine. Absent when keeping is the default and needs no click. */
  onKeepMine?: () => unknown;
  keepLabel: string;
  keepEffect: string;
  /** Take theirs. Absent when theirs does not exist yet; `takeTheirsHint`
   * then says what would make it exist. */
  onTakeTheirs?: () => unknown;
  takeLabel: string;
  takeEffect: string;
  takeTheirsHint?: string;
  /** Open the three-way merge. */
  onMerge?: () => unknown;
  sourceRevision?: () => unknown;
  error?: string | null;
}

</storycode:copy>
<a id="f-a0cc7ee401d2-DivergedBanner"></a>
### `DivergedBanner`

<storycode:copy id="f-a0cc7ee401d2-DivergedBanner">
export function DivergedBanner({
  what,
  reason,
  mine,
  theirs,
  onKeepMine, keepLabel, keepEffect,
  onTakeTheirs, takeLabel, takeEffect,
  takeTheirsHint,
  onMerge,
  error,
  sourceRevision,
}: DivergedBannerProps) {
  const actions = useDivergenceActions({ what, reason, mine, theirs, onKeepMine, keepLabel, keepEffect, onTakeTheirs, takeLabel, takeEffect, takeTheirsHint, onMerge, error, sourceRevision });
  return (
    <div className="banner banner-warn diverged" role="alert" data-testid="diverged">
      <span className="diverged__text">
        <strong>{what}</strong> has diverged: {mine} is not {theirs}. {reason}
      </span>
      <span className="diverged__ways">
        {onKeepMine && (
          <button type="button" className="btn btn-small" disabled={!!actions.pending} onClick={() => actions.invoke("keep")} data-tip={keepEffect}>
            {keepLabel}
          </button>
        )}
        {onTakeTheirs ? (
          <button type="button" className="btn btn-small" disabled={!!actions.pending} onClick={() => actions.invoke("take")} data-tip={takeEffect}>
            {takeLabel}
          </button>
        ) : (
          takeTheirsHint && <span className="muted">{takeTheirsHint}</span>
        )}
        {onMerge && (
          <button type="button" className="btn btn-small btn-primary" disabled={!!actions.pending} onClick={() => actions.invoke("merge")}>
            Merge…
          </button>
        )}
      </span>
      <p className="muted diverged__effects">{onKeepMine && keepEffect} {onTakeTheirs && takeEffect}</p>
      {(error || actions.failure) && <span className="error"> {error || actions.failure}</span>}
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/DivergedBanner.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/DivergedBanner.tsx"><storycode:paste select="#f-a0cc7ee401d2-DivergedBannerProps"/><storycode:paste select="#f-a0cc7ee401d2-DivergedBanner"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/DivergedBanner.tsx](contract.md#output-a0cc7ee401d2)
- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/shell/views.tsx](../../../mechanisms/apps/web/src/shell/views.tsx.md#output-68a8fba10658)

### Tests and fixtures

- [apps/web/src/components/DivergedBanner.test.tsx](../../../mechanisms/apps/web/src/components/DivergedBanner.test.tsx.md#output-c3c47782ddb8)
- [apps/web/src/lib/plainFileSave.test.ts](../../../mechanisms/apps/web/src/lib/plainFileSave.ts.md#output-94afcad10b27)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
