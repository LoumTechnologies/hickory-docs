<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/CommandPathSettings.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-83bac3568d9f"></a>
## `apps/web/src/views/CommandPathSettings.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-83bac3568d9f-CommandPathStatus"></a>
### `CommandPathStatus`

<storycode:copy id="f-83bac3568d9f-CommandPathStatus">
import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiOutcome } from "../lib/uiActions";
import { request } from "../api/client";

export interface CommandPathStatus {
  available: boolean;
  installed: boolean;
  can_remove: boolean;
  command: string;
  source: string;
  conflict: string | null;
  message: string;
}
</storycode:copy>
<a id="f-83bac3568d9f-endpoint"></a>
### `endpoint`

<storycode:copy id="f-83bac3568d9f-endpoint">
const endpoint = "/api/settings/command-path";
</storycode:copy>
<a id="f-83bac3568d9f-readCommandPath"></a>
### `readCommandPath`

<storycode:copy id="f-83bac3568d9f-readCommandPath">
export const readCommandPath = () => request<CommandPathStatus>("GET", endpoint);
</storycode:copy>
<a id="f-83bac3568d9f-capability"></a>
### `capability`

Whether the desktop supplies this action cannot change during a session.
Cache that capability so typing in the command palette does not repeatedly
start Windows PATH discovery. Settings still reads fresh installation state.

<storycode:copy id="f-83bac3568d9f-capability">
// Whether the desktop supplies this action cannot change during a session.
// Cache that capability so typing in the command palette does not repeatedly
// start Windows PATH discovery. Settings still reads fresh installation state.
let capability: Promise<boolean> | null = null;
</storycode:copy>
<a id="f-83bac3568d9f-hasCommandInstaller"></a>
### `hasCommandInstaller`

<storycode:copy id="f-83bac3568d9f-hasCommandInstaller">
export const hasCommandInstaller = () => capability ??= readCommandPath()
  .then((status) => status.available === true).catch(() => false);

</storycode:copy>
<a id="f-83bac3568d9f-CommandPathSettings"></a>
### `CommandPathSettings`

