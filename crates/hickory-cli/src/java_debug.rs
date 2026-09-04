//! Starting a Java debug session, which means starting a language server.
//!
//! # Why this exists at all
//!
//! Every other adapter here is a program hick finds and spawns. `java-debug`
//! is not a program: it is an Eclipse plugin that runs **inside**
//! eclipse.jdt.ls, and the way a client gets a debug session out of it is to
//! send the language server an ordinary `workspace/executeCommand` named
//! `vscode.java.startDebugSession` and read a **port number** out of the
//! reply. From there it is ordinary DAP over TCP, which `hick-dap` already
//! speaks.
//!
//! So this module is the seam `docs/specs/freeform/debugging-the-jvm.md`
//! describes: `hick-dap` must not depend on `hick-lsp` — that inverts the
//! layering and would let every debug session in every language drag a
//! language server behind it — so the crate that owns both ends does the
//! asking, and hands `hick-dap` a port.
//!
//! # Why a dedicated server rather than the editor's
//!
//! A document's Java is woven into a **scratch directory**, and jdt.ls is
//! rooted at one place for its lifetime. The editor's session is rooted at
//! the folder the app opened, which is not where the document's code is. So
//! a debug session gets its own short-lived server, rooted where the code
//! actually is.
//!
//! This is the expensive part and it is not hidden: jdt.ls takes tens of
//! seconds to import a project the first time, and every step of it is
//! reported through `on_output` into the same terminal a build is watched in.
//!
//! # What was measured, on 2026-09-04
//!
//! A directory holding one `Main.java` and **no build file at all** is enough:
//! jdt.ls creates an "invisible project", compiles it with its own bundled
//! compiler (no `javac` on the machine is required), and
//! `vscode.java.startDebugSession` answers with a port. A breakpoint set over
//! that port came back `verified: true` and the program stopped on it. That
//! was the open question in the design, and it is the reason a document does
//! not have to generate a `pom.xml`.

use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use hick_dap::BuildOutput;
use hick_dap::java::installed;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::sync::{Mutex, mpsc};

/// How long to wait for jdt.ls to report itself ready.
///
/// Generous on purpose: this is a JVM starting, an OSGi framework coming up,
/// and a project being imported. A machine that has never run it before is
/// slower again.
const READY_TIMEOUT: Duration = Duration::from_secs(180);

/// How long any one command may take once the server is ready.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(120);

/// What a debug session needs, once the language server has answered.
pub struct Prepared {
    /// The port java-debug is listening on.
    pub port: u16,
    /// The launch arguments java-debug requires: it does not take a
    /// `program`, it takes a main class and a classpath.
    pub launch: Value,
    /// The server, held for the life of the session. Dropping it kills
    /// jdt.ls, and java-debug lives inside jdt.ls.
    server: Child,
}

impl Prepared {
    /// End the language server, and the debug adapter inside it.
    pub async fn shutdown(&mut self) {
        let _ = self.server.kill().await;
    }
}

/// A minimal LSP client: enough to initialize, open a file, hear when the
/// server is ready, and execute three commands.
///
/// Deliberately not `hick_lsp::ChildLspHandle`. That handle serves the
/// EDITOR's session — one per workspace, rooted at the folder the app
/// opened, living as long as the window. This one is rooted at a scratch
/// directory and dies with the debug session, and entangling the two
/// lifetimes would mean a debug session could end the editor's intelligence.
struct Lsp {
    stdin: Mutex<ChildStdin>,
    next_id: AtomicI64,
    replies: Arc<Mutex<std::collections::HashMap<i64, Value>>>,
    status: Mutex<mpsc::UnboundedReceiver<String>>,
}

impl Lsp {
    fn start(child: &mut Child) -> Result<Self> {
        let stdin = child.stdin.take().context("jdt.ls stdin")?;
        let stdout = child.stdout.take().context("jdt.ls stdout")?;
        let replies: Arc<Mutex<std::collections::HashMap<i64, Value>>> = Default::default();
        let (status_tx, status_rx) = mpsc::unbounded_channel();
        let reader_replies = replies.clone();
        tokio::spawn(async move {
            let mut stream = BufReader::new(stdout);
            while let Some(message) = read_message(&mut stream).await {
                if let Some(id) = message.get("id").and_then(Value::as_i64)
                    && message.get("method").is_none()
                {
                    reader_replies.lock().await.insert(id, message);
                } else if message.get("method").and_then(Value::as_str) == Some("language/status")
                    && let Some(kind) = message
                        .get("params")
                        .and_then(|p| p.get("type"))
                        .and_then(Value::as_str)
                {
                    let _ = status_tx.send(kind.to_string());
                }
            }
        });
        Ok(Self {
            stdin: Mutex::new(stdin),
            next_id: AtomicI64::new(0),
            replies,
            status: Mutex::new(status_rx),
        })
    }

