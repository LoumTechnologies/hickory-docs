//! Replay, the publication floor, and whether merges go through hick.
//!
//! Three reads of the repository that the document panes need and the git
//! pane does not. Each is computed on every request rather than stored:
//!
//! - **Replay** recomputes exact lineage at a commit by weaving that commit's
//!   document (`crate::replay`). No new data model, nothing cached, and the
//!   grammar boundary reported as itself rather than as a parse failure.
//! - **The floor** is `merge-base(HEAD, published)` (`crate::floor`). A
//!   recorded floor would be a claim the next fetch falsifies.
//! - **The merge driver** check belongs at project open because `hick init`
//!   installs the pre-commit hook, so a fresh clone has neither the driver
//!   nor the hook that would report it missing (`crate::merge_driver`).
//!
//! Read-only, like `serve/git.rs`: nothing here changes the repository.

use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// How far back the slider goes. Past a few hundred versions the slider is
/// not a slider any more, and the answer is a filter rather than more stops.
const HISTORY_LIMIT: usize = 200;

#[derive(Deserialize)]
pub struct ReplayParams {
    /// The commit to replay at. A short sha is fine — git resolves it.
    pub commit: String,
    /// The generated output whose lineage to report. Absent means "list the
    /// outputs that commit's document produced", which is what the client
    /// needs first when the output set itself changed between versions.
    #[serde(default)]
    pub path: Option<String>,
}

/// The repository root and the document's path within it.
fn located(state: &LocalState, id: &str) -> Result<(std::path::PathBuf, String), ApiError> {
    let abs = state
        .index
        .absolute(id)
        .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;
    let dir = abs.parent().unwrap_or(std::path::Path::new("."));
    let root = crate::replay::git_root(dir).ok_or_else(|| {
        ApiError::unprocessable(
            "this folder is not a git repository, and replay reads old \
             documents out of git. A folder of notes that is not under \
             version control is entirely normal — there is simply no history \
             to slide through."
                .to_string(),
        )
    })?;
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    let abs = std::fs::canonicalize(&abs).unwrap_or(abs);
    let rel = abs
        .strip_prefix(&root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|_| ApiError::internal("the document is outside its own repository"))?;
    Ok((root, rel))
}

/// `GET /api/docs/:id/history` — the commits the time slider can stop at,
/// newest first.
///
/// Empty is a real answer: a document nobody has committed yet has no
/// history, and the slider says so rather than erroring.
pub async fn history(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let (root, rel) = match located(&state, &id) {
        Ok(pair) => pair,
        // Not a repository is not an error here: the pane asks on open.
        Err(_) => return Ok(Json(json!({ "repository": false, "commits": [] }))),
    };
    let commits =
        tokio::task::spawn_blocking(move || crate::replay::history(&root, &rel, HISTORY_LIMIT))
            .await
            .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?;
    Ok(Json(json!({ "repository": true, "commits": commits })))
}

