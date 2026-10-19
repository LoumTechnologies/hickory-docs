/** Explicit semantic actions. Source text and DOM content never register commands. */
export type UiValue = null | boolean | number | string | UiValue[] | { [key: string]: UiValue };
export type UiField =
  | { type: "string"; maxLength: number; choices?: string[] }
  | { type: "number"; minimum: number; maximum: number; integer?: boolean }
  | { type: "boolean" }
  | { type: "array"; items: UiField; maxItems: number };
export type UiActor = { id: string; kind: "human" | "agent" };
export type UiScope = { workspace: string; window: string };
export type UiState = {
  /** Covers every source/UI/run/stop revision relevant to this command. */
  revision: string;
  enabled: boolean;
  reason?: string;
  value?: UiValue;
};
export type UiOutcome =
  | { status: "completed"; value: UiValue }
  | { status: "queued" | "running"; job: string; value?: UiValue }
  | { status: "requires_input" | "unavailable"; reason: string; request?: string };
export type UiDescriptor = {
  handle: string;
  label: string;
  subject: string;
  parent: string;
  parameters: Record<string, UiField>;
  prerequisites: string[];
} & UiState;
export type UiCommand = {
  label: string;
  subject: string;
  parent: string;
  parameters: Record<string, UiField>;
  prerequisites: string[];
  read: () => UiState;
  execute: (args: Record<string, UiValue>, actor: UiActor, signal: AbortSignal) => Promise<UiOutcome>;
};
export type UiInvocation = { handle: string; arguments: Record<string, UiValue>; request_id: string };
export type UiResult =
  | { status: "result"; outcome: UiOutcome }
  | { status: "stale" | "disabled" | "unavailable" | "forbidden" | "invalid" | "conflict" | "error"; reason: string; observation?: UiDescriptor };
export type UiReceipt = {
  stage: "requested" | "result";
  request_id: string;
  actor: UiActor;
  scope: UiScope;
  action: UiDescriptor;
  arguments: Record<string, UiValue>;
  result?: UiResult;
};
type Entry = { command: UiCommand; descriptor?: UiDescriptor };

function canonical(value: unknown): string {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical((value as Record<string, unknown>)[key])}`).join(",")}}`;
}

function valid(args: Record<string, UiValue>, fields: Record<string, UiField>): boolean {
  if (!args || Array.isArray(args) || typeof args !== "object") return false;
  if (Object.keys(args).length !== Object.keys(fields).length) return false;
  return Object.entries(fields).every(([name, field]) => {
    if (!Object.prototype.hasOwnProperty.call(args, name)) return false;
    const value = args[name];
    return validField(value, field);
  });
}
function validField(value: UiValue, field: UiField): boolean {
  switch (field.type) {
    case "string": return typeof value === "string" && [...value].length <= field.maxLength && (!field.choices || field.choices.includes(value));
    case "boolean": return typeof value === "boolean";
    case "number": return typeof value === "number" && Number.isFinite(value) && (!field.integer || Number.isInteger(value)) && value >= field.minimum && value <= field.maximum;
    case "array": return Array.isArray(value) && value.length <= field.maxItems && value.every(item => validField(item, field.items));
  }
}

/** One host-bound window owns one registry; it is never installed on window. */
export class UiActions {
  private entries = new Map<string, Entry>();
  private handles = new Map<string, { entry: Entry; revision: string }>();
  private requests = new Map<string, { actor: UiActor; settled: boolean; signature: string; result: Promise<UiResult>; controller: AbortController }>();
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

  register(key: string, command: UiCommand): () => void {
    if (this.closed) throw Error("UI registry is closed");
    if (this.entries.has(key)) throw Error(`Duplicate UI command: ${key}`);
    // Copy metadata so a caller cannot mutate a disclosed schema in place.
    const entry: Entry = { command: { ...command, parameters: structuredClone(command.parameters), prerequisites: [...command.prerequisites] } };
    this.entries.set(key, entry);
    return () => {
      if (this.entries.get(key) !== entry) return;
      this.entries.delete(key);
      for (const [handle, target] of this.handles) if (target.entry === entry) this.handles.delete(handle);
    };
  }

  descriptor(key: string): UiDescriptor | null {
    const entry = this.entries.get(key);
    return entry ? this.describe(entry) : null;
  }

  groups(limit = 32, offset = 0) {
    if (!Number.isInteger(limit) || limit < 1 || limit > 64 || !Number.isInteger(offset) || offset < 0) throw Error("Invalid observation bounds");
    const parents = [...new Set([...this.entries.values()].map(e => e.command.parent))];
    return { observation: `observation:${this.namespace}:${++this.observation}`, scope: { ...this.scope }, groups: parents.slice(offset, offset + limit), total: parents.length, next: offset + limit < parents.length ? offset + limit : null, coverage: "Only registered controls; remaining UI parity is tracked in #74/#75", edit_baseline: false };
  }

