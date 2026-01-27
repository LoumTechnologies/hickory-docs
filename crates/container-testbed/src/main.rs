use std::path::{Path, PathBuf};

use container::{Container, ContainerConfig, create_async_engine};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use wasmtime::{Engine, Module};

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
// Main loop
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = create_async_engine()?;

    let mut stack: Vec<(String, Container)> = Vec::new();
    push_container(&mut stack, &engine, "alpine").await?;

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    let mut stdout_buf = [0u8; 4096];

    loop {
        let (_, container) = stack.last_mut().expect("stack should not be empty in loop");

        tokio::select! {
            // Container stdout → user terminal
            result = container.read_stdout(&mut stdout_buf) => {
                match result {
                    Ok(0) => {
                        // Container EOF
                        if !pop_container(&mut stack) {
                            break;
                        }
                    }
                    Ok(n) => {
                        let chunk = &stdout_buf[..n];
                        tokio::io::stdout().write_all(chunk).await?;
                        tokio::io::stdout().flush().await?;
                    }
                    Err(e) => {
                        eprintln!("[testbed] stdout read error: {e}");
                        if !pop_container(&mut stack) {
                            break;
                        }
                    }
                }
            }

            // User input → parse meta-commands or forward to container stdin
            line = lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        if let Some(cmd) = parse_meta_command(&line) {
                            match cmd {
                                MetaCommand::From(name) => {
                                    if let Err(e) = push_container(&mut stack, &engine, name).await {
                                        eprintln!("[testbed] Failed to start container '{name}': {e}");
                                    }
                                }
                                MetaCommand::Exit => {
                                    if !pop_container(&mut stack) {
                                        break;
                                    }
                                }
                            }
                        } else {
                            let (_, container) = stack.last_mut().unwrap();
                            container.write_line(&line).await?;
                        }
                    }
                    Ok(None) => {
                        // EOF on stdin
                        break;
                    }
                    Err(e) => {
                        eprintln!("[testbed] stdin read error: {e}");
                        break;
                    }
                }
            }
        }
    }

    // Clean up remaining containers
    while let Some((name, container)) = stack.pop() {
        container.kill();
        eprintln!("[testbed] Killed container: {name}");
    }

    Ok(())
}
