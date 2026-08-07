//! Server-side pipeline execution: per-run temp dirs seeded from the project
//! git checkout, live transcript streaming over the run WS channel, and run
//! persistence for `GET /api/runs/:id` + render overlay.

use std::sync::Arc;
use std::time::Instant;

use axum::http::StatusCode;
use chrono::Utc;
use hick_literate::render::Block;
use hick_literate::{ExecEventHook, PipelineConfig, run_pipeline_live};
use hickory_executor::ExecTranscriptEntry;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::User;
use crate::error::{ApiError, ApiResult};
use crate::executor::build_executor;
use crate::routes::docs::DocRow;
use crate::{AppState, plans};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    Run,
    Check,
}

impl RunKind {
    fn as_str(self) -> &'static str {
        match self {
            RunKind::Run => "run",
            RunKind::Check => "check",
        }
    }
}

/// Current month key for metering ('YYYY-MM').
fn month_key() -> String {
    Utc::now().format("%Y-%m").to_string()
}

/// Enforce the execution-minutes quota (hard stop + upgrade prompt).
pub async fn check_exec_quota(state: &AppState, user: &User) -> ApiResult<()> {
    let ents = plans::resolve(
        &state.catalog,
        &user.plan_key,
        user.price_key.as_deref(),
        &user.billing_status,
    );
    let used_ms: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(wall_ms), 0)::BIGINT FROM usage_ms WHERE user_id = $1 AND month = $2",
    )
    .bind(user.id)
    .bind(month_key())
    .fetch_one(&state.db)
    .await?;
    let limit_ms = ents.exec_minutes_month as i64 * 60_000;
    if used_ms >= limit_ms {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            format!(
                "execution minutes exhausted for this month ({} min on the {} plan) — upgrade your plan to keep running documents",
                ents.exec_minutes_month, ents.plan_key
            ),
        ));
    }
    Ok(())
}

pub(crate) async fn record_usage(state: &AppState, user_id: Uuid, wall_ms: i64) {
    let res = sqlx::query(
        "INSERT INTO usage_ms (user_id, month, wall_ms) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, month) DO UPDATE SET wall_ms = usage_ms.wall_ms + EXCLUDED.wall_ms",
    )
    .bind(user_id)
    .bind(month_key())
    .bind(wall_ms)
    .execute(&state.db)
    .await;
    if let Err(e) = res {
        log::error!("recording usage for {user_id} failed: {e}");
    }
}

/// Start a run/check for a doc. Returns the run id (202 semantics: the
/// pipeline executes in a background task, streaming to the doc's WS room).
pub async fn start_run(
    state: &AppState,
    doc: DocRow,
    user: &User,
    kind: RunKind,
) -> ApiResult<Uuid> {
    check_exec_quota(state, user).await?;

    // Fail fast on unparseable docs so the client gets a 422, not a
    // queued run that instantly fails.
    hick_lang::parse(&doc.source)
        .map_err(|e| ApiError::unprocessable(format!("parse error: {e}")))?;

    let run_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO runs (id, doc_id, user_id, kind, status) VALUES ($1, $2, $3, $4, 'queued')",
    )
    .bind(run_id)
    .bind(doc.id)
    .bind(user.id)
    .bind(kind.as_str())
    .execute(&state.db)
    .await?;

    state.analytics.capture(
        &user.id.to_string(),
        if kind == RunKind::Check {
            "doc_check"
        } else {
            "doc_run"
        },
        json!({ "doc_id": doc.id, "project_id": doc.project_id, "run_id": run_id }),
    );

    let state = state.clone();
    let user_id = user.id;
    tokio::spawn(async move {
        let started = Instant::now();
        let status = match execute_run(&state, run_id, &doc, kind).await {
            Ok(ok) => {
                if ok {
                    "ok"
                } else {
                    "failed"
                }
            }
            Err(e) => {
                log::warn!("run {run_id} errored: {e:#}");
                let _ = sqlx::query("UPDATE runs SET error = $1 WHERE id = $2")
                    .bind(format!("{e:#}"))
                    .bind(run_id)
                    .execute(&state.db)
                    .await;
                "failed"
            }
        };
        let wall_ms = started.elapsed().as_millis() as i64;
        let _ = sqlx::query(
            "UPDATE runs SET status = $1, finished_at = now(), wall_ms = $2 WHERE id = $3",
        )
        .bind(status)
        .bind(wall_ms)
        .bind(run_id)
        .execute(&state.db)
        .await;
        record_usage(&state, user_id, wall_ms).await;
        state
            .rooms
            .publish_run_event(doc.id, &json!({ "run_id": run_id, "status": status }))
            .await;
    });

    Ok(run_id)
}

