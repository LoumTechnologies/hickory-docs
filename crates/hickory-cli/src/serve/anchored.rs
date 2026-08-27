//! Terminals that write the document.
//!
//! The **persistent** binding from
//! `docs/specs/freeform/a-terminal-that-writes-the-document.md`: a terminal
//! anchored to a container in a document, whose typed lines become that
//! cell. The other two bindings need nothing here — an ephemeral terminal
//! writes nothing ever, and a watching one has no input at all.
//!
//! What lives here is only the wiring. The two decisions are elsewhere and
//! pure, deliberately: [`hick_term::anchor::Recording`] decides whether a
//! reported line may be written, and [`crate::anchor::append_command`]
//! decides where it lands. This module holds a table and a task.
//!
//! ## Never anchor silently
//!
//! Two things follow, and both are refusals rather than behaviours.
//!
//! A terminal that is writing into a document says which document and which
//! container, always — the difference between "this disappears" and "this is
//! being committed" is the most important thing on the screen, so the state
//! is readable at any moment rather than announced once.
//!
//! And a shell hick has no hook for **cannot be anchored at all**. Recording
//! depends on the shell reporting what it runs; a fish or `nu` session would
//! record nothing while looking anchored, which is the exact failure the
//! whole design exists to prevent. It is refused by name.

use std::collections::HashMap;

use hick_term::anchor::{Decision, ForeignInput, Recording, Suspension};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// One terminal's binding to a document.
#[derive(Debug, Clone, Serialize)]
pub struct Anchor {
    /// The document id, as the client names it.
    pub doc: String,
    /// The container the cell names. This IS the anchor: several
    /// `hick:exec` blocks sharing a container is already how the language
    /// says "commands that share state".
    pub container: String,
    /// Why recording is stopped, when it is.
    pub suspended: Option<String>,
    /// What is reading the keys instead of the shell, while something is.
    ///
    /// Separate from `suspended` because it is not one: nothing has to be
    /// resumed, and it clears itself when the shell gets the terminal back.
    /// Refreshed when a person types, so it can lag by one keystroke after
    /// they quit a REPL — the terminal's own notice is the timely half, and
    /// this is the standing one.
    pub foreign: Option<String>,
    /// How many lines have gone into the document since anchoring.
    pub recorded: usize,
}

#[derive(Debug, Deserialize)]
pub struct AnchorBody {
    pub doc: String,
    pub container: String,
}

/// Per-session recording state, held beside the terminal registry.
#[derive(Default)]
pub struct Anchors {
    live: Mutex<HashMap<String, Live>>,
}

struct Live {
    doc: String,
    container: String,
    recording: Recording,
    recorded: usize,
    /// Set when the next line must open a new cell rather than grow the last
    /// one — after a resume, because the shell then holds state the document
    /// does not describe.
    fresh_cell: bool,
    /// Whether the "a program is reading these keys" notice has been said for
    /// the program currently holding the terminal. Cleared when the shell
    /// gets it back, so the next one says it again — and so a person editing
    /// a long file in `vi` is told once rather than on every keystroke.
    warned_about_child: bool,
    /// The standing "something else is reading the keys" note, for the bar.
    foreign: Option<String>,
    /// Dropped on unanchor, which stops the task reading typed commands.
    _stop: tokio::sync::oneshot::Sender<()>,
}

impl Anchors {
    /// What this session is bound to, if anything.
    pub async fn get(&self, session: &str) -> Option<Anchor> {
        let live = self.live.lock().await;
        live.get(session).map(|held| Anchor {
            doc: held.doc.clone(),
            container: held.container.clone(),
            suspended: held.recording.suspended().map(Suspension::message),
            foreign: held.foreign.clone(),
            recorded: held.recorded,
        })
    }

