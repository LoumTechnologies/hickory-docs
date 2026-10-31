<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Separate model snippets from trusted host operations

For: Authors using an agent to change documents while retaining review control.

A model snippet cannot import a shell as a fallback. A separately registered terminal action can still run a host process under the existing host policy.

## What must happen

Model snippets execute in Monty without workspace mounts or a CPython/shell fallback. The trusted Python orchestrator runs under supported Seatbelt or Bubblewrap confinement; unavailable confinement refuses hosted execution. Python owns its conversation graph, prompts, retries, budget and completion attempts. Hickory supplies bounded transport, reviewed document edits, host operations and retained receipts. Registered window tools can expose execution, debugger, setup and filesystem effects under local policy. Stop cancels inference/review and terminates the confined process group; it does not approve a proposal or guarantee termination of an independent job already launched by a completed UI operation.

## Limits of the evidence

Trusted runtime installations and host policies remain assumptions. This is not an adversarial same-user security boundary. Standalone development has a different confinement boundary.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/python-execution-has-two-boundaries.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain both confinement layers and explicitly incorporate the later host-capability grant. Remove the implication that every possible workspace effect goes through document-edit approval.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-321d49b7f97d"></a>
## `apps/agent/src/hickory_agent_worker/__main__.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Separate model snippets from trusted host operations](contract.md).

<a id="f-321d49b7f97d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-321d49b7f97d-expression_statement">
"""Private confined worker entry point. Use hickory-agent-dev for development."""
</storycode:copy>
<a id="f-321d49b7f97d-main"></a>
### `main`

<storycode:copy id="f-321d49b7f97d-main">
from .hosted import main

</storycode:copy>
<a id="f-321d49b7f97d-if_statement"></a>
### `if_statement`

<storycode:copy id="f-321d49b7f97d-if_statement">
if __name__ == "__main__":
    main()
</storycode:copy>

### Assemble `apps/agent/src/hickory_agent_worker/__main__.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/src/hickory_agent_worker/__main__.py"><storycode:paste select="#f-321d49b7f97d-expression_statement"/><storycode:paste select="#f-321d49b7f97d-main"/><storycode:paste select="#f-321d49b7f97d-if_statement"/></storycode:file>

<a id="output-0704f5067c4f"></a>
## `apps/agent/tests/cli_cancel.py`

Choose and deliver agent context while preserving the host protocol and the evidence behind a turn.

Related contracts: [Separate model snippets from trusted host operations](contract.md).

<a id="f-0704f5067c4f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0704f5067c4f-expression_statement">
"""Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md.
Drive the real CLI's terminal review with a local provider fixture; cancel, never approve.
"""
</storycode:copy>
<a id="f-0704f5067c4f-http-server"></a>
### `http.server`

<storycode:copy id="f-0704f5067c4f-http-server">
import http.server
</storycode:copy>
<a id="f-0704f5067c4f-json"></a>
### `json`

<storycode:copy id="f-0704f5067c4f-json">
import json
</storycode:copy>
<a id="f-0704f5067c4f-os"></a>
### `os`

<storycode:copy id="f-0704f5067c4f-os">
import os
</storycode:copy>
<a id="f-0704f5067c4f-pty"></a>
### `pty`

<storycode:copy id="f-0704f5067c4f-pty">
import pty
</storycode:copy>
<a id="f-0704f5067c4f-select"></a>
### `select`

<storycode:copy id="f-0704f5067c4f-select">
import select
</storycode:copy>
<a id="f-0704f5067c4f-signal"></a>
### `signal`

<storycode:copy id="f-0704f5067c4f-signal">
import signal
</storycode:copy>
<a id="f-0704f5067c4f-subprocess"></a>
### `subprocess`

<storycode:copy id="f-0704f5067c4f-subprocess">
import subprocess
</storycode:copy>
<a id="f-0704f5067c4f-sys"></a>
### `sys`

<storycode:copy id="f-0704f5067c4f-sys">
import sys
</storycode:copy>
<a id="f-0704f5067c4f-tempfile"></a>
### `tempfile`

<storycode:copy id="f-0704f5067c4f-tempfile">
import tempfile
</storycode:copy>
<a id="f-0704f5067c4f-threading"></a>
### `threading`

<storycode:copy id="f-0704f5067c4f-threading">
import threading
</storycode:copy>
<a id="f-0704f5067c4f-time"></a>
### `time`

