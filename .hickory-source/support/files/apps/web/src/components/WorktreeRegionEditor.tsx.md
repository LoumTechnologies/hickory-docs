<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/WorktreeRegionEditor.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `44118f7ca2e5e41b24295efaa85546be8d618f12b2554c16c12b18a614a1a5bb`.

<book:entry path="apps/web/src/components/WorktreeRegionEditor.tsx">
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { ComparisonEditor } from "./ComparisonEditor";
import { EvidenceDisclosure } from "./EvidenceDisclosure";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome, UiValue } from "../lib/uiActions";
import type { ComparisonModel } from "../lib/projectComparison";

export function WorktreeRegionEditor({ path, values, writable, shared, sourceEpoch, onApply, disabled = false }: {
  path: string; values: Record<string, string>; writable: ReadonlySet<string>; shared: boolean;
  sourceEpoch: string | null; disabled?: boolean;
  onApply: (text: string, target: string, route: "shared" | "just-here") => Promise<unknown>;
}) {
  const parent = `worktree-region:${useId()}`;
  const epoch = JSON.stringify([sourceEpoch, path, values, [...writable].sort(), shared]);
  const currentEpoch = useRef(epoch); currentEpoch.current = epoch;
  const [draft, setDraft] = useState<{ epoch: string; model: ComparisonModel } | null>(null);
  const [route, setRoute] = useState<"shared" | "just-here">(shared ? "shared" : "just-here");
  const actualRoute = shared && route === "shared" ? "shared" : "just-here";
  const [busy, setBusy] = useState(false), [error, setError] = useState("");
  const [receipt, setReceipt] = useState<{ epoch: string; value: UiValue } | null>(null);
  const pending = useRef(false), mounted = useRef(true);
  const capturedId = useRef(`captured-input:${crypto.randomUUID()}`);
  const callbackRevision = useMemo(() => crypto.randomUUID(), [onApply]);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const sources = Object.entries(values).map(([name, text]) => ({ id: name, label: name, kind: "directory" as const, text, target: writable.has(name) ? name : undefined }));
  if (sources.some(source => source.id === capturedId.current)) capturedId.current = `captured-input:${crypto.randomUUID()}`;
  if (sources.length) sources.unshift({ ...sources[0], id: capturedId.current, label: "Captured input", target: undefined });
  const stale = !!draft && draft.epoch !== epoch;
  const model: ComparisonModel | null = draft?.model ?? (sources.length > 1 ? {
    sources, old: sources[0].id, new: sources[1].id, policy: writable.has(sources[1].id) && sourceEpoch ? "new" : "neither", evidence: "unverified",
  } : null);
  const displayed = model ? { ...model, policy: stale || busy || disabled || !sourceEpoch ? "neither" as const : model.policy } : null;
  const change = (next: ComparisonModel) => {
    if (pending.current || disabled || stale || !sourceEpoch || currentEpoch.current !== epoch) return;
    setDraft({ epoch, model: { ...next, policy: writable.has(next.new) ? "new" : "neither", evidence: "unverified" } }); setError("");
  };
  const setWriteRoute = (next: "shared" | "just-here") => {
    if (!pending.current && !disabled && !stale && (next === "just-here" || shared)) setRoute(next);
  };
  const reset = () => { if (!pending.current && !disabled) { setDraft(null); setError(""); } };
  const apply = async (): Promise<UiOutcome> => {
    if (!mounted.current || pending.current || disabled || !draft || stale || !sourceEpoch || currentEpoch.current !== epoch || !model || !writable.has(model.new))
      return { status: "unavailable", reason: "A current edited region, captured snapshot and writable target are required; reset stale input before applying" };
    const selected = model.sources.find(source => source.id === model.new && source.target === model.new);
    if (!selected) return { status: "unavailable", reason: "Selected comparison source is not a writable target" };
    pending.current = true; setBusy(true); setError("");
    try {
      const response = await onApply(selected.text, model.new, actualRoute);
      const value: UiValue = { path, target: model.new, route: actualRoute, response: response === undefined ? null : JSON.parse(JSON.stringify(response)), callback_completed: true,
        scope: "Actual region-apply callback reply. Inspect each target; writes may be partial and no rollback or independent verification is asserted." };
      if (mounted.current && currentEpoch.current === epoch) setReceipt({ epoch, value });
      return { status: "completed", value };
    } catch (cause) {
      const reason = `${String(cause)}. Inspect per-target status and disk before retrying; effects may be partial.`;
      if (mounted.current && currentEpoch.current === epoch) setError(reason);
      return { status: "unavailable", reason };
    } finally { pending.current = false; if (mounted.current) setBusy(false); }
  };
  const revision = useUiRevision(() => [epoch, draft, actualRoute, busy, disabled, callbackRevision]);
  const invoke = useUiCommands([
    { key: `${parent}:route`, command: { parent, subject: parent, label: "Set worktree region write route", parameters: { route: { type: "string", maxLength: 16, choices: shared ? ["shared", "just-here"] : ["just-here"] } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy && !disabled && !stale }), execute: async args => { setWriteRoute(args.route as typeof route); return { status: "completed", value: { route: args.route } }; } } },
    { key: `${parent}:reset`, command: { parent, subject: parent, label: "Reset worktree region draft to current input", parameters: {}, prerequisites: ["Discards this local region draft; does not undo host writes"],
      read: () => ({ revision: revision(), enabled: !busy && !disabled }), execute: async () => { reset(); return { status: "completed", value: { draft_reset: true, host_undo: false } }; } } },
    { key: `${parent}:apply`, command: { parent, subject: parent, label: "Apply worktree region draft", parameters: {}, prerequisites: ["Exact captured snapshot and target authorization remain checked by the existing host endpoint"],
      read: () => ({ revision: revision(), enabled: !busy && !disabled && !!draft && !stale && !!sourceEpoch && !!model && writable.has(model.new) }), execute: apply } },
  ]);
  const human = (id: string) => void invoke(`${parent}:${id}`).catch(cause => { if (mounted.current && currentEpoch.current === epoch) setError(String(cause)); });
  useUiReading("worktree region edit state", { path, route: actualRoute, shared, writable: [...writable], draft: !!draft, stale, busy, parent_disabled: disabled, error,
    snapshot_available: !!sourceEpoch, selected_new: model?.new ?? null, reply_available: receipt?.epoch === epoch,
    scope: "Comparison editors own source reading and local drafts. Applying may write multiple targets; per-target results remain separate from verification." });
  if (!displayed) return null;
  return <section aria-label="Worktree region edit">
    {stale && <p role="alert">The region input or write targets changed. This draft is retained read-only; reset it before editing or applying current input.</p>}
    {!sourceEpoch && <p role="alert">No captured host snapshot is available. Refresh the comparison before writing.</p>}
    <label>Write route <select value={actualRoute} disabled={busy || disabled || stale} onChange={event => setWriteRoute(event.target.value as typeof route)}>
      {shared && <option value="shared">Every target opened for writing</option>}<option value="just-here">Selected New target only</option>
    </select></label>
    <ComparisonEditor path={path} model={displayed} onChange={busy || disabled || stale || !sourceEpoch ? undefined : change} />
    <button className="btn" disabled={busy || disabled || !draft || stale || !sourceEpoch || !model || !writable.has(model.new)} onClick={() => human("apply")}>Apply region to selected targets</button>
    <button type="button" disabled={busy || disabled} onClick={() => human("reset")}>Reset region draft</button>
    {error && <p role="alert">{error}</p>}
    {receipt?.epoch === epoch && <EvidenceDisclosure label="Region apply reply" value={receipt.value}><pre>{JSON.stringify(receipt.value, null, 2)}</pre></EvidenceDisclosure>}
  </section>;
}
</book:entry>

</book:doc>
