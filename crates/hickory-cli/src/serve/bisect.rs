//! Git owns the search; a detached worktree owns each candidate's execution.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    revision::{git, resolve},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

pub type Bisects = Arc<Mutex<HashMap<String, Session>>>;
#[derive(Serialize, Deserialize)]
pub struct Session {
    control: PathBuf,
    inspections: Vec<PathBuf>,
    templates: Vec<super::representation::Representation>,
    pub worktree: PathBuf,
    good: String,
    bad: String,
    candidate: String,
    outcome: Option<String>,
    said: String,
    history: Vec<Value>,
}

fn clean(s: &Session) -> ApiResult<()> {
    let head = git(&s.worktree, &["rev-parse", "HEAD"])?;
    if head.trim() != s.candidate
        || !git(
            &s.worktree,
            &["status", "--porcelain", "--untracked-files=all"],
        )?
        .trim()
        .is_empty()
    {
        return Err(ApiError::conflict(
            "The candidate has been modified. Preserve your experiment and restore the candidate before recording a verdict or ending bisect.",
        ));
    }
    Ok(())
}

fn snapshot(id: &str, s: &Session) -> ApiResult<Value> {
    let graph: Vec<Value> = git(&s.control, &["log", "--topo-order", "--max-count=200", "--format=%H%x1f%P%x1f%s", &s.bad, &format!("^{}", s.good)])?
        .lines().filter_map(|line| { let mut fields=line.splitn(3, '\u{1f}'); Some(json!({"sha":fields.next()?,"parents":fields.next()?.split_whitespace().collect::<Vec<_>>(),"subject":fields.next()?})) }).collect();
    Ok(
        json!({"id":id,"worktree":s.worktree,"good":s.good,"bad":s.bad,"candidate":s.candidate,"outcome":s.outcome,"said":s.said,"modified":clean(s).is_err(),"history":s.history,"inspections":s.inspections,"graph":graph,"commits":git(&s.control, &["bisect", "visualize", "--format=%H %s", "--no-patch"]).unwrap_or_default()}),
    )
}

fn advance(s: &mut Session, said: String) -> ApiResult<()> {
    s.said = said;
    // Git's result is grounded in its remaining candidates, never guessed from a UI count.
    let remaining = git(
        &s.control,
        &[
            "rev-list",
            "refs/bisect/bad",
            "--not",
            "--glob=refs/bisect/good-*",
        ],
    )?;
    if remaining.lines().count() == 1 {
        s.candidate = remaining.trim().into();
        s.outcome = Some("first_bad".into());
    } else if s.said.contains("only 'skip'ped commits left")
        || s.said.contains("first bad commit could be any")
    {
        s.outcome = Some("ambiguous".into());
    } else {
        s.candidate = git(&s.control, &["rev-parse", "BISECT_HEAD"])?
            .trim()
            .into();
    }
    inspect(s)
}

fn inspect(s: &mut Session) -> ApiResult<()> {
    let worktree = s
        .control
        .parent()
        .unwrap()
        .join(format!("review-{:016x}", super::rand_id()));
    git(
        &s.control,
        &[
            "worktree",
            "add",
            "--detach",
            &worktree.to_string_lossy(),
            &s.candidate,
        ],
    )?;
    let local = hickory_workspace::WorkspaceStore::for_project(&worktree)?
        .dir()
        .join("literate-views");
    if !s.templates.is_empty() {
        std::fs::create_dir_all(&local).map_err(anyhow::Error::from)?;
        for view in &s.templates {
            super::store::write_atomic(
                &local.join(format!("{}.json", view.id)),
                &serde_json::to_vec(view).map_err(anyhow::Error::from)?,
            )?;
            super::store::write_atomic(
                &local.join(format!("{}.md", view.id)),
                view.source.as_bytes(),
            )?;
        }
    }
    s.worktree = worktree.clone();
    s.inspections.push(worktree);
    persist(s)
}

fn persist(s: &Session) -> ApiResult<()> {
    super::store::write_atomic(
        &s.control.with_extension("json"),
        &serde_json::to_vec(s).map_err(anyhow::Error::from)?,
    )?;
    Ok(())
}

#[derive(Deserialize)]
pub struct Start {
    pub good: String,
    pub bad: String,
}
pub async fn start(
    State(state): State<LocalState>,
    Json(body): Json<Start>,
) -> ApiResult<Json<Value>> {
    let good = resolve(state.index.root(), &body.good)?;
    let bad = resolve(state.index.root(), &body.bad)?;
    if good == "INDEX" || bad == "INDEX" {
        return Err(ApiError::bad_request(
            "Bisect requires committed revisions.",
        ));
    }
    if good == bad {
        return Err(ApiError::bad_request(
            "Good and bad must name different commits.",
        ));
    }
    let ancestor = std::process::Command::new("git")
        .arg("-C")
        .arg(state.index.root())
        .args(["merge-base", "--is-ancestor", &good, &bad])
        .status()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !ancestor.success() {
        return Err(ApiError::bad_request(
            "Choose a known-good ancestor of the known-bad commit.",
        ));
    }
    let id = format!("{:016x}", super::rand_id());
    let dir = hickory_workspace::WorkspaceStore::for_project(state.index.root())?
        .dir()
        .join("bisect");
    std::fs::create_dir_all(&dir).map_err(|e| ApiError::internal(e.to_string()))?;
    let worktree = dir.join(&id);
    git(
        state.index.root(),
        &[
            "worktree",
            "add",
            "--detach",
            &worktree.to_string_lossy(),
            &bad,
        ],
    )?;
    let mut session = Session {
        control: worktree.clone(),
        inspections: vec![],
        templates: state
            .representations
            .lock()
            .await
            .values()
            .filter(|v| matches!(v.backing, super::representation::Backing::Files { .. }))
            .cloned()
            .collect(),
        worktree,
        good: good.clone(),
        bad: bad.clone(),
        candidate: bad.clone(),
        outcome: None,
        said: String::new(),
        history: vec![],
    };
    let started = git(
        &session.control,
        &["bisect", "start", "--no-checkout", &bad, &good],
    );
    let result = match started {
        Ok(said) => advance(&mut session, said),
        Err(e) => Err(e),
    };
    if let Err(e) = result {
        let _ = git(
            state.index.root(),
            &["worktree", "remove", &session.worktree.to_string_lossy()],
        );
        return Err(e);
    }
    let answer = snapshot(&id, &session)?;
    state.bisects.lock().await.insert(id, session);
    Ok(Json(answer))
}

