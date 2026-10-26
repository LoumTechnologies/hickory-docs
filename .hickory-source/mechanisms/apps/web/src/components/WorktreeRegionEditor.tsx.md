<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/WorktreeRegionEditor.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b6ababcd2290"></a>
## `apps/web/src/components/WorktreeRegionEditor.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b6ababcd2290-WorktreeRegionEditor"></a>
### `WorktreeRegionEditor`

<storycode:copy id="f-b6ababcd2290-WorktreeRegionEditor">
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
</storycode:copy>

### Assemble `apps/web/src/components/WorktreeRegionEditor.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/WorktreeRegionEditor.tsx"><storycode:paste select="#f-b6ababcd2290-WorktreeRegionEditor"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-03a466adc939"></a>
## `apps/web/src/components/WorktreeRegionEditor.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-03a466adc939-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-03a466adc939-expression_statement">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { WorktreeRegionEditor } from "./WorktreeRegionEditor";
afterEach(cleanup);
</storycode:copy>
<a id="f-03a466adc939-tools"></a>
### `tools`

<storycode:copy id="f-03a466adc939-tools">
function tools() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label && action.enabled)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  return { registry, actions, run };
}
</storycode:copy>
<a id="f-03a466adc939-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-03a466adc939-expression_statement-2">
it("shares a real comparison draft, native apply and actual per-target callback replies without duplicate writes", async () => {
  const { registry, actions, run } = tools();
  let complete!: (result: unknown) => void;
  const apply = vi.fn(() => new Promise(resolve => { complete = resolve; }));
  render(<UiActionContext.Provider value={registry}><WorktreeRegionEditor path="file.txt" values={{ a: "old", b: "old" }} writable={new Set(["a", "b"])} shared sourceEpoch="snapshot-one:region-zero" onApply={apply} /></UiActionContext.Provider>);
  await waitFor(() => expect(actions().some(action => action.label === "Replace comparison draft range" && action.enabled)).toBe(true));
  expect(await run("Replace comparison draft range", { from: 0, to: 3, text: "edited" })).toMatchObject({ status: "result", outcome: { value: { draft_changed: true } } });
  await run("Set worktree region write route", { route: "just-here" });
  expect((screen.getByLabelText("Write route") as HTMLSelectElement).value).toBe("just-here");
  fireEvent.change(screen.getByLabelText("Write route"), { target: { value: "shared" } });
  await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Apply region to selected targets" })); });
  expect(apply).toHaveBeenCalledExactlyOnceWith("edited", "a", "shared");
  const applyAction = actions().find(action => action.label === "Apply worktree region draft")!;
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: applyAction.handle, arguments: {}, request_id: "duplicate" })).toMatchObject({ status: "disabled" });
  expect((screen.getByLabelText("Write route") as HTMLSelectElement).disabled).toBe(true);
  const actual = { targets: [{ source: "a", ok: true }, { source: "b", ok: false, error: "fixture conflict" }] };
  await act(async () => { complete(actual); });
  await run("Set evidence visibility: Region apply reply", { open: true });
  const reply = await run("Read disclosed evidence bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(JSON.parse(new TextDecoder().decode(Uint8Array.from(reply.outcome.value.bytes)))).toMatchObject({ response: actual, callback_completed: true });
});
</storycode:copy>
<a id="f-03a466adc939-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-03a466adc939-expression_statement-3">
it("retains a stale draft read-only and requires explicit reset before using a replaced snapshot", async () => {
  const { registry, actions, run } = tools();
  const oldApply = vi.fn(), newApply = vi.fn();
  const panel = (epoch: string, text: string, apply = oldApply) => <UiActionContext.Provider value={registry}><WorktreeRegionEditor path="file.txt" values={{ a: text }} writable={new Set(["a"])} shared={false} sourceEpoch={epoch} onApply={apply} /></UiActionContext.Provider>;
  const view = render(panel("first", "old"));
  await waitFor(() => expect(actions().some(action => action.label === "Replace comparison draft range" && action.enabled)).toBe(true));
  await run("Replace comparison draft range", { from: 0, to: 3, text: "keep this draft" });
  const old = actions().find(action => action.label === "Apply worktree region draft")!.handle;
  view.rerender(panel("second", "new", newApply));
  expect(screen.getByText(/draft is retained read-only/)).toBeTruthy();
  expect((screen.getByRole("button", { name: "Apply region to selected targets" }) as HTMLButtonElement).disabled).toBe(true);
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "old-snapshot" })).not.toMatchObject({ status: "result" });
  expect(newApply).not.toHaveBeenCalled(); expect(oldApply).not.toHaveBeenCalled();
  await run("Reset worktree region draft to current input");
  expect(screen.queryByText(/draft is retained read-only/)).toBeNull();
  expect(actions().some(action => action.label === "Replace comparison draft range" && action.enabled)).toBe(true);
  expect((screen.getByRole("button", { name: "Apply region to selected targets" }) as HTMLButtonElement).disabled).toBe(true);
});
</storycode:copy>

### Assemble `apps/web/src/components/WorktreeRegionEditor.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/WorktreeRegionEditor.test.tsx"><storycode:paste select="#f-03a466adc939-expression_statement"/><storycode:paste select="#f-03a466adc939-tools"/><storycode:paste select="#f-03a466adc939-expression_statement-2"/><storycode:paste select="#f-03a466adc939-expression_statement-3"/></storycode:file>


</storycode:doc>
