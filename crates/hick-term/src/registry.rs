//! Every session in the window, and the one queue across all of them.
//!
//! The registry is what makes attention a property of the *window* rather
//! than of whichever pane you happen to be looking at: a session that needs
//! you claims its place whether its tab is open, buried, or closed.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Result, bail};

use crate::attention::{Claim, attention_order};
use crate::config::TermConfig;
use crate::session::{Session, SessionSpec, SessionSummary};
use crate::turbo::turbo_choice;

/// All live sessions.
pub struct Terminals {
    config: TermConfig,
    sessions: Mutex<Vec<Arc<Session>>>,
    next_id: AtomicU64,
    /// Whether routine declared prompts are answered without you. Off until
    /// someone turns it on, and never restored from disk — an auto-answerer
    /// that survives a restart you did not ask for is a surprise nobody
    /// wants twice.
    turbo: AtomicBool,
}

impl Terminals {
    pub fn new(config: TermConfig) -> Self {
        Terminals {
            config,
            sessions: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
            turbo: AtomicBool::new(false),
        }
    }

    pub fn turbo(&self) -> bool {
        self.turbo.load(Ordering::SeqCst)
    }

    pub fn set_turbo(&self, on: bool) {
        self.turbo.store(on, Ordering::SeqCst);
    }

    /// Answer every prompt turbo is allowed to answer, and say which.
    ///
    /// Explicit, and called from one place, because auto-answering is a side
    /// effect: hiding it inside "list the sessions" would make reading the
    /// queue change the world.
    pub fn sweep_turbo(&self) -> Vec<String> {
        if !self.turbo() {
            return Vec::new();
        }
        let sessions = self.sessions.lock().expect("sessions lock").clone();
        let mut answered = Vec::new();
        for session in sessions {
            let Some(prompt) = session.prompt() else {
                continue;
            };
            let Some(choice) = turbo_choice(&prompt) else {
                continue;
            };
            if session.write(choice.send.as_bytes()).is_ok() {
                session.declare_prompt(None);
                answered.push(session.id.clone());
            }
        }
        answered
    }

    /// Start a session and keep it.
    pub fn open(&self, spec: SessionSpec) -> Result<Arc<Session>> {
        if !spec.cwd.is_dir() {
            bail!(
                "cannot open a terminal in {}: no such directory. Open a folder that \
                 exists, or leave the working directory out to use the session's own.",
                spec.cwd.display()
            );
        }
        let id = format!("term-{}", self.next_id.fetch_add(1, Ordering::SeqCst));
        let session = Session::spawn(id, spec, &self.config)?;
        self.sessions
            .lock()
            .expect("sessions lock")
            .push(session.clone());
        Ok(session)
    }

    /// Start a session in a fresh git worktree off `repo`, on its own branch.
    ///
    /// The answer to two agents editing the same checkout: the session's
    /// branch and dirty state belong to the session, so parallel work does
    /// not become parallel mystery.
    pub fn open_in_worktree(
        &self,
        repo: &Path,
        branch: &str,
        mut spec: SessionSpec,
    ) -> Result<Arc<Session>> {
        let parent = repo.parent().unwrap_or(repo);
        let name = repo.file_name().map(|n| n.to_string_lossy().to_string());
        let dir = parent.join(format!(
            "{}-{branch}",
            name.as_deref().unwrap_or("worktree")
        ));
        crate::git::add_worktree(repo, &dir, branch)?;
        spec.cwd = dir;
        self.open(spec)
    }

    pub fn get(&self, id: &str) -> Option<Arc<Session>> {
        self.sessions
            .lock()
            .expect("sessions lock")
            .iter()
            .find(|s| s.id == id)
            .cloned()
    }

    /// Stop a session and forget it.
    pub fn close(&self, id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().expect("sessions lock");
        let Some(at) = sessions.iter().position(|s| s.id == id) else {
            bail!("no terminal {id} in this session — it may already have been closed");
        };
        let session = sessions.remove(at);
        drop(sessions);
        // A session whose process is already gone is closed, not an error.
        let _ = session.kill();
        Ok(())
    }

    /// Every session, as the API describes them.
    pub fn summaries(&self) -> Vec<SessionSummary> {
        let sessions = self.sessions.lock().expect("sessions lock").clone();
        sessions.iter().map(|s| s.summary()).collect()
    }

    /// The ids of the sessions claiming attention, most-claiming first.
    ///
    /// Monitors are excluded by construction: a dev server that just
    /// restarted is not asking you for anything, and putting it in the queue
    /// is how the queue stops meaning something.
    pub fn attention(&self, summaries: &[SessionSummary]) -> Vec<String> {
        let claims: Vec<Claim> = summaries
            .iter()
            .filter(|s| !s.monitor)
            .filter_map(|s| {
                Some(Claim {
                    id: s.id.clone(),
                    state: state_of(&s.state)?,
                    dirty: s.dirty,
                    since_ms: s.since_ms,
                })
            })
            .collect();
        attention_order(&claims)
    }
}