pub async fn list(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let dir = hickory_workspace::WorkspaceStore::for_project(state.index.root())?
        .dir()
        .join("bisect");
    if dir.exists() {
        let mut sessions = state.bisects.lock().await;
        for entry in std::fs::read_dir(dir).map_err(anyhow::Error::from)? {
            let path = entry.map_err(anyhow::Error::from)?.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let s: Session =
                serde_json::from_slice(&std::fs::read(&path).map_err(anyhow::Error::from)?)
                    .map_err(anyhow::Error::from)?;
            if s.control.exists() {
                sessions
                    .entry(s.control.file_name().unwrap().to_string_lossy().to_string())
                    .or_insert(s);
            }
        }
    }
    let sessions = state.bisects.lock().await;
    Ok(Json(
        json!({"sessions":sessions.iter().map(|(id,s)| snapshot(id,s)).collect::<ApiResult<Vec<_>>>()?}),
    ))
}
#[derive(Deserialize)]
pub struct Verdict {
    pub candidate: String,
    pub verdict: String,
}
pub async fn mark(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Verdict>,
) -> ApiResult<Json<Value>> {
    let mut sessions = state.bisects.lock().await;
    let s = sessions
        .get_mut(&id)
        .ok_or_else(|| ApiError::not_found("Start a new bisect; this session no longer exists."))?;
    if !["good", "bad", "skip"].contains(&body.verdict.as_str()) {
        return Err(ApiError::bad_request("Choose Good, Bad, or Skip."));
    }
    if s.outcome.is_some() || body.candidate != s.candidate {
        return Err(ApiError::conflict(
            "The candidate changed or the search finished. Refresh bisect before classifying.",
        ));
    }
    clean(s)?;
    // Ambiguous skip results exit 2; preserve Git's evidence instead of treating them as a failed search.
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&s.control)
        .args(["bisect", &body.verdict, &s.candidate])
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !output.status.success()
        && !said.contains("first bad commit could be any")
        && !said.contains("only 'skip'ped commits left")
    {
        return Err(ApiError::unprocessable(said));
    }
    s.history
        .push(json!({"candidate":s.candidate,"verdict":body.verdict}));
    advance(s, said)?;
    Ok(Json(snapshot(&id, s)?))
}

pub async fn open(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let sessions = state.bisects.lock().await;
    let s = sessions
        .get(&id)
        .ok_or_else(|| ApiError::not_found("Start bisect again."))?;
    state.open_folder(&s.worktree, super::OpenWhere::NewWindow)?;
    Ok(Json(json!({"ok":true})))
}

pub async fn restore(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let mut sessions = state.bisects.lock().await;
    let s = sessions
        .get_mut(&id)
        .ok_or_else(|| ApiError::not_found("Start bisect again."))?;
    if git(&s.worktree, &["rev-parse", "HEAD"])?.trim() != s.candidate {
        return Err(ApiError::conflict(
            "Candidate HEAD changed. Return it to the recorded candidate before restoring.",
        ));
    }
    if !git(&s.worktree, &["ls-files", "--others", "--exclude-standard"])?
        .trim()
        .is_empty()
    {
        return Err(ApiError::conflict(
            "There are untracked experiment files. Move them outside the candidate worktree before restoring.",
        ));
    }
    let patch = git(&s.worktree, &["diff", "HEAD", "--binary"])?;
    let saved = s
        .worktree
        .parent()
        .unwrap()
        .join(format!("{id}-{:016x}.patch", super::rand_id()));
    super::store::write_atomic(&saved, patch.as_bytes())?;
    // Keep the edited inspection intact; a fresh worktree is the restored candidate.
    inspect(s)?;
    Ok(Json(json!({"patch":saved,"session":snapshot(&id,s)?})))
}

pub async fn finish(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let mut sessions = state.bisects.lock().await;
    let s = sessions
        .get(&id)
        .ok_or_else(|| ApiError::not_found("This bisect has already ended."))?;
    clean(s)?;
    git(&s.control, &["bisect", "reset"])?;
    git(
        state.index.root(),
        &["worktree", "remove", &s.control.to_string_lossy()],
    )?;
    let retained = s.inspections.clone();
    std::fs::remove_file(s.control.with_extension("json")).map_err(anyhow::Error::from)?;
    sessions.remove(&id);
    Ok(Json(json!({"ok":true,"retained_inspections":retained})))
}
