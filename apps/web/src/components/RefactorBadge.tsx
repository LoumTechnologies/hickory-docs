// The output baseline, as document-toolbar chrome.
//
// What it does: pins the document's current woven outputs on the server
// (POST /refactor/begin), then polls the live verdict — weave-only, no cell
// executes — and says either "Outputs match" or which outputs moved.
// Dropping the baseline makes the current outputs the new truth. It is the
// same check `hick equiv` runs.
//
// **Not called "Refactor" any more, and that was worth changing.** In every
// IDE a developer has used, "Refactor" opens a menu of rename / extract /
// inline — code actions on a symbol. Here it started a mode. A word that
// means something specific and different in every other editor is a word
// that costs a person one wrong click and a paragraph of documentation to
// recover from, and the thing it names is better described by what it
// produces: a baseline.
//
// The baseline is session state on the server, so the badge resumes after a
// tab close and re-open by asking for status on mount.

import { useCallback, useEffect, useRef, useState } from "react";

import { api } from "../api/client";
import type { RefactorStatus } from "../api/types";
import { refactorDetail, refactorSummary } from "../lib/refactorSummary";

/** How often the live verdict refreshes while a baseline is pinned. The
 * check weaves the document server-side (cached, never executing), so this
 * is cheap — and a badge that lags keystrokes by a couple of seconds is
 * still telling the truth about the last settled text. */
const POLL_MS = 2500;

export interface Baseline {
  status: RefactorStatus | null;
  active: boolean;
  busy: boolean;
  error: string | null;
  begin: () => void;
  end: () => void;
}

/**
 * The baseline for one document: its live verdict, and the two verbs.
 *
 * A hook rather than state inside the badge, because the badge is no longer
 * the only thing that needs it — the toolbar's overflow menu offers the verb
 * while the badge shows only the verdict, and both have to be looking at the
 * same baseline.
 */
export function useBaseline(docId: string): Baseline {
  const [status, setStatus] = useState<RefactorStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const liveRef = useRef(true);

  const refresh = useCallback(() => {
    api.refactorStatus(docId).then(
      (s) => {
        if (!liveRef.current) return;
        setStatus(s);
        setError(null);
      },
      (e) => liveRef.current && setError(e instanceof Error ? e.message : String(e)),
    );
  }, [docId]);

  // Resume on mount (the baseline outlives this tab), then poll while
  // active. The interval always runs off the LATEST status — an inactive
  // baseline polls nothing.
  useEffect(() => {
    liveRef.current = true;
    refresh();
    return () => {
      liveRef.current = false;
    };
  }, [refresh]);
  const active = status?.active === true;
  useEffect(() => {
    if (!active) return;
    const timer = setInterval(refresh, POLL_MS);
    return () => clearInterval(timer);
  }, [active, refresh]);

  const begin = useCallback(() => {
    setBusy(true);
    api.refactorBegin(docId).then(
      (s) => {
        if (!liveRef.current) return;
        setBusy(false);
        setStatus(s);
        setError(null);
      },
      (e) => {
        if (!liveRef.current) return;
        setBusy(false);
        setError(e instanceof Error ? e.message : String(e));
      },
    );
  }, [docId]);

  const end = useCallback(() => {
    setBusy(true);
    api.refactorEnd(docId).then(
      (s) => {
        if (!liveRef.current) return;
        setBusy(false);
        setStatus(s);
      },
      () => liveRef.current && setBusy(false),
    );
  }, [docId]);

  return { status, active, busy, error, begin, end };
}

/**
 * The verdict, while a baseline is pinned. Nothing at all when none is.
 *
 * An idle mode used to advertise itself with a button in the toolbar; now it
 * lives in the overflow menu, and this slot stays empty until there is
 * something true to say in it.
 */
export function RefactorBadge({ baseline }: { baseline: Baseline }) {
  const { status, busy, error, end } = baseline;
  if (!status || !status.active) {
    return error ? (
      <span className="error refactor-badge__error" role="status">
        {error}
      </span>
    ) : null;
  }

  const summary = refactorSummary(status);
  const detail = refactorDetail(status);
  return (
    <>
      <span
        className={`refactor-badge ${status.clean ? "refactor-badge--clean" : "refactor-badge--dirty"}`}
        role="status"
        {...(detail ? { "data-tip": detail } : {})}
      >
        {status.clean ? "✓ " : "△ "}
        {summary}
      </span>
      <button
        className="btn"
        disabled={busy}
        onClick={end}
        data-tip="Stop comparing. The outputs as they stand become the new baseline."
      >
        Done
      </button>
      {error && (
        <span className="error refactor-badge__error" role="status">
          {error}
        </span>
      )}
    </>
  );
}
