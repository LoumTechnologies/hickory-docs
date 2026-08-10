//! Rendering lineage for agent-authored bytes.
//!
//! `SourceOrigin::Agent { session, turn }` (`hick-flow`) says *which session
//! and turn* produced a byte, and deliberately says nothing about *who*.
//! Authorship composes instead: lineage maps the byte to a document span, and
//! `git blame` on that span gives the commit author — anchored in a commit,
//! signable, and not asserted by whoever happened to run `promote`.
//!
//! Two things here are load-bearing and are why this is a module rather than
//! three lines in `main.rs`:
//!
//! 1. **Unresolvable lineage is a designed outcome, not an error.** Sessions
//!    are per-author and may be private, gitignored, or simply absent. A byte
//!    whose session the reader cannot open still reports the session id, the
//!    turn, and the committing author. Nothing here returns `Err` or unwraps.
//! 2. **The uncommitted working tree is the common case, not an edge case.**
//!    `git blame` reports uncommitted lines as `Not Committed Yet`; those
//!    changes are the current user's by definition, so we say so using the
//!    repository's configured identity.
//!
//! Protects `docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Who is accountable for the bytes, derived from git rather than stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Authorship {
    /// `git blame` named a commit author.
    Committed {
        author: String,
        /// Abbreviated commit id.
        commit: String,
    },
    /// The span is not committed yet, so it is the current user's work by
    /// definition. `who` is the git-configured identity, or a plain
    /// description when git has no identity configured.
    Uncommitted { who: String },
    /// No author could be derived. Carries why, and what to do about it.
    Unknown { reason: String },
}

impl std::fmt::Display for Authorship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Authorship::Committed { author, commit } => {
                write!(f, "committed by {author} in {commit}")
            }
            Authorship::Uncommitted { who } => {
                write!(f, "not committed yet — yours ({who})")
            }
            Authorship::Unknown { reason } => write!(f, "author unknown ({reason})"),
        }
    }
}

/// Whether the reasoning behind the bytes can be read from here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reasoning {
    /// The session file exists and records the turn.
    Available { path: String },
    /// The session cannot be read, or does not record that turn. Carries the
    /// reason so the reader can tell "private" from "typo" from "evicted".
    Unavailable { reason: String },
}

impl std::fmt::Display for Reasoning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reasoning::Available { path } => write!(f, "reasoning in {path}"),
            Reasoning::Unavailable { reason } => {
                write!(f, "reasoning not available to you ({reason})")
            }
        }
    }
}

/// One rendered agent-origin byte range.
#[derive(Debug, Clone)]
pub struct AgentLineage {
    /// The generated output file the bytes live in.
    pub output: String,
    /// 1-based line in that output.
    pub line: usize,
    pub session: String,
    pub turn: usize,
    pub authorship: Authorship,
    pub reasoning: Reasoning,
}

impl std::fmt::Display for AgentLineage {
    /// `foo.rs:42 ← session abc123 turn 7 · committed by … · reasoning …`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} ← session {} turn {} · {} · {}",
            self.output, self.line, self.session, self.turn, self.authorship, self.reasoning
        )
    }
}

/// 1-based line number of `byte` within `content`.
///
/// Offsets past the end clamp to the last line rather than panicking: a
/// provenance map is data, and bad data must not take down a read-only
/// reporting command.
pub fn line_of(content: &str, byte: usize) -> usize {
    let upto = byte.min(content.len());
    content[..upto].bytes().filter(|b| *b == b'\n').count() + 1
}

// ---------------------------------------------------------------------------
// Authorship, derived from git
// ---------------------------------------------------------------------------

fn git(dir: &Path, args: &[&str]) -> Option<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()
}

/// The repository's configured identity, for the uncommitted case.
fn configured_identity(dir: &Path) -> String {
    let value = |key: &str| -> Option<String> {
        let out = git(dir, &["config", "--get", key])?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!s.is_empty()).then_some(s)
    };
    match (value("user.name"), value("user.email")) {
        (Some(name), Some(email)) => format!("{name} <{email}>"),
        (Some(name), None) => name,
        (None, Some(email)) => email,
        (None, None) => "no git identity configured; set user.name and user.email".to_string(),
    }
}