/// `GET /api/docs/:id/replay?commit=…&path=…` — the lineage this document
/// had at that commit.
///
/// Weave-only, never executing: a replay of last March must not run last
/// March's commands against today's machine.
pub async fn replay(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Query(q): Query<ReplayParams>,
) -> ApiResult<Json<Value>> {
    let (root, rel_now) = located(&state, &id)?;

    // The name the document had THEN: `--follow` walks renames, and a replay
    // has to read the old name or it reports the document as absent.
    let commit = q.commit.clone();
    let (root2, rel2) = (root.clone(), rel_now.clone());
    let rel_then = tokio::task::spawn_blocking(move || {
        crate::replay::history(&root2, &rel2, HISTORY_LIMIT)
            .into_iter()
            .find(|c| c.sha.starts_with(&commit) || c.short.starts_with(&commit))
            .map(|c| c.path)
            .unwrap_or(rel2)
    })
    .await
    .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?;

    let outcome = crate::replay::replay_at(&root, &rel_then, &q.commit)
        .await
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;

    let run = match outcome {
        crate::replay::Replay::Woven(run) => run,
        // Stated as the boundary it is, in the words the design pins, and as
        // a 200 rather than an error: the slider has reached the edge of what
        // this build can weave, which is a fact about the tool.
        crate::replay::Replay::GrammarBoundary { commit, detail } => {
            return Ok(Json(json!({
                "commit": commit,
                "path": rel_then,
                "grammar_boundary": true,
                "message": crate::replay::grammar_boundary_message(&commit, &rel_then, &detail),
            })));
        }
    };

    let mut outputs: Vec<&String> = run
        .result
        .files
        .iter()
        .filter(|(_, c)| c.as_text().is_some())
        .map(|(p, _)| p)
        .collect();
    outputs.sort();
    let outputs: Vec<String> = outputs.into_iter().cloned().collect();

    let Some(path) = q.path.clone() else {
        return Ok(Json(json!({
            "commit": q.commit,
            "path": rel_then,
            "grammar_boundary": false,
            "source": run.source,
            "outputs": outputs,
        })));
    };

    let content = run
        .result
        .files
        .get(&path)
        .and_then(|c| c.as_text())
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "at {}, this document produced no output named {path:?} — it \
                 produced: {}. An output that did not exist then is a real \
                 answer, not a missing one.",
                q.commit,
                if outputs.is_empty() {
                    "(nothing)".to_string()
                } else {
                    outputs.join(", ")
                }
            ))
        })?
        .to_string();
    let provenance = crate::output_lineage(&run, &path)
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;

    Ok(Json(json!({
        "commit": q.commit,
        "path": rel_then,
        "grammar_boundary": false,
        "source": run.source,
        "outputs": outputs,
        "output": { "path": path, "content": content, "provenance": provenance },
    })))
}

/// `GET /api/git/floor` — which commits on this branch are still drafts.
pub async fn floor(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let computed = tokio::task::spawn_blocking(move || crate::floor::compute(&root))
        .await
        .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?;
    Ok(Json(match computed {
        Some(floor) => json!({ "repository": true, "floor": floor }),
        None => json!({ "repository": false }),
    }))
}

/// `GET /api/git/merge-driver` — whether `.md` merges go through hick.
///
/// Asked at project open, because a clone that never ran `hick init` has
/// neither the driver nor the hook that would report it missing — and git
/// falls back to its line merge silently, which is the whole problem.
pub async fn merge_driver(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    if !state.folder_open {
        let status = crate::merge_driver::MergeDriverStatus {
            repository: false,
            attributes: false,
            configured: false,
            summary: "No folder is open, so there are no merges to route.".into(),
        };
        return Ok(Json(json!({ "status": status, "ok": true })));
    }
    let root = state.index.root().to_path_buf();
    let status = tokio::task::spawn_blocking(move || crate::merge_driver::status(&root))
        .await
        .map_err(|e| ApiError::internal(format!("the git task failed: {e}")))?;
    Ok(Json(json!({ "status": status, "ok": status.ok() })))
}

