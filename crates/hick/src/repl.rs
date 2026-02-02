//! Interactive REPL for hick.
//!
//! When `hick` is invoked with no arguments, this module provides an
//! interactive session that boots a default container, executes user
//! commands, and records a `.hick` transcript file that can be replayed.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, bail};
use log::{info, warn};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use hick_token::{ContainerCapabilities, NetworkRule};

use crate::executor::ContainerExecutor;
use crate::pool::{ContainerPool, PoolConfig, SharedPool};
use crate::transcript::{CapabilityRule, TranscriptBuilder};

// ---------------------------------------------------------------------------
// Meta-command parsing
// ---------------------------------------------------------------------------

/// A parsed meta-command from the REPL.
#[derive(Debug, PartialEq)]
pub enum MetaCommand {
    Container { name: String, image: String },
    Name { name: String },
    Volume { name: String },
    Mount { volume: String, path: String },
    Fork { from: String, to: String },
    AllowNetwork { host: String, port: String },
    AllowFileRead { path: String },
    AllowFileWrite { path: String },
    DenyNetwork,
    Containers,
    Volumes,
    Transcript,
    Save { filename: String },
    Help,
    Quit,
}

/// Parse a colon-prefixed meta-command. Returns `None` if the input
/// doesn't start with `:` or is unrecognized.
pub fn parse_meta_command(input: &str) -> Option<MetaCommand> {
    let input = input.trim();
    if !input.starts_with(':') {
        return None;
    }

    let without_colon = &input[1..];
    let parts: Vec<&str> = without_colon.splitn(3, char::is_whitespace).collect();

    match parts.first().copied()? {
        "container" => {
            let name = parts.get(1)?.to_string();
            let image = parts.get(2).unwrap_or(&name.as_str()).to_string();
            Some(MetaCommand::Container { name, image })
        }
        "name" => {
            let name = parts.get(1)?.to_string();
            Some(MetaCommand::Name { name })
        }
        "volume" => {
            let name = parts.get(1)?.to_string();
            Some(MetaCommand::Volume { name })
        }
        "mount" => {
            let spec = parts.get(1)?;
            let (volume, path) = spec.split_once(':')?;
            Some(MetaCommand::Mount {
                volume: volume.to_string(),
                path: path.to_string(),
            })
        }
        "fork" => {
            let from = parts.get(1)?.to_string();
            let to = parts.get(2)?.to_string();
            Some(MetaCommand::Fork { from, to })
        }
        "allow" => {
            let sub = parts.get(1)?;
            match *sub {
                "network" => {
                    let target = parts.get(2)?;
                    let (host, port) = target.rsplit_once(':')?;
                    Some(MetaCommand::AllowNetwork {
                        host: host.to_string(),
                        port: port.to_string(),
                    })
                }
                "file-read" => {
                    let path = parts.get(2)?.to_string();
                    Some(MetaCommand::AllowFileRead { path })
                }
                "file-write" => {
                    let path = parts.get(2)?.to_string();
                    Some(MetaCommand::AllowFileWrite { path })
                }
                _ => None,
            }
        }
        "deny" => {
            let sub = parts.get(1)?;
            match *sub {
                "network" => Some(MetaCommand::DenyNetwork),
                _ => None,
            }
        }
        "containers" => Some(MetaCommand::Containers),
        "volumes" => Some(MetaCommand::Volumes),
        "transcript" => Some(MetaCommand::Transcript),
        "save" => {
            let filename = parts.get(1)?.to_string();
            Some(MetaCommand::Save { filename })
        }
        "help" => Some(MetaCommand::Help),
        "quit" | "q" => Some(MetaCommand::Quit),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// REPL session
// ---------------------------------------------------------------------------

/// Tracks which containers have had commands executed (are "booted").
struct ReplSession {
    transcript: TranscriptBuilder,
    current_container: String,
    /// Containers that have had at least one shell command executed.
    booted_containers: HashSet<String>,
    /// Accumulated capabilities per container (applied at boot time).
    pending_caps: HashMap<String, ContainerCapabilities>,
    /// The executor, created on demand.
    executor: Option<ContainerExecutor>,
    images_dir: PathBuf,
    output_path: PathBuf,
    /// Pre-warming pool for fast container startup.
    pool: SharedPool,
}

impl ReplSession {
    fn new(images_dir: PathBuf, output_path: PathBuf, pool: SharedPool) -> Self {
        let mut transcript = TranscriptBuilder::new();
        transcript.declare_container("alpine", "alpine");
        transcript.add_capability("alpine", CapabilityRule::DenyNetwork);

        let mut pending_caps: HashMap<String, ContainerCapabilities> = HashMap::new();
        pending_caps.insert(
            "alpine".to_string(),
            ContainerCapabilities::new().deny_all_network(),
        );

        Self {
            transcript,
            current_container: "alpine".to_string(),
            booted_containers: HashSet::new(),
            pending_caps,
            executor: None,
            images_dir,
            output_path,
            pool,
        }
    }

    async fn ensure_executor(&mut self) -> Result<()> {
        if self.executor.is_none() {
            let caps = self
                .pending_caps
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            self.executor = Some(
                ContainerExecutor::new_with_pool(self.images_dir.clone(), caps, self.pool.clone())
                    .await?,
            );
        }
        Ok(())
    }

    /// Rebuild the executor with updated capabilities. This is needed when
    /// a new container is declared before any commands run — the executor
    /// needs the full capability map at construction time.
    async fn rebuild_executor_caps(&mut self) -> Result<()> {
        if self.executor.is_some() && self.booted_containers.is_empty() {
            // The executor doesn't support adding caps after construction,
            // but if no containers have booted yet, we can rebuild with
            // the full capability map.
            let caps: HashMap<String, ContainerCapabilities> = self
                .pending_caps
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            self.executor = Some(
                ContainerExecutor::new_with_pool(self.images_dir.clone(), caps, self.pool.clone())
                    .await?,
            );
        }
        Ok(())
    }

    async fn execute_command(&mut self, command: &str) -> Result<String> {
        self.ensure_executor().await?;

        let container = self.current_container.clone();
        let executor = self.executor.as_mut().unwrap();

        if !self.booted_containers.contains(&container) {
            // Find the image for this container from the transcript
            let image = self
                .transcript
                .container_names()
                .iter()
                .position(|n| n == &container)
                .and(None::<String>)
                .unwrap_or_else(|| "alpine".to_string());

            executor.ensure_started(&container, &image).await?;
            self.booted_containers.insert(container.clone());
        }

        let output = executor.execute(&container, command).await?;
        self.transcript.record_command(&container, command);
        Ok(output)
    }

    async fn handle_meta_command(&mut self, cmd: MetaCommand) -> MetaCommandResult {
        match cmd {
            MetaCommand::Container { name, image } => {
                if self.transcript.has_container(&name) {
                    return MetaCommandResult::Error(format!("Container '{name}' already exists"));
                }
                self.transcript.declare_container(&name, &image);
                self.transcript
                    .add_capability(&name, CapabilityRule::DenyNetwork);
                self.pending_caps.insert(
                    name.clone(),
                    ContainerCapabilities::new().deny_all_network(),
                );
                if let Err(e) = self.rebuild_executor_caps().await {
                    return MetaCommandResult::Error(format!("Failed to update executor: {e}"));
                }
                // Pre-warm the image in the background
                let pool = self.pool.clone();
                let warm_image = image.clone();
                tokio::spawn(async move {
                    let mut pool_guard = pool.lock().await;
                    if let Err(e) = pool_guard.warm(&warm_image, 1).await {
                        warn!("Pool: background warm failed for '{warm_image}': {e}");
                    }
                });
                self.current_container = name.clone();
                MetaCommandResult::Message(format!("Created container '{name}' (image: {image})"))
            }
            MetaCommand::Name { name } => {
                let current = &self.current_container;
                if self.booted_containers.contains(current) {
                    return MetaCommandResult::Error(
                        "Cannot rename after commands executed".to_string(),
                    );
                }
                let old = current.clone();
                if self.transcript.rename_container(&old, &name) {
                    // Update pending caps key
                    if let Some(caps) = self.pending_caps.remove(&old) {
                        self.pending_caps.insert(name.clone(), caps);
                    }
                    self.current_container = name.clone();
                    if let Err(e) = self.rebuild_executor_caps().await {
                        return MetaCommandResult::Error(format!("Failed to update executor: {e}"));
                    }
                    MetaCommandResult::Message(format!("Renamed '{old}' → '{name}'"))
                } else {
                    MetaCommandResult::Error(format!("Container '{old}' not found"))
                }
            }
            MetaCommand::Volume { name } => {
                self.transcript.create_volume(&name);
                MetaCommandResult::Message(format!("Created volume '{name}'"))
            }
            MetaCommand::Mount { volume, path } => {
                let container = self.current_container.clone();
                self.transcript.mount_volume(&container, &volume, &path);
                MetaCommandResult::Message(format!(
                    "Mounted volume '{volume}' at '{path}' in '{container}'"
                ))
            }
            MetaCommand::Fork { from, to } => {
                if !self.transcript.has_container(&from) {
                    return MetaCommandResult::Error(format!(
                        "Source container '{from}' not declared"
                    ));
                }
                self.transcript.record_fork(&from, &to);
                // Inherit capabilities from source
                let source_caps = self
                    .pending_caps
                    .get(&from)
                    .cloned()
                    .unwrap_or_else(ContainerCapabilities::new);
                self.pending_caps.insert(to.clone(), source_caps);
                if let Some(ref mut executor) = self.executor {
                    executor.register_fork(&to, &from, None);
                }
                self.current_container = to.clone();
                MetaCommandResult::Message(format!("Forked '{from}' → '{to}'"))
            }
            MetaCommand::AllowNetwork { host, port } => {
                let container = self.current_container.clone();
                self.transcript.add_capability(
                    &container,
                    CapabilityRule::AllowNetwork {
                        host: host.clone(),
                        port: port.clone(),
                    },
                );
                if self.booted_containers.contains(&container) {
                    // Record for replay but warn
                    return MetaCommandResult::Warning(format!(
                        "Recorded allow network {host}:{port} for '{container}' \
                         (won't take effect until replay)"
                    ));
                }
                let caps = self.pending_caps.entry(container.clone()).or_default();
                caps.network_rules.push(NetworkRule::allow(&host, &port));
                MetaCommandResult::Message(format!(
                    "Allowed network {host}:{port} for '{container}'"
                ))
            }
            MetaCommand::AllowFileRead { path } => {
                let container = self.current_container.clone();
                self.transcript.add_capability(
                    &container,
                    CapabilityRule::AllowFileRead { path: path.clone() },
                );
                if self.booted_containers.contains(&container) {
                    return MetaCommandResult::Warning(format!(
                        "Recorded allow file-read {path} for '{container}' \
                         (won't take effect until replay)"
                    ));
                }
                let caps = self.pending_caps.entry(container.clone()).or_default();
                caps.file_rules
                    .push(hick_token::FileRule::Read(path.clone()));
                MetaCommandResult::Message(format!("Allowed file-read '{path}' for '{container}'"))
            }
            MetaCommand::AllowFileWrite { path } => {
                let container = self.current_container.clone();
                self.transcript.add_capability(
                    &container,
                    CapabilityRule::AllowFileWrite { path: path.clone() },
                );
                if self.booted_containers.contains(&container) {
                    return MetaCommandResult::Warning(format!(
                        "Recorded allow file-write {path} for '{container}' \
                         (won't take effect until replay)"
                    ));
                }
                let caps = self.pending_caps.entry(container.clone()).or_default();
                caps.file_rules
                    .push(hick_token::FileRule::Write(path.clone()));
                MetaCommandResult::Message(format!("Allowed file-write '{path}' for '{container}'"))
            }
            MetaCommand::DenyNetwork => {
                let container = self.current_container.clone();
                self.transcript
                    .add_capability(&container, CapabilityRule::DenyNetwork);
                if self.booted_containers.contains(&container) {
                    return MetaCommandResult::Warning(format!(
                        "Recorded deny network for '{container}' \
                         (won't take effect until replay)"
                    ));
                }
                let caps = self.pending_caps.entry(container.clone()).or_default();
                caps.network_rules.push(NetworkRule::deny_all());
                MetaCommandResult::Message(format!("Denied network for '{container}'"))
            }
            MetaCommand::Containers => {
                let names = self.transcript.container_names();
                let mut lines = Vec::new();
                for name in &names {
                    let marker = if *name == self.current_container {
                        " *"
                    } else {
                        ""
                    };
                    let booted = if self.booted_containers.contains(name) {
                        " (running)"
                    } else {
                        ""
                    };
                    lines.push(format!("  {name}{marker}{booted}"));
                }
                MetaCommandResult::Message(lines.join("\n"))
            }
            MetaCommand::Volumes => {
                let names = self.transcript.volume_names();
                if names.is_empty() {
                    MetaCommandResult::Message("No volumes".to_string())
                } else {
                    let lines: Vec<String> = names.iter().map(|n| format!("  {n}")).collect();
                    MetaCommandResult::Message(lines.join("\n"))
                }
            }
            MetaCommand::Transcript => MetaCommandResult::Message(self.transcript.to_hick_xml()),
            MetaCommand::Save { filename } => {
                let xml = self.transcript.to_hick_xml();
                match std::fs::write(&filename, &xml) {
                    Ok(()) => {
                        MetaCommandResult::Message(format!("Saved transcript to '{filename}'"))
                    }
                    Err(e) => MetaCommandResult::Error(format!("Failed to save: {e}")),
                }
            }
            MetaCommand::Help => MetaCommandResult::Message(HELP_TEXT.to_string()),
            MetaCommand::Quit => MetaCommandResult::Quit,
        }
    }

    async fn shutdown(&mut self) {
        if let Some(ref mut executor) = self.executor {
            executor.shutdown().await;
        }
        let mut pool_guard = self.pool.lock().await;
        pool_guard.shutdown().await;
    }

    fn save_transcript(&self) -> Result<()> {
        let xml = self.transcript.to_hick_xml();
        let path = &self.output_path;
        if path.exists() {
            eprintln!("Warning: overwriting existing '{}'", path.display());
        }
        std::fs::write(path, &xml)?;
        info!("Saved transcript to {}", path.display());
        Ok(())
    }
}

enum MetaCommandResult {
    Message(String),
    Warning(String),
    Error(String),
    Quit,
}

const HELP_TEXT: &str = "\
Commands:
  <command>                     Execute a shell command in the current container
  :container <name> <image>     Create/switch to a named container (deny-by-default)
  :name <name>                  Rename current container (before any commands)
  :volume <name>                Create a named volume
  :mount <vol>:<path>           Mount a volume in current container
  :fork <from> <to>             Fork a container
  :allow network <host:port>    Grant network access to current container
  :allow file-read <path>       Grant file read to current container
  :allow file-write <path>      Grant file write to current container
  :deny network *               Deny all network (already default)
  :containers                   List all containers
  :volumes                      List all volumes
  :transcript                   Print current .hick transcript
  :save <filename>              Save transcript to specific file
  :help                         Show this help
  :quit                         Exit and save";

// ---------------------------------------------------------------------------
// REPL entry point
// ---------------------------------------------------------------------------

/// Run the interactive REPL.
///
/// Creates a default "alpine" container with deny-all-network, enters a
/// readline loop, and saves `session.hick` on exit.
pub async fn run_repl(images_dir: PathBuf, output_path: PathBuf) -> Result<()> {
    let mut editor = DefaultEditor::new()?;

    // Load history
    let history_path = dirs_history_path();
    let _ = editor.load_history(&history_path);

    // Create container pool and pre-warm the default alpine image
    let pool_config = PoolConfig::new(images_dir.clone());
    info!("Pool: created (max_warm: {})", pool_config.max_warm);
    let pool: SharedPool = Arc::new(tokio::sync::Mutex::new(ContainerPool::new(pool_config)));
    {
        let mut pool_guard = pool.lock().await;
        if let Err(e) = pool_guard.warm("alpine", 1).await {
            warn!("Pool: failed to pre-warm alpine: {e}");
        }
    }

    let mut session = ReplSession::new(images_dir, output_path, pool);

    println!("hick interactive session (type :help for commands, :quit to exit)");
    println!("Default container: alpine (deny-all-network)");

    loop {
        let prompt = format!("[{}] > ", session.current_container);
        match editor.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                editor.add_history_entry(trimmed)?;

                if trimmed.starts_with(':') {
                    match parse_meta_command(trimmed) {
                        Some(cmd) => match session.handle_meta_command(cmd).await {
                            MetaCommandResult::Message(msg) => println!("{msg}"),
                            MetaCommandResult::Warning(msg) => {
                                eprintln!("Warning: {msg}");
                            }
                            MetaCommandResult::Error(msg) => {
                                eprintln!("Error: {msg}");
                            }
                            MetaCommandResult::Quit => break,
                        },
                        None => {
                            eprintln!("Unknown command: {trimmed}");
                            eprintln!("Type :help for available commands");
                        }
                    }
                } else {
                    // Shell command
                    match session.execute_command(trimmed).await {
                        Ok(output) => {
                            if !output.is_empty() {
                                println!("{output}");
                            }
                        }
                        Err(e) => {
                            eprintln!("Execution error: {e}");
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: just continue
                println!("(Ctrl-C to cancel, :quit to exit)");
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: exit
                break;
            }
            Err(e) => {
                bail!("Readline error: {e}");
            }
        }
    }

    // Shutdown and save
    session.shutdown().await;
    session.save_transcript()?;
    println!("Session saved to {}", session.output_path.display());

    // Save history
    let _ = editor.save_history(&history_path);

    Ok(())
}

/// Path to the REPL history file.
fn dirs_history_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".hick_history")
    } else {
        PathBuf::from(".hick_history")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_container_command() {
        assert_eq!(
            parse_meta_command(":container web alpine"),
            Some(MetaCommand::Container {
                name: "web".into(),
                image: "alpine".into(),
            })
        );
    }

    #[test]
    fn parse_container_defaults_image_to_name() {
        assert_eq!(
            parse_meta_command(":container ubuntu"),
            Some(MetaCommand::Container {
                name: "ubuntu".into(),
                image: "ubuntu".into(),
            })
        );
    }

    #[test]
    fn parse_name() {
        assert_eq!(
            parse_meta_command(":name worker"),
            Some(MetaCommand::Name {
                name: "worker".into(),
            })
        );
    }

    #[test]
    fn parse_volume() {
        assert_eq!(
            parse_meta_command(":volume data"),
            Some(MetaCommand::Volume {
                name: "data".into(),
            })
        );
    }

    #[test]
    fn parse_mount() {
        assert_eq!(
            parse_meta_command(":mount data:/mnt/data"),
            Some(MetaCommand::Mount {
                volume: "data".into(),
                path: "/mnt/data".into(),
            })
        );
    }

    #[test]
    fn parse_fork() {
        assert_eq!(
            parse_meta_command(":fork base derived"),
            Some(MetaCommand::Fork {
                from: "base".into(),
                to: "derived".into(),
            })
        );
    }

    #[test]
    fn parse_allow_network() {
        assert_eq!(
            parse_meta_command(":allow network github.com:443"),
            Some(MetaCommand::AllowNetwork {
                host: "github.com".into(),
                port: "443".into(),
            })
        );
    }

    #[test]
    fn parse_allow_file_read() {
        assert_eq!(
            parse_meta_command(":allow file-read /data"),
            Some(MetaCommand::AllowFileRead {
                path: "/data".into(),
            })
        );
    }

    #[test]
    fn parse_allow_file_write() {
        assert_eq!(
            parse_meta_command(":allow file-write /output"),
            Some(MetaCommand::AllowFileWrite {
                path: "/output".into(),
            })
        );
    }

    #[test]
    fn parse_deny_network() {
        assert_eq!(
            parse_meta_command(":deny network *"),
            Some(MetaCommand::DenyNetwork)
        );
    }

    #[test]
    fn parse_list_commands() {
        assert_eq!(
            parse_meta_command(":containers"),
            Some(MetaCommand::Containers)
        );
        assert_eq!(parse_meta_command(":volumes"), Some(MetaCommand::Volumes));
        assert_eq!(
            parse_meta_command(":transcript"),
            Some(MetaCommand::Transcript)
        );
    }

    #[test]
    fn parse_save() {
        assert_eq!(
            parse_meta_command(":save output.hick"),
            Some(MetaCommand::Save {
                filename: "output.hick".into(),
            })
        );
    }

    #[test]
    fn parse_help_and_quit() {
        assert_eq!(parse_meta_command(":help"), Some(MetaCommand::Help));
        assert_eq!(parse_meta_command(":quit"), Some(MetaCommand::Quit));
        assert_eq!(parse_meta_command(":q"), Some(MetaCommand::Quit));
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert_eq!(parse_meta_command(":unknown"), None);
        assert_eq!(parse_meta_command("echo hello"), None);
    }

    #[test]
    fn parse_with_whitespace() {
        assert_eq!(
            parse_meta_command("  :container web alpine  "),
            Some(MetaCommand::Container {
                name: "web".into(),
                image: "alpine".into(),
            })
        );
    }

    fn test_pool() -> SharedPool {
        let config = PoolConfig::new(PathBuf::from("/tmp/images"));
        Arc::new(tokio::sync::Mutex::new(ContainerPool::new(config)))
    }

    #[tokio::test]
    async fn session_meta_commands() {
        let mut session = ReplSession::new(
            PathBuf::from("/tmp/images"),
            PathBuf::from("session.hick"),
            test_pool(),
        );

        // Default container exists
        assert_eq!(session.current_container, "alpine");

        // Create new container
        let result = session
            .handle_meta_command(MetaCommand::Container {
                name: "web".into(),
                image: "alpine".into(),
            })
            .await;
        assert!(matches!(result, MetaCommandResult::Message(_)));
        assert_eq!(session.current_container, "web");

        // Duplicate container
        let result = session
            .handle_meta_command(MetaCommand::Container {
                name: "web".into(),
                image: "alpine".into(),
            })
            .await;
        assert!(matches!(result, MetaCommandResult::Error(_)));

        // Rename
        let result = session
            .handle_meta_command(MetaCommand::Name { name: "api".into() })
            .await;
        assert!(matches!(result, MetaCommandResult::Message(_)));
        assert_eq!(session.current_container, "api");

        // Volume
        let result = session
            .handle_meta_command(MetaCommand::Volume {
                name: "data".into(),
            })
            .await;
        assert!(matches!(result, MetaCommandResult::Message(_)));

        // List containers
        let result = session.handle_meta_command(MetaCommand::Containers).await;
        if let MetaCommandResult::Message(msg) = result {
            assert!(msg.contains("alpine"));
            assert!(msg.contains("api"));
        } else {
            panic!("Expected Message");
        }

        // List volumes
        let result = session.handle_meta_command(MetaCommand::Volumes).await;
        if let MetaCommandResult::Message(msg) = result {
            assert!(msg.contains("data"));
        } else {
            panic!("Expected Message");
        }
    }

    #[tokio::test]
    async fn fork_requires_existing_source() {
        let mut session = ReplSession::new(
            PathBuf::from("/tmp/images"),
            PathBuf::from("session.hick"),
            test_pool(),
        );

        let result = session
            .handle_meta_command(MetaCommand::Fork {
                from: "nonexistent".into(),
                to: "derived".into(),
            })
            .await;
        assert!(matches!(result, MetaCommandResult::Error(_)));
    }

    #[tokio::test]
    async fn transcript_output() {
        let mut session = ReplSession::new(
            PathBuf::from("/tmp/images"),
            PathBuf::from("session.hick"),
            test_pool(),
        );

        let result = session.handle_meta_command(MetaCommand::Transcript).await;
        if let MetaCommandResult::Message(xml) = result {
            assert!(xml.contains("<hick:container name=\"alpine\""));
            assert!(xml.contains("<hick:deny network=\"*\""));
        } else {
            panic!("Expected transcript XML");
        }
    }
}