  private describe(entry: Entry): UiDescriptor {
    const state = entry.command.read();
    const previous = entry.descriptor;
    if (!previous || previous.revision !== state.revision || previous.enabled !== state.enabled || previous.reason !== state.reason || canonical(previous.value) !== canonical(state.value)) {
      if (previous) this.handles.delete(previous.handle);
      const handle = `action:${this.namespace}:${(++this.sequence).toString(36)}`;
      entry.descriptor = { handle, label: entry.command.label, subject: entry.command.subject, parent: entry.command.parent, parameters: entry.command.parameters, prerequisites: entry.command.prerequisites, ...state };
      this.handles.set(handle, { entry, revision: state.revision });
    }
    return structuredClone(entry.descriptor!);
  }

  /** Reads exactly one explicitly requested parent; scrolling never calls this. */
  observe(parent: string, limit = 32, offset = 0) {
    if (!Number.isInteger(limit) || limit < 1 || limit > 64 || !Number.isInteger(offset) || offset < 0) throw Error("Invalid observation bounds");
    const entries = [...this.entries.values()].filter(e => e.command.parent === parent);
    return {
      observation: `observation:${this.namespace}:${++this.observation}`,
      scope: { ...this.scope }, parent, offset,
      actions: entries.slice(offset, offset + limit).map(e => this.describe(e)),
      coverage: { disclosed: Math.min(limit, Math.max(0, entries.length - offset)), total: entries.length },
      next: offset + limit < entries.length ? offset + limit : null,
      edit_baseline: false as const,
    };
  }

  /** Actor and scope come from the trusted host adapter, never model arguments. */
  act(scope: UiScope, actor: UiActor, invocation: UiInvocation): Promise<UiResult> {
    if (this.closed) return Promise.resolve({ status: "unavailable", reason: "Originating UI window is closed" });
    if (scope.window !== this.scope.window || scope.workspace !== this.scope.workspace) return Promise.resolve({ status: "forbidden", reason: "Action belongs to another workspace/window" });
    if (!invocation || typeof invocation.handle !== "string" || typeof invocation.request_id !== "string" || invocation.request_id.length < 1 || invocation.request_id.length > 128) return Promise.resolve({ status: "invalid", reason: "A bounded request_id and action handle are required" });
    let signature: string;
    try { signature = canonical({ actor, invocation }); } catch { return Promise.resolve({ status: "invalid", reason: "Arguments must be JSON" }); }
    const prior = this.requests.get(invocation.request_id);
    if (prior) return prior.signature === signature ? prior.result : Promise.resolve({ status: "conflict", reason: "request_id already used with different arguments or actor" });
    if (this.requests.size >= this.maxRequests) return Promise.resolve({ status: "unavailable", reason: "Window request journal is full; retain receipts and establish a new session" });
    const controller = new AbortController();
    // Defer execution until the request is journaled, including synchronous callbacks.
    const captured = structuredClone(invocation);
    const capturedActor = { ...actor };
    const result = Promise.resolve().then(() => this.execute(capturedActor, captured, controller.signal)).catch((error): UiResult => ({ status: "error", reason: String(error) })).finally(() => { this.requests.get(captured.request_id)!.settled = true; });
    this.requests.set(invocation.request_id, { actor: capturedActor, settled: false, signature, result, controller });
    return result;
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

  private async execute(actor: UiActor, invocation: UiInvocation, signal: AbortSignal): Promise<UiResult> {
    const target = this.handles.get(invocation.handle);
    if (!target || ![...this.entries.values()].includes(target.entry)) return { status: "unavailable", reason: "Unknown, retired, or undisclosed action; observe its parent again" };
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
      await this.record({ stage: "requested", request_id: invocation.request_id, actor, scope: this.scope, action, arguments: invocation.arguments });
      if (signal.aborted) result = { status: "unavailable", reason: "Cancelled before execution" };
      else if (![...this.entries.values()].includes(target.entry)) result = { status: "unavailable", reason: "Command was unregistered before execution" };
      else {
        const current = this.describe(target.entry);
        if (current.handle !== action.handle) result = { status: "stale", reason: "State changed while recording intent", observation: current };
        else if (!this.authorize(actor, current)) result = { status: "forbidden", reason: "Current actor policy denies this command" };
        else result = { status: "result", outcome: await target.entry.command.execute(invocation.arguments, actor, signal) };
      }
    } catch (error) {
      result = { status: "error", reason: String(error) };
    }
    try {
      await this.record({ stage: "result", request_id: invocation.request_id, actor, scope: this.scope, action, arguments: invocation.arguments, result });
    } catch {
      result = { status: "error", reason: "Could not retain the action result; side effects may have occurred. Inspect current state before a new request." };
    } finally { this.busy.delete(action.subject); }
    return result;
  }
}
