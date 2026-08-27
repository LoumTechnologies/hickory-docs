//! One terminal session: a PTY, what it has said, and what it is doing.
//!
//! A session outlives the pane showing it. That is the whole point — closing
//! a tab must not kill a build, and reopening it must not show an empty
//! screen. So the PTY, the scrollback, and the classification all live here,
//! on the server, and a client attaches to a session rather than owning one.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context as _, Result};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::classify::{SessionState, Signals, classify};
use crate::config::TermConfig;
use crate::git::{self, GitFacts};
use crate::prompt::question_in;
use crate::screen::Screen;

/// How a session was asked to exist.
#[derive(Debug, Clone)]
pub struct SessionSpec {
    /// What the tab says. Sessions are named after tasks, not numbered, so
    /// "the one that needs me" is a thing you can point at.
    pub title: String,
    /// Where it runs.
    pub cwd: PathBuf,
    /// The command. Empty means the configured shell.
    pub argv: Vec<String>,
    /// A support process — a dev server, a watcher, a log tail. Monitors are
    /// shown in the dock, never claim attention, and never take focus.
    pub monitor: bool,
}

/// A question a session is waiting on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub question: String,
    /// What the asker will accept. Empty for a guessed prompt: we can see
    /// that something is being asked, not what the answers are.
    pub choices: Vec<Choice>,
    pub source: PromptSource,
}

/// One answer a prompt will take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// What the button says.
    pub label: String,
    /// What gets written to the PTY when it is pressed.
    pub send: String,
    /// Whether taking this choice does something that cannot be undone.
    /// Turbo never picks one of these.
    pub destructive: bool,
}

/// Where a prompt came from, which decides how much it may be trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromptSource {
    /// The program declared it, with its own choices. Structural.
    Declared,
    /// We recognised the shape of a question on screen. A guess.
    Guessed,
}

/// A session as the API describes it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub cwd: String,
    pub monitor: bool,
    pub state: String,
    /// When the session entered this state, epoch millis.
    pub since_ms: u64,
    pub branch: Option<String>,
    pub dirty: bool,
    /// The last line it printed — what a folded row shows.
    pub preview: String,
    /// Whether [`SessionSummary::cwd`] came from the shell (OSC 7) or is the
    /// directory the session was started in.
    ///
    /// Surfaced rather than hidden because the two are not equally
    /// trustworthy: a started-in directory is stale the moment somebody `cd`s,
    /// and a reader deserves to know which one they are looking at.
    #[serde(default)]
    pub cwd_is_live: bool,
    pub prompt: Option<Prompt>,
    pub exit_code: Option<i32>,
}

