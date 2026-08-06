//! LSP bridge (api.md "Editor intelligence — LSP bridge (v0.3)").
//!
//! The server owns one `hick-lsp` session per (project, WS connection),
//! spawned lazily on the first channel-`0x02` frame. The child is the
//! `hick-lsp` binary over stdio (standard Content-Length framing) — the same
//! way editors run it — resolved via `HICK_LSP_BIN`, then next to the server
//! executable, then `$PATH`.
//!
//! Session workdir: the project git checkout seeded into a per-session temp
//! dir (`GitStore::seed_checkout`, exactly like run checkouts). It is seeded
//! once at session start: the document text itself always flows over
//! `didOpen`/`didChange` on the channel, so the on-disk copy only matters for
//! sibling files; a session sees the project as of its first LSP frame.
//!
//! URI spaces:
//! - client ↔ server: `hick:///<doc-path>` (and `hick-output:///<output-path>`
//!   for untranslatable generated-output targets),
//! - server ↔ hick-lsp: `file://<workdir>/<doc-path>`,
//! - hick-lsp child results may reference its virtual files
//!   (`file:///tmp/hick-lsp-vfiles/<hash>/<output-path>`) when a position has
//!   no direct `.hick` mapping; the server maps those through the last run's
//!   provenance into source coordinates, else emits `hick-output:///`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{Context as _, Result, bail};
use hick_lsp::structural::{byte_to_position, position_to_byte};
use hickory_lineage::Provenance;
use serde_json::{Value, json};
use tokio::io::{
    AsyncBufReadExt as _, AsyncReadExt as _, AsyncWriteExt as _, BufReader, BufWriter,
};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::AppState;
use crate::gitstore::GitStore;

/// Prefix under which hick-lsp materializes its virtual files.
pub const VFILE_URI_PREFIX: &str = "file:///tmp/hick-lsp-vfiles/";

/// URI scheme the browser client uses for doc sources.
pub const HICK_URI_PREFIX: &str = "hick:///";

/// URI scheme for generated-output targets that cannot be mapped to source.
pub const HICK_OUTPUT_URI_PREFIX: &str = "hick-output:///";

const SERVER_INIT_ID: &str = "hickory-server:initialize";

// ---------------------------------------------------------------------------
// Session (hick-lsp child process)
// ---------------------------------------------------------------------------

/// A running hick-lsp process bound to a seeded project checkout.
pub struct LspSession {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    workdir_uri: String,
    _workdir: tempfile::TempDir,
}