<storycode:copy id="f-0704f5067c4f-time">
import time
</storycode:copy>
<a id="f-0704f5067c4f-Path"></a>
### `Path`

<storycode:copy id="f-0704f5067c4f-Path">
from pathlib import Path


</storycode:copy>
<a id="f-0704f5067c4f-Provider"></a>
### `Provider`

<storycode:copy id="f-0704f5067c4f-Provider">
class Provider(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        assert self.headers['Authorization'] == 'Bearer fixture-not-a-real-key'
        self.rfile.read(int(self.headers['Content-Length']))
        body = ('data: ' + json.dumps({
            'choices': [{'delta': {'content': json.dumps({'python':
                "tool('create_doc', {'path':'proposal.md', 'input':'Cancelled proposal'})"
            })}}],
            'usage': {'prompt_tokens': 1, 'completion_tokens': 1},
        }) + '\n\ndata: [DONE]\n\n').encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass


</storycode:copy>
<a id="f-0704f5067c4f-main"></a>
### `main`

<storycode:copy id="f-0704f5067c4f-main">
def main(binary, python):
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Provider)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    master, slave = pty.openpty()
    process = None
    try:
        with tempfile.TemporaryDirectory() as root:
            env = dict(os.environ, OPENAI_API_KEY='fixture-not-a-real-key',
                       OPENAI_BASE_URL=f'http://127.0.0.1:{server.server_port}/',
                       HICKORY_KEY_STORE=str(Path(root) / 'absent-keys.json'))
            process = subprocess.Popen(
                [binary, '--python', python, '--dir', root, '--provider', 'openai',
                 '--max-turns', '2', 'Propose a new note'],
                stdin=slave, stdout=slave, stderr=slave, env=env,
                start_new_session=True,
            )
            os.close(slave)
            output = b''
            deadline = time.monotonic() + 30
            while b"Type 'approve'" not in output:
                assert time.monotonic() < deadline, 'CLI never reached terminal approval'
                ready, _, _ = select.select([master], [], [], 0.2)
                if ready:
                    output += os.read(master, 65536)
                assert len(output) < 1_048_576, 'Unexpected CLI output growth'
            process.send_signal(signal.SIGINT)
            assert process.wait(timeout=5) != 0, 'Cancellation was reported successful'
            assert not Path(root, 'proposal.md').exists(), 'Cancelled edit was applied'
            records = list(Path(root, 'sessions').glob('*.md'))
            assert records and 'Python agent cancelled' in records[0].read_text()
            print('CLI cancels promptly while waiting for terminal approval; no write occurred')
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        os.close(master)
        server.shutdown()
        server.server_close()


</storycode:copy>
<a id="f-0704f5067c4f-if_statement"></a>
### `if_statement`

<storycode:copy id="f-0704f5067c4f-if_statement">
if __name__ == '__main__':
    main(*sys.argv[1:])
</storycode:copy>

### Assemble `apps/agent/tests/cli_cancel.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/agent/tests/cli_cancel.py"><storycode:paste select="#f-0704f5067c4f-expression_statement"/><storycode:paste select="#f-0704f5067c4f-http-server"/><storycode:paste select="#f-0704f5067c4f-json"/><storycode:paste select="#f-0704f5067c4f-os"/><storycode:paste select="#f-0704f5067c4f-pty"/><storycode:paste select="#f-0704f5067c4f-select"/><storycode:paste select="#f-0704f5067c4f-signal"/><storycode:paste select="#f-0704f5067c4f-subprocess"/><storycode:paste select="#f-0704f5067c4f-sys"/><storycode:paste select="#f-0704f5067c4f-tempfile"/><storycode:paste select="#f-0704f5067c4f-threading"/><storycode:paste select="#f-0704f5067c4f-time"/><storycode:paste select="#f-0704f5067c4f-Path"/><storycode:paste select="#f-0704f5067c4f-Provider"/><storycode:paste select="#f-0704f5067c4f-main"/><storycode:paste select="#f-0704f5067c4f-if_statement"/></storycode:file>

<a id="output-8403c94b656f"></a>
## `apps/web/src/components/AcpControls.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../acp-agents-are-first-class/contract.md), [Separate model snippets from trusted host operations](contract.md).

<a id="f-8403c94b656f-defaults"></a>
### `defaults`

<storycode:copy id="f-8403c94b656f-defaults">
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

</storycode:copy>
<a id="f-8403c94b656f-useAcp"></a>
### `useAcp`