    async fn send(&self, message: &Value) -> Result<()> {
        let body = serde_json::to_vec(message)?;
        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
            .await?;
        stdin.write_all(&body).await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn notify(&self, method: &str, params: Value) -> Result<()> {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
            .await
    }

    async fn request(&self, method: &str, params: Value, wait: Duration) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await?;
        let deadline = tokio::time::Instant::now() + wait;
        loop {
            if let Some(reply) = self.replies.lock().await.remove(&id) {
                if let Some(error) = reply.get("error") {
                    bail!("the Java language server refused `{method}`: {error}");
                }
                return Ok(reply.get("result").cloned().unwrap_or(Value::Null));
            }
            if tokio::time::Instant::now() >= deadline {
                bail!("the Java language server did not answer `{method}` in time");
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Wait for jdt.ls to say it has finished starting AND importing.
    ///
    /// `ServiceReady` is the one that matters: `Started` arrives before the
    /// project is imported, and a command sent then answers about a project
    /// that does not exist yet.
    async fn wait_ready(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        let deadline = tokio::time::Instant::now() + READY_TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            if left.is_zero() {
                bail!(
                    "the Java language server never reported itself ready within {}s",
                    READY_TIMEOUT.as_secs()
                );
            }
            match tokio::time::timeout(left, status.recv()).await {
                Ok(Some(kind)) if kind == "ServiceReady" => return Ok(()),
                Ok(Some(_)) => continue,
                Ok(None) => bail!("the Java language server ended while starting"),
                Err(_) => continue,
            }
        }
    }
}

async fn read_message<R>(stream: &mut BufReader<R>) -> Option<Value>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut length = None;
    loop {
        let mut line = String::new();
        if stream.read_line(&mut line).await.ok()? == 0 {
            return None;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let mut body = vec![0u8; length?];
    stream.read_exact(&mut body).await.ok()?;
    serde_json::from_slice(&body).ok()
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

/// Start jdt.ls on `project_dir`, and ask it for a debug session for
/// `source`.
///
/// `cache_root` is the real project — where the installed server lives and
/// where jdt.ls's own workspace is kept, so that an import survives a
/// session and the second run is fast.
pub async fn prepare(
    source: &Path,
    project_dir: &Path,
    cache_root: &Path,
    on_output: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<Prepared> {
    let found = installed(cache_root).with_context(|| {
        "Java needs the language server and the debug plugin, and one of them is missing.\n  \
         Next step: run `hick lsp install java` and `hick dap install java`. The debugger runs \
         as a plugin INSIDE the language server, so both are needed — this is the one language \
         here where that is true."
    })?;

    // jdt.ls keeps its own workspace: compiled classes, indexes, and the
    // invisible project it makes for a directory with no build file. Kept
    // per project directory and OUTSIDE the scratch tree, so the second
    // debug session does not pay for the import again.
    let workspace = cache_root
        .join(".hick-cache/jdtls")
        .join(format!("ws-{:x}", seahash(&project_dir.to_string_lossy())));
    std::fs::create_dir_all(&workspace)
        .with_context(|| format!("creating {}", workspace.display()))?;

    let argv = vec![
        "java".to_string(),
        "-Declipse.application=org.eclipse.jdt.ls.core.id1".into(),
        "-Dosgi.bundles.defaultStartLevel=4".into(),
        "-Declipse.product=org.eclipse.jdt.ls.core.product".into(),
        "-Dlog.level=ERROR".into(),
        "-Xmx1G".into(),
        "--add-modules=ALL-SYSTEM".into(),
        "--add-opens".into(),
        "java.base/java.util=ALL-UNNAMED".into(),
        "--add-opens".into(),
        "java.base/java.lang=ALL-UNNAMED".into(),
        "-jar".into(),
        found.launcher.to_string_lossy().into_owned(),
        "-configuration".into(),
        found.configuration.to_string_lossy().into_owned(),
        "-data".into(),
        workspace.to_string_lossy().into_owned(),
    ];
    on_output(BuildOutput::Cmd(format!(
        "java -jar {} (the Java language server, which hosts the debugger)",
        found
            .launcher
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    )));

    let mut server = tokio::process::Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context(
            "starting the Java language server. It needs a `java` on PATH — hick installs the \
             server, never a JDK.",
        )?;
    let lsp = Lsp::start(&mut server)?;

    on_output(BuildOutput::Note(
        "importing the project — the first time on a machine takes a while".into(),
    ));
    lsp.request(
        "initialize",
        json!({
            "processId": std::process::id(),
            "rootUri": file_uri(project_dir),
            "capabilities": { "workspace": { "executeCommand": { "dynamicRegistration": true } } },
            // The whole mechanism. Without this the server starts and knows
            // nothing about debugging: java-debug is a bundle it loads.
            "initializationOptions": {
                "bundles": [found.bundle.to_string_lossy()],
                "workspaceFolders": [file_uri(project_dir)],
            },
        }),
        READY_TIMEOUT,
    )
    .await?;
    lsp.notify("initialized", json!({})).await?;
    lsp.notify(
        "textDocument/didOpen",
        json!({ "textDocument": {
            "uri": file_uri(source),
            "languageId": "java",
            "version": 1,
            "text": std::fs::read_to_string(source).unwrap_or_default(),
        }}),
    )
    .await?;
    lsp.wait_ready().await?;
    on_output(BuildOutput::Note("project imported".into()));

    let main_class = main_class(&lsp, source).await?;
    let class_paths = class_paths(&lsp, source).await?;
    let port = lsp
        .request(
            "workspace/executeCommand",
            json!({
                "command": "vscode.java.startDebugSession"
            }),
            COMMAND_TIMEOUT,
        )
        .await?
        .as_u64()
        .context("the language server did not answer `startDebugSession` with a port")?
        as u16;

    on_output(BuildOutput::Exit(0));
    Ok(Prepared {
        port,
        // java-debug does not take a `program`. It takes the class to run and
        // the classpath to find it on, both of which only the language server
        // knows — which is the other half of why this seam exists.
        launch: json!({
            "mainClass": main_class,
            "classPaths": class_paths,
            "cwd": project_dir.to_string_lossy(),
            "console": "internalConsole",
        }),
        server,
    })
}

/// The class holding `main`, asked of the server and falling back to the
/// file's own name.
///
/// `resolveMainClass` answers for the whole project, which is right when a
/// document generates one program and ambiguous when it generates two — so a
/// class matching this file's name wins over the project's first answer.
async fn main_class(lsp: &Lsp, source: &Path) -> Result<String> {
    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let answered = lsp
        .request(
            "workspace/executeCommand",
            json!({ "command": "vscode.java.resolveMainClass", "arguments": [] }),
            COMMAND_TIMEOUT,
        )
        .await
        .unwrap_or(Value::Null);
    let classes: Vec<String> = answered
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("mainClass").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if let Some(mine) = classes
        .iter()
        .find(|class| class.rsplit('.').next() == Some(stem.as_str()))
    {
        return Ok(mine.clone());
    }
    if let Some(first) = classes.first() {
        return Ok(first.clone());
    }
    if stem.is_empty() {
        bail!("no class with a `main` method was found to run");
    }
    Ok(stem)
}

/// The runtime classpath for the file's own project.
async fn class_paths(lsp: &Lsp, source: &Path) -> Result<Vec<String>> {
    let answered = lsp
        .request(
            "workspace/executeCommand",
            json!({
                "command": "java.project.getClasspaths",
                "arguments": [file_uri(source), "{\"scope\":\"runtime\"}"],
            }),
            COMMAND_TIMEOUT,
        )
        .await?;
    Ok(answered
        .get("classpaths")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default())
}

/// A stable short hash for a workspace directory name.
fn seahash(text: &str) -> u64 {
    // FNV-1a: a few lines, no dependency, and this only has to be stable and
    // collision-free enough to name a cache directory.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_workspace_name_is_stable_for_a_directory() {
        assert_eq!(seahash("/a/b"), seahash("/a/b"));
        assert_ne!(seahash("/a/b"), seahash("/a/c"));
    }
}
