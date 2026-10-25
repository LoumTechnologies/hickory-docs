<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/fleet_pairings.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `fc4d5a4f5d8bc0ebfbd17256a8edc1915f27d59f49c8e74c99d3808922abe2a5`.

<book:entry path="crates/hickory-cli/src/serve/fleet_pairings.rs">
//! Window-owned pairing requests, with host-observed results and actual cancellation.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use hickory_peer::pairing::{PairCancelled, PairPhase};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Host,
    Dial,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Start {
    pub window: String,
    pub request_id: String,
    pub mode: Mode,
    pub phrase: String,
    #[serde(default)]
    pub phone: bool,
}
#[derive(Deserialize)]
pub struct Owner {
    pub window: String,
}
struct Job {
    id: String,
    request: Start,
    outcome: Mutex<Value>,
    cancel: watch::Sender<bool>,
    phase: watch::Receiver<PairPhase>,
    retired: AtomicBool,
}
impl Job {
    fn finished(&self) -> bool {
        !matches!(
            self.outcome.lock().unwrap()["status"].as_str(),
            Some("queued" | "running")
        )
    }
    fn snapshot(&self) -> Value {
        let mut result = self.outcome.lock().unwrap().clone();
        let object = result.as_object_mut().unwrap();
        object.insert("id".into(), json!(self.id));
        object.insert("request_id".into(), json!(self.request.request_id));
        object.insert("mode".into(), json!(self.request.mode));
        object.insert("phase".into(), json!(*self.phase.borrow()));
        object.insert("cancel_requested".into(), json!(*self.cancel.borrow()));
        object.insert("rollback".into(), json!(false));
        result
    }
}
#[derive(Default)]
pub struct Pairings {
    jobs: Mutex<BTreeMap<String, Arc<Job>>>,
}
impl Pairings {
    fn start<F, Fut>(self: &Arc<Self>, request: Start, run: F) -> ApiResult<(Arc<Job>, bool)>
    where
        F: FnOnce(watch::Receiver<bool>, watch::Sender<PairPhase>) -> Fut + Send + 'static,
        Fut: Future<Output = anyhow::Result<Value>> + Send + 'static,
    {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(existing) = jobs.values().find(|job| {
            job.request.window == request.window && job.request.request_id == request.request_id
        }) {
            if existing.request != request {
                return Err(ApiError::conflict(
                    "Pairing request ID reused with different arguments",
                ));
            }
            return Ok((existing.clone(), false));
        }
        if jobs
            .values()
            .any(|job| job.request.window == request.window && !job.finished())
        {
            return Err(ApiError::conflict(
                "This window already has a pairing request; cancel or inspect it before starting another",
            ));
        }
        if jobs.len() >= 64
            || jobs
                .values()
                .filter(|job| job.request.window == request.window)
                .count()
                >= 8
        {
            return Err(ApiError::unavailable(
                "Pairing request limit reached; inspect and forget finished requests first",
            ));
        }
        let id = format!("{:016x}{:016x}", super::rand_id(), super::rand_id());
        let (cancel, receiver) = watch::channel(false);
        let (progress, phase) = watch::channel(PairPhase::Preparing);
        let job = Arc::new(Job {
            id: id.clone(),
            request,
            cancel,
            phase,
            retired: AtomicBool::new(false),
            outcome: Mutex::new(json!({"status":"queued"})),
        });
        jobs.insert(id, job.clone());
        drop(jobs);
        let running = job.clone();
        let work = tokio::spawn(async move {
            *running.outcome.lock().unwrap() = json!({"status":"running"});
            run(receiver, progress).await
        });
        let completed = job.clone();
        let retained = Arc::downgrade(self);
        tokio::spawn(async move {
            let outcome = match work.await {
                Ok(Ok(result)) => json!({"status":"paired","result":result}),
                Ok(Err(error)) if error.downcast_ref::<PairCancelled>().is_some() => {
                    json!({"status":"cancelled","error":error.to_string(),"local_enrollment":false,"remote_effects":"Inspect the remote roster; invitations already sent cannot be retracted"})
                }
                Ok(Err(error)) => {
                    json!({"status":"failed","error":format!("{error:#}"),"effects":"Inspect both rosters before retrying"})
                }
                Err(error) => {
                    json!({"status":"failed","error":format!("Pairing task ended without an outcome: {error}"),"effects":"Unknown; inspect both rosters"})
                }
            };
            *completed.outcome.lock().unwrap() = outcome;
            if let Some(retained) = retained.upgrade() {
                let mut jobs = retained.jobs.lock().unwrap();
                if completed.retired.load(Ordering::Acquire) {
                    jobs.remove(&completed.id);
                }
            }
        });
        Ok((job, true))
    }
    fn owned(&self, owner: &str, id: &str) -> ApiResult<Arc<Job>> {
        self.jobs
            .lock()
            .unwrap()
            .get(id)
            .filter(|job| job.request.window == owner)
            .cloned()
            .ok_or_else(|| {
                ApiError::not_found("Pairing request is absent from this originating window")
            })
    }
    pub(super) fn close_window(&self, owner: &str) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|_, job| {
            if job.request.window != owner {
                return true;
            }
            job.retired.store(true, Ordering::Release);
            if job.finished() {
                return false;
            }
            job.cancel.send_replace(true);
            true // Keep active work until its actual outcome, then release its slot.
        });
    }
}

