<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/uiActionBridge.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5571d9f178fc"></a>
## `apps/web/src/lib/uiActionBridge.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-5571d9f178fc-Context"></a>
### `Context`

<storycode:copy id="f-5571d9f178fc-Context">
import { registerNativeWindowActions } from "./nativeWindowActions";
import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { request } from "../api/client";
import { UiActions, type UiActor, type UiCommand, type UiInvocation, type UiValue } from "./uiActions";

const Context = createContext<UiActions | null>(null);
</storycode:copy>
<a id="f-5571d9f178fc-activeWindow"></a>
### `activeWindow`

<storycode:copy id="f-5571d9f178fc-activeWindow">
let activeWindow: string | undefined;
</storycode:copy>
<a id="f-5571d9f178fc-WindowConnection"></a>
### `WindowConnection`

<storycode:copy id="f-5571d9f178fc-WindowConnection">
type WindowConnection = { ready: Promise<string | undefined>; closed: boolean; failed?: string };
</storycode:copy>
<a id="f-5571d9f178fc-windowConnection"></a>
### `windowConnection`

<storycode:copy id="f-5571d9f178fc-windowConnection">
let windowConnection: WindowConnection | undefined;
</storycode:copy>
<a id="f-5571d9f178fc-useUiScope"></a>
### `useUiScope`

<storycode:copy id="f-5571d9f178fc-useUiScope">
export function useUiScope() { return useContext(Context)?.scope ?? null; }
</storycode:copy>
<a id="f-5571d9f178fc-uiWindow"></a>
### `uiWindow`

<storycode:copy id="f-5571d9f178fc-uiWindow">
export function uiWindow(): string | undefined { return activeWindow; }
</storycode:copy>
<a id="f-5571d9f178fc-captureUiWindow"></a>
### `captureUiWindow`

Capture this connection once; never wait on or adopt a replacement window.

<storycode:copy id="f-5571d9f178fc-captureUiWindow">
/** Capture this connection once; never wait on or adopt a replacement window. */
export async function captureUiWindow(timeoutMs = 3000): Promise<string | undefined> {
  const connection = windowConnection;
  if (!connection) return activeWindow;
  let timeout: ReturnType<typeof setTimeout> | undefined;
  try {
    const window = await Promise.race([connection.ready, new Promise<never>((_resolve, reject) => { timeout = setTimeout(() => reject(Error("UI connection is still initializing. Retry Send after it connects; no request was submitted.")), timeoutMs); })]);
    if (connection.closed) throw Error("The originating UI window closed during context capture. Start a new request in the current window.");
    if (connection.failed) throw Error(`UI connection unavailable: ${connection.failed}. Reopen the window before retrying Send; no request was submitted.`);
    return window;
  } finally { if (timeout !== undefined) clearTimeout(timeout); }
}
</storycode:copy>
<a id="f-5571d9f178fc-useUiRevision"></a>
### `useUiRevision`

Compare private controller state without serializing it into model-visible tokens.

<storycode:copy id="f-5571d9f178fc-useUiRevision">
/** Compare private controller state without serializing it into model-visible tokens. */
export function useUiRevision(read: () => unknown): () => string {
  const previous = useRef<{ signature: string | undefined; token: string } | null>(null);
  return () => {
    const signature = JSON.stringify(read());
    if (!previous.current || previous.current.signature !== signature) previous.current = { signature, token: `ui-revision:${crypto.randomUUID()}` };
    return previous.current.token;
  };
}
</storycode:copy>
<a id="f-5571d9f178fc-Frame"></a>
### `Frame`

<storycode:copy id="f-5571d9f178fc-Frame">
type Frame = { id: string; actor: UiActor; method: string; arguments: Record<string, UiValue> };

</storycode:copy>
<a id="f-5571d9f178fc-dispatchUiFrame"></a>
### `dispatchUiFrame`

Dedicated structured frames only; no DOM scan, text parser or global RPC object.

