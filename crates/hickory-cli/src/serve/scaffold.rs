//! File → New Project: `dotnet new`, as a form, a terminal, and a commit.
//!
//! Five routes, and the shape of them is the argument. Reading the catalogue
//! and reading one template's options are separate because the second is a
//! second `dotnet` process and the dialog only needs it once a template is
//! chosen — a New Project dialog that spawned forty-six help invocations to
//! open would be unusable on a laptop.
//!
//! ## Why there is a preview route
//!
//! The dialog shows the exact commit message it is about to write, the way
//! the Insert panel shows its markup, and for the same reason: a recipe is a
//! text a person will read in `git log` for years, and a dialog that writes
//! one you never saw teaches you nothing. The preview is the same function,
//! called over HTTP, so what is shown and what is committed cannot be two
//! renderers that agree today.
//!
//! ## The scaffolder runs in a terminal
//!
//! It used to run as a `spawn_blocking` subprocess whose stdout was thrown
//! away and whose stderr survived as six joined lines inside an error string.
//! That is the shape this product already rejected for builds and for tests:
//! **a command a person asked for is watched in a terminal, not reported by a
//! spinner**, because a failing command says why in its own words and a
//! summary throws all of that away. So `POST /api/scaffold` opens a
//! `hick_term` session — the same one `POST /api/tests/run` opens — and
//! answers with it, and the dialog hands it to the workspace as a tab.
//! `docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md`.
//!
//! ## Creating is still one act
//!
//! A terminal does not weaken that. The session runs in a scratch directory,
//! and a watcher commits what it wrote **the moment it exits zero**, through
//! an index of its own, with the recipe in the trailers
//! (`crate::scaffold_commit`). There is no moment between the two for an edit
//! to slip into the recipe commit, which is what keeps the commit
//! upgradeable. A scaffold that fails commits nothing and leaves the working
//! tree as it was: there is no half-finished document to keep, because there
//! is no document. The watcher writes its verdict **into the session's own
//! scrollback**, so the commit and the command that earned it are one thing to
//! read. `docs/specs/freeform/lenses.md`, step 3.
//!
//! ## A project may be made anywhere
//!
//! The location is any folder on this machine, not a subfolder of the one the
//! app has open. The repository that records the recipe is then whichever one
//! contains that location — resolved by `scaffold_commit::resolve_target`,
//! never assumed to be the open folder — and a location inside no repository
//! at all is the typed `NotARepository`, answered with an offer to make one
//! (`POST /api/git/init`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

use super::api::{ApiError, ApiResult};
use super::{LocalState, OpenWhere};
use crate::scaffold::{NoDotnetSdk, ScaffoldSpec};
use crate::scaffold_commit::{self, NotARepository, Target};
use crate::toolchain::Toolchain;

/// How often the watcher asks whether the scaffolder has finished. Fast
/// enough that the commit lands while the person is still looking at the
/// terminal, slow enough to be free.
const POLL_MS: u64 = 120;

/// Turn a scaffold failure into an API error, keeping the two distinctions
/// the dialog draws its own screens from.
///
/// A missing SDK is not a failed request, it is a machine that cannot do this
/// at all — so it carries `missing: "dotnet"` in the detail, and the client
/// keys off **that**, never off the sentence. A location outside any
/// repository is the same shape, and carries the folder as well, because the
/// button it earns has to name where it would run `git init`. The lesson is
/// `hick_dap::MissingAdapter`'s: a reworded message must not be able to take
/// a screen away.
fn scaffold_error(e: anyhow::Error) -> ApiError {
    if e.downcast_ref::<NoDotnetSdk>().is_some() {
        return ApiError::unprocessable(format!("{}", NoDotnetSdk))
            .with_detail(json!({ "missing": "dotnet" }));
    }
    if let Some(missing) = e.downcast_ref::<NotARepository>() {
        return ApiError::unprocessable(format!("{missing}")).with_detail(json!({
            "missing": "repository",
            "path": missing.path.to_string_lossy(),
        }));
    }
    ApiError::unprocessable(format!("{e:#}"))
}

