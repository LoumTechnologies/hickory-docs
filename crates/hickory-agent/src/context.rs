//! Context provenance: what was in front of the model when it wrote.
//!
//! Derived from the session record alone — never from anything the model
//! says. A session file is a list of inputs (the user's words, a tool's
//! result, a script's observation, a file a tool showed) and writes (`<hick:wrote>`,
//! recorded by the edit tools). For every write, its context is every input
//! that precedes it in the same session. That is the whole derivation: no
//! judgement about which input "caused" which line, because no algorithm can
//! know that; only the fact, checkable by anyone holding the session, that
//! these inputs were present when these lines were produced.
//!
//! Three kinds of provenance live in this product and must not be confused
//! (`docs/specs/freeform/three-provenances.md`): the weave's byte-exact
//! lineage, this, and whatever the author declares with `cites=`.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::tools::hashline::LineIndex;

/// One input that preceded a write in the session.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContextInput {
    /// A file a tool showed the model (`<hick:read>`).
    File {
        path: String,
        commit: Option<String>,
        sha256: String,
        first_line: usize,
        last_line: usize,
        /// Line of the `<hick:read>` element in the session file.
        session_line: usize,
    },
    /// Anything else the model was given: the user's prompt, a script's
    /// observation, a tool's result. Summarized here; the element itself is
    /// in the session at `session_line`, under `id`.
    Conversation {
        /// `user`, `observation`, or `tool-result`.
        element: String,
        id: Option<String>,
        /// Tool name for a tool result, action source for an observation.
        source: Option<String>,
        /// The first non-empty line, clipped — enough to recognise it by.
        summary: String,
        sha256: String,
        lines: usize,
        session_line: usize,
    },
}

/// One write and everything that was in front of the model when it happened.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WriteContext {
    /// The session file, as given to the deriver.
    pub session: String,
    /// Line of the `<hick:wrote>` element in the session file.
    pub session_line: usize,
    /// The file written, as the session names it.
    pub file: String,
    /// The lines written, as they stood right after the edit (1-based,
    /// inclusive), and their hashline hashes.
    pub first_line: usize,
    pub last_line: usize,
    pub hashes: Vec<String>,
    /// Every input before this write, in session order.
    pub inputs: Vec<ContextInput>,
}

/// A write located in the CURRENT text of its file.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResolvedWrite {
    #[serde(flatten)]
    pub write: WriteContext,
    /// Where the written lines are now, if they can still be found as a
    /// contiguous run of lines with the recorded hashes. `None` means the
    /// lines have since changed — the write happened, the context is a fact,
    /// but nothing in the file today is those bytes.
    pub current_lines: Option<(usize, usize)>,
}

/// Derive every write's context from one session source.
pub fn derive_from_session(session_path: &str, source: &str) -> Vec<WriteContext> {
    let Ok(doc) = hick_lang::parse(source) else {
        return Vec::new();
    };
    // The root is the session element; its children are the turns. A file
    // that is not a session (or a partial one with no root yet) is walked at
    // the top level, which is the same thing for our purposes.
    let nodes: Vec<&hick_lang::HickTag> = match doc.tags().find(|t| t.name == "session") {
        Some(root) => root
            .children
            .iter()
            .filter_map(|n| match n {
                hick_lang::HickNode::Tag(t) => Some(t),
                _ => None,
            })
            .collect(),
        None => doc.tags().collect(),
    };
    let mut inputs: Vec<ContextInput> = Vec::new();
    let mut out = Vec::new();
    for tag in nodes {
        match tag.name.as_str() {
            "read" => {
                let Some(path) = tag.get_attribute("file") else {
                    continue;
                };
                let (first_line, last_line) = parse_lines(tag.get_attribute("lines"));
                inputs.push(ContextInput::File {
                    path: path.to_string(),
                    commit: tag.get_attribute("commit").map(str::to_string),
                    sha256: tag.get_attribute("sha256").unwrap_or_default().to_string(),
                    first_line,
                    last_line,
                    session_line: tag.source_line,
                });
            }
            "user" | "observation" | "tool-result" => {
                let text = tag.text_content();
                let summary = text
                    .lines()
                    .map(str::trim)
                    .find(|l| !l.is_empty())
                    .unwrap_or("")
                    .chars()
                    .take(120)
                    .collect::<String>();
                let source = match tag.name.as_str() {
                    "tool-result" => tag.get_attribute("name").map(str::to_string),
                    "observation" => tag.get_attribute("source").map(str::to_string),
                    _ => None,
                };
                inputs.push(ContextInput::Conversation {
                    element: tag.name.clone(),
                    id: tag.get_attribute("id").map(str::to_string),
                    source,
                    summary,
                    sha256: sha256_hex(text.as_bytes()),
                    lines: text.lines().count(),
                    session_line: tag.source_line,
                });
            }
            "wrote" => {
                let Some(file) = tag.get_attribute("file") else {
                    continue;
                };
                let (first_line, last_line) = parse_lines(tag.get_attribute("lines"));
                let hashes = tag
                    .get_attribute("hashes")
                    .unwrap_or_default()
                    .split_whitespace()
                    .map(str::to_string)
                    .collect();
                out.push(WriteContext {
                    session: session_path.to_string(),
                    session_line: tag.source_line,
                    file: file.to_string(),
                    first_line,
                    last_line,
                    hashes,
                    inputs: inputs.clone(),
                });
            }
            _ => {}
        }
    }
    out
}

