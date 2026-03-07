use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use container::{create_async_engine, Container};
use wasmtime::{Engine, Module};

// ---------------------------------------------------------------------------
// A. Fixture paths
// ---------------------------------------------------------------------------

/// Directory containing converted alpine images.
pub fn fixture_dir() -> PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest).join("../c2w/converted_images/alpine")
}

/// Path to the plain (non-wizer'd) alpine WASM fixture.
///
/// Automatically extracts the `.zip` archive if the `.wasm` file doesn't exist.
pub fn alpine_wasm_path() -> Option<PathBuf> {
    let dir = fixture_dir();
    let wasm = dir.join("3.23.2.wasm");
    if wasm.exists() {
        return Some(wasm);
    }

    let zip = dir.join("3.23.2.wasm.zip");
    if !zip.exists() {
        return None;
    }

    eprintln!("bench: extracting {} ...", zip.display());
    let status = Command::new("unzip")
        .arg("-o")
        .arg("-q")
        .arg(&zip)
        .arg("-d")
        .arg(&dir)
        .status();

    match status {
        Ok(s) if s.success() && wasm.exists() => Some(wasm),
        Ok(s) => {
            eprintln!("bench: unzip exited with {s}");
            None
        }
        Err(e) => {
            eprintln!("bench: failed to run unzip: {e}");
            None
        }
    }
}

/// Path to the wizer-pre-initialized alpine WASM fixture (may not exist).
pub fn alpine_wizer_wasm_path() -> Option<PathBuf> {
    let p = fixture_dir().join("3.23.2-wizer.wasm");
    p.exists().then_some(p)
}

// ---------------------------------------------------------------------------
// B. Prerequisite checks
// ---------------------------------------------------------------------------

/// Checks for external tools needed by conversion benchmarks.
pub struct ConversionPrereqs {
    pub has_iso_tool: bool,
    pub has_wasi_vfs: bool,
    pub has_wizer: bool,
    pub has_template: bool,
}

impl ConversionPrereqs {
    pub fn check() -> Self {
        let has_iso_tool = which::which("mkisofs").is_ok()
            || which::which("genisoimage").is_ok()
            || which::which("xorriso").is_ok();
        let has_wasi_vfs = which::which("wasi-vfs").is_ok();
        let has_wizer = which::which("wizer").is_ok();

        let template_dir = dirs_template_path();
        let has_template = template_dir.exists();

        Self {
            has_iso_tool,
            has_wasi_vfs,
            has_wizer,
            has_template,
        }
    }

    /// Can run the full no-docker conversion (without wizer).
    pub fn can_convert(&self) -> bool {
        self.has_iso_tool && self.has_wasi_vfs && self.has_template
    }

    /// Can run the full no-docker conversion including wizer.
    pub fn can_convert_with_wizer(&self) -> bool {
        self.can_convert() && self.has_wizer
    }

    /// Can run the wizer-only step on an existing WASM.
    pub fn can_wizer_only(&self) -> bool {
        self.has_wizer
    }

    /// Human-readable summary of missing prerequisites.
    pub fn summary(&self) -> String {
        let mut missing = Vec::new();
        if !self.has_iso_tool {
            missing.push("mkisofs/genisoimage/xorriso");
        }
        if !self.has_wasi_vfs {
            missing.push("wasi-vfs");
        }
        if !self.has_wizer {
            missing.push("wizer");
        }
        if !self.has_template {
            missing.push("template dir (~/.cache/c2w/templates/amd64-v0.8.3/)");
        }
        if missing.is_empty() {
            "all prerequisites met".to_string()
        } else {
            format!("missing: {}", missing.join(", "))
        }
    }
}

fn dirs_template_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("c2w/templates/amd64-v0.8.3")
}

// ---------------------------------------------------------------------------
// C. Engine / module caching
// ---------------------------------------------------------------------------

/// Returns a shared wasmtime `Engine` (created once, reused across benchmarks).
pub fn shared_engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| create_async_engine().expect("failed to create async engine"))
}

/// Compile a WASM module from the given path. Returns `None` if the path
/// doesn't exist or compilation fails.
pub fn load_module(path: &Path) -> Option<Module> {
    if !path.exists() {
        return None;
    }
    match Module::from_file(shared_engine(), path) {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("bench: failed to compile {}: {e}", path.display());
            None
        }
    }
}

// ---------------------------------------------------------------------------
// D. Async container helpers
// ---------------------------------------------------------------------------

/// Read stdout until a shell prompt (`/ # ` or `$ `) appears, or timeout.
pub async fn wait_for_prompt(
    container: &mut Container,
    timeout: Duration,
) -> anyhow::Result<String> {
    let mut output = String::new();
    let mut buf = [0u8; 4096];

    let result = tokio::time::timeout(timeout, async {
        loop {
            let n = container.read_stdout(&mut buf).await?;
            if n == 0 {
                break;
            }
            let chunk = String::from_utf8_lossy(&buf[..n]);
            output.push_str(&chunk);
            if output.contains("/ # ") || output.ends_with("$ ") {
                break;
            }
        }
        Ok::<_, anyhow::Error>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(output),
        Ok(Err(e)) => Err(e),
        Err(_) => anyhow::bail!(
            "timed out waiting for prompt after {}s. Output so far:\n{}",
            timeout.as_secs(),
            output
        ),
    }
}

/// Send a shell command, then wait for the next prompt.
pub async fn run_command(container: &mut Container, cmd: &str) -> anyhow::Result<String> {
    container.write_line(cmd).await?;
    wait_for_prompt(container, Duration::from_secs(30)).await
}