/// Derive authorship of `file` line `line` from `git blame`.
///
/// Never fails: every path git can take — missing binary, not a repository,
/// untracked file, uncommitted line — resolves to a variant that still says
/// something useful.
pub fn blame(repo_dir: &Path, file: &Path, line: usize) -> Authorship {
    let line_arg = format!("{line},{line}");
    let file_arg = file.to_string_lossy().to_string();
    let Some(out) = git(
        repo_dir,
        &["blame", "-L", &line_arg, "--porcelain", "--", &file_arg],
    ) else {
        return Authorship::Unknown {
            reason: "git is not on PATH; install git to see who committed this span".to_string(),
        };
    };

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let detail = stderr.lines().next().unwrap_or("git blame failed").trim();
        // An untracked (or newly added) file has no blame history at all, but
        // it does exist on disk — that is uncommitted work, which is the
        // current user's by definition.
        if repo_dir.join(file).exists() {
            return Authorship::Uncommitted {
                who: configured_identity(repo_dir),
            };
        }
        return Authorship::Unknown {
            reason: format!("git blame {file_arg}:{line} failed: {detail}"),
        };
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut lines = stdout.lines();
    let header = lines.next().unwrap_or_default();
    let sha = header.split_whitespace().next().unwrap_or_default();
    // git spells "this line is not in any commit" as an all-zero sha.
    if sha.chars().all(|c| c == '0') {
        return Authorship::Uncommitted {
            who: configured_identity(repo_dir),
        };
    }
    let mut author = None;
    let mut mail = None;
    for l in lines {
        if let Some(rest) = l.strip_prefix("author ") {
            author = Some(rest.trim().to_string());
        } else if let Some(rest) = l.strip_prefix("author-mail ") {
            mail = Some(rest.trim().trim_matches(['<', '>']).to_string());
        }
        if l.starts_with('\t') {
            break;
        }
    }
    let author = match (author, mail) {
        (Some(a), Some(m)) if !m.is_empty() => format!("{a} <{m}>"),
        (Some(a), _) => a,
        (None, Some(m)) => m,
        (None, None) => "unnamed author".to_string(),
    };
    Authorship::Committed {
        author,
        commit: sha.chars().take(8).collect(),
    }
}

// ---------------------------------------------------------------------------
// Reasoning, resolved from the session store
// ---------------------------------------------------------------------------

/// Conventional location of a session: `<project_dir>/sessions/<id>.hick`,
/// matching `hickory_agent::session_file_path`.
pub fn session_path(project_dir: &Path, session: &str) -> PathBuf {
    project_dir.join("sessions").join(format!("{session}.hick"))
}

/// Can this reader open the reasoning behind `session` turn `turn`?
///
/// Returns [`Reasoning::Unavailable`] — never an error — when the session is
/// missing, unreadable, unparseable, or does not record that turn. Sessions
/// are per-author and may be private; that is a designed outcome.
pub fn resolve_reasoning(project_dir: &Path, session: &str, turn: usize) -> Reasoning {
    let path = session_path(project_dir, session);
    let shown = path.display().to_string();
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Reasoning::Unavailable {
                reason: format!(
                    "no session at {shown}; sessions are per-author and may be private, \
                     gitignored, or held by whoever ran the agent"
                ),
            };
        }
        Err(e) => {
            return Reasoning::Unavailable {
                reason: format!("{shown} could not be read: {e}"),
            };
        }
    };
    let doc = match hick_lang::parse_session(&source) {
        Ok(d) => d,
        Err(e) => {
            return Reasoning::Unavailable {
                reason: format!("{shown} is not a readable hick:session document: {e}"),
            };
        }
    };
    let turns = doc
        .nodes
        .iter()
        .filter(|n| matches!(n, hick_lang::SessionNode::Assistant { .. }))
        .count();
    if turn >= turns {
        return Reasoning::Unavailable {
            reason: format!(
                "{shown} records {turns} turn(s), so turn {turn} is not in it; \
                 the session may have been truncated or replaced"
            ),
        };
    }
    Reasoning::Available { path: shown }
}

/// Assemble the report for one agent-origin byte range.
///
/// `doc_span` is the document span the bytes map to, when lineage recorded a
/// byte-precise one; without it there is nothing to blame and authorship is
/// reported as unknown rather than guessed.
pub fn describe(
    project_dir: &Path,
    output: &str,
    line: usize,
    session: &str,
    turn: usize,
    doc_span: Option<(&Path, usize)>,
) -> AgentLineage {
    let authorship = match doc_span {
        Some((doc, doc_line)) => blame(project_dir, doc, doc_line),
        None => Authorship::Unknown {
            reason: "no byte-precise document span was recorded for these bytes, \
                     so there is nothing to git blame"
                .to_string(),
        },
    };
    AgentLineage {
        output: output.to_string(),
        line,
        session: session.to_string(),
        turn,
        authorship,
        reasoning: resolve_reasoning(project_dir, session, turn),
    }
}

#[cfg(test)]
mod tests {
    // Protects docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md
    use super::*;

