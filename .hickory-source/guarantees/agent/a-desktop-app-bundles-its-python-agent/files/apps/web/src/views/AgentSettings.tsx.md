<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/AgentSettings.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A desktop app bundles its own Python agent](../../../../../contract.md)
- [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../../../acp-agents-are-first-class/contract.md)
- [A protected workspace save is validated against the revision it read](../../../../../../the-mounted-workspace-keeps-source-and-output-together/contract.md)

Captured SHA-256: `9f0324c9ad4f0aee42ca363b5f88cc62bf8248fbd43c7b474b04859cd96b31d7`.

<book:entry path="apps/web/src/views/AgentSettings.tsx">
import { useEffect, useState } from "react";
import { acpApi, type AgentCommand } from "../api/acp";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField } from "../lib/uiActions";

export function AgentSettings() {
  const [filesystem, setFilesystem] = useState<{ supported: boolean; bundled: boolean; message: string } | null>(null);
  const [python, setPython] = useState("");
  const [bundled, setBundled] = useState<string | null>(null);
  const [pythonAvailable, setPythonAvailable] = useState(false);
  const [pythonError, setPythonError] = useState<string | null>(null);
  const [pythonSaved, setPythonSaved] = useState(false);
  const [pythonBusy, setPythonBusy] = useState(false);
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState(false);
  useEffect(() => { void acpApi.catalogue().then(r => { setFilesystem(r.workspace_filesystem ?? null); setDraft(JSON.stringify(r.agents.filter(a => a.id !== "python").map(({ id, name, command, args, workspace_filesystem, private_prediction_args, private_prediction_auth_method }) => ({ id, name, command, args, workspace_filesystem, private_prediction_args, private_prediction_auth_method })), null, 2)); }, () => undefined); }, []);
  useEffect(() => { void acpApi.pythonSettings().then(r => { setPython(r.python); setBundled(r.bundled ?? null); setPythonAvailable(r.available); }, () => undefined); }, []);
  const savePython = async () => {
    setPythonBusy(true); setPythonSaved(false); setPythonError(null);
    try { const r = await acpApi.savePython(python.trim()); setPythonSaved(true); setBundled(r.bundled ?? null); setPythonAvailable(r.available); return { value: r }; }
    catch (e) { const error = e instanceof Error ? e.message : String(e); setPythonError(error); return { error }; }
    finally { setPythonBusy(false); }
  };
  const saveCommands = async () => {
    setBusy(true); setError(null); setSaved(false);
    try { await acpApi.save(JSON.parse(draft)); setSaved(true); return { saved: true }; }
    catch (e) { const error = e instanceof Error ? e.message : String(e); setError(error); return { error }; }
    finally { setBusy(false); }
  };
  const revision = useUiRevision(() => [filesystem, python, bundled, pythonAvailable, pythonError, pythonSaved, pythonBusy, draft, error, busy, saved]);
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], enabled = !busy && !pythonBusy) {
    commands.push({ key: `agent-settings/${id}`, command: { label, subject: "agent-settings", parent: "agent-settings", parameters, prerequisites: ["Configures actual local executables under existing host policy; reconnect/start a new thread after changes"], read: () => ({ revision: revision(), enabled }), execute } });
  }
  add("read", "Read agent settings bytes", { offset: { type: "number", minimum: 0, maximum: 100000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ filesystem, python, bundled, python_available: pythonAvailable, python_error: pythonError, python_saved: pythonSaved, python_busy: pythonBusy, commands_draft: draft, error, busy, saved }));
    const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current agent settings" };
    return { status: "completed", value: { encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], next: end < bytes.length ? end : null, total_bytes: bytes.length } };
  }, true);
  add("python-input", "Set Python runtime draft", { executable: { type: "string", maxLength: 8192 } }, async args => { setPython(String(args.executable)); setPythonSaved(false); return { status: "completed", value: { saved: false } }; });
  add("python-save", "Save and check Python runtime", {}, async () => { const result = await savePython(); return result.value ? { status: "completed", value: JSON.parse(JSON.stringify(result.value)) } : { status: "unavailable", reason: result.error ?? "Runtime unavailable" }; });
  add("commands-input", "Set ACP agent commands draft", { json: { type: "string", maxLength: 32768 } }, async args => { setDraft(String(args.json)); setSaved(false); return { status: "completed", value: { saved: false } }; });
  add("commands-save", "Save ACP agent commands", {}, async () => { const result = await saveCommands(); return result.saved ? { status: "completed", value: { saved: true, reconnect_required: true } } : { status: "unavailable", reason: result.error ?? "Commands unavailable" }; }, !!draft && !busy && !pythonBusy);
  useUiCommands(commands);
  return <section className="settings-section" aria-label="Agents">
    <h2>Agents</h2>
    <h3>Hickory Python agent</h3>
    <p>Uses the provider key in Settings, an isolated Python worker and Monty for generated code. Document edit tools require review. Registered host UI controls provide saved-document checks, setup, debugging and profiling where available; inspect actual run receipts.</p>
    <p>Choose a virtual environment with the Hickory agent package installed. <a href="https://github.com/LoumTechnologies/hickory-docs/blob/master/docs/developers/python-agent.md" target="_blank" rel="noreferrer">Runtime setup</a></p>
    {!python.trim() && bundled && <p role="status">Using the runtime bundled with this app: {bundled}</p>}
    <form onSubmit={async e => {
      e.preventDefault(); await savePython();
    }}>
      <label>Python executable<input aria-label="Python executable" value={python} placeholder={bundled ?? "Absolute path to the virtual environment’s bin/python"} onChange={e => { setPython(e.target.value); setPythonSaved(false); }} style={{ width: "100%" }} /></label>
      {!pythonSaved && !python.trim() && pythonAvailable && !bundled && <p role="status">Runtime available. Choose Hickory Python in the Agent pane.</p>}
      <button className="btn" disabled={pythonBusy}>{pythonBusy ? "Checking runtime…" : "Save Python runtime"}</button>
      {pythonSaved && <p role="status">{python.trim() ? "Runtime started in confinement. Choose Hickory Python in the Agent pane." : "Python runtime cleared."}</p>}
      {pythonError && <p className="error" role="alert">{pythonError}</p>}
    </form>
    <h3>ACP agents</h3>
    <p>Run Codex, Claude Agent, or another ACP agent in the agent pane. Each uses its own sign-in and permissions. Commands are stored on this machine.</p>
    <p>Choose an agent in the pane to install its adapter and sign in. Custom commands use an executable plus an array of arguments.</p>
    <p>Private prediction practices require separate <code>private_prediction_args</code> in the command JSON (an empty array is explicit), with no resume/history arguments and native workspace disabled. Set <code>private_prediction_auth_method</code> to an advertised method id when required (Codex uses <code>api-key</code>). Only environment API keys are inherited; ordinary sign-in files and configured model overrides are not copied. Choose its id in the private coach. The adapter must support HTTP MCP; client filesystem, terminal and permissions are refused. External harness internals are outside this managed boundary.</p>
    {filesystem?.supported && <div>
      <h3>Native agent workspace</h3>
      <p>{filesystem.bundled ? filesystem.message : "This build does not include Hickory Workspace. A signed build with the bundled filesystem extension is required."}</p>
      <p>Route native file operations through Hickory to carry generated-file edits back into their documents. Enable for each agent, save, then start a new thread.</p>
      {(() => { try { return (JSON.parse(draft) as AgentCommand[]).map((agent, i) => <label key={agent.id} style={{ display: "block" }}><input type="checkbox" checked={!!agent.workspace_filesystem} disabled={!filesystem.bundled || busy} onChange={e => { const agents: AgentCommand[] = JSON.parse(draft); agents[i].workspace_filesystem = e.target.checked; setDraft(JSON.stringify(agents, null, 2)); setSaved(false); }} /> Use native workspace for {agent.name}</label>); } catch { return null; } })()}
    </div>}
    <form onSubmit={async e => {
      e.preventDefault(); await saveCommands();
    }}>
      <label>Agent commands<textarea aria-label="Agent commands" rows={12} value={draft} onChange={e => { setDraft(e.target.value); setSaved(false); }} style={{ width: "100%", fontFamily: "monospace" }} /></label>
      <button className="btn" disabled={busy || !draft}>{busy ? "Saving…" : "Save agent commands"}</button>
      {saved && <p role="status">Saved. Reconnect the agent to use the new command.</p>}
      {error && <p className="error" role="alert">{error}</p>}
    </form>
  </section>;
}
</book:entry>

</book:doc>
