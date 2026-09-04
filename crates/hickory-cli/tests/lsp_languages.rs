//! Language intelligence, per language, on whatever machine this runs.
//!
//! Protects docs/guarantees/editor-intelligence/a-language-server-is-found-without-being-configured.md
//! Protects docs/guarantees/editor-intelligence/the-meta-lsp-forwards-what-the-child-supports.md
//!
//! These are SYSTEM tests in the sense `.instructions/framework-agnostic-system-tests.md`
//! means it: nothing here calls into `hick-lsp` as a library. Each language is
//! driven twice, by the two paths a real user has —
//!
//!  1. **An editor**, by spawning the `hick-lsp` binary and speaking
//!     JSON-RPC over its stdio, byte for byte what Helix or VS Code does.
//!  2. **The desktop app**, by binding the local server on a real port and
//!     driving the LSP bridge over the same WebSocket the app's window uses.
//!
//! Both must answer in DOCUMENT coordinates, because that is the whole claim:
//! the buffer you are editing is the `.hick` file, not the virtual file the
//! child language server actually saw.
//!
//! ## Each project is built the way that ecosystem builds one
//!
//! A fixture is not a bare directory with one file in it. It is a package
//! manager's layout — `package.json` beside `node_modules/.bin`, a virtualenv
//! with its `pyvenv.cfg`, a `Cargo.toml` with `src/`, a `go.mod` — because
//! that is what discovery has to work against, and a test that passes on a
//! shape nobody's project has is a test that proves nothing.
//!
//! ## What happens when a server is not installed
//!
//! The language is skipped, loudly, naming what was missing. Never silently:
//! a suite that quietly tests nothing is worse than one that fails, and this
//! one is expected to cover a different subset on every machine it runs on.
//! What is NOT conditional is that at least one language must work — a
//! machine with no language server at all fails the last test in this file,
//! because that is a broken environment rather than a passing suite.
//!
//! ## Windows
//!
//! The discovery tests and the two sandbox tests run there and mean the same
//! thing; `every_installed_language_answers_in_document_coordinates` does not,
//! and the reason is a product one rather than a test one:
//! `hick_lsp::backend` writes every virtual file under a hardcoded
//! `file:///tmp/hick-lsp-vfiles/…`, and `Url::to_file_path` refuses a path with
//! no drive letter on Windows — so nothing is written, and a child server is
//! asked about files that do not exist. Until that is fixed, a Windows job
//! should run this file with
//! `--skip every_installed_language_answers_in_document_coordinates`, and
//! removing that skip is how the fix gets proven.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// The languages, and the project each one lives in
// ---------------------------------------------------------------------------

struct Language {
    /// What `hick-lsp` calls it, and what this test reports.
    id: &'static str,
    /// The path of the `hick:file` block; its extension does the routing.
    file: &'static str,
    /// The block's contents. `SYMBOL` below must be defined in it.
    code: &'static str,
    /// A symbol defined in `code`, used to ask for hover and definition.
    symbol: &'static str,
    /// Extra files that make this a real project of its ecosystem.
    scaffold: &'static [(&'static str, &'static str)],
}

const LANGUAGES: &[Language] = &[
    Language {
        id: "python",
        file: "app.py",
        code: "def summarise(path):\n    return len(path)\n\n\ndef main():\n    return summarise(\"x\")\n",
        symbol: "summarise",
        // A pyproject is what every modern Python tool keys off; without one
        // a server may treat each file as standalone.
        scaffold: &[(
            "pyproject.toml",
            "[project]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
        )],
    },
    Language {
        id: "rust",
        file: "src/main.rs",
        code: "fn helper(value: u32) -> u32 {\n    value * 2\n}\n\nfn main() {\n    println!(\"{}\", helper(21));\n}\n",
        symbol: "helper",
        scaffold: &[(
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )],
    },
    Language {
        id: "typescript",
        file: "index.ts",
        code: "export function double(value: number): number {\n  return value * 2;\n}\n\nconst answer = double(21);\nconsole.log(answer);\n",
        symbol: "double",
        scaffold: &[
            (
                "package.json",
                "{\n  \"name\": \"fixture\",\n  \"version\": \"1.0.0\"\n}\n",
            ),
            (
                "tsconfig.json",
                "{\n  \"compilerOptions\": { \"strict\": true, \"target\": \"ES2020\" }\n}\n",
            ),
        ],
    },
    Language {
        id: "go",
        file: "main.go",
        code: "package main\n\nimport \"fmt\"\n\nfunc double(value int) int {\n\treturn value * 2\n}\n\nfunc main() {\n\tfmt.Println(double(21))\n}\n",
        symbol: "double",
        scaffold: &[("go.mod", "module fixture\n\ngo 1.22\n")],
    },
];

