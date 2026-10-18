import type { AgentCommand } from "../api/acp";

const preferenceKey = "hickory.agent";
export function preferredAgent(): string {
  try { return (localStorage.getItem(preferenceKey) === "builtin" ? "python" : localStorage.getItem(preferenceKey)) || "python"; }
  catch { return "python"; }
}
export function rememberAgent(backend: string) {
  try { localStorage.setItem(preferenceKey, backend); } catch { /* Storage is optional. */ }
}

export function AgentPicker({ backend, agents, disabled, detecting, error, onChange, onRefresh }: {
  backend: string; agents: AgentCommand[]; disabled: boolean; detecting: boolean;
  error: string | null; onChange: (backend: string) => void; onRefresh: () => void;
}) {
  const python = agents.find(a => a.id === "python");
  const installed = agents.filter(a => a.available && a.id !== "python");
  const missing = agents.filter(a => !a.available && a.id !== "python");
  return <div className="acp-controls">
    <div className="acp-settings">
      <label>Agent<select aria-label="Agent" value={backend} disabled={disabled} onChange={e => onChange(e.target.value)}>
        {python && <option value="python">{python.name}{python.available ? "" : " — configure runtime"}</option>}
        {!agents.some(a => a.id === backend) && <option value={backend}>{backend} (not configured)</option>}
        {installed.length > 0 && <optgroup label="Installed ACP agents">{installed.map(a => <option key={a.id} value={a.id}>{a.name} (ACP)</option>)}</optgroup>}
        {missing.length > 0 && <optgroup label="More agents">{missing.map(a => <option key={a.id} value={a.id}>{a.name} — {a.installable ? "install adapter" : "adapter not found"}</option>)}</optgroup>}
      </select></label>
      <button className="btn-link" disabled={disabled || detecting} onClick={onRefresh}>{detecting ? "Checking agents…" : "Refresh agents"}</button>
    </div>
    {error && <p className="chat-note error" role="alert">{error}</p>}
    <p className="muted">{backend === "python" ? "The Python agent uses your provider API key, confined Python and Monty. Each edit needs approval; checks remain unrun. Switching agents starts a new thread."
      : `${agents.find(a => a.id === backend)?.name ?? backend} connects through ACP using its own login. Switching agents starts a new thread; previous conversations stay in the tree.`}</p>
  </div>;
}