<storycode:copy id="f-83bac3568d9f-CommandPathSettings">
export function CommandPathSettings() {
  const [status, setStatus] = useState<CommandPathStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const epoch = useRef(0);
  const parent = `command-path:${useId()}`;
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let live = true; const stamp = ++epoch.current;
    readCommandPath().then(
      (answer) => { if (live && stamp === epoch.current && typeof answer.available === "boolean") setStatus(answer); },
      reason => { if (live && stamp === epoch.current) setError(String(reason)); }, // A headless engine may have no desktop installer.
    );
    return () => { live = false; };
  }, []);
  const apply = async (action: "install" | "remove"): Promise<UiOutcome> => {
    if (pending.current) return { status: "unavailable", reason: "Command installer operation already running" };
    pending.current = true; const stamp = ++epoch.current; setBusy(true); setError(null);
    try {
      const actual = await request<CommandPathStatus>("POST", endpoint, { action }); if (stamp === epoch.current) setStatus(actual);
      return { status: "completed", value: { action, available: actual.available, installed: actual.installed, can_remove: actual.can_remove, conflict: actual.conflict, message: actual.message, scope: "Returned desktop installer status; not a new CLI execution" } };
    } catch (error) { const reason = error instanceof Error ? error.message : String(error); setError(reason); return { status: "unavailable", reason }; }
    finally { pending.current = false; setBusy(false); }
  };
  const revision = useUiRevision(() => [status, busy, error]);
  const invoke = useUiCommands([
    { key: `${parent}:install`, command: { label: "Install hick command", parent, subject: parent, parameters: {}, prerequisites: ["Desktop installer available; no conflicting command"], read: () => ({ revision: revision(), enabled: !busy && !!status?.available && !status.installed && !status.conflict, reason: busy ? "Installer busy" : status?.message ?? error ?? "Desktop installer status unavailable" }), execute: async () => apply("install") } },
    { key: `${parent}:remove`, command: { label: "Remove hick command", parent, subject: parent, parameters: {}, prerequisites: ["The host recognizes a removable installation"], read: () => ({ revision: revision(), enabled: !busy && !!status?.can_remove, reason: busy ? "Installer busy" : status?.message ?? error ?? "Desktop installer status unavailable" }), execute: async () => apply("remove") } },
    { key: `${parent}:refresh`, command: { label: "Refresh hick command status", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !busy }), execute: async () => {
      if (pending.current) return { status: "unavailable", reason: "Command installer operation already running" };
      pending.current = true; const stamp = ++epoch.current; setBusy(true);
      try { const actual = await readCommandPath(); if (stamp === epoch.current) { setStatus(actual); setError(null); } return { status: "completed", value: { available: actual.available, installed: actual.installed, message: actual.message } }; }
      catch (error) { const reason = String(error); setError(reason); return { status: "unavailable", reason }; }
      finally { pending.current = false; setBusy(false); }
    } } },
    { key: `${parent}:read`, command: { label: "Read hick command status bytes", parent, subject: parent, parameters: { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ status, busy, error, scope: "Desktop command installer; unsupported/unavailable host status is explicit" }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside command status" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    } } },
  ]);
  const click = (key: string) => { void invoke(`${parent}:${key}`).catch(reason => setError(String(reason))); };
  if (!status) return null;
  return (
    <section className="settings__appearance" aria-label="Command line" id="command-line">
      <h2 className="settings__section-title">Command line</h2>
      <p className="muted"><code>hick</code> opens Hickory Docs. <code>hick .</code> opens a folder;
        <code> hick run</code>, <code>hick test</code>, and <code>hick up</code> use the CLI.</p>
      <div className="settings-row">
        <div className="settings-row__who">
          <span className="settings-row__label">hick command</span>
          <span className="settings-row__state">{status.installed ? "Installed" : "Not installed"}</span>
        </div>
        <div className="settings-row__actions">
          {status.available && !status.installed && !status.conflict && (
            <button className="btn" disabled={busy} onClick={() => click("install")}>Install hick command…</button>
          )}
          {status.can_remove && (
            <button className="btn btn-quiet" disabled={busy} onClick={() => click("remove")}>Remove hick command</button>
          )}
          <button className="btn btn-quiet" disabled={busy} onClick={() => click("refresh")}>Refresh command status</button>
        </div>
      </div>
      <p className="muted settings__note" role="status">{status.message}</p>
      <p className="mono">{status.command}</p>
      {status.conflict && <p className="error">Another hick command is installed at {status.conflict}.
        Remove that installation or adjust PATH before installing this one.</p>}
      {error && <p className="error" role="alert">{error}</p>}
    </section>
  );
}
</storycode:copy>

### Assemble `apps/web/src/views/CommandPathSettings.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/CommandPathSettings.tsx"><storycode:paste select="#f-83bac3568d9f-CommandPathStatus"/><storycode:paste select="#f-83bac3568d9f-endpoint"/><storycode:paste select="#f-83bac3568d9f-readCommandPath"/><storycode:paste select="#f-83bac3568d9f-capability"/><storycode:paste select="#f-83bac3568d9f-hasCommandInstaller"/><storycode:paste select="#f-83bac3568d9f-CommandPathSettings"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-7f450949f2a8"></a>
## `apps/web/src/views/CommandPathSettings.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [The App Installs Its Command For The User](../../../../../guarantees/release/the-app-installs-its-command-for-the-user/contract.md).

<a id="f-7f450949f2a8-initial"></a>
### `initial`

<storycode:copy id="f-7f450949f2a8-initial">
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, it, expect, vi } from "vitest";
import { installMockHandler } from "../api/client";
import { CommandPathSettings, type CommandPathStatus } from "./CommandPathSettings";
const initial: CommandPathStatus = { available: true, installed: false, can_remove: false,
  command: "/home/me/.local/bin/hick", source: "/app/hick", conflict: null, message: "Install for your user." };