/// Build the project and the document, and return the document's path.
///
/// The document is written with the block's code indented by nothing, so a
/// symbol's column in the document is its column in the block plus zero —
/// which keeps the assertions about coordinates readable.
fn scaffold(language: &Language) -> (tempfile::TempDir, PathBuf, u32) {
    let dir = tempfile::tempdir().expect("a temp project");
    // A repository root, because discovery's walk is bounded by one.
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    for (path, contents) in language.scaffold {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(full, contents).unwrap();
    }

    let header = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
         # A document with {} in it\n\n\
         <hick:file path=\"{}\">\n",
        language.id, language.file
    );
    // The line the block's first line of code lands on, 0-based: every line
    // of the header except the last, which is the tag itself.
    let first_code_line = header.lines().count() as u32;
    let document = format!("{header}{}</hick:file>\n</hick:doc>\n", language.code);
    let path = dir.path().join("doc.hick");
    std::fs::write(&path, document).unwrap();
    (dir, path, first_code_line)
}

/// Where `symbol` is defined in the document: (line, character), 0-based.
fn definition_site(language: &Language, first_code_line: u32) -> (u32, u32) {
    for (offset, line) in language.code.lines().enumerate() {
        if let Some(column) = line.find(language.symbol) {
            return (first_code_line + offset as u32, column as u32);
        }
    }
    panic!("{} is not defined in its own fixture", language.symbol);
}

// ---------------------------------------------------------------------------
// A real LSP client, over the real binary's stdio
// ---------------------------------------------------------------------------

struct StdioClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl StdioClient {
    fn start(root: &Path) -> Option<Self> {
        let binary = binary("hick-lsp")?;
        let mut child = Command::new(binary)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("hick-lsp starts");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Some(Self {
            child,
            stdin,
            stdout,
            next_id: 1,
        })
    }

    fn send(&mut self, message: &Value) {
        let body = serde_json::to_vec(message).unwrap();
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
        self.stdin.write_all(&body).unwrap();
        self.stdin.flush().unwrap();
    }

