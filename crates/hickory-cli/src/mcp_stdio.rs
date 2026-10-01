use super::*;

/// Where this server records, when nobody named a file.
///
/// Recording used to be opt-in (`--session` / `HICKORY_SESSION`), and the
/// registration `hick init` writes sets neither — so an external agent
/// editing through MCP left **no** record that an agent had been there.
/// `hick context`, the whole provenance family whose job is answering "what
/// was in front of the model when it wrote these lines", reported nothing for
/// documents an agent had just rewritten. Silence is the wrong default for
/// the one thing this product exists to know.
///
/// One file per server process, which is one file per conversation — the same
/// unit `hick agent` and the app's dock already use.
///
/// It defaults on only where the answer is unambiguous: a project that has
/// run `hick init`, recognised by the `sessions/` line that command writes
/// into `.gitignore`. Elsewhere — an MCP server started in someone's home
/// directory, a repository that has never seen hick — creating folders
/// nobody asked for would be the worse mistake, so it stays silent and
/// `HICKORY_SESSION` remains the way in.
fn default_session_log() -> Option<PathBuf> {
    let root = std::env::current_dir().ok()?;
    let sessions = root.join("sessions");
    if !sessions.is_dir() {
        let ignore = std::fs::read_to_string(root.join(".gitignore")).ok()?;
        if !ignore.lines().any(|l| l.trim() == "sessions/") {
            return None;
        }
    }
    // The same `sessions/<timestamp>-<slug>.md` convention `hick agent`
    // writes, so one folder holds every conversation whoever had it.
    Some(hickory_agent::session_file_path(&root, "mcp"))
}

/// Serve MCP on stdin/stdout until the client closes the stream.
///
/// One request at a time, on purpose: the sessions this server holds are
/// single-writer over a document, and interleaving two edits to one file would
/// reintroduce exactly the staleness the design exists to prevent.
pub async fn serve(default_doc: Option<PathBuf>, params: Vec<(String, String)>) -> Result<()> {
    let executor = ExecutorChoice::from_env()?.build().await?;
    let mut server = Server {
        root: std::env::current_dir()?,
        sessions: HashMap::new(),
        executor: executor.clone(),
        params,
        default_doc,
        session_log: crate::doc_tools::session_from(None).or_else(default_session_log),
        debuggers: crate::debug_sessions::Registry::new(),
    };

    let options = crate::engine::mcp::Options {
        root: server.root.clone(),
        doc: server.default_doc.clone(),
        params: server.params.clone(),
        executor: ExecutorChoice::from_env()?,
        session: server.session_log.clone(),
    };
    let mut remote = None;
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                // Parse errors have no id to answer against; report and keep
                // serving rather than dropping the connection.
                eprintln!("mcp: ignoring unparseable message: {e}");
                continue;
            }
        };
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let id = request.get("id").cloned();
        let empty = json!({});
        let params = request.get("params").unwrap_or(&empty).clone();

        // A notification (no id) gets no response — answering one is a
        // protocol violation that some clients treat as fatal.
        if id.is_none() {
            continue;
        }

        let result = if method == "tools/call" && std::env::var_os(crate::engine::WORKER).is_none()
        {
            if remote.is_none() {
                remote = Some(crate::engine::mcp::Remote::new(options.clone()).await?);
            }
            Ok(remote
                .as_ref()
                .unwrap()
                .call(&params)
                .await
                .unwrap_or_else(crate::engine::mcp::refusal))
        } else {
            server.handle(&method, &params).await
        };
        let response = match result {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, message)) => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
            }
        };
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }

    executor.shutdown().await.ok();
    Ok(())
}

pub(crate) async fn engine_server(options: crate::engine::mcp::Options) -> Result<Server> {
    Ok(Server {
        root: options.root,
        sessions: HashMap::new(),
        executor: options.executor.build().await?,
        params: options.params,
        default_doc: options.doc,
        session_log: options.session,
        debuggers: crate::debug_sessions::Registry::new(),
    })
}
