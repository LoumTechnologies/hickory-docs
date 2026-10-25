<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/AcpControls.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../../../acp-agents-are-first-class/contract.md)
- [Python agent code crosses two explicit boundaries](../../../../../contract.md)

Captured SHA-256: `bed6a5569158f5b1b2af11716dbb26eee55e8d8463237a95473aa57205a174ad`.

<book:entry path="apps/web/src/components/AcpControls.tsx">
import { useCallback, useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision, UiActionsVisibility } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { DiffView } from "./DiffView";
import { unifiedDiff } from "../lib/diff";
import { acpApi, type AcpState, type AcpUpdate, type AgentCommand, type ConfigOption, type AcpPermission } from "../api/acp";

const defaults: AgentCommand[] = [
  { id: "codex", name: "Codex", command: "codex-acp", args: [] },
  { id: "claude", name: "Claude Agent", command: "claude-agent-acp", args: [] },
];

export function useAcp(doc: string, backend: string, session: string | undefined, running: string | null, hydrated = true) {
  const [agents, setAgents] = useState(defaults);
  const [state, setState] = useState<AcpState | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [detecting, setDetecting] = useState(false);
  const [catalogueError, setCatalogueError] = useState<string | null>(null);
  const version = useRef(0);
  const refreshAgents = useCallback(async () => {
    setDetecting(true); setCatalogueError(null);
    try { setAgents((await acpApi.catalogue()).agents); }
    catch { setCatalogueError("Could not check installed agents. Try Refresh agents again."); }
    finally { setDetecting(false); }
  }, []);
  useEffect(() => {
    void refreshAgents();
    const focus = () => { void refreshAgents(); };
    window.addEventListener("focus", focus);
    return () => window.removeEventListener("focus", focus);
  }, [refreshAgents]);
  const connect = useCallback(async (): Promise<Operation> => {
    const stamp = ++version.current;
    setBusy(true); setError(null); setState(null);
    try { const next = await (backend === "python" ? acpApi.state(doc, backend) : acpApi.connect(doc, backend, session)); if (stamp === version.current) setState(next); return { state: next, superseded: stamp !== version.current }; }
    catch (e) { const error = e instanceof Error ? e.message : String(e); if (stamp === version.current) setError(error); return { error, superseded: stamp !== version.current }; }
    finally { if (stamp === version.current) setBusy(false); }
  }, [doc, backend, session]);
  const pythonAvailable = backend === "python" && agents.find(agent => agent.id === "python")?.available;
  useEffect(() => {
    if (hydrated && !running) void connect();
    // A running turn owns its connection; do not reconnect on optimistic UI updates.
  }, [connect, backend, hydrated, pythonAvailable]);
  useEffect(() => {
    if (!running && !state?.ready) return;
    const stamp = version.current;
    let live = true;
    const poll = () => { void Promise.resolve(acpApi.state(doc, backend)).then(s => { if (s && live && stamp === version.current) setState(s); }, () => undefined); };
    poll(); const timer = setInterval(poll, running ? 500 : 1500);
    return () => { live = false; clearInterval(timer); };
  }, [running, backend, doc, state?.ready]);
  const act = async (operation: () => Promise<AcpState>): Promise<Operation> => {
    const stamp = ++version.current;
    setBusy(true); setError(null);
    try { const next = await operation(); if (stamp === version.current) setState(next); return { state: next, superseded: stamp !== version.current }; }
    catch (e) { const error = e instanceof Error ? e.message : String(e); if (stamp === version.current) setError(error); return { error, superseded: stamp !== version.current }; }
    finally { if (stamp === version.current) setBusy(false); }
  };
  const install = async (): Promise<Operation> => {
    const stamp = ++version.current;
    setBusy(true); setError(null);
    try {
      setAgents((await acpApi.install(backend)).agents);
      if (stamp === version.current) return await connect();
      return { superseded: true };
    }
    catch (e) { const error = e instanceof Error ? e.message : String(e); if (stamp === version.current) setError(error); return { error, superseded: stamp !== version.current }; }
    finally { if (stamp === version.current) setBusy(false); }
  };
  return { agents, state, busy, error, connect, install, act, setState, setError, refreshAgents, detecting, catalogueError };
}