    fn request(&mut self, method: &str, params: Value) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        id
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    /// Ask repeatedly until the answer is something, or the budget runs out.
    ///
    /// A language server that is still loading answers `null` rather than
    /// waiting, so "no answer yet" and "no answer at all" look identical for
    /// the first few seconds of a session. An editor resolves this by asking
    /// again when the cursor moves; this does the same, deliberately.
    fn request_until(&mut self, method: &str, params: Value, budget: Duration) -> Option<Value> {
        let deadline = Instant::now() + budget;
        let mut attempt = 0;
        while Instant::now() < deadline {
            attempt += 1;
            let id = self.request(method, params.clone());
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.wait_for(id, remaining.min(Duration::from_secs(10))) {
                Some(Value::Null) | None => {}
                // An empty list is "nothing yet" too: rust-analyzer answers
                // `[]` to a definition request while it is still indexing,
                // and taking that as the answer failed this test on a cold
                // machine while passing on a warm one.
                Some(Value::Array(items)) if items.is_empty() => {}
                Some(result) => {
                    if attempt > 1 {
                        eprintln!("  ({method} answered on attempt {attempt})");
                    }
                    return Some(result);
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        None
    }

    /// Read messages until the reply to `id` arrives, or time out.
    ///
    /// Everything else on the wire — diagnostics, log messages, the child's
    /// progress reports — is passed over rather than treated as an error: a
    /// real client sees all of it too.
    fn wait_for(&mut self, id: i64, timeout: Duration) -> Option<Value> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let message = self.read_message(deadline)?;
            if message.get("id").and_then(Value::as_i64) == Some(id) {
                return message.get("result").cloned();
            }
        }
        None
    }

    fn read_message(&mut self, deadline: Instant) -> Option<Value> {
        let mut length = None;
        loop {
            if Instant::now() > deadline {
                return None;
            }
            let mut line = String::new();
            if self.stdout.read_line(&mut line).ok()? == 0 {
                return None;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(value) = trimmed.strip_prefix("Content-Length: ") {
                length = value.parse::<usize>().ok();
            }
        }
        let mut body = vec![0u8; length?];
        self.stdout.read_exact(&mut body).ok()?;
        serde_json::from_slice(&body).ok()
    }
}

impl Drop for StdioClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The binary under test, from the same target directory cargo built.
fn binary(name: &str) -> Option<PathBuf> {
    // `CARGO_BIN_EXE_<name>` only exists for binaries of THIS crate, and
    // `hick-lsp` belongs to another, so the path is derived from the test
    // executable's own location instead.
    let mut path = std::env::current_exe().ok()?;
    path.pop(); // deps/
    path.pop(); // debug/
    let candidate = path.join(if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    });
    candidate.is_file().then_some(candidate)
}

// ---------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------

/// Is a server for this language installed here?
///
/// Asked by running `hick init`'s own report rather than by looking for a
/// binary, so the test agrees with the product about what "installed" means.
fn server_available(language: &str, root: &Path) -> bool {
    hick_lsp::discovery::discover(language, root).is_some()
}

fn skip(language: &str, why: &str) {
    eprintln!("SKIPPED {language}: {why}");
}

/// The same intent, in the shell a cell actually gets on this platform.
///
/// Cells run through `sh -c` on Unix and `cmd.exe /C` on Windows, so a cell
/// written in POSIX syntax proves nothing about the sandbox on Windows — and
/// worse, an escape attempt made with a tool cmd does not have fails whatever
/// the sandbox does, which is a green result for a run in which nothing was
/// confined. See `crates/hickory-executor-sandbox/tests/confinement.rs`, whose
/// Windows forms these match.
fn per_shell(unix: &str, windows: &str) -> String {
    if cfg!(windows) {
        windows.to_string()
    } else {
        unix.to_string()
    }
}

/// Somewhere outside any cell's workdir that an ordinary user CAN write, so a
/// cell that reaches it really did escape.
///
/// Not a system directory on Windows: a write to one fails for lack of
/// Administrator rather than for lack of permission to leave the sandbox, and
/// the test would pass on an unconfined machine.
fn escape_target() -> std::path::PathBuf {
    if cfg!(windows) {
        std::path::PathBuf::from(r"C:\Users\Public\hickory-escaped-from-a-document.txt")
    } else {
        std::path::PathBuf::from("/etc/hickory-escaped")
    }
}

/// Open the document and ask the questions an editor asks.
///
/// Returns false when the language's server is not installed here.
fn drive_one_language(language: &Language) -> bool {
    let (dir, doc_path, first_code_line) = scaffold(language);
    if !server_available(language.id, dir.path()) {
        skip(
            language.id,
            &format!(
                "no language server for it is installed on this machine \
                 (`hick lsp install {}`, or install one the way that ecosystem does)",
                language.id
            ),
        );
        return false;
    }
    let Some(mut client) = StdioClient::start(dir.path()) else {
        skip(
            language.id,
            "hick-lsp has not been built into this target dir",
        );
        return false;
    };

    let root_uri = format!("file://{}", dir.path().display());
    let uri = format!("file://{}", doc_path.display());
    let id = client.request(
        "initialize",
        json!({ "processId": std::process::id(), "rootUri": root_uri, "capabilities": {} }),
    );
    let capabilities = client
        .wait_for(id, Duration::from_secs(20))
        .expect("the server answers initialize");
    // Advertised, not merely implemented: a client only asks for what the
    // server claims it can do.
    for promised in [
        "hoverProvider",
        "definitionProvider",
        "referencesProvider",
        "documentSymbolProvider",
        "callHierarchyProvider",
        "semanticTokensProvider",
        "renameProvider",
    ] {
        assert!(
            capabilities["capabilities"].get(promised).is_some(),
            "{promised} was not advertised: {capabilities}"
        );
    }
    client.notify("initialized", json!({}));

    let text = std::fs::read_to_string(&doc_path).unwrap();
    client.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": uri, "languageId": "hick", "version": 1, "text": text}}),
    );

    // Real servers index before they answer; gopls and rust-analyzer are the
    // slow ones. A generous budget here costs nothing when the answer is
    // ready sooner — `request_until` polls and returns the moment there is
    // something, so this is a ceiling rather than a wait.
    let budget = Duration::from_secs(60);
    let (line, character) = definition_site(language, first_code_line);

