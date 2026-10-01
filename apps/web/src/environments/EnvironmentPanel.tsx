import type { EnvironmentFinding } from "../api/environments";
import type { Environments } from "./useEnvironments";
import { showTerminalRequest } from "../lib/revealLine";
import "./environments.css";

function Finding({ finding, environments }: { finding: EnvironmentFinding; environments: Environments }) {
  const session = environments.running[finding.project];
  const busy = !!session || environments.pending === finding.project;
  return <div className="environment-finding">
    <p><strong>{finding.manager}</strong> · {finding.message}</p>
    <p className="environment-directory">{finding.project} · {finding.target}</p>
    {finding.actions.map((action) => <div key={action.id}>
      <code>{action.argv.map((part) => /\s/.test(part) ? JSON.stringify(part) : part).join(" ")}</code>
      <p>{action.effects}</p>
      <button type="button" disabled={busy} onClick={() => void environments.act(finding, action.id)}>
        {busy ? "Installing…" : action.label}
      </button>
    </div>)}
    {session && <button type="button" onClick={() => showTerminalRequest(session, `${finding.manager}: dependencies`)}>Show terminal</button>}
    {finding.install_url && <a href={finding.install_url} target="_blank" rel="noreferrer">Install {finding.manager}</a>}
    {finding.manager_choices.map((choice) => <button type="button" key={choice} onClick={() => void environments.choose(finding, choice)}>Use {choice}</button>)}
    {finding.details && <details><summary>Details</summary><pre>{finding.details}</pre></details>}
  </div>;
}

export function EnvironmentPanel({ environments }: { environments: Environments }) {
  return <section className="environment-panel" aria-label="Project environments">
    <header><strong>Project environments</strong><button type="button" onClick={() => void environments.refresh(true)}>Recheck</button></header>
    {environments.error && <p role="alert">{environments.error}</p>}
    {!environments.findings.length && <p>No supported package-manager projects found.</p>}
    {environments.findings.map((finding) => <details key={`${finding.project}:${finding.manager}`} open={finding.state !== "ready" && finding.state !== "unknown"}>
      <summary>{finding.manager} · {finding.project} · {finding.state}</summary>
      <Finding finding={finding} environments={environments} />
    </details>)}
  </section>;
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
