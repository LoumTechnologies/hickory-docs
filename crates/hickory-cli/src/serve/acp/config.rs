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
}

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

pub fn commands(state: &LocalState) -> Result<Vec<AgentCommand>> {
    let path = directory(state).join("agents.json");
    let mut commands = vec![
        AgentCommand {
            id: "codex".into(),
            name: "Codex".into(),
            command: "codex-acp".into(),
            args: vec![],
            workspace_filesystem: false,
        },
        AgentCommand {
            id: "claude".into(),
            name: "Claude Agent".into(),
            command: "claude-agent-acp".into(),
            args: vec![],
            workspace_filesystem: false,
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

pub fn validate(commands: &[AgentCommand]) -> Result<()> {
    let mut ids = std::collections::HashSet::new();
    for agent in commands {
        if agent.id.is_empty()
            || agent.id == "builtin"
            || !agent
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !ids.insert(&agent.id)
        {
            bail!(
                "each agent needs a unique id using letters, numbers or hyphens; builtin is reserved"
            );
        }
        if agent.name.trim().is_empty() || agent.command.trim().is_empty() {
            bail!("{} needs a name and an executable command", agent.id);
        }
    }
    Ok(())
}

pub fn executable(command: &str, directory: &Path) -> Option<PathBuf> {
    let direct = PathBuf::from(command);
    if direct.is_absolute() {
        return direct.is_file().then_some(direct);
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
        .find(|p| p.is_file())
}

pub fn process(agent: &AgentCommand, state: &LocalState) -> Result<tokio::process::Command> {
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
