<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/uiActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0cc430d9eb03"></a>
## `apps/web/src/lib/uiActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-0cc430d9eb03-UiValue"></a>
### `UiValue`

Explicit semantic actions. Source text and DOM content never register commands.

<storycode:copy id="f-0cc430d9eb03-UiValue">
/** Explicit semantic actions. Source text and DOM content never register commands. */
export type UiValue = null | boolean | number | string | UiValue[] | { [key: string]: UiValue };
</storycode:copy>
<a id="f-0cc430d9eb03-UiField"></a>
### `UiField`

<storycode:copy id="f-0cc430d9eb03-UiField">
export type UiField =
  | { type: "string"; maxLength: number; choices?: string[] }
  | { type: "number"; minimum: number; maximum: number; integer?: boolean }
  | { type: "boolean" }
  | { type: "array"; items: UiField; maxItems: number };
</storycode:copy>
<a id="f-0cc430d9eb03-UiActor"></a>
### `UiActor`

<storycode:copy id="f-0cc430d9eb03-UiActor">
export type UiActor = { id: string; kind: "human" | "agent" };
</storycode:copy>
<a id="f-0cc430d9eb03-UiScope"></a>
### `UiScope`

<storycode:copy id="f-0cc430d9eb03-UiScope">
export type UiScope = { workspace: string; window: string };
</storycode:copy>
<a id="f-0cc430d9eb03-UiState"></a>
### `UiState`

<storycode:copy id="f-0cc430d9eb03-UiState">
export type UiState = {
  /** Covers every source/UI/run/stop revision relevant to this command. */
  revision: string;
  enabled: boolean;
  reason?: string;
  value?: UiValue;
};
</storycode:copy>
<a id="f-0cc430d9eb03-UiOutcome"></a>
### `UiOutcome`

<storycode:copy id="f-0cc430d9eb03-UiOutcome">
export type UiOutcome =
  | { status: "completed"; value: UiValue }
  | { status: "queued" | "running"; job: string; value?: UiValue }
  | { status: "requires_input" | "unavailable"; reason: string; request?: string };
</storycode:copy>
<a id="f-0cc430d9eb03-UiDescriptor"></a>
### `UiDescriptor`

<storycode:copy id="f-0cc430d9eb03-UiDescriptor">
export type UiDescriptor = {
  handle: string;
  label: string;
  subject: string;
  parent: string;
  parameters: Record<string, UiField>;
  prerequisites: string[];
} & UiState;
</storycode:copy>
<a id="f-0cc430d9eb03-UiCommand"></a>
### `UiCommand`

<storycode:copy id="f-0cc430d9eb03-UiCommand">
export type UiCommand = {
  label: string;
  subject: string;
  parent: string;
  parameters: Record<string, UiField>;
  prerequisites: string[];
  read: () => UiState;
  execute: (args: Record<string, UiValue>, actor: UiActor, signal: AbortSignal) => Promise<UiOutcome>;
};
</storycode:copy>
<a id="f-0cc430d9eb03-UiInvocation"></a>
### `UiInvocation`

<storycode:copy id="f-0cc430d9eb03-UiInvocation">
export type UiInvocation = { handle: string; arguments: Record<string, UiValue>; request_id: string };
</storycode:copy>
<a id="f-0cc430d9eb03-UiResult"></a>
### `UiResult`

<storycode:copy id="f-0cc430d9eb03-UiResult">
export type UiResult =
  | { status: "result"; outcome: UiOutcome }
  | { status: "stale" | "disabled" | "unavailable" | "forbidden" | "invalid" | "conflict" | "error"; reason: string; observation?: UiDescriptor };
</storycode:copy>
<a id="f-0cc430d9eb03-UiReceipt"></a>
### `UiReceipt`

<storycode:copy id="f-0cc430d9eb03-UiReceipt">
export type UiReceipt = {
  stage: "requested" | "result";
  request_id: string;
  actor: UiActor;
  scope: UiScope;
  action: UiDescriptor;
  arguments: Record<string, UiValue>;
  result?: UiResult;
};
</storycode:copy>
<a id="f-0cc430d9eb03-Entry"></a>
### `Entry`

<storycode:copy id="f-0cc430d9eb03-Entry">
type Entry = { key: string; command: UiCommand; descriptor?: UiDescriptor };

</storycode:copy>
<a id="f-0cc430d9eb03-canonical"></a>
### `canonical`

<storycode:copy id="f-0cc430d9eb03-canonical">
function canonical(value: unknown): string {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical((value as Record<string, unknown>)[key])}`).join(",")}}`;
}

</storycode:copy>
<a id="f-0cc430d9eb03-freezeTree"></a>
### `freezeTree`

<storycode:copy id="f-0cc430d9eb03-freezeTree">
function freezeTree<T>(value: T): T {
  const pending: unknown[] = [value];
  while (pending.length) {
    const item = pending.pop();
    if (!item || typeof item !== "object" || Object.isFrozen(item)) continue;
    Object.freeze(item);
    for (const child of Object.values(item)) if (child && typeof child === "object") pending.push(child);
  }
  return value;
}
</storycode:copy>
<a id="f-0cc430d9eb03-snapshot"></a>
### `snapshot`

Capture exactly the JSON representation retained/delivered by the protocol.