<storycode:copy id="f-5571d9f178fc-dispatchUiFrame">
/** Dedicated structured frames only; no DOM scan, text parser or global RPC object. */
export async function dispatchUiFrame(registry: UiActions, frame: Frame): Promise<unknown> {
  const args = frame.arguments;
  if (frame.actor.kind !== "agent") throw Error("Host UI frames must identify an agent");
  switch (frame.method) {
    case "ui_observe": {
      const parent = args.parent ?? "root";
      const limit = args.limit ?? 32;
      const offset = args.offset ?? 0;
      if (typeof parent !== "string" || typeof limit !== "number" || typeof offset !== "number" || Object.keys(args).some(k => !["parent", "limit", "offset"].includes(k))) throw Error("Invalid UI observation arguments");
      const result = parent === "root" ? registry.groups(limit, offset) : registry.observe(parent, limit, offset);
      return { ...result, observation_bytes: new TextEncoder().encode(JSON.stringify(result)).length, token_cost: "Provider-specific; not measured here" };
    }
    case "ui_act": {
      if (Object.keys(args).some(k => !["handle", "arguments", "request_id"].includes(k))) throw Error("Invalid UI invocation arguments");
      const result = await registry.act(registry.scope, frame.actor, args as unknown as UiInvocation);
      return { ...result, result_bytes: new TextEncoder().encode(JSON.stringify(result)).length, token_cost: "Provider-specific; not measured here" };
    }
    case "ui_cancel": {
      if (typeof args.request_id !== "string" || Object.keys(args).length !== 1) throw Error("Invalid cancellation arguments");
      return { cancellation_requested: registry.cancel(registry.scope, frame.actor, args.request_id), rollback: "not asserted" };
    }
    default: throw Error("Unknown UI method");
  }
}

</storycode:copy>
<a id="f-5571d9f178fc-UiActionsProvider"></a>
### `UiActionsProvider`

<storycode:copy id="f-5571d9f178fc-UiActionsProvider">
export function UiActionsProvider({ children }: { children: ReactNode }) {
  const [registry, setRegistry] = useState<UiActions | null>(null);
  useEffect(() => {
    let stopped = false;
    let settle!: (window: string | undefined) => void;
    const origin: WindowConnection = { ready: new Promise(resolve => { settle = resolve; }), closed: false };
    windowConnection = origin;
    let window: string | undefined;
    let connection: UiActions | undefined;
    async function connect() {
      const scope = await request<{ workspace: string; window: string }>("POST", "/api/v1/ui/windows", {});
      if (typeof scope?.workspace !== "string" || !scope.workspace.length || scope.workspace.length > 512 || typeof scope?.window !== "string" || !/^[A-Za-z0-9_-]{1,128}$/.test(scope.window)) throw Error("Invalid shared UI protocol v1 scope; update both the app and engine before sending an attached task");
      window = scope.window;
      if (stopped) { void request("DELETE", `/api/v1/ui/windows/${scope.window}`).catch(() => {}); return; }
      const actions = new UiActions(scope,
        // Only explicit controllers register. Each controller preserves actual
        // actor policy and returns required human input for privileged decisions.
        () => true,
        receipt => request("POST", `/api/v1/ui/windows/${scope.window}/receipts`, receipt).then(() => {}));
      if ("__TAURI_INTERNALS__" in globalThis) registerNativeWindowActions(actions);
      activeWindow = scope.window;
      settle(scope.window);
      connection = actions;
      setRegistry(actions);
      while (!stopped) {
        try {
          const { request: frame } = await request<{ request: Frame | null }>("GET", `/api/v1/ui/windows/${scope.window}/requests`);
          if (stopped) break;
          if (frame) {
            // Run independently so the poll can accept cancellation while a
            // command is awaiting its real executor. Host queue bounds apply.
            void dispatchUiFrame(actions, frame).catch(error => ({ status: "error", reason: String(error) }))
              .then(result => frame.id ? request("POST", `/api/v1/ui/windows/${scope.window}/reply`, { id: frame.id, result }) : undefined).catch(() => {});
          }
        } catch {
          // No re-registration: old turns must not silently attach a new window.
          if (!stopped) await new Promise(resolve => setTimeout(resolve, 1000));
        }
      }
    }
    void connect().catch(error => { origin.failed = (error as { status?: number })?.status === 404 ? "Shared UI protocol v1 is unavailable; update both the app and engine before sending an attached task" : String(error); settle(undefined); /* Ordinary UI stays available; Send reports this connection failure. */ });
    return () => {
      stopped = true;
      origin.closed = true;
      settle(undefined);
      if (windowConnection === origin) windowConnection = undefined;
      connection?.dispose();
      if (activeWindow === window) activeWindow = undefined;
      if (window) void request("DELETE", `/api/v1/ui/windows/${window}`).catch(() => {});
    };
  }, []);
  return <Context.Provider value={registry}>{children}</Context.Provider>;
}