</storycode:copy>
<a id="f-7f450949f2a8-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7f450949f2a8-expression_statement">
afterEach(() => { cleanup(); installMockHandler(null as never); });
</storycode:copy>
<a id="f-7f450949f2a8-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-7f450949f2a8-expression_statement-2">
// Guarantee: docs/guarantees/release/the-app-installs-its-command-for-the-user.md
it("installs and removes through the desktop API, showing the resulting status", async () => {
  const calls = vi.fn();
  installMockHandler(async (method, _path, body) => {
    if (method === "GET") return initial;
    calls(body);
    const installed = (body as { action: string }).action === "install";
    return { ...initial, installed, can_remove: installed, message: "Open a new terminal." };
  });
  render(<CommandPathSettings />);
  fireEvent.click(await screen.findByRole("button", { name: "Install hick command…" }));
  fireEvent.click(await screen.findByRole("button", { name: "Remove hick command" }));
  await waitFor(() => expect(calls.mock.calls).toEqual([[{ action: "install" }], [{ action: "remove" }]]));
  expect(await screen.findByText("Not installed")).toBeTruthy();
});
</storycode:copy>
<a id="f-7f450949f2a8-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-7f450949f2a8-expression_statement-3">
it("shows a conflict without offering to overwrite it", async () => {
  installMockHandler(async () => ({ ...initial, conflict: "/other/hick" }));
  render(<CommandPathSettings />);
  expect(await screen.findByText(/Another hick command/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Install hick command…" })).toBeNull();
});
</storycode:copy>
<a id="f-7f450949f2a8-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-7f450949f2a8-expression_statement-4">
it("shows installation failures and allows retry", async () => {
  installMockHandler(async (method) => {
    if (method === "GET") return initial;
    throw new Error("Cannot write your shell profile; check its permissions.");
  });
  render(<CommandPathSettings />);
  fireEvent.click(await screen.findByRole("button", { name: "Install hick command…" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Cannot write");
  expect((screen.getByRole("button", { name: "Install hick command…" }) as HTMLButtonElement).disabled).toBe(false);
});
</storycode:copy>
<a id="f-7f450949f2a8-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-7f450949f2a8-expression_statement-5">
it("hides the section for a headless engine", async () => {
  const read = vi.fn().mockRejectedValue(new Error("Not found"));
  installMockHandler(read);
  const { container } = render(<CommandPathSettings />);
  await waitFor(() => expect(read).toHaveBeenCalled());
  expect(container.textContent).toBe("");
});

</storycode:copy>
<a id="f-7f450949f2a8-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-7f450949f2a8-expression_statement-6">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares registered install with the real host status and disables conflicting operations", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const calls = vi.fn();
  installMockHandler(async (method, _path, body) => {
    if (method === "GET") return initial;
    calls(body); return { ...initial, installed: true, can_remove: true, message: "Actual installed status" };
  });
  render(<UiActionContext.Provider value={registry}><CommandPathSettings /></UiActionContext.Provider>);
  await screen.findByRole("button", { name: "Install hick command…" });
  const parent = registry.groups().groups.find(g => g.startsWith("command-path:"))!;
  const install = registry.observe(parent).actions.find(a => a.label === "Install hick command")!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain(initial.source);
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: install.handle, arguments: {}, request_id: "install" })).toMatchObject({ outcome: { value: { installed: true, message: "Actual installed status" } } }); });
  expect(calls).toHaveBeenCalledExactlyOnceWith({ action: "install" });
  expect(registry.observe(parent).actions.find(a => a.label === "Install hick command")!.enabled).toBe(false);
  expect(registry.observe(parent).actions.find(a => a.label === "Remove hick command")!.enabled).toBe(true);
  const read = registry.observe(parent).actions.find(a => a.label === "Read hick command status bytes")!;
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  expect(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).toContain(initial.command);
});
</storycode:copy>

### Assemble `apps/web/src/views/CommandPathSettings.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/CommandPathSettings.test.tsx"><storycode:paste select="#f-7f450949f2a8-initial"/><storycode:paste select="#f-7f450949f2a8-expression_statement"/><storycode:paste select="#f-7f450949f2a8-expression_statement-2"/><storycode:paste select="#f-7f450949f2a8-expression_statement-3"/><storycode:paste select="#f-7f450949f2a8-expression_statement-4"/><storycode:paste select="#f-7f450949f2a8-expression_statement-5"/><storycode:paste select="#f-7f450949f2a8-expression_statement-6"/></storycode:file>


</storycode:doc>