<storycode:copy id="f-8403c94b656f-useAcp">
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
    try { const next = await (backend === "python" ? acpApi.state(doc, backend, session ?? "") : acpApi.connect(doc, backend, session)); if (stamp === version.current) setState(next); return { state: next, superseded: stamp !== version.current }; }
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

</storycode:copy>
<a id="f-8403c94b656f-Controls"></a>
### `Controls`

<storycode:copy id="f-8403c94b656f-Controls">
type Controls = ReturnType<typeof useAcp>;
</storycode:copy>
<a id="f-8403c94b656f-Operation"></a>
### `Operation`

<storycode:copy id="f-8403c94b656f-Operation">
type Operation = { state?: AcpState; error?: string; superseded?: boolean };
</storycode:copy>
<a id="f-8403c94b656f-operationOutcome"></a>
### `operationOutcome`

<storycode:copy id="f-8403c94b656f-operationOutcome">
function operationOutcome(result: Operation): UiOutcome {
  if (result.error) return { status: "unavailable", reason: result.error };
  if (result.superseded) return { status: "unavailable", reason: "The selected connection changed during this host operation; inspect the originating adapter before retrying because it may have completed" };
  return { status: "completed", value: { backend: result.state?.backend ?? null, ready: result.state?.ready ?? false, error: result.state?.error ?? null, session: result.state?.session ?? null, scope: "Returned host state; no provider turn or tool completion certified" } };
}
</storycode:copy>
<a id="f-8403c94b656f-AcpControls"></a>
### `AcpControls`

<storycode:copy id="f-8403c94b656f-AcpControls">
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
    if (state?.edits?.mode === "auto-accept") add("review-policy", "Require document edit approval again", {}, !busy && !running, async () => operationOutcome(await control.act(() => acpApi.edits(doc, { mode: "review" }, backend))));
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
  const reviewPolicy = settingsOpen && state?.edits?.mode === "auto-accept" && <button className="btn-link" disabled={busy || running} onClick={() => click("review-policy")}>Require document edit approval again</button>;
  if (backend === "python") return <div className="acp-controls" aria-label="Python agent controls">
    {reviewPolicy}
    {(error || state?.error) && <p role="alert" className="chat-note error">{error || state?.error}</p>}
    {settingsOpen && !state?.ready && <p>Set the Python runtime in <a href="#/settings">Settings → Agents</a>, then <button className="btn-link" disabled={busy} onClick={() => click("connect")}>Check runtime</button>.</p>}
  </div>;
  const agent = agents.find(a => a.id === backend);
  return <div className="acp-controls" aria-label="Agent controls">
    {settingsOpen && <div className="acp-settings">
      {reviewPolicy}
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

</storycode:copy>
<a id="f-8403c94b656f-PermissionControl"></a>
### `PermissionControl`

<storycode:copy id="f-8403c94b656f-PermissionControl">
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

</storycode:copy>
<a id="f-8403c94b656f-AdapterTool"></a>
### `AdapterTool`

<storycode:copy id="f-8403c94b656f-AdapterTool">
function AdapterTool({ tool }: { tool: AcpUpdate }) {
  const [open, setOpen] = useState(tool.status === "pending");
  const subject = `adapter-tool:${useId()}`;
  const revision = useUiRevision(() => [tool.toolCallId, tool.status, open]);
  useUiCommands([{ key: subject, command: { label: `Set adapter tool details visibility: ${(tool.title ?? tool.kind ?? "Agent tool").slice(0, 256)}`, parent: subject, subject, parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { open, status: tool.status ?? null } }), execute: async args => { setOpen(args.open as boolean); return { status: "completed", value: args.open }; } } }]);
  return <details className="acp-tool" open={open} onToggle={event => setOpen(event.currentTarget.open)}><summary>{tool.title ?? tool.kind ?? "Agent tool"} <span className="muted">{tool.status}</span></summary><UiActionsVisibility visible={open}><ToolDetails tool={tool} /></UiActionsVisibility></details>;
}

</storycode:copy>
<a id="f-8403c94b656f-ConfigControl"></a>
### `ConfigControl`

<storycode:copy id="f-8403c94b656f-ConfigControl">
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

</storycode:copy>
<a id="f-8403c94b656f-ToolDetails"></a>
### `ToolDetails`

<storycode:copy id="f-8403c94b656f-ToolDetails">
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
</storycode:copy>

