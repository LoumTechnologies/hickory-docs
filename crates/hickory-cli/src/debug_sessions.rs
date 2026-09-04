//! Live debug sessions, held by id.
//!
//! The MCP tools and the app's debug channel both need "the session I started
//! a moment ago", and neither can hold it themselves — one answers stateless
//! tool calls, the other a socket that may reconnect. So sessions live here.
//!
//! Two rules do most of the work:
//!
//! * **A session is bounded.** It ends on an idle timeout whether or not
//!   anyone remembers it. A debugger that outlives the conversation that
//!   started it is a process holding a workdir open, and the first symptom is
//!   a later run failing for no visible reason.
//! * **A session is a reader.** It runs in a scratch clone of the project and
//!   its outputs are discarded, so no amount of stepping can change the
//!   document, the files it generates, or a committed transcript.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use hick_dap::{Breakpoint, BreakpointStatus, BuildOutput, Launch, Mapping, Session};
use tokio::sync::Mutex;

/// How long a session may sit untouched before it is ended for us.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// A started session, plus what a caller needs to keep talking to it.
pub struct Live {
    pub session: Session,
    /// The scratch directory a DOCUMENT's debuggee runs in. Deleted with
    /// the session. `None` for a plain file, which runs in its own project —
    /// see [`Registry::start`].
    scratch: Option<tempfile::TempDir>,
    /// The thread the program last stopped on, so a caller need not repeat it.
    pub thread_id: Mutex<Option<i64>>,
    last_touched: Mutex<Instant>,
}

impl Live {
    /// Where a document's debuggee ran — for tests that check it is gone
    /// afterwards. A plain file has no scratch copy and answers `None`.
    pub fn scratch_path(&self) -> Option<&Path> {
        self.scratch.as_ref().map(|dir| dir.path())
    }

    async fn touch(&self) {
        *self.last_touched.lock().await = Instant::now();
    }

    async fn idle_for(&self) -> Duration {
        self.last_touched.lock().await.elapsed()
    }
}

