use std::collections::VecDeque;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

use container::{Container, ContainerConfig, create_async_engine};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wasmtime::{Engine, Module};

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
            // Raw input: no echo, no line buffering, no signal keys, no extensions
            raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
            // No CR→NL translation, no XON/XOFF flow control
            raw.c_iflag &= !(libc::ICRNL | libc::IXON);
            // Return after 1 byte, no timeout
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
    From(&'a str),
    Exit,
}

fn parse_meta_command(line: &str) -> Option<MetaCommand<'_>> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix(":from ") {
        let name = rest.trim();
        if !name.is_empty() {
            return Some(MetaCommand::From(name));
        }
    }
    if trimmed == ":exit" {
        return Some(MetaCommand::Exit);
    }
    None
}

// ---------------------------------------------------------------------------
// Image path resolution
// ---------------------------------------------------------------------------

fn resolve_image_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let images_dir = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("c2w/converted_images");

    // 1. Check for exact file: {name}.wasm
    let exact = images_dir.join(format!("{name}.wasm"));
    if exact.is_file() {
        return exact;
    }

    // 2. Check for directory: {name}/, pick latest .wasm by lexicographic sort
    let dir = images_dir.join(name);
    if dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&dir)
    {
        let mut wasm_files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "wasm"))
            .collect();
        wasm_files.sort();
        if let Some(latest) = wasm_files.last() {
            return latest.clone();
        }
    }

    // 3. Fallback: return path as-is (wasmtime gives clear "file not found" error)
    PathBuf::from(name)
}

// ---------------------------------------------------------------------------
// Container stack management
// ---------------------------------------------------------------------------

async fn push_container(
    stack: &mut Vec<(String, Container)>,
    engine: &Engine,
    name: &str,
    networking: bool,
) -> anyhow::Result<()> {
    let wasm_path = resolve_image_path(name);
    eprintln!("[testbed] Loading image: {}", wasm_path.display());

    let module = Module::from_file(engine, &wasm_path)?;
    let mut config = ContainerConfig::new(engine.clone(), module);
    config.networking = networking;
    let container = Container::start(config).await?;

    let net_status = if networking { " (networking enabled)" } else { "" };
    eprintln!("[testbed] Started container: {name}{net_status}");
    stack.push((name.to_string(), container));
    Ok(())
}

fn pop_container(stack: &mut Vec<(String, Container)>) -> bool {
    if let Some((name, container)) = stack.pop() {
        container.kill();
        eprintln!("[testbed] Exited container: {name}");
        if let Some((current_name, _)) = stack.last() {
            eprintln!("[testbed] Returning to container: {current_name}");
        }
    }
    !stack.is_empty()
}

// ---------------------------------------------------------------------------
// ANSI terminal query filtering
// ---------------------------------------------------------------------------

