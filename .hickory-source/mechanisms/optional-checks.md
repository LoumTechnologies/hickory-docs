<checks:doc xmlns:checks="http://www.hickorydocs.com/1.0">
# Observe optional checks without coupling the editor to a runner

For maintainers replacing local-ci or changing check visualization. The checks
API carries normalized run facts; Git only supplies a revision when a caller
explicitly requests a commit run. Neither opening a pane nor reading status
starts a service or workload. Missing configuration means disabled.

## Contract

Checks have their own tool tab and API, independent of Git history. A configured
provider failure is unavailable, never a passing result. The pane retains the last
snapshot, labels it stale and retries in the background without overlapping polls.
The post-commit adapter supports local, local-ci and off modes, preserves the staged
verification hook, captures HEAD before detaching and disconnects terminal streams.

The focused adapter and UI tests below exercise normalization, failure display and
late reply rejection. The process tests exercise the actual CLI with disposable
providers and repositories; live local-ci compatibility is checked separately.
Visible native desktop acceptance remains a human check. See
[the maintainer setup guide](../../docs/developers/checks.md) for configuration and monitoring.

## Focused verification, 2026-11-06

Executed `just checks-integration-test` with the sibling local-ci debug binary:
two Rust safety witnesses, two pane witnesses, and the real-provider subprocess,
HTTP, browser and Git-hook flow passed. The fixture used separate local-ci and
Hickory state directories. Real commits exercised local, local-ci and off modes;
a newer HEAD before worker release retained the original submitted commit ID.
The browser displayed real results and visibly stale results after provider stop.
The screenshot was inspected for compact layout. TypeScript checking, API adapter
agreement, focused Rust formatting, source/output reconstruction and ownership
audit passed. Broad regressions and packaged native-window acceptance were not run.

## Configuration

Keep the provider choice in user-owned application state keyed by the canonical folder. Repository content must never select an executable merely because the Checks pane opens. Deserialize it into a strict type before use. No discovery of sibling binaries or Git configuration belongs in the observer.

<checks:copy id="optional-checks-configuration">
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncReadExt;

pub fn config_path(root: &Path) -> Result<PathBuf, String> {
    use sha2::{Digest, Sha256};
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let key = hex::encode(Sha256::digest(root.as_os_str().as_encoded_bytes()));
    Ok(hickory_workspace::data_root()
        .map_err(|e| e.to_string())?
        .join("checks")
        .join(format!("{key}.json")))
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub executable: PathBuf,
    pub command: Vec<String>,
    #[serde(default)]
    pub workflow: bool,
}
impl Config {
    pub fn load(root: &Path) -> Result<Option<Self>, String> {
        let path = config_path(root)?;
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(format!(
                    "Cannot read {}: {e}; check permissions",
                    path.display()
                ));
            }
        };
        Self::parse(&path, &bytes).map(Some)
    }
    fn parse(path: &Path, bytes: &[u8]) -> Result<Self, String> {
        let config: Self = serde_json::from_slice(bytes).map_err(|e| {
            format!(
                "Invalid {}: {e}; rerun just checks-configure",
                path.display()
            )
        })?;
        if !config.executable.is_absolute()
            || config.command.is_empty()
            || config.command[0].is_empty()
        {
            return Err("Checks need an absolute executable and nonempty command; rerun just checks-configure".into());
        }
        Ok(config)
    }
}
</checks:copy>

## Provider transport

Only this transport invokes local-ci. Bound observation time and response size, kill a timed out observation client, and never start the provider service while observing. Explicit submissions may spend longer exporting a commit; they wait for durable acceptance without a client deadline, while the Git hook remains detached. This keeps failures out of unrelated editor requests.

