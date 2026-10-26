<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# ACP Agents Work In The Agent Pane And Leave A Hickory Record

For: Authors using an agent to change documents while retaining review control.

Given an installed ACP v1 agent, when it is selected in the Agent pane, then
Hickory launches its executable directly in the workspace, negotiates its
capabilities, and connects a private loopback MCP server containing Hickory's
existing document tools. The downloaded desktop executable also supplies the
stdio proxy; a separate `hick` installation is not required.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/acp-agents-are-first-class.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-c2b0eace1865"></a>
## `crates/hickory-cli/src/serve/acp/config.rs`

Related contracts: [ACP Agents Work In The Agent Pane And Leave A Hickory Record](contract.md).

<a id="f-c2b0eace1865-AgentCommand"></a>
### `AgentCommand`

Machine-local agent commands. No credentials are returned by this catalogue.

<storycode:copy id="f-c2b0eace1865-AgentCommand">
//! Machine-local agent commands. No credentials are returned by this catalogue.
use super::super::LocalState;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentCommand {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub workspace_filesystem: bool,
    /// Explicit fresh-context arguments, independent of ordinary resume/config args.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_prediction_args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_prediction_auth_method: Option<String>,
}

</storycode:copy>
<a id="f-c2b0eace1865-directory"></a>
### `directory`

<storycode:copy id="f-c2b0eace1865-directory">
pub fn directory(state: &LocalState) -> PathBuf {
    state
        .ui
        .path
        .as_ref()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(PathBuf::from)
                .unwrap_or_default()
                .join(".hickory")
        })
        .join("agents")
}

</storycode:copy>
<a id="f-c2b0eace1865-commands"></a>
### `commands`

<storycode:copy id="f-c2b0eace1865-commands">
pub fn commands(state: &LocalState) -> Result<Vec<AgentCommand>> {
    let path = directory(state).join("agents.json");
    let mut commands = vec![
        AgentCommand {
            id: "codex".into(),
            name: "Codex".into(),
            command: "codex-acp".into(),
            args: vec![],
            workspace_filesystem: false,
            private_prediction_args: None,
            private_prediction_auth_method: None,
        },
        AgentCommand {
            id: "claude".into(),
            name: "Claude Agent".into(),
            command: "claude-agent-acp".into(),
            args: vec![],
            workspace_filesystem: false,
            private_prediction_args: None,
            private_prediction_auth_method: None,
        },
    ];
    match std::fs::read_to_string(&path) {
        Ok(raw) => {
            let custom: Vec<AgentCommand> = serde_json::from_str(&raw)
                .with_context(|| format!("reading {}", path.display()))?;
            validate(&custom)?;
            for agent in custom {
                commands.retain(|c| c.id != agent.id);
                commands.push(agent);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    }
    Ok(commands)
}

</storycode:copy>
<a id="f-c2b0eace1865-validate"></a>
### `validate`

<storycode:copy id="f-c2b0eace1865-validate">
pub fn validate(commands: &[AgentCommand]) -> Result<()> {
    let mut ids = std::collections::HashSet::new();
    for agent in commands {
        if agent.id.is_empty()
            || matches!(agent.id.as_str(), "builtin" | "python")
            || !agent
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !ids.insert(&agent.id)
        {
            bail!(
                "each agent needs a unique id using letters, numbers or hyphens; builtin and python are reserved"
            );
        }
        anyhow::ensure!(
            agent
                .private_prediction_auth_method
                .as_ref()
                .is_none_or(|method| !method.is_empty() && method.len() <= 128),
            "Private ACP authentication method must be bounded"
        );
        if let Some(args) = &agent.private_prediction_args {
            anyhow::ensure!(
                args.len() <= 128
                    && args
                        .iter()
                        .all(|arg| arg.len() <= 8192 && !arg.contains('\0'))
                    && args.iter().map(String::len).sum::<usize>() <= 65536,
                "Private ACP arguments exceed the bounded command profile"
            );
        }
        if agent.name.trim().is_empty() || agent.command.trim().is_empty() {
            bail!("{} needs a name and an executable command", agent.id);
        }
    }
    Ok(())
}

</storycode:copy>
<a id="f-c2b0eace1865-executable"></a>
### `executable`

<storycode:copy id="f-c2b0eace1865-executable">
pub fn executable(command: &str, directory: &Path) -> Option<PathBuf> {
    let direct = PathBuf::from(command);
    if direct.is_absolute() {
        return is_executable(&direct).then_some(direct);
    }
    let mut paths: Vec<PathBuf> =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
    paths.insert(0, directory.join("node_modules/.bin"));
    if let Some(home) = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
    {
        paths.push(home.join(".local/bin"));
        paths.push(home.join(".cargo/bin"));
        // Apps launched from a dock do not inherit a shell's nvm PATH.
        if let Ok(versions) = std::fs::read_dir(home.join(".nvm/versions/node")) {
            let mut versions: Vec<_> = versions.flatten().map(|v| v.path().join("bin")).collect();
            versions.sort();
            versions.reverse();
            paths.extend(versions);
        }
    }
    paths.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ]);
    paths
        .into_iter()
        .flat_map(|p| {
            let bare = p.join(command);
            if cfg!(windows) {
                vec![
                    bare,
                    p.join(format!("{command}.exe")),
                    p.join(format!("{command}.cmd")),
                ]
            } else {
                vec![bare]
            }
        })
        .find(|p| is_executable(p))
}