fn hick_lsp_program() -> PathBuf {
    if let Ok(p) = std::env::var("HICK_LSP_BIN")
        && !p.is_empty()
    {
        return PathBuf::from(p);
    }
    if let Ok(exe) = std::env::current_exe() {
        // Server binary dir (target/debug, /app), and its parent for test
        // executables living in target/debug/deps.
        for dir in exe
            .parent()
            .into_iter()
            .flat_map(|d| [d, d.parent().unwrap_or(d)])
        {
            let candidate = dir.join("hick-lsp");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    PathBuf::from("hick-lsp")
}

impl LspSession {
    /// Spawn hick-lsp against a fresh checkout of the project and complete the
    /// `initialize` handshake server-side. Returns the session plus the stream
    /// of messages (responses + notifications) coming back from hick-lsp,
    /// still in workdir/vfile URI space.
    pub async fn start(
        git: &GitStore,
        project_id: Uuid,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Value>)> {
        let workdir = tempfile::tempdir().context("creating LSP session workdir")?;
        git.seed_checkout(project_id, workdir.path())
            .context("seeding LSP session checkout")?;
        let workdir_uri = format!("file://{}/", workdir.path().display());

        let program = hick_lsp_program();
        let mut child = Command::new(&program)
            .current_dir(workdir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("spawning hick-lsp at {}", program.display()))?;

        let stdin = BufWriter::new(child.stdin.take().expect("piped stdin"));
        let stdout = child.stdout.take().expect("piped stdout");

        let (msg_tx, msg_rx) = mpsc::unbounded_channel::<Value>();
        let (init_tx, init_rx) = oneshot::channel::<()>();
        tokio::spawn(read_loop(stdout, msg_tx, init_tx));

        let mut session = LspSession {
            child,
            stdin,
            workdir_uri,
            _workdir: workdir,
        };

        session
            .send(&json!({
                "jsonrpc": "2.0",
                "id": SERVER_INIT_ID,
                "method": "initialize",
                "params": {
                    "processId": std::process::id(),
                    "rootUri": session.workdir_uri.trim_end_matches('/'),
                    "capabilities": {},
                },
            }))
            .await?;

        match tokio::time::timeout(std::time::Duration::from_secs(10), init_rx).await {
            Ok(Ok(())) => {}
            _ => bail!("hick-lsp did not answer initialize within 10s"),
        }
        session
            .send(&json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }))
            .await?;
        Ok((session, msg_rx))
    }

    /// `file://<workdir>/` — the prefix doc paths are mounted under.
    pub fn workdir_uri(&self) -> &str {
        &self.workdir_uri
    }

    /// Write one framed JSON-RPC message to hick-lsp.
    pub async fn send(&mut self, msg: &Value) -> Result<()> {
        let body = serde_json::to_string(msg)?;
        self.stdin
            .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
            .await?;
        self.stdin.write_all(body.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Graceful shutdown: LSP shutdown/exit, then kill.
    pub async fn shutdown(mut self) {
        let _ = self
            .send(&json!({
                "jsonrpc": "2.0",
                "id": "hickory-server:shutdown",
                "method": "shutdown",
            }))
            .await;
        let _ = self
            .send(&json!({ "jsonrpc": "2.0", "method": "exit" }))
            .await;
        let _ =
            tokio::time::timeout(std::time::Duration::from_millis(500), self.child.wait()).await;
        let _ = self.child.kill().await;
    }
}

async fn read_loop(
    stdout: tokio::process::ChildStdout,
    msg_tx: mpsc::UnboundedSender<Value>,
    init_tx: oneshot::Sender<()>,
) {
    let mut reader = BufReader::new(stdout);
    let mut init_tx = Some(init_tx);
    loop {
        // Headers.
        let mut content_length: Option<usize> = None;
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => return,
                Ok(_) => {}
                Err(_) => return,
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(v) = trimmed.strip_prefix("Content-Length:") {
                content_length = v.trim().parse().ok();
            }
        }
        let Some(len) = content_length else { return };
        let mut body = vec![0u8; len];
        if reader.read_exact(&mut body).await.is_err() {
            return;
        }
        let Ok(msg) = serde_json::from_slice::<Value>(&body) else {
            continue;
        };
        // The server-issued handshake responses never reach the client.
        if let Some(id) = msg.get("id").and_then(|i| i.as_str()) {
            if id == SERVER_INIT_ID {
                if let Some(tx) = init_tx.take() {
                    let _ = tx.send(());
                }
                continue;
            }
            if id.starts_with("hickory-server:") {
                continue;
            }
        }
        // Server-directed requests from hick-lsp (client/registerCapability,
        // workspace/configuration, window/workDoneProgress/create …) must not
        // leak to the browser; the bridge is not a full client, so drop them.
        if msg.get("method").is_some() && msg.get("id").is_some() {
            continue;
        }
        if msg_tx.send(msg).is_err() {
            return;
        }
    }
}

/// The canned `initialize` result the server answers with when the browser
/// client sends `initialize` itself (the contract says the client starts at
/// `didOpen`, but answering keeps standard LSP client libraries happy).
pub fn bridge_initialize_result() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": 1,
            "hoverProvider": true,
            "definitionProvider": true,
            "referencesProvider": true,
            "completionProvider": {},
        },
        "serverInfo": { "name": "hickory-lsp-bridge" },
    })
}