</storycode:copy>
<a id="f-5571d9f178fc-UiActionsVisibility"></a>
### `UiActionsVisibility`

Keep mounted editors alive without disclosing controls on hidden surfaces.

<storycode:copy id="f-5571d9f178fc-UiActionsVisibility">
/** Keep mounted editors alive without disclosing controls on hidden surfaces. */
export function UiActionsVisibility({ visible, children }: { visible: boolean; children: ReactNode }) {
  const registry = useContext(Context);
  return <Context.Provider value={visible ? registry : null}>{children}</Context.Provider>;
}

</storycode:copy>
<a id="f-5571d9f178fc-useUiCommands"></a>
### `useUiCommands`

Bind real controllers without retaining stale render closures.

<storycode:copy id="f-5571d9f178fc-useUiCommands">
/** Bind real controllers without retaining stale render closures. */
export function useUiCommands(commands: { key: string; command: UiCommand }[]) {
  const registry = useContext(Context);
  const latest = useRef(commands); latest.current = commands;
  const signature = JSON.stringify(commands.map(({ key, command }) => [key, command.label, command.subject, command.parent, command.parameters, command.prerequisites]));
  const registered = useRef(new Map<string, { signature: string; remove: () => void }>());
  useEffect(() => {
    if (!registry) return;
    const keys = new Set(latest.current.map(entry => entry.key));
    for (const [key, entry] of registered.current) if (!keys.has(key)) { entry.remove(); registered.current.delete(key); }
    for (const { key, command } of latest.current) {
      const metadata = JSON.stringify([command.label, command.subject, command.parent, command.parameters, command.prerequisites]);
      const previous = registered.current.get(key);
      // A sibling appearing/disappearing must not retire this unchanged owner
      // while its intent is being persisted. Its live revision still gates execution.
      if (previous?.signature === metadata) continue;
      previous?.remove();
      const remove = registry.register(key, {
        ...command,
        read: () => latest.current.find(c => c.key === key)?.command.read() ?? { revision: "removed", enabled: false, reason: "Control unavailable" },
        execute: (args, actor, signal) => {
          const current = latest.current.find(c => c.key === key);
          if (!current) return Promise.resolve({ status: "unavailable", reason: "Control unavailable" });
          return current.command.execute(args, actor, signal);
        },
      });
      registered.current.set(key, { signature: metadata, remove });
    }
  }, [registry, signature]);
  useEffect(() => () => { for (const entry of registered.current.values()) entry.remove(); registered.current.clear(); }, [registry]);
  return async (key: string, arguments_: Record<string, UiValue> = {}): Promise<void> => {
    const current = latest.current.find(c => c.key === key);
    if (!current) return;
    const descriptor = registry?.descriptor(key);
    if (!registry || !descriptor) { await current.command.execute(arguments_, { id: "local-user", kind: "human" }, new AbortController().signal); return; }
    const result = await registry.act(registry.scope, { id: "local-user", kind: "human" }, { handle: descriptor.handle, arguments: arguments_, request_id: crypto.randomUUID() });
    if (result.status !== "result") throw Error(result.reason);
    if (result.outcome.status === "unavailable") throw Error(result.outcome.reason);
  };
}

</storycode:copy>
<a id="f-5571d9f178fc-UiActionContext"></a>
### `UiActionContext`

Test/embedding injection uses the same registry and controllers.

<storycode:copy id="f-5571d9f178fc-UiActionContext">
/** Test/embedding injection uses the same registry and controllers. */
export const UiActionContext = Context;

</storycode:copy>
<a id="f-5571d9f178fc-useUiPendingHumanActions"></a>
### `useUiPendingHumanActions`

Snapshot only local human requests; menu dismissal must not retire a click before intent persistence.

<storycode:copy id="f-5571d9f178fc-useUiPendingHumanActions">
/** Snapshot only local human requests; menu dismissal must not retire a click before intent persistence. */
export function useUiPendingHumanActions(excludingSubject?: string) {
  const registry = useContext(Context);
  return () => registry?.pending({ id: "local-user", kind: "human" }, excludingSubject) ?? [];
}
</storycode:copy>