fn state_of(name: &str) -> Option<crate::classify::SessionState> {
    use crate::classify::SessionState::*;
    Some(match name {
        "needs-you" => NeedsYou,
        "working" => Working,
        "idle" => Idle,
        "finished" => Finished,
        "failed" => Failed,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Prompt, PromptSource};

    fn spec(cwd: &Path, monitor: bool) -> SessionSpec {
        SessionSpec {
            title: "test".to_string(),
            cwd: cwd.to_path_buf(),
            // `cat` holds the terminal open and echoes, which is enough to be
            // a session without depending on any particular shell.
            argv: vec!["cat".to_string()],
            monitor,
        }
    }

    /// Protects docs/guarantees/terminal/a-terminal-outlives-its-pane.md
    #[test]
    fn a_session_replays_what_it_said_to_a_client_that_arrives_late() {
        let dir = tempfile::tempdir().unwrap();
        let terminals = Terminals::new(TermConfig::default());
        let session = terminals.open(spec(dir.path(), false)).unwrap();

        session.write(b"hello\n").unwrap();
        // `cat` echoes; give the reader thread a moment to see it.
        for _ in 0..50 {
            if !session.attach().0.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let (replay, _live) = session.attach();
        let text = String::from_utf8_lossy(&replay);
        assert!(text.contains("hello"), "replayed: {text:?}");

        terminals.close(&session.id).unwrap();
        assert!(terminals.get(&session.id).is_none());
    }

    /// Protects docs/guarantees/terminal/the-attention-queue-ranks-by-claim.md
    #[test]
    fn a_monitor_never_enters_the_queue_however_loudly_it_fails() {
        let dir = tempfile::tempdir().unwrap();
        let terminals = Terminals::new(TermConfig::default());
        let watcher = terminals.open(spec(dir.path(), true)).unwrap();
        let task = terminals.open(spec(dir.path(), false)).unwrap();
        watcher.declare_prompt(Some(Prompt {
            question: "restart?".to_string(),
            choices: Vec::new(),
            source: PromptSource::Declared,
        }));
        task.declare_prompt(Some(Prompt {
            question: "may I write src/main.rs?".to_string(),
            choices: Vec::new(),
            source: PromptSource::Declared,
        }));

        let summaries = terminals.summaries();
        assert_eq!(terminals.attention(&summaries), vec![task.id.clone()]);

        terminals.close(&watcher.id).unwrap();
        terminals.close(&task.id).unwrap();
    }

    /// Asking twice in a row is the ordinary case — the client polls — and the
    /// second ask is the one where nothing has changed. Bounded rather than
    /// plainly called, because the bug this catches (taking the state lock
    /// twice in one pass) hangs instead of failing.
    ///
    /// Protects docs/guarantees/terminal/a-session-says-what-it-is-doing.md
    #[test]
    fn asking_what_a_session_is_doing_twice_answers_twice() {
        let dir = tempfile::tempdir().unwrap();
        let terminals = Terminals::new(TermConfig::default());
        let session = terminals.open(spec(dir.path(), false)).unwrap();

        let (tx, rx) = std::sync::mpsc::channel();
        let polled = session.clone();
        std::thread::spawn(move || {
            let first = polled.summary().state;
            let second = polled.summary().state;
            let _ = tx.send((first, second));
        });
        let (first, second) = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("summary answers rather than hanging");
        assert_eq!(first, second);

        terminals.close(&session.id).unwrap();
    }

    /// Protects docs/guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse.md
    #[test]
    fn turbo_leaves_a_guessed_prompt_for_a_person_even_when_it_is_on() {
        let dir = tempfile::tempdir().unwrap();
        let terminals = Terminals::new(TermConfig::default());
        let session = terminals.open(spec(dir.path(), false)).unwrap();
        terminals.set_turbo(true);

        session.declare_prompt(Some(Prompt {
            question: "Delete build/? [y/N]".to_string(),
            choices: vec![crate::session::Choice {
                label: "Yes".to_string(),
                send: "y\n".to_string(),
                destructive: false,
            }],
            source: PromptSource::Guessed,
        }));
        assert!(terminals.sweep_turbo().is_empty());
        assert!(session.prompt().is_some(), "the prompt is still waiting");

        terminals.close(&session.id).unwrap();
    }

    #[test]
    fn turbo_off_answers_nothing_at_all() {
        let dir = tempfile::tempdir().unwrap();
        let terminals = Terminals::new(TermConfig::default());
        let session = terminals.open(spec(dir.path(), false)).unwrap();
        session.declare_prompt(Some(Prompt {
            question: "Read Cargo.toml?".to_string(),
            choices: vec![crate::session::Choice {
                label: "Allow".to_string(),
                send: "1\n".to_string(),
                destructive: false,
            }],
            source: PromptSource::Declared,
        }));
        assert!(terminals.sweep_turbo().is_empty());

        terminals.set_turbo(true);
        assert_eq!(terminals.sweep_turbo(), vec![session.id.clone()]);
        assert!(session.prompt().is_none(), "answered, so no longer waiting");

        terminals.close(&session.id).unwrap();
    }

    #[test]
    fn a_terminal_in_a_directory_that_does_not_exist_says_so() {
        let terminals = Terminals::new(TermConfig::default());
        let err = terminals
            .open(spec(Path::new("/no/such/place/at/all"), false))
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(err.contains("no such directory"), "{err}");
    }
}
