import { useCallback, useEffect, useRef, useState } from "react";
import { DiffView } from "./DiffView";
import { unifiedDiff } from "../lib/diff";
import { acpApi, type AcpState, type AcpUpdate, type AgentCommand, type ConfigOption } from "../api/acp";

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
  const connect = useCallback(async () => {
    const stamp = ++version.current;
    setBusy(true); setError(null); setState(null);
    try { const next = await acpApi.connect(doc, backend, session); if (stamp === version.current) setState(next); }
    catch (e) { if (stamp === version.current) setError(e instanceof Error ? e.message : String(e)); }
    finally { if (stamp === version.current) setBusy(false); }
  }, [doc, backend, session]);
  useEffect(() => {
    if (backend === "builtin") { ++version.current; setState(null); setError(null); setBusy(false); return; }
    if (hydrated && !running) void connect();
    // A running turn owns its connection; do not reconnect on optimistic UI updates.
  }, [connect, backend, hydrated]);
  useEffect(() => {
    if (backend === "builtin" || (!running && !state?.ready)) return;
    const stamp = version.current;
    let live = true;
    const poll = () => { void Promise.resolve(acpApi.state(doc)).then(s => { if (s && live && stamp === version.current) setState(s); }, () => undefined); };
    poll(); const timer = setInterval(poll, running ? 500 : 1500);
    return () => { live = false; clearInterval(timer); };
  }, [running, backend, doc, state?.ready]);
  const act = async (operation: () => Promise<AcpState>) => {
    const stamp = ++version.current;
    setBusy(true); setError(null);
    try { const next = await operation(); if (stamp === version.current) setState(next); }
    catch (e) { if (stamp === version.current) setError(e instanceof Error ? e.message : String(e)); }
    finally { if (stamp === version.current) setBusy(false); }
  };
  const install = async () => {
    const stamp = ++version.current;
    setBusy(true); setError(null);
    try {
      setAgents((await acpApi.install(backend)).agents);
      if (stamp === version.current) await connect();
    }
    catch (e) { if (stamp === version.current) setError(e instanceof Error ? e.message : String(e)); }
    finally { if (stamp === version.current) setBusy(false); }
  };
  return { agents, state, busy, error, connect, install, act, setState, setError, refreshAgents, detecting, catalogueError };
}

