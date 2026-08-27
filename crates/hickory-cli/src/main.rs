//! `hick` — run, verify, weave, and promote executable documents.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, Result, bail};
use clap::{Parser, Subcommand};

use hickory_cli::{
    CacheMode, CheckFailure, CheckOutcome, DocRun, ExecutorChoice, RunMode, block_model_json,
    check_failures, check_outcome, expand_docs, run_doc, run_doc_cached, unverifiable_message,
    write_outputs_detailed,
};

/// What `hick --version` reports.
///
/// A downloaded binary should name the release it came from, not the
/// workspace's `Cargo.toml` number, which nobody bumps between releases and
/// which would make every unstable build claim to be `0.1.0`.
/// `scripts/dist.sh` sets `HICKORY_VERSION` when it builds an artifact; a
/// plain `cargo build` leaves it unset and falls back to the crate version.
const VERSION: &str = match option_env!("HICKORY_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

#[derive(Parser)]
#[command(
    name = "hick",
    version = VERSION,
    about = "Reproducible, verifiable, executable documents"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Execute a document (or every document in a directory) and write its
    /// outputs, including woven markdown.
    Run(RunArgs),
    /// Verification mode: re-execute and report drift or missing baselines.
    ///
    /// This is `test` and not `check` on purpose: `cargo check` promises
    /// "don't build, don't run", while this re-executes every cell in the
    /// document — the slowest, most side-effecting verb here. It is the
    /// same verb as `cargo test` / `npm test`: run it all, tell me what
    /// broke.
    ///
    /// Four outcomes, each with its own exit code — a user-facing contract
    /// CI scripts branch on:
    ///
    ///   0  verified      re-derivation matches what is committed; nothing to do
    ///   1  drifted       a committed output or <hick:transform> passage is out
    ///                    of date — re-run `hick run` (or `hick refresh`)
    ///                    and commit the result; safe for CI to auto-fix
    ///   2  unverifiable  a cell has no baseline at all — it neither executed
    ///                    nor was answered from a recording — so nothing was
    ///                    checked; give it a baseline or stop freezing it
    ///   3  expectation   a <hick:expect> did not hold: the document claims
    ///      failed        something untrue of its own output. A human decides
    ///                    whether the claim or the code is wrong; never auto-fix
    ///
    /// When more than one is present the strongest wins, in the order
    /// verified < drifted < unverifiable < expectation-failed. Unverifiable
    /// beats drift because drift computed from a document that could not
    /// fully derive is not trustworthy; a failed expectation beats both
    /// because it is the only one that says something is definitely wrong,
    /// and the only one no automation may act on by itself.
    #[command(verbatim_doc_comment)]
    Test(TestArgs),
    /// Weave a folder and keep it woven: write every document's output files,
    /// then watch. A change to a document re-weaves it; a change saved in one
    /// of the generated files is carried back into the document it came from.
    ///
    /// This is the command that makes `.hick` documents editable with any
    /// editor. Runs until interrupted.
    Up(UpArgs),
    /// Open a folder or a document in the desktop app, the way `code .` does.
    ///
    /// Returns immediately; the app outlives the terminal.
    ///
    /// `hick` and the app are separate downloads — the archive carries the
    /// command line, the app arrives as a .dmg, an AppImage or an .msi — so
    /// this finds one that may not be installed and says so plainly if it is
    /// not. `HICKORY_DESKTOP` points at a copy directly.
    Open(OpenArgs),
    /// Weave without executing: cached transcripts where present, otherwise
    /// blocks are marked never-run.
    Weave(WeaveArgs),
    /// Print the byte-precise lineage (Provenance[]) of a generated output
    /// file: which source spans produced each byte range. Weaves without
    /// executing.
    Lineage(LineageArgs),
    /// Print a document's CONTEXT provenance: for every run of lines an
    /// agent wrote, what was in front of the model when it wrote them —
    /// files at a hash and commit, the prompt, tool results, observations —
    /// derived from the session files near the document, never from what the
    /// model said. This is a different provenance from `lineage` (the weave's
    /// byte-exact derivation) and from anything a document declares with
    /// `cites=`; see docs/specs/freeform/three-provenances.md.
    Context(ContextArgs),
    /// Print a document's DECLARED provenance: every element that says
    /// `cites="…"` and what those selectors resolve to. An author's assertion
    /// — a model's via refresh, or a person's — and printed as one; it proves
    /// nothing, unlike `lineage` (the weave) and `context` (the session
    /// record). See docs/specs/freeform/three-provenances.md.
    Cites(ContextArgs),
    /// Promote a session document into a clean pipeline document.
    Promote(PromoteArgs),
    /// Adopt a plain file into a literate document, byte-exactly: wrap its
    /// content in a `<hick:file>` block and verify the weave reproduces the
    /// file before anything is written. The file does not change; its
    /// ownership does.
    Adopt(AdoptArgs),
    /// Turn transcripts into notes: every file in the notes folder's inbox
    /// becomes a `.hick` note holding the original bytes verbatim, with its
    /// summary and action items left empty and stale.
    ///
    /// Nothing here calls a model, so this works offline and without a key —
    /// `hick refresh` is what fills the summaries in later. The source file is
    /// moved into `inbox/ingested/`, never deleted.
    Ingest(IngestArgs),
    /// Import a conversation another harness recorded as a hick:session
    /// document in `sessions/`, so it opens in the app as the conversation it
    /// was and counts as context provenance. Offline, no model, nothing is
    /// deleted. Today: Claude Code's `~/.claude/projects/…/<session>.jsonl`.
    Import(ImportArgs),
    /// Distil a session into what carries to the next attempt at it: a
    /// prompt, the tests you kept, the approaches you ruled out.
    ///
    /// A re-run is not a reenactment — the second session is a complete
    /// record of itself and stands without the first, which is a draft
    /// `hick init` already gitignores. What moves between them is not the
    /// transcript; it is what you learned, distilled into inputs.
    ///
    /// This writes the opening prompt and leaves the rest as empty slots for
    /// you to fill: the tests worth carrying are the ones you READ and kept,
    /// because an agent that had already seen one implementation tends to pin
    /// that implementation's incidental choices rather than the behaviour you
    /// wanted.
    Carry(CarryArgs),
    /// Check that two documents weave identical outputs — the refactor
    /// gate: restructure a document freely, then prove no generated byte
    /// moved. Exit 0 when equivalent, 1 with a diff report when not.
    Equiv(EquivArgs),
    /// Run the AI agent on a prompt; the session is written as a
    /// hick:session document under the project's sessions/ directory.
    Agent(AgentArgs),
    /// Rewrite stale `<hick:transform>` passages from their current inputs.
    /// The ONLY command that calls a model.
    Refresh(RefreshArgs),
    /// Set up a local git repository for hick: pre-commit drift gate,
    /// .gitignore entry, an AGENTS.md section for coding agents, the `hick`
    /// MCP registration, and editor wiring — `hick-lsp` registered for
    /// *.hick where a project file can do it, and `.hick-lsp.json` written
    /// from whichever language servers this repo's editor config already
    /// names. Idempotent — re-run any time to refresh the managed blocks.
    Init(InitArgs),
    /// Read and edit a document through hashline anchors and lineage — the
    /// same tool set the built-in agent uses, for any coding agent that can
    /// run a command.
    #[command(subcommand)]
    Doc(DocCommand),
    /// Serve the document tool set over MCP on stdio, for Claude Code, Codex,
    /// Grok CLI, or any other MCP client. `hick init` writes the
    /// registration for the harnesses it finds.
    Mcp(McpArgs),
    /// Search the project the way semble does: ask in natural language or
    /// code, get ranked chunks with exact file:line. Lexical ranking works
    /// offline out of the box; `--install-model` adds semantic ranking.
    Search(SearchArgs),
    /// Deduce a folder's architecture from its code — no model, no toolchain
    /// — and print a diagram topology (scene JSON for `renderer="graph"`, or
    /// mermaid). Name-resolved via tree-sitter, so it is a place to start
    /// looking, not a compiler's call graph; SCIP in a `hick:exec` cell is
    /// the precise version, and both emit the same shape.
    ///
    /// `--refresh DOC` rewrites the named `<hick:copy>` fragment in DOC with
    /// the fresh topology instead of printing — the deterministic sibling of
    /// `hick refresh`. A diagram pasting that fragment keeps its layout: the
    /// layout lives in the diagram's own body, keyed by node id, and this
    /// command never touches it.
    Diagram(DiagramArgs),
    /// Show the commits a re-emission of this folder's documents WOULD
    /// produce. Emits nothing.
    ///
    /// One stage is one commit — a stage being a document plus the files it
    /// generates — so the shape comes from the document rather than from the
    /// order you typed, and a re-emission is deterministic given the
    /// document. Commits above the publication floor are drafts a re-emission
    /// would replace; below it they are records, and nothing may re-produce
    /// one.
    Emit(EmitArgs),
    /// Run the broker: one road out for a sealed machine, with a toll booth
    /// on it.
    ///
    /// A first-party CONNECT proxy and nothing else. It does not execute
    /// documents, does not store them, and never accepts a connection from
    /// outside the local network. Run it on a DIFFERENT machine than the
    /// sealed one.
    ///
    /// Policy is per host: `allow` forwards, `deny` refuses with a sentence
    /// the agent can repeat. An unlisted host is denied.
    #[command(subcommand)]
    Broker(BrokerCommand),
    /// What every writer did to this folder, below the last commit — and how
    /// to go back. Local to you and to this machine; never committed, never
    /// shared, evicted on a timer.
    History(HistoryArgs),
    /// Whether this machine is sealed, and whether the seal actually holds.
    ///
    /// A sealed machine holds no credential worth stealing and has exactly
    /// one road out. It is NOT airgapped — a machine that talks to a model is
    /// on a network — and this never claims otherwise.
    Sealed(SealedArgs),
    /// This machine's identity, and the other machines you have paired it
    /// with.
    ///
    /// A machine is a keypair; a fleet is a mutual list of public keys under
    /// your own state directory, never in the repository. There is no
    /// account, no directory, and no server to sign in to — and revoking a
    /// machine is deleting its key, which is complete, because no server
    /// holds a session you cannot reach.
    ///
    /// `hick fleet serve` makes this machine's session reachable by them;
    /// `hick fleet attach --listen` puts it on a local port, so the app or a
    /// browser pointed there is looking at that machine.
    #[command(subcommand)]
    Fleet(FleetCommand),
    /// Show or install the language servers that power the editor.
    #[command(subcommand)]
    Lsp(LspCommand),
    /// Show or install the code indexers that answer questions about the
    /// whole project rather than about one file. In ADDITION to the language
    /// servers, never in place of them.
    #[command(subcommand)]
    Index(IndexCommand),
    /// Show or install the debug adapters that power breakpoints.
    #[command(subcommand)]
    Dap(DapCommand),
    /// Show or install the backends that evaluate a table's formulas.
    #[command(subcommand)]
    Formula(FormulaCommand),
    /// Show or install the local model that ranks completions and search.
    #[command(subcommand)]
    Model(ModelCommand),
    /// Internal: record what an unwatched edit moved, before it is committed.
    ///
    /// Not for people. The pre-commit hook calls it, and it does nothing at
    /// all unless continuity is on for the project. It never fails a commit:
    /// the check IS the repair, and a rule with no escape hatch gets the hook
    /// disabled entirely.
    #[command(name = "repair", hide = true)]
    Repair(RepairArgs),
    /// Internal: git's merge driver for `*.hick`.
    ///
    /// Not for people. `hick init` defines it in this clone's config and
    /// routes `*.hick` at it from `.gitattributes`; git invokes it with the
    /// three sides of a merge. Running it by hand merges nothing you meant to
    /// merge.
    #[command(name = "merge-driver", hide = true)]
    MergeDriver(MergeDriverArgs),
    /// Internal: run a command inside a Windows AppContainer.
    ///
    /// Not for people. On Windows the sandbox is applied by the process that
    /// CREATES the confined process, so `hick` re-invokes itself here rather
    /// than shipping a second binary to do it. Hidden because it is an
    /// implementation detail of `HICKORY_EXECUTOR=sandbox`, and running it by
    /// hand confines nothing you would want confined.
    #[command(name = "__sandbox-run", hide = true)]
    SandboxRun(SandboxRunArgs),
}