// ---------------------------------------------------------------------------
// URI rewriting
// ---------------------------------------------------------------------------

/// Replace `from`-prefixed string values with `to` + rest, everywhere in the
/// message. LSP carries URIs in many positions (params, results, diagnostics);
/// prefix rewriting over all strings is exact for our disjoint URI spaces.
pub fn rewrite_uri_prefix(value: &mut Value, from: &str, to: &str) {
    match value {
        Value::String(s) => {
            if let Some(rest) = s.strip_prefix(from) {
                *s = format!("{to}{rest}");
            }
        }
        Value::Array(items) => {
            for item in items {
                rewrite_uri_prefix(item, from, to);
            }
        }
        Value::Object(obj) => {
            for (_, v) in obj.iter_mut() {
                rewrite_uri_prefix(v, from, to);
            }
        }
        _ => {}
    }
}

/// The generated-output path a hick-lsp virtual-file URI refers to
/// (`file:///tmp/hick-lsp-vfiles/<hash>/<path>` → `<path>`).
pub fn vfile_output_path(uri: &str) -> Option<&str> {
    let rest = uri.strip_prefix(VFILE_URI_PREFIX)?;
    let (_hash, path) = rest.split_once('/')?;
    (!path.is_empty()).then_some(path)
}

// ---------------------------------------------------------------------------
// Provenance translation of generated-output locations
// ---------------------------------------------------------------------------

/// Cached per-message translation data.
pub struct OutputTranslator {
    /// output path → (content, provenance) for the doc's last successful run.
    outputs: HashMap<String, Option<(String, Vec<Provenance>)>>,
    /// project doc path → source.
    doc_sources: HashMap<String, String>,
}

impl OutputTranslator {
    /// Prefetch the outputs named by vfile URIs in `msg` plus all project doc
    /// sources; a doc without runs simply yields no translatable outputs.
    pub async fn prepare(state: &AppState, doc_id: Uuid, project_id: Uuid, msg: &Value) -> Self {
        let mut paths = HashSet::new();
        collect_vfile_paths(msg, &mut paths);

        let mut outputs = HashMap::new();
        if !paths.is_empty() {
            let run_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM runs
                 WHERE doc_id = $1 AND kind = 'run' AND status = 'ok'
                 ORDER BY started_at DESC LIMIT 1",
            )
            .bind(doc_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
            for path in paths {
                let row: Option<(String, Value)> = match run_id {
                    Some(run_id) => sqlx::query_as(
                        "SELECT content, provenance FROM run_outputs
                         WHERE run_id = $1 AND path = $2",
                    )
                    .bind(run_id)
                    .bind(&path)
                    .fetch_optional(&state.db)
                    .await
                    .ok()
                    .flatten(),
                    None => None,
                };
                let parsed = row.and_then(|(content, prov)| {
                    serde_json::from_value::<Vec<Provenance>>(prov)
                        .ok()
                        .map(|p| (content, p))
                });
                outputs.insert(path, parsed);
            }
        }

        let doc_sources: HashMap<String, String> = if outputs.is_empty() {
            HashMap::new()
        } else {
            sqlx::query_as::<_, (String, String)>(
                "SELECT path, source FROM docs WHERE project_id = $1",
            )
            .bind(project_id)
            .fetch_all(&state.db)
            .await
            .map(|rows| rows.into_iter().collect())
            .unwrap_or_default()
        };

        OutputTranslator {
            outputs,
            doc_sources,
        }
    }

    /// Nothing to translate?
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    /// Map one output byte offset through provenance: `(doc_path, src_byte)`.
    fn map_byte(&self, provenance: &[Provenance], byte: usize) -> Option<(String, usize)> {
        // An exclusive end byte may sit exactly on an entry boundary; probe
        // the containing entry, else the one ending at `byte`.
        let entry = provenance
            .iter()
            .find(|p| p.start <= byte && byte < p.end)
            .or_else(|| provenance.iter().find(|p| p.end == byte && p.start < p.end))?;
        let (doc_path, s, _e) = entry.origin.location()?;
        Some((doc_path.to_string(), s + (byte - entry.start)))
    }

