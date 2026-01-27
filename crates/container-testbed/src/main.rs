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
        .join("container2wasm/converted_images");

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
) -> anyhow::Result<()> {
    let wasm_path = resolve_image_path(name);
    eprintln!("[testbed] Loading image: {}", wasm_path.display());

    let module = Module::from_file(engine, &wasm_path)?;
    let config = ContainerConfig::new(engine.clone(), module);
    let container = Container::start(config).await?;

    eprintln!("[testbed] Started container: {name}");
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
        if i + 3 <= src.len() && src[i] == 0x1b && src[i + 1] == b'[' && src[i + 2] == b'6'
            && i + 4 <= src.len() && src[i + 3] == b'n'
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
// Main loop
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _raw = RawModeGuard::enable();

    let engine = create_async_engine()?;

    let mut stack: Vec<(String, Container)> = Vec::new();
    push_container(&mut stack, &engine, "alpine").await?;

    let mut stdin = tokio::io::stdin();
    let mut stdin_buf = [0u8; 256];
    let mut stdout_buf = [0u8; 4096];

    // Meta-command state: lines starting with `:` are buffered locally and
    // echoed by the testbed rather than forwarded to the container.
    let mut meta_buf: Vec<u8> = Vec::new();
    let mut in_meta = false;
    let mut at_line_start = true;
    let mut should_exit = false;

    loop {
        let (_, container) = stack.last_mut().expect("stack should not be empty in loop");

        tokio::select! {
            // Container stdout → user terminal
            result = container.read_stdout(&mut stdout_buf) => {
                match result {
                    Ok(0) => {
                        if !pop_container(&mut stack) {
                            break;
                        }
                    }
                    Ok(n) => {
                        let filtered = strip_dsr(&stdout_buf[..n]);
                        if !filtered.is_empty() {
                            tokio::io::stdout().write_all(&filtered).await?;
                            tokio::io::stdout().flush().await?;
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
                                        tokio::io::stdout().write_all(b"\r\n").await?;
                                        tokio::io::stdout().flush().await?;

                                        if let Some(cmd) = parse_meta_command(&line) {
                                            match cmd {
                                                MetaCommand::From(name) => {
                                                    let name = name.to_string();
                                                    if let Err(e) = push_container(&mut stack, &engine, &name).await {
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
                                            if let Some((_, c)) = stack.last_mut() {
                                                c.write_stdin(&meta_buf).await?;
                                                c.write_stdin(b"\n").await?;
                                            }
                                        }
                                        meta_buf.clear();
                                        in_meta = false;
                                    } else {
                                        // Forward newline to container
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(b"\n").await?;
                                        }
                                    }
                                    at_line_start = true;
                                }

                                // Ctrl+C
                                0x03 => {
                                    if in_meta {
                                        meta_buf.clear();
                                        in_meta = false;
                                        at_line_start = true;
                                        tokio::io::stdout().write_all(b"^C\r\n").await?;
                                        tokio::io::stdout().flush().await?;
                                    } else {
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(&[0x03]).await?;
                                        }
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
                                            tokio::io::stdout().write_all(b"\x08 \x08").await?;
                                            tokio::io::stdout().flush().await?;
                                        }
                                    } else {
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(&[byte]).await?;
                                        }
                                        at_line_start = false;
                                    }
                                }

                                // Regular byte
                                _ => {
                                    if at_line_start && byte == b':' {
                                        // Start of a potential meta-command — buffer locally
                                        in_meta = true;
                                        meta_buf.push(byte);
                                        tokio::io::stdout().write_all(&[byte]).await?;
                                        tokio::io::stdout().flush().await?;
                                    } else if in_meta {
                                        meta_buf.push(byte);
                                        tokio::io::stdout().write_all(&[byte]).await?;
                                        tokio::io::stdout().flush().await?;
                                    } else {
                                        // Forward to container — it handles echoing
                                        if let Some((_, c)) = stack.last_mut() {
                                            c.write_stdin(&[byte]).await?;
                                        }
                                        at_line_start = false;
                                    }
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

    Ok(())
}