#[derive(Subcommand)]
enum BrokerCommand {
    /// Serve the proxy.
    Serve {
        /// Address to bind. Loopback by default: a broker exposed to the
        /// internet is an open proxy, which is the one thing it must never
        /// become.
        #[arg(long, default_value = "127.0.0.1:7749")]
        bind: String,
    },
    /// What the policy says, and where the log is.
    Status,
    /// Set one host's verb.
    Allow {
        host: String,
    },
    Deny {
        host: String,
    },
    /// Show what the broker did.
    Log {
        /// How many lines, newest last.
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

#[derive(clap::Args)]
struct SealedArgs {
    /// Check the seal and report every finding.
    #[arg(long)]
    check: bool,
    /// Seal this machine (or, with --broker, change where it reaches the
    /// model). Sealing is asserted by the person standing at the machine.
    #[arg(long)]
    set: bool,
    /// The broker this machine reaches the model through.
    #[arg(long)]
    broker: Option<String>,
    /// Unseal it.
    #[arg(long)]
    unset: bool,
}

#[derive(Subcommand)]
enum FleetCommand {
    /// This machine's name and key fingerprint.
    Whoami,
    /// Pair with another machine using a short phrase — no line to copy.
    ///
    /// `--new` on one machine prints a phrase and waits; typing that phrase on
    /// the other pairs them BOTH ways in one exchange, which the invitation
    /// flow needs two of.
    ///
    /// The phrase is a one-time secret with a deadline: both machines derive
    /// the same throwaway keypair from it, one listens under it and the other
    /// dials it, and they trade real keys over that. Nobody is trusted for
    /// identity, and no server is involved — but anyone who learns the phrase
    /// inside the window can pair too, so read it out, use it once, and
    /// compare the fingerprints afterwards.
    Pair {
        /// Generate a phrase and wait for the other machine.
        #[arg(long)]
        new: bool,
        /// The phrase the other machine printed.
        phrase: Option<String>,
        /// Say this machine is a phone.
        #[arg(long)]
        phone: bool,
    },
    /// Print an invitation for another machine to accept.
    ///
    /// Self-contained: it carries this machine's name and PUBLIC key and
    /// nothing that has to be fetched, so it works over a LAN, a phone
    /// screenshot, or read aloud. Enrolment is SSH's ceremony — run this on
    /// each machine and accept it on the other.
    Invite {
        /// Say this machine is a phone: it reads and captures, and can never
        /// be granted `execute` because it has no executor.
        #[arg(long)]
        phone: bool,
    },
    /// Accept an invitation another machine printed.
    Accept {
        /// The whole `hick-fleet:…` line.
        invitation: String,
    },
    /// Every machine in this fleet, with its fingerprint and grants.
    List,
    /// Make this machine's session reachable by the machines it has paired
    /// with, and serve them under their grants.
    ///
    /// Durable state never travels over this: two machines editing the same
    /// document commit and push, and git reconciles. What crosses is the
    /// LIVE room. Close both sessions without committing and the machines
    /// diverge — that is correct, because this is a window onto a session and
    /// not a replication protocol.
    Serve {
        /// The loopback address of the session to expose — what `hick up` or
        /// the app is already serving.
        #[arg(long, default_value = "127.0.0.1:7317")]
        session: String,
    },
    /// Attach to another machine's session.
    ///
    /// With `--listen`, serve it on a local port: point the desktop app or a
    /// browser there and the window is that machine's session, with your
    /// grants already applied. Without it, make one request and print the
    /// answer — enough to prove the channel and the refusals from a terminal.
    Attach {
        /// The machine's name, as `hick fleet list` shows it.
        machine: String,
        /// Its dialable address, from `hick fleet serve` on that machine.
        #[arg(long)]
        addr: String,
        /// Serve that session here, on this loopback address, until
        /// interrupted. Loopback only: the port IS the remote session.
        #[arg(long)]
        listen: Option<String>,
        /// The one request to make, when not listening.
        #[arg(long, default_value = "/api/health")]
        path: String,
        #[arg(long, default_value = "GET")]
        method: String,
    },
    /// Grant or revoke a verb for one machine.
    Grant {
        /// The machine's name.
        machine: String,
        /// `view`, `edit`, or `execute`.
        grant: String,
        /// Take it away instead of giving it.
        #[arg(long)]
        revoke: bool,
    },
    /// Remove a machine. Revocation is deleting its key, and it is complete.
    Remove { machine: String },
}

#[derive(Subcommand)]
enum LspCommand {
    /// List every language server this can install, and whether it can.
    ///
    /// Installing is never automatic: it fetches from the network and runs
    /// the package's own setup scripts, and this tool does neither without
    /// being asked. What is already on your machine is always preferred.
    List,
    /// Install a language's server into `.hick-cache/servers/`, confined.
    ///
    /// The installer runs in the same sandbox a document's cells run in —
    /// `npm install` and `uv pip install` execute arbitrary setup code, so
    /// they are treated as what they are. Nothing is written outside
    /// `.hick-cache/`, which `hick init` already keeps out of git.
    Install(LspInstallArgs),
}

#[derive(Subcommand)]
enum ModelCommand {
    /// Whether the local model is installed, and what it is for.
    List,
    /// Download the embedding model into `.hick-cache/models/embed/`.
    ///
    /// The one place this product touches the network, and only when asked.
    /// Everything it improves works without it: completions fall back to
    /// frequency, search to lexical ranking. Neither is a degraded mode that
    /// warns at you — they are what the tool does on a machine that has never
    /// been online.
    Install(ModelInstallArgs),
}

#[derive(clap::Args)]
struct ModelInstallArgs {
    /// The project to install into. Defaults to the current directory.
    #[arg(long)]
    root: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
enum FormulaCommand {
    /// List the languages formulas can be written in, and whether this
    /// machine can run each one.
    ///
    /// Unlike `lsp` and `dap`, nothing here is fetched. A formula backend is
    /// a small script this binary carries; what a machine needs is the
    /// interpreter, which it either has or does not.
    List,
    /// Write a language's backend into `.hick-cache/formula/`.
    ///
    /// Rarely necessary by hand: the first formula in a document installs
    /// what it needs. Writing sixty lines into a cache with no network
    /// involved is not an act worth asking permission for — which is exactly
    /// why it can be automatic here and cannot be for a language server.
    Install(FormulaInstallArgs),
}

#[derive(clap::Args)]
struct FormulaInstallArgs {
    /// Languages to install. All of them when none is named.
    languages: Vec<String>,
    /// The project to install into. Defaults to the current directory.
    #[arg(long)]
    root: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
enum DapCommand {
    /// List every debug adapter this can install, and whether it can.
    ///
    /// Installing is never automatic, for the same reason it is not for
    /// language servers: it fetches from the network and runs the package's
    /// own setup scripts. What is already on your machine is preferred.
    List,
    /// Install a language's debug adapter into `.hick-cache/adapters/`,
    /// confined.
    Install(LspInstallArgs),
}

#[derive(clap::Args)]
struct LspInstallArgs {
    /// Languages to install servers for, e.g. `python`. Omit for all of them.
    languages: Vec<String>,
    /// The project to install into. Defaults to the current directory.
    #[arg(long)]
    root: Option<PathBuf>,
}

#[derive(clap::Args)]
struct DiagramArgs {
    /// The folder to read. Defaults to the current directory.
    path: Option<PathBuf>,
    /// Output form: `scene` (JSON for a graph diagram / copy fragment) or
    /// `mermaid`.
    #[arg(long, default_value = "scene")]
    format: String,
    /// Node granularity: `dir` (one node per top-level directory) or `file`.
    #[arg(long, default_value = "dir")]
    group: String,
    /// Rewrite this document's topology fragment in place instead of
    /// printing.
    #[arg(long)]
    refresh: Option<PathBuf>,
    /// The `<hick:copy id=…>` fragment `--refresh` rewrites.
    #[arg(long, default_value = "arch-topology")]
    fragment: String,
}

#[derive(clap::Args)]
struct SearchArgs {
    /// The query, in natural language or code. Omit with --related or
    /// --install-model.
    query: Option<String>,
    /// The folder to search. Defaults to the current directory.
    #[arg(long)]
    root: Option<PathBuf>,
    /// How many results to show.
    #[arg(long = "top-k", default_value_t = 8)]
    top_k: usize,
    /// Find code related to FILE:LINE (e.g. src/app.py:42) instead of
    /// answering a query.
    #[arg(long)]
    related: Option<String>,
    /// Download the semantic embedding model (~30 MB) into .hick-cache/.
    /// Explicit and never automatic — the only part of search that touches
    /// the network. Search works without it, ranking lexically.
    #[arg(long = "install-model")]
    install_model: bool,
}

#[derive(clap::Args)]
struct SandboxRunArgs {
    /// The one directory the confined command may write; also its cwd.
    #[arg(long)]
    workdir: PathBuf,
    /// Grant outbound network. Absent means no network at all.
    #[arg(long)]
    allow_network: bool,
    /// The command, after `--`.
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

#[derive(clap::Args)]
struct McpArgs {
    /// The document tool calls act on when they name none.
    #[arg(long = "doc")]
    doc: Option<PathBuf>,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    #[arg(long = "features", value_delimiter = ',')]
    features: Vec<String>,
}

/// The document tool set. Every subcommand opens a session on DOC, runs one
/// tool, and prints the observation.
///
/// The workflow they are built for: read a surface, edit against the hashes
/// you just read, verify. Anchors are content hashes, so an edit against a
/// line that has changed since you read it is refused rather than misapplied
/// — re-read and try again.
#[derive(Subcommand)]
enum DocCommand {
    /// Print the document source, every line prefixed with its 4-hex content
    /// hash. Those hashes are the anchors `edit` takes.
    Read(DocReadArgs),
    /// Print a woven output file, hashline-rendered. `--lineage` also marks
    /// which lines are editable and where each came from in the document.
    ReadOutput(DocReadOutputArgs),
    /// Read any project file, hashline-rendered, read-only — a data export,
    /// a config, a source file. Recorded in the session as context.
    ReadFile(DocReadFileArgs),
    /// Edit an output file; the change is mapped back into the document
    /// byte-exactly through lineage. This is how CODE should be edited.
    EditOutput(DocEditArgs),
    /// Edit the document source directly. This is how STRUCTURE and PROSE
    /// should be edited — and where lineage refusals send you.
    Edit(DocEditArgs),
    /// Execute the document for real (every exec cell, every expectation) and
    /// write its outputs. Run this before calling an edit done.
    Verify(DocVerifyArgs),
}

#[derive(clap::Args)]
struct DocCommonArgs {
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param, global = true)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    #[arg(long = "features", value_delimiter = ',', global = true)]
    features: Vec<String>,
    /// Print `{"tool", "ok", "text"}` instead of the observation alone.
    #[arg(long = "json", global = true)]
    json: bool,
    /// Append this call and its result to a `hick:session` document, creating
    /// it if needed. Defaults to `HICKORY_SESSION`; unset, nothing is
    /// recorded. This is how work done by an OUTSIDE agent still produces a
    /// session `hick promote` can compact.
    #[arg(long = "session", global = true)]
    session: Option<PathBuf>,
}

#[derive(clap::Args)]
struct DocReadArgs {
    /// The `.hick` document. Omit when the working directory holds exactly
    /// one.
    doc: Option<PathBuf>,
    /// Read an UPSTREAM document instead (by name or path). The primary
    /// document's `hick:upstream` closure is listed in the output.
    #[arg(long = "upstream")]
    upstream: Option<String>,
    #[command(flatten)]
    common: DocCommonArgs,
}

#[derive(clap::Args)]
struct DocReadOutputArgs {
    /// The `.hick` document. Omit when the working directory holds exactly
    /// one.
    doc: Option<PathBuf>,
    /// Which output file, as the document names it.
    #[arg(long = "path")]
    path: String,
    /// Annotate each range with where it came from and whether it can be
    /// edited through the output. Read this before editing.
    #[arg(long = "lineage")]
    lineage: bool,
    #[command(flatten)]
    common: DocCommonArgs,
}

#[derive(clap::Args)]
struct DocReadFileArgs {
    /// The `.hick` document whose session this read belongs to. Omit when
    /// the working directory holds exactly one.
    doc: Option<PathBuf>,
    /// The file, relative to the document's directory. Must be inside the
    /// project (the document's git repository, or its directory).
    #[arg(long = "path")]
    path: String,
    /// First line to show (1-based).
    #[arg(long = "from")]
    from: Option<usize>,
    /// Last line to show (1-based, inclusive).
    #[arg(long = "to")]
    to: Option<usize>,
    #[command(flatten)]
    common: DocCommonArgs,
}

#[derive(clap::Args)]
struct DocEditArgs {
    /// The `.hick` document. Omit when the working directory holds exactly
    /// one.
    doc: Option<PathBuf>,
    /// Which output file (edit-output only).
    #[arg(long = "path")]
    path: Option<String>,
    /// Replace this line run: `hash` for one line, `first..last` for a range.
    #[arg(long = "run")]
    run: Option<String>,
    /// Insert BELOW this line instead of replacing anything; `^` inserts at
    /// the top of the file.
    #[arg(long = "after")]
    after: Option<String>,
    /// Which occurrence to use when the anchor matches more than one place
    /// (1-based).
    #[arg(long = "occurrence")]
    occurrence: Option<usize>,
    /// Edit an UPSTREAM document instead (edit only).
    #[arg(long = "upstream")]
    upstream: Option<String>,
    /// The replacement text. Omit (or pass `-`) to read it from stdin, which
    /// is what code should come through. Pass an empty string to DELETE the
    /// anchored run.
    #[arg(long = "input")]
    input: Option<String>,
    #[command(flatten)]
    common: DocCommonArgs,
}

#[derive(clap::Args)]
struct DocVerifyArgs {
    /// The `.hick` document. Omit when the working directory holds exactly
    /// one.
    doc: Option<PathBuf>,
    #[command(flatten)]
    common: DocCommonArgs,
}

#[derive(clap::Args)]
struct RunArgs {
    /// A `.hick` document or a directory of documents.
    path: PathBuf,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    /// Shorthand for `--param features=a,b`; feeds `<hick:when test="...">`.
    #[arg(long = "features", value_delimiter = ',')]
    features: Vec<String>,
    /// Output directory (default: each document's own directory).
    #[arg(long = "out")]
    out: Option<PathBuf>,
    /// Record every executed cell into the project's
    /// `.hick-cache/transcripts/`, and answer a cell from its recording when
    /// one still matches. A cell that declares `freeze="true"` records itself
    /// on its first run without this flag; `--cache` extends the same
    /// treatment to every other cell in the document.
    #[arg(long)]
    cache: bool,
    /// Freeze every cell that does not declare otherwise: answer it from its
    /// recording rather than executing it. A cell's own `freeze="false"`
    /// still wins. A cell with no recording yet is executed once and
    /// recorded — establishing the baseline is what `run` is for. To assert
    /// that every cell is already recorded, without executing anything, use
    /// `hick test --freeze`, which never records.
    #[arg(long)]
    freeze: bool,
    /// Emit the block model as JSON on stdout instead of a summary.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct TestArgs {
    /// `.hick` documents or directories of them — as many as you like.
    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    /// Shorthand for `--param features=a,b`; feeds `<hick:when test="...">`.
    #[arg(long = "features", value_delimiter = ',')]
    features: Vec<String>,
    /// Directory holding the committed outputs (default: each document's own
    /// directory).
    #[arg(long = "out")]
    out: Option<PathBuf>,
    /// Freeze every cell that does not declare otherwise: verify it against
    /// its recording instead of executing it, and report any cell with no
    /// recording as unverifiable (exit 2).
    ///
    /// There is deliberately no --cache here, and `test` writes no recording
    /// under any flag: it must never create the baseline it then compares
    /// against. Record with `hick run <doc>`.
    #[arg(long)]
    freeze: bool,
    /// Emit the block model as JSON on stdout in addition to failures.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct UpArgs {
    /// A directory of documents, or a single `.hick` document.
    /// Default: the working directory.
    path: Option<PathBuf>,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    #[arg(long = "features", value_delimiter = ',')]
    features: Vec<String>,
    /// Execute exec cells on every change rather than weaving from recorded
    /// transcripts.
    ///
    /// Without this, saving a file never runs anything: cells are answered
    /// from `.hick-cache/transcripts/` and cells with no recording are marked
    /// never-run. With it, a changed document is re-executed in full, so the
    /// generated files always reflect code that really ran — at the cost of
    /// running that code every time you save.
    #[arg(long)]
    run: bool,
}

#[derive(clap::Args)]
struct WeaveArgs {
    /// A `.hick` document.
    path: PathBuf,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    /// Shorthand for `--param features=a,b`; feeds `<hick:when test="...">`.
    #[arg(long = "features", value_delimiter = ',')]
    features: Vec<String>,
    /// Output directory (default: the document's directory).
    #[arg(long = "out")]
    out: Option<PathBuf>,
    /// Emit the block model as JSON on stdout instead of a summary.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct ContextArgs {
    /// A `.hick` document.
    doc: PathBuf,
    /// Emit JSON on stdout instead of a summary.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct LineageArgs {
    /// A `.hick` document.
    doc: PathBuf,
    /// The generated output file to trace (its `<hick:file path>` value).
    #[arg(long = "output")]
    output: String,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Emit the Provenance[] JSON on stdout instead of a summary.
    #[arg(long)]
    json: bool,
    /// Replay: report the lineage the document had AT this commit.
    ///
    /// The old document is read out of git — nothing is checked out — and
    /// woven without executing anything, so the answer is recomputed exactly
    /// rather than recalled from a store. It gives the state *at* a commit,
    /// never the thread between two of them.
    ///
    /// Replay works back to the last grammar change: an older document is
    /// woven by today's binary, and this product reserves the right to change
    /// the grammar. Past that boundary the tool says so in those words.
    #[arg(long = "at", value_name = "COMMIT")]
    at: Option<String>,
    /// List the commits that touched this document, newest first, and stop.
    /// The values `--at` takes.
    #[arg(long)]
    history: bool,
}

#[derive(clap::Args)]
struct OpenArgs {
    /// A folder of documents, or a single `.hick` document. Defaults to the
    /// working directory, which is what makes `hick open .` the whole gesture.
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[derive(clap::Args)]
struct EmitArgs {
    /// The folder whose documents to plan. Defaults to the current one.
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Emit the plan as JSON instead of a summary.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
struct CarryArgs {
    /// The session document to distil — a file under `sessions/`.
    session: PathBuf,
    /// Where to write it. Defaults to `carries/<date>-<slug>.hick` beside the
    /// sessions folder, never inside it: `sessions/` is gitignored, and a
    /// carry that vanished with the session it distils would be pointless.
    #[arg(long = "out")]
    out: Option<PathBuf>,
}

/// `hick history …`
///
/// Every one of these works offline and in a folder that is not a repository,
/// which is most of the point: git's resolution is a commit, and this is the
/// interval below one.
/// `hick index …`
#[derive(Debug, clap::Subcommand)]
enum IndexCommand {
    /// Which indexers exist, and which this machine can install.
    List,
    /// Fetch an indexer, confined.
    Install {
        /// Languages to install. Empty installs everything possible.
        languages: Vec<String>,
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Run an indexer over this project and write an index.
    ///
    /// The index is produced and **nothing reads it yet** — see
    /// `docs/specs/freeform/an-index-beside-the-language-server.md` for the
    /// licence decision that is deliberately left open.
    Build {
        /// The language to index.
        language: String,
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Debug, clap::Args)]
struct HistoryArgs {
    #[command(subcommand)]
    command: Option<HistoryCommand>,
    /// A file, to filter to its own timeline — a filter over the same acts,
    /// never a second structure. `hick history` with no argument is the whole
    /// list, which is the thing people reach for.
    path: Option<String>,
    /// How many to show.
    #[arg(long, default_value_t = 20)]
    limit: usize,
}

#[derive(Debug, clap::Subcommand)]
enum HistoryCommand {
    /// What one act did, file by file.
    Show {
        /// The act id, or an unambiguous prefix of one.
        act: String,
    },
    /// Put back what an act overwrote.
    ///
    /// Only where the bytes on disk are still what the act wrote; a file that
    /// has moved on since is reported rather than silently skipped.
    Revert {
        act: String,
        /// One file out of the act, rather than all of them.
        #[arg(long)]
        path: Option<String>,
    },
    /// Forget what is stored — everything, or everything about one path.
    ///
    /// An honest verb rather than a redactor this product does not have. A
    /// file that briefly contained a secret is the obvious case.
    Forget {
        #[arg(long)]
        path: Option<String>,
        /// Forget the whole store for this folder.
        #[arg(long, conflicts_with = "path")]
        all: bool,
    },
}

#[derive(clap::Args)]
struct RepairArgs {
    /// A directory inside the repository. Defaults to the current one.
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[derive(clap::Args)]
struct MergeDriverArgs {
    /// %O — the common ancestor's version.
    #[arg(long)]
    base: PathBuf,
    /// %A — our version, AND the file the result must be written to.
    #[arg(long)]
    ours: PathBuf,
    /// %B — their version.
    #[arg(long)]
    theirs: PathBuf,
    /// %L — the conflict-marker size git asked for.
    #[arg(long = "marker-size", default_value_t = 7)]
    marker_size: usize,
    /// %P — the path in the work tree, for messages.
    #[arg(long, default_value = "(unknown path)")]
    path: String,
}

#[derive(clap::Args)]
struct PromoteArgs {
    /// A `hick:session` document.
    session: PathBuf,
    /// Where to write the promoted pipeline (default: stdout).
    #[arg(long = "out")]
    out: Option<PathBuf>,
}

#[derive(clap::Args)]
struct AdoptArgs {
    /// The plain file to adopt (not a `.hick` document, not binary).
    file: PathBuf,
    /// Append a block to this existing document instead of creating
    /// `<stem>.hick` beside the file. The document must live at or above
    /// the file, and its other outputs must come out byte-identical.
    #[arg(long = "into")]
    into: Option<PathBuf>,
}

#[derive(clap::Args)]
struct IngestArgs {
    /// The notes folder whose inbox to drain — or, with `--from`, the `.hick`
    /// document whose cell to ingest. Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Ingest this one file instead of draining the inbox. It is still moved
    /// into `inbox/ingested/` afterwards.
    #[arg(long)]
    file: Option<PathBuf>,
    /// Ingest the output volume of the cell this id names, into the document
    /// given as the path: `hick ingest --from '#scaffold' app.hick`.
    ///
    /// The other door. `dotnet new webapi` writes forty files nobody typed,
    /// and this is how they become bytes the document owns — ordinary
    /// `hick:file` blocks under an `<hick:ingested>` element recording the
    /// run — so a clone reproduces the scaffold without running it again.
    /// Files the project's .gitignore would ignore are filtered out and
    /// counted; a file that is not UTF-8 text is refused by name, because a
    /// document body is raw bytes with no encoding to hide a binary in.
    #[arg(long, value_name = "SELECTOR")]
    from: Option<String>,
}

#[derive(clap::Args)]
struct ImportArgs {
    #[command(subcommand)]
    source: ImportSource,
}

#[derive(clap::Subcommand)]
enum ImportSource {
    /// A Claude Code transcript: `~/.claude/projects/<project>/<session>.jsonl`.
    /// Each file becomes `sessions/<date>-<title>.hick`; the same file
    /// imported twice names one document and is skipped the second time.
    ClaudeCode {
        /// One or more `.jsonl` transcripts.
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Where to write. Defaults to `./sessions`.
        #[arg(long, default_value = "sessions")]
        out: PathBuf,
        /// Overwrite a session that was imported before.
        #[arg(long)]
        force: bool,
        /// Print the converted document instead of writing it (one file).
        #[arg(long)]
        stdout: bool,
    },
}

#[derive(clap::Args)]
struct EquivArgs {
    /// The document as it stands (the baseline).
    first: PathBuf,
    /// The restructured document to compare against it.
    second: PathBuf,
}

#[derive(clap::Args)]
struct InitArgs {
    /// Directory inside the git repository to initialize (default: cwd).
    #[arg(default_value = ".")]
    dir: PathBuf,
}

#[derive(clap::Args)]
struct RefreshArgs {
    /// A `.hick` document (or directory of them).
    path: PathBuf,
    /// Rewrite every transform, not only the stale ones.
    #[arg(long)]
    all: bool,
    /// Report what would be rewritten without calling a model.
    #[arg(long = "dry-run")]
    dry_run: bool,
    /// Model id override (interpreted by the selected provider).
    #[arg(long = "model")]
    model: Option<String>,
    /// LLM provider: anthropic, openai, deepseek, grok, openrouter, or gab. Defaults to
    /// HICKORY_LLM_PROVIDER, else to whichever provider's API key is set
    /// (anthropic when several are).
    #[arg(long = "provider")]
    provider: Option<String>,
}

#[derive(clap::Args)]
struct AgentArgs {
    /// The task prompt for the agent.
    prompt: String,
    /// A `.hick` document to give the agent as context.
    #[arg(long = "doc")]
    doc: Option<PathBuf>,
    /// Project directory (sessions land in `<dir>/sessions/`; default: cwd).
    #[arg(long = "dir")]
    dir: Option<PathBuf>,
    /// Model id override (default: the provider's default model).
    #[arg(long = "model")]
    model: Option<String>,
    /// LLM provider: anthropic, openai, deepseek, grok, openrouter, or gab. Defaults to
    /// HICKORY_LLM_PROVIDER, else to whichever provider's API key is set
    /// (anthropic when several are).
    #[arg(long = "provider")]
    provider: Option<String>,
    /// Maximum LLM turns before giving up.
    #[arg(long = "max-turns", default_value_t = 20)]
    max_turns: usize,
}

/// How much stack the work gets, on every platform.
///
/// Windows gives a process's main thread 1 MiB; Linux and macOS give 8. The
/// runtime's `block_on` runs on that thread, so the whole command — parse,
/// weave, execute, the async chain under all of it — lives inside whatever the
/// platform happened to choose. `hick up --run` overflowed 1 MiB in a debug
/// build and died with "thread 'main' has overflowed its stack", taking the
/// loop down mid-edit (#23).
///
/// It is depth, not a runaway: an unbounded recursion overflows whatever it is
/// given, and the release build of the same command on the same machine does
/// not crash — release frames are simply smaller. So the fix is to stop
/// inheriting a number that differs 8x across platforms, rather than to chase
/// a recursion that is not there. 16 MiB is reserved address space, not
/// committed memory; the pages are only ever touched if something needs them.
const STACK_SIZE: usize = 16 * 1024 * 1024;

fn main() -> ExitCode {
    match std::thread::Builder::new()
        .name("hick".to_string())
        .stack_size(STACK_SIZE)
        .spawn(run)
        .expect("failed to start the main thread")
        .join()
    {
        Ok(code) => code,
        // Re-raise rather than turning it into an exit code: a panic should
        // still look like a panic, with the same status it has always had.
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

fn run() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let cli = Cli::parse();

    let runtime = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");
    let outcome = runtime.block_on(async {
        match cli.command {
            Command::Run(args) => cmd_run(args).await,
            Command::Test(args) => cmd_test(args).await,
            Command::Up(args) => cmd_up(args).await,
            Command::Weave(args) => cmd_weave(args).await,
            Command::Lineage(args) => cmd_lineage(args).await,
            Command::Context(args) => cmd_context(args),
            Command::Cites(args) => cmd_cites(args),
            Command::Promote(args) => cmd_promote(args),
            Command::Adopt(args) => cmd_adopt(args).await,
            Command::Ingest(args) => cmd_ingest(args).await,
            Command::Import(args) => cmd_import(args),
            Command::Carry(args) => cmd_carry(args),
            Command::Emit(args) => cmd_emit(args).await,
            Command::Open(args) => cmd_open(args),
            Command::Equiv(args) => cmd_equiv(args).await,
            Command::Agent(args) => cmd_agent(args).await,
            Command::Refresh(args) => cmd_refresh(args).await,
            Command::Init(args) => cmd_init(args),
            Command::Doc(cmd) => cmd_doc(cmd).await,
            Command::Search(args) => cmd_search(args).await,
            Command::Diagram(args) => cmd_diagram(args),
            Command::Broker(cmd) => cmd_broker(cmd).await,
            Command::Sealed(args) => cmd_sealed(args),
            Command::Fleet(cmd) => cmd_fleet(cmd).await,
            Command::Lsp(cmd) => cmd_lsp(cmd),
            Command::Formula(cmd) => cmd_formula(cmd),
            Command::Model(cmd) => cmd_model(cmd).await,
            Command::Dap(cmd) => cmd_dap(cmd),
            Command::Repair(args) => cmd_repair(args),
            Command::Index(command) => cmd_index(command),
            Command::History(args) => cmd_history(args),
            Command::MergeDriver(args) => cmd_merge_driver(args),
            Command::SandboxRun(args) => cmd_sandbox_run(args),
            Command::Mcp(args) => {
                hickory_cli::mcp::serve(
                    args.doc,
                    params_with_features(&args.params, &args.features),
                )
                .await?;
                Ok(ExitCode::SUCCESS)
            }
        }
    });

    match outcome {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

/// Say when a cell's OUTPUT looks like it carried a credential.
///
/// A warning, and deliberately not the refusal an anchored terminal makes.
/// The spec is right that this is the same exposure — a command that *prints*
/// a token has always been recorded by the transcript, in every cell — but
/// its argument for stopping does not carry over, and the difference is worth
/// stating rather than quietly copying the mechanism.
///
/// In a terminal the recording is automatic and unattended, and a false
/// positive costs a suspension a person can see and resume from. In a
/// document there is nothing to resume: declining to record a cell's output
/// changes what the document weaves, so the same false positive would report
/// drift, fail `hick test`, and keep failing until somebody changed the
/// program's output. A heuristic that can break a build is a different trade
/// from one that can pause a recording.
///
/// So the author — who is present, and who wrote the command that produced
/// the output — is told, and the output IS recorded. Anything stronger than
/// this needs a way for a document to say "yes, I meant that", which does not
/// exist yet.
fn warn_about_secrets_in_output(run: &DocRun) {
    let mut flagged: Vec<(&str, usize)> = Vec::new();
    for (container, entries) in &run.result.transcripts {
        for entry in entries {
            let looks = entry
                .output
                .lines()
                .any(hick_term::anchor::looks_like_a_secret);
            if looks {
                flagged.push((container.as_str(), entry.source_line.unwrap_or(0)));
            }
        }
    }
    for (container, line) in flagged {
        eprintln!(
            "  note: output of the `{container}` cell at line {line} looks like it contains a \
             credential, and it HAS been recorded.\n        \
             A transcript is committed with the document. If that was not meant, keep the \
             secret out of the output (read it from the environment, or print a placeholder) \
             and run again."
        );
    }
}

fn print_run_summary(run: &DocRun, outputs: &hickory_cli::WrittenOutputs) {
    let n_expect = run.result.expectations.len();
    let n_failed = run.result.expectations.iter().filter(|o| !o.passed).count();
    eprintln!(
        "{}: {} file(s) written, {} expectation(s) ({} failed)",
        run.doc_path.display(),
        outputs.written.len(),
        n_expect,
        n_failed
    );
    for path in &outputs.written {
        eprintln!("  wrote {}", path.display());
    }
    for path in &outputs.preserved {
        eprintln!(
            "  kept {} (no recording for the cell that produces it; \
             the file on disk was left as it is)",
            path.display()
        );
    }
    warn_about_secrets_in_output(run);
    for outcome in &run.result.expectations {
        if !outcome.passed {
            eprintln!(
                "  expectation FAILED ({} line {}): {}",
                outcome.container.as_deref().unwrap_or("agent cell"),
                outcome.line,
                outcome.detail
            );
        }
    }
}

/// Fold `--features a,b` into the param list as the `features` variable —
/// the same variable `hick-literate`'s feature system reads (enabled
/// features then become condition variables for `<hick:when test="...">`).
fn params_with_features(params: &[(String, String)], features: &[String]) -> Vec<(String, String)> {
    let mut params = params.to_vec();
    if !features.is_empty() {
        params.push(("features".to_string(), features.join(",")));
    }
    params
}

async fn cmd_run(args: RunArgs) -> Result<ExitCode> {
    let executor_choice = ExecutorChoice::from_env()?;
    let docs = expand_docs(&args.path)?;
    let params = params_with_features(&args.params, &args.features);
    let mut json_blocks = Vec::new();
    for doc_path in &docs {
        let run = run_doc_cached(
            doc_path,
            &params,
            RunMode::Execute,
            executor_choice,
            hick_literate::cache_mode(args.cache, args.freeze),
        )
        .await?;
        let outputs = write_outputs_detailed(&run, args.out.as_deref())?;
        if args.json {
            json_blocks.push(block_model_json(&run)?);
        } else {
            print_run_summary(&run, &outputs);
        }
    }
    if args.json {
        emit_json(json_blocks)?;
    }
    Ok(ExitCode::SUCCESS)
}

async fn cmd_test(args: TestArgs) -> Result<ExitCode> {
    let executor_choice = ExecutorChoice::from_env()?;
    let mut docs = Vec::new();
    for path in &args.paths {
        docs.extend(expand_docs(path)?);
    }
    let params = params_with_features(&args.params, &args.features);
    let mut worst = CheckOutcome::Verified;
    let mut json_blocks = Vec::new();
    for doc_path in &docs {
        // Verify, not Execute: a cell with no baseline must be REPORTED as
        // unverifiable, not abort the run at the first one.
        let run = run_doc_cached(
            doc_path,
            &params,
            RunMode::Verify,
            executor_choice,
            // No `--cache` half to pass: `test` reads recordings but must
            // never write one, or it would manufacture the baseline it then
            // reports as verified. `RunMode::Verify` enforces that in the
            // pipeline whatever mode is selected here.
            if args.freeze {
                CacheMode::Require
            } else {
                CacheMode::Off
            },
        )
        .await?;
        let mut failures = check_failures(&run, args.out.as_deref())?;
        // Transform passages are checked from the SOURCE, not from a re-run:
        // no model is called, so this stays free and deterministic in CI.
        failures.extend(hickory_cli::stale_transforms(&run.doc_path, &run.source)?);
        if args.json {
            json_blocks.push(block_model_json(&run)?);
        }
        if failures.is_empty() {
            eprintln!("ok: {}", doc_path.display());
            continue;
        }
        worst = worst.max(check_outcome(&failures));
        for failure in &failures {
            match failure {
                CheckFailure::Expectation(o) => {
                    let span = o
                        .span
                        .map(|(s, e)| format!(" (bytes {s}..{e})"))
                        .unwrap_or_default();
                    eprintln!(
                        "FAIL {}:{}{span} [container '{}', match {}]\n  {}\n  expected:\n{}\n  actual:\n{}",
                        o.doc,
                        o.line,
                        o.container.as_deref().unwrap_or("agent cell"),
                        o.mode,
                        o.detail,
                        indent(&o.expected),
                        indent(&o.actual),
                    );
                }
                CheckFailure::Drift {
                    doc,
                    output_path,
                    detail,
                } => {
                    eprintln!(
                        "FAIL {} -> {}: {detail}",
                        doc.display(),
                        output_path.display()
                    );
                }
                CheckFailure::StaleTransform {
                    doc,
                    line,
                    select,
                    instruct,
                } => {
                    eprintln!(
                        "STALE {}:{line}: the passage written from '{select}' no longer \
                         matches its input\n  instruction: {instruct}\n  \
                         fix: hick refresh {}",
                        doc.display(),
                        doc.display(),
                    );
                }
                CheckFailure::Unverifiable { doc, cell, reason } => {
                    eprintln!("{}", unverifiable_message(doc, cell, reason));
                }
            }
        }
    }
    if args.json {
        emit_json(json_blocks)?;
    }
    // The configured-driver check has to live somewhere that runs WITHOUT
    // having been installed, because `hick init` installs the pre-commit
    // hook: a clone that never ran it has neither the driver nor the thing
    // that would report the driver missing, which is precisely the clone the
    // check exists for. It reports and never changes the exit code — the four
    // outcomes are a contract CI scripts branch on, and CI never merges.
    // See docs/specs/freeform/provenance-across-versions.md.
    if let Some(first) = docs.first() {
        let dir = first.parent().unwrap_or(std::path::Path::new("."));
        let status = hickory_cli::merge_driver::status(dir);
        if status.repository && !status.ok() {
            eprintln!("hick test: merge driver — {}", status.summary);
        }
    }
    match worst {
        CheckOutcome::Verified => {}
        CheckOutcome::Drifted => eprintln!(
            "hick test: DRIFTED (exit 1) — committed output is out of date with what \
             the document produces. Re-run `hick run <doc>` (or `hick refresh <doc>` \
             for a stale hick:transform) and commit the result."
        ),
        CheckOutcome::Unverifiable => eprintln!(
            "hick test: NOT VERIFIED (exit 2) — at least one cell has no baseline, so this \
             document was not actually checked against anything"
        ),
        CheckOutcome::ExpectationFailed => eprintln!(
            "hick test: EXPECTATION FAILED (exit 3) — a hick:expect did not hold, so the \
             document claims something untrue of its own output. This is not drift and must \
             not be regenerated away: decide whether the claim or the code is wrong."
        ),
    }
    Ok(ExitCode::from(worst.exit_code()))
}

/// `hick up [path] [--run]` — weave a folder and keep it woven until
/// interrupted.
fn cmd_diagram(args: DiagramArgs) -> Result<ExitCode> {
    let root = match &args.path {
        Some(p) => p.clone(),
        None => std::env::current_dir()?,
    };
    let group = match args.group.as_str() {
        "dir" => hickory_cli::diagram::Grouping::Dir,
        "file" => hickory_cli::diagram::Grouping::File,
        other => {
            eprintln!(
                "`--group {other}` is not a granularity. Use `dir` (one node per \
                 top-level directory, the default) or `file` (one node per source file)."
            );
            return Ok(ExitCode::FAILURE);
        }
    };
    let mermaid = match args.format.as_str() {
        "scene" => false,
        "mermaid" => true,
        other => {
            eprintln!(
                "`--format {other}` is not an output form. Use `scene` (JSON for a \
                 `renderer=\"graph\"` diagram or a copy fragment, the default) or `mermaid`."
            );
            return Ok(ExitCode::FAILURE);
        }
    };
    let topology = hickory_cli::diagram::generate_topology(&root, group)?;
    if topology.nodes.is_empty() {
        eprintln!(
            "no source files under {} that structure understands — nothing to draw. \
             (Gitignored files are skipped, and so are files over 512 KB.)",
            root.display()
        );
        return Ok(ExitCode::FAILURE);
    }
    if let Some(doc_path) = &args.refresh {
        let document = std::fs::read_to_string(doc_path)?;
        match hickory_cli::diagram::refresh_fragment(&document, &args.fragment, &topology) {
            Ok(next) => {
                if next != document {
                    std::fs::write(doc_path, next)?;
                }
                println!(
                    "refreshed `#{}` in {} ({} nodes, {} edges)",
                    args.fragment,
                    doc_path.display(),
                    topology.nodes.len(),
                    topology.edges.len()
                );
                Ok(ExitCode::SUCCESS)
            }
            Err(message) => {
                eprintln!("{message}");
                Ok(ExitCode::FAILURE)
            }
        }
    } else {
        print!("{}", hickory_cli::diagram::render(&topology, mermaid));
        Ok(ExitCode::SUCCESS)
    }
}

async fn cmd_search(args: SearchArgs) -> Result<ExitCode> {
    let root = match &args.root {
        Some(r) => r.clone(),
        None => std::env::current_dir()?,
    };
    if args.install_model {
        hickory_cli::search_install::install_model(&root).await?;
        if args.query.is_none() && args.related.is_none() {
            return Ok(ExitCode::SUCCESS);
        }
    }

    // Indexing and embedding are CPU work; keep them off the async runtime.
    let related = args.related.clone();
    let query = args.query.clone();
    let top_k = args.top_k;
    let (semantic, hits) = tokio::task::spawn_blocking(move || -> Result<_> {
        let engine = hick_search::SearchEngine::open(&root)?;
        let hits = match (&related, &query) {
            (Some(spec), _) => {
                let (file, line) = hick_search::parse_file_line(spec)?;
                engine.related(&file, line, top_k)?
            }
            (None, Some(q)) => engine.search(q, top_k),
            (None, None) => anyhow::bail!(
                "nothing to search for.\n  Pass a query (hick search \"where are outputs \
                 written\"), or --related FILE:LINE for similar code."
            ),
        };
        Ok((engine.semantic(), hits))
    })
    .await??;

    if !semantic {
        eprintln!(
            "(lexical ranking only — `hick search --install-model` adds semantic \
             ranking, a one-time ~30 MB download)"
        );
    }
    if hits.is_empty() {
        println!("no matches");
        return Ok(ExitCode::SUCCESS);
    }
    for hit in hits {
        println!("{}:{}-{}", hit.path, hit.start_line, hit.end_line);
        for line in hit.snippet.lines().filter(|l| !l.trim().is_empty()).take(3) {
            println!("    {line}");
        }
        println!();
    }
    Ok(ExitCode::SUCCESS)
}

async fn cmd_up(args: UpArgs) -> Result<ExitCode> {
    hickory_cli::up::run(hickory_cli::up::UpConfig {
        root: args.path.unwrap_or_else(|| PathBuf::from(".")),
        params: params_with_features(&args.params, &args.features),
        run: args.run,
        executor: ExecutorChoice::from_env()?,
    })
    .await?;
    Ok(ExitCode::SUCCESS)
}

async fn cmd_weave(args: WeaveArgs) -> Result<ExitCode> {
    let docs = expand_docs(&args.path)?;
    let params = params_with_features(&args.params, &args.features);
    let mut json_blocks = Vec::new();
    for doc_path in &docs {
        let run = run_doc(doc_path, &params, RunMode::Weave, ExecutorChoice::Local).await?;
        let outputs = write_outputs_detailed(&run, args.out.as_deref())?;
        if args.json {
            json_blocks.push(block_model_json(&run)?);
        } else {
            print_run_summary(&run, &outputs);
            if !run.result.never_run.is_empty() {
                eprintln!(
                    "  {} block(s) never run (no cached transcript)",
                    run.result.never_run.len()
                );
            }
        }
    }
    if args.json {
        emit_json(json_blocks)?;
    }
    Ok(ExitCode::SUCCESS)
}

/// `hick lineage <doc> --output <path> [--json]` — the same Provenance[]
/// the server serves from GET /api/docs/:id/outputs/file, computed locally
/// from a weave (no execution).
fn cmd_cites(args: ContextArgs) -> Result<ExitCode> {
    let source = std::fs::read_to_string(&args.doc)
        .with_context(|| format!("reading {}", args.doc.display()))?;
    let cites = hickory_cli::declared_cites(&args.doc, &source)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&cites)?);
        return Ok(ExitCode::SUCCESS);
    }
    if cites.is_empty() {
        println!(
            "{}: nothing declares a citation (no cites= attribute)",
            args.doc.display()
        );
        return Ok(ExitCode::SUCCESS);
    }
    println!(
        "{}: {} declared citation(s) — assertions by the author, not derived",
        args.doc.display(),
        cites.len()
    );
    for c in &cites {
        println!(
            "\n{} at lines {}-{} cites {:?}:",
            c.from.element, c.from.first_line, c.from.last_line, c.select
        );
        if c.to.is_empty() {
            println!("    (nothing matches — a dangling citation)");
        }
        for t in &c.to {
            println!(
                "    {} lines {}-{}  {}{}",
                t.path,
                t.first_line,
                t.last_line,
                t.element,
                t.id.as_deref()
                    .map(|i| format!(" #{i}"))
                    .unwrap_or_default()
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_context(args: ContextArgs) -> Result<ExitCode> {
    let source = std::fs::read_to_string(&args.doc)
        .with_context(|| format!("reading {}", args.doc.display()))?;
    let writes = hickory_agent::context::context_for_document(&args.doc, &source);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&writes)?);
        return Ok(ExitCode::SUCCESS);
    }
    if writes.is_empty() {
        println!(
            "{}: no agent writes recorded in any sessions/ directory beside it or above it",
            args.doc.display()
        );
        return Ok(ExitCode::SUCCESS);
    }
    println!(
        "{}: {} agent write(s) on record",
        args.doc.display(),
        writes.len()
    );
    for w in &writes {
        let now = match w.current_lines {
            Some((a, b)) if a == b => format!("now line {a}"),
            Some((a, b)) => format!("now lines {a}-{b}"),
            None => "no longer present as written".to_string(),
        };
        println!(
            "\nlines {}-{} as written ({now}) — {}:{}",
            w.write.first_line, w.write.last_line, w.write.session, w.write.session_line
        );
        println!("  in context when written:");
        for input in &w.write.inputs {
            match input {
                hickory_agent::context::ContextInput::File {
                    path,
                    commit,
                    sha256,
                    first_line,
                    last_line,
                    ..
                } => println!(
                    "    file  {path} lines {first_line}-{last_line}  sha256 {}{}",
                    &sha256[..sha256.len().min(12)],
                    commit
                        .as_deref()
                        .map(|c| format!("  commit {}", &c[..c.len().min(12)]))
                        .unwrap_or_default()
                ),
                hickory_agent::context::ContextInput::Conversation {
                    element,
                    id,
                    source,
                    summary,
                    lines,
                    session_line,
                    ..
                } => println!(
                    "    {element:<12} {}{}  ({lines} line(s), session line {session_line}): {summary}",
                    id.as_deref().unwrap_or("-"),
                    source
                        .as_deref()
                        .map(|s| format!(" {s}"))
                        .unwrap_or_default()
                ),
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// The document's repository root and its repository-relative path, or a
/// refusal that says which of the two is missing.
fn doc_in_repo(doc: &std::path::Path) -> Result<(PathBuf, String)> {
    let abs = std::fs::canonicalize(doc)
        .with_context(|| format!("could not resolve {}", doc.display()))?;
    let dir = abs.parent().unwrap_or(std::path::Path::new("."));
    let root = hickory_cli::replay::git_root(dir).ok_or_else(|| {
        anyhow::anyhow!(
            "{} is not inside a git repository, and replay reads old documents \
             out of git.\n  \
             Next step: run `git init` in the folder, or drop `--at` to see the \
             lineage of the document as it stands.",
            doc.display()
        )
    })?;
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    let rel = abs
        .strip_prefix(&root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|_| {
            anyhow::anyhow!(
                "{} is not under the repository at {}",
                doc.display(),
                root.display()
            )
        })?;
    Ok((root, rel))
}

/// `hick lineage --history` — the commits `--at` accepts.
fn cmd_lineage_history(args: &LineageArgs) -> Result<ExitCode> {
    let (root, rel) = doc_in_repo(&args.doc)?;
    let commits = hickory_cli::replay::history(&root, &rel, 200);
    if commits.is_empty() {
        println!(
            "no commits touch {rel} — which is entirely normal for a document \
             that has not been committed yet. Replay has nowhere to go until \
             there is a commit to go to."
        );
        return Ok(ExitCode::SUCCESS);
    }
    for c in &commits {
        println!("{}  {}  {}", c.short, c.author, c.subject);
    }
    Ok(ExitCode::SUCCESS)
}

/// Weave the document as it stood at `commit`, or report the boundary.
async fn replay_run(args: &LineageArgs, commit: &str) -> Result<Option<DocRun>> {
    let (root, rel) = doc_in_repo(&args.doc)?;
    // The name the document had THEN, which a rename makes different from
    // the name it has now.
    let rel_then = hickory_cli::replay::history(&root, &rel, 500)
        .into_iter()
        .find(|c| c.sha.starts_with(commit) || c.short.starts_with(commit))
        .map(|c| c.path)
        .unwrap_or(rel.clone());
    match hickory_cli::replay::replay_at(&root, &rel_then, commit).await? {
        hickory_cli::replay::Replay::Woven(run) => Ok(Some(*run)),
        hickory_cli::replay::Replay::GrammarBoundary { commit, detail } => {
            eprintln!(
                "{}",
                hickory_cli::replay::grammar_boundary_message(&commit, &rel_then, &detail)
            );
            Ok(None)
        }
    }
}

async fn cmd_lineage(args: LineageArgs) -> Result<ExitCode> {
    if args.history {
        return cmd_lineage_history(&args);
    }
    let run = match &args.at {
        Some(commit) => match replay_run(&args, commit).await? {
            Some(run) => run,
            None => return Ok(ExitCode::from(1)),
        },
        None => {
            run_doc(
                &args.doc,
                &args.params,
                RunMode::Weave,
                ExecutorChoice::Local,
            )
            .await?
        }
    };
    let provenance = hickory_cli::output_lineage(&run, &args.output)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&provenance)?);
    } else {
        eprintln!(
            "{} -> {}: {} provenance span(s)",
            args.doc.display(),
            args.output,
            provenance.len()
        );
        for p in &provenance {
            match &p.origin {
                hickory_lineage::Origin::Synthetic => {
                    println!("{:>8}..{:<8} synthetic", p.start, p.end);
                }
                // Agent bytes are reported by session and turn, with
                // authorship derived from git — printed after the table so a
                // long line never breaks the columns.
                hickory_lineage::Origin::Agent { session, turn, .. } => {
                    println!(
                        "{:>8}..{:<8} {:<12} session {session} turn {turn}",
                        p.start, p.end, "agent"
                    );
                }
                // Ingested bytes name the run they arrived from, not just
                // where they sit: "this arrived on that day from that run" is
                // the difference between an honest blame and one that says
                // you wrote a scaffolder's forty files.
                hickory_lineage::Origin::Ingested {
                    doc_path,
                    span,
                    run,
                } => {
                    let short: String = run.chars().take(12).collect();
                    println!(
                        "{:>8}..{:<8} {:<12} {doc_path} bytes {}..{} · run {short}",
                        p.start, p.end, "ingested", span.0, span.1
                    );
                }
                origin => {
                    let (doc_path, s, e) = origin.location().unwrap_or(("?", 0, 0));
                    let kind = match origin {
                        hickory_lineage::Origin::Literal { .. } => "literal",
                        hickory_lineage::Origin::Paste { .. } => "paste",
                        hickory_lineage::Origin::Exec { .. } => "exec",
                        hickory_lineage::Origin::Variable { .. } => "variable",
                        hickory_lineage::Origin::Substitution { .. } => "substitution",
                        hickory_lineage::Origin::Agent { .. }
                        | hickory_lineage::Origin::Ingested { .. }
                        | hickory_lineage::Origin::Synthetic => unreachable!(),
                    };
                    println!(
                        "{:>8}..{:<8} {kind:<12} {doc_path} bytes {s}..{e}",
                        p.start, p.end
                    );
                }
            }
        }
        // Sessions are per-author and may be private; an unresolvable one is
        // reported, not an error.
        let agent = hickory_cli::agent_lineage_report(&run, &args.output)?;
        if !agent.is_empty() {
            println!();
            for entry in &agent {
                println!("{entry}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Where the broker and the seal keep their files: the per-user state
/// directory, like every other thing that belongs to a machine and not to a
/// folder it has open.
fn broker_dir() -> Result<PathBuf> {
    let dir = hickory_workspace::state_root()?.join("broker");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// `hick broker …` — the toll booth.
async fn cmd_broker(cmd: BrokerCommand) -> Result<ExitCode> {
    use hickory_broker::{BrokerLog, Policy, Verb};

    let dir = broker_dir()?;
    let policy_path = dir.join("policy.json");
    let log_path = dir.join("broker.jsonl");

    match cmd {
        BrokerCommand::Serve { bind } => {
            let policy = Policy::load(&policy_path)?;
            let listener = tokio::net::TcpListener::bind(&bind)
                .await
                .with_context(|| format!("could not bind {bind}"))?;
            let addr = listener.local_addr()?;
            eprintln!("broker listening on {addr}");
            eprintln!("  policy: {}", policy_path.display());
            eprintln!("  log:    {}", log_path.display());
            eprintln!(
                "  {} host(s) listed; anything else is denied.",
                policy.hosts.len()
            );
            eprintln!(
                "  This is one road out with a toll booth on it. It is NOT an \
                 airgap: a machine that talks to a model is on a network."
            );
            eprintln!(
                "  `allow` and `deny` only in this build — the broker stays \
                 OUTSIDE every connection, so it sees a hostname and a byte \
                 count and never a body."
            );
            let broker = std::sync::Arc::new(hickory_broker::Broker {
                policy,
                log: std::sync::Arc::new(BrokerLog::at(&log_path)),
                now: std::sync::Arc::new(hickory_cli::serve::now_rfc3339_public),
            });
            broker.serve(listener).await?;
        }
        BrokerCommand::Status => {
            let policy = Policy::load(&policy_path)?;
            println!("policy: {}", policy_path.display());
            println!("log:    {}", log_path.display());
            println!("default: {}", policy.default.as_str());
            if policy.hosts.is_empty() {
                println!(
                    "no hosts listed, so everything is denied. `hick broker allow \
                     api.anthropic.com` is usually the first line."
                );
            }
            for (host, verb) in &policy.hosts {
                println!("  {:<40} {}", host, verb.as_str());
                if verb.reads_the_body() {
                    // Never discovered, always stated.
                    println!(
                        "    the broker is INSIDE this connection and can read \
                         every byte of it, in both directions"
                    );
                }
            }
        }
        BrokerCommand::Allow { host } => {
            let mut policy = Policy::load(&policy_path)?;
            policy.set(&host, Verb::Allow, false)?;
            policy.save(&policy_path)?;
            println!("allow {host}");
        }
        BrokerCommand::Deny { host } => {
            let mut policy = Policy::load(&policy_path)?;
            policy.set(&host, Verb::Deny, false)?;
            policy.save(&policy_path)?;
            println!("deny {host}");
        }
        BrokerCommand::Log { limit } => {
            let entries = BrokerLog::at(&log_path).read();
            let start = entries.len().saturating_sub(limit);
            for entry in &entries[start..] {
                println!(
                    "{}  {:<9} {}:{}{}",
                    entry.at,
                    entry.verb.as_str(),
                    entry.host,
                    entry.port,
                    entry
                        .bytes
                        .map(|b| format!("  {b} bytes"))
                        .unwrap_or_default()
                );
            }
            if entries.is_empty() {
                println!("nothing yet — the broker has forwarded and refused nothing.");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// `hick sealed` — what this machine asserts about itself, and whether it holds.
fn cmd_sealed(args: SealedArgs) -> Result<ExitCode> {
    use hickory_broker::seal::{SealConfig, check_seal};

    let path = broker_dir()?.join("sealed.json");
    let mut config = SealConfig::load(&path);

    if args.unset {
        config.sealed = false;
        config.save(&path)?;
        println!("unsealed. This machine may hold provider keys again.");
        return Ok(ExitCode::SUCCESS);
    }
    if args.set || args.broker.is_some() {
        config.sealed = true;
        if let Some(broker) = args.broker {
            config.broker = Some(broker);
        }
        config.save(&path)?;
        println!(
            "sealed. This machine holds no credential worth stealing and has one \
             road out{}.",
            config
                .broker
                .as_deref()
                .map(|b| format!(", through {b}"))
                .unwrap_or_default()
        );
        println!(
            "  It is NOT airgapped: a machine that talks to a model is on a \
             network, and calling it airgapped would be a nearly-true sentence."
        );
        if !args.check {
            return Ok(ExitCode::SUCCESS);
        }
    }

    let env: std::collections::BTreeMap<String, String> = std::env::vars().collect();
    // A key store on this machine is a thing to find, not a thing to assume.
    let key_file = hickory_workspace::state_root()
        .map(|d| d.join("keys.json"))
        .ok()
        .filter(|p| p.exists())
        .is_some();
    let check = check_seal(&config, &env, key_file, default_route_denied());

    for finding in &check.findings {
        println!(
            "{} {}",
            if finding.ok { "ok  " } else { "FAIL" },
            finding.check
        );
        for line in finding.detail.lines() {
            println!("     {line}");
        }
    }
    if check.ok() {
        println!();
        println!("The seal holds: one road out, with a toll booth on it.");
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Whether the default route is denied at the OS.
///
/// `None` where we cannot tell — reported, never assumed. This VERIFIES and
/// does not perform: reconfiguring somebody's networking for them is not this
/// command's business.
fn default_route_denied() -> Option<bool> {
    match std::env::var("HICKORY_SEALED_ROUTE_DENIED").as_deref() {
        Ok("1") | Ok("true") => Some(true),
        Ok("0") | Ok("false") => Some(false),
        _ => None,
    }
}

/// `hick fleet …` — identity for one engineer's several machines.
async fn cmd_fleet(cmd: FleetCommand) -> Result<ExitCode> {
    use hickory_fleet::{Fleet, Grant, Identity, Invitation, Kind};

    let base = hickory_workspace::state_root()?;
    let name = hickory_fleet::default_machine_name();
    let identity = Identity::load_or_create(&base, &name)?;
    let fleet = Fleet::at(identity.dir());

    match cmd {
        FleetCommand::Whoami => {
            println!("{}  {}", identity.name, identity.fingerprint());
            println!("  key    {}", identity.public_key());
            println!("  keys   {}", identity.dir().display());
            println!(
                "  The private half never leaves this machine and is never in a \
                 repository. Name it something else with HICKORY_MACHINE_NAME."
            );
        }
        FleetCommand::Invite { phone } => {
            let kind = if phone { Kind::Phone } else { Kind::Desktop };
            println!("{}", identity.invitation(kind).encode());
            eprintln!(
                "Paste that on the machine you are pairing with, as \
                 `hick fleet accept <line>`.\n\
                 Then do the same in the other direction: a fleet is a MUTUAL \
                 list of keys, so each machine has to accept the other. It \
                 carries this machine's public key only.\n\
                 Fingerprint to check against the other end: {}",
                identity.fingerprint()
            );
        }
        FleetCommand::Accept { invitation } => {
            let invitation = Invitation::decode(&invitation)?;
            let today = hickory_cli::ingest::today().unwrap_or_else(|| "unknown".to_string());
            let machine = fleet.add(&invitation, &today)?;
            println!(
                "added \"{}\" ({}), grants: {}",
                machine.name,
                machine.fingerprint(),
                machine
                    .grants
                    .iter()
                    .map(|g| g.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "  `execute` is off: it means code runs on this machine as you, \
                 and \"my laptop was stolen\" should not read as \"every machine \
                 I own now executes whatever the thief types\". Give it \
                 deliberately with `hick fleet grant {} execute`.",
                machine.name
            );
            println!(
                "  Nothing is reachable yet — this is identity, not \
                 connectivity."
            );
        }
        FleetCommand::List => {
            let machines = fleet.machines();
            println!(
                "this machine: {}  {}",
                identity.name,
                identity.fingerprint()
            );
            if machines.is_empty() {
                println!(
                    "no other machines paired. `hick fleet invite` prints a line \
                     to paste on another machine, and `hick fleet accept` takes \
                     the one it prints back."
                );
                return Ok(ExitCode::SUCCESS);
            }
            for machine in machines {
                println!(
                    "{:<20} {:<12} {:<50} {}",
                    machine.name,
                    match machine.kind {
                        Kind::Phone => "phone",
                        Kind::Desktop => "desktop",
                    },
                    machine.fingerprint(),
                    machine
                        .grants
                        .iter()
                        .map(|g| g.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
        }
        FleetCommand::Pair { new, phrase, phone } => {
            return cmd_fleet_pair(&identity, &fleet, new, phrase.as_deref(), phone).await;
        }
        FleetCommand::Serve { session } => {
            return cmd_fleet_serve(&identity, fleet, &session).await;
        }
        FleetCommand::Attach {
            machine,
            addr,
            listen,
            path,
            method,
        } => {
            return cmd_fleet_attach(
                &identity,
                &fleet,
                &machine,
                &addr,
                &method,
                &path,
                listen.as_deref(),
            )
            .await;
        }
        FleetCommand::Grant {
            machine,
            grant,
            revoke,
        } => {
            let grant = Grant::parse(&grant)?;
            let updated = fleet.set_grant(&machine, grant, !revoke)?;
            println!(
                "{}: {}",
                updated.name,
                updated
                    .grants
                    .iter()
                    .map(|g| g.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if grant == Grant::Execute && !revoke {
                println!(
                    "  What `execute` costs is a property of that machine, not of \
                     the grant: a cell under the sandbox or Docker is confined to \
                     its own workdir, and a shell is not."
                );
            }
        }
        FleetCommand::Remove { machine } => {
            if fleet.remove(&machine)? {
                println!(
                    "removed {machine}. Its key is gone from this machine, and \
                     that is the whole of the revocation — no server holds a \
                     session you cannot reach."
                );
            } else {
                println!("no machine called {machine:?} is in this fleet");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// `hick fleet pair` — enrol by a spoken phrase, both ways at once.
async fn cmd_fleet_pair(
    identity: &hickory_fleet::Identity,
    fleet: &hickory_fleet::Fleet,
    new: bool,
    phrase: Option<&str>,
    phone: bool,
) -> Result<ExitCode> {
    use hickory_peer::Phrase;

    let kind = if phone {
        hickory_fleet::Kind::Phone
    } else {
        hickory_fleet::Kind::Desktop
    };
    let reach = hickory_peer::Reach::from_env()?;
    let today = hickory_cli::ingest::today().unwrap_or_else(|| "unknown".to_string());

    let paired = match (new, phrase) {
        (true, _) => {
            let phrase = Phrase::generate();
            println!("{phrase}");
            eprintln!();
            eprintln!("  Type that on the other machine: `hick fleet pair <phrase>`.");
            eprintln!("  It pairs BOTH ways in one go — no second round needed.");
            eprintln!("  Waiting… the phrase works once and expires in 2 minutes.");
            hickory_peer::host(identity, fleet, &phrase, kind, &reach, &today).await?
        }
        (false, Some(raw)) => {
            let phrase = Phrase::parse(raw)?;
            eprintln!("dialling that phrase…");
            hickory_peer::join(identity, fleet, &phrase, kind, &reach, &today).await?
        }
        (false, None) => anyhow::bail!(
            "say which end you are.\n  \
             On the first machine: `hick fleet pair --new` prints a phrase and \
             waits.\n  \
             On the second: `hick fleet pair <phrase>` with what it printed."
        ),
    };

    println!();
    println!("paired with \"{}\"", paired.machine.name);
    println!("  them  {}", paired.machine.fingerprint());
    println!("  you   {}", paired.ours);
    println!(
        "  Compare those two lines on both screens. That is the only check \
         that catches somebody who guessed the phrase — the phrase itself \
         proves nothing about who used it."
    );
    println!(
        "  Grants: {}. `execute` is off until you give it deliberately.",
        paired
            .machine
            .grants
            .iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(ExitCode::SUCCESS)
}

/// `hick fleet serve` — be reachable by your own machines.
async fn cmd_fleet_serve(
    identity: &hickory_fleet::Identity,
    fleet: hickory_fleet::Fleet,
    session: &str,
) -> Result<ExitCode> {
    let local: std::net::SocketAddr = session.parse().with_context(|| {
        format!(
            "{session} is not an address like `127.0.0.1:7317`. It is the \
             loopback address of the session you are exposing — what `hick up` \
             or the app already serves."
        )
    })?;
    let machines = fleet.machines();
    if machines.is_empty() {
        eprintln!(
            "no machines are paired, so nothing could reach this session even \
             if it were listening.\n  \
             Next step: `hick fleet invite` here, `hick fleet accept` there, and \
             the same in the other direction — a fleet is mutual."
        );
        return Ok(ExitCode::SUCCESS);
    }

    let reach = hickory_peer::Reach::from_env()?;
    {
        let server = std::sync::Arc::new(
            hickory_peer::PeerServer::bind(identity, fleet, local, reach.clone()).await?,
        );
        let addr = server.addr();
        eprintln!("this machine: {} {}", identity.name, identity.fingerprint());
        eprintln!("  serving the session at {local}");
        eprintln!("  {}", reach.summary());
        eprintln!();
        eprintln!("  Dial it from another machine with:");
        eprintln!(
            "    hick fleet attach {} --addr '{}'",
            identity.name,
            serde_json::to_string(&addr).unwrap_or_default()
        );
        eprintln!();
        eprintln!(
            "  Durable state does NOT travel over this. Two machines editing one \
             document commit and push, and git reconciles; what crosses here is \
             the live room. Close both without committing and they diverge."
        );
        server.serve().await?;
    }
    Ok(ExitCode::SUCCESS)
}

/// `hick fleet attach` — reach one of your machines and ask it something.
#[allow(clippy::too_many_arguments)]
async fn cmd_fleet_attach(
    identity: &hickory_fleet::Identity,
    fleet: &hickory_fleet::Fleet,
    machine: &str,
    addr: &str,
    method: &str,
    path: &str,
    listen: Option<&str>,
) -> Result<ExitCode> {
    // Its key must be one we hold — the check runs at BOTH ends, because a
    // fleet is mutual and dialling a machine we have not enrolled would mean
    // trusting whatever answered.
    if fleet.get(machine).is_none() {
        anyhow::bail!(
            "no machine called {machine:?} is in this fleet, so there is no key \
             to check what answers against.\n  \
             A fleet is a MUTUAL list: run `hick fleet invite` on that machine \
             and `hick fleet accept` here.",
        );
    }
    let addr: iroh::EndpointAddr = serde_json::from_str(addr).with_context(|| {
        format!(
            "{addr} is not an address `hick fleet serve` printed. Copy the whole \
             quoted value it gave you."
        )
    })?;

    let reach = hickory_peer::Reach::from_env()?;

    // Serving it on a local port is the usable shape: the desktop app is
    // already a client of a server it need not be co-located with, so this is
    // all that stands between "a proven channel" and "the other machine's
    // session in your window".
    if let Some(listen) = listen {
        let bind: std::net::SocketAddr = listen
            .parse()
            .with_context(|| format!("{listen} is not an address like `127.0.0.1:7318`"))?;
        let peer = hickory_peer::Attached::connect(identity, addr, reach).await?;
        report_path(peer.how().await);
        eprintln!("serving {machine}'s session on http://{bind}");
        eprintln!("  Point the app or a browser there. Your grants apply: a");
        eprintln!("  refusal comes back as a 403 saying which grant it needed.");
        eprintln!("  Ctrl-C to stop. Nothing is written to this machine.");
        peer.forward(bind).await?;
        return Ok(ExitCode::SUCCESS);
    }

    let peer = hickory_peer::Attached::connect(identity, addr, reach).await?;
    let answer = peer.request(method, path).await?;
    let how = peer.how().await;
    peer.close().await;

    report_path(how);
    println!("{answer}");
    Ok(ExitCode::SUCCESS)
}

/// Say who is in the path. Every time, not buried in a setting: a third party
/// carrying your traffic is a thing to be told rather than to discover.
fn report_path(how: hickory_peer::Reachability) {
    match how {
        hickory_peer::Reachability::Direct => {
            eprintln!("connected directly — nothing in the middle.")
        }
        hickory_peer::Reachability::Relayed => eprintln!(
            "connected THROUGH A RELAY — no direct path was found, so your \
             encrypted traffic is transiting somebody else's server. Set \
             HICKORY_FLEET_RELAY to your own relay, or `direct` to refuse \
             relays entirely."
        ),
        hickory_peer::Reachability::None => {
            eprintln!("connected, but the path could not be determined.")
        }
    }
}

/// `hick repair` — the pre-commit repair.
///
/// Says what it RECORDED, never what it refused, and exits 0 whatever
/// happens: an unwatched edit leaves its correspondence behind before it is
/// committed, and that is the whole of it.
fn cmd_repair(args: RepairArgs) -> Result<ExitCode> {
    let Some(root) = hickory_cli::replay::git_root(&args.path) else {
        return Ok(ExitCode::SUCCESS);
    };
    let today = hickory_cli::ingest::today().unwrap_or_else(|| "unknown".to_string());
    match hickory_cli::continuity::repair_staged(&root, &today) {
        Ok(0) => {}
        Ok(n) => eprintln!(
            "hick repair: recorded {n} correspondence(s) for edits nothing was \
             watching. Continuity is on for this project."
        ),
        // A repair that could not be written is worth saying and is not worth
        // blocking a commit over: the record is the feature, not a gate.
        Err(e) => eprintln!("hick repair: could not record continuity ({e:#}); the commit stands"),
    }
    Ok(ExitCode::SUCCESS)
}

/// `hick merge-driver` — git's three-way merge for `*.hick`.
///
/// Git's contract: write the result to `%A` either way, exit 0 for a clean
/// merge and non-zero for a conflict.
fn cmd_index(command: IndexCommand) -> Result<ExitCode> {
    use hickory_cli::index_install;

    match command {
        IndexCommand::List => {
            println!("Code indexers answer questions about the whole project — where else is");
            println!("this used, across documents and the files they weave. They are IN ADDITION");
            println!("to language servers, never in place of them: a server knows your unsaved");
            println!("buffer and an index does not, and an index spans the project and a server");
            println!("does not.\n");
            for language in [
                "typescript",
                "javascript",
                "python",
                "rust",
                "csharp",
                "java",
            ] {
                println!("  {language}");
                for line in textwrap_lines(&index_install::how_to_get(language)) {
                    println!("      {line}");
                }
            }
            println!();
            report_catalogue(
                "Of those, the indexers `hick index install` can fetch:",
                index_install::plans(),
                "hick index install",
                index_install::INDEXERS_DIR,
            );
            println!();
            println!("Nothing in hick reads an index yet. `hick index build` produces one, and");
            println!("no navigation feature consults it — see");
            println!("docs/specs/freeform/an-index-beside-the-language-server.md.");
            Ok(ExitCode::SUCCESS)
        }

        IndexCommand::Install { languages, root } => {
            let root = match root {
                Some(root) => root,
                None => std::env::current_dir().context("resolving the current directory")?,
            };
            let languages = chosen_languages(languages, index_install::plans());
            if languages.is_empty() {
                println!(
                    "Nothing to install: none of the installers' tools ({}) are on this \
                     machine.\nInstall one of them, or install an indexer yourself — either way \
                     hick will find it.",
                    installer_tools(index_install::plans()),
                );
                return Ok(ExitCode::SUCCESS);
            }
            for language in &languages {
                println!("Installing the {language} indexer, sandboxed…");
                let prefix = index_install::install(&root, language)?;
                println!("  installed into {}", prefix.display());
            }
            Ok(ExitCode::SUCCESS)
        }

        IndexCommand::Build { language, root } => {
            let root = match root {
                Some(root) => root,
                None => std::env::current_dir().context("resolving the current directory")?,
            };
            let Some(recipe) = index_install::recipe(&language) else {
                bail!(
                    "hick does not know how to run an indexer for {language}.\n{}",
                    index_install::how_to_get(&language)
                );
            };
            let Some(binary) = index_install::discover(&language, &root) else {
                bail!(
                    "no {language} indexer on this machine.\n{}",
                    index_install::how_to_get(&language)
                );
            };
            // Into `.hick-cache/`, which `hick init` already ignores: an index
            // is a build artifact of this machine and must never land in the
            // repository.
            let out_dir = root.join(".hick-cache/index");
            std::fs::create_dir_all(&out_dir)
                .with_context(|| format!("creating {}", out_dir.display()))?;
            let out = out_dir.join(format!("{language}.scip"));

            println!("Indexing {language} with {}…", binary.display());
            let status = std::process::Command::new(&binary)
                .args(recipe.args)
                .arg(recipe.output_flag)
                .arg(&out)
                .current_dir(&root)
                .status()
                .with_context(|| format!("running {}", binary.display()))?;
            if !status.success() {
                bail!(
                    "the {language} indexer failed ({status}). Its own output says why — hick \
                     ran it and did not interpret it."
                );
            }
            let size = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
            println!("  wrote {} ({size} bytes)", out.display());
            // Said every time, because a person who does not know this will
            // reasonably assume navigation just got better.
            println!();
            println!("Nothing in hick reads this yet. Reading it means linking the `scip` crate,");
            println!("which is Apache-2.0 — permissive, so it passes the rule in AGENTS.md and");
            println!("contradicts its \"MIT only\" heading. That is a decision to make");
            println!("deliberately, and it has not been made.");
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_history(args: HistoryArgs) -> Result<ExitCode> {
    use hickory_workspace::history::ActKind;

    let root = std::env::current_dir().context("resolving the current directory")?;
    let Some(history) = hickory_cli::history::open(&root) else {
        // Not an error: a machine with no data directory still runs
        // everything else, and saying so beats a stack trace.
        println!("No local history for this folder — there is nowhere on this machine to keep it.");
        return Ok(ExitCode::SUCCESS);
    };

    let (command, path, limit) = (args.command, args.path, args.limit);
    match command {
        // No subcommand is the list, because "what happened to this folder"
        // is the question people arrive with.
        None => {
            let acts = match &path {
                Some(path) => history.acts_for(path),
                None => history.acts(),
            };
            if acts.is_empty() {
                println!(
                    "Nothing recorded yet{}.\n\
                     Local history starts when something writes: a run, a weave, a \
                     find-and-replace, an ingest, the agent, the merge driver.",
                    path.map(|p| format!(" for {p}")).unwrap_or_default()
                );
                return Ok(ExitCode::SUCCESS);
            }
            for act in acts.iter().take(limit) {
                let files = act.changed().count();
                println!(
                    "{}  {}  {:<12} {:>3} file(s){}",
                    act.id,
                    act.at,
                    act.kind.as_str(),
                    files,
                    act.detail
                        .as_deref()
                        .map(|d| format!("  {d}"))
                        .unwrap_or_default()
                );
            }
            if acts.len() > limit {
                println!("… and {} more", acts.len() - limit);
            }
            println!(
                "\nStored under {} — local to you and this machine. Not a backup: same disk, \
                 same account, evicted on a timer.",
                history.dir().display()
            );
            Ok(ExitCode::SUCCESS)
        }

        Some(HistoryCommand::Show { act }) => {
            let Some(found) = history.act(&act) else {
                bail!(
                    "no act {act} in this folder's local history. `hick history` lists them; an \
                     id prefix works as long as it names only one."
                );
            };
            println!("{}  {}  {}", found.id, found.at, found.kind.as_str());
            if let Some(detail) = &found.detail {
                println!("  {detail}");
            }
            for file in found.changed() {
                let what = match (&file.before, &file.after) {
                    (None, Some(_)) => "created",
                    (Some(_), None) => "deleted",
                    _ => "changed",
                };
                println!("  {what:<8} {}", file.path);
            }
            if !found.kind.is_revertable() {
                println!(
                    "\nThis act wrote generated files, so `revert` is refused: the next run \
                     would undo it. Change what generates them instead."
                );
            }
            Ok(ExitCode::SUCCESS)
        }

        Some(HistoryCommand::Revert { act, path }) => {
            let Some(found) = history.act(&act) else {
                bail!("no act {act} in this folder's local history. `hick history` lists them.");
            };
            let out = history.revert(&root, &found, path.as_deref())?;
            for path in &out.restored {
                println!("  put back {path}");
            }
            // Never silently skipped: somebody undoing a batch is already
            // unsure what happened, and a quiet partial revert is how they
            // come to trust a state that never existed.
            for path in &out.moved_on {
                println!(
                    "  left    {path} — it is not what {} wrote any more, so putting the old \
                     bytes back would be a second unasked-for write on top of the first",
                    found.id
                );
            }
            for (path, why) in &out.refused {
                println!("  refused {path} — {why}");
            }
            if out.restored.is_empty() {
                println!("Nothing was put back.");
                return Ok(ExitCode::SUCCESS);
            }
            // Going back is itself an act, so the way back from a bad revert
            // is the same list. A history you can fall out of is a history
            // nobody trusts.
            let writes: Vec<(std::path::PathBuf, Vec<u8>)> = out
                .restored
                .iter()
                .filter_map(|p| {
                    let full = root.join(p);
                    std::fs::read(&full).ok().map(|bytes| (full, bytes))
                })
                .collect();
            hickory_cli::history::record(
                &root,
                ActKind::Revert,
                Some(format!("reverted {}", found.id)),
                &writes,
            );
            Ok(ExitCode::SUCCESS)
        }

        Some(HistoryCommand::Forget { path, all }) => {
            if path.is_none() && !all {
                bail!(
                    "say what to forget: `--path <file>` for one file's versions, or `--all` \
                     for this folder's whole local history. Neither can be undone."
                );
            }
            let removed = history.forget(path.as_deref())?;
            match path {
                Some(path) => println!("Forgot {removed} act(s) about {path}."),
                None => println!("Forgot {removed} act(s), and every version they held."),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_merge_driver(args: MergeDriverArgs) -> Result<ExitCode> {
    match hickory_cli::merge_driver::run(
        &args.base,
        &args.ours,
        &args.theirs,
        args.marker_size,
        &args.path,
    )? {
        hickory_cli::merge_driver::MergeOutcome::Clean => Ok(ExitCode::SUCCESS),
        hickory_cli::merge_driver::MergeOutcome::Conflicted { reason } => {
            eprintln!("hick merge: {reason}");
            Ok(ExitCode::from(1))
        }
    }
}

/// `hick open [path]` — hand a folder to the desktop app and return.
fn cmd_open(args: OpenArgs) -> Result<ExitCode> {
    if !args.path.exists() {
        anyhow::bail!(
            "{} does not exist, so there is nothing to open.\n  \
             `hick open` takes a folder of documents or a single `.hick` file, \
             and defaults to the working directory.",
            args.path.display()
        );
    }
    let Some(app) =
        hickory_cli::open_app::find(|name| std::env::var(name).ok(), |path| path.exists())
    else {
        anyhow::bail!("{}", hickory_cli::open_app::not_installed());
    };
    hickory_cli::open_app::open(&app, &args.path)?;
    eprintln!("opening {} in Hickory Docs", args.path.display());
    Ok(ExitCode::SUCCESS)
}

/// `hick emit` — what a re-emission would produce. Emits nothing.
async fn cmd_emit(args: EmitArgs) -> Result<ExitCode> {
    let plan = hickory_cli::emission::plan_for(&args.path).await?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(if plan.allowed() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        });
    }

    println!("{}", plan.summary);
    if let Some(floor) = &plan.floor {
        println!("  floor: {}", floor.summary);
    }
    for commit in &plan.commits {
        println!();
        println!("  {}", commit.subject);
        println!("    from {}", commit.document);
        match &commit.replaces {
            // Re-emission REBUILDS the frontier rather than patching it: this
            // is `jj squash --into` with the destination computed.
            Some(sha) => println!("    would replace draft {}", &sha[..sha.len().min(12)]),
            None => println!("    would be a new commit"),
        }
        for file in &commit.files {
            println!("    {file}");
        }
        if commit.files.is_empty() {
            println!("    (this document does not weave right now, so its files are unknown)");
        }
    }

    if !plan.published.is_empty() {
        println!();
        for sha in &plan.published {
            println!("  REFUSED: {} is published", &sha[..sha.len().min(12)]);
        }
        println!(
            "  Publication is what makes emission one-way. Below the floor a \
             commit is a record — someone else may be holding it — so emission \
             appends and never amends there.\n  \
             Next step: put the work on a branch above the floor, and re-emit \
             that."
        );
        return Ok(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

/// `hick carry <session>` — what moves to the next attempt.
fn cmd_carry(args: CarryArgs) -> Result<ExitCode> {
    let today = hickory_cli::ingest::today().unwrap_or_else(|| "unknown".to_string());
    let outcome =
        hickory_cli::carry::carry_from_session(&args.session, args.out.as_deref(), &today)?;
    let body = std::fs::read_to_string(&outcome.path).unwrap_or_default();
    println!("carried {} into {}", outcome.from, outcome.path.display());
    let slots = hickory_cli::carry::unfilled_slots(&body);
    if !slots.is_empty() {
        println!(
            "  Still empty, and yours to fill: {}.\n  \
             The tests worth carrying are the ones you READ and kept — an agent \
             that had already seen one implementation writes tests that pin its \
             incidental choices, and carrying all of them binds the next attempt \
             to the first one's arbitrary decisions while looking like a free \
             choice.",
            slots.join(", ")
        );
    }
    println!(
        "  The first session is a draft you may discard — `sessions/` is \
         gitignored, so that is already the default. This is not, and must not \
         be: a carried requirement whose only support is a session nobody has \
         reads as pulled out of the air."
    );
    Ok(ExitCode::SUCCESS)
}

async fn cmd_adopt(args: AdoptArgs) -> Result<ExitCode> {
    let outcome = match &args.into {
        Some(doc) => hickory_cli::adopt::adopt_into(&args.file, doc).await?,
        None => hickory_cli::adopt::adopt_new(&args.file).await?,
    };
    eprintln!(
        "adopted {} into {} ({}) — byte-exact, verified by weave",
        args.file.display(),
        outcome.doc_path.display(),
        if outcome.created {
            "new document"
        } else {
            "appended block"
        },
    );
    Ok(ExitCode::SUCCESS)
}

/// Drain the inbox, or ingest one named file.
///
/// Every outcome is reported, including the skips: a file that quietly stayed
/// in the inbox with no explanation is how someone concludes the feature is
/// broken.
/// `hick import claude-code …`: every outcome printed, including what was
/// left out — an import that says "done" and is quietly missing half the
/// conversation is how someone stops trusting the record.
fn cmd_import(args: ImportArgs) -> Result<ExitCode> {
    use hickory_cli::claude_code::{Outcome, import_file};
    let ImportSource::ClaudeCode {
        files,
        out,
        force,
        stdout,
    } = args.source;
    if stdout {
        if files.len() != 1 {
            eprintln!(
                "--stdout takes exactly one file; {} were given.",
                files.len()
            );
            return Ok(ExitCode::from(2));
        }
        let jsonl = std::fs::read_to_string(&files[0])
            .with_context(|| format!("reading {}", files[0].display()))?;
        let converted = hickory_cli::claude_code::convert(&jsonl)?;
        print!("{}", converted.hick);
        report_import(&files[0], None, &converted.stats);
        return Ok(ExitCode::SUCCESS);
    }
    let mut failed = false;
    for file in &files {
        match import_file(file, &out, force) {
            Ok(Outcome::Written { path, converted }) => {
                report_import(file, Some(&path), &converted.stats);
            }
            Ok(Outcome::Exists { path }) => {
                println!(
                    "{} — already imported as {} (use --force to overwrite)",
                    file.display(),
                    path.display()
                );
            }
            Err(e) => {
                failed = true;
                eprintln!("{} — not imported: {e:#}", file.display());
            }
        }
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn report_import(source: &Path, written: Option<&Path>, stats: &hickory_cli::claude_code::Stats) {
    let where_ = match written {
        Some(p) => format!("→ {}", p.display()),
        None => "→ stdout".to_string(),
    };
    eprintln!(
        "{} {where_}: {} prompt(s), {} reply(ies), {} reasoning, {} tool call(s), {} result(s), {} context",
        source.display(),
        stats.prompts,
        stats.assistant_messages,
        stats.reasoning,
        stats.tool_calls,
        stats.tool_results,
        stats.context,
    );
    for (why, n) in &stats.adapted {
        eprintln!("  adapted: {n} × {why}");
    }
    for (why, n) in &stats.dropped {
        eprintln!("  dropped: {n} × {why}");
    }
    for problem in &stats.problems {
        eprintln!("  PROBLEM: {problem}");
    }
}

/// `hick ingest --from '#cell' doc.hick` — the scaffolder's door.
///
/// See `docs/specs/freeform/owning-what-a-scaffolder-wrote.md`. The report
/// names what was filtered as well as what was written: "38 files" and "38
/// files, 2 skipped" are different facts about the same run, and a user who
/// is not told which one they got cannot tell an ingest from a partial one.
async fn cmd_ingest_from_exec(args: &IngestArgs, selector: &str) -> Result<ExitCode> {
    let doc = &args.path;
    if doc.is_dir() || doc.extension().and_then(|e| e.to_str()) != Some("hick") {
        anyhow::bail!(
            "`--from` ingests a cell's output into a document, so the path has \
             to be a `.hick` file — got {}.\n  \
             Next step: `hick ingest --from '{selector}' path/to/document.hick`.",
            doc.display()
        );
    }
    let today = hickory_cli::ingest::today().unwrap_or_else(|| "unknown".to_string());
    let outcome = hickory_cli::ingest_exec::ingest_from_exec(
        doc,
        selector,
        ExecutorChoice::from_env()?,
        &today,
    )
    .await?;

    println!(
        "ingested {} file(s) from {} into {}",
        outcome.ingested.len(),
        outcome.from,
        outcome.doc_path.display(),
    );
    println!("  run {}", outcome.fingerprint);
    if !outcome.skipped.is_empty() {
        println!(
            "  {} file(s) skipped — this project's .gitignore would ignore them, \
             and a document that owns the source must not carry build output:",
            outcome.skipped.len()
        );
        for path in outcome.skipped.iter().take(10) {
            println!("    {path}");
        }
        if outcome.skipped.len() > 10 {
            println!("    … ({} more)", outcome.skipped.len() - 10);
        }
    }
    println!(
        "  These bytes are the document's now: edit them like any others, and \
         `hick lineage` reports them as ingested rather than as text you wrote."
    );

    // A re-ingest is a three-way merge, so what it DID is the report.
    let m = &outcome.merge;
    if let Some(base) = &outcome.base_commit {
        println!(
            "  Merged against the ingest committed in {} — the bytes as they \
             were ingested are the base, this run is theirs, your edits are ours.",
            &base[..base.len().min(12)]
        );
        let list = |label: &str, paths: &[String]| {
            if paths.is_empty() {
                return;
            }
            println!("  {label}: {}", paths.len());
            for path in paths.iter().take(10) {
                println!("    {path}");
            }
            if paths.len() > 10 {
                println!("    … ({} more)", paths.len() - 10);
            }
        };
        list("changed by this run", &m.merged);
        list("added by this run", &m.added);
        list(
            "no longer produced, and you had not changed them, so removed",
            &m.removed,
        );
        // Named rather than counted: a tool does not get to delete somebody's
        // edit because a scaffolder changed its mind.
        list(
            "no longer produced, but you HAD changed them, so kept",
            &m.kept,
        );
        if m.is_empty_of_change() && m.merged.is_empty() {
            println!("  Nothing moved: this run produced what the last one did.");
        }
        if !m.conflicted.is_empty() {
            println!();
            for path in &m.conflicted {
                println!("  CONFLICT {path}");
            }
            println!(
                "  {} file(s) were changed on both sides in the same place. The \
                 markers are in the document, where the resolution belongs — \
                 resolve them there and they become ordinary document bytes with \
                 ordinary provenance.\n  \
                 Note that a scaffolder randomises things (a user-secrets id, a \
                 GUID, a timestamp), so some of these are noise rather than a \
                 real disagreement.",
                m.conflicted.len()
            );
        }
    }
    if outcome.recorded > 0 {
        println!(
            "  {} correspondence(s) recorded — continuity is on for this project.",
            outcome.recorded
        );
    }
    if outcome.merge.conflicted.is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

async fn cmd_ingest(args: IngestArgs) -> Result<ExitCode> {
    if let Some(selector) = &args.from {
        return cmd_ingest_from_exec(&args, selector).await;
    }
    let config = hickory_cli::ingest::InboxConfig::from_env()?;

    let outcomes = match &args.file {
        Some(file) => {
            let notes_dir = if args.path.as_os_str() == "." {
                file.parent()
                    .and_then(|p| p.parent())
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf()
            } else {
                args.path.clone()
            };
            vec![hickory_cli::ingest::ingest_one(file, &notes_dir, &config)?]
        }
        None => hickory_cli::ingest::ingest_inbox(&args.path, &config)?,
    };

    if outcomes.is_empty() {
        let arriving = hickory_cli::ingest::in_flight_count(&args.path, &config);
        if arriving > 0 {
            println!(
                "nothing to ingest yet: {arriving} file(s) in {} are still \
                 downloading.\n  \
                 A half-written file is a truncated transcript, so they are left \
                 alone until the transfer finishes.\n  \
                 Next step: wait for the download to complete and run this again — \
                 or run `hick up`, which watches the inbox and needs nothing \
                 from you.",
                config.inbox(&args.path).display()
            );
            return Ok(ExitCode::SUCCESS);
        }
        println!(
            "nothing to ingest: {} has no files in it.\n  \
             Drop a transcript export (.vtt, .srt, .sbv from Google Meet, or the \
             markdown a meeting assistant produces) in there and run this again.\n  \
             A Google Docs transcript must be exported first: File > Download > \
             Markdown.",
            config.inbox(&args.path).display()
        );
        return Ok(ExitCode::SUCCESS);
    }

    let mut ingested = 0usize;
    for outcome in &outcomes {
        println!("{outcome}");
        if let hickory_cli::ingest::Outcome::Waiting { .. } = outcome {
            println!(
                "  It will be picked up on the next pass. `hick up` watches the \
                 inbox continuously, so a download finishing there needs nothing \
                 from you."
            );
        }
        if matches!(outcome, hickory_cli::ingest::Outcome::Ingested { .. }) {
            ingested += 1;
        }
    }
    if ingested > 0 {
        println!(
            "\n{ingested} note(s) written. Their summaries are empty and stale \
             until you run `hick refresh` — ingest never calls a model."
        );
    }
    Ok(ExitCode::SUCCESS)
}

async fn cmd_equiv(args: EquivArgs) -> Result<ExitCode> {
    use hickory_cli::{RunMode, run_doc};
    let first = run_doc(&args.first, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .with_context(|| format!("weaving {}", args.first.display()))?;
    let second = run_doc(&args.second, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .with_context(|| format!("weaving {}", args.second.display()))?;
    let diffs = hick_literate::equiv::compare_outputs(&first.result.files, &second.result.files);
    if diffs.is_empty() {
        eprintln!(
            "equivalent: {} and {} weave identical outputs ({} files)",
            args.first.display(),
            args.second.display(),
            first.result.files.len(),
        );
        return Ok(ExitCode::SUCCESS);
    }
    for diff in &diffs {
        eprintln!("{}", hick_literate::equiv::format_diff(diff));
    }
    eprintln!(
        "NOT equivalent: {} output(s) differ between {} and {}",
        diffs.len(),
        args.first.display(),
        args.second.display(),
    );
    Ok(ExitCode::FAILURE)
}

fn cmd_promote(args: PromoteArgs) -> Result<ExitCode> {
    use hick_literate::promote::{PromoteOpts, promote};
    let source = std::fs::read_to_string(&args.session)
        .with_context(|| format!("failed to read {}", args.session.display()))?;
    let project_dir = args
        .session
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let session_name = args
        .session
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| args.session.display().to_string());
    let result = promote(&PromoteOpts {
        session_source: &source,
        project_dir: &project_dir,
        session_name: &session_name,
    })?;
    match &args.out {
        Some(path) => {
            std::fs::write(path, &result.promoted_source)?;
            eprintln!(
                "promoted {} -> {} ({} of {} writes surviving)",
                args.session.display(),
                path.display(),
                result.surviving_writes,
                result.total_writes
            );
        }
        None => print!("{}", result.promoted_source),
    }
    Ok(ExitCode::SUCCESS)
}

async fn cmd_agent(args: AgentArgs) -> Result<ExitCode> {
    use std::io::Write as _;

    use hickory_agent::{
        AgentConfig, AgentEvent, KeyStore, client_for_with_store, resolve_selector_with_store,
        run_agent,
    };

    let project_dir = match &args.dir {
        Some(dir) => dir.clone(),
        None => std::env::current_dir()?,
    };
    // With no --provider, the environment decides: HICKORY_LLM_PROVIDER,
    // else the provider whose key is present. `client_for` then reports an
    // unknown provider or a missing key by name, before anything is executed.
    // A key entered in the desktop app's Settings counts here too: the same
    // file is read, before the environment, so one machine has one answer to
    // "which key".
    let store = KeyStore::desktop();
    let provider = resolve_selector_with_store(args.provider.as_deref(), &store)?;
    let llm = client_for_with_store(&provider, args.model.as_deref(), &store)?;
    // Executor selection follows HICKORY_EXECUTOR, same as run/test.
    let executor = ExecutorChoice::from_env()?.build().await?;

    let mut config = AgentConfig::new(args.prompt, &project_dir);
    // `--doc` is the session's primary document: it enables the edit tool
    // set (read_doc/read_output/edit_output/edit_doc/verify) instead of
    // inlining the source as context.
    config.doc_path = args.doc.clone();
    config.max_turns = args.max_turns;

    let mut on_event = |event: AgentEvent| {
        let mut stdout = std::io::stdout();
        match &event {
            AgentEvent::SessionStarted {
                model,
                session_path,
            } => eprintln!("agent: model {model}, session {session_path}"),
            AgentEvent::Token { data } => {
                let _ = write!(stdout, "{data}");
                let _ = stdout.flush();
            }
            // Reasoning goes to stderr, dimmed by position rather than
            // colour: it is the model thinking, not the answer, and a pipe
            // reading stdout must not receive it as one.
            AgentEvent::Reasoning { data } => {
                eprint!("{data}");
            }
            AgentEvent::ResponseComplete { .. } => {
                let _ = writeln!(stdout);
            }
            AgentEvent::ScriptStarted { lang, .. } => eprintln!("agent: running {lang} script"),
            AgentEvent::ScriptFinished { result } => {
                if !result.stdout.is_empty() {
                    let _ = write!(stdout, "{}", result.stdout);
                }
                if !result.stderr.is_empty() {
                    eprint!("{}", result.stderr);
                }
                eprintln!(
                    "agent: script exited with {}",
                    result
                        .exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".into())
                );
            }
            AgentEvent::ToolStarted { name, .. } => eprintln!("agent: running tool {name}"),
            AgentEvent::ToolFinished { name, ok, text } => {
                let _ = writeln!(stdout, "{text}");
                eprintln!(
                    "agent: tool {name} {}",
                    if *ok { "ok" } else { "refused/failed" }
                );
            }
            AgentEvent::Reprompt { reason, .. } => {
                eprintln!("agent: re-prompting after malformed response ({reason})");
            }
            AgentEvent::Error { message } => eprintln!("agent: error: {message}"),
            AgentEvent::TurnUsage {
                usage,
                cost_usd,
                total_cost_usd,
                ..
            } => {
                let cost =
                    |c: &Option<f64>| c.map(|v| format!("${v:.4}")).unwrap_or_else(|| "?".into());
                eprintln!(
                    "agent: turn usage in={} cache_write={} cache_read={} out={} ({}, session total {})",
                    usage.input_tokens,
                    usage.cache_creation_input_tokens,
                    usage.cache_read_input_tokens,
                    usage.output_tokens,
                    cost(cost_usd),
                    cost(total_cost_usd),
                );
            }
            AgentEvent::UserMessage { .. } | AgentEvent::Thinking | AgentEvent::Done { .. } => {}
        }
    };

    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event).await?;
    executor.shutdown().await?;

    eprintln!(
        "agent: finished in {} turn(s); session written to {}{}",
        outcome.turns,
        outcome.session_path.display(),
        outcome
            .total_cost_usd
            .map(|c| format!("; spend ${c:.4}"))
            .unwrap_or_default()
    );
    println!("{}", outcome.session_path.display());
    Ok(ExitCode::SUCCESS)
}

/// `hick model` — the optional local model, and what it changes.
async fn cmd_model(command: ModelCommand) -> Result<ExitCode> {
    let root = std::env::current_dir().context("resolving the current directory")?;
    match command {
        ModelCommand::List => {
            let where_ = hick_search::model_dir(&root);
            if hick_search::model_available(&root) {
                println!("Local model: installed at {}", where_.display());
                println!("  Completions rank by what this project is about as well as by");
                println!("  how often a name is used; search ranks semantically too.");
            } else {
                println!("Local model: not installed.");
                println!("  Everything works without it — completions rank by frequency");
                println!("  and search ranks lexically. `hick model install` adds the");
                println!("  semantic half; it is the one command here that uses the network.");
            }
            Ok(ExitCode::SUCCESS)
        }
        ModelCommand::Install(args) => {
            let root = args.root.unwrap_or(root);
            hickory_cli::search_install::install_model(&root).await?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// `hick formula` — what can evaluate a table, and putting it in place.
fn cmd_formula(command: FormulaCommand) -> Result<ExitCode> {
    use hick_formula::backend;

    match command {
        FormulaCommand::List => {
            println!("Languages a table's formulas can be written in:\n");
            for entry in backend::BACKENDS {
                match backend::find_interpreter(entry) {
                    Some(interpreter) => {
                        println!("  {:<12} ready — {interpreter}", entry.language)
                    }
                    None => println!(
                        "  {:<12} needs one of: {}",
                        entry.language,
                        entry.interpreters.join(", ")
                    ),
                }
            }
            println!();
            println!("Nothing here is downloaded. A backend is a small script this binary");
            println!("carries; the first formula in a document writes it into");
            println!(".hick-cache/formula/ and runs it with the interpreter above.");
            Ok(ExitCode::SUCCESS)
        }
        FormulaCommand::Install(args) => {
            let root = match args.root {
                Some(root) => root,
                None => std::env::current_dir().context("resolving the current directory")?,
            };
            let languages: Vec<String> = if args.languages.is_empty() {
                backend::BACKENDS
                    .iter()
                    .map(|b| b.language.to_string())
                    .collect()
            } else {
                args.languages
            };
            let mut failed = false;
            for language in &languages {
                match backend::install(&root, language) {
                    Ok(path) => println!("{language}: {}", path.display()),
                    Err(error) => {
                        // A missing interpreter is not a failed install of
                        // the OTHER languages; keep going and report at the
                        // end, so `hick formula install` on a machine with
                        // python but no node still installs python.
                        eprintln!("{language}: {error:#}");
                        failed = true;
                    }
                }
            }
            Ok(if failed {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}

/// The distinct tools a catalogue's installers need, in catalogue order.
///
/// Derived rather than written down: the message that names them is read by
/// somebody who has none of them, and a hardcoded list that fell behind the
/// catalogue would send that person to install the wrong thing.
fn installer_tools(plans: Vec<hickory_cli::tool_install::InstallPlan>) -> String {
    let mut tools: Vec<String> = Vec::new();
    for plan in plans {
        if !tools.contains(&plan.tool) {
            tools.push(plan.tool);
        }
    }
    tools.join(", ")
}

fn cmd_lsp(command: LspCommand) -> Result<ExitCode> {
    use hickory_cli::lsp_install;

    match command {
        LspCommand::List => {
            report_catalogue(
                "Language servers `hick lsp install` can fetch:",
                lsp_install::plans(),
                "hick lsp install",
                lsp_install::SERVERS_DIR,
            );
            Ok(ExitCode::SUCCESS)
        }
        LspCommand::Install(args) => {
            let root = match args.root {
                Some(root) => root,
                None => std::env::current_dir().context("resolving the current directory")?,
            };
            let languages = chosen_languages(args.languages, lsp_install::plans());
            if languages.is_empty() {
                println!(
                    "Nothing to install: none of the installers' tools ({}) are on this \
                     machine.\nInstall one of them, or install a language server yourself — \
                     either way `hick-lsp` will find it.",
                    installer_tools(lsp_install::plans()),
                );
                return Ok(ExitCode::SUCCESS);
            }
            for language in &languages {
                println!("Installing the {language} language server, sandboxed…");
                let prefix = lsp_install::install(&root, language)?;
                println!("  installed into {}", prefix.display());
            }
            println!(
                "\nOpen a document — the server is found automatically, with no configuration."
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn cmd_dap(command: DapCommand) -> Result<ExitCode> {
    use hickory_cli::dap_install;

    match command {
        DapCommand::List => {
            // Both halves, because the error a person gets here from says
            // "`hick dap list` names them" about the languages hick can
            // DEBUG — and this command used to name only the ones it can
            // INSTALL. Someone with a Go file was sent to a page that did
            // not mention Go.
            println!("Languages hick can debug:\n");
            for language in hick_dap::known_languages() {
                println!("  {language}");
                for line in textwrap_lines(&hick_dap::how_to_get(language)) {
                    println!("      {line}");
                }
            }
            println!();
            report_catalogue(
                "Of those, the adapters `hick dap install` can fetch:",
                dap_install::plans(),
                "hick dap install",
                dap_install::ADAPTERS_DIR,
            );
            Ok(ExitCode::SUCCESS)
        }
        DapCommand::Install(args) => {
            let root = match args.root {
                Some(root) => root,
                None => std::env::current_dir().context("resolving the current directory")?,
            };
            let languages = chosen_languages(args.languages, dap_install::plans());
            if languages.is_empty() {
                println!(
                    "Nothing to install: none of the installers' tools ({}) are on this \
                     machine.\nInstall one of them, or install an adapter yourself — either way \
                     hick will find it.",
                    installer_tools(dap_install::plans()),
                );
                return Ok(ExitCode::SUCCESS);
            }
            for language in &languages {
                println!("Installing the {language} debug adapter, sandboxed…");
                let prefix = dap_install::install(&root, language)?;
                println!("  installed into {}", prefix.display());
            }
            println!("\nSet a breakpoint in a document — the adapter is found automatically.");
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Wrap a sentence at a width a terminal can hold, on word boundaries.
///
/// Not a dependency: this is the only place anything here wraps prose, and a
/// crate for it would be a larger claim than the job.
fn textwrap_lines(text: &str) -> Vec<String> {
    const WIDTH: usize = 72;
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > WIDTH {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// The languages to install: what was asked for, or everything possible.
///
/// Installing everything is a choice a person can reasonably make, but it
/// must be the one they typed, so it only happens on a bare `install`.
fn chosen_languages(
    asked: Vec<String>,
    plans: Vec<hickory_cli::tool_install::InstallPlan>,
) -> Vec<String> {
    if !asked.is_empty() {
        return asked;
    }
    plans
        .into_iter()
        .filter(|plan| plan.blocked.is_none())
        .map(|plan| plan.language)
        .collect()
}

/// Print a catalogue the same way for both tools.
fn report_catalogue(
    heading: &str,
    plans: Vec<hickory_cli::tool_install::InstallPlan>,
    command: &str,
    prefix: &str,
) {
    println!("{heading}\n");
    for plan in plans {
        println!("  {} — {}", plan.language, plan.package);
        println!("      {}", plan.reason);
        match plan.blocked {
            Some(reason) => println!("      unavailable: {reason}"),
            None => println!("      ready: `{command} {}`", plan.language),
        }
    }
    println!(
        "\nAnything already installed on your machine is preferred over these.\nInstalls are \
         confined: they may write only {prefix}, and nothing else on your machine."
    );
}

/// The Windows sandbox launcher (see `Command::SandboxRun`).
///
/// Its exit code IS the confined command's, because the executor above it
/// reads the exit status to decide whether the cell failed. Swallowing a
/// non-zero code here would turn every failing cell into a passing one.
#[cfg(windows)]
fn cmd_sandbox_run(args: SandboxRunArgs) -> Result<ExitCode> {
    use hickory_executor_sandbox::appcontainer::{Confinement, run};

    let command = args.command.join(" ");
    let code = run(Confinement {
        workdir: &args.workdir,
        command: &command,
        allow_network: args.allow_network,
    })?;
    // A Windows exit code is a u32; ExitCode carries a u8. Anything that does
    // not fit is reported as failure rather than truncated, because
    // truncation can turn a non-zero code into zero.
    Ok(match u8::try_from(code) {
        Ok(byte) => ExitCode::from(byte),
        Err(_) => ExitCode::FAILURE,
    })
}

/// On every other platform this subcommand cannot do anything, and says so.
///
/// It exists here rather than being compiled out so that the argument
/// parsing, the help text and the dispatch are identical on all platforms —
/// a subcommand that vanishes per target is one that only breaks on the
/// target nobody is building.
#[cfg(not(windows))]
fn cmd_sandbox_run(_args: SandboxRunArgs) -> Result<ExitCode> {
    anyhow::bail!(
        "`hick __sandbox-run` is the Windows sandbox launcher and does nothing on this \
         platform.\nOn Linux and macOS the sandbox is applied by `bwrap` or `sandbox-exec`, \
         which the executor invokes directly — there is nothing here to run by hand.\n\
         If you meant to run a document confined, use `HICKORY_EXECUTOR=sandbox hick run <doc>`."
    )
}

fn cmd_init(args: InitArgs) -> Result<ExitCode> {
    let report = hickory_cli::run_init(&args.dir)?;
    hickory_cli::print_init_report(&report);
    Ok(ExitCode::SUCCESS)
}

fn emit_json(mut docs: Vec<serde_json::Value>) -> Result<()> {
    let value = if docs.len() == 1 {
        docs.pop().unwrap()
    } else {
        serde_json::Value::Array(docs)
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn indent(s: &str) -> String {
    s.lines()
        .map(|l| format!("    | {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `hick refresh` — rewrite stale transform passages from their inputs.
///
/// This is the only command in the tool that calls a model, and that is a
/// deliberate boundary: `run`, `test`, and `weave` stay free, offline, and
/// deterministic, so a document containing LLM-written prose is still safe to
/// verify in CI.
///
/// The model is shown the previous passage along with the new input. That is
/// what keeps refreshes stable — and what makes a passage you edited by hand
/// survive: your wording is the starting point, not something to be
/// regenerated over.
async fn cmd_refresh(args: RefreshArgs) -> Result<ExitCode> {
    use hickory_agent::{LlmClient, Message, Role};

    let docs = expand_docs(&args.path)?;
    let mut rewrote = 0usize;
    let mut stale_total = 0usize;

    for doc_path in &docs {
        let source = std::fs::read_to_string(doc_path)?;
        // The same view `hick test` checks against, so a passage this writes is
        // not stale the instant it is written.
        let parsed = hickory_cli::transform_document(doc_path, &source)?;

        // Collect the work first: each transform's input, instruction, current
        // passage, and the byte span of its body.
        let mut jobs: Vec<RefreshJob> = Vec::new();
        for tag in hickory_cli::own_transforms(&parsed) {
            let (select, instruct) = hickory_cli::transform_spec(tag);
            let recorded = tag.get_attribute("from").unwrap_or_default().to_string();
            let input = hickory_cli::transform_input(&parsed, &select);
            let fingerprint = hick_lang::transform_fingerprint(&input, &instruct);
            if fingerprint == recorded && !args.all {
                continue;
            }
            stale_total += 1;
            let Some((open_from, body_from, body_to)) = body_span(&source, tag) else {
                eprintln!(
                    "skip {}:{}: cannot locate the passage body",
                    doc_path.display(),
                    tag.source_line
                );
                continue;
            };
            let previous = source[body_from..body_to].to_string();
            jobs.push(RefreshJob {
                open_from,
                body_from,
                body_to,
                input,
                instruct,
                previous,
                fingerprint,
            });
        }

        if jobs.is_empty() {
            continue;
        }
        if args.dry_run {
            for job in &jobs {
                println!("would refresh {}: {}", doc_path.display(), job.instruct);
            }
            continue;
        }
        let store = hickory_agent::KeyStore::desktop();
        let llm = hickory_agent::client_for_with_store(
            &hickory_agent::resolve_selector_with_store(args.provider.as_deref(), &store)?,
            args.model.as_deref(),
            &store,
        )?;

        // Apply back-to-front so earlier spans stay valid.
        jobs.sort_by_key(|j| std::cmp::Reverse(j.open_from));
        let mut updated = source.clone();
        for RefreshJob {
            open_from,
            body_from: from,
            body_to: to,
            input,
            instruct,
            previous,
            fingerprint,
        } in jobs
        {
            // A never-written passage (ingest leaves them empty) gets a
            // prompt that says so: shown an empty "previous passage" to
            // preserve, a model tends to preserve the scaffolding instead.
            let prompt = if previous.trim().is_empty() {
                format!(
                    "Write a passage from the input below.\n\n\
                     Instruction: {instruct}\n\n\
                     Input:\n{input}\n\n\
                     Reply with the passage only. No preamble, no code fences, and do \
                     not repeat the instruction or the input."
                )
            } else {
                format!(
                    "Rewrite the passage below so it is accurate for the current input.\n\n\
                     Instruction: {instruct}\n\n\
                     Current input:\n{input}\n\n\
                     Previous passage (keep its voice, structure, and any wording that is \
                     still correct — change only what the new input requires):\n{previous}\n\n\
                     Reply with the passage only. No preamble, no code fences."
                )
            };
            let passage = llm
                .complete(vec![
                    Message::new(
                        Role::System,
                        "You rewrite short documentation passages. You preserve the author's \
                         voice and change as little as possible.",
                    ),
                    Message::new(Role::User, prompt),
                ])
                .await?;
            let passage = format!("\n{}\n", passage.trim());
            updated.replace_range(from..to, &passage);
            // What the passage itself names as its sources — `#id` tokens
            // that match the fragments it was shown — becomes `cites=`. That
            // is DECLARED provenance: the model's own claim about what it
            // leaned on, kept apart from `from=` (what it was shown) and
            // drawn by the app as an assertion.
            let cites = cited_ids(&passage, &input);
            let mut attrs: Vec<(&str, &str)> = vec![
                ("from", &fingerprint),
                ("provider", llm.provider_name()),
                ("model", llm.model_name()),
            ];
            if !cites.is_empty() {
                attrs.push(("cites", &cites));
            }
            // Re-stamp the fingerprint this passage now attests to, and name
            // the model that wrote it: the fingerprint says which bytes under
            // which instruction, and this says by whom — the same thing a
            // session records, and the one fact nobody can reconstruct later.
            updated = restamp(&updated, open_from, from, &attrs);
            rewrote += 1;
        }
        std::fs::write(doc_path, &updated)?;
        println!("refreshed {}", doc_path.display());
    }

    if args.dry_run {
        println!("{stale_total} transform(s) stale");
    } else {
        println!("{rewrote} passage(s) rewritten");
    }
    Ok(ExitCode::SUCCESS)
}

/// The `#id`s a passage mentions that name fragments of its input — the
/// `[#id]` labels the input carries — as a comma-separated selector list, in
/// order of first mention, each once. Empty when the passage cites nothing.
fn cited_ids(passage: &str, input: &str) -> String {
    let shown: std::collections::HashSet<&str> = input
        .lines()
        .filter_map(|l| l.strip_prefix("[#"))
        .filter_map(|l| l.split(']').next())
        .collect();
    let mut out: Vec<&str> = Vec::new();
    let mut rest = passage;
    while let Some(i) = rest.find('#') {
        let after = &rest[i + 1..];
        let end = after
            .find(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(after.len());
        let id = &after[..end];
        if !id.is_empty() && shown.contains(id) && !out.contains(&id) {
            out.push(id);
        }
        rest = &after[end..];
    }
    out.iter()
        .map(|id| format!("#{id}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// One stale passage to rewrite: where its tag opens, where its body lies,
/// and what the model needs to see.
struct RefreshJob {
    open_from: usize,
    body_from: usize,
    body_to: usize,
    input: String,
    instruct: String,
    previous: String,
    fingerprint: String,
}

/// Byte span of a tag: where its opening tag starts, and its body — between
/// the `>` of the opening tag and the `<` of its closing tag.
fn body_span(source: &str, tag: &hick_lang::HickTag) -> Option<(usize, usize, usize)> {
    let open = tag.source_span?;
    let start = open.end;
    let close = source[start..].find("</")? + start;
    Some((open.start, start, close))
}

/// Set attributes on the opening tag at `open_start..body_start`, rewriting
/// each that is present and inserting the rest after the tag name.
///
/// Works from the tag's own span, not a search for `<hick:transform`: a
/// document is free to bind any prefix (`<slack:transform>` is the same
/// element), and a refresh that could not find the tag it had just rewritten
/// would leave the passage new and the fingerprint old — stale forever.
fn restamp(source: &str, open_start: usize, body_start: usize, attrs: &[(&str, &str)]) -> String {
    let mut tag_text = source[open_start..body_start].to_string();
    // Reverse, so attributes inserted after the tag name end up in the order
    // given rather than each one pushing the last further right.
    for (name, value) in attrs.iter().rev() {
        let needle = format!("{name}=\"");
        match tag_text.find(&needle) {
            Some(i) => {
                let value_start = i + needle.len();
                let value_end = value_start + tag_text[value_start..].find('"').unwrap_or(0);
                tag_text.replace_range(value_start..value_end, value);
            }
            None => {
                // Insert right after the tag name: `<prefix:transform`.
                let name_end = tag_text
                    .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    .unwrap_or(tag_text.len());
                tag_text.insert_str(name_end, &format!(" {name}=\"{value}\""));
            }
        }
    }
    let mut out = source.to_string();
    out.replace_range(open_start..body_start, &tag_text);
    out
}

/// `hick doc <tool>` — one tool, one process, one observation.
///
/// The whole command group is a translation layer: arguments become a
/// [`ToolInvocation`], `hickory_cli::doc_tools::run_doc_tool` executes it
/// through the same `EditSession` the built-in agent drives, and the
/// observation is printed. No editing logic lives here, which is the point —
/// an external coding agent and our own agent must not be able to drift
/// apart in what an edit means.
async fn cmd_doc(cmd: DocCommand) -> Result<ExitCode> {
    use hickory_cli::doc_tools::{
        DocToolRequest, OutputFormat, edit_args, print_outcome, read_input, run_doc_tool,
    };

    /// Resolve the document: the one named, or the only one here.
    fn resolve_doc(doc: Option<PathBuf>) -> Result<PathBuf> {
        if let Some(d) = doc {
            return Ok(d);
        }
        let cwd = std::env::current_dir()?;
        hickory_cli::doc_tools::sole_document(&cwd).ok_or_else(|| {
            anyhow::anyhow!(
                "no document given, and {} does not hold exactly one .hick file.\n\
                 Name the document: hick doc <command> path/to/doc.hick",
                cwd.display()
            )
        })
    }

    let (doc, tool, args, input, common) = match cmd {
        DocCommand::Read(a) => {
            let args = a
                .upstream
                .map(|u| vec![("doc".to_string(), u)])
                .unwrap_or_default();
            (resolve_doc(a.doc)?, "read_doc", args, None, a.common)
        }
        DocCommand::ReadOutput(a) => {
            let mut args = vec![("path".to_string(), a.path)];
            if a.lineage {
                args.push(("with_lineage".to_string(), "true".to_string()));
            }
            (resolve_doc(a.doc)?, "read_output", args, None, a.common)
        }
        DocCommand::ReadFile(a) => {
            let mut args = vec![("path".to_string(), a.path)];
            if let Some(from) = a.from {
                args.push(("from".to_string(), from.to_string()));
            }
            if let Some(to) = a.to {
                args.push(("to".to_string(), to.to_string()));
            }
            (resolve_doc(a.doc)?, "read_file", args, None, a.common)
        }
        DocCommand::EditOutput(a) => {
            if a.path.is_none() {
                anyhow::bail!(
                    "edit-output needs --path <output file>. \
                     `hick doc read-output --path …` lists what a document produces."
                );
            }
            let args = edit_args(
                a.path.as_deref(),
                a.run.as_deref(),
                a.after.as_deref(),
                a.occurrence,
            )?;
            let input = read_input(a.input.as_deref())?;
            (resolve_doc(a.doc)?, "edit_output", args, input, a.common)
        }
        DocCommand::Edit(a) => {
            let mut args = edit_args(None, a.run.as_deref(), a.after.as_deref(), a.occurrence)?;
            if let Some(u) = a.upstream {
                args.insert(0, ("doc".to_string(), u));
            }
            let input = read_input(a.input.as_deref())?;
            (resolve_doc(a.doc)?, "edit_doc", args, input, a.common)
        }
        DocCommand::Verify(a) => (resolve_doc(a.doc)?, "verify", Vec::new(), None, a.common),
    };

    let request = DocToolRequest {
        doc,
        tool: tool.to_string(),
        args,
        input,
        params: params_with_features(&common.params, &common.features),
        format: if common.json {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        },
        session: hickory_cli::doc_tools::session_from(common.session),
    };
    let outcome = run_doc_tool(&request).await?;
    let code = print_outcome(&outcome, request.format)?;
    Ok(ExitCode::from(code))
}

#[cfg(test)]
mod restamp_tests {
    use super::restamp;

    /// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
    #[test]
    fn restamp_rewrites_an_existing_fingerprint_and_inserts_the_model() {
        let src =
            "<hick:transform select=\"#a\" from=\"old\" instruct=\"x\">\nbody\n</hick:transform>";
        let body = src.find('>').unwrap() + 1;
        let out = restamp(
            src,
            0,
            body,
            &[
                ("from", "new1"),
                ("provider", "anthropic"),
                ("model", "claude-sonnet-5"),
            ],
        );
        assert!(out.starts_with(
            "<hick:transform provider=\"anthropic\" model=\"claude-sonnet-5\" select=\"#a\" from=\"new1\" instruct=\"x\">\nbody"
        ), "{out}");
    }

    /// The passage's own `#id` mentions, filtered to what it was shown,
    /// become `cites=` — declared, and only ever what the input labelled.
    #[test]
    fn cited_ids_keeps_only_ids_the_input_labelled() {
        let input = "[#m1] Sentence.\n\n[#transcript-u3] Sam: The SLO.\n\n[#p95] 212 ms.";
        let passage = "BACKED by [#transcript-u3] and #p95; also #p95 again and #nope and #m1.";
        assert_eq!(super::cited_ids(passage, input), "#transcript-u3,#p95,#m1");
        assert_eq!(super::cited_ids("nothing here", input), "");
    }

    /// A document may bind any prefix to the namespace; refresh must find the
    /// tag it is rewriting by its span, not by the spelling `<hick:transform`.
    #[test]
    fn restamp_works_under_a_rebound_prefix() {
        let src = "intro\n<slack:transform select=\"#a\" instruct=\"x\">\nbody\n</slack:transform>";
        let open = src.find("<slack:").unwrap();
        let body = src.find('>').unwrap() + 1;
        let out = restamp(src, open, body, &[("from", "abcd1234")]);
        assert!(
            out.contains("<slack:transform from=\"abcd1234\" select=\"#a\""),
            "{out}"
        );
    }
}