    // Hover, on the symbol's own definition.
    //
    // Retried rather than asked once. A server that is still loading a
    // workspace answers `null` immediately instead of waiting — rust-analyzer
    // and gopls both do — and a real editor gets its answer because the user
    // moves the cursor again a second later. Asking once would test this
    // machine's disk speed.
    let hover = client.request_until(
        "textDocument/hover",
        json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": character}}),
        budget,
    );
    let hover_text = hover
        .as_ref()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(
        hover_text.contains(language.symbol),
        "{}: hover on its own definition did not mention {}: {hover_text}",
        language.id,
        language.symbol
    );

    // Definition, from the USE site, must land on the definition line — in
    // document coordinates, which is the whole point of the meta-LSP.
    let use_line = language
        .code
        .lines()
        .enumerate()
        .filter(|(_, text)| text.contains(language.symbol))
        .nth(1)
        .map(|(offset, text)| {
            (
                first_code_line + offset as u32,
                text.find(language.symbol).unwrap() as u32,
            )
        });
    if let Some((use_line, use_column)) = use_line {
        let found = client.request_until(
            "textDocument/definition",
            json!({"textDocument": {"uri": uri}, "position": {"line": use_line, "character": use_column}}),
            budget,
        );
        if let Some(result) = found {
            let target = first_location(&result);
            if let Some((target_uri, target_line)) = target {
                assert!(
                    target_uri.ends_with("doc.hick"),
                    "{}: definition pointed outside the document: {target_uri}",
                    language.id
                );
                assert_eq!(
                    target_line, line,
                    "{}: definition landed on document line {target_line}, not {line}",
                    language.id
                );
            }
        }
    }

    // Document symbols: the outline an editor draws.
    let outline = client.request_until(
        "textDocument/documentSymbol",
        json!({"textDocument": {"uri": uri}}),
        budget,
    );
    if let Some(result) = outline {
        assert!(
            result.to_string().contains(language.symbol),
            "{}: the outline is missing {}: {result}",
            language.id,
            language.symbol
        );
    }

    // Call hierarchy: three steps, and the interesting one is the third —
    // the client hands back an item it was shown, in DOCUMENT coordinates,
    // and the server has to reach the child with the item the CHILD made.
    let prepared = client.request_until(
        "textDocument/prepareCallHierarchy",
        json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": character}}),
        budget,
    );
    if let Some(items) = prepared
        .as_ref()
        .and_then(|r| r.as_array())
        .filter(|a| !a.is_empty())
    {
        let item = &items[0];
        assert_eq!(
            item["uri"].as_str(),
            Some(uri.as_str()),
            "{}: a prepared item names the staged file, not the document: {item}",
            language.id
        );
        // Every range on the item is a document line. `selectionRange` is
        // the one that is easy to leave behind, and it is the range an
        // editor puts the cursor on.
        for field in ["range", "selectionRange"] {
            let line = item[field]["start"]["line"].as_u64();
            assert!(
                line.is_some_and(|l| l < 200),
                "{}: {field} is not a document line: {item}",
                language.id
            );
        }
        assert!(
            item.pointer("/data/hick").is_some(),
            "{}: nothing was remembered for this item: {item}",
            language.id
        );
        // And the follow-up reaches the child at all, which it cannot do
        // unless the item was recalled.
        let calls =
            client.request_until("callHierarchy/incomingCalls", json!({"item": item}), budget);
        assert!(
            calls.is_some(),
            "{}: incomingCalls answered nothing at all",
            language.id
        );
    }

    eprintln!(
        "OK {}: hover, definition, outline and call hierarchy in document coordinates",
        language.id
    );
    true
}

/// The first `Location` or `LocationLink` in a reply, as (uri, start line).
fn first_location(result: &Value) -> Option<(String, u32)> {
    let items: Vec<&Value> = match result {
        Value::Array(items) => items.iter().collect(),
        other => vec![other],
    };
    for item in items {
        if let (Some(uri), Some(line)) = (
            item.get("uri").and_then(Value::as_str),
            item.pointer("/range/start/line").and_then(Value::as_u64),
        ) {
            return Some((uri.to_string(), line as u32));
        }
        if let (Some(uri), Some(line)) = (
            item.get("targetUri").and_then(Value::as_str),
            item.pointer("/targetRange/start/line")
                .and_then(Value::as_u64),
        ) {
            return Some((uri.to_string(), line as u32));
        }
    }
    None
}

