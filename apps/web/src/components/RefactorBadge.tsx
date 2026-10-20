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

import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome, UiValue } from "../lib/uiActions";
import { api } from "../api/client";
import type { RefactorStatus } from "../api/types";
import { refactorDetail, refactorSummary } from "../lib/refactorSummary";

/** How often the live verdict refreshes while a baseline is pinned. The
 * check weaves the document server-side (cached, never executing), so this
 * is cheap — and a badge that lags keystrokes by a couple of seconds is
 * still telling the truth about the last settled text. */
const POLL_MS = 2500;

export interface Baseline {
  scope: string;
  status: RefactorStatus | null;
  active: boolean;
  busy: boolean;
  error: string | null;
  begin: () => Promise<UiOutcome>;
  end: () => Promise<UiOutcome>;
  refresh: () => Promise<UiOutcome>;
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
  const scope = useMemo(() => crypto.randomUUID(), [docId]);
  const [snapshot, setSnapshot] = useState<{ owner: string; status: RefactorStatus } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const currentDoc = useRef(docId); currentDoc.current = docId;
  const life = useRef({ owner: docId, live: false, sequence: 0, pending: false });
  const status = snapshot?.owner === docId ? snapshot.status : null;
  const request = useCallback(async (kind: "status" | "begin" | "end"): Promise<UiOutcome> => {
    const captured = life.current;
    if (!captured.live || captured.owner !== docId || currentDoc.current !== docId || captured.pending)
      return { status: "unavailable", reason: "Baseline owner is closed, changed or busy" };
    const ticket = ++captured.sequence;
    if (kind !== "status") { captured.pending = true; setBusy(true); }
    try {
      const response = await (kind === "status" ? api.refactorStatus(docId) : kind === "begin" ? api.refactorBegin(docId) : api.refactorEnd(docId));
      if (captured.live && currentDoc.current === docId && captured.sequence === ticket) { setSnapshot({ owner: docId, status: response }); setError(null); }
      return { status: "completed", value: { doc_id: docId, response: JSON.parse(JSON.stringify(response)) as UiValue,
        scope: "Actual baseline API reply. Weave-only comparison; no cell execution or independent output verification." } };
    } catch (cause) {
      const reason = `${String(cause)}. Inspect this document's baseline before retrying; no rollback is asserted.`;
      if (captured.live && currentDoc.current === docId && captured.sequence === ticket) setError(reason);
      return { status: "unavailable", reason };
    } finally { if (kind !== "status") { captured.pending = false; if (captured.live && currentDoc.current === docId) setBusy(false); } }
  }, [docId]);
  useEffect(() => {
    const captured = { owner: docId, live: true, sequence: 0, pending: false }; life.current = captured;
    setSnapshot(null); setBusy(false); setError(null);
    void request("status");
    return () => { captured.live = false; captured.sequence++; };
  }, [docId, request]);
  const active = status?.active === true;
  useEffect(() => {
    if (!active) return;
    const timer = setInterval(() => { void request("status"); }, POLL_MS);
    return () => clearInterval(timer);
  }, [active, request]);
  return { scope, status, active, busy: life.current.owner === docId && busy, error: life.current.owner === docId ? error : null, begin: () => request("begin"), end: () => request("end"), refresh: () => request("status") };
}

/**
 * The verdict, while a baseline is pinned. Nothing at all when none is.
 *
 * An idle mode used to advertise itself with a button in the toolbar; now it
 * lives in the overflow menu, and this slot stays empty until there is
 * something true to say in it.
 */
export function RefactorBadge({ baseline }: { baseline: Baseline }) {
  const { status, busy, error, end, refresh } = baseline;
  const parent = `output-baseline:${useId()}`;
  const revision = useUiRevision(() => [baseline.scope, status, busy, error]);
  useUiCommands([
    ...(status?.active ? [{ key: `${parent}:end`, command: { parent, subject: parent, label: "Stop comparing outputs to baseline", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy }), execute: end } }] : []),
    ...(status?.active || error ? [{ key: `${parent}:refresh`, command: { parent, subject: parent, label: "Refresh output baseline verdict", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy }), execute: refresh } }] : []),
  ]);
  useUiReading("output baseline verdict", { active: status?.active ?? null, busy, error, summary: refactorSummary(status), detail: refactorDetail(status),
    scope: "Last host-observed weave-only comparison; no cell execution or independent verification" });
  if (!status || !status.active) {
    return error ? (
      <span className="error refactor-badge__error" role="status">
        {error}<button type="button" disabled={busy} onClick={() => void refresh()}>Refresh baseline verdict</button>
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
      <button className="btn" disabled={busy} onClick={() => void refresh()}>Refresh baseline verdict</button>
      <button
        className="btn"
        disabled={busy}
        onClick={() => void end()}
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