fn parse_lines(attr: Option<&str>) -> (usize, usize) {
    let Some(attr) = attr else { return (0, 0) };
    match attr.split_once('-') {
        Some((a, b)) => (a.parse().unwrap_or(0), b.parse().unwrap_or(0)),
        None => {
            let n = attr.parse().unwrap_or(0);
            (n, n)
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// The `sessions/` directories that may hold a document's sessions: beside
/// the document and in every ancestor up to the project root (the git top
/// level, or the filesystem root when there is none). `hick agent --dir`
/// puts sessions under the project directory, which is usually one of these.
pub fn session_dirs_for(doc_path: &Path) -> Vec<PathBuf> {
    let start = doc_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let start = start.canonicalize().unwrap_or(start);
    let top = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&start)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        .and_then(|p| p.canonicalize().ok());
    let mut dirs = Vec::new();
    let mut cur = Some(start.as_path());
    while let Some(dir) = cur {
        dirs.push(dir.join("sessions"));
        if top.as_deref() == Some(dir) {
            break;
        }
        cur = dir.parent();
    }
    dirs
}

/// Whether a `<hick:wrote file=…>` names `doc_path`. The session records the
/// path the edit tool was given (relative to wherever it ran, or absolute);
/// compare canonically when both resolve, else by whole-segment suffix.
fn names_document(file: &str, session_dir: &Path, doc_path: &Path) -> bool {
    let doc = doc_path.canonicalize().ok();
    let candidates = [
        PathBuf::from(file),
        session_dir.join(file),
        session_dir
            .parent()
            .map(|p| p.join(file))
            .unwrap_or_default(),
    ];
    if let Some(doc) = &doc {
        for c in &candidates {
            if c.canonicalize().ok().as_ref() == Some(doc) {
                return true;
            }
        }
    }
    let a = file.replace('\\', "/");
    let b = doc_path.display().to_string().replace('\\', "/");
    a == b || a.ends_with(&format!("/{b}")) || b.ends_with(&format!("/{a}"))
}

/// Every write to `doc_path` found in the sessions near it, resolved against
/// `doc_source` (its current text), oldest session first.
pub fn context_for_document(doc_path: &Path, doc_source: &str) -> Vec<ResolvedWrite> {
    let index = LineIndex::new(doc_source);
    let mut out = Vec::new();
    for dir in session_dirs_for(doc_path) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "hick"))
            .collect();
        files.sort();
        for session in files {
            let Ok(source) = std::fs::read_to_string(&session) else {
                continue;
            };
            if !hick_lang::is_session_source(&source) {
                continue;
            }
            for write in derive_from_session(&session.display().to_string(), &source) {
                if !names_document(&write.file, &dir, doc_path) {
                    continue;
                }
                let current_lines = locate(&index, &write.hashes);
                out.push(ResolvedWrite {
                    write,
                    current_lines,
                });
            }
        }
    }
    out
}

