<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/AgentPicker.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../../../../agent/acp-agents-are-first-class/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `e4f94c64b590f3d9ef0e18fa56ecacd6d49ab1d3e934a1afea5a8fd8cdc2b4b1`.

<book:entry path="apps/web/src/components/AgentPicker.tsx">
import { useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { AgentCommand } from "../api/acp";

const preferenceKey = "hickory.agent";
export function preferredAgent(): string {
  try { return (localStorage.getItem(preferenceKey) === "builtin" ? "python" : localStorage.getItem(preferenceKey)) || "python"; }
  catch { return "python"; }
}
export function rememberAgent(backend: string) {
  try { localStorage.setItem(preferenceKey, backend); } catch { /* Storage is optional. */ }
}

export function AgentPicker({ scope, backend, agents, disabled, detecting, error, onChange, onRefresh }: {
  scope: string; backend: string; agents: AgentCommand[]; disabled: boolean; detecting: boolean;
  error: string | null; onChange: (backend: string) => void; onRefresh: () => void | Promise<void>;
}) {
  const parent = `agent-picker:${useId()}`;
  const owner = useRef({ scope });
  if (owner.current.scope !== scope) owner.current = { scope };
  const [reported, setReported] = useState<{ owner: object; message: string } | null>(null);
  const interactionError = reported?.owner === owner.current ? reported.message : "";
  const perform = (key: string, args: Record<string, import("../lib/uiActions").UiValue> = {}) => {
    const captured = owner.current; setReported(null);
    void invoke(key, args).catch(reason => { if (owner.current === captured) setReported({ owner: captured, message: String(reason) }); });
  };
  const revision = useUiRevision(() => [scope, backend, agents, disabled, detecting, error, interactionError]);
  const invoke = useUiCommands([
    { key: `${parent}:choose`, command: { label: "Select conversation agent", parent, subject: parent, parameters: { backend: { type: "string", maxLength: 4096, choices: [...new Set([backend, ...agents.map(a => a.id)])] } }, prerequisites: ["Switching starts a new thread; inspect actual connection status before sending"], read: () => ({ revision: revision(), enabled: !disabled, value: backend, reason: disabled ? "Conversation is running or starting" : undefined }), execute: async args => { onChange(args.backend as string); return { status: "completed", value: { backend_requested: args.backend, connection: "Observe actual agent readiness; selection is not a completed login" } }; } } },
    { key: `${parent}:refresh`, command: { label: "Refresh conversation agent catalogue", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !disabled && !detecting }), execute: async () => { await onRefresh(); return { status: "completed", value: { refresh_returned: true, interpretation: "Read current catalogue and error; availability is not a configured login" } }; } } },
    { key: `${parent}:read`, command: { label: "Read conversation agent catalogue bytes", parent, subject: parent, parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ backend, agents, disabled, detecting, error, interactionError }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current agent catalogue" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    } } },
  ]);
  const python = agents.find(a => a.id === "python");
  const installed = agents.filter(a => a.available && a.id !== "python");
  const missing = agents.filter(a => !a.available && a.id !== "python");
  return <div className="acp-controls">
    <div className="acp-settings">
      <label>Agent<select aria-label="Agent" value={backend} disabled={disabled} onChange={e => perform(`${parent}:choose`, { backend: e.target.value })}>
        {python && <option value="python">{python.name}{python.available ? "" : " — configure runtime"}</option>}
        {!agents.some(a => a.id === backend) && <option value={backend}>{backend} (not configured)</option>}
        {installed.length > 0 && <optgroup label="Installed ACP agents">{installed.map(a => <option key={a.id} value={a.id}>{a.name} (ACP)</option>)}</optgroup>}
        {missing.length > 0 && <optgroup label="More agents">{missing.map(a => <option key={a.id} value={a.id}>{a.name} — {a.installable ? "install adapter" : "adapter not found"}</option>)}</optgroup>}
      </select></label>
      <button className="btn-link" disabled={disabled || detecting} onClick={() => perform(`${parent}:refresh`)}>{detecting ? "Checking agents…" : "Refresh agents"}</button>
    </div>
    {interactionError && <p className="chat-note error" role="alert">{interactionError}</p>}
    {error && <p className="chat-note error" role="alert">{error}</p>}
    <p className="muted">{backend === "python" ? "The Python agent uses your provider API key and confined Python. Document-edit previews require approval; shared UI actions use this host’s existing permissions and can run programs or change files. Checks remain unrun. Switching agents starts a new thread."
      : `${agents.find(a => a.id === backend)?.name ?? backend} connects through ACP using its own login. Switching agents starts a new thread; previous conversations stay in the tree.`}</p>
  </div>;
}
</book:entry>

</book:doc>