/// Execute the pipeline for one run. Returns Ok(true) on success,
/// Ok(false) when a check found failures.
async fn execute_run(
    state: &AppState,
    run_id: Uuid,
    doc: &DocRow,
    kind: RunKind,
) -> anyhow::Result<bool> {
    sqlx::query("UPDATE runs SET status = 'running' WHERE id = $1")
        .bind(run_id)
        .execute(&state.db)
        .await?;

    // Per-run temp dir seeded from the project git checkout; the doc's
    // current DB source wins over whatever is committed.
    let tmp = tempfile::tempdir()?;
    state
        .git
        .seed_checkout_async(doc.project_id, tmp.path())
        .await?;
    let doc_file = tmp.path().join(&doc.path);
    if let Some(parent) = doc_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&doc_file, &doc.source)?;

    // Stage the document's woven files into the checkout BEFORE executing.
    //
    // `hick:file` content is a RESULT of the pipeline, so a cell that runs a
    // file the same document assembles — the central move of literate
    // programming, and what the grand tour demonstrates — could only ever work
    // on the second run, once a previous run had committed that file. On a
    // fresh document the input volume was seeded from a checkout that did not
    // contain it yet and the cell died with "No such file or directory".
    //
    // Weaving is pure (no execution), so doing it first costs a few
    // milliseconds and makes the first run behave like every later one. Files
    // that depend on exec output are woven again from the real transcripts
    // afterwards; this only guarantees they EXIST when a volume is seeded.
    {
        let doc_file = doc_file.clone();
        let staged = tokio::task::spawn_blocking(move || -> anyhow::Result<usize> {
            let handle = tokio::runtime::Handle::current();
            let woven = handle.block_on(hickory_cli::run_doc(
                &doc_file,
                &[],
                hickory_cli::RunMode::Weave,
                hickory_cli::ExecutorChoice::Local,
            ))?;
            // MISSING files only: a file already in the checkout is the
            // committed baseline that `check` compares against, and
            // overwriting it with a weave-mode copy would manufacture drift.
            Ok(hickory_cli::write_missing_outputs(&woven, None)?.len())
        })
        .await?;
        match staged {
            Ok(n) if n > 0 => log::debug!("staged {n} woven file(s) for run {run_id}"),
            Ok(_) => {}
            // A document that cannot be woven will fail the real run in a
            // moment with a better message; do not pre-empt it here.
            Err(e) => log::debug!("pre-run weave for {run_id} produced nothing: {e:#}"),
        }
    }

    // Live streaming: each finished exec block's events go out on the run
    // channel as they happen, keyed exactly like the block model
    // (`exec_id = "{container}:{line}"`, `t` in ms).
    let (event_tx, mut event_rx) =
        tokio::sync::mpsc::unbounded_channel::<(String, usize, ExecTranscriptEntry)>();
    let hook: ExecEventHook = {
        let event_tx = event_tx.clone();
        Arc::new(
            move |container: &str, line: usize, entry: &ExecTranscriptEntry| {
                let _ = event_tx.send((container.to_string(), line, entry.clone()));
            },
        )
    };
    drop(event_tx);

    let forwarder = {
        let state = state.clone();
        let doc_id = doc.id;
        tokio::spawn(async move {
            let mut streamed: Vec<(String, usize, ExecTranscriptEntry)> = Vec::new();
            while let Some((container, line, entry)) = event_rx.recv().await {
                let exec_id = format!("{container}:{line}");
                for event in &entry.events {
                    state
                        .rooms
                        .publish_run_event(
                            doc_id,
                            &json!({
                                "run_id": run_id,
                                "exec_id": exec_id,
                                "event": event,
                            }),
                        )
                        .await;
                }
                streamed.push((container, line, entry));
            }
            streamed
        })
    };

    let executor = build_executor(state.config.executor).await?;
    let doc_name = doc.path.clone();
    let sources = vec![(doc_name.as_str(), doc.source.as_str())];
    let config = PipelineConfig {
        working_dir: Some(doc_file.parent().unwrap_or(tmp.path()).to_path_buf()),
        max_rounds: 1,
        on_exec: Some(hook),
    };
    let pipeline = run_pipeline_live(&sources, &config, &[], None, executor.clone()).await;
    // Closing the hook's channel ends the forwarder.
    drop(config);
    let streamed = forwarder.await.unwrap_or_default();

    match pipeline {
        Ok(result) => {
            let parsed =
                hick_lang::parse(&doc.source).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;
            let run = hickory_cli::DocRun {
                doc_path: doc_file.clone(),
                source: doc.source.clone(),
                doc: parsed,
                result,
            };
            let blocks = hickory_cli::block_model(&run);
            let mut ok = !blocks.iter().any(|b| {
                matches!(
                    b,
                    Block::Exec { status, .. } if status == "failed"
                )
            });
            if kind == RunKind::Check {
                let mut failures = hickory_cli::check_failures(&run, None)?;
                // Transform passages are attested, not reproduced: their check
                // is a fingerprint comparison against the source, and never
                // calls a model. See hickory_cli::stale_transforms.
                failures.extend(hickory_cli::stale_transforms(&run.doc_path, &run.source)?);
                if !failures.is_empty() {
                    ok = false;
                    let detail: Vec<String> = failures
                        .iter()
                        .map(|f| match f {
                            hickory_cli::CheckFailure::Expectation(o) => format!(
                                "expectation failed in '{}' at line {}: {}",
                                o.container, o.line, o.detail
                            ),
                            hickory_cli::CheckFailure::Drift {
                                output_path,
                                detail,
                                ..
                            } => {
                                format!("drift in {}: {detail}", output_path.display())
                            }
                            hickory_cli::CheckFailure::StaleTransform { line, select, .. } => {
                                format!(
                                    "line {line}: the passage written from '{select}' no longer \
                                 matches its input — run `hickory refresh`"
                                )
                            }
                        })
                        .collect();
                    sqlx::query("UPDATE runs SET error = $1 WHERE id = $2")
                        .bind(detail.join("\n"))
                        .bind(run_id)
                        .execute(&state.db)
                        .await?;
                }
            }
            store_blocks(state, run_id, &run_blocks_from_model(&blocks)).await?;
            // Persist generated outputs + byte-precise provenance so the
            // /outputs endpoints never re-execute (api.md v0.2).
            if kind == RunKind::Run && ok {
                store_run_outputs(state, run_id, doc.id, &run.result).await?;
            }
            // A successful `run` commits its outputs (woven markdown,
            // generated files) back to the project repo — that commit is the
            // baseline later `check` runs verify drift against.
            if kind == RunKind::Run && ok {
                // Materialize woven markdown + generated files into the run
                // tree first — run_pipeline_live returns them in memory.
                hickory_cli::write_outputs(&run, None)?;
                // Only the pipeline's declared outputs go back into the repo
                // (see GitStore::commit_outputs), plus the doc itself, which
                // is the seed every later run/render reads. Output paths are
                // relative to the document's directory; the repo mirrors the
                // run tree, so re-root them on the doc's parent.
                let prefix = std::path::Path::new(&doc.path)
                    .parent()
                    .unwrap_or(std::path::Path::new(""));
                let mut outputs: Vec<String> = run
                    .result
                    .files
                    .keys()
                    .map(|k| prefix.join(k).to_string_lossy().into_owned())
                    .collect();
                outputs.push(doc.path.clone());
                outputs.sort();
                outputs.dedup();
                state
                    .git
                    .commit_outputs(
                        doc.project_id,
                        tmp.path(),
                        &outputs,
                        &format!("hickory run: outputs of {}", doc.path),
                    )
                    .await?;
            }
            executor.shutdown().await.ok();
            Ok(ok)
        }
        Err(e) => {
            // Pipeline aborted (an exec failed or a document error). Persist
            // what streamed so the transcript survives for GET /api/runs/:id.
            let blocks: Vec<Value> = streamed
                .iter()
                .map(|(container, line, entry)| {
                    let failed = entry.events.iter().any(|ev| {
                        matches!(ev, hickory_executor::TranscriptEvent::Exit { code, .. } if *code != 0)
                    });
                    json!({
                        "exec_id": format!("{container}:{line}"),
                        "status": if failed { "failed" } else { "ok" },
                        "transcript": entry.events,
                    })
                })
                .collect();
            store_blocks(state, run_id, &blocks).await?;
            executor.shutdown().await.ok();
            Err(e)
        }
    }
}