### Assemble `apps/web/src/components/AcpControls.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AcpControls.tsx"><storycode:paste select="#f-8403c94b656f-defaults"/><storycode:paste select="#f-8403c94b656f-useAcp"/><storycode:paste select="#f-8403c94b656f-Controls"/><storycode:paste select="#f-8403c94b656f-Operation"/><storycode:paste select="#f-8403c94b656f-operationOutcome"/><storycode:paste select="#f-8403c94b656f-AcpControls"/><storycode:paste select="#f-8403c94b656f-PermissionControl"/><storycode:paste select="#f-8403c94b656f-AdapterTool"/><storycode:paste select="#f-8403c94b656f-ConfigControl"/><storycode:paste select="#f-8403c94b656f-ToolDetails"/></storycode:file>

## Test the behavior: `indirect_execution_and_outside_proposals_are_not_capabilities`

This is the maintained test fragment from `crates/hickory-cli/tests/python_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-9bda4649fd61-indirect_execution_and_outside_proposals_are_not_capabilities"></a>
### `indirect_execution_and_outside_proposals_are_not_capabilities`

<storycode:copy id="f-9bda4649fd61-indirect_execution_and_outside_proposals_are_not_capabilities">
// Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
#[tokio::test]
async fn indirect_execution_and_outside_proposals_are_not_capabilities() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let destination = outside.path().join("escaped.md");
    let client = scripted([
        json!({"python":"tool('debug_start', {})"}).to_string(),
        json!({"python":"tool('begin_changeset', {'workspace':'/'})"}).to_string(),
        json!({"python":"tool('write_file', {'path':'hacked', 'content':'bypass'})"}).to_string(),
        json!({"python":format!("tool('create_doc', {{'path':{}, 'input':'AI proposal'}})", serde_json::to_string(&destination.display().to_string()).unwrap())}).to_string(),
        json!({"done":"The bypasses were refused"}).to_string(),
    ]);
    let mut review = FixedReview {
        accept: true,
        count: 0,
    };
    run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Attempt review bypasses".into(),
            max_turns: 6,
        },
        &client,
        &mut review,
    )
    .await
    .unwrap();
    assert_eq!(review.count, 0);
    assert!(!destination.exists());
    assert!(!root.path().join("hacked").exists());
}

</storycode:copy>

## Test the behavior: `approved_executable_document_content_does_not_execute_during_reads_or_completion`

This is the maintained test fragment from `crates/hickory-cli/tests/python_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-9bda4649fd61-approved_executable_document_content_does_not_execute_during_reads_or_completion"></a>
### `approved_executable_document_content_does_not_execute_during_reads_or_completion`

<storycode:copy id="f-9bda4649fd61-approved_executable_document_content_does_not_execute_during_reads_or_completion">
// Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
#[tokio::test]
async fn approved_executable_document_content_does_not_execute_during_reads_or_completion() {
    let Some(python) = python() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let marker = root.path().join("unreviewed-output");
    let source = format!(
        r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="work" image="host" />
<hick:exec container="work">touch {}</hick:exec>
</hick:doc>
"#,
        marker.display()
    );
    let client = scripted([
        json!({"python":format!("tool('create_doc', {{'path':'executable.md', 'input':{}}})", serde_json::to_string(&source).unwrap())}).to_string(),
        json!({"python":"tool('read_doc', {'doc':'executable.md'})"}).to_string(),
        json!({"python":"tool('verify', {'doc':'executable.md'})"}).to_string(),
        json!({"done":"Source artifact only; native execution unrun"}).to_string(),
    ]);
    struct Approve(usize);
    #[async_trait::async_trait]
    impl Review for Approve {
        async fn approve(&mut self, _: &Proposal) -> Result<bool> {
            self.0 += 1;
            Ok(true)
        }
    }
    let mut review = Approve(0);
    run(
        Config {
            python,
            root: root.path().into(),
            doc: None,
            prompt: "Propose executable content without running it".into(),
            max_turns: 5,
        },
        &client,
        &mut review,
    )
    .await
    .unwrap();
    assert_eq!(review.0, 1);
    assert_eq!(
        std::fs::read_to_string(root.path().join("executable.md")).unwrap(),
        source
    );
    assert!(
        !marker.exists(),
        "authored commands must not bypass edit review"
    );
}
</storycode:copy>

<a id="output-5226e9a5ffd6"></a>
## `crates/hickory-executor-sandbox/src/agent_process.rs`