/// The same, for a scaffolder that is not `dotnet`.
///
/// A missing tool is a machine that cannot do this at all, not a failed
/// request, and the dialog draws its own screen from `missing` rather than
/// from the sentence — the rule `hick_dap::MissingAdapter` set and
/// `NoDotnetSdk` followed. Extending it was not optional: without this, a
/// machine with no `uv` answered "no such file or directory" and the dialog
/// had nothing to key off.
fn scaffold_error_for(toolchain: Toolchain, e: anyhow::Error) -> ApiError {
    if toolchain == Toolchain::Dotnet {
        return scaffold_error(e);
    }
    let shape = toolchain.shape();
    let text = format!("{e:#}");
    if text.contains("not on this machine's PATH") {
        return ApiError::unprocessable(format!(
            "no `{program}` on this machine's PATH, so there is nothing to scaffold {label} \
             with.\n  Next step: install {program}, then reopen this dialog — hick will find \
             it. Nothing here installs a language toolchain for you.",
            program = shape.program,
            label = shape.label,
        ))
        .with_detail(json!({ "missing": shape.program }));
    }
    ApiError::unprocessable(text)
}

/// Which scaffolder a request is about.
///
/// Absent means `dotnet`, which is what every client sent before there was
/// more than one — a dialog that has not been updated keeps working.
#[derive(Deserialize, Default)]
pub struct ToolchainQuery {
    #[serde(default)]
    pub toolchain: Option<String>,
}

/// Read the toolchain out of whatever named it, refusing an id nothing
/// serves rather than quietly scaffolding with the wrong tool.
fn chosen(id: Option<&str>) -> ApiResult<Toolchain> {
    match id.map(str::trim).filter(|id| !id.is_empty()) {
        None => Ok(Toolchain::Dotnet),
        Some(id) => Toolchain::parse(id).ok_or_else(|| {
            ApiError::bad_request(format!(
                "no scaffolder called `{id}`. This build knows {}.",
                Toolchain::ALL
                    .iter()
                    .map(|t| t.id())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }),
    }
}

/// `GET /api/scaffold/templates` — what this machine can scaffold, and where
/// a project would land by default.
pub async fn templates(
    State(state): State<LocalState>,
    Query(query): Query<ToolchainQuery>,
) -> ApiResult<Json<Value>> {
    let toolchain = chosen(query.toolchain.as_deref())?;
    // `dotnet new list` is a subprocess that reads a template cache off disk;
    // on a cold cache it rebuilds it, which is seconds rather than
    // milliseconds. Holding a runtime worker for that is what
    // `spawn_blocking` is for.
    let catalog = tokio::task::spawn_blocking(move || toolchain.catalog())
        .await
        .map_err(|e| ApiError::internal(format!("the template listing did not finish: {e}")))?
        .map_err(|e| scaffold_error_for(toolchain, e))?;
    let image = toolchain.image(&catalog.sdk_version);
    // Measured, never declared: a dialog offering `uv` on a machine without
    // it is a dialog that fails after the person has filled in a form. The
    // same rule `hick lang` follows for every other capability.
    let installed = tokio::task::spawn_blocking(Toolchain::installed)
        .await
        .unwrap_or_default();
    let root = state.index.root().to_path_buf();
    let root = root.canonicalize().unwrap_or(root);
    Ok(Json(json!({
        "kind": toolchain.id(),
        "toolchains": installed
            .iter()
            .map(|(tool, version)| json!({
                "id": tool.id(),
                "label": tool.shape().label,
                "language": tool.shape().language,
                "version": version,
                "templated": tool.shape().templated,
            }))
            .collect::<Vec<_>>(),
        "sdk_version": catalog.sdk_version,
        "image": image,
        // The location field opens on the folder the app has open, spelled
        // absolutely: the person is choosing a place on their machine, and a
        // field that starts as `.` hides which place that is.
        "location": root.to_string_lossy(),
        "separator": std::path::MAIN_SEPARATOR_STR,
        // Whether the program hosting this engine can put a native folder
        // chooser in front of anyone. A browser tab cannot, and a button that
        // always fails is worse than no button.
        "can_pick_folder": state.has_shell(),
        "templates": catalog.templates,
    })))
}

#[derive(Deserialize)]
pub struct DetailQuery {
    /// The template's short name (`webapi`).
    pub template: String,
    /// Which language's options to ask for. Absent means the template's own
    /// default, which is what `dotnet new <name> --help` answers with.
    #[serde(default)]
    pub language: Option<String>,
    /// Which scaffolder. Absent means `dotnet`.
    #[serde(default)]
    pub toolchain: Option<String>,
}

/// `GET /api/scaffold/options?template=webapi&language=C%23` — one template's
/// options, as fields.
pub async fn options(
    State(_state): State<LocalState>,
    Query(query): Query<DetailQuery>,
) -> ApiResult<Json<Value>> {
    if query.template.trim().is_empty() {
        return Err(ApiError::bad_request(
            "name a template — `?template=webapi`. `GET /api/scaffold/templates` lists them.",
        ));
    }
    let toolchain = chosen(query.toolchain.as_deref())?;
    let template = query.template.clone();
    let language = query.language.clone().filter(|l| !l.trim().is_empty());
    let detail =
        tokio::task::spawn_blocking(move || toolchain.detail(&template, language.as_deref()))
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "reading the template's options did not finish: {e}"
                ))
            })?
            .map_err(|e| scaffold_error_for(toolchain, e))?;
    Ok(Json(
        serde_json::to_value(detail).unwrap_or_else(|_| json!({})),
    ))
}