/// Project the rendered block model into the run shape from api.md:
/// `[{exec_id, status, transcript}]`.
pub fn run_blocks_from_model(blocks: &[Block]) -> Vec<Value> {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::Exec {
                id,
                status,
                transcript,
                ..
            } => Some(json!({
                "exec_id": id,
                "status": status,
                "transcript": transcript.clone().unwrap_or_default(),
            })),
            _ => None,
        })
        .collect()
}

/// Persist each text output file of a successful run together with its
/// api.md `Provenance[]`, keyed by run so `GET /api/docs/:id/outputs*`
/// serves the last successful run without re-executing.
async fn store_run_outputs(
    state: &AppState,
    run_id: Uuid,
    doc_id: Uuid,
    result: &hick_literate::PipelineResult,
) -> anyhow::Result<()> {
    for (path, content) in &result.files {
        let Some(text) = content.as_text() else {
            continue; // binary outputs carry no byte-precise text lineage
        };
        let provenance = result
            .provenance_maps
            .get(path)
            .map(hickory_lineage::from_provenance_map)
            .unwrap_or_default();
        sqlx::query(
            "INSERT INTO run_outputs (run_id, doc_id, path, language, content, provenance)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (run_id, path) DO UPDATE
                 SET content = EXCLUDED.content, provenance = EXCLUDED.provenance",
        )
        .bind(run_id)
        .bind(doc_id)
        .bind(path)
        .bind(hickory_lineage::language_for_path(path))
        .bind(text)
        .bind(serde_json::to_value(&provenance)?)
        .execute(&state.db)
        .await?;
    }
    Ok(())
}