<storycode:copy id="f-0cc430d9eb03-snapshot">
/** Capture exactly the JSON representation retained/delivered by the protocol. */
function snapshot<T>(value: T): T { return freezeTree(JSON.parse(JSON.stringify(value)) as T); }

</storycode:copy>
<a id="f-0cc430d9eb03-valid"></a>
### `valid`

Require a JSON object with exactly the declared argument names before checking each field. Reject both omitted required arguments and undeclared extras; a caller cannot bypass the schema with an additional property.

<storycode:copy id="f-0cc430d9eb03-valid">
function valid(args: Record<string, UiValue>, fields: Record<string, UiField>): boolean {
  if (!args || Array.isArray(args) || typeof args !== "object") return false;
  if (Object.keys(args).length !== Object.keys(fields).length) return false;
  return Object.entries(fields).every(([name, field]) => {
    if (!Object.prototype.hasOwnProperty.call(args, name)) return false;
    const value = args[name];
    return validField(value, field);
  });
}
</storycode:copy>
<a id="f-0cc430d9eb03-validField"></a>
### `validField`

Apply each declared bound at runtime: string length counts Unicode code points, numbers must be finite and within range, integer fields reject fractional values, and arrays recurse through their item schema and maximum length.

<storycode:copy id="f-0cc430d9eb03-validField">
function validField(value: UiValue, field: UiField): boolean {
  switch (field.type) {
    case "string": return typeof value === "string" && [...value].length <= field.maxLength && (!field.choices || field.choices.includes(value));
    case "boolean": return typeof value === "boolean";
    case "number": return typeof value === "number" && Number.isFinite(value) && (!field.integer || Number.isInteger(value)) && value >= field.minimum && value <= field.maximum;
    case "array": return Array.isArray(value) && value.length <= field.maxItems && value.every(item => validField(item, field.items));
  }
}

</storycode:copy>
<a id="f-0cc430d9eb03-UiActions"></a>
### Bind a window

One registry belongs to one workspace/window scope. Separate maps retain active commands, disclosed handles and journaled requests; an action handle is a capability for a particular observed state, not a free-form command name.

<storycode:copy id="f-0cc430d9eb03-UiActions">
/** One host-bound window owns one registry; it is never installed on window. */
export class UiActions {
  private entries = new Map<string, Entry>();
  private handles = new Map<string, { entry: Entry; revision: string }>();
  private restrictions: ((key: string, command: UiCommand) => boolean)[] = [];
  private requests = new Map<string, { entry?: Entry; actor: UiActor; subject: string | null; settled: boolean; signature: string; result: Promise<UiResult>; controller: AbortController }>();
  private busy = new Set<string>();
  private sequence = 0;
  private observation = 0;
  private closed = false;
  private namespace = crypto.randomUUID();

  readonly scope: UiScope;

  constructor(
    scope: UiScope,
    private authorize: (actor: UiActor, command: UiDescriptor) => boolean,
    /** Must persist in the host session before reporting success. */
    private record: (receipt: UiReceipt) => Promise<void>,
    private maxRequests = 4096,
  ) { this.scope = Object.freeze({ ...scope }); }

</storycode:copy>
<a id="f-0cc430d9eb03-UiActions--register-copied-command-metadata"></a>
### Register copied command metadata

Copy parameter metadata at registration so later caller mutation cannot change an already disclosed schema. Unregister only the same entry and retire every handle referring to it.

<storycode:copy id="f-0cc430d9eb03-UiActions--register-copied-command-metadata">
  register(key: string, command: UiCommand): () => void {
    if (this.closed) throw Error("UI registry is closed");
    if (this.entries.has(key)) throw Error(`Duplicate UI command: ${key}`);
    // Copy metadata so a caller cannot mutate a disclosed schema in place.
    const entry: Entry = { key, command: { ...command, parameters: structuredClone(command.parameters), prerequisites: [...command.prerequisites] } };
    this.entries.set(key, entry);
    return () => {
      if (this.entries.get(key) !== entry) return;
      this.entries.delete(key);
      for (const [handle, target] of this.handles) if (target.entry === entry) this.handles.delete(handle);
    };
  }

</storycode:copy>
<a id="f-0cc430d9eb03-UiActions--retain-window-restrictions"></a>
### Retain window restrictions

Restrictions accumulate for the window lifetime. Checking all retained restrictions prevents a hidden or remounted widget from restoring a capability that the trusted host already confined.

<storycode:copy id="f-0cc430d9eb03-UiActions--retain-window-restrictions">
  /** Trusted host-only, monotonic restriction for this window's lifetime.
   * Hiding/restarting a widget cannot reopen prior capabilities or cached results.
   */
  confine(allow: (key: string, command: UiCommand) => boolean): void { this.restrictions.push(allow); }
  private permitted(entry: Entry): boolean { return this.restrictions.every(allow => allow(entry.key, entry.command)); }

</storycode:copy>
<a id="f-0cc430d9eb03-UiActions--refresh-handles-when-state-changes"></a>
### Refresh handles when state changes

An observed handle remains usable only while its revision, enabled state, reason and JSON value agree. Capture the descriptor as immutable JSON and return a clone so consumers cannot mutate the retained observation.

<storycode:copy id="f-0cc430d9eb03-UiActions--refresh-handles-when-state-changes">
  descriptor(key: string): UiDescriptor | null {
    const entry = this.entries.get(key);
    return entry && this.permitted(entry) ? this.describe(entry) : null;
  }

