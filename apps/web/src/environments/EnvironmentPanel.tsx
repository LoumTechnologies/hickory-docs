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

export function EnvironmentPanel({ environments }: { environments: Environments }) {
  return <section className="environment-panel" aria-label="Project environments">
    <header><strong>Project environments</strong><button type="button" onClick={() => void environments.refresh(true)}>Recheck</button></header>
    {environments.error && <p role="alert">{environments.error}</p>}
    {!environments.findings.length && <p>No supported package-manager projects found.</p>}
    {environments.findings.map((finding) => <EnvironmentGroup key={`${finding.project}:${finding.manager}`} finding={finding} environments={environments} />)}
  </section>;
}

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

export function EnvironmentNotice({ environments }: { environments: Environments }) {
  const finding = environments.notices[0];
  if (!finding) return null;
  return <aside className="environment-notice" role="status" aria-label="Project environment needs attention">
    <Finding finding={finding} environments={environments} />
    {environments.error && <p role="alert">{environments.error}</p>}
    <button type="button" onClick={() => environments.dismiss(finding)}>Dismiss</button>
  </aside>;
}