    /// Every anchored session, so the window can say which terminals are
    /// writing without asking one at a time.
    pub async fn all(&self) -> HashMap<String, Anchor> {
        let live = self.live.lock().await;
        live.iter()
            .map(|(id, held)| {
                (
                    id.clone(),
                    Anchor {
                        doc: held.doc.clone(),
                        container: held.container.clone(),
                        suspended: held.recording.suspended().map(Suspension::message),
                        foreign: held.foreign.clone(),
                        recorded: held.recorded,
                    },
                )
            })
            .collect()
    }

    async fn remove(&self, session: &str) -> bool {
        self.live.lock().await.remove(session).is_some()
    }
}

/// Decide what one reported line does, and grow the document if it earns it.
///
/// Returns the message to show in the terminal, when there is one. Split out
/// from the task so the whole path — decision, document edit, what a person
/// is told — is reachable from a test without a PTY.
pub async fn observe_line(
    state: &super::LocalState,
    session: &str,
    number: u64,
    text: &str,
) -> Option<String> {
    let mut live = state.anchors.live.lock().await;
    let held = live.get_mut(session)?;
    let decision = held.recording.observe(number, text);
    match decision {
        Decision::Record(line) => {
            let doc = held.doc.clone();
            let container = held.container.clone();
            let fresh = std::mem::take(&mut held.fresh_cell);
            held.recorded += 1;
            // The lock is not held across the document write: a slow disk
            // must not stop the PTY reader, and nothing else may touch this
            // session's state in between because only one task reads it.
            drop(live);
            if let Err(error) = write_line(state, &doc, &container, &line, fresh).await {
                return Some(format!(
                    "recording paused — the document could not be written\n   {error:#}"
                ));
            }
            None
        }
        Decision::Suspend(why) => Some(why.message()),
        Decision::Ignore => None,
    }
}

/// Put one line into the document, through the room if one is live.
async fn write_line(
    state: &super::LocalState,
    doc: &str,
    container: &str,
    line: &str,
    fresh_cell: bool,
) -> anyhow::Result<()> {
    // Read through the room when there is one: a collaborator's unsaved
    // keystrokes live in the CRDT for up to the persist debounce, and
    // appending to the file's text instead would drop them.
    let current = match state.rooms.get(doc).await {
        Some(room) => room.text().await,
        None => state
            .read_source(doc)
            .map_err(|e| anyhow::anyhow!("{}", e.detail()))?,
    };
    let next = crate::anchor::append_command(&current, container, line, fresh_cell);
    // Refuse to write a document that would no longer parse rather than
    // leaving somebody with a file their own tool cannot open.
    hick_lang::parse(&next)
        .map_err(|e| anyhow::anyhow!("appending this line would break the document: {e}"))?;
    state
        .write_source(doc, &next)
        .map_err(|e| anyhow::anyhow!("{}", e.detail()))?;
    state.rooms.apply_external_source(doc, &next).await;
    Ok(())
}

/// Bind a session to a container in a document, and start recording.
pub async fn anchor(
    state: &super::LocalState,
    session_id: &str,
    body: AnchorBody,
) -> anyhow::Result<Anchor> {
    let session = state
        .terminals
        .get(session_id)
        .ok_or_else(|| anyhow::anyhow!("no terminal {session_id} in this session"))?;

    // Never anchor silently: a shell that cannot report what it runs would
    // record nothing while the window said it was recording.
    if !session.reports_commands() {
        anyhow::bail!(
            "this terminal cannot be anchored: recording needs a shell hook, and hick has one \
             for bash and zsh only.\n\
             The terminal still works — nothing about using it changes. To anchor one, open a \
             terminal running bash or zsh (`HICKORY_SHELL` chooses), and check \
             HICKORY_SHELL_INTEGRATION is not set to 0."
        );
    }
    if state.index.absolute(&body.doc).is_none() {
        anyhow::bail!("no document {} in this session", body.doc);
    }

    let (stop, mut stopped) = tokio::sync::oneshot::channel();
    let mut typed = session.typed_commands();
    {
        let mut live = state.anchors.live.lock().await;
        live.insert(
            session_id.to_string(),
            Live {
                doc: body.doc.clone(),
                container: body.container.clone(),
                recording: Recording::new(),
                recorded: 0,
                // The first line grows whatever cell this container already
                // has, which is what makes anchoring twice in one sitting
                // continue rather than fragment.
                fresh_cell: false,
                warned_about_child: false,
                foreign: None,
                _stop: stop,
            },
        );
    }

    let owned = state.clone();
    let id = session_id.to_string();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stopped => return,
                next = typed.recv() => match next {
                    Ok(command) => {
                        if let Some(message) =
                            observe_line(&owned, &id, command.number, &command.text).await
                        {
                            notify(&owned, &id, &message).await;
                        }
                    }
                    // Lagged means commands were missed, and a cell with a
                    // hole in it claims a run that does not reproduce. There
                    // is no honest recovery: stop.
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                        let message = format!(
                            "recording paused — {missed} command(s) went by faster than they \
                             could be written\n   the shell ran them; the document did not \
                             record them"
                        );
                        notify(&owned, &id, &message).await;
                        owned.anchors.remove(&id).await;
                        return;
                    }
                    Err(_) => return,
                },
            }
        }
    });

    state
        .anchors
        .get(session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("the anchor went away while it was being made"))
}

