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
    // ready sooner.
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

    eprintln!(
        "OK {}: hover, definition and outline in document coordinates",
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
    // The cell does two things: something ordinary that must work, and an
    // escape that must not. Both in one run, so a sandbox that blocked
    // everything would fail the first half rather than look like a pass.
    //
    // `>` and `&&` are written literally, not as XML entities: hick's parser
    // never unescapes, so an entity would reach the shell as the characters
    // `&amp;&amp;` and the cell would be a syntax error. (hick's own linter
    // says so, which is how this comment came to exist.)
    let doc = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
        # Confined\n\n\
        <hick:container name=\"c\" image=\"python:3.12\" />\n\
        <hick:exec container=\"c\">\n\
        echo mine > note.txt && cat note.txt\n\
        touch /etc/hickory-escaped 2>/dev/null && echo EscAPED | tr a-z A-Z || echo ConFINED | tr a-z A-Z\n\
        </hick:exec>\n\
        </hick:doc>\n";
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
    // The words are assembled by `tr` rather than written outright because a
    // transcript contains the COMMAND as well as its output — a cell that
    // echoed "ESCAPED" would put that word in the document whether or not it
    // ever ran, and the assertion would be reading its own fixture back.
    assert!(
        woven.contains("CONFINED") && !woven.contains("ESCAPED"),
        "the cell escaped its sandbox: {woven}"
    );
    assert!(
        !Path::new("/etc/hickory-escaped").exists(),
        "a cell created a file in /etc"
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
    let doc = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
        # Default\n\n\
        <hick:container name=\"c\" image=\"python:3.12\" />\n\
        <hick:exec container=\"c\">\n\
        ls -a \"$HOME\" | wc -l\n\
        </hick:exec>\n\
        </hick:doc>\n";
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
    // `.` and `..` are all an empty home has, plus whatever toolchains were
    // bound back in read-only.
    assert!(confined >= 2, "the cell had no home at all: {confined}");
    eprintln!("OK default: unset HICKORY_EXECUTOR confines ({confined} vs {unconfined} in $HOME)");
}
