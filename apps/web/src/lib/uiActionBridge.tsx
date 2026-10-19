import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { request } from "../api/client";
import { UiActions, type UiActor, type UiCommand, type UiInvocation, type UiValue } from "./uiActions";

const Context = createContext<UiActions | null>(null);
let activeWindow: string | undefined;
export function uiWindow(): string | undefined { return activeWindow; }
/** Compare private controller state without serializing it into model-visible tokens. */
export function useUiRevision(read: () => unknown): () => string {
  const previous = useRef<{ signature: string | undefined; token: string } | null>(null);
  return () => {
    const signature = JSON.stringify(read());
    if (!previous.current || previous.current.signature !== signature) previous.current = { signature, token: `ui-revision:${crypto.randomUUID()}` };
    return previous.current.token;
  };
}
type Frame = { id: string; actor: UiActor; method: string; arguments: Record<string, UiValue> };

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
      return registry.act(registry.scope, frame.actor, args as unknown as UiInvocation);
    }
    case "ui_cancel": {
      if (typeof args.request_id !== "string" || Object.keys(args).length !== 1) throw Error("Invalid cancellation arguments");
      return { cancellation_requested: registry.cancel(registry.scope, frame.actor, args.request_id), rollback: "not asserted" };
    }
    default: throw Error("Unknown UI method");
  }
}

export function UiActionsProvider({ children }: { children: ReactNode }) {
  const [registry, setRegistry] = useState<UiActions | null>(null);
  useEffect(() => {
    let stopped = false;
    let window: string | undefined;
    let connection: UiActions | undefined;
    async function connect() {
      const scope = await request<{ workspace: string; window: string }>("POST", "/api/ui/windows", {});
      window = scope.window;
      if (stopped) { void request("DELETE", `/api/ui/windows/${scope.window}`).catch(() => {}); return; }
      const actions = new UiActions(scope,
        // Only explicit controllers register. Each controller preserves actual
        // actor policy and returns required human input for privileged decisions.
        () => true,
        receipt => request("POST", `/api/ui/windows/${scope.window}/receipts`, receipt).then(() => {}));
      activeWindow = scope.window;
      connection = actions;
      setRegistry(actions);
      while (!stopped) {
        try {
          const { request: frame } = await request<{ request: Frame | null }>("GET", `/api/ui/windows/${scope.window}/requests`);
          if (stopped) break;
          if (frame) {
            // Run independently so the poll can accept cancellation while a
            // command is awaiting its real executor. Host queue bounds apply.
            void dispatchUiFrame(actions, frame).catch(error => ({ status: "error", reason: String(error) }))
              .then(result => frame.id ? request("POST", `/api/ui/windows/${scope.window}/reply`, { id: frame.id, result }) : undefined).catch(() => {});
          }
        } catch {
          // No re-registration: old turns must not silently attach a new window.
          if (!stopped) await new Promise(resolve => setTimeout(resolve, 1000));
        }
      }
    }
    void connect().catch(() => { /* Old/hostless browser builds retain normal UI. */ });
    return () => {
      stopped = true;
      connection?.dispose();
      if (activeWindow === window) activeWindow = undefined;
      if (window) void request("DELETE", `/api/ui/windows/${window}`).catch(() => {});
    };
  }, []);
  return <Context.Provider value={registry}>{children}</Context.Provider>;
}

/** Keep mounted editors alive without disclosing controls on hidden surfaces. */
export function UiActionsVisibility({ visible, children }: { visible: boolean; children: ReactNode }) {
  const registry = useContext(Context);
  return <Context.Provider value={visible ? registry : null}>{children}</Context.Provider>;
}

/** Bind real controllers without retaining stale render closures. */
export function useUiCommands(commands: { key: string; command: UiCommand }[]) {
  const registry = useContext(Context);
  const latest = useRef(commands); latest.current = commands;
  const signature = JSON.stringify(commands.map(({ key, command }) => [key, command.label, command.subject, command.parent, command.parameters, command.prerequisites]));
  useEffect(() => {
    if (!registry) return;
    const removals = latest.current.map(({ key, command }) => registry.register(key, {
      ...command,
      read: () => latest.current.find(c => c.key === key)?.command.read() ?? { revision: "removed", enabled: false, reason: "Control unavailable" },
      execute: (args, actor, signal) => {
        const current = latest.current.find(c => c.key === key);
        if (!current) return Promise.resolve({ status: "unavailable", reason: "Control unavailable" });
        return current.command.execute(args, actor, signal);
      },
    }));
    return () => removals.forEach(remove => remove());
  }, [registry, signature]);
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

/** Test/embedding injection uses the same registry and controllers. */
export const UiActionContext = Context;
