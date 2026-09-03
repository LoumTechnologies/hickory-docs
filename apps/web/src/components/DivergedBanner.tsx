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
  onKeepMine?: () => void;
  /** Take theirs. Absent when theirs does not exist yet; `takeTheirsHint`
   * then says what would make it exist. */
  onTakeTheirs?: () => void;
  takeTheirsHint?: string;
  /** Open the three-way merge. */
  onMerge?: () => void;
  error?: string | null;
}

export function DivergedBanner({
  what,
  reason,
  mine,
  theirs,
  onKeepMine,
  onTakeTheirs,
  takeTheirsHint,
  onMerge,
  error,
}: DivergedBannerProps) {
  return (
    <div className="banner banner-warn diverged" role="alert" data-testid="diverged">
      <span className="diverged__text">
        <strong>{what}</strong> has diverged: {mine} is not {theirs}. {reason}
      </span>
      <span className="diverged__ways">
        {onKeepMine && (
          <button type="button" className="btn btn-small" onClick={onKeepMine}>
            Keep mine
          </button>
        )}
        {onTakeTheirs ? (
          <button type="button" className="btn btn-small" onClick={onTakeTheirs}>
            Take theirs
          </button>
        ) : (
          takeTheirsHint && <span className="muted">{takeTheirsHint}</span>
        )}
        {onMerge && (
          <button type="button" className="btn btn-small btn-primary" onClick={onMerge}>
            Merge…
          </button>
        )}
      </span>
      {error && <span className="error"> {error}</span>}
    </div>
  );
}