  groups(limit = 32, offset = 0) {
    if (!Number.isInteger(limit) || limit < 1 || limit > 64 || !Number.isInteger(offset) || offset < 0) throw Error("Invalid observation bounds");
    const parents = [...new Set([...this.entries.values()].filter(entry => this.permitted(entry)).map(e => e.command.parent))];
    return { observation: `observation:${this.namespace}:${++this.observation}`, scope: { ...this.scope }, groups: parents.slice(offset, offset + limit), total: parents.length, next: offset + limit < parents.length ? offset + limit : null, coverage: "Only registered controls; remaining UI parity is tracked in #74/#75", edit_baseline: false };
  }

  private describe(entry: Entry): UiDescriptor {
    const state = entry.command.read();
    const previous = entry.descriptor;
    if (!previous || previous.revision !== state.revision || previous.enabled !== state.enabled || previous.reason !== state.reason || canonical(previous.value) !== canonical(state.value)) {
      if (previous) this.handles.delete(previous.handle);
      const handle = `action:${this.namespace}:${(++this.sequence).toString(36)}`;
      entry.descriptor = snapshot({ handle, label: entry.command.label, subject: entry.command.subject, parent: entry.command.parent, parameters: entry.command.parameters, prerequisites: entry.command.prerequisites, ...state });
      this.handles.set(handle, { entry, revision: state.revision });
    }
    return structuredClone(entry.descriptor!);
  }

</storycode:copy>
<a id="f-0cc430d9eb03-UiActions--observe-an-explicit-parent"></a>
### Observe an explicit parent

Bound observations and require an explicit parent. Disclose only registered, permitted entries in that parent; scrolling is not an authorization to inspect more UI state.

<storycode:copy id="f-0cc430d9eb03-UiActions--observe-an-explicit-parent">
  /** Reads exactly one explicitly requested parent; scrolling never calls this. */
  observe(parent: string, limit = 32, offset = 0) {
    if (!Number.isInteger(limit) || limit < 1 || limit > 64 || !Number.isInteger(offset) || offset < 0) throw Error("Invalid observation bounds");
    const entries = [...this.entries.values()].filter(e => e.command.parent === parent && this.permitted(e));
    return {
      observation: `observation:${this.namespace}:${++this.observation}`,
      scope: { ...this.scope }, parent, offset,
      actions: entries.slice(offset, offset + limit).map(e => this.describe(e)),
      coverage: { disclosed: Math.min(limit, Math.max(0, entries.length - offset)), total: entries.length },
      next: offset + limit < entries.length ? offset + limit : null,
      edit_baseline: false as const,
    };
  }

</storycode:copy>
<a id="f-0cc430d9eb03-UiActions--journal-before-execution"></a>
### Journal before execution

Take actor and scope from the trusted host. Reject cross-window invocations, require bounded request identifiers, and bind each identifier to one canonical actor/invocation signature. Journal the promise before executing even synchronous callbacks, so a repeated request returns the same result instead of repeating its effects.

<storycode:copy id="f-0cc430d9eb03-UiActions--journal-before-execution">
  /** Actor and scope come from the trusted host adapter, never model arguments. */
  act(scope: UiScope, actor: UiActor, invocation: UiInvocation): Promise<UiResult> {
    if (this.closed) return Promise.resolve({ status: "unavailable", reason: "Originating UI window is closed" });
    if (scope.window !== this.scope.window || scope.workspace !== this.scope.workspace) return Promise.resolve({ status: "forbidden", reason: "Action belongs to another workspace/window" });
    if (!invocation || typeof invocation.handle !== "string" || typeof invocation.request_id !== "string" || invocation.request_id.length < 1 || invocation.request_id.length > 128) return Promise.resolve({ status: "invalid", reason: "A bounded request_id and action handle are required" });
    let signature: string;
    try { signature = canonical({ actor, invocation }); } catch { return Promise.resolve({ status: "invalid", reason: "Arguments must be JSON" }); }
    const prior = this.requests.get(invocation.request_id);
    if (prior?.entry && !this.permitted(prior.entry)) return Promise.resolve({ status: "forbidden", reason: "Originating window is confined; prior results remain withheld" });
    if (prior) return prior.signature === signature ? prior.result : Promise.resolve({ status: "conflict", reason: "request_id already used with different arguments or actor" });
    if (this.requests.size >= this.maxRequests) return Promise.resolve({ status: "unavailable", reason: "Window request journal is full; retain receipts and establish a new session" });
    const controller = new AbortController();
    // Defer execution until the request is journaled, including synchronous callbacks.
    const captured = freezeTree(structuredClone(invocation));
    const capturedActor = Object.freeze({ ...actor });
    const capturedEntry = this.handles.get(captured.handle)?.entry;
    const result = Promise.resolve().then(() => this.execute(capturedActor, captured, controller.signal)).then(result => Object.isFrozen(result) ? result : snapshot(result)).catch((error): UiResult => Object.freeze({ status: "error", reason: String(error) })).then(result => capturedEntry && !this.permitted(capturedEntry) && !["forbidden", "unavailable", "invalid", "conflict"].includes(result.status) ? snapshot<UiResult>({ status: "unavailable", reason: "Window became confined; this result remains withheld" }) : result).finally(() => { this.requests.get(captured.request_id)!.settled = true; });
    this.requests.set(invocation.request_id, { entry: this.handles.get(captured.handle)?.entry, actor: capturedActor, subject: this.handles.get(captured.handle)?.entry.command.subject ?? null, settled: false, signature, result, controller });
    return result;
  }

</storycode:copy>
<a id="f-0cc430d9eb03-UiActions--cancel-with-the-original-actor"></a>
### Cancel with the original actor

Expose pending work only for the recorded actor. Cancellation requires the same scope and actor and aborts the retained controller. Closing the registry retires its entries and aborts unsettled work; cancellation does not claim to undo completed effects.

<storycode:copy id="f-0cc430d9eb03-UiActions--cancel-with-the-original-actor">
  /** Trusted UI chrome may retain its controller until a click's journaled actions settle. */
  pending(actor: UiActor, excludingSubject?: string): { id: string; result: Promise<UiResult> }[] {
    return [...this.requests].filter(([, request]) => !request.settled && (excludingSubject === undefined || request.subject !== excludingSubject) && request.actor.id === actor.id && request.actor.kind === actor.kind).map(([id, request]) => ({ id, result: request.result }));
  }

