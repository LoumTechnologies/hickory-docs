//! Same retained DAP owner and ledger gate, with a trusted two-stop question plan.
//! The host supplies prior visible verdict/source metadata; target values never leave DAP.
use super::private_predictions::{Bound, SealRequest};
use anyhow::{Result, ensure};
use hick_literate::prediction::{
    ManagedOutput, SealedRequest, debugger::interactive::RetainedDebugger,
};
use hick_observe::prediction::{Scenario, Seal, Target};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub initial: Scenario,
    pub next: Scenario,
}
#[derive(Default)]
pub(super) struct Controller {
    live: Option<RetainedDebugger>,
    pending: Option<SealedRequest>,
    prior_visible: Value,
    last: Option<ManagedOutput>,
    complete: bool,
    run: Option<String>,
    expired: bool,
}
impl Controller {
    pub(super) async fn reset(&mut self) {
        if let Some(live) = &mut self.live {
            live.shutdown().await;
        }
        *self = Self::default();
    }
    pub(super) fn status(&self) -> Value {
        json!({"phase":if self.expired {"expired"}else if self.complete {"complete"}else if self.pending.is_some(){"sealed"}else if self.live.is_some(){"paused_needs_prediction"}else{"not_started"},"last":self.last,"actual_values":"withheld"})
    }
}
impl Plan {
    pub(super) fn validate(&self, question: &Scenario, target: &Target) -> Result<()> {
        ensure!(
            matches!(target,Target::Combined {debugger,..} if matches!(debugger.as_ref(),Target::Debugger {hit:2,..})),
            "Retained coach requires a validated combined second-hit question"
        );
        ensure!(
            self.initial.inputs == question.inputs && self.next.inputs == question.inputs,
            "Retained questions must preserve host inputs"
        );
        let keys = [self.initial.key()?, self.next.key()?, question.key()?];
        ensure!(
            keys[0] != keys[1] && keys[0] != keys[2] && keys[1] != keys[2],
            "Each host-selected semantic question needs a distinct scenario family"
        );
        Ok(())
    }
}
impl Bound {
    pub(super) async fn debugger_tool(
        self: &std::sync::Arc<Self>,
        name: &str,
        args: Value,
    ) -> Result<Value> {
        let plan = self.retained_plan.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No trusted retained-stop plan is bound to this practice")
        })?;
        let mut session = self.session.lock().await;
        ensure!(!self.is_retired(), "Private session ended");
        session
            .refresh_source()
            .await
            .map_err(|_| anyhow::anyhow!("Captured source changed outside the session"))?;
        let mut identity = self.identity.read().unwrap().clone();
        identity.source = format!("{:x}", Sha256::digest(session.approved_source()));
        self.inputs
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Host input anchors required"))?
            .validate(&identity, session.approved_source())?;
        let runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Configured runtime profile required"))?;
        ensure!(
            runtime.capture().await? == identity.environment,
            "Retained runtime differs from host identity"
        );
        let whole_target = self
            .source_question
            .rebind(&identity, session.approved_source())?;
        let mut controller = self.retained.lock().await;
        if name == "seal_debugger_prediction" {
            let body: SealRequest = serde_json::from_value(args).map_err(|_| {
                anyhow::anyhow!("Provide expected_source_hash, predicted predicate and rationale")
            })?;
            ensure!(
                body.expected_source_hash == identity.source,
                "Prediction source snapshot is stale"
            );
            ensure!(
                controller.pending.is_none() && !controller.complete,
                "Execute the pending seal, or edit source before starting another retained sequence"
            );
            let Target::Combined { debugger, .. } = &whole_target else {
                unreachable!("validated plan")
            };
            let mut target = debugger.as_ref().clone();
            let initial = controller.live.is_none();
            if initial {
                if let Target::Debugger { hit, .. } = &mut target {
                    *hit = 1;
                }
            }
            let seal = Seal {
                scenario: if initial {
                    plan.initial.clone()
                } else {
                    plan.next.clone()
                },
                identity: identity.clone(),
                target: target.clone(),
                predicted: body.predicted,
                rationale: body.rationale,
            };
            let request_id = format!("retained:{}", crate::engine::window_token());
            let request = if initial {
                let receipt = self.ledger.seal(
                    self.ledger.history(None)?.generation,
                    seal,
                    self.actor.clone(),
                )?;
                SealedRequest {
                    seal: receipt.id,
                    request_id,
                    actor: self.actor.clone(),
                    identity,
                    target,
                }
            } else {
                let visible = json!({"scope":"Recorded Hickory source, questions, comparison and actor-scoped ledger state before this future seal. Supervisor request/delivery context is retained separately; external client/provider context is unestablished.","source":{"sha256":identity.source,"bytes":session.approved_source()},"question":self.question,"retained_plan":plan,"target":whole_target,"prior_comparison":controller.last,"prior_history":self.ledger.predictor_history(None,&self.actor,&self.scenario_keys()?)?});
                let request = controller.live.as_ref().unwrap().seal_next(
                    seal,
                    request_id,
                    visible.clone(),
                )?;
                controller.prior_visible = visible;
                request
            };
            let value = json!({"sealed":request.seal,"source_hash":request.identity.source,"executed":false,"phase":if initial {"before_launch"}else{"before_continue"},"current_certification":false});
            controller.expired = false;
            controller.pending = Some(request);
            return Ok(value);
        }
        ensure!(
            name == "continue_debugger_prediction"
                && args.as_object().is_some_and(|args| args.is_empty()),
            "Continue takes an empty argument object"
        );
        let Some(request) = controller.pending.take() else {
            if let Some(last) = &controller.last {
                let mut last = last.clone();
                last.dispatched = false;
                return Ok(serde_json::to_value(last)?);
            }
            anyhow::bail!("Seal a debugger prediction before launch or Continue");
        };
        let source = session.approved_source();
        ensure!(
            request.identity == identity,
            "Retained source or environment changed; seal again after trusted recovery"
        );
        let visible = controller.prior_visible.clone();
        let result = if let Some(live) = &mut controller.live {
            let result = live.resume(&request, &visible).await;
            live.shutdown().await;
            controller.complete = true;
            controller.live = None;
            result?
        } else {
            let whole = SealedRequest {
                target: whole_target,
                ..request.clone()
            };
            let (live, result) = RetainedDebugger::start_in_combined_source(
                &self.ledger,
                &request,
                &whole,
                source,
                self.path.parent().unwrap(),
                Some(runtime),
            )
            .await?;
            controller.live = live;
            controller.run = Some(request.seal.clone());
            if controller.live.is_some() {
                let owner = std::sync::Arc::downgrade(self);
                let run = request.seal.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(120)).await;
                    if let Some(bound) = owner.upgrade() {
                        bound.expire_retained(&run).await;
                    }
                });
            }
            controller.complete = controller.live.is_none();
            result
        };
        controller.prior_visible = json!({"source_hash":identity.source,"comparison":result,"requirement":plan.initial.requirement_reference,"target_values":"withheld"});
        controller.last = Some(result.clone());
        Ok(serde_json::to_value(result)?)
    }
}