/// Tell a person their keystrokes are going somewhere the document cannot
/// see — once per program, not once per keystroke.
///
/// Called from the input path rather than a timer, because the question is
/// not "is a child running" (every ordinary command makes that true for as
/// long as it takes) but "are the keys I am typing right now going to one".
/// A build holding the terminal is a build; a person typing into a REPL is a
/// person whose lines will not appear, and only the second needs saying.
pub async fn note_foreign_input(state: &super::LocalState, session_id: &str) {
    let Some(session) = state.terminals.get(session_id) else {
        return;
    };
    let child = session.input_goes_to_a_child();
    let message = {
        let mut live = state.anchors.live.lock().await;
        let Some(held) = live.get_mut(session_id) else {
            return;
        };
        // Nothing to say while recording is already stopped for a reason a
        // person has been told about — two notices about one silence is how
        // the one that matters stops being read.
        if held.recording.suspended().is_some() {
            return;
        }
        if !child {
            held.warned_about_child = false;
            held.foreign = None;
            return;
        }
        let message = ForeignInput {
            program: session.foreground_program(),
            full_screen: session.alternate_screen(),
        }
        .message();
        held.foreign = Some(message.clone());
        if held.warned_about_child {
            // Said once per program, not once per keystroke: a person
            // editing a long file in `vi` needs telling, not narrating.
            return;
        }
        held.warned_about_child = true;
        message
    };
    notify(state, session_id, &message).await;
}

/// Say something in the terminal itself, at the moment it happens.
///
/// Never in a diff afterwards: a person who does not know recording stopped
/// will assume the cell holds what they did.
async fn notify(state: &super::LocalState, session: &str, message: &str) {
    let Some(live) = state.terminals.get(session) else {
        return;
    };
    // Dim, on its own lines, and prefixed the way the spec writes it. This
    // goes to the SCREEN, not to the shell: it is drawn into the session's
    // own scrollback so it appears where the person is looking.
    let painted = format!("\r\n\x1b[33m⏸ {}\x1b[0m\r\n", message.replace('\n', "\r\n"));
    live.inject(painted.as_bytes());
}

/// Unbind a session. The terminal keeps working; the document stops
/// receiving.
pub async fn unanchor(state: &super::LocalState, session_id: &str) -> bool {
    state.anchors.remove(session_id).await
}

/// Resume after a suspension, which always starts a new cell.
pub async fn resume(state: &super::LocalState, session_id: &str) -> anyhow::Result<Anchor> {
    {
        let mut live = state.anchors.live.lock().await;
        let held = live
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("terminal {session_id} is not anchored"))?;
        if held.recording.resume() {
            held.fresh_cell = true;
        }
    }
    state
        .anchors
        .get(session_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("terminal {session_id} is not anchored"))
}