/// What the dialog is asking to be scaffolded and committed.
#[derive(Deserialize)]
pub struct CreateRequest {
    /// The folder the project's own folder is made **in**: absolute,
    /// `~`-prefixed, or relative to the folder the app has open. Empty means
    /// the open folder, which is what the dialog starts on.
    #[serde(default)]
    pub location: String,
    /// Make a git repository in the location first, if it is not already in
    /// one. The dialog's checkbox — ticked for it by the preview, which knows
    /// before the person presses anything whether one is needed.
    ///
    /// A checkbox rather than a screen after the refusal: by the time you
    /// have typed a location and a project name you have said what you want,
    /// and being stopped to confirm one `git init` is a question the form
    /// could have asked while you were reading it.
    #[serde(default)]
    pub init_repository: bool,
    /// Where to open the project once it is committed. Never before, and
    /// never at all if the scaffolder failed — a window that vanishes and
    /// takes the terminal explaining the failure with it is the worst
    /// possible answer to a failure.
    #[serde(default = "no_window")]
    pub open: OpenWhere,
    /// Which scaffolder. Absent means `dotnet`, so a client that predates
    /// there being a choice keeps working.
    #[serde(default)]
    pub toolchain: Option<String>,
    /// `spec.output` arrives as the project's folder *name*; what the recipe
    /// records is the same folder relative to the repository, which only
    /// `resolve_target` can work out.
    #[serde(flatten)]
    pub spec: ScaffoldSpec,
}

fn no_window() -> OpenWhere {
    OpenWhere::None
}

/// The place in the message a preview cannot fill in yet.
const OUTPUT_PENDING: &str = "<the tree hash, once it is committed>";

/// The spec as it will be recorded: the same one, with `output` moved from
/// "the folder you named" to "that folder, from the repository root", which
/// is where a replay runs the recipe from.
fn recorded(spec: &ScaffoldSpec, target: &Target) -> ScaffoldSpec {
    ScaffoldSpec {
        output: target.output.clone(),
        ..spec.clone()
    }
}