  cancel(scope: UiScope, actor: UiActor, request_id: string): boolean {
    if (scope.workspace !== this.scope.workspace || scope.window !== this.scope.window) return false;
    const pending = this.requests.get(request_id);
    if (!pending || pending.settled) return false;
    // Ownership is checked against the host actor recorded at invocation.
    if (pending.actor.id !== actor.id || pending.actor.kind !== actor.kind) return false;
    pending.controller.abort();
    return true;
  }

  dispose(): void {
    this.closed = true;
    for (const request of this.requests.values()) if (!request.settled) request.controller.abort();
    this.entries.clear();
    this.handles.clear();
  }

</storycode:copy>
<a id="f-0cc430d9eb03-UiActions--recheck-around-durable-intent"></a>
### Recheck around durable intent

Require a live disclosed handle, unchanged state, current actor permission and valid arguments before recording intent. Recheck registration, confinement, state and permission after that await. Capture and retain the result separately: if recording fails after effects, report that uncertainty rather than suggesting a safe automatic retry.

<storycode:copy id="f-0cc430d9eb03-UiActions--recheck-around-durable-intent">
  private async execute(actor: UiActor, invocation: UiInvocation, signal: AbortSignal): Promise<UiResult> {
    const target = this.handles.get(invocation.handle);
    if (!target || ![...this.entries.values()].includes(target.entry)) return { status: "unavailable", reason: "Unknown, retired, or undisclosed action; observe its parent again" };
    if (!this.permitted(target.entry)) return { status: "forbidden", reason: "Originating window is confined; this capability is unavailable" };
    const action = this.describe(target.entry);
    if (action.handle !== invocation.handle) return { status: "stale", reason: "State changed; inspect refreshed action", observation: action };
    if (!this.authorize(actor, action)) return { status: "forbidden", reason: "Current actor policy denies this command" };
    if (!action.enabled) return { status: "disabled", reason: action.reason ?? "Command disabled", observation: action };
    if (!valid(invocation.arguments, action.parameters)) return { status: "invalid", reason: "Arguments do not match the disclosed schema" };
    if (this.busy.has(action.subject)) return { status: "conflict", reason: "Another action is executing on this subject" };
    if (signal.aborted) return { status: "unavailable", reason: "Cancelled before execution" };
    this.busy.add(action.subject);
    let result: UiResult;
    try {
      // Persist intent before invoking any side effect. Failure closes the gate.
      try {
        await this.record({ stage: "requested", request_id: invocation.request_id, actor, scope: this.scope, action, arguments: invocation.arguments });
      } catch (error) {
        // Do not overwrite an intent-retention failure with the later, less
        // useful "result" failure. The handler has not run, so callers can
        // safely repair the host connection and submit a new request.
        return Object.freeze({ status: "error", reason: `Could not retain the action intent; no action was submitted: ${String(error)}` });
      }
      if (signal.aborted) result = { status: "unavailable", reason: "Cancelled before execution" };
      else if (![...this.entries.values()].includes(target.entry)) result = { status: "unavailable", reason: "Command was unregistered before execution" };
      else if (!this.permitted(target.entry)) result = { status: "forbidden", reason: "Originating window became confined before execution" };
      else {
        const current = this.describe(target.entry);
        if (current.handle !== action.handle) result = { status: "stale", reason: "State changed while recording intent", observation: current };
        else if (!this.authorize(actor, current)) result = { status: "forbidden", reason: "Current actor policy denies this command" };
        else result = { status: "result", outcome: await target.entry.command.execute(invocation.arguments, actor, signal) };
      }
    } catch (error) {
      result = { status: "error", reason: String(error) };
    }
    if (!this.permitted(target.entry)) result = { status: "unavailable", reason: "Window became confined; effects may have occurred but this result remains withheld" };
    try { result = snapshot(result); }
    catch { result = Object.freeze({ status: "error", reason: "Could not capture the JSON action result; side effects may have occurred. Inspect current state before a new request." }); }
    try {
      await this.record({ stage: "result", request_id: invocation.request_id, actor, scope: this.scope, action, arguments: invocation.arguments, result });
    } catch (error) {
      result = { status: "error", reason: `Could not retain the action result: ${String(error)}. Side effects may have occurred. Inspect current state before a new request.` };
    } finally { this.busy.delete(action.subject); }
    return result;
  }
}
</storycode:copy>


