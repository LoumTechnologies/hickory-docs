//! Talking to a hick code model server, so a generator does not have to.
//!
//! The Rust twin of `code-models/clients/python/hick_model.py`, and it exists
//! for the measured reason that one does: every generator written against a
//! model server re-implements the same forty lines — spawn the process, frame
//! the JSON, notice that GraphQL reports failure inside a successful-looking
//! body — and on the warehouse's generator that plumbing was 36% of the file
//! while the rules it served were 9%.
//!
//! ```text
//! let mut model = Model::start("hick-model-rust", ".")?;
//! for ty in describe_types(&mut model)?.types {
//!     println!("{}", ty.name);
//! }
//! ```
//!
//! Dropped into a generator's own crate by
//! `hick code-model rust --runtime --target rust -o <dir>`, beside the typed
//! client `--client` emits. Vendored rather than published: a package in a
//! generator means a restore, a lock file and a network hole in a cell that
//! needs none. Its only dependencies are `serde` and `serde_json`, which any
//! crate deserialising a GraphQL response already has.

// A generator uses two or three of these; the rest are here because the next
// generator uses the others. An unused helper in a vendored runtime is not a
// warning worth making somebody silence in their own crate.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

use serde::de::DeserializeOwned;
use serde_json::Value;

/// The server refused, or could not be reached.
#[derive(Debug)]
pub enum ModelError {
    /// No binary at that name — the most common failure by a wide margin, and
    /// the one worth a sentence rather than an errno.
    NotFound { binary: String, source: std::io::Error },
    /// The pipe broke, or the server closed without answering.
    Transport(String),
    /// GraphQL answered, and the answer was a refusal.
    Refused(Vec<String>),
    /// The response did not fit the type the query was generated for.
    Shape(serde_json::Error),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::NotFound { binary, source } => write!(
                f,
                "no model server at `{binary}`: {source}\n  \
                 `hick lang` shows which languages have one. A document that uses \
                 one should declare it — `<hick:needs bin=\"{binary}\" />` — so \
                 this is caught before the cell runs rather than inside the script."
            ),
            ModelError::Transport(message) => write!(f, "{message}"),
            ModelError::Refused(messages) => {
                write!(f, "the model refused the query:\n  {}", messages.join("\n  "))
            }
            ModelError::Shape(error) => write!(
                f,
                "the response did not fit the generated type: {error}\n  \
                 The query text and its types are generated together, so this \
                 means the SERVER changed. Regenerate the client."
            ),
        }
    }
}

impl std::error::Error for ModelError {}

/// One server, held open across a generation pass.
///
/// Held open because loading the workspace is the entire cost — about a
/// second for a 224k-line Cargo workspace, against sub-millisecond per query
/// afterwards — so a generator that spawns per question pays it per question.
///
/// One question at a time: the protocol is line-delimited and interleaving
/// two replies on one pipe has no framing to recover from.
pub struct Model {
    child: Child,
    stdout: BufReader<std::process::ChildStdout>,
    binary: String,
}

impl Model {
    /// Spawn `binary`, rooted at `root`.
    pub fn start(binary: &str, root: &str) -> Result<Self, ModelError> {
        let mut child = Command::new(binary)
            .arg(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // The server's own diagnostics — how many files it read, what it
            // could not bind — go to the terminal rather than into the
            // protocol. stdout carries the protocol and nothing else.
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|source| ModelError::NotFound {
                binary: binary.to_string(),
                source,
            })?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ModelError::Transport("the model server has no stdout".into()))?;
        Ok(Self {
            child,
            stdout: BufReader::new(stdout),
            binary: binary.to_string(),
        })
    }

    /// Ask one question and deserialise the answer into `T`.
    ///
    /// `T` is what `--client` generated for this query. The `data` object is
    /// what is decoded, never the envelope — an envelope-shaped struct would
    /// make every generated type carry a wrapper nobody reads.
    pub fn query<T: DeserializeOwned>(
        &mut self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<T, ModelError> {
        let data = self.ask(query, variables)?;
        serde_json::from_value(data).map_err(ModelError::Shape)
    }

    /// Ask one question, untyped.
    pub fn ask(&mut self, query: &str, variables: Option<Value>) -> Result<Value, ModelError> {
        let mut request = serde_json::json!({ "query": query });
        if let Some(variables) = variables {
            request["variables"] = variables;
        }
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| ModelError::Transport("the model server's stdin closed".into()))?;
        writeln!(stdin, "{request}")
            .and_then(|()| stdin.flush())
            .map_err(|e| ModelError::Transport(format!("could not reach {}: {e}", self.binary)))?;

        let mut line = String::new();
        let read = self
            .stdout
            .read_line(&mut line)
            .map_err(|e| ModelError::Transport(format!("could not read from {}: {e}", self.binary)))?;
        if read == 0 || line.trim().is_empty() {
            return Err(ModelError::Transport(format!(
                "the {} model server closed without answering — see its output above",
                self.binary
            )));
        }
        let answer: Value = serde_json::from_str(&line).map_err(|e| {
            ModelError::Transport(format!("the model server sent something that is not JSON: {e}"))
        })?;

        // GraphQL reports failure inside a 200-shaped body, so an unchecked
        // caller treats "you asked for a field that does not exist" as an
        // empty result and generates nothing, silently. This is the one line
        // the whole file exists for.
        // Written without a let-chain: this file is dropped into whatever
        // crate the generator lives in, and that crate may be on edition
        // 2021, where a let-chain is a syntax error rather than a style. A
        // runtime that only compiles on the newest edition is one somebody
        // has to edit before they can use it.
        let errors: Vec<String> = answer
            .get("errors")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .map(|e| {
                        e.get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("(no message)")
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default();
        if !errors.is_empty() {
            return Err(ModelError::Refused(errors));
        }
        Ok(answer.get("data").cloned().unwrap_or(Value::Null))
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        // Closing stdin is how the server is asked to stop; waiting is what
        // keeps a generator from leaving one behind for every query it made.
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

/// A buffer for building source text.
///
/// The other third of a generator, and the third nobody enjoys: appending to
/// a list and remembering where the commas go. `join` exists because a
/// parameter list is the most common thing a generator emits and getting its
/// separators right by hand is where the fiddly bugs live.
#[derive(Debug, Default)]
pub struct Emit {
    lines: Vec<String>,
}

impl Emit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn line(&mut self, line: impl Into<String>) -> &mut Self {
        self.lines.push(line.into());
        self
    }

    pub fn lines<I: IntoIterator<Item = S>, S: Into<String>>(&mut self, lines: I) -> &mut Self {
        self.lines.extend(lines.into_iter().map(Into::into));
        self
    }

    /// One entry per line with `sep` between — a parameter list.
    pub fn join(&mut self, parts: &[String], sep: &str) -> &mut Self {
        if !parts.is_empty() {
            self.lines.push(parts.join(sep));
        }
        self
    }

    pub fn blank(&mut self) -> &mut Self {
        self.lines.push(String::new());
        self
    }

    pub fn text(&self) -> String {
        let mut out = self.lines.join("\n");
        out.push('\n');
        out
    }

    pub fn write(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => {
                std::fs::create_dir_all(parent)?;
            }
            _ => {}
        }
        std::fs::write(path, self.text())
    }
}