### Assemble `apps/web/src/lib/uiActionBridge.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiActionBridge.tsx"><storycode:paste select="#f-5571d9f178fc-Context"/><storycode:paste select="#f-5571d9f178fc-activeWindow"/><storycode:paste select="#f-5571d9f178fc-WindowConnection"/><storycode:paste select="#f-5571d9f178fc-windowConnection"/><storycode:paste select="#f-5571d9f178fc-useUiScope"/><storycode:paste select="#f-5571d9f178fc-uiWindow"/><storycode:paste select="#f-5571d9f178fc-captureUiWindow"/><storycode:paste select="#f-5571d9f178fc-useUiRevision"/><storycode:paste select="#f-5571d9f178fc-Frame"/><storycode:paste select="#f-5571d9f178fc-dispatchUiFrame"/><storycode:paste select="#f-5571d9f178fc-UiActionsProvider"/><storycode:paste select="#f-5571d9f178fc-UiActionsVisibility"/><storycode:paste select="#f-5571d9f178fc-useUiCommands"/><storycode:paste select="#f-5571d9f178fc-UiActionContext"/><storycode:paste select="#f-5571d9f178fc-useUiPendingHumanActions"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-2cc5832c66b7"></a>
## `apps/web/src/lib/uiActionBridge.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-2cc5832c66b7-scope"></a>
### `scope`

<storycode:copy id="f-2cc5832c66b7-scope">
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { UiActions, type UiReceipt } from "./uiActions";
import { dispatchUiFrame, UiActionContext, UiActionsVisibility, useUiCommands } from "./uiActionBridge";
import { MoreActions } from "../components/MoreActions";
import { CardRail } from "../editor/CardRail";
import type { DocCard } from "../editor/cards";

const scope = { workspace: "practice", window: "actual-origin" };
</storycode:copy>
<a id="f-2cc5832c66b7-agent"></a>
### `agent`

<storycode:copy id="f-2cc5832c66b7-agent">
const agent = { id: "python:turn", kind: "agent" as const };
</storycode:copy>
<a id="f-2cc5832c66b7-registry"></a>
### `registry`

<storycode:copy id="f-2cc5832c66b7-registry">
function registry() {
  const receipts: UiReceipt[] = [];
  const actions = new UiActions(scope, () => true, async receipt => { receipts.push(receipt); });
  return { actions, receipts };
}