type Controls = ReturnType<typeof useAcp>;
type Operation = { state?: AcpState; error?: string; superseded?: boolean };
function operationOutcome(result: Operation): UiOutcome {
  if (result.error) return { status: "unavailable", reason: result.error };
  if (result.superseded) return { status: "unavailable", reason: "The selected connection changed during this host operation; inspect the originating adapter before retrying because it may have completed" };
  return { status: "completed", value: { backend: result.state?.backend ?? null, ready: result.state?.ready ?? false, error: result.state?.error ?? null, session: result.state?.session ?? null, scope: "Returned host state; no provider turn or tool completion certified" } };
}
export function AcpControls({ doc, backend, running, control, settingsOpen = true, settingsOnly = false, showTools = true }: { doc: string; backend: string; running: boolean; control: Controls; settingsOpen?: boolean; settingsOnly?: boolean; showTools?: boolean }) {
  const { state, busy, error, agents } = control;
  const parent = `adapter-connection:${useId()}`;
  const [commandsOpen, setCommandsOpen] = useState(false);
  const revision = useUiRevision(() => [doc, backend, running, state, busy, error, agents, settingsOpen, settingsOnly, showTools, commandsOpen]);
  const availableAgent = agents.find(a => a.id === backend);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled }), execute } });
  add("read", "Read adapter connection bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, true, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ doc, backend, running, busy, error, ready: state?.ready ?? false, adapter_error: state?.error ?? null, settings: settingsOpen ? { agent: availableAgent ?? null, modes: state?.modes ?? null, config: state?.configOptions ?? [], auth_methods: state?.authMethods ?? [], commands: commandsOpen ? state?.commands ?? [] : "Expand Agent commands first" } : null, permission_count: !settingsOnly ? state?.permissions?.length ?? 0 : null, tool_count: !settingsOnly && showTools && running ? state?.tools?.length ?? 0 : null }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside connection state" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  if (settingsOpen) {
    if (backend === "python" ? !state?.ready : !running) add("connect", backend === "python" ? "Check conversation Python runtime" : "Reconnect conversation adapter", {}, !busy, async () => operationOutcome(await control.connect()));
    if (backend !== "python") {
      if (!state?.ready && availableAgent?.installable && !availableAgent.available) add("install", `Install conversation adapter: ${availableAgent.name.slice(0, 256)}`, {}, !busy, async () => operationOutcome(await control.install()));
      if (!state?.ready) for (const method of state?.authMethods?.filter(m => !m.type || m.type === "agent") ?? []) add(`auth:${method.id}`, `Authenticate conversation adapter: ${method.name.slice(0, 256)}`, {}, !busy, async (_args, actor) => {
        if (actor.kind === "agent") return { status: "requires_input", request: `adapter-auth:${doc}:${backend}:${method.id}`, reason: "A human completes the adapter's sign-in and consent; read actual readiness afterward" };
        return operationOutcome(await control.act(() => acpApi.authenticate(doc, method.id)));
      });
      if (!state?.configOptions?.length && state?.modes) add("mode", "Set adapter session mode", { mode: { type: "string", maxLength: 4096, choices: state.modes.availableModes.map(m => m.id) } }, !busy && !running, async (args, actor) => {
        if (actor.kind === "agent") return { status: "requires_input", request: `adapter-mode:${doc}:${backend}`, reason: "A human chooses the adapter execution mode; inspect its actual returned state" };
        return operationOutcome(await control.act(() => acpApi.configure(doc, "__mode", args.mode as string)));
      });
      if (state?.commands?.length) add("commands", "Set adapter commands visibility", { open: { type: "boolean" } }, true, async args => { setCommandsOpen(args.open as boolean); return { status: "completed", value: args.open }; });
    }
  }
  const invoke = useUiCommands(commands);
  const click = (key: string, args: Parameters<typeof invoke>[1] = {}) => { void invoke(`${parent}:${key}`, args).catch(reason => control.setError(String(reason))); };
  if (backend === "python") return <div className="acp-controls" aria-label="Python agent controls">
    {(error || state?.error) && <p role="alert" className="chat-note error">{error || state?.error}</p>}
    {settingsOpen && !state?.ready && <p>Set the Python runtime in <a href="#/settings">Settings → Agents</a>, then <button className="btn-link" disabled={busy} onClick={() => click("connect")}>Check runtime</button>.</p>}
  </div>;
  const agent = agents.find(a => a.id === backend);
  return <div className="acp-controls" aria-label="Agent controls">
    {settingsOpen && <div className="acp-settings">
      {!state?.configOptions?.length && state?.modes && <label>Mode<select aria-label="Mode" value={state.modes.currentModeId} disabled={busy || running}
        onChange={e => click("mode", { mode: e.target.value })}>
        {state.modes.availableModes.map(mode => <option key={mode.id} value={mode.id}>{mode.name}</option>)}
      </select></label>}
      {state?.configOptions?.map(option => <ConfigControl key={option.id} scope={`${doc}:${backend}:${state.session ?? "new"}`} option={option} disabled={busy || running}
        onChange={value => control.act(() => acpApi.configure(doc, option.id, value))} />)}
      {!state?.ready && !busy && agent?.installable && !agent.available &&
        <button className="btn" onClick={() => click("install")}>Install {agent.name} adapter</button>}
      {!state?.ready && !busy && state?.authMethods?.filter(m => !m.type || m.type === "agent").map(m =>
        <button className="btn" key={m.id} data-tip={m.description} onClick={() => click(`auth:${m.id}`)}>{m.name}</button>)}
      {!running && <button className="btn-link" disabled={busy} onClick={() => click("connect")}>{busy ? "Connecting…" : "Reconnect"}</button>}
    </div>}
    {!settingsOnly && busy && <p className="muted" role="status">Connecting to {agent?.name ?? backend}… Sign-in may open in your browser.</p>}
    {!settingsOnly && (error || state?.error) && <p role="alert" className="chat-note error">{error || state?.error}</p>}
    {settingsOpen && !busy && agent?.available === false && <p className="muted">{agent.cli_available ? `${agent.name} CLI was found, but its ACP adapter is still needed. ` : ""}{agent.installable ? "Install the adapter above to connect." : "Install Node.js and npm to install a known adapter, or set its executable in Settings → Agents."}</p>}
    {settingsOpen && !busy && !state?.ready && <p className="muted">Use your agent’s own account or API key. Agent commands follow that agent’s permissions and sandbox.</p>}
    {settingsOpen && state?.commands?.length ? <details className="acp-commands" open={commandsOpen} onToggle={event => setCommandsOpen(event.currentTarget.open)}><summary>Agent commands</summary>
      {state.commands.map(c => <p key={c.name}><code>/{c.name}</code> — {c.description}</p>)}</details> : null}
    {!settingsOnly && state?.permissions?.map(permission => <PermissionControl key={permission.id} doc={doc} backend={backend} permission={permission} control={control} />)}
    {!settingsOnly && showTools && running && state?.tools?.map(tool => <AdapterTool key={tool.toolCallId} tool={tool} />)}
  </div>;
}