async fn store_blocks(state: &AppState, run_id: Uuid, blocks: &[Value]) -> anyhow::Result<()> {
    sqlx::query("UPDATE runs SET blocks = $1 WHERE id = $2")
        .bind(Value::Array(blocks.to_vec()))
        .bind(run_id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// Weave (no-execute) render of a doc from its project checkout, used by
/// `GET /api/docs/:id/render`. Runs against the real checkout dir so
/// includes resolve; nothing is executed in weave mode.
///
/// Every step here is either synchronous filesystem work (the checkout seed,
/// which copies the project tree) or CPU-bound parsing/weaving. Running that
/// on an async runtime worker starves every other request on the same
/// thread — under render load `GET /api/health` stopped answering at all —
/// so the whole body runs on the blocking pool.
pub async fn weave_blocks(state: &AppState, doc: &DocRow) -> anyhow::Result<Vec<Block>> {
    // A `hick:session` is a document too — the conversation the agent wrote,
    // stored in the project. It has no `hick:doc` root and nothing to weave,
    // so weaving it is not an error: it simply renders no cells, and the
    // editor renders its turns inline. Erroring here made every session the
    // agent produced impossible to OPEN in the product that produced it.
    if hick_lang::is_session_source(&doc.source) {
        return Ok(Vec::new());
    }
    let git = state.git.clone();
    let doc = doc.clone();
    let handle = tokio::runtime::Handle::current();
    tokio::task::spawn_blocking(move || {
        // Render from a temp seed too: the DB source may be newer than git.
        let tmp = tempfile::tempdir()?;
        git.seed_checkout(doc.project_id, tmp.path())?;
        let doc_file = tmp.path().join(&doc.path);
        if let Some(parent) = doc_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&doc_file, &doc.source)?;
        // `run_doc` is `async` but performs no I/O await in weave mode; this
        // thread is already off the runtime's worker pool.
        let run = handle.block_on(hickory_cli::run_doc(
            &doc_file,
            &[],
            hickory_cli::RunMode::Weave,
            hickory_cli::ExecutorChoice::Local,
        ))?;
        Ok(hickory_cli::block_model(&run))
    })
    .await?
}

/// [`weave_blocks`] as the JSON array the render route caches and serves.
/// Serialization is part of the blocking task for the same reason the weave
/// is: it is CPU work proportional to document size.
pub async fn weave_blocks_json(state: &AppState, doc: &DocRow) -> anyhow::Result<Value> {
    let blocks = weave_blocks(state, doc).await?;
    Ok(tokio::task::spawn_blocking(move || {
        Value::Array(
            blocks
                .iter()
                .map(|b| serde_json::to_value(b).expect("serializable block"))
                .collect(),
        )
    })
    .await?)
}