/// Find `hashes` as a contiguous run in the index; 1-based inclusive lines.
fn locate(index: &LineIndex, hashes: &[String]) -> Option<(usize, usize)> {
    if hashes.is_empty() || hashes.len() > index.hashes.len() {
        return None;
    }
    index
        .hashes
        .windows(hashes.len())
        .position(|w| w == hashes)
        .map(|i| (i + 1, i + hashes.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-08-22T00:00:00Z">
<hick:user id="in0">Compute the numbers from the export.</hick:user>
<hick:assistant>
<hick:tool name="read_file"><hick:arg name="path">data/x.csv</hick:arg></hick:tool>
</hick:assistant>
<hick:tool-result id="in1" name="read_file" ok="true">
file: data/x.csv (2 lines)
1a2b|a,b
3c4d|1,2
</hick:tool-result>
<hick:read file="data/x.csv" commit="abc123" sha256="ff" lines="1-2"/>
<hick:assistant>
<hick:tool name="edit_doc"><hick:arg name="after">^</hick:arg><hick:input>Finding: one.</hick:input></hick:tool>
</hick:assistant>
<hick:tool-result id="in2" name="edit_doc" ok="true">
edited note.hick and re-wove.
</hick:tool-result>
<hick:wrote file="note.hick" lines="1-1" hashes="HASH"/>
<hick:assistant>
<hick:action lang="sh">echo later</hick:action>
</hick:assistant>
<hick:observation id="in3" source="action-0" exit="0">later</hick:observation>
</hick:session>
"#;

    /// Guarantee: docs/guarantees/agent/context-provenance-is-derived-from-the-session.md
    #[test]
    fn a_write_sees_every_input_before_it_and_none_after() {
        let hash = crate::tools::hashline::line_hash("Finding: one.");
        let source = SESSION.replace("HASH", &hash);
        let writes = derive_from_session("s.hick", &source);
        assert_eq!(writes.len(), 1);
        let w = &writes[0];
        assert_eq!(
            (w.file.as_str(), w.first_line, w.last_line),
            ("note.hick", 1, 1)
        );
        // The prompt, the read_file result, the file it showed — and not the
        // observation that came later.
        assert_eq!(w.inputs.len(), 4, "{:?}", w.inputs);
        assert!(
            matches!(&w.inputs[0], ContextInput::Conversation { element, .. } if element == "user")
        );
        assert!(
            matches!(&w.inputs[2], ContextInput::File { path, commit: Some(c), first_line: 1, last_line: 2, .. } if path == "data/x.csv" && c == "abc123")
        );
        assert!(
            matches!(&w.inputs[3], ContextInput::Conversation { element, source: Some(s), .. } if element == "tool-result" && s == "edit_doc")
        );

        // Resolved against the document as it stands now.
        let idx = LineIndex::new("Title\nFinding: one.\nMore\n");
        assert_eq!(locate(&idx, &w.hashes), Some((2, 2)));
        let idx = LineIndex::new("Title\nFinding: two.\n");
        assert_eq!(locate(&idx, &w.hashes), None);
    }

    #[test]
    fn context_for_document_finds_sessions_beside_the_document() {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("note.hick");
        std::fs::write(&doc, "Title\nFinding: one.\n").unwrap();
        std::fs::create_dir_all(dir.path().join("sessions")).unwrap();
        let hash = crate::tools::hashline::line_hash("Finding: one.");
        std::fs::write(
            dir.path().join("sessions/one.hick"),
            SESSION.replace("HASH", &hash),
        )
        .unwrap();
        let writes = context_for_document(&doc, &std::fs::read_to_string(&doc).unwrap());
        assert_eq!(writes.len(), 1, "{writes:?}");
        assert_eq!(writes[0].current_lines, Some((2, 2)));
        assert!(writes[0].write.session.ends_with("sessions/one.hick"));
    }
}