<checks:copy id="optional-checks-provider-transport">
pub async fn invoke(config: &Config, root: &Path, args: &[String]) -> Result<Value, String> {
    let mut child = tokio::process::Command::new(&config.executable)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            format!(
                "Cannot launch {}: {e}; rebuild local-ci or rerun just checks-configure",
                config.executable.display()
            )
        })?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let work = async {
        let read = async |stream: tokio::process::ChildStdout| {
            let mut bytes = Vec::new();
            stream.take(2_097_153).read_to_end(&mut bytes).await?;
            Ok::<_, std::io::Error>(bytes)
        };
        let err = async {
            let mut bytes = Vec::new();
            stderr.take(65_537).read_to_end(&mut bytes).await?;
            Ok::<_, std::io::Error>(bytes)
        };
        let (out, err) = tokio::try_join!(read(stdout), err).map_err(|e| e.to_string())?;
        if out.len() > 2_097_152 || err.len() > 65_536 {
            return Err(
                "local-ci response exceeded the display limit; use local-ci runs directly".into(),
            );
        }
        let status = child.wait().await.map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!(
                "local-ci is unavailable: {}; run local-ci service start and retry",
                String::from_utf8_lossy(&err).trim()
            ));
        }
        if args.first().is_some_and(|a| a == "run") {
            let id = String::from_utf8(out).map_err(|e| e.to_string())?;
            if id.trim().is_empty() || id.trim().contains(char::is_whitespace) {
                return Err(
                    "local-ci returned no single run ID; inspect local-ci runs before retrying"
                        .into(),
                );
            }
            Ok(serde_json::json!({ "id": id.trim() }))
        } else {
            serde_json::from_slice(&out).map_err(|e| {
                format!("Unsupported local-ci response: {e}; check the installed local-ci version")
            })
        }
    };
    if args.first().is_some_and(|a| a == "run") {
        work.await
    } else {
        tokio::time::timeout(std::time::Duration::from_secs(15), work)
            .await
            .map_err(|_| {
                "local-ci did not respond within 15 seconds; check its service and retry"
                    .to_string()
            })?
    }
}
</checks:copy>

## Normalized snapshot

Translate only the fields the UI needs. Filter by canonical project directory, including parent workflow identity; do not leak another project’s commands. Unknown provider states stay unknown. Source revision is an opaque annotation and the core never invokes Git.

<checks:copy id="optional-checks-normalized-snapshot">
#[derive(Debug, Serialize)]
pub struct Run {
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub status: String,
    pub revision: Option<String>,
    pub created_ms: u64,
    pub code: Option<i64>,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub availability: &'static str,
    pub message: Option<String>,
    pub runs: Vec<Run>,
}
pub fn normalize(value: Value, root: &Path) -> Result<Vec<Run>, String> {
    let rows = value["runs"]
        .as_array()
        .ok_or("local-ci returned no runs array; check its version")?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut runs = Vec::new();
    for row in rows {
        let request = &row["request"];
        let project = request["project"]
            .as_str()
            .or_else(|| request["cwd"].as_str());
        if !project.is_some_and(|p| Path::new(p).canonicalize().is_ok_and(|p| p == root)) {
            continue;
        }
        let id = row["id"]
            .as_str()
            .ok_or("local-ci run has no ID; check its version")?;
        let raw = row["status"].as_str().unwrap_or("unknown");
        let status = match raw {
            "queued"
            | "running"
            | "finishing"
            | "observing"
            | "passed"
            | "failed"
            | "blocked"
            | "cancelled"
            | "timed_out"
            | "infrastructure_failed"
            | "incomplete" => raw,
            _ => "unknown",
        };
        let args = request["args"].as_array();
        let name = request["check"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| {
                let args: Vec<_> = args
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect();
                let start = args.iter().position(|v| *v == "--").map_or(0, |i| i + 1);
                args[start..].join(" ")
            });
        runs.push(Run {
            id: id.into(),
            parent: request["parent"].as_str().map(str::to_owned),
            name,
            status: status.into(),
            revision: request["source"]["revision"].as_str().map(str::to_owned),
            created_ms: row["created_ms"].as_u64().unwrap_or(0),
            code: row["code"].as_i64(),
        });
    }
    runs.sort_by_key(|r| std::cmp::Reverse(r.created_ms));
    Ok(runs)
}
pub async fn snapshot(root: &Path) -> Snapshot {
    let result = async {
        let Some(config) = Config::load(root)? else {
            return Ok(None);
        };
        let value = invoke(&config, root, &["runs".into()]).await?;
        normalize(value, root).map(Some)
    }
    .await;
    match result {
        Ok(Some(runs)) => Snapshot { availability: "ready", message: None, runs },
        Ok(None) => Snapshot { availability: "disabled", message: Some("Configure local-ci with just checks-configure EXECUTABLE [COMMAND…] in the opened folder.".into()), runs: vec![] },
        Err(message) => Snapshot { availability: "unavailable", message: Some(message), runs: vec![] },
    }
}
</checks:copy>

