<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/environments/EnvironmentPanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a428d5761b36"></a>
## `apps/web/src/environments/EnvironmentPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [A write exposes its proposed effects](../../../../../guarantees/authoring/a-write-exposes-its-proposed-effects/contract.md), [A project environment offers its own sync](../../../../../guarantees/editor-intelligence/a-project-environment-offers-its-own-sync/contract.md).

<a id="f-a428d5761b36-Finding"></a>
### `Finding`

<storycode:copy id="f-a428d5761b36-Finding">
import { ChangePreview } from "../components/ChangePreview";
import { useId, useRef, useState } from "react";
import { useUiCommands } from "../lib/uiActionBridge";
import type { EnvironmentFinding } from "../api/environments";
import type { Environments } from "./useEnvironments";
import { showTerminalRequest } from "../lib/revealLine";
import "./environments.css";

function Finding({ finding, environments }: { finding: EnvironmentFinding; environments: Environments }) {
  const instance = useId();
  const [actionError, setActionError] = useState("");
  const session = environments.running[finding.project];
  const busy = !!session || environments.pending === finding.project;
  const parent = `environment:${finding.project}:${instance}`;
  const invoke = useUiCommands([
    ...finding.actions.map(action => ({ key: `${parent}/${action.id}`, command: {
      label: action.label, subject: `environment:${finding.project}`, parent, parameters: {}, prerequisites: [action.effects],
      read: () => ({ revision: `${finding.revision}:${busy}:${session ?? ""}`, enabled: !busy, reason: busy ? "Environment action is already running" : undefined,
        value: { manager: finding.manager, state: finding.state, message: finding.message, details: finding.details, argv: action.argv, effects: action.effects, terminal: session ?? null } }),
      execute: async () => {
        const result = await environments.act(finding, action.id);
        return result?.session ? { status: "queued" as const, job: result.session.id, value: { terminal: result.session.id, title: result.session.title, dependency_setup: "not yet verified" } }
          : { status: "unavailable" as const, reason: result?.error ?? "Environment executor returned no session" };
      },
    } })),
    ...finding.manager_choices.map(choice => ({ key: `${parent}/manager:${choice}`, command: {
      label: `Use ${choice}`, subject: `environment:${finding.project}`, parent, parameters: {}, prerequisites: [],
      read: () => ({ revision: finding.revision, enabled: true, value: finding.manager }),
      execute: async () => {
        const result = await environments.choose(finding, choice);
        return result?.ok ? { status: "completed" as const, value: { manager_selected: choice } } : { status: "unavailable" as const, reason: result?.error ?? "Manager selection unavailable" };
      },
    } })),
  ]);
  const perform = (key: string) => { setActionError(""); void invoke(key).catch(error => setActionError(String(error))); };
  return <div className="environment-finding">
    {actionError && <p role="alert">{actionError}</p>}
    <p><strong>{finding.manager}</strong> · {finding.message}</p>
    <p className="environment-directory">{finding.project} · {finding.target}</p>
    {finding.actions.map((action) => <div key={action.id}>
      <ChangePreview heading="Command and effects" effects={action.effects}>
      <code>{action.argv.map((part) => /\s/.test(part) ? JSON.stringify(part) : part).join(" ")}</code>
      </ChangePreview>
      <button type="button" disabled={busy} onClick={() => perform(`${parent}/${action.id}`)}>
        {busy ? "Installing…" : action.label}
      </button>
    </div>)}
    {session && <button type="button" onClick={() => showTerminalRequest(session, `${finding.manager}: dependencies`)}>Show terminal</button>}
    {finding.install_url && <a href={finding.install_url} target="_blank" rel="noreferrer">Install {finding.manager}</a>}
    {finding.manager_choices.map((choice) => <button type="button" key={choice} onClick={() => perform(`${parent}/manager:${choice}`)}>Use {choice}</button>)}
    {finding.details && <details><summary>Details</summary><pre>{finding.details}</pre></details>}
  </div>;
}

</storycode:copy>
<a id="f-a428d5761b36-EnvironmentPanel"></a>
### `EnvironmentPanel`

<storycode:copy id="f-a428d5761b36-EnvironmentPanel">
export function EnvironmentPanel({ environments }: { environments: Environments }) {
  return <section className="environment-panel" aria-label="Project environments">
    <header><strong>Project environments</strong><button type="button" onClick={() => void environments.refresh(true)}>Recheck</button></header>
    {environments.error && <p role="alert">{environments.error}</p>}
    {!environments.findings.length && <p>No supported package-manager projects found.</p>}
    {environments.findings.map((finding) => <EnvironmentGroup key={`${finding.project}:${finding.manager}`} finding={finding} environments={environments} />)}
  </section>;
}

</storycode:copy>
<a id="f-a428d5761b36-EnvironmentGroup"></a>
### `EnvironmentGroup`