    /// Translate one `(vfile uri, LSP range)` location into hick space:
    /// `Some((uri, range))` — `hick:///` with source positions when provenance
    /// maps it, else `hick-output:///` with the original output positions.
    pub fn translate_location(&self, uri: &str, range: &Value) -> Option<(String, Value)> {
        let path = vfile_output_path(uri)?;
        let fallback = || Some((format!("{HICK_OUTPUT_URI_PREFIX}{path}"), range.clone()));
        let Some(Some((content, provenance))) = self.outputs.get(path) else {
            return fallback();
        };
        let (Some(sb), Some(eb)) = (
            lsp_range_byte(content, range, "start"),
            lsp_range_byte(content, range, "end"),
        ) else {
            return fallback();
        };
        let (Some((sdoc, s_src)), Some((edoc, e_src))) =
            (self.map_byte(provenance, sb), self.map_byte(provenance, eb))
        else {
            return fallback();
        };
        if sdoc != edoc || e_src < s_src {
            return fallback();
        }
        let Some(source) = self.doc_sources.get(&sdoc) else {
            return fallback();
        };
        let (sl, sc) = byte_to_position(source, s_src);
        let (el, ec) = byte_to_position(source, e_src);
        Some((
            format!("{HICK_URI_PREFIX}{sdoc}"),
            json!({
                "start": { "line": sl, "character": sc },
                "end": { "line": el, "character": ec },
            }),
        ))
    }

    /// Rewrite every vfile location in the message in place.
    pub fn rewrite(&self, value: &mut Value) {
        match value {
            Value::Array(items) => {
                for item in items {
                    self.rewrite(item);
                }
            }
            Value::Object(obj) => {
                let loc = obj
                    .get("uri")
                    .and_then(|u| u.as_str())
                    .filter(|u| u.starts_with(VFILE_URI_PREFIX))
                    .and_then(|u| Some((u.to_string(), obj.get("range")?.clone())));
                if let Some((uri, range)) = loc
                    && let Some((new_uri, new_range)) = self.translate_location(&uri, &range)
                {
                    obj.insert("uri".into(), Value::String(new_uri));
                    obj.insert("range".into(), new_range);
                    return;
                }
                let link = obj
                    .get("targetUri")
                    .and_then(|u| u.as_str())
                    .filter(|u| u.starts_with(VFILE_URI_PREFIX))
                    .and_then(|u| Some((u.to_string(), obj.get("targetRange")?.clone())));
                if let Some((uri, range)) = link
                    && let Some((new_uri, new_range)) = self.translate_location(&uri, &range)
                {
                    obj.insert("targetUri".into(), Value::String(new_uri));
                    obj.insert("targetRange".into(), new_range.clone());
                    obj.insert("targetSelectionRange".into(), new_range);
                    return;
                }
                for (_, v) in obj.iter_mut() {
                    self.rewrite(v);
                }
            }
            _ => {}
        }
    }
}

fn collect_vfile_paths(value: &Value, out: &mut HashSet<String>) {
    match value {
        Value::String(s) => {
            if let Some(path) = vfile_output_path(s) {
                out.insert(path.to_string());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_vfile_paths(item, out);
            }
        }
        Value::Object(obj) => {
            for v in obj.values() {
                collect_vfile_paths(v, out);
            }
        }
        _ => {}
    }
}

fn lsp_range_byte(content: &str, range: &Value, which: &str) -> Option<usize> {
    let line = range.pointer(&format!("/{which}/line"))?.as_u64()? as u32;
    let ch = range.pointer(&format!("/{which}/character"))?.as_u64()? as u32;
    position_to_byte(content, line, ch)
}

