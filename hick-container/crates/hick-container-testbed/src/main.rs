mod container_manager;

use std::collections::HashMap;
use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use hick_net_bridge::NetworkBridge;
use hick_connection_filter::{ConnectionFilter, ConnectionVerdict, NetworkRequest};
use hick_container::create_async_engine;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{RwLock, mpsc, oneshot};

use container_manager::{ContainerManager, PROMPT, TestbedState, strip_dsr};

// ---------------------------------------------------------------------------
// Interactive connection filter
// ---------------------------------------------------------------------------

struct ApprovalRequest {
    request: NetworkRequest,
    source_label: Option<String>,
    response_tx: oneshot::Sender<ConnectionVerdict>,
}

struct InteractiveFilter {
    tx: mpsc::Sender<ApprovalRequest>,
    cache: Mutex<HashMap<String, ConnectionVerdict>>,
    label: Option<String>,
}

impl InteractiveFilter {
    fn new(tx: mpsc::Sender<ApprovalRequest>) -> Self {
        Self {
            tx,
            cache: Mutex::new(HashMap::new()),
            label: None,
        }
    }

    fn cache_key(request: &NetworkRequest) -> String {
        match request {
            NetworkRequest::Tcp { dest } => format!("tcp:{dest}"),
            NetworkRequest::Dns { domain } => format!("dns:{domain}"),
        }
    }
}

#[async_trait]
impl ConnectionFilter for InteractiveFilter {
    async fn filter(&self, request: &NetworkRequest) -> ConnectionVerdict {
        let key = Self::cache_key(request);

        if let Some(&verdict) = self.cache.lock().unwrap().get(&key) {
            return verdict;
        }

        let (response_tx, response_rx) = oneshot::channel();
        let approval = ApprovalRequest {
            request: match request {
                NetworkRequest::Tcp { dest } => NetworkRequest::Tcp { dest: *dest },
                NetworkRequest::Dns { domain } => NetworkRequest::Dns {
                    domain: domain.clone(),
                },
            },
            source_label: self.label.clone(),
            response_tx,
        };
        if self.tx.send(approval).await.is_err() {
            return ConnectionVerdict::Allow;
        }
        match response_rx.await {
            Ok(verdict) => {
                self.cache.lock().unwrap().insert(key, verdict);
                verdict
            }
            Err(_) => ConnectionVerdict::Allow,
        }
    }
}

// ---------------------------------------------------------------------------
// Terminal raw mode
// ---------------------------------------------------------------------------

struct RawModeGuard {
    orig: libc::termios,
    fd: i32,
}

impl RawModeGuard {
    fn enable() -> Option<Self> {
        let fd = std::io::stdin().as_raw_fd();
        unsafe {
            let mut orig: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut orig) != 0 {
                return None;
            }
            let mut raw = orig;
            raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
            raw.c_iflag &= !(libc::ICRNL | libc::IXON);
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;
            if libc::tcsetattr(fd, libc::TCSAFLUSH, &raw) != 0 {
                return None;
            }
            Some(Self { orig, fd })
        }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSAFLUSH, &self.orig);
        }
    }
}

// ---------------------------------------------------------------------------
// Meta-command parsing
// ---------------------------------------------------------------------------

enum MetaCommand<'a> {
    From {
        image: &'a str,
        name: Option<&'a str>,
    },
    Name(&'a str),
    Switch(&'a str),
    List,
    Kill,
    Detach,
    Exit,
    Help,
}

fn parse_meta_command(line: &str) -> Option<MetaCommand<'_>> {
    let trimmed = line.trim();

    if let Some(rest) = trimmed.strip_prefix(":from ") {
        let rest = rest.trim();
        if rest.is_empty() {
            return None;
        }
        if let Some((image, name)) = rest.rsplit_once(" as ") {
            let image = image.trim();
            let name = name.trim();
            if !image.is_empty() && !name.is_empty() {
                return Some(MetaCommand::From {
                    image,
                    name: Some(name),
                });
            }
        }
        return Some(MetaCommand::From {
            image: rest,
            name: None,
        });
    }

    if let Some(rest) = trimmed.strip_prefix(":name ") {
        let name = rest.trim();
        if !name.is_empty() {
            return Some(MetaCommand::Name(name));
        }
    }

    if let Some(rest) = trimmed.strip_prefix(":switch ") {
        let name = rest.trim();
        if !name.is_empty() {
            return Some(MetaCommand::Switch(name));
        }
    }

    match trimmed {
        ":ls" => Some(MetaCommand::List),
        ":kill" => Some(MetaCommand::Kill),
        ":detach" => Some(MetaCommand::Detach),
        ":exit" => Some(MetaCommand::Exit),
        ":help" | ":h" => Some(MetaCommand::Help),
        _ => None,
    }
}

