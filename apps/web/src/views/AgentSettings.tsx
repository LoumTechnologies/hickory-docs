import { useEffect, useState } from "react";
import { acpApi, type AgentCommand } from "../api/acp";

export function AgentSettings() {
  const [filesystem, setFilesystem] = useState<{ supported: boolean; bundled: boolean; message: string } | null>(null);
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState(false);
  useEffect(() => { void acpApi.catalogue().then(r => { setFilesystem(r.workspace_filesystem ?? null); setDraft(JSON.stringify(r.agents.map(({ id, name, command, args, workspace_filesystem }) => ({ id, name, command, args, workspace_filesystem })), null, 2)); }, () => undefined); }, []);
  return <section className="settings-section" aria-label="Agents">
    <h2>Agents</h2>
    <p>Run Codex, Claude Agent, or another ACP agent in the agent pane. Each uses its own sign-in and permissions. Commands are stored on this machine.</p>
    <p>Choose an agent in the pane to install its adapter and sign in. Custom commands use an executable plus an array of arguments.</p>
    {filesystem?.supported && <div>
      <h3>Native agent workspace</h3>
      <p>{filesystem.bundled ? filesystem.message : "This build does not include Hickory Workspace. A signed build with the bundled filesystem extension is required."}</p>
      <p>Route native file operations through Hickory to carry generated-file edits back into their documents. Enable for each agent, save, then start a new thread.</p>
      {(() => { try { return (JSON.parse(draft) as AgentCommand[]).map((agent, i) => <label key={agent.id} style={{ display: "block" }}><input type="checkbox" checked={!!agent.workspace_filesystem} disabled={!filesystem.bundled || busy} onChange={e => { const agents: AgentCommand[] = JSON.parse(draft); agents[i].workspace_filesystem = e.target.checked; setDraft(JSON.stringify(agents, null, 2)); setSaved(false); }} /> Use native workspace for {agent.name}</label>); } catch { return null; } })()}
    </div>}
    <form onSubmit={async e => {
      e.preventDefault(); setBusy(true); setError(null); setSaved(false);
      try { await acpApi.save(JSON.parse(draft)); setSaved(true); }
      catch (e) { setError(e instanceof Error ? e.message : String(e)); }
      finally { setBusy(false); }
    }}>
      <label>Agent commands<textarea aria-label="Agent commands" rows={12} value={draft} onChange={e => { setDraft(e.target.value); setSaved(false); }} style={{ width: "100%", fontFamily: "monospace" }} /></label>
      <button className="btn" disabled={busy || !draft}>{busy ? "Saving…" : "Save agent commands"}</button>
      {saved && <p role="status">Saved. Reconnect the agent to use the new command.</p>}
      {error && <p className="error" role="alert">{error}</p>}
    </form>
  </section>;
}
