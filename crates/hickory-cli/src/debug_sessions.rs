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
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use hick_dap::{Breakpoint, BreakpointStatus, Launch, Mapping, Session};
use tokio::sync::Mutex;

/// How long a session may sit untouched before it is ended for us.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// A started session, plus what a caller needs to keep talking to it.
pub struct Live {
    pub session: Session,
    /// The scratch directory the debuggee runs in. Deleted with the session.
    _scratch: tempfile::TempDir,
    /// The thread the program last stopped on, so a caller need not repeat it.
    pub thread_id: Mutex<Option<i64>>,
    last_touched: Mutex<Instant>,
}

impl Live {
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

    /// Start a session over one document, in a scratch clone of its project.
    ///
    /// The clone is what makes the isolation guarantee true rather than
    /// intended: the debuggee's writes land in a temp directory that is
    /// deleted when the session ends, so stepping through a cell that writes
    /// a file cannot touch the repository.
    pub async fn start(
        &self,
        document: &Path,
        breakpoints: &[Breakpoint],
        program: Option<&str>,
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
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
        let program = match program {
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
        let adapter = hick_dap::adapter_for(&program, project)?;
        tracing_adapter(&adapter);

        let (session, statuses) = Session::start(
            Launch {
                adapter: adapter.command,
                program: program.clone(),
                cwd: scratch.path().to_path_buf(),
                extra: serde_json::json!({}),
            },
            Arc::new(mapping),
            breakpoints,
        )
        .await?;

        let id = format!("dbg-{}", self.next_id.fetch_add(1, Ordering::SeqCst));
        let live = Arc::new(Live {
            session,
            _scratch: scratch,
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

    /// End one session.
    pub async fn stop(&self, id: &str) -> Result<()> {
        let live = self.sessions.lock().await.remove(id);
        match live {
            Some(live) => {
                live.session.shutdown().await;
                Ok(())
            }
            None => bail!("no debug session `{id}` to stop"),
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
        let error = match registry.start(&doc, &[], None).await {
            Ok(_) => panic!("a document with no code started a debugger"),
            Err(error) => format!("{error:#}"),
        };
        assert!(error.contains("nothing to debug"), "{error}");
        assert!(error.contains("hick dap list"), "{error}");
    }
}