</storycode:copy>
<a id="f-c2b0eace1865-is_executable"></a>
### `is_executable`

<storycode:copy id="f-c2b0eace1865-is_executable">
fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

</storycode:copy>
<a id="f-c2b0eace1865-process"></a>
### `process`

<storycode:copy id="f-c2b0eace1865-process">
pub fn process(agent: &AgentCommand, state: &LocalState) -> Result<tokio::process::Command> {
    super::ensure_ordinary(state)?;
    let dir = directory(state);
    let executable = executable(&agent.command, &dir).with_context(|| format!("{} is not installed. Install its adapter here, or set its executable in Settings → Agents.", agent.name))?;
    let managed = matches!(agent.command.as_str(), "codex-acp" | "claude-agent-acp")
        .then(|| {
            dir.join("node_modules/@agentclientprotocol")
                .join(&agent.command)
                .join("dist/index.js")
        })
        .filter(|path| path.is_file());
    let mut cmd = if let Some(script) = managed {
        let node = self::executable("node", &dir)
            .context("This adapter needs Node.js. Install Node.js, then reconnect.")?;
        let mut cmd = tokio::process::Command::new(node);
        cmd.arg(script);
        cmd
    } else {
        anyhow::ensure!(
            executable.extension().is_none_or(|ext| ext != "cmd"),
            "Use the Node executable and the adapter script as its first argument instead of a .cmd wrapper."
        );
        tokio::process::Command::new(executable)
    };
    cmd.args(&agent.args).current_dir(state.index.root());
    // npm wrappers use /usr/bin/env node, including in a downloaded macOS app.
    if let Some(node) =
        self::executable("node", &dir).and_then(|p| p.parent().map(Path::to_path_buf))
    {
        let mut paths = vec![node];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        cmd.env("PATH", std::env::join_paths(paths)?);
    }
    // Agents keep their own login and permissions. Never inject Hickory's API keys.
    Ok(cmd)
}

</storycode:copy>
<a id="f-c2b0eace1865-npm_process"></a>
### `npm_process`

<storycode:copy id="f-c2b0eace1865-npm_process">
pub fn npm_process(npm: &Path, directory: &Path) -> Result<tokio::process::Command> {
    if npm.extension().is_some_and(|ext| ext == "cmd") {
        let node = executable("node", directory)
            .context("Install Node.js before installing the adapter.")?;
        let script = npm
            .parent()
            .context("npm has no directory")?
            .join("node_modules/npm/bin/npm-cli.js");
        anyhow::ensure!(
            script.is_file(),
            "Cannot locate npm's script. Reinstall Node.js, then try again."
        );
        let mut cmd = tokio::process::Command::new(node);
        cmd.arg(script);
        Ok(cmd)
    } else {
        Ok(tokio::process::Command::new(npm))
    }
}

</storycode:copy>
<a id="f-c2b0eace1865-tests"></a>
### `tests`

<storycode:copy id="f-c2b0eace1865-tests">
#[cfg(test)]
mod tests {
    use super::*;