const HELP_TEXT: &str = "\
\r\n  :from <image>              Start a new container\
\r\n  :from <image> as <name>    Start a named container\
\r\n  :switch <name>             Switch to a named container\
\r\n  :name <name>               Name/rename the active container\
\r\n  :ls                        List all containers\
\r\n  :kill                      Kill the active container\
\r\n  :detach                    Detach (leave running in background)\
\r\n  :exit                      Same as :kill\
\r\n  :help                      Show this help\
\r\n  Ctrl+C                     Kill active container / exit testbed\
\r\n  Ctrl+\\                     Force quit\
\r\n";

const TESTBED_HINT: &[u8] = b"Type :help for commands, :from <image> to start a container\r\n";

// ---------------------------------------------------------------------------
// Input mode
// ---------------------------------------------------------------------------

enum InputMode {
    Normal,
    AwaitingApproval(oneshot::Sender<ConnectionVerdict>),
}

async fn show_testbed_prompt(stdout: &mut tokio::io::Stdout) -> anyhow::Result<()> {
    stdout.write_all(TESTBED_HINT).await?;
    stdout.write_all(b"[testbed] > ").await?;
    stdout.flush().await?;
    Ok(())
}

/// Display the shell prompt if the active container is already interactive.
/// New/booting containers will emit their own prompt naturally.
async fn show_prompt_if_interactive(
    manager: &mut ContainerManager,
    stdout: &mut tokio::io::Stdout,
) -> anyhow::Result<()> {
    if let Some(mc) = manager.active_mut()
        && mc.terminal.testbed_state == TestbedState::Interactive
    {
        stdout.write_all(PROMPT).await?;
        stdout.flush().await?;
        mc.terminal.at_line_start = true;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let _raw = RawModeGuard::enable();

    let engine = create_async_engine()?;
    let bridge = Arc::new(RwLock::new(NetworkBridge::new()));

    let (approval_tx, mut approval_rx) = mpsc::channel::<ApprovalRequest>(16);
    let filter: Arc<dyn ConnectionFilter> = Arc::new(InteractiveFilter::new(approval_tx.clone()));

    let mut manager = ContainerManager::new(engine, filter.clone(), bridge);
    manager.create("alpine", None).await?;

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut stdin_buf = [0u8; 256];
    let mut stdout_buf = [0u8; 4096];

    let mut should_exit = false;
    let mut input_mode = InputMode::Normal;

    // Testbed-prompt line buffer (used when no active container)
    let mut testbed_line_buf: Vec<u8> = Vec::new();

    loop {
        if manager.has_active() {
            // === Active container mode ===
            let mc = manager.active_mut().unwrap();

            tokio::select! {
                result = mc.container.read_stdout(&mut stdout_buf) => {
                    let mc = manager.active_mut().unwrap();
                    match result {
                        Ok(0) => {
                            manager.kill_active().await;
                            if !manager.has_active() {
                                stdout.write_all(b"\r\n").await?;
                                show_testbed_prompt(&mut stdout).await?;
                            } else {
                                show_prompt_if_interactive(&mut manager, &mut stdout).await?;
                            }
                        }
                        Ok(n) => {
                            let after_dsr = strip_dsr(&stdout_buf[..n]);
                            let t = &mut mc.terminal;

                            let after_suppress = if t.testbed_state == TestbedState::Interactive {
                                t.suppressor.process(&after_dsr)
                            } else {
                                after_dsr
                            };

                            let (to_print, prompt_detected) = t.prompt_matcher.process(&after_suppress);

                            if !to_print.is_empty() {
                                stdout.write_all(&to_print).await?;
                            }

                            if prompt_detected {
                                if t.testbed_state == TestbedState::Booting {
                                    t.testbed_state = TestbedState::Interactive;
                                }
                                if matches!(input_mode, InputMode::AwaitingApproval(_)) {
                                    input_mode = InputMode::Normal;
                                }
                                stdout.write_all(PROMPT).await?;
                                t.at_line_start = true;
                            }

                            if !to_print.is_empty() || prompt_detected {
                                stdout.flush().await?;
                            }
                        }
                        Err(e) => {
                            eprintln!("[testbed] stdout read error: {e}");
                            manager.kill_active().await;
                            if !manager.has_active() {
                                stdout.write_all(b"\r\n").await?;
                                show_testbed_prompt(&mut stdout).await?;
                            } else {
                                show_prompt_if_interactive(&mut manager, &mut stdout).await?;
                            }
                        }
                    }
                }

                Some(req) = approval_rx.recv() => {
                    let label_part = req.source_label
                        .as_ref()
                        .map(|l| format!(" [{l}]"))
                        .unwrap_or_default();
                    let prompt_msg = format!(
                        "\r\n[firewall]{label_part} {} \u{2014} allow? (y/n) ",
                        req.request
                    );
                    stdout.write_all(prompt_msg.as_bytes()).await?;
                    stdout.flush().await?;
                    input_mode = InputMode::AwaitingApproval(req.response_tx);
                }

                result = stdin.read(&mut stdin_buf) => {
                    match result {
                        Ok(0) => break,
                        Ok(n) => {
                            for &byte in &stdin_buf[..n] {
                                // Firewall verdict handling
                                if let InputMode::AwaitingApproval(_) = &input_mode {
                                    let verdict = match byte {
                                        b'y' | b'Y' => {
                                            stdout.write_all(b"y\r\n").await?;
                                            stdout.flush().await?;
                                            Some(ConnectionVerdict::Allow)
                                        }
                                        b'n' | b'N' => {
                                            stdout.write_all(b"n\r\n").await?;
                                            stdout.flush().await?;
                                            Some(ConnectionVerdict::Block)
                                        }
                                        _ => None,
                                    };
                                    if let Some(v) = verdict {
                                        let mode = std::mem::replace(
                                            &mut input_mode,
                                            InputMode::Normal,
                                        );
                                        if let InputMode::AwaitingApproval(tx) = mode {
                                            let _ = tx.send(v);
                                        }
                                    }
                                    continue;
                                }

                                // Each byte processing gets a fresh borrow
                                let Some(mc) = manager.active_mut() else {
                                    break;
                                };
                                let t = &mut mc.terminal;

                                match byte {
                                    b'\r' | b'\n' => {
                                        if t.in_meta {
                                            let line = String::from_utf8_lossy(&t.meta_buf).to_string();
                                            stdout.write_all(b"\r\n").await?;
                                            stdout.flush().await?;

                                            if let Some(cmd) = parse_meta_command(&line) {
                                                // Clear meta state before executing command
                                                // (command may change active container)
                                                if let Some(mc) = manager.active_mut() {
                                                    mc.terminal.meta_buf.clear();
                                                    mc.terminal.in_meta = false;
                                                }
                                                handle_meta_command(
                                                    cmd,
                                                    &mut manager,
                                                    &mut stdout,
                                                )
                                                .await?;

                                                if !manager.has_active() {
                                                    show_testbed_prompt(&mut stdout).await?;
                                                    break; // exit byte loop
                                                } else {
                                                    show_prompt_if_interactive(&mut manager, &mut stdout).await?;
                                                }
                                            } else {
                                                // Not a valid meta-command — forward to container
                                                let mc = manager.active_mut().unwrap();
                                                if mc.terminal.testbed_state == TestbedState::Interactive {
                                                    for &b in mc.terminal.meta_buf.clone().iter() {
                                                        mc.terminal.suppressor.push(b);
                                                    }
                                                    mc.terminal.suppressor.push(b'\r');
                                                    mc.terminal.suppressor.push(b'\n');
                                                }
                                                let buf = mc.terminal.meta_buf.clone();
                                                mc.container.write_stdin(&buf).await?;
                                                mc.container.write_stdin(b"\n").await?;
                                                mc.terminal.meta_buf.clear();
                                                mc.terminal.in_meta = false;
                                            }
                                        } else {
                                            let mc = manager.active_mut().unwrap();
                                            if mc.terminal.testbed_state == TestbedState::Interactive {
                                                stdout.write_all(b"\r\n").await?;
                                                stdout.flush().await?;
                                                mc.terminal.suppressor.push(b'\r');
                                                mc.terminal.suppressor.push(b'\n');
                                                mc.container.write_stdin(b"\n").await?;
                                            } else {
                                                mc.container.write_stdin(b"\n").await?;
                                            }
                                            mc.terminal.at_line_start = true;
                                        }
                                    }

                                    // Ctrl+C — kill active container
                                    0x03 => {
                                        if t.in_meta {
                                            t.meta_buf.clear();
                                            t.in_meta = false;
                                            t.at_line_start = true;
                                        }
                                        stdout.write_all(b"^C\r\n").await?;
                                        stdout.flush().await?;
                                        manager.kill_active().await;
                                        if !manager.has_active() {
                                            show_testbed_prompt(&mut stdout).await?;
                                        } else {
                                            show_prompt_if_interactive(&mut manager, &mut stdout).await?;
                                        }
                                        break; // exit byte loop
                                    }

                                    // Ctrl+\ — force quit testbed
                                    0x1c => {
                                        should_exit = true;
                                        break;
                                    }

                                    // Backspace / Delete
                                    0x7f | 0x08 => {
                                        if t.in_meta {
                                            if t.meta_buf.pop().is_some() {
                                                if t.meta_buf.is_empty() {
                                                    t.in_meta = false;
                                                    t.at_line_start = true;
                                                }
                                                stdout.write_all(b"\x08 \x08").await?;
                                                stdout.flush().await?;
                                            }
                                        } else if t.testbed_state == TestbedState::Interactive {
                                            stdout.write_all(b"\x08 \x08").await?;
                                            stdout.flush().await?;
                                            t.suppressor.pop_last();
                                            mc.container.write_stdin(&[byte]).await?;
                                            t.at_line_start = false;
                                        } else {
                                            mc.container.write_stdin(&[byte]).await?;
                                            t.at_line_start = false;
                                        }
                                    }

                                    // Printable ASCII
                                    0x20..=0x7e => {
                                        if t.at_line_start && byte == b':' {
                                            t.in_meta = true;
                                            t.meta_buf.push(byte);
                                            stdout.write_all(&[byte]).await?;
                                            stdout.flush().await?;
                                        } else if t.in_meta {
                                            t.meta_buf.push(byte);
                                            stdout.write_all(&[byte]).await?;
                                            stdout.flush().await?;
                                        } else if t.testbed_state == TestbedState::Interactive {
                                            stdout.write_all(&[byte]).await?;
                                            stdout.flush().await?;
                                            t.suppressor.push(byte);
                                            mc.container.write_stdin(&[byte]).await?;
                                            t.at_line_start = false;
                                        } else {
                                            mc.container.write_stdin(&[byte]).await?;
                                            t.at_line_start = false;
                                        }
                                    }

                                    // Other bytes (escape sequences, control chars)
                                    _ => {
                                        mc.container.write_stdin(&[byte]).await?;
                                        t.at_line_start = false;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[testbed] stdin read error: {e}");
                            should_exit = true;
                        }
                    }
                }
            }
        } else {
            // === Testbed prompt mode (no active container) ===
            tokio::select! {
                Some(req) = approval_rx.recv() => {
                    let label_part = req.source_label
                        .as_ref()
                        .map(|l| format!(" [{l}]"))
                        .unwrap_or_default();
                    let prompt_msg = format!(
                        "\r\n[firewall]{label_part} {} \u{2014} allow? (y/n) ",
                        req.request
                    );
                    stdout.write_all(prompt_msg.as_bytes()).await?;
                    stdout.flush().await?;
                    input_mode = InputMode::AwaitingApproval(req.response_tx);
                }

                result = stdin.read(&mut stdin_buf) => {
                    match result {
                        Ok(0) => break,
                        Ok(n) => {
                            for &byte in &stdin_buf[..n] {
                                // Firewall verdict handling
                                if let InputMode::AwaitingApproval(_) = &input_mode {
                                    let verdict = match byte {
                                        b'y' | b'Y' => {
                                            stdout.write_all(b"y\r\n").await?;
                                            stdout.flush().await?;
                                            Some(ConnectionVerdict::Allow)
                                        }
                                        b'n' | b'N' => {
                                            stdout.write_all(b"n\r\n").await?;
                                            stdout.flush().await?;
                                            Some(ConnectionVerdict::Block)
                                        }
                                        _ => None,
                                    };
                                    if let Some(v) = verdict {
                                        let mode = std::mem::replace(
                                            &mut input_mode,
                                            InputMode::Normal,
                                        );
                                        if let InputMode::AwaitingApproval(tx) = mode {
                                            let _ = tx.send(v);
                                        }
                                        stdout.write_all(b"[testbed] > ").await?;
                                        stdout.flush().await?;
                                    }
                                    continue;
                                }

                                match byte {
                                    b'\r' | b'\n' => {
                                        stdout.write_all(b"\r\n").await?;
                                        stdout.flush().await?;

                                        if !testbed_line_buf.is_empty() {
                                            let line = String::from_utf8_lossy(&testbed_line_buf).to_string();
                                            testbed_line_buf.clear();

                                            if let Some(cmd) = parse_meta_command(&line) {
                                                handle_meta_command(
                                                    cmd,
                                                    &mut manager,
                                                    &mut stdout,
                                                )
                                                .await?;

                                                if manager.has_active() {
                                                    show_prompt_if_interactive(&mut manager, &mut stdout).await?;
                                                    break;
                                                }
                                            } else {
                                                let msg = format!(
                                                    "Unknown command: {line}. Type :help for commands.\r\n"
                                                );
                                                stdout.write_all(msg.as_bytes()).await?;
                                            }
                                        }

                                        if !manager.has_active() {
                                            stdout.write_all(b"[testbed] > ").await?;
                                            stdout.flush().await?;
                                        }
                                    }

                                    // Ctrl+C — exit testbed
                                    0x03 => {
                                        testbed_line_buf.clear();
                                        stdout.write_all(b"^C\r\n").await?;
                                        stdout.flush().await?;
                                        should_exit = true;
                                        break;
                                    }

                                    // Ctrl+\ — force quit
                                    0x1c => {
                                        should_exit = true;
                                        break;
                                    }

                                    // Backspace
                                    0x7f | 0x08 => {
                                        if testbed_line_buf.pop().is_some() {
                                            stdout.write_all(b"\x08 \x08").await?;
                                            stdout.flush().await?;
                                        }
                                    }

                                    // Printable
                                    0x20..=0x7e => {
                                        testbed_line_buf.push(byte);
                                        stdout.write_all(&[byte]).await?;
                                        stdout.flush().await?;
                                    }

                                    _ => {}
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[testbed] stdin read error: {e}");
                            should_exit = true;
                        }
                    }
                }
            }
        }

        if should_exit {
            break;
        }
    }

    manager.kill_all().await;
    drop(_raw);
    std::process::exit(0);
}

// ---------------------------------------------------------------------------
// Meta-command handler
// ---------------------------------------------------------------------------

async fn handle_meta_command(
    cmd: MetaCommand<'_>,
    manager: &mut ContainerManager,
    stdout: &mut tokio::io::Stdout,
) -> anyhow::Result<()> {
    match cmd {
        MetaCommand::From { image, name } => {
            let image = image.to_string();
            let name_owned = name.map(|n| n.to_string());
            if let Err(e) = manager.create(&image, name_owned.as_deref()).await {
                eprintln!("[testbed] Failed to start container '{image}': {e}");
            }
        }
        MetaCommand::Name(name) => {
            if !manager.name_active(name).await {
                eprintln!("[testbed] No active container to name");
            }
        }
        MetaCommand::Switch(name) => {
            if !manager.switch_to_name(name) {
                eprintln!("[testbed] No container named '{name}'");
            }
        }
        MetaCommand::List => {
            let infos = manager.list();
            if infos.is_empty() {
                stdout.write_all(b"  (no containers)\r\n").await?;
            } else {
                let header = format!(
                    "  {:>3}  {:<10} {:<10} {:<14} {}\r\n",
                    "ID", "NAME", "IMAGE", "IP", "STATUS"
                );
                stdout.write_all(header.as_bytes()).await?;
                for info in &infos {
                    let line = format!("{info}\r\n");
                    stdout.write_all(line.as_bytes()).await?;
                }
            }
            stdout.flush().await?;
        }
        MetaCommand::Kill | MetaCommand::Exit => {
            manager.kill_active().await;
        }
        MetaCommand::Detach => {
            manager.detach();
        }
        MetaCommand::Help => {
            stdout.write_all(HELP_TEXT.as_bytes()).await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}