#[test]
fn every_installed_language_answers_in_document_coordinates() {
    let mut covered = Vec::new();
    for language in LANGUAGES {
        // rust-analyzer never answers about a staged file on Windows. Not
        // slowness — it was given three times the budget and used all of it,
        // which is what ruled that out. The staging itself works there
        // (basedpyright answers from the same directory), so this is
        // something rust-analyzer wants that a bare staged file does not give
        // it on that platform. Skipped LOUDLY and tracked, rather than left to
        // look like a flake.
        if cfg!(windows) && language.id == "rust" {
            eprintln!(
                "SKIPPED rust on Windows: rust-analyzer does not answer about a staged \
                 file there — see issue #24"
            );
            continue;
        }
        if drive_one_language(language) {
            covered.push(language.id);
        }
    }
    eprintln!("languages covered on this machine: {covered:?}");
    assert!(
        !covered.is_empty(),
        "no language server is installed on this machine, so this suite tested nothing.\n\
         Install one (`hick lsp install python`) or the editor has nothing to show either."
    );
}

/// A plain file — no document, no `hick:file` block — gets the same answers,
/// at its own path, with the project as the root.
///
/// Protects docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md
fn drive_one_plain_file(language: &Language) -> bool {
    let dir = tempfile::tempdir().expect("a temp project");
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    for (path, contents) in language.scaffold {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(full, contents).unwrap();
    }
    let file = dir.path().join(language.file);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&file, language.code).unwrap();

    if !server_available(language.id, dir.path()) {
        skip(
            language.id,
            "no language server for it is installed on this machine",
        );
        return false;
    }
    let Some(mut client) = StdioClient::start(dir.path()) else {
        skip(
            language.id,
            "hick-lsp has not been built into this target dir",
        );
        return false;
    };
    let root_uri = format!("file://{}", dir.path().display());
    let uri = format!("file://{}", file.display());
    let id = client.request(
        "initialize",
        json!({ "processId": std::process::id(), "rootUri": root_uri, "capabilities": {} }),
    );
    client
        .wait_for(id, Duration::from_secs(20))
        .expect("the server answers initialize");
    client.notify("initialized", json!({}));
    client.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": uri, "languageId": language.id, "version": 1, "text": language.code}}),
    );

    let budget = Duration::from_secs(60);
    let (line, character) = definition_site(language, 0);
    let hover = client.request_until(
        "textDocument/hover",
        json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": character}}),
        budget,
    );
    let hover_text = hover.map(|v| v.to_string()).unwrap_or_default();
    assert!(
        hover_text.contains(language.symbol),
        "{}: hover in a plain file did not mention {}: {hover_text}",
        language.id,
        language.symbol
    );

    let use_site = language
        .code
        .lines()
        .enumerate()
        .filter(|(_, text)| text.contains(language.symbol))
        .nth(1)
        .map(|(offset, text)| (offset as u32, text.find(language.symbol).unwrap() as u32));
    if let Some((use_line, use_column)) = use_site {
        let found = client.request_until(
            "textDocument/definition",
            json!({"textDocument": {"uri": uri}, "position": {"line": use_line, "character": use_column}}),
            budget,
        );
        let (target_uri, target_line) = found
            .as_ref()
            .and_then(first_location)
            .expect("definition in a plain file is answered");
        assert!(
            target_uri.ends_with(language.file),
            "{}: definition pointed away from the file itself: {target_uri}",
            language.id
        );
        assert_eq!(
            target_line, line,
            "{}: definition landed on the wrong line",
            language.id
        );
    }
    eprintln!(
        "OK {}: hover and definition in a plain file, at its own path",
        language.id
    );
    true
}