/// `POST /api/git/merge-driver` — run `hick init` on the open folder.
///
/// The banner that reports a missing driver used to end with "run `hick
/// init` in this repository", which is a command to go and type; the rule in
/// `a-missing-debugger-is-a-button.md` is that a fixable failure is a button.
/// This is the same `run_init` the CLI runs — the hook, `.gitignore`,
/// `.gitattributes`, the driver definition, the editor and agent files — in
/// process, so the desktop app needs no `hick` on its `PATH` to do it. The
/// answer says what changed and re-reads the status the banner asked about.
pub async fn run_init(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    if !state.folder_open {
        return Err(ApiError::bad_request(
            "Open a folder before running hick init. It configures that folder's repository.",
        ));
    }
    let root = state.index.root().to_path_buf();
    let (report, status) = tokio::task::spawn_blocking(move || {
        let report = crate::init::run_init(&root).map_err(|e| format!("{e:#}"))?;
        Ok::<_, String>((report, crate::merge_driver::status(&root)))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the init task failed: {e}")))?
    .map_err(ApiError::unprocessable)?;
    Ok(Json(json!({
        "changed": {
            "hook": report.hook_changed,
            "gitignore": report.gitignore_changed,
            "gitattributes": report.gitattributes_changed,
            "merge_driver": report.merge_driver_changed,
            "agents_md": report.agents_md_changed,
            "mcp_json": report.mcp_json_changed,
        },
        "hook_path": report.hook_path.display().to_string(),
        "status": status,
        "ok": status.ok(),
    })))
}

// ---------------------------------------------------------------------------
// The continuity switch
// ---------------------------------------------------------------------------

/// `GET /api/settings/continuity` — is continuity on for this project?
///
/// Off by default, and the whole of the feature rides this one switch: no
/// ribbon, no journal, no pre-commit repair. It is per-user state rather than
/// a `HICKORY_` variable because it is a preference and not machine
/// configuration, and it lives in `hickory-workspace` rather than the browser
/// because the things that WRITE a journal — the server, the merge driver,
/// the pre-commit hook — cannot read `localStorage`.
pub async fn get_continuity(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let enabled = crate::continuity::enabled(&root);
    let journal = crate::continuity::Journal::at(&root);
    let entries = if enabled { journal.read().len() } else { 0 };
    Ok(Json(json!({
        "enabled": enabled,
        "journal_path": format!("{}/{}",
            crate::continuity::JOURNAL_DIR, crate::continuity::JOURNAL_FILE),
        "entries": entries,
        // Said plainly, because a record that is gitignored answers a
        // different question from one that is committed: only a committed
        // journal is a thing CI could check.
        "committed": journal_is_tracked(&root),
    })))
}

#[derive(Deserialize)]
pub struct ContinuityBody {
    pub enabled: bool,
}

/// `PUT /api/settings/continuity` — turn it on or off.
pub async fn put_continuity(
    State(state): State<LocalState>,
    Json(body): Json<ContinuityBody>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let store = hickory_workspace::WorkspaceStore::for_project(&root)
        .map_err(|e| ApiError::internal(format!("{e:#}")))?;
    store
        .set_continuity(body.enabled)
        .map_err(|e| ApiError::internal(format!("{e:#}")))?;
    Ok(Json(json!({ "enabled": body.enabled })))
}

/// Whether the journal is tracked by git — that is, whether this project
/// deleted the `.gitignore` line `hick init` writes.
fn journal_is_tracked(root: &std::path::Path) -> bool {
    std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "check-ignore",
            "-q",
            "--no-index",
            &format!(
                "{}/{}",
                crate::continuity::JOURNAL_DIR,
                crate::continuity::JOURNAL_FILE
            ),
        ])
        .output()
        // exit 0 = ignored, 1 = not ignored, anything else = could not tell.
        .map(|o| o.status.code() == Some(1))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// The fleet, as something to look at
// ---------------------------------------------------------------------------

/// `GET /api/fleet` — this machine, and the machines paired with it.
///
/// **Nothing is reachable.** The peer channel is not built, so this is a
/// roster and not a presence list, and it says so rather than leaving a reader
/// to assume the machines can see each other.
pub async fn fleet(State(_state): State<LocalState>) -> ApiResult<Json<Value>> {
    let answer = tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let base = hickory_workspace::state_root().map_err(|e| format!("{e:#}"))?;
        let name = hickory_fleet::default_machine_name();
        let identity =
            hickory_fleet::Identity::load_or_create(&base, &name).map_err(|e| format!("{e:#}"))?;
        let fleet = hickory_fleet::Fleet::at(identity.dir());
        // How this machine would be reachable, if it were serving. A bad
        // value is reported rather than silently becoming the default: it
        // decides who is in the path.
        let reach = hickory_peer::Reach::from_env();
        let (posture, reach_note) = match &reach {
            Ok(reach) => (
                match reach {
                    hickory_peer::Reach::Default => "default",
                    hickory_peer::Reach::Own(_) => "own",
                    hickory_peer::Reach::Direct => "direct",
                },
                reach.summary(),
            ),
            Err(e) => ("invalid", format!("{e:#}")),
        };
        Ok(json!({
            "this_machine": {
                "name": identity.name,
                "fingerprint": identity.fingerprint(),
            },
            "machines": fleet.machines(),
            "reach": posture,
            "reach_note": reach_note,
            // Said here, because a pane listing machines is exactly where
            // somebody would assume they are already connected.
            "note": "These are the machines whose keys this one holds. Run \
                     `hick fleet serve` here to make this session reachable by \
                     them, and `hick fleet attach` there to reach it.",
        }))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the fleet task failed: {e}")))?
    .map_err(ApiError::internal)?;
    Ok(Json(answer))
}

/// Open this machine's identity and fleet, or say why not.
fn fleet_of() -> Result<(hickory_fleet::Identity, hickory_fleet::Fleet), ApiError> {
    let base = hickory_workspace::state_root().map_err(|e| ApiError::internal(format!("{e:#}")))?;
    let name = hickory_fleet::default_machine_name();
    let identity = hickory_fleet::Identity::load_or_create(&base, &name)
        .map_err(|e| ApiError::internal(format!("{e:#}")))?;
    let fleet = hickory_fleet::Fleet::at(identity.dir());
    Ok((identity, fleet))
}

#[derive(Deserialize)]
pub struct InviteParams {
    /// Say this machine is a phone: it reads and captures, and can never be
    /// granted `execute` because it has no executor.
    #[serde(default)]
    pub phone: bool,
}

/// `GET /api/fleet/invite` — the line another machine accepts.
///
/// Not reachable over the peer channel: enrolling a machine is something you
/// do standing at one. See `hickory_peer::grants`.
pub async fn fleet_invite(Query(q): Query<InviteParams>) -> ApiResult<Json<Value>> {
    let (identity, _) = fleet_of()?;
    let kind = if q.phone {
        hickory_fleet::Kind::Phone
    } else {
        hickory_fleet::Kind::Desktop
    };
    Ok(Json(json!({
        "invitation": identity.invitation(kind).encode(),
        "fingerprint": identity.fingerprint(),
        // Said with the invitation, because pairing in one direction only is
        // the usual reason a connection is later refused.
        "note": "Paste this on the other machine. Then do the same in the \
                 other direction — a fleet is a MUTUAL list of keys, so each \
                 machine has to accept the other. It carries this machine's \
                 PUBLIC key only.",
    })))
}

#[derive(Deserialize)]
pub struct AcceptBody {
    pub invitation: String,
}

/// `POST /api/fleet/accept` — enrol the machine an invitation names.
pub async fn fleet_accept(Json(body): Json<AcceptBody>) -> ApiResult<Json<Value>> {
    let (_, fleet) = fleet_of()?;
    let invitation = hickory_fleet::Invitation::decode(&body.invitation)
        .map_err(|e| ApiError::bad_request(format!("{e:#}")))?;
    let today = crate::ingest::today().unwrap_or_else(|| "unknown".to_string());
    let machine = fleet
        .add(&invitation, &today)
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "machine": machine })))
}