Related contracts: [Separate model snippets from trusted host operations](contract.md).

<a id="f-5226e9a5ffd6-quoted"></a>
### `quoted`

A separate allowlist for trusted Python orchestration, not the cell profile.
Workspace and provider access happen only through the parent process's pipe.

<storycode:copy id="f-5226e9a5ffd6-quoted">
//! A separate allowlist for trusted Python orchestration, not the cell profile.
//! Workspace and provider access happen only through the parent process's pipe.
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::policy::Sandbox;

fn quoted(path: &Path) -> Result<String> {
    let path = path.to_str().context("Agent runtime paths must be UTF-8")?;
    Ok(format!(
        "\"{}\"",
        path.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

</storycode:copy>
<a id="f-5226e9a5ffd6-runtime_roots"></a>
### `runtime_roots`

Resolve an installed venv, not an editable checkout or a shell command.

<storycode:copy id="f-5226e9a5ffd6-runtime_roots">
/// Resolve an installed venv, not an editable checkout or a shell command.
pub fn runtime_roots(python: &Path) -> Result<Vec<PathBuf>> {
    ensure!(
        python.is_absolute(),
        "Pass an absolute path to a venv Python"
    );
    let bin = python.parent().context("Python has no parent")?;
    let venv = bin
        .parent()
        .context("Python has no environment directory")?;
    ensure!(
        venv.join("pyvenv.cfg").is_file(),
        "Python must belong to an installed virtual environment"
    );
    ensure!(
        bin.join("monty").is_file(),
        "Install the locked Hickory agent environment first (Monty is missing)"
    );
    let real_python = python.canonicalize()?;
    let base = real_python
        .parent()
        .and_then(Path::parent)
        .context("Python has no runtime root")?;
    let roots = vec![venv.canonicalize()?, base.to_path_buf()];
    for root in &roots {
        ensure!(
            root != Path::new("/") && root != Path::new("/Users") && root != Path::new("/home"),
            "Runtime read root is too broad"
        );
    }
    Ok(roots)
}

</storycode:copy>
<a id="f-5226e9a5ffd6-command"></a>
### `command`

No shell, inherited environment, inherited cwd, network grant or fallback.

<storycode:copy id="f-5226e9a5ffd6-command">
/// No shell, inherited environment, inherited cwd, network grant or fallback.
pub fn command(python: &Path, scratch: &Path) -> Result<Command> {
    let roots = runtime_roots(python)?;
    if let Some(home) = std::env::var_os("HOME") {
        for root in &roots {
            ensure!(
                !Path::new(&home).starts_with(root),
                "Runtime root would expose the user's home"
            );
        }
    }
    let scratch = scratch.canonicalize()?;
    let args = ["-I", "-B", "-m", "hickory_agent_worker"];
    let mut command = match Sandbox::detect() {
        Sandbox::Seatbelt => {
            let mut profile = String::from(
                "(version 1)(deny default)(allow process*)(allow signal (target self))\
                 (allow sysctl-read)(allow file-read-metadata)\
                 (allow file-read* (subpath \"/usr\") (subpath \"/System/Library\")\
                 (subpath \"/System/Cryptexes/OS\") (subpath \"/System/Volumes/Preboot/Cryptexes/OS\")\
                 (subpath \"/Library/Apple\")\
                 (literal \"/\") (literal \"/dev/null\") (literal \"/dev/urandom\") (literal \"/dev/random\"))\
                 (allow file-write* (literal \"/dev/null\"))",
            );
            for root in roots {
                profile.push_str(&format!("(allow file-read* (subpath {}))", quoted(&root)?));
            }
            profile.push_str(&format!(
                "(allow file-read* file-write* (subpath {}))",
                quoted(&scratch)?
            ));
            let mut cmd = Command::new("/usr/bin/sandbox-exec");
            cmd.args(["-p", &profile]).arg(python).args(args);
            cmd
        }
        Sandbox::Bubblewrap => {
            let mut cmd = Command::new("bwrap");
            cmd.args([
                "--die-with-parent",
                "--new-session",
                "--unshare-all",
                "--clearenv",
                "--proc",
                "/proc",
                "--dev",
                "/dev",
                "--tmpfs",
                "/tmp",
            ]);
            for path in ["/usr", "/lib", "/lib64", "/bin", "/etc/ld.so.cache"] {
                if Path::new(path).exists() {
                    cmd.arg("--ro-bind").arg(path).arg(path);
                }
            }
            for root in roots {
                cmd.arg("--ro-bind").arg(&root).arg(&root);
            }
            cmd.arg("--bind")
                .arg(&scratch)
                .arg(&scratch)
                .arg("--chdir")
                .arg(&scratch)
                .args(["--setenv", "LANG", "C.UTF-8", "--setenv", "HOME"])
                .arg(&scratch)
                .args(["--setenv", "TMPDIR"])
                .arg(&scratch)
                .arg("--")
                .arg(python)
                .args(args);
            cmd
        }
        Sandbox::AppContainer => bail!(
            "The Python agent process profile is not implemented on Windows; refusing unconfined execution"
        ),
        Sandbox::None => {
            bail!("No working OS sandbox for the Python agent; refusing unconfined execution")
        }
    };
    command
        .env_clear()
        .env("LANG", "C.UTF-8")
        .env("HOME", &scratch)
        .env("TMPDIR", &scratch)
        .current_dir(&scratch);
    // Descendants inherit confinement. The supervisor kills this process group
    // on every exit, including protocol errors and timeout, not just Python.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    Ok(command)
}

</storycode:copy>
<a id="f-5226e9a5ffd6-tests"></a>
### `tests`

<storycode:copy id="f-5226e9a5ffd6-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/python-execution-has-two-boundaries.md
    #[test]
    fn a_general_python_or_relative_command_is_not_a_runtime() {
        assert!(runtime_roots(Path::new("python3")).is_err());
        assert!(runtime_roots(Path::new("/usr/bin/python3")).is_err());
    }
    #[test]
    fn profile_paths_cannot_inject_rules() {
        assert_eq!(
            quoted(Path::new("/tmp/a\"b\\c")).unwrap(),
            "\"/tmp/a\\\"b\\\\c\""
        );
    }
}
</storycode:copy>

### Assemble `crates/hickory-executor-sandbox/src/agent_process.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-sandbox/src/agent_process.rs"><storycode:paste select="#f-5226e9a5ffd6-quoted"/><storycode:paste select="#f-5226e9a5ffd6-runtime_roots"/><storycode:paste select="#f-5226e9a5ffd6-command"/><storycode:paste select="#f-5226e9a5ffd6-tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/agent/src/hickory_agent_worker/__main__.py](contract.md#output-321d49b7f97d)
- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/contract.md#output-afffc8ba20eb)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/hosted.py.md#output-0e03aa3e36b2)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/runtime.py.md#output-56e1a215e79c)
- [apps/web/src/components/AcpControls.tsx](contract.md#output-8403c94b656f)
- [apps/web/src/components/AgentChanges.tsx](../../../mechanisms/apps/web/src/components/AgentChanges.tsx.md#output-d82ea3dcdcf9)
- [crates/hickory-agent/src/context.rs](../../../mechanisms/crates/hickory-agent/src/context.rs.md#output-92769b9184f8)
- [crates/hickory-cli/src/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/python_agent.rs.md#output-609d98dd1fad)
- [crates/hickory-cli/src/serve/acp/edits.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/edits.rs.md#output-076ee922e67a)
- [crates/hickory-cli/src/serve/agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent.rs.md#output-2ce16640df92)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/python_agent.rs.md#output-89d47ddc390d)
- [crates/hickory-executor-sandbox/src/agent_process.rs](contract.md#output-5226e9a5ffd6)

### Tests and fixtures

- [apps/agent/tests/cli_cancel.py](contract.md#output-0704f5067c4f)
- [apps/agent/tests/test_worker.py](../../../mechanisms/apps/agent/tests/test_worker.py.md#output-9f33e52791d8)
- [apps/web/e2e/python-agent.spec.ts](../../../mechanisms/apps/web/e2e/python-agent.spec.ts.md#output-c40e3ff7e23b)
- [apps/web/src/components/AgentChanges.test.tsx](../../../mechanisms/apps/web/src/components/AgentChanges.tsx.md#output-7ab5cdb69360)
- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- [apps/web/src/views/AgentSettings.test.tsx](../../../mechanisms/apps/web/src/views/AgentSettings.test.tsx.md#output-c3e11636cd88)
- [crates/hickory-cli/tests/python_agent.rs](../../../mechanisms/crates/hickory-cli/tests/python_agent.rs.md#output-9bda4649fd61)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_python_agent.rs.md#output-08f0cd3133cc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