function PermissionControl({ doc, backend, permission, control }: { doc: string; backend: string; permission: AcpPermission; control: Controls }) {
  const parent = `adapter-permission:${useId()}`;
  const [answering, setAnswering] = useState(false);
  const pending = useRef(false);
  const revision = useUiRevision(() => [doc, backend, permission, answering]);
  const invoke = useUiCommands([{ key: parent, command: {
    label: "Answer adapter permission", parent, subject: parent,
    parameters: { option: { type: "string", maxLength: 4096, choices: permission.options.map(o => o.optionId) } }, prerequisites: ["The human chooses an actual current adapter permission option"],
    read: () => ({ revision: revision(), enabled: !answering }),
    execute: async (args, actor): Promise<UiOutcome> => {
      if (actor.kind === "agent") return { status: "requires_input", request: `adapter-permission:${doc}:${backend}:${permission.id}`, reason: "The human must choose this adapter permission; inspect current request/status after that decision" };
      if (pending.current) return { status: "unavailable", reason: "Permission answer already in flight" };
      pending.current = true; setAnswering(true);
      try {
        const actual = await acpApi.permission(doc, permission.id, args.option as string);
        if (!actual.answered) { const reason = "The host did not accept this permission answer; refresh actual adapter state"; control.setError(reason); return { status: "unavailable", reason }; }
        control.setState(s => s ? { ...s, permissions: s.permissions?.filter(p => p.id !== permission.id) } : s);
        return { status: "completed", value: { request: permission.id, option: args.option, answered: actual.answered, tool_completion: "not established" } };
      } catch (e) { const reason = e instanceof Error ? e.message : String(e); control.setError(reason); return { status: "unavailable", reason }; }
      finally { pending.current = false; setAnswering(false); }
    },
  } }, { key: `${parent}:read`, command: {
    label: "Read adapter permission bytes", parent, subject: parent,
    parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }),
    execute: async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ doc, backend, permission, answering }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside permission request" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    },
  } }]);
  return <section className="acp-permission" role="group" aria-label="Agent permission">
    <strong>{permission.toolCall.title ?? "The agent needs your permission"}</strong>
    <ToolDetails tool={permission.toolCall} />
    <div className="acp-settings">{permission.options.map(option => <button className="btn" key={option.optionId} disabled={answering} onClick={() => void invoke(parent, { option: option.optionId }).catch(reason => control.setError(String(reason)))}>{option.name}</button>)}</div>
  </section>;
}