/// Remove Device Status Report queries (ESC[6n) from container output so the
/// host terminal doesn't respond with Cursor Position Reports that leak back
/// into the container as garbage input.
fn strip_dsr(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        if i + 3 <= src.len()
            && src[i] == 0x1b
            && src[i + 1] == b'['
            && src[i + 2] == b'6'
            && i + 4 <= src.len()
            && src[i + 3] == b'n'
        {
            // Skip ESC[6n
            i += 4;
        } else {
            out.push(src[i]);
            i += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Echo suppressor
// ---------------------------------------------------------------------------

/// Suppresses container echo of bytes we already displayed locally.
///
/// When we locally echo a typed character, we push it into the queue. When the
/// container later sends back that same byte, we pop it from the queue instead
/// of printing it (it was already shown). On mismatch the queue is cleared and
/// the byte passes through — the container is sending something unexpected.
///
/// ANSI escape sequences from the container (starting with ESC `[`) are always
/// passed through without attempting to match them against the queue.
struct EchoSuppressor {
    queue: VecDeque<u8>,
}

impl EchoSuppressor {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn push(&mut self, byte: u8) {
        self.queue.push_back(byte);
    }

    fn pop_last(&mut self) {
        self.queue.pop_back();
    }

    /// Process a slice of container output bytes. Returns only the bytes that
    /// should be printed (i.e. not suppressed).
    fn process(&mut self, src: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(src.len());
        let mut i = 0;
        while i < src.len() {
            // Pass ANSI escape sequences through without matching
            if src[i] == 0x1b && i + 1 < src.len() && src[i + 1] == b'[' {
                out.push(src[i]);
                i += 1;
                // Copy the rest of the escape sequence (up to the final byte)
                while i < src.len() {
                    out.push(src[i]);
                    if src[i].is_ascii_alphabetic() || src[i] == b'~' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            if let Some(&expected) = self.queue.front() {
                if src[i] == expected {
                    // Match — suppress this byte
                    self.queue.pop_front();
                } else {
                    // Mismatch — flush: the container is sending something
                    // different from what we echoed, so stop suppressing.
                    self.queue.clear();
                    out.push(src[i]);
                }
            } else {
                // Queue empty — nothing to suppress
                out.push(src[i]);
            }
            i += 1;
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Prompt matcher
// ---------------------------------------------------------------------------

const PROMPT: &[u8] = b"/ # ";

/// Streaming pattern matcher for the BusyBox ash prompt `/ # `.
///
/// Buffers bytes that partially match the prompt pattern. When the full pattern
/// is matched, the buffered bytes are discarded (suppressed) and the caller is
/// notified via the returned flag. On mismatch, buffered partial-match bytes
/// are flushed to output.
struct PromptMatcher {
    matched: usize,
    buffer: Vec<u8>,
}

impl PromptMatcher {
    fn new() -> Self {
        Self {
            matched: 0,
            buffer: Vec::new(),
        }
    }

    /// Process container output. Returns `(bytes_to_print, prompt_detected)`.
    fn process(&mut self, src: &[u8]) -> (Vec<u8>, bool) {
        let mut out = Vec::with_capacity(src.len());
        let mut detected = false;

        for &byte in src {
            if byte == PROMPT[self.matched] {
                self.buffer.push(byte);
                self.matched += 1;
                if self.matched == PROMPT.len() {
                    // Full prompt matched — suppress the buffered bytes
                    self.buffer.clear();
                    self.matched = 0;
                    detected = true;
                }
            } else {
                // Mismatch — flush buffered partial match
                if !self.buffer.is_empty() {
                    out.extend_from_slice(&self.buffer);
                    self.buffer.clear();
                }
                self.matched = 0;
                // Check if the current byte starts a new match
                if byte == PROMPT[0] {
                    self.buffer.push(byte);
                    self.matched = 1;
                } else {
                    out.push(byte);
                }
            }
        }

        (out, detected)
    }
}

// ---------------------------------------------------------------------------
// Testbed state
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
enum TestbedState {
    /// Kernel is booting — forward container output, detect first prompt.
    Booting,
    /// Shell is interactive — local echo, suppress container echo.
    Interactive,
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _raw = RawModeGuard::enable();

    let engine = create_async_engine()?;

    // Enable networking by default for all containers
    let networking = true;

    let mut stack: Vec<(String, Container)> = Vec::new();
    push_container(&mut stack, &engine, "alpine", networking).await?;

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut stdin_buf = [0u8; 256];
    let mut stdout_buf = [0u8; 4096];

    // Meta-command state: lines starting with `:` are buffered locally and
    // echoed by the testbed rather than forwarded to the container.
    let mut meta_buf: Vec<u8> = Vec::new();
    let mut in_meta = false;
    let mut at_line_start = true;
    let mut should_exit = false;

    // Local echo + suppression state
    let mut state = TestbedState::Booting;
    let mut suppressor = EchoSuppressor::new();
    let mut prompt_matcher = PromptMatcher::new();

    loop {
        let (_, container) = stack.last_mut().expect("stack should not be empty in loop");

        tokio::select! {
            // Container stdout → filter pipeline → user terminal
            result = container.read_stdout(&mut stdout_buf) => {
                match result {
                    Ok(0) => {
                        if !pop_container(&mut stack) {
                            break;
                        }
                    }
                    Ok(n) => {
                        // Pipeline: strip DSR → echo suppressor → prompt matcher → print
                        let after_dsr = strip_dsr(&stdout_buf[..n]);

                        let after_suppress = if state == TestbedState::Interactive {
                            suppressor.process(&after_dsr)
                        } else {
                            after_dsr
                        };

                        let (to_print, prompt_detected) = prompt_matcher.process(&after_suppress);

                        if !to_print.is_empty() {
                            stdout.write_all(&to_print).await?;
                        }

                        if prompt_detected {
                            if state == TestbedState::Booting {
                                state = TestbedState::Interactive;
                            }
                            // Print our own prompt
                            stdout.write_all(PROMPT).await?;
                            at_line_start = true;
                        }

                        if !to_print.is_empty() || prompt_detected {
                            stdout.flush().await?;
                        }
                    }
                    Err(e) => {
                        eprintln!("[testbed] stdout read error: {e}");
                        if !pop_container(&mut stack) {
                            break;
                        }
                    }
                }
            }

            // User input → raw bytes, forwarded or buffered as meta-command
            result = stdin.read(&mut stdin_buf) => {
                match result {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        for &byte in &stdin_buf[..n] {
                            match byte {
                                // Enter (CR in raw mode since ICRNL is off)
                                b'\r' | b'\n' => {
                                    if in_meta {
                                        let line = String::from_utf8_lossy(&meta_buf).to_string();
                                        // Echo newline locally for the meta-command line
                                        stdout.write_all(b"\r\n").await?;
                                        stdout.flush().await?;

                                        if let Some(cmd) = parse_meta_command(&line) {
                                            match cmd {
                                                MetaCommand::From(name) => {
                                                    let name = name.to_string();
                                                    if let Err(e) = push_container(&mut stack, &engine, &name, networking).await {
                                                        eprintln!("[testbed] Failed to start container '{name}': {e}");
                                                    }
                                                }
                                                MetaCommand::Exit => {
                                                    if !pop_container(&mut stack) {
                                                        should_exit = true;
                                                        break;
                                                    }
                                                }
                                            }
                                        } else {
                                            // Not a valid meta-command — forward to container
                                            // and add to suppress queue (already echoed locally)
                                            if let Some((_, c)) = stack.last_mut() {
                                                if state == TestbedState::Interactive {
                                                    for &b in &meta_buf {
                                                        suppressor.push(b);
                                                    }
                                                    suppressor.push(b'\r');
                                                    suppressor.push(b'\n');
                                                }
                                                c.write_stdin(&meta_buf).await?;
                                                c.write_stdin(b"\n").await?;
                                            }
                                        }
                                        meta_buf.clear();
                                        in_meta = false;
                                    } else if state == TestbedState::Interactive {
                                        // Echo locally and suppress container echo
                                        stdout.write_all(b"\r\n").await?;
                                        stdout.flush().await?;
                                        suppressor.push(b'\r');
                                        suppressor.push(b'\n');
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(b"\n").await?;
                                        }
                                    } else {
                                        // Booting — just forward
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(b"\n").await?;
                                        }
                                    }
                                    at_line_start = true;
                                }

                                // Ctrl+C — exit current container (or quit if last)
                                0x03 => {
                                    if in_meta {
                                        meta_buf.clear();
                                        in_meta = false;
                                        at_line_start = true;
                                    }
                                    stdout.write_all(b"^C\r\n").await?;
                                    stdout.flush().await?;
                                    if !pop_container(&mut stack) {
                                        should_exit = true;
                                        break;
                                    }
                                }

                                // Ctrl+\ — force quit testbed
                                0x1c => {
                                    should_exit = true;
                                    break;
                                }

                                // Backspace / Delete
                                0x7f | 0x08 => {
                                    if in_meta {
                                        if meta_buf.pop().is_some() {
                                            if meta_buf.is_empty() {
                                                in_meta = false;
                                                at_line_start = true;
                                            }
                                            stdout.write_all(b"\x08 \x08").await?;
                                            stdout.flush().await?;
                                        }
                                    } else if state == TestbedState::Interactive {
                                        // Echo backspace locally and suppress container echo
                                        stdout.write_all(b"\x08 \x08").await?;
                                        stdout.flush().await?;
                                        suppressor.pop_last();
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(&[byte]).await?;
                                        }
                                        at_line_start = false;
                                    } else {
                                        // Booting — just forward
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(&[byte]).await?;
                                        }
                                        at_line_start = false;
                                    }
                                }

                                // Printable ASCII
                                0x20..=0x7e => {
                                    if at_line_start && byte == b':' {
                                        // Start of a potential meta-command — buffer locally
                                        in_meta = true;
                                        meta_buf.push(byte);
                                        stdout.write_all(&[byte]).await?;
                                        stdout.flush().await?;
                                    } else if in_meta {
                                        meta_buf.push(byte);
                                        stdout.write_all(&[byte]).await?;
                                        stdout.flush().await?;
                                    } else if state == TestbedState::Interactive {
                                        // Echo locally and suppress container echo
                                        stdout.write_all(&[byte]).await?;
                                        stdout.flush().await?;
                                        suppressor.push(byte);
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(&[byte]).await?;
                                        }
                                        at_line_start = false;
                                    } else {
                                        // Booting — just forward, no echo
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(&[byte]).await?;
                                        }
                                        at_line_start = false;
                                    }
                                }

                                // Other bytes (escape sequences, control chars)
                                // Forward to container without local echo or suppress queue
                                _ => {
                                    if let Some((_, c)) = stack.last_mut() {
                                        c.write_stdin(&[byte]).await?;
                                    }
                                    at_line_start = false;
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

        if should_exit {
            break;
        }
    }

    // Clean up remaining containers
    while let Some((name, container)) = stack.pop() {
        container.kill();
        eprintln!("[testbed] Killed container: {name}");
    }

    // Restore the terminal now rather than relying on implicit drop, so we
    // can force-exit below without leaving the terminal in raw mode.
    drop(_raw);

    // A blocking stdin read on the tokio thread-pool cannot be cancelled,
    // which prevents the runtime from shutting down.  Force-exit instead.
    std::process::exit(0);
}