impl Bound {
    pub(super) fn scenario_keys(&self) -> Result<Vec<String>> {
        let mut keys = vec![self.question.key()?];
        if let Some(plan) = &self.retained_plan {
            keys.extend([plan.initial.key()?, plan.next.key()?]);
        }
        Ok(keys)
    }
    pub(super) fn retained_ready(
        &self,
        identity: &hick_observe::prediction::Identity,
    ) -> Result<bool> {
        let Some(plan) = &self.retained_plan else {
            return Ok(true);
        };
        let keys = [plan.initial.key()?, plan.next.key()?];
        let history = self
            .ledger
            .predictor_history(Some(identity), &self.actor, &keys)?;
        Ok(keys.iter().all(|key| {
            history
                .attempts
                .iter()
                .rev()
                .find(|attempt| &attempt.scenario == key)
                .is_some_and(|attempt| {
                    matches!(
                        attempt.classification,
                        hick_observe::prediction::Classification::FirstAttemptVerified
                            | hick_observe::prediction::Classification::InformedPredictionVerified
                            | hick_observe::prediction::Classification::VerifiedAfterCorrection
                            | hick_observe::prediction::Classification::VerifiedAfterPriorAttempt
                    )
                })
        }))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Command {
    name: String,
    arguments: Value,
}
pub(super) async fn command(
    axum::extract::State(state): axum::extract::State<super::LocalState>,
    axum::Json(body): axum::Json<Command>,
) -> super::api::ApiResult<axum::Json<Value>> {
    let bound = state
        .private_prediction
        .as_ref()
        .ok_or_else(|| super::api::ApiError::unprocessable("No private prediction session"))?;
    if !["seal_debugger_prediction", "continue_debugger_prediction"].contains(&body.name.as_str()) {
        return Err(super::api::ApiError::unprocessable(
            "Unsupported retained debugger command",
        ));
    }
    Ok(axum::Json(
        bound
            .mcp_tool(
                state.executor,
                &json!({"name":body.name,"arguments":body.arguments}),
            )
            .await,
    ))
}

impl Bound {
    pub(super) async fn expire_retained(&self, run: &str) {
        let _source = self.session.lock().await;
        let mut controller = self.retained.lock().await;
        if controller.run.as_deref() == Some(run) && controller.live.is_some() {
            controller.reset().await;
            controller.expired = true;
        }
    }
    pub(crate) async fn shutdown_retained(&self) {
        self.retained.lock().await.reset().await;
    }
    pub(crate) fn investigation_busy(&self) -> bool {
        self.agent.try_lock().map_or(true, |run| {
            super::private_prediction_agent::running(run.as_ref())
        })
    }
}