/// A live session.
pub struct Session {
    pub id: String,
    pub spec: SessionSpec,
    screen: Mutex<Screen>,
    output: broadcast::Sender<Arc<Vec<u8>>>,
    /// Commands the shell reported, for whoever is recording them.
    typed: broadcast::Sender<crate::command::TypedCommand>,
    writer: Mutex<Box<dyn Write + Send>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    /// Read only by `foreground_child`, which is Unix-only — a ConPTY has no
    /// process group to compare this against. Kept rather than `cfg`-ed away
    /// because it is the child's identity, not a Unix detail, and the next
    /// thing that wants it will want it on both platforms.
    #[cfg_attr(windows, allow(dead_code))]
    child_pid: Option<u32>,
    last_output: Mutex<Instant>,
    exit: Mutex<Option<i32>>,
    /// A prompt the program itself declared. Guessed prompts are not stored:
    /// they are re-derived from the screen, so they clear themselves when the
    /// question scrolls away.
    declared: Mutex<Option<Prompt>>,
    /// The state last reported, and when it began — so "oldest waiter first"
    /// means what it says.
    state_since: Mutex<(SessionState, u64)>,
    git: Mutex<Option<GitFacts>>,
    /// The generated shell startup files, held so they outlive the shell that
    /// is reading them and go when the session goes.
    _integration: Option<crate::shell_integration::Integration>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl Session {
    /// Start a session: open a PTY, spawn the command, and begin reading.
    pub fn spawn(id: String, spec: SessionSpec, config: &TermConfig) -> Result<Arc<Session>> {
        let pty = native_pty_system();
        let pair = pty
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to open a pseudo-terminal")?;

        let mut cmd = if spec.argv.is_empty() {
            CommandBuilder::new(&config.shell)
        } else {
            let mut c = CommandBuilder::new(&spec.argv[0]);
            for arg in &spec.argv[1..] {
                c.arg(arg);
            }
            c
        };
        cmd.cwd(&spec.cwd);
        // Programs that ask what they are talking to should get a truthful
        // answer; xterm.js is xterm-256color.
        cmd.env("TERM", "xterm-256color");
        // And a truthful answer to "which terminal", which is not the same as
        // no answer.
        //
        // Whatever launched the app is in this process's environment, so a
        // session started from Terminal.app inherited `TERM_PROGRAM=
        // Apple_Terminal` and the shell then ran `/etc/zshrc_Apple_Terminal` —
        // OSC 7 by accident, plus Terminal.app's session-history machinery,
        // and only when the app happened to be launched from a terminal. From
        // the Dock it got neither. Behaviour that depends on how the app was
        // started is the kind nobody can reproduce.
        //
        // Naming ourselves fixes that in the honest direction: it is true, it
        // stops `/etc/zshrc_$TERM_PROGRAM` matching somebody else's file, and
        // it is how a user's own config can tell it is us. `shell_integration`
        // is what provides OSC 7 now, on purpose rather than by inheritance.
        cmd.env("TERM_PROGRAM", "HickoryDocs");

        // Teach the shell to report where it is, but only when the session IS
        // a shell. A session given its own `argv` is running somebody's
        // command, and rewriting the startup of `zsh -f -i` or `cargo watch`
        // would be changing what they asked for.
        //
        // `TERM_PROGRAM` is deliberately left alone — see `shell_integration`
        // for what claiming to be Terminal.app would actually do.
        let integration = if spec.argv.is_empty() && config.integrate_shell {
            crate::shell_integration::Integration::install(&config.shell)
        } else {
            None
        };
        if let Some(integration) = &integration {
            for (key, value) in integration.env() {
                cmd.env(key, value);
            }
            for arg in integration.args() {
                cmd.arg(arg);
            }
        }

        let child = pair.slave.spawn_command(cmd).with_context(|| {
            let what = spec
                .argv
                .first()
                .cloned()
                .unwrap_or_else(|| config.shell.clone());
            format!(
                "could not start '{what}' in {}. Check that the program exists and is \
                 executable, or set HICKORY_SHELL to a shell you have.",
                spec.cwd.display()
            )
        })?;
        let child_pid = child.process_id();
        // The slave side must be dropped or the PTY never reports EOF.
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .context("failed to read from the pseudo-terminal")?;
        let writer = pair
            .master
            .take_writer()
            .context("failed to write to the pseudo-terminal")?;

        let (output, _) = broadcast::channel(1024);
        // Small on purpose: a listener that has fallen this far behind on
        // typed commands has missed some, and `Lagged` telling it so is what
        // lets the anchor suspend rather than write a cell with a hole in it.
        let (typed, _) = broadcast::channel(256);
        let session = Arc::new(Session {
            id,
            _integration: integration,
            git: Mutex::new(git::facts(&spec.cwd)),
            screen: Mutex::new(Screen::new(24, 80, config.scrollback_lines)),
            output,
            typed,
            writer: Mutex::new(writer),
            master: Mutex::new(pair.master),
            child: Mutex::new(child),
            child_pid,
            last_output: Mutex::new(Instant::now()),
            exit: Mutex::new(None),
            declared: Mutex::new(None),
            state_since: Mutex::new((SessionState::Working, now_ms())),
            spec,
        });

        session.clone().read_forever(reader);
        Ok(session)
    }

    /// Pump the PTY into the scrollback and out to whoever is attached.
    ///
    /// A blocking read on its own thread, not a tokio task: this is a real
    /// file descriptor whose read cannot be cancelled, and parking a runtime
    /// worker on it would starve the server under a handful of sessions.
    fn read_forever(self: Arc<Self>, mut reader: Box<dyn Read + Send>) {
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let bytes = Arc::new(buf[..n].to_vec());
                        let mut typed = Vec::new();
                        if let Ok(mut screen) = self.screen.lock() {
                            screen.feed(&bytes[..]);
                            typed = screen.take_typed();
                        }
                        // Outside the screen lock: a subscriber that is slow
                        // must not hold up the PTY reader, and a session
                        // nobody is recording has no subscribers at all.
                        for command in typed {
                            let _ = self.typed.send(command);
                        }
                        if let Ok(mut last) = self.last_output.lock() {
                            *last = Instant::now();
                        }
                        // No receivers is the normal case: a session nobody is
                        // watching still runs, and its output still lands in
                        // the scrollback above.
                        let _ = self.output.send(bytes);
                    }
                }
            }
        });
    }

    /// Every command the shell reports from now on.
    ///
    /// Nothing is replayed: a recording starts when a person anchors the
    /// terminal, and commands typed before that belong to the part of the
    /// session the document does not claim.
    pub fn typed_commands(&self) -> broadcast::Receiver<crate::command::TypedCommand> {
        self.typed.subscribe()
    }

    /// Whether this session's shell reports what it runs.
    ///
    /// False for a shell hick has no hook for (fish, nu, a bare `sh`) and for
    /// `HICKORY_SHELL_INTEGRATION=0`. A terminal that cannot report its
    /// commands cannot be anchored to a document, and saying so is the whole
    /// of "never anchor silently".
    pub fn reports_commands(&self) -> bool {
        self._integration.is_some()
    }

    /// Attach: everything said so far, then everything said from now on.
    ///
    /// Taken together under the screen lock, so nothing is missed and nothing
    /// arrives twice in the seam between replay and live output.
    pub fn attach(&self) -> (Vec<u8>, broadcast::Receiver<Arc<Vec<u8>>>) {
        let screen = self.screen.lock().expect("screen lock");
        let receiver = self.output.subscribe();
        (screen.replay().to_vec(), receiver)
    }

    /// Say something IN the terminal without saying it TO the shell.
    ///
    /// The bytes go to the screen model and to whoever is watching, and not
    /// to the PTY — so a notice appears where the person is looking without
    /// the shell ever seeing it as input. That distinction is the whole
    /// reason this is not `write`: sending "recording paused" to the shell
    /// would run it as a command.
    pub fn inject(&self, bytes: &[u8]) {
        let bytes = Arc::new(bytes.to_vec());
        if let Ok(mut screen) = self.screen.lock() {
            screen.feed(&bytes[..]);
            // Whatever this painted is ours, and must never be mistaken for
            // something the shell reported.
            let _ = screen.take_typed();
        }
        let _ = self.output.send(bytes);
    }

    /// Type into the session.
    pub fn write(&self, bytes: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock().expect("writer lock");
        writer.write_all(bytes).context("the terminal is gone")?;
        writer.flush().context("the terminal is gone")?;
        Ok(())
    }

    /// Follow the pane's size. Both halves matter: the child needs the
    /// winsize to lay out, and the screen model needs it to agree about what
    /// the last line is.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        self.master
            .lock()
            .expect("pty lock")
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to resize the terminal")?;
        if let Ok(mut screen) = self.screen.lock() {
            screen.resize(rows, cols);
        }
        Ok(())
    }

    /// Interrupt whatever is running, the way ^C does.
    pub fn interrupt(&self) -> Result<()> {
        self.write(b"\x03")
    }

    /// End the session for good.
    pub fn kill(&self) -> Result<()> {
        let mut child = self.child.lock().expect("child lock");
        child
            .kill()
            .context("failed to stop the terminal's process")?;
        Ok(())
    }

    /// The session's visible screen as text — a peek, without attaching.
    /// See [`crate::screen::Screen::contents`].
    pub fn screen_text(&self) -> String {
        self.screen.lock().expect("screen lock").contents()
    }

    /// Whether the program has taken over the whole screen.
    pub fn alternate_screen(&self) -> bool {
        self.screen.lock().expect("screen lock").alternate_screen()
    }

    /// Record a prompt the program declared for itself.
    ///
    /// The structural path: a caller that speaks the program's protocol —
    /// `hickory-agent`, today — knows the question and the choices, so the
    /// attention card can offer buttons rather than a blind input line.
    pub fn declare_prompt(&self, prompt: Option<Prompt>) {
        *self.declared.lock().expect("prompt lock") = prompt;
    }

    /// The prompt this session is waiting on, declared or guessed.
    pub fn prompt(&self) -> Option<Prompt> {
        if let Some(declared) = self.declared.lock().expect("prompt lock").clone() {
            return Some(declared);
        }
        if self.exit_code().is_some() {
            return None;
        }
        // The whole visible screen, not just its last line: a drawn menu puts
        // its footer under its choices, so the question is several lines up.
        let contents = self.screen.lock().expect("screen lock").contents();
        question_in(&contents).map(|question| Prompt {
            question,
            choices: Vec::new(),
            source: PromptSource::Guessed,
        })
    }

    /// The child's exit code, once it has one.
    fn exit_code(&self) -> Option<i32> {
        if let Some(code) = *self.exit.lock().expect("exit lock") {
            return Some(code);
        }
        let status = self.child.lock().expect("child lock").try_wait().ok()?;
        let code = status.map(|s| s.exit_code() as i32)?;
        *self.exit.lock().expect("exit lock") = Some(code);
        Some(code)
    }

    /// Whether something other than the session's own shell holds the
    /// terminal. `None` where the platform will not say.
    ///
    /// Windows is one of those platforms, and this is the `None` the signature
    /// already promised rather than a gap opened here. A ConPTY has no process
    /// group to ask about, so `portable-pty` offers `process_group_leader` on
    /// Unix only — calling it unconditionally is what stopped the whole
    /// workspace compiling for Windows, unnoticed, because no job built that
    /// target until CI grew one.
    ///
    /// The cost is that `classify` cannot tell "a command is running" from "the
    /// shell is idle" on Windows, so a session there leans on its other signals
    /// (output, exit code, a declared prompt). Worth knowing before trusting the
    /// attention queue on that platform.
    #[cfg(windows)]
    fn foreground_child(&self) -> Option<bool> {
        None
    }

    /// Whether something other than the session's own shell holds the
    /// terminal. `None` where the platform will not say.
    #[cfg(not(windows))]
    fn foreground_child(&self) -> Option<bool> {
        let leader = self
            .master
            .lock()
            .expect("pty lock")
            .process_group_leader()?;
        let shell = self.child_pid?;
        Some(leader as i64 != shell as i64)
    }

    /// Re-read the git facts. Called when a session settles, not on a timer:
    /// running `git status` in a loop against a big repository is how a
    /// terminal starts costing more than the work inside it.
    pub fn refresh_git(&self) {
        let facts = git::facts(&self.spec.cwd);
        *self.git.lock().expect("git lock") = facts;
    }

    /// What this session is doing, and everything the API says about it.
    pub fn summary(&self) -> SessionSummary {
        let exit = self.exit_code();
        let quiet_ms = self
            .last_output
            .lock()
            .map(|last| last.elapsed().as_millis() as u64)
            .unwrap_or(0);
        let prompt = self.prompt();
        let state = classify(Signals {
            exit,
            prompt_pending: prompt.is_some(),
            foreground_child: self.foreground_child(),
            quiet_ms,
        });

        // One scoped lock, taken once. The guard must be gone before
        // refresh_git, and before anything else reads the clock: std mutexes
        // are not reentrant, so a second lock inside this scope is a deadlock
        // rather than an error — the whole server hanging on its first
        // unchanged state.
        let (since_ms, changed) = {
            let mut since = self.state_since.lock().expect("state lock");
            let changed = since.0 != state;
            if changed {
                *since = (state, now_ms());
            }
            (since.1, changed)
        };
        // A session that just stopped running may have left changes behind;
        // that is exactly when the queue needs to know, and the only moment
        // worth paying for `git status`.
        if changed && matches!(state, SessionState::Finished | SessionState::Failed) {
            self.refresh_git();
        }
        let git = self
            .git
            .lock()
            .expect("git lock")
            .clone()
            .unwrap_or_default();

        // One lock for both, because taking the screen lock twice in a row is
        // the kind of thing that becomes a deadlock when someone later moves a
        // line between them.
        let (preview, live_cwd) = {
            let screen = self.screen.lock().expect("screen lock");
            (screen.preview(), screen.cwd().map(str::to_string))
        };
        let cwd_is_live = live_cwd.is_some();

        SessionSummary {
            id: self.id.clone(),
            title: self.spec.title.clone(),
            // Where the shell says it is, falling back to where it was
            // started. A `cd` moves a session out of the folder you are
            // looking at, and a tree that kept showing it there would be
            // pointing at the wrong place with complete confidence.
            cwd: live_cwd.unwrap_or_else(|| self.spec.cwd.display().to_string()),
            cwd_is_live,
            monitor: self.spec.monitor,
            state: state.as_str().to_string(),
            since_ms,
            branch: git.branch,
            dirty: git.dirty,
            preview,
            prompt,
            exit_code: exit,
        }
    }

    /// The state alone, for callers that only want to rank.
    pub fn state(&self) -> SessionState {
        let exit = self.exit_code();
        let quiet_ms = self
            .last_output
            .lock()
            .map(|last| last.elapsed().as_millis() as u64)
            .unwrap_or(0);
        classify(Signals {
            exit,
            prompt_pending: self.prompt().is_some(),
            foreground_child: self.foreground_child(),
            quiet_ms,
        })
    }
}