## Adapter tests

These focused assertions protect the contract above: a malformed provider cannot manufacture a pass, and another workspace never appears in this one.

<checks:copy id="optional-checks-adapter-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    // Contract: .hickory-source/mechanisms/optional-checks.md
    #[test]
    fn isolates_projects_and_preserves_unknown_and_revision() {
        let root = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let rows = json!({"runs":[
            {"id":"one","status":"new-provider-state","request":{"project":root.path(),"source":{"revision":"abc"},"args":["run","--","just","check"]}},
            {"id":"two","status":"passed","request":{"project":other.path()}}
        ]});
        let runs = normalize(rows, root.path()).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "unknown");
        assert_eq!(runs[0].revision.as_deref(), Some("abc"));
        assert_eq!(runs[0].name, "just check");
        assert!(normalize(json!({}), root.path()).is_err());
    }
    #[tokio::test]
    async fn absent_is_disabled_and_repository_cannot_configure_execution() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(snapshot(root.path()).await.availability, "disabled");
        assert!(Config::parse(root.path(), b"{}").is_err());
        // A file supplied by a checkout cannot authorize process execution.
        std::fs::write(root.path().join(".hickory-checks.local.json"), "{}").unwrap();
        assert_eq!(snapshot(root.path()).await.availability, "disabled");
    }
}
</checks:copy>

<checks:file path="crates/hickory-cli/src/checks.rs"><checks:paste select="#optional-checks-configuration"/><checks:paste select="#optional-checks-provider-transport"/><checks:paste select="#optional-checks-normalized-snapshot"/><checks:paste select="#optional-checks-adapter-tests"/></checks:file>

## Command interface

The explicit CLI configures a local executable or submits a run. Commit source selection belongs here, at the Git boundary, rather than in the checks observer. A revision is required to be a full object ID so delayed hooks cannot accidentally check a newer HEAD.

<checks:copy id="optional-checks-command-interface">
use hickory_cli::checks::{Config, config_path, invoke, snapshot};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("checks: {error}");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let root = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let path = config_path(&root)?;
    match args.next().as_deref() {
        Some("configure") => {
            let executable = args
                .next()
                .ok_or("Supply the local-ci executable path; use --off to disable")?;
            if executable == "--off" {
                match std::fs::remove_file(&path) {
                    Ok(()) => (),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                    Err(e) => {
                        return Err(format!(
                            "Cannot remove {}: {e}; check its permissions",
                            path.display()
                        ));
                    }
                }
                return Ok(());
            }
            let executable = std::path::PathBuf::from(executable)
                .canonicalize()
                .map_err(|e| format!("Cannot find local-ci: {e}; build ../local-ci first"))?;
            let mut command: Vec<_> = args.collect();
            if command.is_empty() {
                command = vec!["just".into(), "check".into()];
            }
            let config = Config {
                executable,
                command,
                workflow: false,
            };
            if !config.executable.is_file() || config.command[0].is_empty() {
                return Err("Choose a local-ci executable file and a nonempty command; rerun just checks-configure".into());
            }
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
                format!(
                    "Cannot create checks configuration directory {}: {e}; check permissions",
                    path.parent().unwrap().display()
                )
            })?;
            std::fs::write(&path, serde_json::to_vec_pretty(&config).unwrap())
                .map_err(|e| format!("Cannot write {}: {e}; check permissions", path.display()))?;
            println!(
                "Configured {}; use just checks-hook local-ci to enable commit runs",
                path.display()
            );
        }
        Some("status") => {
            let result = snapshot(&root).await;
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            if result.availability == "unavailable" {
                return Err("Provider unavailable; see the diagnostic above".into());
            }
        }
        Some("run") => {
            let config = Config::load(&root)?
                .ok_or("Run just checks-configure EXECUTABLE [COMMAND…] first")?;
            let mut command = vec!["run".into(), "--detach".into()];
            if config.workflow {
                command.push("--workflow".into());
            }
            if let Some(revision) = args.next() {
                if ![40, 64].contains(&revision.len())
                    || !revision.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(
                        "Commit runs need a full Git object ID; use git rev-parse HEAD".into(),
                    );
                }
                command.extend([
                    "--source".into(),
                    "git".into(),
                    "--repo".into(),
                    root.to_string_lossy().into_owned(),
                    "--revision".into(),
                    revision,
                ]);
            } else {
                command.extend(["--workspace".into(), root.to_string_lossy().into_owned()]);
            }
            command.push("--".into());
            command.extend(config.command);
            // Submission can prepare a snapshot; it does not wait for checks.
            let value = invoke(
                &Config {
                    command: vec![],
                    ..config
                },
                &root,
                &command,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&value).unwrap());
        }
        _ => return Err("Use configure EXECUTABLE [COMMAND…], status, or run [COMMIT]".into()),
    }
    Ok(())
}
</checks:copy>