/// `POST /api/scaffold/preview` — the exact commit, without making it.
///
/// This route never refuses for want of a repository. The person is still
/// typing, and a preview that blanks out the moment the location leaves a
/// repository teaches them nothing; it says `repository: null` instead and
/// the dialog offers to make one before the button is ever pressed.
pub async fn preview(
    State(state): State<LocalState>,
    Json(body): Json<CreateRequest>,
) -> ApiResult<Json<Value>> {
    let toolchain = chosen(body.toolchain.as_deref())?;
    let root = state.index.root().to_path_buf();
    match scaffold_commit::resolve_target(&root, &body.location, &body.spec.output) {
        Ok(target) => {
            let spec = recorded(&body.spec, &target);
            Ok(Json(json!({
                "output": target.output,
                "folder": target.dir.to_string_lossy(),
                "repository": target.repo_root.to_string_lossy(),
                "command": toolchain.command(&spec),
                "message": toolchain.commit_message(&spec, OUTPUT_PENDING, &target.output),
            })))
        }
        Err(e) => {
            let missing = e.downcast_ref::<NotARepository>().cloned();
            // Without a repository there is no root to make `-o` relative to,
            // so the command is previewed with the folder as named — true of
            // what will run, and honest about what is not yet decided.
            let output = body.spec.output.trim().trim_end_matches('/').to_string();
            let spec = ScaffoldSpec {
                output: output.clone(),
                ..body.spec
            };
            Ok(Json(json!({
                "output": output,
                "folder": missing
                    .as_ref()
                    .map(|m| m.path.join(&output).to_string_lossy().into_owned()),
                "repository": Value::Null,
                "needs_repository": missing.as_ref().map(|m| m.path.to_string_lossy()),
                "problem": format!("{e:#}"),
                "command": toolchain.command(&spec),
                "message": toolchain.commit_message(&spec, OUTPUT_PENDING, &output),
            })))
        }
    }
}

/// What became of one scaffold, keyed by the terminal session that ran it.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// The scaffolder is still running.
    Running,
    /// It exited zero and the commit was made.
    Committed(Box<CommittedScaffold>),
    /// It exited non-zero, or the commit could not be made. Nothing was
    /// committed either way.
    Failed { reason: String },
}

#[derive(Debug, Clone)]
pub struct CommittedScaffold {
    pub sha: String,
    pub short: String,
    pub files: Vec<String>,
    pub message: String,
    pub output: String,
    pub output_tree: String,
    pub repository: String,
}

/// Every scaffold this session has started, by terminal session id.
pub type Scaffolds = Arc<Mutex<HashMap<String, Outcome>>>;

fn record(scaffolds: &Scaffolds, id: &str, outcome: Outcome) {
    if let Ok(mut map) = scaffolds.lock() {
        map.insert(id.to_string(), outcome);
    }
}

/// `POST /api/scaffold` — run the scaffolder in a terminal, and commit what
/// it wrote the moment it exits zero.
///
/// Answers `202` with the session, the way `POST /api/tests/run` answers with
/// one: the work is watchable, not finished. `GET /api/scaffold/result` says
/// how it ended.
pub async fn create(
    State(state): State<LocalState>,
    Json(body): Json<CreateRequest>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let toolchain = chosen(body.toolchain.as_deref())?;
    if body.spec.name.trim().is_empty() {
        return Err(ApiError::bad_request(match toolchain {
            Toolchain::Dotnet => {
                "a project needs a name — it becomes the .NET root namespace and the \
                 assembly name."
            }
            _ => "a project needs a name — it becomes the package name.",
        }));
    }
    let root = state.index.root().to_path_buf();
    // The checkbox, honoured before the location is resolved. "Create a git
    // repository" means "see to it that there is one", so a location already
    // inside one is left alone — never nested inside it.
    if body.init_repository {
        let location = scaffold_commit::absolute_folder(&root, &body.location);
        tokio::task::spawn_blocking(move || scaffold_commit::ensure_repository(&location))
            .await
            .map_err(|e| ApiError::internal(format!("making the repository did not finish: {e}")))?
            .map_err(scaffold_error)?;
    }
    let target = scaffold_commit::resolve_target(&root, &body.location, &body.spec.output)
        .map_err(scaffold_error)?;
    let spec = recorded(&body.spec, &target);

    // The cheap refusals first, and all of them before a terminal exists: a
    // session that opens only to print "that folder is not empty" is a tab
    // the person has to close for a mistake a field could have caught.
    let (target, spec) =
        tokio::task::spawn_blocking(move || -> anyhow::Result<(Target, ScaffoldSpec)> {
            scaffold_commit::refuse_occupied(&target.repo_root, &target.output)?;
            // Before a terminal exists: "the tool is not installed" must stay
            // a typed refusal rather than arriving as a shell's "command not
            // found" in a tab nobody asked for.
            toolchain.version()?;
            Ok((target, spec))
        })
        .await
        .map_err(|e| ApiError::internal(format!("checking the scaffold did not finish: {e}")))?
        .map_err(|e| scaffold_error_for(toolchain, e))?;

    // Into scratch, never the repository: a scaffolder that fails halfway
    // leaves nothing behind, and one that succeeds is committed whole. The
    // directory the scaffolder writes is named after the project's own
    // folder, so the line on screen reads like the recipe.
    let scratch = tempfile::tempdir()
        .map_err(|e| ApiError::internal(format!("could not make a scratch directory: {e}")))?;
    let leaf = target
        .output
        .rsplit('/')
        .next()
        .filter(|l| !l.is_empty())
        .unwrap_or("out")
        .to_string();
    let argv = toolchain.argv(&spec, &leaf);
    let session = state
        .terminals
        .open(hick_term::SessionSpec {
            title: format!("New project: {}", spec.name),
            cwd: scratch.path().to_path_buf(),
            argv,
            monitor: false,
        })
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;

    record(&state.scaffolds, &session.id, Outcome::Running);
    let watcher = Watch {
        state: state.clone(),
        toolchain,
        session_id: session.id.clone(),
        scratch,
        into: leaf,
        target: target.clone(),
        spec,
        open: body.open,
    };
    tokio::spawn(watcher.run());

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "session": session.summary(),
            "output": target.output,
            "folder": target.dir.to_string_lossy(),
            "repository": target.repo_root.to_string_lossy(),
        })),
    ))
}