<storycode:copy id="f-a428d5761b36-EnvironmentGroup">
function EnvironmentGroup({ finding, environments }: { finding: EnvironmentFinding; environments: Environments }) {
  const instance = useId();
  const details = useRef<HTMLDetailsElement>(null);
  const [open, setOpen] = useState(finding.state !== "ready" && finding.state !== "unknown");
  const [error, setError] = useState("");
  const key = `environment-disclosure:${instance}`;
  const invoke = useUiCommands([{ key, command: {
    label: `${finding.manager} · ${finding.project} · ${finding.state}`, subject: key, parent: "environments", parameters: {}, prerequisites: [],
    read: () => ({ revision: `${finding.revision}:${details.current?.open}`, enabled: !!details.current, value: { open: details.current?.open ?? open, state: finding.state, manager: finding.manager, project: finding.project } }),
    execute: async () => {
      if (!details.current) return { status: "unavailable", reason: "Environment group unavailable" };
      details.current.open = !details.current.open;
      setOpen(details.current.open);
      return { status: "completed", value: { open: details.current.open, children: "Observe groups after this UI transition" } };
    },
  } }]);
  return <details ref={details} open={open} onToggle={event => setOpen(event.currentTarget.open)}>
    <summary onClick={event => { event.preventDefault(); setError(""); void invoke(key).catch(reason => setError(String(reason))); }}>{finding.manager} · {finding.project} · {finding.state}</summary>
    {error && <p role="alert">{error}</p>}
    {open && <Finding finding={finding} environments={environments} />}
  </details>;
}

</storycode:copy>
<a id="f-a428d5761b36-EnvironmentNotice"></a>
### `EnvironmentNotice`

<storycode:copy id="f-a428d5761b36-EnvironmentNotice">
export function EnvironmentNotice({ environments }: { environments: Environments }) {
  const finding = environments.notices[0];
  if (!finding) return null;
  return <aside className="environment-notice" role="status" aria-label="Project environment needs attention">
    <Finding finding={finding} environments={environments} />
    {environments.error && <p role="alert">{environments.error}</p>}
    <button type="button" onClick={() => environments.dismiss(finding)}>Dismiss</button>
  </aside>;
}
</storycode:copy>

### Assemble `apps/web/src/environments/EnvironmentPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/environments/EnvironmentPanel.tsx"><storycode:paste select="#f-a428d5761b36-Finding"/><storycode:paste select="#f-a428d5761b36-EnvironmentPanel"/><storycode:paste select="#f-a428d5761b36-EnvironmentGroup"/><storycode:paste select="#f-a428d5761b36-EnvironmentNotice"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-f41d08c2423e"></a>
## `apps/web/src/environments/EnvironmentPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A write exposes its proposed effects](../../../../../guarantees/authoring/a-write-exposes-its-proposed-effects/contract.md), [A project environment offers its own sync](../../../../../guarantees/editor-intelligence/a-project-environment-offers-its-own-sync/contract.md).

<a id="f-f41d08c2423e-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md

<storycode:copy id="f-f41d08c2423e-expression_statement">
// Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EnvironmentNotice, EnvironmentPanel } from "./EnvironmentPanel";
import type { EnvironmentFinding } from "../api/environments";
import type { Environments } from "./useEnvironments";

afterEach(cleanup);
</storycode:copy>
<a id="f-f41d08c2423e-finding"></a>
### `finding`

<storycode:copy id="f-f41d08c2423e-finding">
const finding: EnvironmentFinding = {
  project: "/repo/analysis", manager: "uv", target: "host", profile: "manager defaults", revision: "one",
  state: "environment-missing", message: "Python dependencies are not installed.", details: "No .venv exists.",
  actions: [{ id: "sync", label: "Sync dependencies", argv: ["uv", "sync", "--locked"], effects: "Keeps uv.lock unchanged." }],
  install_url: null, interpreter: null, manager_choices: [],
};
</storycode:copy>
<a id="f-f41d08c2423e-controller"></a>
### `controller`

<storycode:copy id="f-f41d08c2423e-controller">
const controller = (over: Partial<Environments> = {}): Environments => ({
  findings: [finding], running: {}, notices: [finding], pending: null, error: null,
  refresh: vi.fn(), act: vi.fn(), dismiss: vi.fn(), choose: vi.fn(), ...over,
});
</storycode:copy>
<a id="f-f41d08c2423e-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f41d08c2423e-expression_statement-2">
describe("project environment affordances", () => {
  it("shows command and directory beside the sync action and submits that finding", () => {
    const environments = controller();
    render(<EnvironmentNotice environments={environments} />);
    expect(screen.getByText("uv sync --locked")).toBeTruthy();
    expect(screen.getByText("/repo/analysis · host")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Sync dependencies" }));
    expect(environments.act).toHaveBeenCalledWith(finding, "sync");
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(environments.dismiss).toHaveBeenCalledWith(finding);
  });
  it("disables a repair while its terminal is running", () => {
    render(<EnvironmentNotice environments={controller({ running: { [finding.project]: "session" } })} />);
    expect((screen.getByRole("button", { name: "Installing…" }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByRole("button", { name: "Show terminal" })).toBeTruthy();
  });
  it("clears the notice when the recheck succeeds and keeps readiness in the expandable panel", () => {
    const environments = controller({ findings: [{ ...finding, state: "ready", message: "Python environment is ready.", actions: [] }], notices: [] });
    const { container } = render(<><EnvironmentNotice environments={environments} /><EnvironmentPanel environments={environments} /></>);
    expect(screen.queryByRole("status")).toBeNull();
    expect(container.querySelector("details")?.open).toBe(false);
    fireEvent.click(screen.getByRole("button", { name: "Recheck" }));
    expect(environments.refresh).toHaveBeenCalledWith(true);
  });
});
</storycode:copy>

### Assemble `apps/web/src/environments/EnvironmentPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/environments/EnvironmentPanel.test.tsx"><storycode:paste select="#f-f41d08c2423e-expression_statement"/><storycode:paste select="#f-f41d08c2423e-finding"/><storycode:paste select="#f-f41d08c2423e-controller"/><storycode:paste select="#f-f41d08c2423e-expression_statement-2"/></storycode:file>


</storycode:doc>