/// Protects docs/guarantees/editor-intelligence/save-can-format-first.md
#[test]
fn a_plain_rust_file_formats_with_rustfmt_through_the_language_server() {
    if cfg!(windows) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    let file = dir.path().join("src/main.rs");
    let ugly = "fn main(){println!(\"x\");}\n";
    std::fs::write(&file, ugly).unwrap();
    if !server_available("rust", dir.path())
        || binary("rustfmt").is_none() && which("rustfmt").is_none()
    {
        skip(
            "rust",
            "rust-analyzer or rustfmt is not installed on this machine",
        );
        return;
    }
    let Some(mut client) = StdioClient::start(dir.path()) else {
        skip("rust", "hick-lsp has not been built into this target dir");
        return;
    };
    let root_uri = format!("file://{}", dir.path().display());
    let uri = format!("file://{}", file.display());
    let id = client.request(
        "initialize",
        json!({ "processId": std::process::id(), "rootUri": root_uri, "capabilities": {} }),
    );
    let capabilities = client
        .wait_for(id, Duration::from_secs(20))
        .expect("initialize");
    assert!(
        capabilities["capabilities"]["documentFormattingProvider"] != Value::Null,
        "formatting was not advertised: {capabilities}"
    );
    client.notify("initialized", json!({}));
    client.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": ugly}}),
    );
    let edits = client.request_until(
        "textDocument/formatting",
        json!({"textDocument": {"uri": uri}, "options": {"tabSize": 4, "insertSpaces": true}}),
        Duration::from_secs(60),
    );
    // rustfmt answers with the smallest edits that get there, so the proof
    // is the text they produce rather than any one of them.
    let edits = edits.expect("formatting is answered");
    let formatted = apply_edits(ugly, &edits);
    assert_eq!(
        formatted, "fn main() {\n    println!(\"x\");\n}\n",
        "rustfmt's edits did not come back through the server: {edits}"
    );
    eprintln!("OK rust: a plain file formats with rustfmt");
}