<checks:file path="crates/hickory-cli/src/bin/hickory-checks.rs"><checks:paste select="#optional-checks-command-interface"/></checks:file>

## Http boundary

An independent read-only endpoint means provider latency cannot block Git operations, file loading or source edits.

<checks:copy id="optional-checks-http-boundary">
use super::LocalState;
use axum::{Json, extract::State};

pub async fn status(State(state): State<LocalState>) -> Json<crate::checks::Snapshot> {
    Json(crate::checks::snapshot(state.index.root()).await)
}
</checks:copy>

<checks:file path="crates/hickory-cli/src/serve/checks.rs"><checks:paste select="#optional-checks-http-boundary"/></checks:file>

## Browser contract

The browser sees normalized facts and availability, with no local-ci endpoint token or execution controls.

<checks:copy id="optional-checks-browser-contract">
export interface CheckRun {
  id: string;
  parent: string | null;
  name: string;
  status: string;
  revision: string | null;
  created_ms: number;
  code: number | null;
}
export interface ChecksSnapshot {
  availability: "ready" | "disabled" | "unavailable";
  message: string | null;
  runs: CheckRun[];
}
</checks:copy>

<checks:file path="apps/web/src/api/checks.ts"><checks:paste select="#optional-checks-browser-contract"/></checks:file>

## Polling view

Poll only while mounted, serially and independently from editor state. Keep the last successful snapshot during outages, explicitly label it stale, and retire replies when the pane unmounts. The stable table provides workflow parent IDs and revision labels without changing the Git pane.

<checks:copy id="optional-checks-polling-view">
import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { ChecksSnapshot } from "../api/checks";

export function ChecksPane() {
  const [snapshot, setSnapshot] = useState<ChecksSnapshot | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [stale, setStale] = useState(false);
  useEffect(() => {
    let live = true;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const next = await api.checks();
        if (!live) return;
        if (next.availability === "unavailable") {
          setMessage(next.message); setStale(true);
        } else {
          setSnapshot(next); setMessage(next.message); setStale(false);
        }
      } catch (error) {
        if (!live) return;
        setStale(true);
        setMessage(`${error instanceof Error ? error.message : String(error)}. Check the configured checks provider and retry.`);
      }
      if (live) timer = setTimeout(poll, 3000);
    };
    void poll();
    return () => { live = false; clearTimeout(timer); };
  }, []);
  return <section aria-label="Checks" style={{ padding: "0.75rem", overflow: "auto", height: "100%", fontSize: "0.84rem" }}>
    <h2 style={{ fontSize: "1rem", margin: "0 0 0.5rem" }}>Checks</h2>
    <p className="muted">Recent runs for this folder · updates every 3 seconds</p>
    {message && <p role={stale ? "alert" : "status"}>{message}</p>}
    {stale && <p role="status">Status unavailable. Last results below may be stale; retrying automatically.</p>}
    {!snapshot && !stale && <p role="status">Reading checks…</p>}
    {snapshot?.availability === "ready" && snapshot.runs.length === 0 && <p>No recent checks for this folder.</p>}
    {!!snapshot?.runs.length && <table style={{ width: "100%", textAlign: "left" }}>
      <thead><tr><th>Check / run</th><th>Status</th><th>Source</th><th>Started</th></tr></thead>
      <tbody>{snapshot.runs.map(run => <tr key={run.id}>
        <td><div>{run.name || run.id}</div><small className="muted">{run.id}{run.parent ? ` · child of ${run.parent}` : ""}</small></td>
        <td style={{ whiteSpace: "nowrap" }}>{run.status.replaceAll("_", " ")}{run.code !== null ? ` (${run.code})` : ""}</td>
        <td>{run.revision ? <code title={run.revision}>{run.revision.slice(0, 12)}</code> : "Workspace (no immutable revision)"}</td>
        <td>{run.created_ms ? new Date(run.created_ms).toLocaleString() : "Unknown"}</td>
      </tr>)}</tbody>
    </table>}
  </section>;
}
</checks:copy>