pub fn routes() -> Router<LocalState> {
    Router::new()
        .route("/v1/fleet/pairings", post(start).get(list))
        .route("/v1/fleet/pairings/{id}", get(status).delete(forget))
        .route("/v1/fleet/pairings/{id}/cancel", post(cancel))
}
fn window(state: &LocalState, owner: &str) -> ApiResult<()> {
    state
        .ui_actions
        .ensure_window(owner)
        .map_err(|error| ApiError::unavailable(error.to_string()))
}
async fn start(
    State(state): State<LocalState>,
    Json(mut body): Json<Start>,
) -> ApiResult<Json<Value>> {
    window(&state, &body.window)?;
    if body.request_id.is_empty()
        || body.request_id.len() > 128
        || !body
            .request_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
    {
        return Err(ApiError::bad_request(
            "A bounded pairing request ID is required",
        ));
    }
    if body.phrase.len() > 1024 {
        return Err(ApiError::bad_request("Pairing phrase is too long"));
    }
    let phrase = hickory_peer::Phrase::parse(&body.phrase)
        .map_err(|error| ApiError::bad_request(error.to_string()))?;
    body.phrase = phrase.as_str().to_owned();
    let captured = body.clone();
    let hub = state.ui_actions.clone();
    let (job, started) = state.fleet_pairings.start(body, move |cancel, progress| async move {
        // Close/start races cannot enter pairing after their originating window vanished.
        hub.ensure_window(&captured.window)?;
        let (identity, fleet) = super::history::fleet_of().map_err(|error| anyhow::anyhow!(error.detail().to_owned()))?;
        let kind = if captured.phone { hickory_fleet::Kind::Phone } else { hickory_fleet::Kind::Desktop };
        let reach = hickory_peer::Reach::from_env()?;
        let today = crate::ingest::today().unwrap_or_else(|| "unknown".into());
        let paired = match captured.mode {
            Mode::Host => hickory_peer::pairing::host_cancellable(&identity, &fleet, &phrase, kind, &reach, &today, cancel, Some(progress)).await?,
            Mode::Dial => hickory_peer::pairing::join_cancellable(&identity, &fleet, &phrase, kind, &reach, &today, cancel, Some(progress)).await?,
        };
        Ok(json!({"machine":paired.machine,"their_fingerprint":paired.machine.fingerprint(),"our_fingerprint":paired.ours,
            "note":"Compare the fingerprints on both machines; pairing is not independent identity verification"}))
    })?;
    // If close raced with insertion, cancel this newly retained request as well.
    if state.ui_actions.ensure_window(&job.request.window).is_err() {
        job.cancel.send_replace(true);
    }
    Ok(Json(json!({"started":started,"job":job.snapshot()})))
}
async fn list(
    State(state): State<LocalState>,
    Query(owner): Query<Owner>,
) -> ApiResult<Json<Value>> {
    window(&state, &owner.window)?;
    let jobs = state
        .fleet_pairings
        .jobs
        .lock()
        .unwrap()
        .values()
        .filter(|job| job.request.window == owner.window)
        .map(|job| job.snapshot())
        .collect::<Vec<_>>();
    Ok(Json(
        json!({"jobs":jobs,"scope":"In-memory requests for this originating window; restart invalidates IDs"}),
    ))
}
async fn status(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Query(owner): Query<Owner>,
) -> ApiResult<Json<Value>> {
    window(&state, &owner.window)?;
    Ok(Json(
        state.fleet_pairings.owned(&owner.window, &id)?.snapshot(),
    ))
}
async fn cancel(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Query(owner): Query<Owner>,
) -> ApiResult<Json<Value>> {
    window(&state, &owner.window)?;
    let job = state.fleet_pairings.owned(&owner.window, &id)?;
    if !job.finished() {
        job.cancel.send_replace(true);
    }
    Ok(Json(
        json!({"cancellation_requested":true,"job":job.snapshot(),"scope":"Inspect the actual result; completed enrollment is not rolled back"}),
    ))
}
async fn forget(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Query(owner): Query<Owner>,
) -> ApiResult<Json<Value>> {
    window(&state, &owner.window)?;
    let job = state.fleet_pairings.owned(&owner.window, &id)?;
    if !job.finished() {
        return Err(ApiError::conflict(
            "Cancel and inspect the pending request before forgetting it",
        ));
    }
    state.fleet_pairings.jobs.lock().unwrap().remove(&id);
    Ok(Json(json!({"forgotten":id,"rollback":false})))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(window: &str, id: &str) -> Start {
        Start {
            window: window.into(),
            request_id: id.into(),
            mode: Mode::Host,
            phrase: "fixture".into(),
            phone: false,
        }
    }
    async fn finished(job: &Job) -> Value {
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while !job.finished() {
                tokio::task::yield_now().await;
            }
            job.snapshot()
        })
        .await
        .expect("fixture job must settle")
    }
    #[tokio::test]
    async fn retries_share_one_job_and_cannot_change_arguments_or_owner() {
        let jobs = Arc::new(Pairings::default());
        let (job, started) = jobs
            .start(request("one", "retry"), |mut cancel, _| async move {
                while !*cancel.borrow_and_update() {
                    cancel.changed().await?;
                }
                Err(PairCancelled.into())
            })
            .unwrap();
        assert!(started);
        let (retry, started) = jobs
            .start(request("one", "retry"), |_, _| async {
                panic!("retry must not run")
            })
            .unwrap();
        assert!(!started);
        assert!(Arc::ptr_eq(&job, &retry));
        let mut changed = request("one", "retry");
        changed.phone = true;
        assert!(
            jobs.start(changed, |_, _| async { Ok(Value::Null) })
                .is_err()
        );
        assert!(
            jobs.start(request("one", "second"), |_, _| async { Ok(Value::Null) })
                .is_err()
        );
        assert!(jobs.owned("two", &job.id).is_err());
        job.cancel.send_replace(true);
        let result = finished(&job).await;
        assert_eq!(result["status"], "cancelled");
        assert_eq!(result["local_enrollment"], false);
        assert_eq!(result["rollback"], false);
    }
    #[tokio::test]
    async fn cancellation_does_not_relabel_completed_enrollment() {
        let jobs = Arc::new(Pairings::default());
        let (job, _) = jobs
            .start(request("one", "completed"), |mut cancel, _| async move {
                while !*cancel.borrow_and_update() {
                    cancel.changed().await?;
                }
                // A fixture for a worker that completed enrollment before observing cancel.
                Ok(json!({"enrolled":true}))
            })
            .unwrap();
        job.cancel.send_replace(true);
        let result = finished(&job).await;
        assert_eq!(result["status"], "paired");
        assert_eq!(result["result"]["enrolled"], true);
        assert_eq!(result["cancel_requested"], true);
    }
    #[tokio::test]
    async fn closed_window_cancels_work_and_releases_retained_slots() {
        let jobs = Arc::new(Pairings::default());
        let (job, _) = jobs
            .start(request("one", "close"), |mut cancel, _| async move {
                while !*cancel.borrow_and_update() {
                    cancel.changed().await?;
                }
                Err(PairCancelled.into())
            })
            .unwrap();
        jobs.close_window("two");
        assert!(!*job.cancel.borrow());
        jobs.close_window("one");
        assert_eq!(finished(&job).await["status"], "cancelled");
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while jobs.owned("one", &job.id).is_ok() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let (job, _) = jobs
            .start(request("done", "close"), |_, _| async { Ok(Value::Null) })
            .unwrap();
        finished(&job).await;
        jobs.close_window("done");
        assert!(jobs.owned("done", &job.id).is_err());
    }
    #[tokio::test]
    async fn finished_requests_are_bounded_and_failures_remain_inspectable() {
        let jobs = Arc::new(Pairings::default());
        for index in 0..8 {
            let (job, _) = jobs
                .start(request("one", &index.to_string()), |_, _| async {
                    anyhow::bail!("fixture failure")
                })
                .unwrap();
            let result = finished(&job).await;
            assert_eq!(result["status"], "failed");
            assert!(
                result["error"]
                    .as_str()
                    .unwrap()
                    .contains("fixture failure")
            );
            assert!(jobs.owned("one", &job.id).is_ok());
        }
        assert!(
            jobs.start(request("one", "overflow"), |_, _| async { Ok(Value::Null) })
                .is_err()
        );
    }
}
</book:entry>

</book:doc>