/// The half of a scaffold that happens after the request is answered.
struct Watch {
    state: LocalState,
    session_id: String,
    /// Held for as long as the run, and dropped with this task — which is
    /// what deletes the scaffolder's scratch directory.
    scratch: tempfile::TempDir,
    /// The directory name inside the scratch that the scaffolder wrote.
    into: String,
    target: Target,
    spec: ScaffoldSpec,
    toolchain: Toolchain,
    /// Where the project is opened once it is committed.
    open: OpenWhere,
}

impl Watch {
    async fn run(self) {
        let Some(exit) = self.wait_for_exit().await else {
            // The session was closed before it finished. Nothing ran to
            // completion, so nothing is committed and nothing is claimed.
            record(
                &self.state.scaffolds,
                &self.session_id,
                Outcome::Failed {
                    reason: "the terminal was closed before the scaffolder finished, so nothing \
                             was committed."
                        .to_string(),
                },
            );
            return;
        };
        if exit != 0 {
            // Named for the tool that actually ran: "`dotnet new` exited 1"
            // on a machine where `uv` failed sends a person to the wrong
            // place entirely.
            let phrase = self.toolchain.phrase(&self.spec.template);
            self.say(&format!(
                "\r\n\x1b[31mNothing was committed.\x1b[0m `{phrase}` exited {exit}; the \
                 repository is exactly as it was.\r\n"
            ));
            record(
                &self.state.scaffolds,
                &self.session_id,
                Outcome::Failed {
                    reason: format!(
                        "`{phrase}` exited {exit}. Nothing was committed — its own output, \
                         above, says why."
                    ),
                },
            );
            return;
        }

        let from = self.scratch.path().join(&self.into);
        let repo_root = self.target.repo_root.clone();
        let spec = self.spec.clone();
        let toolchain = self.toolchain;
        let committed = tokio::task::spawn_blocking(move || {
            scaffold_commit::commit_scaffold(&repo_root, toolchain, &spec, &from)
        })
        .await;
        match committed {
            Ok(Ok(commit)) => {
                self.say(&format!(
                    "\r\n\x1b[32mCommitted {short}\x1b[0m — {n} file{s} in {output}/, in {repo}.\r\n\
                     The command above is in the commit's trailers, so this scaffold can be \
                     replayed with a newer SDK later.\r\n",
                    short = commit.short,
                    n = commit.files.len(),
                    s = if commit.files.len() == 1 { "" } else { "s" },
                    output = self.target.output,
                    repo = self.target.repo_root.display(),
                ));
                record(
                    &self.state.scaffolds,
                    &self.session_id,
                    Outcome::Committed(Box::new(CommittedScaffold {
                        sha: commit.sha,
                        short: commit.short,
                        files: commit.files,
                        message: commit.message,
                        output: self.target.output.clone(),
                        output_tree: commit.output_tree,
                        repository: self.target.repo_root.to_string_lossy().into_owned(),
                    })),
                );
                self.open_it();
            }
            Ok(Err(e)) => {
                let reason = format!("{e:#}");
                self.say(&format!(
                    "\r\n\x1b[31mThe scaffolder ran, but the commit did not.\x1b[0m {reason}\r\n"
                ));
                record(
                    &self.state.scaffolds,
                    &self.session_id,
                    Outcome::Failed { reason },
                );
            }
            Err(e) => {
                let reason = format!("committing the scaffold did not finish: {e}");
                self.say(&format!("\r\n\x1b[31m{reason}\x1b[0m\r\n"));
                record(
                    &self.state.scaffolds,
                    &self.session_id,
                    Outcome::Failed { reason },
                );
            }
        }
    }