### Assemble `apps/web/src/lib/uiActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiActions.ts"><storycode:paste select="#f-0cc430d9eb03-UiValue"/><storycode:paste select="#f-0cc430d9eb03-UiField"/><storycode:paste select="#f-0cc430d9eb03-UiActor"/><storycode:paste select="#f-0cc430d9eb03-UiScope"/><storycode:paste select="#f-0cc430d9eb03-UiState"/><storycode:paste select="#f-0cc430d9eb03-UiOutcome"/><storycode:paste select="#f-0cc430d9eb03-UiDescriptor"/><storycode:paste select="#f-0cc430d9eb03-UiCommand"/><storycode:paste select="#f-0cc430d9eb03-UiInvocation"/><storycode:paste select="#f-0cc430d9eb03-UiResult"/><storycode:paste select="#f-0cc430d9eb03-UiReceipt"/><storycode:paste select="#f-0cc430d9eb03-Entry"/><storycode:paste select="#f-0cc430d9eb03-canonical"/><storycode:paste select="#f-0cc430d9eb03-freezeTree"/><storycode:paste select="#f-0cc430d9eb03-snapshot"/><storycode:paste select="#f-0cc430d9eb03-valid"/><storycode:paste select="#f-0cc430d9eb03-validField"/><storycode:paste select="#f-0cc430d9eb03-UiActions"/><storycode:paste select="#f-0cc430d9eb03-UiActions--register-copied-command-metadata"/><storycode:paste select="#f-0cc430d9eb03-UiActions--retain-window-restrictions"/><storycode:paste select="#f-0cc430d9eb03-UiActions--refresh-handles-when-state-changes"/><storycode:paste select="#f-0cc430d9eb03-UiActions--observe-an-explicit-parent"/><storycode:paste select="#f-0cc430d9eb03-UiActions--journal-before-execution"/><storycode:paste select="#f-0cc430d9eb03-UiActions--cancel-with-the-original-actor"/><storycode:paste select="#f-0cc430d9eb03-UiActions--recheck-around-durable-intent"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-f5b85cef7913"></a>
## `apps/web/src/lib/uiActions.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md).

<a id="f-f5b85cef7913-scope"></a>
### `scope`

<storycode:copy id="f-f5b85cef7913-scope">
import { describe, expect, it, vi } from "vitest";
import { UiActions, type UiCommand, type UiReceipt, type UiState, type UiValue } from "./uiActions";

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
const scope = { workspace: "practice", window: "one" };
</storycode:copy>
<a id="f-f5b85cef7913-actor"></a>
### `actor`

<storycode:copy id="f-f5b85cef7913-actor">
const actor = { id: "python:turn1", kind: "agent" as const };
</storycode:copy>
<a id="f-f5b85cef7913-fixture"></a>
### `fixture`

<storycode:copy id="f-f5b85cef7913-fixture">
function fixture() {
  let state: UiState = { revision: "source:1/stop:1", enabled: true, value: "λ😀" };
  const receipts: UiReceipt[] = [];
  const execute = vi.fn<UiCommand["execute"]>().mockResolvedValue({ status: "completed", value: "executed" });
  const policy = vi.fn(() => true);
  const registry = new UiActions(scope, policy, async receipt => { receipts.push(receipt); });
  const command: UiCommand = { label: "Run", subject: "deadline.md#one", parent: "editor", parameters: {}, prerequisites: ["saved source"], read: () => state, execute };
  const unregister = registry.register("run-one", command);
  const handle = registry.observe("editor").actions[0].handle;
  const act = (request_id = "request-1", arguments_ = {}) => registry.act(scope, actor, { handle, arguments: arguments_, request_id });
  return { registry, execute, policy, receipts, command, unregister, handle, act, state: (next: Partial<UiState>) => { state = { ...state, ...next }; } };
}