<checks:file path="apps/web/src/views/ChecksPane.tsx"><checks:paste select="#optional-checks-polling-view"/></checks:file>

## View tests

The UI witness checks that a provider failure does not erase earlier facts or claim a fresh pass. It also exercises disabled setup guidance and a late response after unmount.

<checks:copy id="optional-checks-view-tests">
// Contract: .hickory-source/mechanisms/optional-checks.md
import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { ChecksPane } from "./ChecksPane";
vi.mock("../api/client", () => ({ api: { checks: vi.fn() } }));
afterEach(() => { cleanup(); vi.useRealTimers(); vi.resetAllMocks(); });
it("keeps results visibly stale through provider failure", async () => {
  vi.useFakeTimers();
  vi.mocked(api.checks).mockResolvedValueOnce({ availability: "ready", message: null, runs: [
    { id: "r1", parent: null, name: "just check", status: "passed", revision: "abcdef", created_ms: 1, code: 0 },
  ] }).mockResolvedValue({ availability: "unavailable", message: "Start local-ci service", runs: [] });
  render(<ChecksPane />);
  await act(async () => {});
  expect(screen.getByText("passed (0)")).toBeTruthy();
  await act(async () => { await vi.advanceTimersByTimeAsync(3000); });
  expect(screen.getByText(/may be stale/)).toBeTruthy();
  expect(screen.getByText("passed (0)")).toBeTruthy();
  expect(screen.getByText("Start local-ci service")).toBeTruthy();
});
it("shows disabled setup and retires late requests", async () => {
  vi.mocked(api.checks).mockResolvedValue({ availability: "disabled", message: "Use just checks-configure", runs: [] });
  const view = render(<ChecksPane />);
  expect(await screen.findByText("Use just checks-configure")).toBeTruthy();
  view.unmount();
  let finish!: (value: Awaited<ReturnType<typeof api.checks>>) => void;
  vi.mocked(api.checks).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const late = render(<ChecksPane />);
  late.unmount();
  await act(async () => { finish({ availability: "ready", message: null, runs: [] }); });
  expect(screen.queryByText("No recent checks for this folder.")).toBeNull();
});
</checks:copy>

<checks:file path="apps/web/src/views/ChecksPane.test.tsx"><checks:paste select="#optional-checks-view-tests"/></checks:file>

## Configure and observe

Keep setup procedural. Provider choice and Git trigger choice are separate, reversible operations. No sibling checkout is needed by a shipped app; the configured binary may be installed anywhere.

<checks:entry path="docs/developers/checks.md">
# Visualize local checks

Open **Checks** in the bottom status bar. It shows up to the provider’s latest
200 runs, filtered to the opened folder, including workflow children when the
provider records them. Status refreshes every three seconds while the tab is open.
An unavailable provider leaves earlier results visibly stale. Opening the tab
never starts a service or a workload.

From this repository root:

```sh
# Build local-ci separately, or use an already installed binary.
just -f ../local-ci/justfile build
just checks-configure ../local-ci/target/debug/local-ci just check
just checks-submit
just checks-provider-status
```

`checks-configure` accepts an executable path followed by command argv. Omit argv
for `just check`. Arguments containing spaces must be quoted individually. The
configuration file printed by that command stores the canonical executable, command argv,
and a `workflow` boolean (default false). It lives under Hickory’s per-user app data
directory (`HICKORY_STATE_DIR` when set), in `checks/`, keyed by the canonical
opened folder. Repository files cannot enable this integration or choose an executable. Set `workflow` true for a controller that
registers child checks with local-ci. Ordinary subprocesses appear as one aggregate
run. Configure each opened folder separately; no Git repository is required for
workspace runs. A workspace run uses live files and has no immutable revision.
The repository recipes build the Hickory adapter as needed; Rust/Cargo is required.

Select post-commit behavior independently:

```sh
just checks-hook local-ci  # submit the exact commit to the configured provider
just checks-hook local    # existing local runner (default)
just checks-hook off      # no automatic checks
```

This sets clone-local `hickory.postCommitChecks`; it does not replace hooks or
change the staged source verification gate. Install this checkout’s hooks with
`just literate-bootstrap` on a fresh clone. The hook captures the commit ID before
detaching; subsequent commits cannot change which revision that submission uses.
Git returns without waiting for checks. Explicit submissions can spend time exporting
large commits and have no client deadline; observation requests are limited to 15
seconds. Submission errors go to the Git common
directory’s `checks/hook.log`. A failed submission never claims the commit passed.

For a manual immutable run, pass a full commit ID to `just checks-submit COMMIT`.
local-ci’s optional Git adapter exports committed bytes without Git metadata;
choose a workload that can run in that exported tree. `just check` is the default
compile check. `just ci` requests the broader repository suite and may require
additional tools and installed dependencies. `just checks` needs Git metadata and
is intended for the existing local runner rather than an exported tree.

To stop using the provider, select `just checks-hook local` or `off`, then run
`just checks-configure --off`. This removes only that folder’s integration config;
it does not cancel local-ci runs or stop its service. Configure a rebuilt binary
again if its path changes.

Monitor results yourself with the Checks pane, `just checks-provider-status`, or
`local-ci runs` / `local-ci status RUN_ID`. For the existing runner use
`just checks-status`. Read `checks/hook.log` if a commit has no corresponding run.
Broad regression execution and monitoring remain user-owned.

Focused verification: `just checks-integration-test`. To exercise the real
provider and detached Git hook in disposable workspaces, run
`just checks-integration-test /absolute/path/to/local-ci`. This does not run the
repository’s broad regression workload.
</checks:entry>

## Process witness

Use ordinary subprocesses and disposable repositories as a framework-independent system witness. The real provider runs a tiny command, not the regression suite. A private service root avoids touching the developer’s existing runs. The hook spy records dispatch arguments only; it does not impersonate provider status.