/// Full outbound rewrite: hick-lsp message → client message.
pub async fn rewrite_outbound(
    state: &AppState,
    doc_id: Uuid,
    project_id: Uuid,
    workdir_uri: &str,
    mut msg: Value,
) -> Value {
    let translator = OutputTranslator::prepare(state, doc_id, project_id, &msg).await;
    if !translator.is_empty() {
        translator.rewrite(&mut msg);
    }
    rewrite_uri_prefix(&mut msg, workdir_uri, HICK_URI_PREFIX);
    msg
}

/// Inbound rewrite: client message → hick-lsp message.
pub fn rewrite_inbound(msg: &mut Value, workdir_uri: &str) {
    rewrite_uri_prefix(msg, HICK_URI_PREFIX, workdir_uri);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_rewrite_is_recursive_and_prefix_only() {
        let mut v = json!({
            "params": {
                "textDocument": { "uri": "hick:///guide.hick" },
                "unrelated": "hick without prefix",
                "nested": [{ "uri": "hick:///a/b.hick" }],
            }
        });
        rewrite_uri_prefix(&mut v, HICK_URI_PREFIX, "file:///tmp/w/");
        assert_eq!(
            v["params"]["textDocument"]["uri"],
            "file:///tmp/w/guide.hick"
        );
        assert_eq!(v["params"]["nested"][0]["uri"], "file:///tmp/w/a/b.hick");
        assert_eq!(v["params"]["unrelated"], "hick without prefix");
    }

    #[test]
    fn vfile_output_path_strips_hash() {
        assert_eq!(
            vfile_output_path("file:///tmp/hick-lsp-vfiles/12345/src/main.rs"),
            Some("src/main.rs")
        );
        assert_eq!(vfile_output_path("file:///tmp/hick-lsp-vfiles/12345"), None);
        assert_eq!(vfile_output_path("file:///elsewhere/main.rs"), None);
    }

    #[test]
    fn translator_maps_output_range_to_source_or_output_uri() {
        use hickory_lineage::Origin;
        let content = "fn alpha() {}\nfn beta() {}\n";
        let source = "<doc>\nfn alpha() {}\n</doc>\n";
        let provenance = vec![
            Provenance {
                start: 0,
                end: 13,
                origin: Origin::Paste {
                    doc_path: "guide.hick".into(),
                    span: (6, 19),
                },
            },
            Provenance {
                start: 13,
                end: 14,
                origin: Origin::Synthetic,
            },
        ];
        let mut outputs = HashMap::new();
        outputs.insert(
            "src/main.rs".to_string(),
            Some((content.to_string(), provenance)),
        );
        let mut doc_sources = HashMap::new();
        doc_sources.insert("guide.hick".to_string(), source.to_string());
        let t = OutputTranslator {
            outputs,
            doc_sources,
        };

        // Bytes 3..8 of the output ("alpha") → source bytes 9..14 → line 1.
        let range = json!({
            "start": { "line": 0, "character": 3 },
            "end": { "line": 0, "character": 8 },
        });
        let (uri, mapped) = t
            .translate_location("file:///tmp/hick-lsp-vfiles/99/src/main.rs", &range)
            .unwrap();
        assert_eq!(uri, "hick:///guide.hick");
        assert_eq!(mapped["start"]["line"], 1);
        assert_eq!(mapped["start"]["character"], 3);
        assert_eq!(mapped["end"]["character"], 8);

        // A range reaching into the synthetic byte keeps the output URI.
        let range = json!({
            "start": { "line": 0, "character": 13 },
            "end": { "line": 1, "character": 2 },
        });
        let (uri, mapped) = t
            .translate_location("file:///tmp/hick-lsp-vfiles/99/src/main.rs", &range)
            .unwrap();
        assert_eq!(uri, "hick-output:///src/main.rs");
        assert_eq!(mapped, range);
    }
}