</storycode:copy>
<a id="f-2cc5832c66b7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2cc5832c66b7-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
describe("registered human UI and structured agent transport", () => {
  it("retires controls on a hidden mounted surface and requires fresh handles after reopening", async () => {
    const { actions } = registry();
    const content = (visible: boolean) => <UiActionContext.Provider value={actions}><UiActionsVisibility visible={visible}><MoreActions label="Hidden panel controls"><button>Child</button></MoreActions></UiActionsVisibility></UiActionContext.Provider>;
    const { rerender } = render(content(true));
    const handle = actions.observe("menus").actions[0].handle;
    rerender(content(false)); expect(actions.observe("menus").actions).toEqual([]);
    expect(await actions.act(scope, agent, { handle, arguments: {}, request_id: "hidden" })).toMatchObject({ status: "unavailable" });
    rerender(content(true));
    expect(actions.observe("menus").actions[0].handle).not.toBe(handle);
  });
  it("lets the agent discover and operate the real menu, then the human close it", async () => {
    const { actions, receipts } = registry();
    render(<UiActionContext.Provider value={actions}><MoreActions label="Execution actions"><button>Existing child</button></MoreActions></UiActionContext.Provider>);
    const root = await dispatchUiFrame(actions, { id: "1", actor: agent, method: "ui_observe", arguments: {} }) as { groups: string[]; observation_bytes: number };
    expect(root.groups).toEqual(["menus"]);
    expect(root.observation_bytes).toBeGreaterThan(0);
    const descriptor = actions.observe("menus").actions[0];
    expect(descriptor.value).toBe(false);
    await act(async () => {
      const result = await dispatchUiFrame(actions, { id: "2", actor: agent, method: "ui_act", arguments: { handle: descriptor.handle, arguments: {}, request_id: "agent-open" } });
      expect(result).toMatchObject({ status: "result", outcome: { status: "completed", value: { open: true } } });
    });
    const menu = screen.getByLabelText("Execution actions").closest("details")!;
    expect(menu.open).toBe(true);
    fireEvent.click(screen.getByLabelText("Execution actions"));
    await waitFor(() => expect(menu.open).toBe(false));
    expect(receipts.filter(r => r.stage === "result").map(r => r.actor.kind)).toEqual(["agent", "human"]);
    expect(screen.queryByText(descriptor.handle)).toBeNull();
  });

  // An intent failure must remain readable outside the clipped rail and dismissible.
  it("shows a complete rail failure outside the toolbar without running the cell", async () => {
    const message = "Originating UI window stopped responding " + "host detail ".repeat(40);
    const actions = new UiActions(scope, () => true, async () => { throw Error(message); });
    const callback = vi.fn();
    const card: DocCard = { key: "failed-exec", kind: "exec", index: 0, at: 1, from: 1, to: 4, label: "printf" };
    const mounted = render(<UiActionContext.Provider value={actions}><CardRail documentId="failure" view={null} cards={[card]} stateOf={() => "idle"} openKey={null} renderedAt={[]} actionsOf={() => ["run"]} replayingAt={[]} onAction={callback} /></UiActionContext.Provider>);
    fireEvent.click(mounted.getByRole("button", { name: /printf.*click to run it/ }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain(message);
    expect(alert.closest(".cm-card-rail")).toBeNull();
    expect(alert.parentElement).toBe(document.body);
    expect(callback).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(screen.queryByRole("alert")).toBeNull();
    mounted.unmount();
  });

  it("runs the existing cell callback once per request and never claims runtime verification", async () => {
    const { actions, receipts } = registry();
    const callback = vi.fn();
    const card: DocCard = { key: "exec-1", kind: "exec", index: 0, at: 1, from: 1, to: 4, label: "λ😀" };
    const props = { documentId: "deadline", view: null, cards: [card], stateOf: () => "idle" as const, openKey: null, renderedAt: [], actionsOf: () => ["run" as const], replayingAt: [], onAction: callback };
    const { rerender } = render(<UiActionContext.Provider value={actions}><CardRail {...props} /></UiActionContext.Provider>);
    const parent = actions.groups().groups[0];
    const descriptor = actions.observe(parent).actions[0];
    const frame = { id: "1", actor: agent, method: "ui_act", arguments: { handle: descriptor.handle, arguments: {}, request_id: "run-request" } };
    await act(async () => {
      const result = await dispatchUiFrame(actions, frame);
      expect(result).toMatchObject({ status: "result", outcome: { value: { ui_handler_returned: true, runtime_verification: "not established" } } });
      await dispatchUiFrame(actions, frame);
    });
    expect(callback).toHaveBeenCalledExactlyOnceWith(card, "run", 0);
    fireEvent.click(screen.getByRole("button", { name: /click to run it/ }));
    await waitFor(() => expect(callback).toHaveBeenCalledTimes(2));
    expect(receipts.filter(r => r.stage === "requested").map(r => r.actor.kind)).toEqual(["agent", "human"]);
    rerender(<UiActionContext.Provider value={actions}><CardRail {...props} stateOf={() => "running"} /></UiActionContext.Provider>);
    const running = actions.observe(parent).actions[0];
    expect(running.enabled).toBe(false);
    expect(await dispatchUiFrame(actions, { ...frame, arguments: { ...frame.arguments, handle: running.handle, request_id: "disabled" } })).toMatchObject({ status: "disabled" });
    expect(callback).toHaveBeenCalledTimes(2);
  });

  it("uses latest callbacks and unregisters closed controls", async () => {
    const { actions } = registry(); const first = vi.fn(); const next = vi.fn();
    const card: DocCard = { key: "one", kind: "diagram", index: 0, at: 0, from: 0, to: 1, label: "Diagram" };
    const props = { view: null, cards: [card], stateOf: () => "idle" as const, openKey: null, renderedAt: [], actionsOf: () => ["source" as const], replayingAt: [] };
    const { rerender, unmount } = render(<UiActionContext.Provider value={actions}><CardRail {...props} onAction={first} /></UiActionContext.Provider>);
    const descriptor = actions.observe(actions.groups().groups[0]).actions[0];
    rerender(<UiActionContext.Provider value={actions}><CardRail {...props} onAction={next} /></UiActionContext.Provider>);
    await act(async () => { await actions.act(scope, agent, { handle: descriptor.handle, arguments: {}, request_id: "toggle" }); });
    expect(first).not.toHaveBeenCalled(); expect(next).toHaveBeenCalledOnce();
    unmount();
    expect(actions.groups().groups).toEqual([]);
    expect(await actions.act(scope, agent, { handle: descriptor.handle, arguments: {}, request_id: "closed" })).toMatchObject({ status: "unavailable" });
  });

  it("rejects forged actor/scope fields and unstructured action text", async () => {
    const { actions } = registry();
    await expect(dispatchUiFrame(actions, { id: "1", actor: agent, method: "ui_act", arguments: { handle: "action:forged", arguments: {}, request_id: "r", window: "another" } })).rejects.toThrow("Invalid");
    await expect(dispatchUiFrame(actions, { id: "1", actor: { id: "human", kind: "human" }, method: "ui_observe", arguments: {} })).rejects.toThrow("agent");
    await expect(dispatchUiFrame(actions, { id: "1", actor: agent, method: "quoted action:forged", arguments: {} })).rejects.toThrow("Unknown");
    await expect(dispatchUiFrame(actions, { id: "1", actor: agent, method: "ui_observe", arguments: { limit: 100000 } })).rejects.toThrow("bounds");
  });
});

</storycode:copy>
<a id="f-2cc5832c66b7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-2cc5832c66b7-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reports the known UTF-8 result payload size separately from provider token cost", async () => {
  const { actions } = registry();
  actions.register("unicode", { parent: "reading", subject: "reading", label: "Read Unicode", parameters: {}, prerequisites: [], read: () => ({ revision: "one", enabled: true }), execute: async () => ({ status: "completed", value: { text: "λ😀" } }) });
  const handle = actions.observe("reading").actions[0].handle;
  const result = await dispatchUiFrame(actions, { id: "unicode", actor: agent, method: "ui_act", arguments: { handle, arguments: {}, request_id: "unicode" } });
  expect(result).toMatchObject({ status: "result", result_bytes: 78, token_cost: "Provider-specific; not measured here", outcome: { value: { text: "λ😀" } } });
});

</storycode:copy>
<a id="f-2cc5832c66b7-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-2cc5832c66b7-expression_statement-3">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("keeps an unchanged command registered while a sibling changes during intent persistence", async () => {
  let release!: () => void;
  const retained = new Promise<void>(resolve => { release = resolve; });
  const actions = new UiActions(scope, () => true, receipt => receipt.stage === "requested" ? retained : Promise.resolve());
  const execute = vi.fn(async () => ({ status: "completed" as const, value: null }));
  function Controls({ sibling }: { sibling: boolean }) {
    const command = { parent: "same", subject: "same", label: "Keep", parameters: {}, prerequisites: [], read: () => ({ revision: "unchanged", enabled: true }), execute };
    useUiCommands([{ key: "keep", command }, ...(sibling ? [{ key: "sibling", command: { ...command, label: "Sibling" } }] : [])]);
    return null;
  }
  const content = (sibling: boolean) => <UiActionContext.Provider value={actions}><Controls sibling={sibling} /></UiActionContext.Provider>;
  const view = render(content(false));
  const handle = actions.descriptor("keep")!.handle;
  const pending = actions.act(scope, agent, { handle, arguments: {}, request_id: "intent" });
  await act(async () => { await Promise.resolve(); });
  view.rerender(content(true));
  expect(actions.descriptor("keep")!.handle).toBe(handle);
  release();
  expect(await pending).toMatchObject({ status: "result", outcome: { status: "completed" } });
  expect(execute).toHaveBeenCalledOnce();
  view.unmount();
  expect(actions.groups().groups).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/lib/uiActionBridge.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiActionBridge.test.tsx"><storycode:paste select="#f-2cc5832c66b7-scope"/><storycode:paste select="#f-2cc5832c66b7-agent"/><storycode:paste select="#f-2cc5832c66b7-registry"/><storycode:paste select="#f-2cc5832c66b7-expression_statement"/><storycode:paste select="#f-2cc5832c66b7-expression_statement-2"/><storycode:paste select="#f-2cc5832c66b7-expression_statement-3"/></storycode:file>


</storycode:doc>
