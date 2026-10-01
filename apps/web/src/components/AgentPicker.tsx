import type { AgentCommand } from "../api/acp";

const preferenceKey = "hickory.agent";
export function preferredAgent(): string {
  try { return localStorage.getItem(preferenceKey) || "builtin"; }
  catch { return "builtin"; }
}
export function rememberAgent(backend: string) {
  try { localStorage.setItem(preferenceKey, backend); } catch { /* Storage is optional. */ }
}

export function AgentPicker({ backend, agents, disabled, detecting, error, onChange, onRefresh }: {
  backend: string; agents: AgentCommand[]; disabled: boolean; detecting: boolean;
  error: string | null; onChange: (backend: string) => void; onRefresh: () => void;
}) {
  const installed = agents.filter(a => a.available);
  const missing = agents.filter(a => !a.available);
  return <div className="acp-controls">
    <div className="acp-settings">
      <label>Agent<select aria-label="Agent" value={backend} disabled={disabled} onChange={e => onChange(e.target.value)}>
        <option value="builtin">Hickory (built-in)</option>
        {backend !== "builtin" && !agents.some(a => a.id === backend) && <option value={backend}>{backend} (not configured)</option>}
        {installed.length > 0 && <optgroup label="Installed ACP agents">{installed.map(a => <option key={a.id} value={a.id}>{a.name} (ACP)</option>)}</optgroup>}
        {missing.length > 0 && <optgroup label="More agents">{missing.map(a => <option key={a.id} value={a.id}>{a.name} — {a.installable ? "install adapter" : "adapter not found"}</option>)}</optgroup>}
      </select></label>
      <button className="btn-link" disabled={disabled || detecting} onClick={onRefresh}>{detecting ? "Checking agents…" : "Refresh agents"}</button>
    </div>
    {error && <p className="chat-note error" role="alert">{error}</p>}
    <p className="muted">{backend === "builtin" ? "Hickory’s built-in agent uses a provider API key. Choose an ACP agent to use its own login."
      : `${agents.find(a => a.id === backend)?.name ?? backend} connects through ACP using its own login. Switching agents starts a new thread; previous conversations stay in the tree.`}</p>
  </div>;
}