function AdapterTool({ tool }: { tool: AcpUpdate }) {
  const [open, setOpen] = useState(tool.status === "pending");
  const subject = `adapter-tool:${useId()}`;
  const revision = useUiRevision(() => [tool.toolCallId, tool.status, open]);
  useUiCommands([{ key: subject, command: { label: `Set adapter tool details visibility: ${(tool.title ?? tool.kind ?? "Agent tool").slice(0, 256)}`, parent: subject, subject, parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { open, status: tool.status ?? null } }), execute: async args => { setOpen(args.open as boolean); return { status: "completed", value: args.open }; } } }]);
  return <details className="acp-tool" open={open} onToggle={event => setOpen(event.currentTarget.open)}><summary>{tool.title ?? tool.kind ?? "Agent tool"} <span className="muted">{tool.status}</span></summary><UiActionsVisibility visible={open}><ToolDetails tool={tool} /></UiActionsVisibility></details>;
}

function ConfigControl({ scope, option, disabled, onChange }: { scope: string; option: ConfigOption; disabled: boolean; onChange: (value: string | boolean) => Promise<Operation> }) {
  const subject = `adapter-config:${useId()}`;
  const revision = useUiRevision(() => [scope, option, disabled]);
  const values = option.options?.flatMap(o => "value" in o ? [o.value] : o.options.map(v => v.value)) ?? [];
  useUiCommands([{ key: subject, command: {
    label: `Set adapter configuration: ${option.name.slice(0, 256)}`, parent: subject, subject,
    parameters: { value: option.type === "boolean" ? { type: "boolean" } : { type: "string", maxLength: 16384, choices: values } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: !disabled, value: option.currentValue }),
    execute: async (args, actor) => {
      if (actor.kind === "agent" && /permission|sandbox|approval|(?:^|_)mode(?:$|_)/i.test(`${option.category ?? ""}_${option.id}`)) return { status: "requires_input", request: `adapter-config:${scope}:${option.id}`, reason: "A human must choose adapter permission/execution policy; observe the actual configuration afterward" };
      return operationOutcome(await onChange(args.value as string | boolean));
    },
  } }]);
  if (option.type === "boolean") return <label data-tip={option.description}><input type="checkbox" checked={option.currentValue === true} disabled={disabled} onChange={e => void onChange(e.target.checked)} />{option.name}</label>;
  return <label data-tip={option.description}>{option.name}<select aria-label={option.name} value={String(option.currentValue)} disabled={disabled} onChange={e => void onChange(e.target.value)}>
    {option.options?.map(o => "value" in o ? <option key={o.value} value={o.value}>{o.name}</option> : <optgroup key={o.group} label={o.name}>{o.options.map(v => <option key={v.value} value={v.value}>{v.name}</option>)}</optgroup>)}
  </select></label>;
}

export function ToolDetails({ tool }: { tool: AcpUpdate }) {
  const subject = `adapter-tool-reading:${useId()}`;
  const revision = useUiRevision(() => tool);
  useUiCommands([{ key: subject, command: {
    label: "Read adapter tool details bytes", parent: subject, subject,
    parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }),
    execute: async args => {
      const bytes = new TextEncoder().encode(JSON.stringify(tool));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current tool details" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    },
  } }]);
  return <div className="acp-tool-details">
    {tool.locations?.map((location, i) => <p key={i}><code>{location.path}</code>{location.line !== undefined ? `:${location.line + 1}` : ""}</p>)}
    {tool._meta?.terminal_output_delta?.data && <pre>{tool._meta.terminal_output_delta.data}</pre>}
    {tool.rawOutput !== undefined && <pre>{typeof tool.rawOutput === "string" ? tool.rawOutput : JSON.stringify(tool.rawOutput, null, 2)}</pre>}
    {tool.rawInput !== undefined && <pre>{typeof tool.rawInput === "string" ? tool.rawInput : JSON.stringify(tool.rawInput, null, 2)}</pre>}
    {tool.content?.map((c, i) => c.type === "diff" ? <DiffView key={i} path={c.path ?? "document"} diff={unifiedDiff(c.path ?? "document", c.oldText ?? "", c.newText ?? "")} binary={false} staged={false} label="agent edit" /> : c.content?.text ? <pre key={i}>{c.content.text}</pre> : null)}
  </div>;
}
</book:entry>

</book:doc>