#[derive(Deserialize)]
pub struct GrantBody {
    pub machine: String,
    /// `view`, `edit` or `execute`.
    pub grant: String,
    pub on: bool,
}

/// `PUT /api/fleet/grant` — give or take one verb for one machine.
pub async fn fleet_grant(Json(body): Json<GrantBody>) -> ApiResult<Json<Value>> {
    let (_, fleet) = fleet_of()?;
    let grant = hickory_fleet::Grant::parse(&body.grant)
        .map_err(|e| ApiError::bad_request(format!("{e:#}")))?;
    let machine = fleet
        .set_grant(&body.machine, grant, body.on)
        // A phone refusing `execute` is not a server error: it is the answer.
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({ "machine": machine })))
}

#[derive(Deserialize)]
pub struct RemoveBody {
    pub machine: String,
}

/// `POST /api/fleet/remove` — revocation, which is deleting a key.
pub async fn fleet_remove(Json(body): Json<RemoveBody>) -> ApiResult<Json<Value>> {
    let (_, fleet) = fleet_of()?;
    let removed = fleet
        .remove(&body.machine)
        .map_err(|e| ApiError::internal(format!("{e:#}")))?;
    Ok(Json(json!({
        "removed": removed,
        "note": "Its key is gone from this machine, and that is the whole of \
                 the revocation — no server holds a session you cannot reach.",
    })))
}