type Controls = ReturnType<typeof useAcp>;
export function AcpControls({ doc, backend, running, control, settingsOpen = true, settingsOnly = false }: { doc: string; backend: string; running: boolean; control: Controls; settingsOpen?: boolean; settingsOnly?: boolean }) {
  if (backend === "builtin") return null;
  const { state, busy, error, agents } = control;
  const agent = agents.find(a => a.id === backend);
  return <div className="acp-controls" aria-label="Agent controls">
    {settingsOpen && <div className="acp-settings">
      {!state?.configOptions?.length && state?.modes && <label>Mode<select aria-label="Mode" value={state.modes.currentModeId} disabled={busy || running}
        onChange={e => void control.act(() => acpApi.configure(doc, "__mode", e.target.value))}>
        {state.modes.availableModes.map(mode => <option key={mode.id} value={mode.id}>{mode.name}</option>)}
      </select></label>}
      {state?.configOptions?.map(option => <ConfigControl key={option.id} option={option} disabled={busy || running}
        onChange={value => void control.act(() => acpApi.configure(doc, option.id, value))} />)}
      {!state?.ready && !busy && agent?.installable && !agent.available &&
        <button className="btn" onClick={() => void control.install()}>Install {agent.name} adapter</button>}
      {!state?.ready && !busy && state?.authMethods?.filter(m => !m.type || m.type === "agent").map(m =>
        <button className="btn" key={m.id} data-tip={m.description} onClick={() => void control.act(() => acpApi.authenticate(doc, m.id))}>{m.name}</button>)}
      {!running && <button className="btn-link" disabled={busy} onClick={() => void control.connect()}>{busy ? "Connecting…" : "Reconnect"}</button>}
    </div>}
    {!settingsOnly && busy && <p className="muted" role="status">Connecting to {agent?.name ?? backend}… Sign-in may open in your browser.</p>}
    {!settingsOnly && (error || state?.error) && <p role="alert" className="chat-note error">{error || state?.error}</p>}
    {settingsOpen && !busy && agent?.available === false && <p className="muted">{agent.cli_available ? `${agent.name} CLI was found, but its ACP adapter is still needed. ` : ""}{agent.installable ? "Install the adapter above to connect." : "Install Node.js and npm to install a known adapter, or set its executable in Settings → Agents."}</p>}
    {settingsOpen && !busy && !state?.ready && <p className="muted">Use your agent’s own account or API key. Agent commands follow that agent’s permissions and sandbox.</p>}
    {settingsOpen && state?.commands?.length ? <details className="acp-commands"><summary>Agent commands</summary>
      {state.commands.map(c => <p key={c.name}><code>/{c.name}</code> — {c.description}</p>)}</details> : null}
    {!settingsOnly && state?.permissions?.map(permission => <section key={permission.id} className="acp-permission" role="group" aria-label="Agent permission">
      <strong>{permission.toolCall.title ?? "The agent needs your permission"}</strong>
      <ToolDetails tool={permission.toolCall} />
      <div className="acp-settings">{permission.options.map(option =>
        <button className="btn" key={option.optionId} onClick={async () => {
          try { await acpApi.permission(doc, permission.id, option.optionId); control.setState(s => s ? { ...s, permissions: s.permissions?.filter(p => p.id !== permission.id) } : s); }
          catch (e) { control.setError(e instanceof Error ? e.message : String(e)); }
        }}>{option.name}</button>)}</div>
    </section>)}
    {!settingsOnly && running && state?.tools?.map(tool => <details key={tool.toolCallId} className="acp-tool" open={tool.status === "pending"}>
      <summary>{tool.title ?? tool.kind ?? "Agent tool"} <span className="muted">{tool.status}</span></summary><ToolDetails tool={tool} />
    </details>)}
  </div>;
}

function ConfigControl({ option, disabled, onChange }: { option: ConfigOption; disabled: boolean; onChange: (value: string | boolean) => void }) {
  if (option.type === "boolean") return <label data-tip={option.description}><input type="checkbox" checked={option.currentValue === true} disabled={disabled} onChange={e => onChange(e.target.checked)} />{option.name}</label>;
  return <label data-tip={option.description}>{option.name}<select aria-label={option.name} value={String(option.currentValue)} disabled={disabled} onChange={e => onChange(e.target.value)}>
    {option.options?.map(o => "value" in o ? <option key={o.value} value={o.value}>{o.name}</option> : <optgroup key={o.group} label={o.name}>{o.options.map(v => <option key={v.value} value={v.value}>{v.name}</option>)}</optgroup>)}
  </select></label>;
}

export function ToolDetails({ tool }: { tool: AcpUpdate }) {
  return <div className="acp-tool-details">
    {tool.locations?.map((location, i) => <p key={i}><code>{location.path}</code>{location.line !== undefined ? `:${location.line + 1}` : ""}</p>)}
    {tool._meta?.terminal_output_delta?.data && <pre>{tool._meta.terminal_output_delta.data}</pre>}
    {tool.rawOutput !== undefined && <pre>{typeof tool.rawOutput === "string" ? tool.rawOutput : JSON.stringify(tool.rawOutput, null, 2)}</pre>}
    {tool.rawInput !== undefined && <pre>{typeof tool.rawInput === "string" ? tool.rawInput : JSON.stringify(tool.rawInput, null, 2)}</pre>}
    {tool.content?.map((c, i) => c.type === "diff" ? <DiffView key={i} path={c.path ?? "document"} diff={unifiedDiff(c.path ?? "document", c.oldText ?? "", c.newText ?? "")} binary={false} staged={false} label="agent edit" /> : c.content?.text ? <pre key={i}>{c.content.text}</pre> : null)}
  </div>;
}