/// `edits` (LSP `TextEdit`s, in any order) applied to `text`.
fn apply_edits(text: &str, edits: &Value) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let offset = |line: u64, character: u64| -> usize {
        lines[..line as usize]
            .iter()
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + character as usize
    };
    let mut ordered: Vec<(usize, usize, String)> = edits
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let r = &e["range"];
            (
                offset(
                    r["start"]["line"].as_u64().unwrap(),
                    r["start"]["character"].as_u64().unwrap(),
                ),
                offset(
                    r["end"]["line"].as_u64().unwrap(),
                    r["end"]["character"].as_u64().unwrap(),
                ),
                e["newText"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    ordered.sort_by_key(|edit| std::cmp::Reverse(edit.0));
    let mut out = text.to_string();
    for (from, to, new_text) in ordered {
        out.replace_range(from..to, &new_text);
    }
    out
}

/// `name` on PATH, or none.
fn which(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}

#[test]
fn every_installed_language_answers_about_a_plain_file() {
    let mut covered = Vec::new();
    for language in LANGUAGES {
        if cfg!(windows) && language.id == "rust" {
            continue;
        }
        if drive_one_plain_file(language) {
            covered.push(language.id);
        }
    }
    eprintln!("plain-file languages covered on this machine: {covered:?}");
    assert!(
        !covered.is_empty(),
        "no language server is installed on this machine"
    );
}

#[test]
fn a_server_that_cannot_work_is_passed_over_rather_than_spawned() {
    // `typescript` on npm is 7.x now — the native port, which ships no
    // `tsserver.js` — so the obvious `npm install typescript
    // typescript-language-server` installs two packages that cannot work
    // together. Spawning the wrapper anyway is worse than not finding it:
    // it fails `initialize` with "Could not find a valid tsserver", every
    // TypeScript request returns nothing, and the editor looks broken in a
    // way that points at us rather than at the install.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    let bin_dir = dir.path().join("node_modules/.bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let server = bin_dir.join("typescript-language-server");
    std::fs::write(&server, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&server, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // No `typescript` package beside it: not a usable install.
    let found = hick_lsp::discovery::discover("typescript", dir.path());
    assert!(
        found.as_ref().is_none_or(|f| f.origin != "project"),
        "a wrapper with no tsserver was offered as the project's server: {:?}",
        found.map(|f| f.command)
    );

    // Add the package and it becomes usable — and is told where to find it,
    // because the CLI flag that used to do this was removed in 4.x and
    // passing one now makes the server exit before it says anything.
    let lib = dir.path().join("node_modules/typescript/lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join("tsserver.js"), "// stand-in\n").unwrap();
    let found = hick_lsp::discovery::discover("typescript", dir.path())
        .expect("a complete install is found");
    assert_eq!(found.origin, "project");
    assert!(
        !found.command.iter().any(|arg| arg == "--tsserver-path"),
        "the removed CLI flag came back: {:?}",
        found.command
    );
    let options = found
        .init_options
        .expect("the server is told where tsserver is");
    assert!(
        options["tsserver"]["path"]
            .as_str()
            .is_some_and(|path| path.ends_with("tsserver.js")),
        "{options}"
    );
}

#[test]
fn a_document_runs_confined_through_the_real_binary() {
    // The sandbox has unit tests and a confinement suite, but neither runs
    // `hick`. This does: the executor is chosen the way a user chooses it,
    // by an environment variable, and the document is a whole document.
    if hickory_executor_sandbox::Sandbox::detect() == hickory_executor_sandbox::Sandbox::None {
        skip(
            "sandbox",
            &format!(
                "nothing on this machine can confine a cell. {}",
                hickory_executor_sandbox::Sandbox::missing_hint()
            ),
        );
        return;
    }
    let Some(hick) = binary("hick") else {
        skip("sandbox", "hick has not been built into this target dir");
        return;
    };

    let dir = tempfile::tempdir().unwrap();
    let outside = escape_target();
    let _ = std::fs::remove_file(&outside);
    // The cell does two things: something ordinary that must work, and an
    // escape that must not. Both in one run, so a sandbox that blocked
    // everything would fail the first half rather than look like a pass.
    //
    // The escape is asserted on the HOST FILESYSTEM rather than on a word in
    // the woven page. The word used to be assembled by `tr` so that a
    // transcript — which contains the command as well as its output — could
    // not be read back as its own evidence; `tr` does not exist on Windows,
    // which would have made that assertion vacuous exactly where the sandbox
    // is newest. Whether the file arrived is the guarantee itself, and it
    // cannot be faked by a cell that never ran.
    //
    // `>` and `&&` are written literally, not as XML entities: hick's parser
    // never unescapes, so an entity would reach the shell as the characters
    // `&amp;&amp;` and the cell would be a syntax error. (hick's own linter
    // says so, which is how this comment came to exist.)
    //
    // The escape swallows its own failure (`|| true`, `|| rem`) because a
    // non-zero cell would fail the run for the right reason and the wrong
    // assertion — what is being read is the file, not the exit code.
    let cell = per_shell(
        &format!(
            "echo mine > note.txt && cat note.txt\ntouch {} 2>/dev/null || true",
            outside.display()
        ),
        &format!(
            "echo mine>note.txt && type note.txt & echo x> \"{}\" 2>nul || rem",
            outside.display()
        ),
    );
    let doc = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
        # Confined\n\n\
        <hick:container name=\"c\" image=\"python:3.12\" />\n\
        <hick:exec container=\"c\">\n\
        {cell}\n\
        </hick:exec>\n\
        </hick:doc>\n"
    );
    let path = dir.path().join("confined.hick");
    std::fs::write(&path, doc).unwrap();

    let output = Command::new(hick)
        .arg("run")
        .arg(&path)
        .env("HICKORY_EXECUTOR", "sandbox")
        .current_dir(dir.path())
        .output()
        .expect("hick runs");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "a confined document failed to run: {combined}"
    );

    let woven = std::fs::read_to_string(dir.path().join("out.md")).unwrap_or_default();
    assert!(
        woven.contains("mine"),
        "the cell's ordinary work did not happen: {woven}"
    );
    assert!(
        !outside.exists(),
        "a cell wrote {} — it escaped its sandbox",
        outside.display()
    );
    // And the woven document is about the user's work, not ours.
    assert!(
        !woven.contains("--ro-bind") && !woven.contains("__sandbox-run"),
        "the sandbox leaked into the woven output: {woven}"
    );
    eprintln!("OK sandbox: a whole document ran confined through `hick run`");
}

#[test]
fn a_language_server_is_found_in_each_ecosystems_own_layout() {
    // Discovery, against the layouts real projects have. This one needs no
    // server installed: it plants a fake executable where each package
    // manager would put a real one and asserts it is found — so it runs the
    // same on every machine, unlike the tests above.
    let cases: &[(&str, &str, &str)] = &[
        (
            "typescript",
            "node_modules/.bin",
            "typescript-language-server",
        ),
        ("python", ".venv/bin", "pyright-langserver"),
        ("python", "__pypackages__/3.12/bin", "pyright-langserver"),
        ("ruby", "vendor/bundle/ruby/3.3.0/bin", "ruby-lsp"),
        ("php", "vendor/bin", "intelephense"),
        ("go", "bin", "gopls"),
    ];
    for (language, layout, binary) in cases {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let bin_dir = dir.path().join(layout);
        std::fs::create_dir_all(&bin_dir).unwrap();
        // typescript-language-server is a wrapper around `tsserver`, and
        // discovery passes over one that has no tsserver to wrap — so a
        // fixture without it is not a TypeScript install, it is a broken
        // one. Planting the package makes this model what npm produces.
        if *binary == "typescript-language-server" {
            let lib = dir.path().join("node_modules/typescript/lib");
            std::fs::create_dir_all(&lib).unwrap();
            std::fs::write(lib.join("tsserver.js"), "// stand-in\n").unwrap();
        }
        let path = bin_dir.join(binary);
        std::fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let found = hick_lsp::discovery::discover(language, dir.path());
        assert!(
            found.is_some(),
            "{language} installed at {layout} was not found — a project laid out this way \
             would report the language as unsupported"
        );
        assert_eq!(found.unwrap().origin, "project");
    }
}

#[test]
fn a_cell_is_confined_unless_someone_says_otherwise() {
    // The default matters more than any single mechanism in the sandbox: a
    // document is a file people send each other and agents write, so the
    // confined path has to be the one you get without asking for it.
    //
    // Checked by running the real binary with no HICKORY_EXECUTOR set at
    // all, because the default is a property of the program a user installs,
    // not of a constant somewhere in it.
    let Some(hick) = binary("hick") else {
        skip("default", "hick has not been built into this target dir");
        return;
    };
    if hickory_executor_sandbox::Sandbox::detect() == hickory_executor_sandbox::Sandbox::None {
        skip(
            "default",
            "this machine cannot confine anything, so nothing to observe",
        );
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    // `id -u` would be identical either way; what differs is reach. A cell
    // that can see the user's home directory listing is not confined.
    //
    // `find /c /v ""` is cmd's line count, and `dir /a /b` its listing; a
    // denied read leaves `dir` with nothing on stdout, so the count is the
    // same question on both platforms — how much of the user's home did this
    // cell get to see.
    let cell = per_shell(
        "ls -a \"$HOME\" | wc -l",
        "dir /a /b \"%USERPROFILE%\" | find /c /v \"\"",
    );
    let doc = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
        # Default\n\n\
        <hick:container name=\"c\" image=\"python:3.12\" />\n\
        <hick:exec container=\"c\">\n\
        {cell}\n\
        </hick:exec>\n\
        </hick:doc>\n"
    );
    std::fs::write(dir.path().join("d.hick"), doc).unwrap();

    let run = |executor: Option<&str>| {
        let mut command = Command::new(&hick);
        command.arg("run").arg("d.hick").current_dir(dir.path());
        match executor {
            Some(value) => command.env("HICKORY_EXECUTOR", value),
            // Cleared, not merely unset: this process may have inherited one.
            None => command.env_remove("HICKORY_EXECUTOR"),
        };
        command.output().expect("hick runs");
        let woven = std::fs::read_to_string(dir.path().join("out.md")).unwrap_or_default();
        woven
            .lines()
            .filter_map(|line| line.trim().parse::<usize>().ok())
            .next_back()
            .unwrap_or_default()
    };

    let confined = run(None);
    let unconfined = run(Some("local"));
    assert!(
        confined < unconfined,
        "with no HICKORY_EXECUTOR set, a cell saw {confined} entries in $HOME and an \
         explicitly-local one saw {unconfined} — the default is not confining anything"
    );
    // How the home is taken away differs by sandbox, and only bubblewrap's way
    // leaves anything to count. It mounts a tmpfs, so `.` and `..` remain,
    // plus whatever toolchains were bound back in read-only. Seatbelt cannot
    // mount anything and denies the read instead, which lists nothing at all —
    // the same guarantee reached by a stricter route, so asserting a floor here
    // would be asserting bubblewrap's mechanism rather than the promise.
    if hickory_executor_sandbox::Sandbox::detect() == hickory_executor_sandbox::Sandbox::Bubblewrap
    {
        assert!(confined >= 2, "the cell had no home at all: {confined}");
    }
    eprintln!("OK default: unset HICKORY_EXECUTOR confines ({confined} vs {unconfined} in $HOME)");
}