    fn temp_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = dir.path();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.name", "Test Author"],
            vec!["config", "user.email", "test@example.com"],
        ] {
            let out = git(p, &args).expect("git available");
            assert!(out.status.success(), "git {args:?} failed");
        }
        dir
    }

    #[test]
    fn line_of_is_one_based_and_clamps() {
        let c = "a\nb\nc";
        assert_eq!(line_of(c, 0), 1);
        assert_eq!(line_of(c, 2), 2);
        assert_eq!(line_of(c, 4), 3);
        // Past the end: clamps rather than panicking.
        assert_eq!(line_of(c, 9_999), 3);
    }

    #[test]
    fn missing_session_still_reports_id_and_turn() {
        let dir = tempfile::tempdir().unwrap();
        let l = describe(dir.path(), "foo.rs", 42, "abc123", 7, None);
        let rendered = l.to_string();
        assert!(
            rendered.starts_with("foo.rs:42 ← session abc123 turn 7 · "),
            "{rendered}"
        );
        assert!(
            rendered.contains("reasoning not available to you"),
            "{rendered}"
        );
        // The reason names the path it looked for — actionable, not a shrug.
        assert!(rendered.contains("sessions/abc123.hick"), "{rendered}");
    }

    #[test]
    fn unreadable_session_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            resolve_reasoning(dir.path(), "private", 0),
            Reasoning::Unavailable { .. }
        ));
    }

    #[test]
    fn readable_session_resolves_the_turn() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path(dir.path(), "s1");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:session xmlns:hick=\"http://www.hickorydocs.com/1.0\" start=\"2026-01-01T00:00:00Z\">\n\
             <hick:user>do the thing</hick:user>\n\
             <hick:assistant>turn zero</hick:assistant>\n\
             <hick:assistant>turn one</hick:assistant>\n\
             </hick:session>\n",
        )
        .unwrap();
        assert!(matches!(
            resolve_reasoning(dir.path(), "s1", 1),
            Reasoning::Available { .. }
        ));
        // A turn the session does not record degrades, and says why.
        match resolve_reasoning(dir.path(), "s1", 9) {
            Reasoning::Unavailable { reason } => assert!(reason.contains("turn 9"), "{reason}"),
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[test]
    fn blame_names_the_commit_author() {
        let dir = temp_repo();
        std::fs::write(dir.path().join("doc.hick"), "line one\nline two\n").unwrap();
        assert!(
            git(dir.path(), &["add", "doc.hick"])
                .unwrap()
                .status
                .success()
        );
        assert!(
            git(dir.path(), &["commit", "-q", "-m", "add doc"])
                .unwrap()
                .status
                .success()
        );
        match blame(dir.path(), Path::new("doc.hick"), 2) {
            Authorship::Committed { author, commit } => {
                assert!(author.contains("Test Author"), "{author}");
                assert!(!commit.is_empty());
            }
            other => panic!("expected Committed, got {other:?}"),
        }
    }

    #[test]
    fn uncommitted_work_is_attributed_to_the_current_user() {
        let dir = temp_repo();
        std::fs::write(dir.path().join("doc.hick"), "line one\n").unwrap();
        assert!(
            git(dir.path(), &["add", "doc.hick"])
                .unwrap()
                .status
                .success()
        );
        assert!(
            git(dir.path(), &["commit", "-q", "-m", "add doc"])
                .unwrap()
                .status
                .success()
        );
        std::fs::write(dir.path().join("doc.hick"), "line one\nbrand new line\n").unwrap();
        match blame(dir.path(), Path::new("doc.hick"), 2) {
            Authorship::Uncommitted { who } => assert!(who.contains("Test Author"), "{who}"),
            other => panic!("expected Uncommitted, got {other:?}"),
        }
        // An untracked file is the same story.
        std::fs::write(dir.path().join("new.hick"), "fresh\n").unwrap();
        assert!(matches!(
            blame(dir.path(), Path::new("new.hick"), 1),
            Authorship::Uncommitted { .. }
        ));
    }

    #[test]
    fn outside_a_repository_authorship_is_unknown_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("doc.hick"), "x\n").unwrap();
        // The file exists but there is no repository: git blame fails and we
        // fall back to "uncommitted", which is still true and still useful.
        let a = blame(dir.path(), Path::new("doc.hick"), 1);
        assert!(matches!(a, Authorship::Uncommitted { .. }), "{a:?}");
        // A path that does not exist at all cannot be attributed.
        let a = blame(dir.path(), Path::new("nope.hick"), 1);
        assert!(matches!(a, Authorship::Unknown { .. }), "{a:?}");
    }
}