#[derive(Deserialize)]
pub struct PairBody {
    /// Absent: generate a phrase and wait. Present: dial that phrase.
    #[serde(default)]
    pub phrase: Option<String>,
    #[serde(default)]
    pub phone: bool,
}

/// `POST /api/fleet/pair` — enrol by a spoken phrase, both ways at once.
///
/// Blocks for as long as the pairing window, which is the point: the caller is
/// a person who has just read a phrase out loud and is waiting for the other
/// machine to answer.
///
/// Not reachable over the peer channel, like every other fleet write.
pub async fn fleet_pair(Json(body): Json<PairBody>) -> ApiResult<Json<Value>> {
    let (identity, fleet) = fleet_of()?;
    let kind = if body.phone {
        hickory_fleet::Kind::Phone
    } else {
        hickory_fleet::Kind::Desktop
    };
    let reach =
        hickory_peer::Reach::from_env().map_err(|e| ApiError::bad_request(format!("{e:#}")))?;
    let today = crate::ingest::today().unwrap_or_else(|| "unknown".to_string());

    let paired = match &body.phrase {
        Some(raw) => {
            let phrase = hickory_peer::Phrase::parse(raw)
                .map_err(|e| ApiError::bad_request(format!("{e:#}")))?;
            hickory_peer::join(&identity, &fleet, &phrase, kind, &reach, &today).await
        }
        None => {
            let phrase = hickory_peer::Phrase::generate();
            // The phrase has to reach the caller BEFORE the wait, or there is
            // nothing to read out. So hosting is its own route below.
            return Err(ApiError::bad_request(format!(
                "generate the phrase first with GET /api/fleet/phrase, then \
                 POST it back to host it. (This one would have been {phrase}.)"
            )));
        }
    }
    .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;

    Ok(Json(json!({
        "machine": paired.machine,
        "their_fingerprint": paired.machine.fingerprint(),
        "our_fingerprint": paired.ours,
        "note": "Compare those two fingerprints on both screens. That is the \
                 only check that catches somebody who guessed the phrase — the \
                 phrase itself proves nothing about who used it.",
    })))
}

/// `GET /api/fleet/phrase` — a phrase to read out, before hosting it.
pub async fn fleet_phrase() -> ApiResult<Json<Value>> {
    let phrase = hickory_peer::Phrase::generate();
    Ok(Json(json!({
        "phrase": phrase.to_string(),
        "seconds": hickory_peer::pairing::WINDOW.as_secs(),
        "note": "Read this to the other machine and type it there. It works \
                 once and expires — anyone who learns it inside that window can \
                 pair too, which is why it is generated rather than chosen.",
    })))
}

#[derive(Deserialize)]
pub struct HostBody {
    pub phrase: String,
    #[serde(default)]
    pub phone: bool,
}

/// `POST /api/fleet/host` — wait for the other machine to dial this phrase.
pub async fn fleet_host(Json(body): Json<HostBody>) -> ApiResult<Json<Value>> {
    let (identity, fleet) = fleet_of()?;
    let kind = if body.phone {
        hickory_fleet::Kind::Phone
    } else {
        hickory_fleet::Kind::Desktop
    };
    let reach =
        hickory_peer::Reach::from_env().map_err(|e| ApiError::bad_request(format!("{e:#}")))?;
    let today = crate::ingest::today().unwrap_or_else(|| "unknown".to_string());
    let phrase = hickory_peer::Phrase::parse(&body.phrase)
        .map_err(|e| ApiError::bad_request(format!("{e:#}")))?;

    let paired = hickory_peer::host(&identity, &fleet, &phrase, kind, &reach, &today)
        .await
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(json!({
        "machine": paired.machine,
        "their_fingerprint": paired.machine.fingerprint(),
        "our_fingerprint": paired.ours,
    })))
}
