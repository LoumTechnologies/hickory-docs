<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/debug_sessions.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-24a9413a429b"></a>
## `crates/hickory-cli/src/debug_sessions.rs`

Related contracts: [Launch the artifact built from the selected compiled project](../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [Debug the latest atomic project that still contains the displayed bytes](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md), [Debugging A Document Never Writes To The Project](../../../../guarantees/debugging/debugging-never-writes-to-the-project/contract.md), [Keep language tools and debug launches tied to their captured project](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md).

<a id="f-24a9413a429b-profiles"></a>
### `profiles`

Live debug sessions, held by id.
The MCP tools and the app's debug channel both need "the session I started
a moment ago", and neither can hold it themselves — one answers stateless
tool calls, the other a socket that may reconnect. So sessions live here.
Two rules do most of the work:
* **A session is bounded.** It ends on an idle timeout whether or not
anyone remembers it. A debugger that outlives the conversation that
started it is a process holding a workdir open, and the first symptom is
a later run failing for no visible reason.
* **A session is a reader.** It runs in a scratch clone of the project and
its outputs are discarded, so no amount of stepping can change the
document, the files it generates, or a committed transcript.

<storycode:copy id="f-24a9413a429b-profiles">
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

#[path = "debug_profiles.rs"]
mod profiles;
</storycode:copy>
<a id="f-24a9413a429b-reading"></a>
### `reading`

<storycode:copy id="f-24a9413a429b-reading">
#[path = "debug_reading.rs"]
mod reading;
</storycode:copy>
<a id="f-24a9413a429b-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-24a9413a429b-use_declaration">
pub(crate) use profiles::{safe_cwd, source_document};
</storycode:copy>
<a id="f-24a9413a429b-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-24a9413a429b-use_declaration-2">
pub use profiles::{LaunchContext, ProfileSelection};
</storycode:copy>
<a id="f-24a9413a429b-IDLE_TIMEOUT"></a>
### `IDLE_TIMEOUT`

How long a session may sit untouched before it is ended for us.

<storycode:copy id="f-24a9413a429b-IDLE_TIMEOUT">
/// How long a session may sit untouched before it is ended for us.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);
</storycode:copy>
<a id="f-24a9413a429b-Live"></a>
### `Live`

Keep positional operations ordered through response publication. The owned
session mutex can span initial launch without hiding the real session identity;
breakpoint configuration has its own ordered lane so it remains usable while
execution waits for a stop. Explicit Stop bypasses both waits and closes the
adapter. These are engine operation lifetimes, independent of any window's editor.

A started session, plus what a caller needs to keep talking to it.

<storycode:copy id="f-24a9413a429b-Live">
/// A started session, plus what a caller needs to keep talking to it.
#[derive(Clone)]
pub struct BreakpointIntent {
    pub request_id: String,
    pub source_hash: String,
}

pub(crate) struct PublishedStack {
    pub stop: u64,
    pub thread: i64,
    pub frame_ids: Vec<i64>,
}

pub struct Live {
    /// Exact frames from the last interactive stopped response.
    pub(crate) published_stack: Mutex<Option<PublishedStack>>,
    /// Positive references actually reported in the current stopped stack.
    pub adapter_sources: Mutex<HashMap<(u64, i64, i64), i64>>,
    pub breakpoint_intents: std::sync::Mutex<HashMap<String, BreakpointIntent>>,
    pub value_references: Mutex<HashMap<(u64, i64, i64), std::collections::HashSet<i64>>>,
    pub source_aliases: Vec<String>,
    pub operation: Arc<Mutex<()>>,
    pub breakpoint_operation: Mutex<()>,
    pub stop_generation: Mutex<u64>,
    pub stopped: Mutex<Option<hick_dap::Stopped>>,
    pub observations: Mutex<hick_observe::Run>,
    pub reading: Mutex<Option<hick_observe::reading::Reading>>,
    pub session: Session,
    /// Owned constructed runtime or compiled-test artifacts, deleted with the session.
    /// Plain source debugging normally uses its project directly; selected compiled
    /// tests retain an owned build target while keeping a source-side working directory.
    scratch: Option<tempfile::TempDir>,
    /// The thread the program last stopped on, so a caller need not repeat it.
    pub thread_id: Mutex<Option<i64>>,
    last_touched: Mutex<Instant>,
}
</storycode:copy>
<a id="debug-reference-provenance"></a>
### Keep composite references inside their observed frame

Adapters can reuse numeric references. Record the roots returned by locals,
evaluation and expansion under the exact stop, selected thread and frame.
Unknown references refuse before adapter access. Motion and thread selection
retire this bounded table instead of granting a new frame an old capability.

<storycode:copy id="debug-reference-provenance">
impl Live {
    /// A variablesReference is meaningful only where the adapter reported it.
    /// Retire the entire table after motion or thread selection; selecting a
    /// caller still needs that caller's own observed reference roots.
    pub async fn remember_values(
        &self,
        stop: u64,
        thread: i64,
        frame: i64,
        values: &[hick_dap::Variable],
    ) -> Result<()> {
        let mut table = self.value_references.lock().await;
        anyhow::ensure!(
            table.contains_key(&(stop, thread, frame)) || table.len() < 64,
            "Variable frame budget exceeded. Resume or restart the debugger before reading more frames."
        );
        let known = table.entry((stop, thread, frame)).or_default();
        for value in values {
            anyhow::ensure!(
                (0..=i32::MAX as i64).contains(&value.variables_reference),
                "Adapter reported an invalid variable reference. Read this frame again."
            );
            if value.variables_reference > 0 {
                known.insert(value.variables_reference);
            }
        }
        anyhow::ensure!(
            known.len() <= 16384,
            "Variable reference budget exceeded. Resume or restart before expanding more values."
        );
        Ok(())
    }
    pub async fn require_reference(
        &self,
        stop: u64,
        thread: i64,
        frame: i64,
        reference: i64,
    ) -> Result<()> {
        anyhow::ensure!(
            reference > 0
                && self
                    .value_references
                    .lock()
                    .await
                    .get(&(stop, thread, frame))
                    .is_some_and(|values| values.contains(&reference)),
            "Variable reference was not reported in this stopped frame. Read its locals or evaluate the expression again."
        );
        Ok(())
    }
}
</storycode:copy>

<a id="f-24a9413a429b-impl-Live"></a>
### `impl-Live`

<storycode:copy id="f-24a9413a429b-impl-Live">
impl Live {
    /// Owned runtime/artifact directory, for lifetime and cleanup checks.
    /// Plain source launches usually have none; compiled selected tests retain
    /// their build artifacts here even when their working directory is source-side.
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
</storycode:copy>
<a id="f-24a9413a429b-Registry"></a>
### `Registry`

<storycode:copy id="f-24a9413a429b-Registry">
#[derive(Default)]
pub struct Registry {
    sessions: Mutex<HashMap<String, Arc<Live>>>,
    next_id: AtomicU64,
    session_namespace: std::sync::OnceLock<Result<String, String>>,
    projects: Arc<hick_lsp::ProjectContexts>,
    host_root: Option<PathBuf>,
}
</storycode:copy>
<a id="debug-session-identity"></a>
### Keep session identity distinct across engine lifetimes

A restarted engine resets its counter. Reusing the old session spelling could
make delayed cleanup from a previously owned launch stop a new session. Allocate
one namespace per registry and keep its local counter inside that namespace;
frontends continue treating the entire identifier as opaque.

<storycode:copy id="debug-session-identity">
impl Registry {
    fn ensure_session_namespace(&self) -> Result<&str> {
        let value=self.session_namespace.get_or_init(|| {
            let mut nonce=[0u8;16];
            getrandom::fill(&mut nonce).map_err(|error|format!("Cannot allocate a debugger identity: {error}. Restart the engine when system randomness is available."))?;
            Ok(nonce.iter().map(|byte|format!("{byte:02x}")).collect())
        });
        value.as_deref().map_err(|error| anyhow::anyhow!("{error}"))
    }
    fn next_session_identity(&self) -> Result<String> {
        Ok(format!(
            "dbg-{}-{}",
            self.ensure_session_namespace()?,
            self.next_id.fetch_add(1, Ordering::SeqCst)
        ))
    }
}
#[cfg(test)]
mod session_identity_tests {
    use super::*;
    #[test]
    fn delayed_owned_cleanup_cannot_name_a_restarted_engines_session() {
        let previous = Registry::new();
        let first = previous.next_session_identity().unwrap();
        let second = previous.next_session_identity().unwrap();
        let restarted = Registry::new();
        let after_restart = restarted.next_session_identity().unwrap();
        assert_ne!(first, second);
        assert_ne!(first, after_restart);
        assert_ne!(second, after_restart);
    }
}
</storycode:copy>

<a id="f-24a9413a429b-impl-Registry"></a>
### `impl-Registry`

<storycode:copy id="f-24a9413a429b-impl-Registry">
impl Registry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_projects(projects: Arc<hick_lsp::ProjectContexts>, host_root: PathBuf) -> Self {
        Self {
            projects,
            host_root: Some(host_root),
            ..Self::default()
        }
    }
    pub async fn start_captured(
        &self,
        uri: &tower_lsp::lsp_types::Url,
        breakpoints: &[Breakpoint],
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        self.ensure_session_namespace()?;
        self.start_captured_with_sources(uri, None, breakpoints, &[], on_build)
            .await
    }
    pub async fn start_captured_with_sources(
        &self,
        uri: &tower_lsp::lsp_types::Url,
        requested_program: Option<&str>,
        breakpoints: &[Breakpoint],
        sources: &[hick_dap::SourceBreakpoints],
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        self.ensure_session_namespace()?;
        let (id, path, context) = self.projects.get(uri).map_err(anyhow::Error::msg)?;
        let scratch = tempfile::tempdir()?;
        let (entry, mapping) = if let Some(program) = &context.recorded_fence {
            anyhow::ensure!(path == hick_lsp::ProjectContext::RECORDED_MESSAGE,
                "Debug the original message rather than its isolated program copy.");
            anyhow::ensure!(requested_program.is_none_or(|target| target == program),
                "The requested program is not the selected recorded fence. Choose Debug again.");
            let mapping = Mapping::recorded_fence(&context.tree, scratch.path(), &id, program)?;
            let [source] = sources else {
                anyhow::bail!("Recorded fence Debug needs its exact message revision. Reopen the fence and retry.");
            };
            anyhow::ensure!(source.source.is_empty() && source.document.as_path() == mapping.document()
                && source.source_hash.is_some() && source.source_hash == mapping.source_hash(mapping.document()),
                "The recorded fence does not match this message revision. Reopen the fence and retry.");
            let entry = scratch.path().join(program);
            let selected = context.tree.get(program).context("Recorded fence program is unavailable")?;
            std::fs::write(&entry, selected.bytes.as_ref())?;
            (entry, mapping)
        } else {
            hick_lsp::write_tree(scratch.path(), &context.tree).map_err(anyhow::Error::msg)?;
            let mapping = Mapping::captured(&context.tree, scratch.path(), &id, &path)?;
            (scratch.path().join(&path), mapping)
        };
        let adapter =
            hick_dap::adapter_for(&entry, self.host_root.as_deref().unwrap_or(scratch.path()))?;
        let program = hick_dap::build(&entry, scratch.path(), scratch.path(), on_build).await?;
        let (session, statuses) = Session::start_with_sources(
            Launch {
                transport: adapter.transport,
                multi_session: adapter.multi_session,
                adapter_extra: adapter.launch_extra.clone(),
                adapter: adapter.command,
                program,
                cwd: scratch.path().into(),
                extra: serde_json::json!({}),
            },
            Arc::new(mapping),
            breakpoints,
            sources,
        )
        .await?;
        self.register(session, Some(scratch), statuses).await
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
        self.ensure_session_namespace()?;
        self.start_at(
            document,
            breakpoints,
            program,
            &hick_lsp::ProjectEndpoint::New,
            on_build,
        )
        .await
    }
    pub async fn start_at(
        &self,
        document: &Path,
        breakpoints: &[Breakpoint],
        program: Option<&str>,
        endpoint: &hick_lsp::ProjectEndpoint,
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        self.ensure_session_namespace()?;
        self.start_at_with_sources(document, breakpoints, program, endpoint, &[], on_build)
            .await
    }
    pub async fn start_at_with_sources(
        &self,
        document: &Path,
        breakpoints: &[Breakpoint],
        program: Option<&str>,
        endpoint: &hick_lsp::ProjectEndpoint,
        sources: &[hick_dap::SourceBreakpoints],
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        self.ensure_session_namespace()?;
        if !is_document(document) {
            // No folder was named, so the repository the file is in stands
            // in for it — the same bound a plain file's language server
            // falls back to.
            let root = repository_of(document);
            return self
                .start_plain_with_sources(document, &root, breakpoints, sources, on_build)
                .await;
        }
        let source = std::fs::read_to_string(document)
            .with_context(|| format!("reading {}", document.display()))?;
        let project = document.parent().unwrap_or(Path::new("."));

        let scratch = tempfile::tempdir().context("making a scratch directory for the session")?;
        let (mapping, files, selected_program) =
            crate::project_documents::debug_state_at(document, &source, scratch.path(), endpoint)?;
        // Which file to run, when the document generates more than one.
        //
        // Guessing was the old behaviour and it is only right by accident: a
        // document with two Python files got the first one, with nothing on
        // screen to say which. Naming it is the caller's job; falling back to
        // the first debuggable file keeps "just debug this" working.
        let entry = match selected_program.as_deref().or(program) {
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

        let (session, statuses) = Session::start_with_sources(
            Launch {
                transport: adapter.transport,
                multi_session: adapter.multi_session,
                adapter_extra: adapter.launch_extra.clone(),
                adapter: adapter.command,
                program: program.clone(),
                cwd: scratch.path().to_path_buf(),
                extra: serde_json::json!({}),
            },
            Arc::new(mapping),
            breakpoints,
            sources,
        )
        .await?;

        self.register(session, Some(scratch), statuses).await
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
        self.ensure_session_namespace()?;
        self.start_plain_with_sources(file, root, breakpoints, &[], on_build)
            .await
    }
    pub async fn start_plain_with_sources(
        &self,
        file: &Path,
        root: &Path,
        breakpoints: &[Breakpoint],
        sources: &[hick_dap::SourceBreakpoints],
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        self.ensure_session_namespace()?;
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
        let extra = if hick_dap::language_of(file) == Some("python") {
            hick_project_env::python_interpreter(file.parent().unwrap_or(root), root)
                .map(|python| serde_json::json!({"python": python}))
                .unwrap_or_else(|| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };
        let (session, statuses) = Session::start_with_sources(
            Launch {
                transport: adapter.transport,
                multi_session: adapter.multi_session,
                adapter_extra: adapter.launch_extra.clone(),
                adapter: adapter.command,
                program,
                cwd,
                extra,
            },
            Arc::new(Mapping::identity(file)),
            breakpoints,
            sources,
        )
        .await?;
        self.register(session, None, statuses).await
    }

    #[cfg(test)]
    pub(crate) async fn register_test_session(&self, session: Session) -> Result<(String, Arc<Live>)> {
        let (id, live, _) = self.register(session, None, vec![]).await?;
        Ok((id, live))
    }

    async fn register(
        &self,
        session: Session,
        scratch: Option<tempfile::TempDir>,
        statuses: Vec<BreakpointStatus>,
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        let id = match self.next_session_identity() {
            Ok(id) => id,
            Err(error) => {
                session.shutdown().await;
                return Err(error);
            }
        };
        let observations = hick_observe::Run::new(
            session.observation_source.clone().into_iter().collect(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
        let source_aliases = session
            .observation_source
            .as_ref()
            .and_then(|source| {
                let root = self.host_root.as_ref()?;
                let path = Path::new(&source.uri).canonicalize().ok()?;
                let root = root.canonicalize().ok()?;
                let relative = path
                    .strip_prefix(root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");
                Some(vec![relative.clone(), format!("hick:///{relative}")])
            })
            .unwrap_or_default();
        let live = Arc::new(Live {
            breakpoint_intents: std::sync::Mutex::new(HashMap::new()),
            published_stack: Mutex::new(None),
            adapter_sources: Mutex::new(HashMap::new()),
            value_references: Mutex::new(HashMap::new()),
            source_aliases,
            operation: Arc::new(Mutex::new(())),
            breakpoint_operation: Mutex::new(()),
            stop_generation: Mutex::new(1),
            stopped: Mutex::new(None),
            observations: Mutex::new(observations),
            reading: Mutex::new(None),
            session,
            scratch,
            thread_id: Mutex::new(None),
            last_touched: Mutex::new(Instant::now()),
        });
        self.sessions.lock().await.insert(id.clone(), live.clone());
        Ok((id, live, statuses))
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
</storycode:copy>
<a id="f-24a9413a429b-tracing_adapter"></a>
### `tracing_adapter`

<storycode:copy id="f-24a9413a429b-tracing_adapter">
fn tracing_adapter(found: &hick_dap::Discovered) {
    tracing::info!(
        adapter = found.adapter,
        origin = found.origin,
        command = found.command.join(" "),
        "discovered a debug adapter"
    );
}
</storycode:copy>
<a id="f-24a9413a429b-is_document"></a>
### `is_document`

The language of a generated file, by extension, for adapter routing.
Whether a path is a Markdown document, as opposed to a file to debug as
itself. By extension: the same line `hick-lsp` draws for a plain file's
language server.

<storycode:copy id="f-24a9413a429b-is_document">
/// The language of a generated file, by extension, for adapter routing.
///
/// Whether a path is a Markdown document, as opposed to a file to debug as
/// itself. By extension: the same line `hick-lsp` draws for a plain file's
/// language server.
pub fn is_document(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
}
</storycode:copy>
<a id="f-24a9413a429b-repository_of"></a>
### `repository_of`

The nearest ancestor holding a `.git`, else the file's own directory.

<storycode:copy id="f-24a9413a429b-repository_of">
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
</storycode:copy>
<a id="f-24a9413a429b-project_dir_of"></a>
### `project_dir_of`

The directory a plain file's program runs in: the nearest project above
it (`Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, a
`.csproj`), stopping at `root`, else the file's own directory.
The same rule the run-test gutter uses to pick where `cargo test` runs,
for the same reason: a program reads its config and writes its output
relative to where its own tools run it.

<storycode:copy id="f-24a9413a429b-project_dir_of">
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
</storycode:copy>
<a id="f-24a9413a429b-tests"></a>
### `tests`

<storycode:copy id="f-24a9413a429b-tests">
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
        let doc = dir.path().join("d.md");
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

    #[tokio::test(flavor = "multi_thread")]
    async fn a_markdown_csharp_document_builds_and_stops_on_its_own_line() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        #[cfg(unix)]
        {
            let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
            if cache.join("adapters/netcoredbg/netcoredbg").exists() {
                std::os::unix::fs::symlink(cache, root.join(".hick-cache")).unwrap();
            }
        }
        if hick_dap::discover("csharp", root).is_none() {
            eprintln!("SKIPPED: no C# debug adapter (`hick dap install csharp`)");
            return;
        }
        if std::process::Command::new("dotnet")
            .arg("--version")
            .status()
            .map(|status| !status.success())
            .unwrap_or(true)
        {
            eprintln!("SKIPPED: no .NET SDK on this machine");
            return;
        }
        let doc = root.join("greeter.md");
        std::fs::write(
            &doc,
            r#"# Greeter

<hick:file path="app/Greeter.csproj">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
  </PropertyGroup>
</Project>
</hick:file>

<hick:file path="app/Program.cs">
class Program
{
    static void Main()
    {
        System.Console.WriteLine("Hello");
    }
}
</hick:file>
"#,
        )
        .unwrap();

        let registry = Registry::new();
        let breakpoints = [Breakpoint {
            line: 16,
            condition: None,
            hit_condition: None,
            log_message: None,
        }];
        let (_id, live, _) = registry
            .start(&doc, &breakpoints, None, &mut |_| {})
            .await
            .expect("the Markdown document's C# project starts");
        let stopped = live
            .session
            .wait_for_stop(Duration::from_secs(60))
            .await
            .expect("the adapter answers")
            .expect("the C# breakpoint stops the program");
        let frames = live.session.stack(stopped.thread_id).await.unwrap();
        assert_eq!(frames[0].line, Some(16), "{frames:?}");
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
        assert!(is_document(Path::new("notes/a.md")));
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/debug_sessions.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.


## Connect source-test preparation to the existing session registry

The separate launcher owns runtime assembly and provider configuration. It calls
the same registry registration and session lifecycle, so test launches do not
create a parallel debugger or bypass source/stop authority.

<storycode:copy id="source-test-debug-launcher-module">
#[path = "test_debug_sessions.rs"]
mod test_launcher;
pub(crate) use test_launcher::TestLaunch;
</storycode:copy>
<storycode:file path="crates/hickory-cli/src/debug_sessions.rs"><storycode:paste select="#f-24a9413a429b-profiles"/><storycode:paste select="#source-test-debug-launcher-module"/><storycode:paste select="#f-24a9413a429b-reading"/><storycode:paste select="#f-24a9413a429b-use_declaration-2"/><storycode:paste select="#f-24a9413a429b-use_declaration"/><storycode:paste select="#f-24a9413a429b-IDLE_TIMEOUT"/><storycode:paste select="#f-24a9413a429b-Live"/><storycode:paste select="#debug-reference-provenance"/><storycode:paste select="#f-24a9413a429b-impl-Live"/><storycode:paste select="#f-24a9413a429b-Registry"/><storycode:paste select="#debug-session-identity"/><storycode:paste select="#f-24a9413a429b-impl-Registry"/><storycode:paste select="#f-24a9413a429b-tracing_adapter"/><storycode:paste select="#f-24a9413a429b-is_document"/><storycode:paste select="#f-24a9413a429b-repository_of"/><storycode:paste select="#f-24a9413a429b-project_dir_of"/><storycode:paste select="#f-24a9413a429b-tests"/></storycode:file>

</storycode:doc>