#[derive(Default)]
pub struct Registry {
    sessions: Mutex<HashMap<String, Arc<Live>>>,
    next_id: AtomicU64,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a session over one document, in a scratch clone of its project
    /// — or over a plain file, in place.
    ///
    /// For a document the clone is what makes the isolation guarantee true
    /// rather than intended: the debuggee's writes land in a temp directory
    /// that is deleted when the session ends, so stepping through a cell
    /// that writes a file cannot touch the repository.
    ///
    /// A file that is not a document — `src/main.rs`, `app.py` — has nothing
    /// woven to protect and no transcript to keep honest; it is the person's
    /// own program in the person's own checkout, and it runs where their
    /// `cargo run` would, with `root` as the folder the app opened. See
    /// [`Self::start_plain`].
    pub async fn start(
        &self,
        document: &Path,
        breakpoints: &[Breakpoint],
        program: Option<&str>,
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        if !is_document(document) {
            // No folder was named, so the repository the file is in stands
            // in for it — the same bound a plain file's language server
            // falls back to.
            let root = repository_of(document);
            return self
                .start_plain(document, &root, breakpoints, on_build)
                .await;
        }
        let source = std::fs::read_to_string(document)
            .with_context(|| format!("reading {}", document.display()))?;
        let project = document.parent().unwrap_or(Path::new("."));

        let scratch = tempfile::tempdir().context("making a scratch directory for the session")?;
        let mapping = Mapping::for_document(document, &source, scratch.path())?;

        // Weave the document's files into the scratch directory. This is the
        // program the debugger will run, and it is a copy on purpose.
        let files = hick_dap::weave_into(&source, scratch.path())?;
        // Which file to run, when the document generates more than one.
        //
        // Guessing was the old behaviour and it is only right by accident: a
        // document with two Python files got the first one, with nothing on
        // screen to say which. Naming it is the caller's job; falling back to
        // the first debuggable file keeps "just debug this" working.
        let entry = match program {
            Some(named) => {
                let wanted = scratch.path().join(named);
                files
                    .iter()
                    .find(|path| **path == wanted)
                    .cloned()
                    .with_context(|| {
                        format!(
                            "this document does not generate {named}. It generates: {}",
                            files
                                .iter()
                                .filter_map(|p| p.strip_prefix(scratch.path()).ok())
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    })?
            }
            None => hick_dap::entry_point(&files)?,
        };
        let adapter = hick_dap::adapter_for(&entry, project)?;
        tracing_adapter(&adapter);

        // A compiled language's generated file is not a program: `Program.cs`
        // is source, and what netcoredbg launches is the assembly a build
        // produces. `entry_point` and `adapter_for` above are unchanged and
        // still name the SOURCE, because the source is what has a language;
        // this is the step that turns it into the thing to launch. For
        // Python, Node and Go it returns the path it was given.
        //
        // `on_build` carries the build's own output out to whoever asked for
        // the session, and it is called on the failing path too — a build
        // that fails says why in MSBuild's words, and losing those in favour
        // of "build failed" is the thing this exists to prevent.
        let program = hick_dap::build(&entry, scratch.path(), project, on_build).await?;

        let (session, statuses) = Session::start(
            Launch {
                transport: adapter.transport,
                adapter_extra: adapter.launch_extra.clone(),
                adapter: adapter.command,
                program: program.clone(),
                cwd: scratch.path().to_path_buf(),
                extra: serde_json::json!({}),
            },
            Arc::new(mapping),
            breakpoints,
        )
        .await?;

        Ok(self.register(session, Some(scratch), statuses).await)
    }

    /// Start a session over a file that is not a document, in its own
    /// project.
    ///
    /// The same adapters, the same session API, the same breakpoints and
    /// frames — with three differences that are all the same difference:
    /// the file is debugged **as itself**. The mapping is the identity
    /// (`Mapping::identity`), so a breakpoint on line 12 is a breakpoint on
    /// line 12 of the file; the build, when the language needs one, runs in
    /// the nearest project above the file and into that project's own
    /// output (`hick_dap::build_plain`); and the program runs with the
    /// project directory as its working directory, because that is where
    /// the person's own `cargo run` or `python app.py` would run it.
    ///
    /// `root` is the folder the app opened. Discovery looks for adapters
    /// from there, the way it does for a document, and no project file above
    /// it counts.
    pub async fn start_plain(
        &self,
        file: &Path,
        root: &Path,
        breakpoints: &[Breakpoint],
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        if !file.is_file() {
            bail!(
                "{} is not a file in this folder, so there is nothing to debug.",
                file.display()
            );
        }
        let adapter = hick_dap::adapter_for(file, root)?;
        tracing_adapter(&adapter);
        let program = hick_dap::build_plain(file, root, on_build).await?;
        let cwd = project_dir_of(root, file);
        let (session, statuses) = Session::start(
            Launch {
                transport: adapter.transport,
                adapter_extra: adapter.launch_extra.clone(),
                adapter: adapter.command,
                program,
                cwd,
                extra: serde_json::json!({}),
            },
            Arc::new(Mapping::identity(file)),
            breakpoints,
        )
        .await?;
        Ok(self.register(session, None, statuses).await)
    }

    async fn register(
        &self,
        session: Session,
        scratch: Option<tempfile::TempDir>,
        statuses: Vec<BreakpointStatus>,
    ) -> (String, Arc<Live>, Vec<BreakpointStatus>) {
        let id = format!("dbg-{}", self.next_id.fetch_add(1, Ordering::SeqCst));
        let live = Arc::new(Live {
            session,
            scratch,
            thread_id: Mutex::new(None),
            last_touched: Mutex::new(Instant::now()),
        });
        self.sessions.lock().await.insert(id.clone(), live.clone());
        (id, live, statuses)
    }

    /// Look a session up, and mark it as still wanted.
    pub async fn get(&self, id: &str) -> Result<Arc<Live>> {
        self.sweep().await;
        let live = self
            .sessions
            .lock()
            .await
            .get(id)
            .cloned()
            .with_context(|| {
                format!(
                    "no debug session `{id}`. It may have ended on its own: a session that sits \
                     untouched for {} minutes is closed, because a debugger nobody is watching is \
                     a process holding a working directory open.",
                    IDLE_TIMEOUT.as_secs() / 60
                )
            })?;
        live.touch().await;
        Ok(live)
    }

    /// End one session, and complain if there was nothing to end.
    pub async fn stop(&self, id: &str) -> Result<()> {
        if self.reap(id).await {
            Ok(())
        } else {
            bail!("no debug session `{id}` to stop")
        }
    }

    /// End one session if it is still here; true when this call did the work.
    ///
    /// The debuggee finishing and an explicit stop legitimately race — the
    /// app reaps the session the moment the program ends, and the person may
    /// press Stop a beat later. Whoever gets here second must find "already
    /// gone", not an error, so this is the idempotent form `stop` and the
    /// finished-program path both stand on. The adapter process is killed by
    /// the shutdown, and the scratch clone is deleted when the last handle to
    /// the session drops.
    pub async fn reap(&self, id: &str) -> bool {
        let live = self.sessions.lock().await.remove(id);
        match live {
            Some(live) => {
                live.session.shutdown().await;
                true
            }
            None => false,
        }
    }

    /// End every session. Called when the process that owns them is going
    /// away, so nothing outlives the thing that started it.
    pub async fn stop_all(&self) {
        let sessions: Vec<Arc<Live>> = self.sessions.lock().await.drain().map(|(_, v)| v).collect();
        for live in sessions {
            live.session.shutdown().await;
        }
    }

    /// Close anything that has been idle too long.
    async fn sweep(&self) {
        let mut expired = Vec::new();
        {
            let sessions = self.sessions.lock().await;
            for (id, live) in sessions.iter() {
                if live.idle_for().await > IDLE_TIMEOUT {
                    expired.push(id.clone());
                }
            }
        }
        for id in expired {
            if let Some(live) = self.sessions.lock().await.remove(&id) {
                tracing::info!(session = %id, "closing an idle debug session");
                live.session.shutdown().await;
            }
        }
    }

    pub async fn len(&self) -> usize {
        self.sessions.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

fn tracing_adapter(found: &hick_dap::Discovered) {
    tracing::info!(
        adapter = found.adapter,
        origin = found.origin,
        command = found.command.join(" "),
        "discovered a debug adapter"
    );
}

/// The language of a generated file, by extension, for adapter routing.
///
/// Whether a path is a `.hick` document, as opposed to a file to debug as
/// itself. By extension: the same line `hick-lsp` draws for a plain file's
/// language server.
pub fn is_document(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "hick")
}

/// The nearest ancestor holding a `.git`, else the file's own directory.
fn repository_of(file: &Path) -> PathBuf {
    let own = file.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut dir = own.as_path();
    loop {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
        let Some(parent) = dir.parent() else {
            return own;
        };
        dir = parent;
    }
}

/// The directory a plain file's program runs in: the nearest project above
/// it (`Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, a
/// `.csproj`), stopping at `root`, else the file's own directory.
///
/// The same rule the run-test gutter uses to pick where `cargo test` runs,
/// for the same reason: a program reads its config and writes its output
/// relative to where its own tools run it.
fn project_dir_of(root: &Path, file: &Path) -> PathBuf {
    const MANIFESTS: [&str; 5] = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        ".csproj",
    ];
    let own = file.parent().unwrap_or(root).to_path_buf();
    let mut dir = own.as_path();
    loop {
        let Ok(entries) = std::fs::read_dir(dir) else {
            break;
        };
        let has_manifest = entries.flatten().any(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            MANIFESTS.iter().any(|m| {
                if m.starts_with('.') {
                    name.ends_with(m)
                } else {
                    name == *m
                }
            })
        });
        if has_manifest {
            return dir.to_path_buf();
        }
        if dir == root {
            break;
        }
        let Some(parent) = dir.parent() else { break };
        dir = parent;
    }
    own
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn an_unknown_session_says_why_it_might_be_gone() {
        let registry = Registry::new();
        let error = match registry.get("dbg-404").await {
            Ok(_) => panic!("a session that was never started was found"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("dbg-404"), "{error}");
        // The likely cause, named: sessions expire, and a caller that does
        // not know that reads "no such session" as a bug.
        assert!(error.contains("untouched"), "{error}");
    }

    #[tokio::test]
    async fn stopping_something_that_is_not_running_is_an_error_not_a_panic() {
        let registry = Registry::new();
        assert!(registry.stop("dbg-nope").await.is_err());
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn a_document_with_nothing_debuggable_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("d.hick");
        std::fs::write(
            &doc,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             # Only prose\n\
             </hick:doc>\n",
        )
        .unwrap();
        let registry = Registry::new();
        let error = match registry.start(&doc, &[], None, &mut |_| {}).await {
            Ok(_) => panic!("a document with no code started a debugger"),
            Err(error) => format!("{error:#}"),
        };
        assert!(error.contains("nothing to debug"), "{error}");
        assert!(error.contains("hick dap list"), "{error}");
    }

    #[test]
    fn a_plain_file_runs_in_the_nearest_project_above_it() {
        // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("crates/foo/src")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        std::fs::write(root.join("crates/foo/Cargo.toml"), "").unwrap();
        let file = root.join("crates/foo/src/main.rs");
        std::fs::write(&file, "").unwrap();
        assert_eq!(project_dir_of(root, &file), root.join("crates/foo"));
        // A stray script with no manifest anywhere runs where it lives.
        let loose = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(loose.path().join("tools")).unwrap();
        let script = loose.path().join("tools/x.py");
        std::fs::write(&script, "").unwrap();
        assert_eq!(
            project_dir_of(loose.path(), &script),
            loose.path().join("tools")
        );
        assert!(!is_document(&script));
        assert!(is_document(Path::new("notes/a.hick")));
    }
}