<checks:copy id="optional-checks-process-witness">
"""Focused black-box checks integration; pass adapter and real local-ci paths."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import re
import signal
import selectors
import urllib.request


def main():
    adapter, provider, fixture = map(lambda value: str(Path(value).resolve()), sys.argv[1:4])
    hook = Path(__file__).resolve().parents[1] / ".githooks/post-commit"
    with tempfile.TemporaryDirectory(prefix="hickory-checks-") as directory:
        root = Path(directory)
        work = root / "work"
        work.mkdir()
        env = dict(os.environ, LOCAL_CI_HOME=str(root / "service"), HICKORY_STATE_DIR=str(root / "hickory-state"))
        def run(argv, cwd=work, ok=True):
            result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, text=True, timeout=30)
            assert not ok or result.returncode == 0, (argv, result.stdout, result.stderr)
            return result
        origin = None
        def observe():
            if origin:
                with urllib.request.urlopen(origin + "/api/checks", timeout=20) as response:
                    return json.load(response)
            return json.loads(run([adapter, "status"]).stdout)
        # Contract: .hickory-source/mechanisms/optional-checks.md
        assert observe()["availability"] == "disabled"
        (work / "note.md").write_text("# Readable scratch note\n")
        server = subprocess.Popen([fixture, str(work)], env=env, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        selector = selectors.DefaultSelector()
        selector.register(server.stdout, selectors.EVENT_READ)
        assert selector.select(20), "Native API fixture did not start"
        origin = server.stdout.readline().strip()
        selector.close()
        assert origin.startswith("http://127.0.0.1:")
        try:
            assert observe()["availability"] == "disabled"
            run([adapter, "configure", provider, sys.executable, "-c", "print('argv with spaces')"])
            assert observe()["availability"] == "unavailable"  # reads never start service
            submitted = json.loads(run([adapter, "run"]).stdout)["id"]
            deadline = time.monotonic() + 15
            while True:
                status = observe()
                row = next((r for r in status["runs"] if r["id"] == submitted), None)
                if row and row["status"] == "passed":
                    break
                assert time.monotonic() < deadline, status
                time.sleep(0.1)
            assert row["revision"] is None
            run(["git", "init", "-b", "master"])
            run(["git", "config", "user.name", "Checks test"])
            run(["git", "config", "user.email", "checks@example.invalid"])
            # No global hooks or signatures enter the disposable fixture.
            run(["git", "config", "core.hooksPath", str(root / "empty-hooks")])
            run(["git", "config", "commit.gpgsign", "false"])
            (work / "file").write_text("committed input\n")
            run(["git", "add", "file"])
            run(["git", "commit", "-m", "fixture"])
            revision = run(["git", "rev-parse", "HEAD"]).stdout.strip()
            immutable = json.loads(run([adapter, "run", revision]).stdout)["id"]
            deadline = time.monotonic() + 15
            while True:
                row = next((r for r in observe()["runs"] if r["id"] == immutable), None)
                if row and row["status"] == "passed":
                    break
                assert time.monotonic() < deadline, row
                time.sleep(0.1)
            assert row["revision"] == revision
            # Bring up the ordinary web host against the native API for one focused flow.
            repo = Path(__file__).resolve().parents[1]
            with (root / "vite.log").open("w+") as log:
                web_env = dict(env, HICKORY_API_ORIGIN=origin)
                web = subprocess.Popen(["npm", "--prefix", "apps/web", "run", "dev"], cwd=repo, env=web_env, stdout=log, stderr=log, start_new_session=True)
                try:
                    deadline = time.monotonic() + 20
                    while True:
                        log.seek(0)
                        match = re.search(r"Local:.*?(http://127\.0\.0\.1:\d+/)", log.read())
                        if match:
                            break
                        assert web.poll() is None and time.monotonic() < deadline, "Web host did not start"
                        time.sleep(0.1)
                    result = subprocess.run(["node", "scripts/test-checks-ui.mjs", match[1].rstrip("/"), provider], cwd=repo, env=env, capture_output=True, text=True, timeout=45)
                    assert result.returncode == 0, (result.stdout, result.stderr)
                    print(result.stdout.strip())
                finally:
                    os.killpg(web.pid, signal.SIGTERM)
                    web.wait(timeout=10)
            assert run([adapter, "run", "HEAD"], ok=False).returncode != 0
            # The Git adapter dispatches an immutable ID and exits before the spy.
            bin_dir = root / "bin"
            bin_dir.mkdir()
            marker = root / "dispatched.json"
            gate = root / "release-worker"
            spy = bin_dir / "just"
            spy.write_text(f"#!{sys.executable}\nimport json,sys,time\nfrom pathlib import Path\nwhile not Path({str(gate)!r}).exists(): time.sleep(0.01)\ntarget = Path({str(marker)!r})\ntemporary = target.with_suffix('.tmp')\ntemporary.write_text(json.dumps(sys.argv[1:]))\ntemporary.replace(target)\n")
            spy.chmod(0o755)
            env["PATH"] = str(bin_dir) + os.pathsep + env["PATH"]
            hooks = root / "hooks"
            hooks.mkdir()
            installed = hooks / "post-commit"
            installed.write_bytes(hook.read_bytes())
            installed.chmod(0o755)
            run(["git", "config", "core.hooksPath", str(hooks)])
            for mode in ["local-ci", "local"]:
                marker.unlink(missing_ok=True)
                gate.unlink(missing_ok=True)
                run(["git", "config", "hickory.postCommitChecks", mode])
                run(["git", "commit", "--allow-empty", "-m", "hook " + mode])
                assert not marker.exists(), "Git returned only after its detached worker finished"
                committed = run(["git", "rev-parse", "HEAD"]).stdout.strip()
                expected = ["checks-submit", committed] if mode == "local-ci" else ["checks"]
                if mode == "local-ci":
                    # Advancing HEAD before the detached worker wakes cannot retarget it.
                    run(["git", "config", "hickory.postCommitChecks", "off"])
                    run(["git", "commit", "--allow-empty", "-m", "newer commit"])
                gate.touch()
                deadline = time.monotonic() + 5
                while not marker.exists():
                    assert time.monotonic() < deadline
                    time.sleep(0.05)
                assert json.loads(marker.read_text()) == expected
            marker.unlink()
            run(["git", "config", "hickory.postCommitChecks", "off"])
            run(["git", "commit", "--allow-empty", "-m", "disabled hook"])
            time.sleep(1.2)
            assert not marker.exists()
            run([adapter, "configure", "--off"])
            assert observe()["availability"] == "disabled"
        finally:
            run([provider, "service", "stop"], ok=False)
            server.terminate()
            server.wait(timeout=10)
    print("Optional checks: real provider, immutable source and detached hook witnesses passed")


if __name__ == "__main__":
    main()
</checks:copy>
<checks:file path="scripts/test-checks-integration.py"><checks:paste select="#optional-checks-process-witness"/></checks:file>

## socket-fixture

This small launcher serves the real native API over a dynamically assigned loopback socket. The Python witness can drive the public HTTP contract without an in-process framework client.

<checks:copy id="optional-checks-socket-fixture">
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("scratch folder"));
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: Some(root.join("keys.json")),
        ui_settings_path: Some(root.join("ui.json")),
    })
    .await?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    println!("http://{}", listener.local_addr()?);
    axum::serve(listener, prepared.router).await?;
    Ok(())
}
</checks:copy>
<checks:file path="crates/hickory-cli/examples/checks_fixture.rs"><checks:paste select="#optional-checks-socket-fixture"/></checks:file>

## live-browser-witness

Drive the actual Checks button and real API-backed pane. The browser also verifies that an unavailable provider preserves stale results while the editor remains independently accessible. This is a browser-host witness, not native window acceptance.

<checks:copy id="optional-checks-live-browser-witness">
// Contract: .hickory-source/mechanisms/optional-checks.md
import { chromium } from "../apps/web/node_modules/playwright-core/index.mjs";
import { execFileSync } from "node:child_process";
const [url, provider] = process.argv.slice(2);
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage();
  const response = await page.request.get(`${url}/api/projects/local/docs`);
  const docs = await response.json();
  const doc = docs.find(doc => doc.path === "note.md");
  if (!doc) throw new Error("Missing scratch note");
  await page.goto(`${url}/#/docs/${doc.id}`);
  await page.getByRole("button", { name: "Checks", exact: true }).click();
  const pane = page.getByRole("region", { name: "Checks", exact: true });
  await pane.getByText("passed (0)", { exact: true }).first().waitFor();
  execFileSync(provider, ["service", "stop"], { env: process.env });
  await pane.getByText(/Last results below may be stale/).waitFor({ timeout: 20000 });
  if (await pane.getByText("passed (0)", { exact: true }).count() === 0) throw new Error("Lost stale results");
  await page.screenshot({ path: process.env.HICKORY_CHECKS_SCREENSHOT ?? "/tmp/hickory-checks.png", fullPage: true });
  console.log("Checks button, live results and stale provider recovery passed");
} finally {
  await browser.close();
}
</checks:copy>
<checks:file path="scripts/test-checks-ui.mjs"><checks:paste select="#optional-checks-live-browser-witness"/></checks:file>

</checks:doc>