    /// Open the project, if the person asked for a window.
    ///
    /// Last, and only after the commit: taking over this window is a restart
    /// (a session is a process), which takes this very terminal with it — so
    /// it must never happen while there is still something in that terminal
    /// worth reading. A failure to open is said in the terminal and nowhere
    /// else; the project exists and is committed either way, and that is the
    /// part that mattered.
    fn open_it(&self) {
        if self.open == OpenWhere::None {
            return;
        }
        if self.open == OpenWhere::ThisWindow {
            self.say(
                "\r\nOpening it in this window — a session is a process here, so this window \
                 restarts and this terminal goes with it.\r\n",
            );
        }
        if let Err(e) = self.state.open_folder(&self.target.dir, self.open) {
            self.say(&format!(
                "\r\n\x1b[33mThe project is committed; opening it is what failed.\x1b[0m \
                 {e:#}\r\n"
            ));
        }
    }

    /// Poll until the session has an exit code, or until it is gone.
    async fn wait_for_exit(&self) -> Option<i32> {
        loop {
            let session = self.state.terminals.get(&self.session_id)?;
            let summary = session.summary();
            if let Some(code) = summary.exit_code {
                return Some(code);
            }
            tokio::time::sleep(std::time::Duration::from_millis(POLL_MS)).await;
        }
    }

    /// Say something in the terminal without saying it to any shell: the
    /// verdict belongs beside the output that earned it, and this session has
    /// no shell to mistake it for input.
    fn say(&self, text: &str) {
        if let Some(session) = self.state.terminals.get(&self.session_id) {
            session.inject(text.as_bytes());
        }
    }
}

#[derive(Deserialize)]
pub struct ResultQuery {
    /// The terminal session `POST /api/scaffold` answered with.
    pub session: String,
}

/// `GET /api/scaffold/result?session=<id>` — how the scaffold ended.
///
/// The terminal shows the person what happened; this is how the *app* finds
/// out, so the tree can refresh and the history pane can point at the new
/// commit. Two readers of one act, neither pretending to be the other.
pub async fn result(
    State(state): State<LocalState>,
    Query(query): Query<ResultQuery>,
) -> ApiResult<Json<Value>> {
    let outcome = state
        .scaffolds
        .lock()
        .ok()
        .and_then(|map| map.get(&query.session).cloned());
    match outcome {
        None => Err(ApiError::not_found(format!(
            "no scaffold was started in session {:?}",
            query.session
        ))),
        Some(Outcome::Running) => Ok(Json(json!({ "state": "running" }))),
        Some(Outcome::Failed { reason }) => Ok(Json(json!({ "state": "failed", "error": reason }))),
        Some(Outcome::Committed(c)) => Ok(Json(json!({
            "state": "committed",
            "sha": c.sha,
            "short": c.short,
            "files": c.files,
            "message": c.message,
            "output": c.output,
            "output_tree": c.output_tree,
            "repository": c.repository,
        }))),
    }
}