</storycode:copy>
<a id="f-f5b85cef7913-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f5b85cef7913-expression_statement">
describe("explicit UI command dispatch", () => {
  it("validates integer and bounded array arguments before terminal dispatch", async () => {
    const f = fixture();
    f.registry.register("argv", { ...f.command, parent: "terminal", parameters: { argv: { type: "array", maxItems: 2, items: { type: "string", maxLength: 2 } }, offset: { type: "number", integer: true, minimum: 0, maximum: 100 } } });
    const handle = f.registry.observe("terminal").actions[0].handle;
    const invoke = (arguments_: Record<string, UiValue>, request_id: string) => f.registry.act(scope, actor, { handle, arguments: arguments_, request_id });
    for (const [i, argv] of [["abc"], ["a", "b", "c"], [1], "a"].entries()) {
      expect(await invoke({ argv, offset: 0 }, String(i))).toMatchObject({ status: "invalid" });
    }
    expect(await invoke({ argv: ["λ😀"], offset: 0.5 }, "fraction")).toMatchObject({ status: "invalid" });
    expect(f.execute).not.toHaveBeenCalled();
    expect(await invoke({ argv: ["λ😀"], offset: 0 }, "valid-array")).toMatchObject({ status: "result" });
    expect(f.execute).toHaveBeenCalledOnce();
  });
  it("discloses only the requested parent and invokes the registered semantic handler", async () => {
    const f = fixture();
    expect(f.registry.observe("closed menu").actions).toEqual([]);
    const view = f.registry.observe("editor");
    expect(view.actions[0]).toMatchObject({ label: "Run", subject: "deadline.md#one", value: "λ😀", enabled: true });
    expect(view.edit_baseline).toBe(false);
    expect(await f.act()).toEqual({ status: "result", outcome: { status: "completed", value: "executed" } });
    expect(f.execute).toHaveBeenCalledOnce();
    expect(f.execute.mock.calls[0][1]).toEqual(actor);
    expect(f.receipts.map(r => r.stage)).toEqual(["requested", "result"]);
    expect(f.receipts[0].result).toBeUndefined();
    expect(f.registry.observe("editor").observation).not.toBe(view.observation);
  });
  it("distinguishes duplicate labels by subject and bounds disclosure", () => {
    const f = fixture();
    f.registry.register("run-two", { ...f.command, subject: "deadline.md#two" });
    const first = f.registry.observe("editor", 1);
    const second = f.registry.observe("editor", 1, first.next!);
    expect(first.coverage).toEqual({ disclosed: 1, total: 2 });
    expect(first.actions[0].handle).not.toBe(second.actions[0].handle);
    expect(second.actions[0].subject).toBe("deadline.md#two");
    expect(() => f.registry.observe("editor", 10000)).toThrow();
  });
  it("rejects stale source/stop and returns a refreshed bounded descriptor", async () => {
    const f = fixture(); f.state({ revision: "source:2/stop:2" });
    expect(await f.act()).toMatchObject({ status: "stale", observation: { revision: "source:2/stop:2" } });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("checks enabled state and policy at invocation", async () => {
    const f = fixture(); f.policy.mockReturnValue(false);
    expect(await f.act()).toMatchObject({ status: "forbidden" });
    f.policy.mockReturnValue(true); f.state({ enabled: false, reason: "Save first" });
    const handle = f.registry.observe("editor").actions[0].handle;
    expect(await f.registry.act(scope, actor, { handle, arguments: {}, request_id: "disabled" })).toMatchObject({ status: "disabled", reason: "Save first" });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("never resolves invented, quoted, cross-window or unregistered references", async () => {
    const f = fixture();
    for (const handle of ["action:forged", `Run [${f.handle}]`, JSON.stringify(f.handle)]) {
      expect(await f.registry.act(scope, actor, { handle, arguments: {}, request_id: handle })).toMatchObject({ status: "unavailable" });
    }
    expect(await f.registry.act({ ...scope, window: "two" }, actor, { handle: f.handle, arguments: {}, request_id: "other-window" })).toMatchObject({ status: "forbidden" });
    const other = new UiActions(scope, () => true, async () => {});
    expect(await other.act(scope, actor, { handle: f.handle, arguments: {}, request_id: "other-registry" })).toMatchObject({ status: "unavailable" });
    f.unregister(); expect(await f.act()).toMatchObject({ status: "unavailable" });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("validates typed inputs, required keys and unknown parameters", async () => {
    const f = fixture();
    f.registry.register("input", { ...f.command, parent: "dialog", parameters: { text: { type: "string", maxLength: 4 }, count: { type: "number", minimum: 1, maximum: 3 }, enabled: { type: "boolean" } } });
    const handle = f.registry.observe("dialog").actions[0].handle;
    const invalid: Record<string, UiValue>[] = [{}, { text: "abcde", count: 1, enabled: true }, { text: "ok", count: Infinity, enabled: true }, { text: "ok", count: 1, enabled: "true" }, { text: "ok", count: 1, enabled: true, extra: 1 }];
    for (const [i, arguments_] of invalid.entries()) {
      expect(await f.registry.act(scope, actor, { handle, arguments: arguments_, request_id: String(i) })).toMatchObject({ status: "invalid" });
    }
    expect(await f.registry.act(scope, actor, { handle, arguments: { text: "λ😀", count: 2, enabled: false }, request_id: "valid" })).toMatchObject({ status: "result" });
    expect(f.execute).toHaveBeenCalledOnce();
  });
  it("deduplicates retries while allowing intentional repeat runs and rejects key reuse", async () => {
    const f = fixture();
    await Promise.all([f.act(), f.act()]); expect(f.execute).toHaveBeenCalledOnce();
    expect(await f.act("request-1", { unexpected: 1 })).toMatchObject({ status: "conflict" });
    await f.act("intentional-repeat"); expect(f.execute).toHaveBeenCalledTimes(2);
  });
  it("cancels only owned requests and passes the signal to the real handler", async () => {
    const f = fixture();
    f.execute.mockImplementation(async (_args, _actor, signal) => {
      await new Promise<void>(resolve => signal.addEventListener("abort", () => resolve(), { once: true }));
      return { status: "unavailable", reason: "Cancelled by executor" };
    });
    const pending = f.act();
    await vi.waitFor(() => expect(f.execute).toHaveBeenCalledOnce());
    expect(f.registry.cancel(scope, { id: "other", kind: "agent" }, "request-1")).toBe(false);
    expect(f.registry.cancel(scope, actor, "request-1")).toBe(true);
    expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  });
  it("serializes actions on the same subject and captures parameters before asynchronous execution", async () => {
    const f = fixture(); let release!: () => void;
    f.execute.mockImplementation(async () => { await new Promise<void>(resolve => { release = resolve; }); return { status: "completed", value: null }; });
    const arguments_ = {}; const pending = f.act("one", arguments_);
    Object.assign(arguments_, { injected: true });
    await vi.waitFor(() => expect(f.execute).toHaveBeenCalledOnce());
    expect(await f.act("two")).toMatchObject({ status: "conflict" });
    expect(f.execute.mock.calls[0][0]).toEqual({});
    release(); await pending;
  });
  it("rechecks state and permission after retaining intent", async () => {
    let revision = "one";
    const execute = vi.fn<UiCommand["execute"]>();
    const registry = new UiActions(scope, () => true, async receipt => { if (receipt.stage === "requested") revision = "two"; });
    registry.register("run", { ...fixture().command, read: () => ({ revision, enabled: true }), execute });
    const handle = registry.observe("editor").actions[0].handle;
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r" })).toMatchObject({ status: "stale" });
    expect(execute).not.toHaveBeenCalled();
  });
  it("fails closed when intent cannot be retained and does not invent a completed result", async () => {
    const f = fixture();
    const registry = new UiActions(scope, () => true, async () => { throw Error("disk full"); });
    registry.register("run", f.command);
    const handle = registry.observe("editor").actions[0].handle;
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r" })).toMatchObject({ status: "error", reason: "Could not retain the action intent; no action was submitted: Error: disk full" });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("cannot revive a command unregistered while intent is being retained", async () => {
    const f = fixture(); let remove!: () => void;
    const registry = new UiActions(scope, () => true, async receipt => { if (receipt.stage === "requested") remove(); });
    remove = registry.register("run", f.command);
    const handle = registry.observe("editor").actions[0].handle;
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r" })).toMatchObject({ status: "unavailable" });
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r2" })).toMatchObject({ status: "unavailable" });
    expect(registry.observe("editor").actions).toEqual([]);
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("honors revoked policy after asynchronous intent persistence", async () => {
    const f = fixture(); let permitted = true;
    const registry = new UiActions(scope, () => permitted, async receipt => { if (receipt.stage === "requested") permitted = false; });
    registry.register("run", f.command);
    const handle = registry.observe("editor").actions[0].handle;
    expect(await registry.act(scope, actor, { handle, arguments: {}, request_id: "r" })).toMatchObject({ status: "forbidden" });
    expect(f.execute).not.toHaveBeenCalled();
  });
  it("does not execute a request cancelled before its handler starts", async () => {
    const f = fixture(); const pending = f.act();
    expect(f.registry.cancel(scope, actor, "request-1")).toBe(true);
    expect(await pending).toMatchObject({ status: "unavailable" });
    expect(f.execute).not.toHaveBeenCalled();
    expect(f.registry.cancel(scope, actor, "request-1")).toBe(false);
  });
  it("retains uncertain-result errors and never repeats completed side effects", async () => {
    const f = fixture();
    const registry = new UiActions(scope, () => true, async receipt => { if (receipt.stage === "result") throw Error("disk full"); });
    registry.register("run", f.command);
    const invocation = { handle: registry.observe("editor").actions[0].handle, arguments: {}, request_id: "r" };
    expect(await registry.act(scope, actor, invocation)).toMatchObject({ status: "error", reason: expect.stringContaining("Error: disk full") });
    await registry.act(scope, actor, invocation);
    expect(f.execute).toHaveBeenCalledOnce();
  });
  it("refuses new requests at the journal limit without evicting retry protection", async () => {
    const f = fixture(); const registry = new UiActions(scope, () => true, async () => {}, 1);
    registry.register("run", f.command);
    const invocation = { handle: registry.observe("editor").actions[0].handle, arguments: {}, request_id: "r" };
    await registry.act(scope, actor, invocation);
    expect(await registry.act(scope, actor, { ...invocation, request_id: "r2" })).toMatchObject({ status: "unavailable" });
    expect(await registry.act(scope, actor, invocation)).toMatchObject({ status: "result" });
    expect(f.execute).toHaveBeenCalledOnce();
  });
  it("disposes a closed window without dispatching queued handlers", async () => {
    const f = fixture(); const pending = f.act();
    f.registry.dispose();
    expect(await pending).toMatchObject({ status: "unavailable" });
    expect(await f.act("after-close")).toMatchObject({ status: "unavailable" });
    expect(f.registry.groups().groups).toEqual([]);
    expect(() => f.registry.register("new", f.command)).toThrow("closed");
    expect(f.execute).not.toHaveBeenCalled();
  });
});

</storycode:copy>
<a id="f-f5b85cef7913-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f5b85cef7913-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("snapshots mutable outcome data before retention and prevents caller tampering with retries", async () => {
  const f = fixture(), live = { items: [{ state: "before" }] };
  f.execute.mockResolvedValue({ status: "completed", value: live });
  const pending = f.act();
  const result = await pending;
  live.items[0].state = "after";
  expect(result).toMatchObject({ status: "result", outcome: { value: { items: [{ state: "before" }] } } });
  if (result.status !== "result" || result.outcome.status !== "completed") throw Error("Expected completed snapshot");
  const delivered = result.outcome.value as typeof live;
  expect(() => { delivered.items[0].state = "spoofed"; }).toThrow(TypeError);
  expect(f.act()).toBe(pending);
  expect(await f.act()).toEqual(result);
  expect(f.receipts[1].result).toEqual(result);
  expect(f.execute).toHaveBeenCalledOnce();
});

</storycode:copy>
<a id="f-f5b85cef7913-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-f5b85cef7913-expression_statement-3">
it("refuses a known handle when mutable disclosed metadata changed without a manual revision bump", async () => {
  const f = fixture(), value = { count: 1 };
  f.state({ value });
  const handle = f.registry.observe("editor").actions[0].handle;
  value.count = 2;
  expect(await f.registry.act(scope, actor, { handle, arguments: {}, request_id: "changed-metadata" })).toMatchObject({ status: "stale", observation: { value: { count: 2 } } });
  expect(f.execute).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-f5b85cef7913-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-f5b85cef7913-expression_statement-4">
it("retains an uncertain result and refuses to repeat side effects when an executor returns non-serializable data", async () => {
  const f = fixture(), value: Record<string, UiValue> = {}; value.self = value;
  f.execute.mockResolvedValue({ status: "completed", value });
  expect(await f.act()).toMatchObject({ status: "error", reason: expect.stringContaining("side effects may have occurred") });
  expect(await f.act()).toMatchObject({ status: "error" });
  expect(f.execute).toHaveBeenCalledOnce();
  expect(f.receipts.map(r => r.stage)).toEqual(["requested", "result"]);
});

</storycode:copy>
<a id="f-f5b85cef7913-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-f5b85cef7913-expression_statement-5">
it("monotonically confines disclosure, undispatched actions and cached ordinary results", async () => {
  const f = fixture();
  await f.act("before-confinement");
  f.registry.register("safe", { ...f.command, parent: "private", subject: "private", label: "Private metadata", execute: async () => ({ status: "completed", value: "withheld" }) });
  f.registry.confine((_key, command) => command.parent === "private");
  expect(f.registry.groups().groups).toEqual(["private"]);
  expect(f.registry.observe("editor").actions).toEqual([]);
  expect(f.registry.descriptor("run-one")).toBeNull();
  expect(await f.act("before-confinement")).toMatchObject({ status: "forbidden" });
  expect(await f.act("after-confinement")).toMatchObject({ status: "forbidden" });
  expect(f.execute).toHaveBeenCalledOnce();
  f.registry.confine(() => true); // A later permissive guard cannot reopen it.
  expect(f.registry.observe("editor").actions).toEqual([]);
});

</storycode:copy>
<a id="f-f5b85cef7913-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-f5b85cef7913-expression_statement-6">
it("withholds an ordinary in-flight result and its retained result receipt when confinement begins", async () => {
  const f = fixture(); let release!: () => void;
  f.execute.mockImplementation(async () => { await new Promise<void>(resolve => { release = resolve; }); return { status: "completed", value: "held-out-secret" }; });
  const pending = f.act("in-flight");
  await vi.waitFor(() => expect(release).toBeTypeOf("function"));
  f.registry.confine(() => false); release();
  const result = await pending;
  expect(result).toMatchObject({ status: "unavailable" });
  expect(JSON.stringify(result)).not.toContain("held-out-secret");
  expect(JSON.stringify(f.receipts.find(receipt => receipt.stage === "result"))).not.toContain("held-out-secret");
  expect(await f.act("in-flight")).toMatchObject({ status: "forbidden" });
});

</storycode:copy>
<a id="f-f5b85cef7913-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-f5b85cef7913-expression_statement-7">
it("refuses an ordinary handler if confinement starts during intent persistence", async () => {
  const f = fixture(); let release!: () => void;
  const registry = new UiActions(scope, () => true, receipt => receipt.stage === "requested" ? new Promise<void>(resolve => { release = resolve; }) : Promise.resolve());
  registry.register("run", f.command);
  const handle = registry.observe("editor").actions[0].handle;
  const pending = registry.act(scope, actor, { handle, arguments: {}, request_id: "pending-intent" });
  await vi.waitFor(() => expect(release).toBeTypeOf("function"));
  registry.confine(() => false); release();
  expect(await pending).toMatchObject({ status: "unavailable" });
  expect(f.execute).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/lib/uiActions.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiActions.test.ts"><storycode:paste select="#f-f5b85cef7913-scope"/><storycode:paste select="#f-f5b85cef7913-actor"/><storycode:paste select="#f-f5b85cef7913-fixture"/><storycode:paste select="#f-f5b85cef7913-expression_statement"/><storycode:paste select="#f-f5b85cef7913-expression_statement-2"/><storycode:paste select="#f-f5b85cef7913-expression_statement-3"/><storycode:paste select="#f-f5b85cef7913-expression_statement-4"/><storycode:paste select="#f-f5b85cef7913-expression_statement-5"/><storycode:paste select="#f-f5b85cef7913-expression_statement-6"/><storycode:paste select="#f-f5b85cef7913-expression_statement-7"/></storycode:file>


</storycode:doc>