    // Guarantee: docs/guarantees/agent/acp-agents-are-first-class.md
    #[test]
    fn detects_a_managed_adapter_without_launching_it() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("node_modules/.bin");
        std::fs::create_dir_all(&bin).unwrap();
        let adapter = bin.join("hickory-test-acp");
        std::fs::write(&adapter, "not a running process").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert!(executable("hickory-test-acp", dir.path()).is_none());
            assert!(executable(adapter.to_str().unwrap(), dir.path()).is_none());
            std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert_eq!(
            executable("hickory-test-acp", dir.path()),
            Some(adapter.clone())
        );
        assert_eq!(
            executable(adapter.to_str().unwrap(), dir.path()),
            Some(adapter)
        );
        assert!(executable(bin.to_str().unwrap(), dir.path()).is_none());
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/acp/config.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/acp/config.rs"><storycode:paste select="#f-c2b0eace1865-AgentCommand"/><storycode:paste select="#f-c2b0eace1865-directory"/><storycode:paste select="#f-c2b0eace1865-commands"/><storycode:paste select="#f-c2b0eace1865-validate"/><storycode:paste select="#f-c2b0eace1865-executable"/><storycode:paste select="#f-c2b0eace1865-is_executable"/><storycode:paste select="#f-c2b0eace1865-process"/><storycode:paste select="#f-c2b0eace1865-npm_process"/><storycode:paste select="#f-c2b0eace1865-tests"/></storycode:file>

<a id="output-a3b5b37c33a3"></a>
## `crates/hickory-cli/src/serve/acp/workspace.rs`

Related contracts: [ACP Agents Work In The Agent Pane And Leave A Hickory Record](contract.md).

<a id="f-a3b5b37c33a3-Snapshot"></a>
### `Snapshot`

Apply external edits against the room revision the tool actually read.
CRDT operations preserve concurrent keystrokes outside the changed region.

<storycode:copy id="f-a3b5b37c33a3-Snapshot">
//! Apply external edits against the room revision the tool actually read.
//! CRDT operations preserve concurrent keystrokes outside the changed region.
use super::client::RoomContext;
use anyhow::Result;
use std::collections::HashMap;
use yrs::sync::{Message, SyncMessage};
use yrs::updates::{decoder::Decode as _, encoder::Encode as _};
use yrs::{ReadTxn as _, Text as _, Transact as _, Update};

pub type Snapshot = (String, Vec<u8>);

</storycode:copy>
<a id="f-a3b5b37c33a3-capture"></a>
### `capture`

<storycode:copy id="f-a3b5b37c33a3-capture">
pub async fn capture(context: &RoomContext) -> Result<HashMap<String, Snapshot>> {
    let mut snapshots = HashMap::new();
    for (id, _) in context.index.entries() {
        if let Some(room) = context.rooms.get(&id).await {
            let snapshot = room.snapshot().await;
            if let Some(path) = context.index.absolute(&id) {
                super::super::store::write_atomic(&path, snapshot.0.as_bytes())?;
            }
            snapshots.insert(id, snapshot);
        }
    }
    Ok(snapshots)
}

</storycode:copy>
<a id="f-a3b5b37c33a3-apply"></a>
### `apply`

<storycode:copy id="f-a3b5b37c33a3-apply">
pub async fn apply(
    context: &RoomContext,
    id: &str,
    base: &Snapshot,
    source: &str,
) -> Result<String> {
    let Some(room) = context.rooms.get(id).await else {
        return Ok(source.into());
    };
    if base.0 == source {
        return Ok(room.text().await);
    }
    // This detached doc shares history, but mints the external edit under a
    // fresh identity. Applying its update cannot delete later room operations.
    let doc = hickory_collab::doc_with_client_id(super::super::rand_id() & ((1 << 53) - 1));
    let text = doc.get_or_insert_text("source");
    let update = {
        let mut txn = doc.transact_mut();
        txn.apply_update(Update::decode_v1(&base.1)?)?;
        let before = txn.state_vector();
        let (start, len, inserted) = hickory_collab::text_delta(&base.0, source);
        if len > 0 {
            text.remove_range(&mut txn, start as u32, len as u32);
        }
        if !inserted.is_empty() {
            text.insert(&mut txn, start as u32, inserted);
        }
        txn.encode_diff_v1(&before)
    };
    let payload = Message::Sync(SyncMessage::Update(update)).encode_v1();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    context
        .rooms
        .handle_yjs_payload(&room, doc.client_id(), &tx, &payload, true)
        .await?;
    Ok(room.text().await)
}

</storycode:copy>
<a id="f-a3b5b37c33a3-finish"></a>
### `finish`

<storycode:copy id="f-a3b5b37c33a3-finish">
pub async fn finish(context: &RoomContext, snapshots: HashMap<String, Snapshot>) -> Result<()> {
    for (id, base) in snapshots {
        if let Some(path) = context.index.absolute(&id) {
            let source = std::fs::read_to_string(&path)?;
            if source != base.0 {
                let merged = apply(context, &id, &base, &source).await?;
                super::super::store::write_atomic(&path, merged.as_bytes())?;
            }
        }
    }
    Ok(())
}

</storycode:copy>
<a id="f-a3b5b37c33a3-tests"></a>
### `tests`

<storycode:copy id="f-a3b5b37c33a3-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/acp-agents-are-first-class.md
    #[tokio::test]
    async fn external_edit_keeps_concurrent_typing_and_utf16_coordinates() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let original = "# 📝 Note\n\nhello\n";
        std::fs::write(root.join("note.md"), original).unwrap();
        let prepared = crate::serve::prepare(crate::serve::ServeOptions {
            target: root.clone(),
            port: 0,
            params: vec![],
            executor: crate::ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        })
        .await
        .unwrap();
        let context = RoomContext {
            index: prepared.state.index.clone(),
            rooms: prepared.state.rooms.clone(),
        };
        let id = context.index.sole().unwrap().0;
        let room = context.rooms.get_or_create(&id).await.unwrap();
        let base = room.snapshot().await;
        context
            .rooms
            .apply_external_source(&id, &format!("{original}My concurrent note.\n"))
            .await;
        let merged = apply(
            &context,
            &id,
            &base,
            &original.replace("hello", "agent edit"),
        )
        .await
        .unwrap();
        assert_eq!(merged, "# 📝 Note\n\nagent edit\nMy concurrent note.\n");
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/acp/workspace.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/acp/workspace.rs"><storycode:paste select="#f-a3b5b37c33a3-Snapshot"/><storycode:paste select="#f-a3b5b37c33a3-capture"/><storycode:paste select="#f-a3b5b37c33a3-apply"/><storycode:paste select="#f-a3b5b37c33a3-finish"/><storycode:paste select="#f-a3b5b37c33a3-tests"/></storycode:file>

## Test the behavior: `desktop_proxy_routes_acp_connection_configuration_and_turns`

This is the maintained test fragment from `crates/hickory-cli/tests/engine_lifecycle.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6ea35164aaa4-desktop_proxy_routes_acp_connection_configuration_and_turns"></a>
### `desktop_proxy_routes_acp_connection_configuration_and_turns`

<storycode:copy id="f-6ea35164aaa4-desktop_proxy_routes_acp_connection_configuration_and_turns">
// Guarantee: docs/guarantees/agent/acp-agents-are-first-class.md
#[tokio::test(flavor = "multi_thread")]
async fn desktop_proxy_routes_acp_connection_configuration_and_turns() {
    use futures::StreamExt;
    let project = Project::new();
    // Use the current Markdown document surface for this regression.
    std::fs::remove_file(project.dir.path().join("note.md")).unwrap();
    std::fs::write(
        project.dir.path().join("note.md"),
        DOC.replace("weave=\"note.md\"", "weave=\"reading.md\""),
    )
    .unwrap();
    let client = project.client(project.dir.path(), "ACP");
    let listing = docs(&client).await;
    assert!(
        listing.is_array(),
        "document listing through desktop proxy: {listing}"
    );
    let id = listing[0]["id"].as_str().unwrap().to_owned();
    let peer = PathBuf::from(env!("CARGO_BIN_EXE_hick"))
        .parent()
        .unwrap()
        .join("examples")
        .join(format!("acp_fixture{}", std::env::consts::EXE_SUFFIX));
    assert!(peer.is_file(), "build the ACP peer with just test-engine");
    let http = reqwest::Client::new();
    let response = http
        .put(format!("{}/api/agents", client.url))
        .json(&json!([{"id":"fixture","name":"ACP fixture","command":peer,"args":["--auth"]}]))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let base = format!("{}/api/docs/{id}/agent", client.url);
    let response = http
        .post(format!("{base}/acp"))
        .json(&json!({"backend":"fixture"}))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let connected: Value = response.json().await.unwrap();
    assert_eq!(connected["ready"], false, "{connected}");
    let response = http
        .post(format!("{base}/acp/authenticate"))
        .json(&json!({"method_id":"login"}))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let authenticated: Value = response.json().await.unwrap();
    assert_eq!(authenticated["ready"], true, "{authenticated}");
    let response = http.get(format!("{base}/acp")).send().await.unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let response = http
        .post(format!("{base}/acp/configure"))
        .json(&json!({"config_id":"model","value":"other"}))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let (mut socket, _) = tokio_tungstenite::connect_async(format!(
        "{}/api/ws?doc=doc:{id}",
        client.url.replacen("http://", "ws://", 1)
    ))
    .await
    .unwrap();
    let response = http
        .post(&base)
        .json(&json!({"backend":"fixture","prompt":"hello"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202, "{}", response.text().await.unwrap());
    let sent: Value = response.json().await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let response = http.get(format!("{base}/turns")).send().await.unwrap();
        assert!(
            response.status().is_success(),
            "{}",
            response.text().await.unwrap()
        );
        let turns: Value = response.json().await.unwrap();
        if let Some(turn) = turns["turns"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == sent["session_id"] && t["status"] != "running")
        {
            assert_eq!(turn["status"], "ok", "{turn}");
            assert_eq!(turn["provider"], "acp:fixture");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "ACP turn never finished: {turns}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut streamed = false;
        while let Some(message) = socket.next().await {
            if let tokio_tungstenite::tungstenite::Message::Binary(bytes) = message.unwrap()
                && bytes.first() == Some(&0x01)
            {
                let event: Value = serde_json::from_slice(&bytes[1..]).unwrap();
                if event["run_id"] != sent["session_id"] {
                    continue;
                }
                streamed |= event["event"]["kind"] == "token";
                if event["status"] == "ok" {
                    assert!(streamed);
                    return;
                }
            }
        }
        panic!("agent socket ended before the turn completed");
    })
    .await
    .expect("agent turn never arrived through the desktop WebSocket");
    socket.close(None).await.unwrap();
}

</storycode:copy>

## Test the behavior: `DOC`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_acp.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2fce65a349e6-DOC"></a>
### `DOC`

Protects docs/guarantees/agent/acp-agents-are-first-class.md.
A protocol peer is necessary to test races, permissions and hung-process
cancellation deterministically. It is never a fallback for missing login.

<storycode:copy id="f-2fce65a349e6-DOC">
//! Protects docs/guarantees/agent/acp-agents-are-first-class.md.
//! A protocol peer is necessary to test races, permissions and hung-process
//! cancellation deterministically. It is never a fallback for missing login.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::{path::PathBuf, time::Duration};

const DOC: &str = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/AcpControls.tsx](../python-execution-has-two-boundaries/contract.md#output-8403c94b656f)
- [apps/web/src/components/AcpRecord.tsx](../../../mechanisms/apps/web/src/components/AcpRecord.tsx.md#output-1861640bb163)
- [apps/web/src/components/AgentPicker.tsx](../../verification/predictions-precede-managed-observation/implementation/apps/web/src/components/AgentPicker.tsx.md#output-2cb1fb2fe5c8)
- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [apps/web/src/views/AgentSettings.tsx](../a-desktop-app-bundles-its-python-agent/contract.md#output-5b4b3eeb9113)
- [crates/hickory-cli/src/engine/client.rs](../../../mechanisms/crates/hickory-cli/src/engine/client.rs.md#output-5c9c7cadce49)
- [crates/hickory-cli/src/engine/watch.rs](../../../mechanisms/crates/hickory-cli/src/engine/watch.rs.md#output-dc5bee2ae567)
- [crates/hickory-cli/src/serve/acp/config.rs](contract.md#output-c2b0eace1865)
- [crates/hickory-cli/src/serve/acp/record.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/record.rs.md#output-56d0a133283e)
- [crates/hickory-cli/src/serve/acp/workspace.rs](contract.md#output-a3b5b37c33a3)
- [crates/hickory-cli/src/serve/agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent.rs.md#output-2ce16640df92)
- [crates/hickory-cli/src/up/state.rs](../../../mechanisms/crates/hickory-cli/src/up/state.rs.md#output-d4f59b7595b5)

### Tests and fixtures

- [apps/web/src/components/AcpControls.test.tsx](../../../mechanisms/apps/web/src/components/AcpControls.test.tsx.md#output-616d79251e2d)
- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- [apps/web/src/components/FeatureSettings.test.tsx](../../../mechanisms/apps/web/src/components/FeatureSettings.test.tsx.md#output-637fcce1408f)
- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../mechanisms/crates/hickory-cli/tests/engine_lifecycle.rs.md#output-6ea35164aaa4)
- [crates/hickory-cli/tests/serve_acp.rs](../../../mechanisms/crates/hickory-cli/tests/serve_acp.rs.md#output-2fce65a349e6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
