<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/private_practice.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `c8a0186132c1573bf4369774152825f61d605c87b2a8934768a764b7a8a9131b`.

<book:entry path="crates/hickory-cli/src/serve/private_practice.rs">
//! Explicit human creation through the existing tour and host startup protocol.
use super::{
    LocalState, PrivatePredictionDebuggerPlan, PrivatePredictionStartup,
    api::{ApiError, ApiResult},
};
use anyhow::{Result, ensure};
use axum::{Json, extract::State};
use hick_literate::prediction::{
    OutputEncoding, SourceInputs, environment::Profile, output_target,
};
use hick_observe::prediction::{Actor, Identity, Predicate, Scenario, Target, digest};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
const SOURCE: &str = include_str!("../../../../examples/ai-engineer-private-deadline.md");
fn needs_input(reason: impl std::fmt::Display) -> Json<Value> {
    Json(
        json!({"created":false,"requires_input":{"request":"private-practice-environment","reason":reason.to_string()}}),
    )
}
/// Reuse an already configured debugpy installation; creation never installs it
/// or chooses weaker execution permissions. Machine discovery needs no link.
fn adapter(root: &Path, configured: &Path) -> Result<()> {
    if hick_dap::discover("python", root).is_some() {
        return Ok(());
    }
    let found = hick_dap::discover("python", configured).ok_or_else(|| {
        anyhow::anyhow!(
            "Configure Python debugpy in the current host before creating this practice"
        )
    })?;
    ensure!(
        found.command.len() == 3
            && found.command[1] == "-m"
            && found.command[2] == "debugpy.adapter",
        "Unsupported Python adapter recipe"
    );
    let interpreter = PathBuf::from(&found.command[0]);
    let home = interpreter
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("Configured Python environment unavailable"))?;
    ensure!(
        home.join("pyvenv.cfg").is_file(),
        "Configure a discoverable Python debugpy environment for the new practice"
    );
    let destination = root.join(".hick-cache/adapters");
    std::fs::create_dir_all(&destination)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(home, &destination.join("python"))?;
    #[cfg(not(unix))]
    anyhow::bail!(
        "This host needs a machine-discoverable Python debugpy installation; private practice creation does not change platform link permissions"
    );
    ensure!(
        hick_dap::discover("python", root).is_some(),
        "Python adapter is unavailable in the private practice"
    );
    Ok(())
}
async fn seed(parent: &Path, configured: &Path) -> Result<(PathBuf, PrivatePredictionStartup)> {
    let id = hick_context_store::fresh_id();
    let owner = parent.join(&id);
    std::fs::create_dir(&owner)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&owner, std::fs::Permissions::from_mode(0o700))?;
    }
    let result=async {
        let root=owner.join("workspace");
        let private=owner.join("private-ledger");
        std::fs::create_dir(&root)?;
        std::fs::create_dir(&private)?;
        std::fs::write(root.join("deadline-spec.md"),SOURCE)?;
        std::fs::write(root.join(".gitignore"),".hick-cache/\nsessions/\n.hick/\n")?;
        std::fs::write(root.join("README.md"),"# Private prediction practice\n\nSynthetic retry source and inputs. Use the anchored coach to delegate a prediction, initial/future debugger stops, reviewed correction and unchanged verification. Model calls use your configured provider only after explicit Delegate. Target values remain withheld. The ledger is outside this workspace. End retains history and private admission. Output and scratch DAP checks are separate executions.\n")?;
        adapter(&root,configured)?;
        let profile=Profile::PythonDebugpy {project:root.clone()};
        let environment=profile.capture().await?;
        let document=hick_lang::parse(SOURCE)?;
        let exec=document.all_tags().into_iter().find(|tag|tag.name=="exec").ok_or_else(||anyhow::anyhow!("Practice execution unavailable"))?;
        let captures=hick_literate::capture::collect(&document.nodes).map_err(anyhow::Error::msg)?;
        ensure!(captures.len()==1 && captures[0].specs.len()==1,"Practice capture ambiguous");
        let spec=&captures[0].specs[0];
        let scratch=tempfile::tempdir()?;
        let mapping=hick_dap::Mapping::for_document(Path::new("held-out.md"),SOURCE,scratch.path())?;
        let line=mapping.to_document(&scratch.path().join(&spec.file),spec.line.checked_sub(1).ok_or_else(||anyhow::anyhow!("Invalid practice line"))?).ok_or_else(||anyhow::anyhow!("Practice mapping unavailable"))?;
        let start=SOURCE.split_inclusive('\n').take(line as usize).map(str::len).sum::<usize>();
        let end=start+SOURCE.split_inclusive('\n').nth(line as usize).ok_or_else(||anyhow::anyhow!("Practice source span unavailable"))?.len();
        let source=format!("{:x}",Sha256::digest(SOURCE));
        let inputs=json!([[80,80],[10,20],[]]);
        let question=|family:&str,reference:&str,requirement:Predicate|Scenario {family:family.into(),inputs:inputs.clone(),requirement_reference:reference.into(),requirement,identity_known:true};
        let requirement=Predicate::Fields(std::collections::BTreeMap::from([
            ("output".into(),Predicate::Fields(std::collections::BTreeMap::from([("long".into(),Predicate::NumberAtMost(100.0)),("short".into(),Predicate::Equals(json!(30))),("empty".into(),Predicate::Equals(json!(0)))]))),
            ("debugger".into(),Predicate::Equals(json!(20))),
        ]));
        let startup=PrivatePredictionStartup {ledger_root:private,seal:String::new(),request_id:format!("practice:{id}"),identity:Identity {source:source.clone(),inputs:digest(&inputs)?,dependencies:Default::default(),environment},target:Target::Combined {output:Box::new(output_target("c",exec.source_line,OutputEncoding::Json)),debugger:Box::new(Target::Debugger {source:"root".into(),source_revision:source,start_byte:start as u64,end_byte:end as u64,expression:"allowance".into(),hit:2,frame:Some("retry".into())})},actor:Actor {kind:"agent".into(),id:format!("practice:{id}")},encoding:OutputEncoding::Json,question:Some(question("retry-total-deadline","Synthetic source/inputs; real configured execution. One 100ms total budget, including retries, with short/empty cases and the second allowance retained",requirement)),runtime:Some(profile),inputs:Some(SourceInputs {value:inputs.clone(),anchors:vec![SOURCE.lines().find(|line|line.starts_with("print(json.dumps(")).ok_or_else(||anyhow::anyhow!("Practice input anchor unavailable"))?.into()]}),retained:Some(PrivatePredictionDebuggerPlan {initial:question("retry-initial-allowance","Initial allowance is the full 100ms total budget",Predicate::Equals(json!(100))),next:question("retry-next-allowance","After the first 80ms attempt, the second allowance is 20ms",Predicate::Equals(json!(20)))})};
        super::engineer_tour::initialize_git(&root)?;
        Ok((root,startup))
    }.await;
    if result.is_err() {
        let _ = std::fs::remove_dir_all(owner);
    }
    result
}
pub(super) async fn create(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    if state.executor != crate::ExecutorChoice::Local {
        return Ok(needs_input(
            "This private Python/DAP practice requires an explicitly selected local executor. Reopen the host with HICKORY_EXECUTOR=local to authorize that execution environment; creation does not downgrade the current executor.",
        ));
    }
    let parent = hickory_workspace::state_root()
        .map_err(|e| ApiError::unprocessable(e.to_string()))?
        .join("private-practices");
    std::fs::create_dir_all(&parent).map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let parent = parent
        .canonicalize()
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let current = state
        .index
        .root()
        .canonicalize()
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    if parent.starts_with(&current) {
        return Err(ApiError::unprocessable(
            "Private practice storage must be outside the current workspace",
        ));
    }
    let (root, profile) = match seed(&parent, &current).await {
        Ok(created) => created,
        Err(error) => return Ok(needs_input(error)),
    };
    let mut options = crate::engine::Attach::new(root.join("deadline-spec.md"), state.executor);
    options.prediction = Some(profile);
    let connection=crate::engine::connect(options).await.map_err(|error|ApiError::unprocessable(format!("Private startup refused: {error}. No ordinary opening was attempted. Retain {} for trusted recovery",root.display())))?;
    let opened = state.open_folder(&root, super::OpenWhere::NewWindow);
    if opened.is_err() {
        connection.end_private().await.map_err(|_|ApiError::unprocessable("Opening failed and private startup needs trusted recovery; history remains retained"))?;
    }
    connection.detach().await.map_err(|_| {
        ApiError::unprocessable("Private practice created; temporary host client could not detach")
    })?;
    Ok(Json(
        json!({"created":true,"path":root,"opened":opened.is_ok(),"open_error":opened.err().map(|error|error.to_string()),"private_prediction":true,"executed":false,"provider_started":false,"history_outside_workspace":true}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
    #[tokio::test]
    #[ignore = "requires installed Python/debugpy; explicitly run the focused creation witness"]
    async fn seed_is_isolated_unsealed_and_retained_expiry_preserves_evidence() {
        let parent = tempfile::tempdir().unwrap();
        let configured = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (root, profile) = seed(parent.path(), &configured).await.unwrap();
        assert!(!profile.ledger_root.starts_with(&root));
        assert!(profile.seal.is_empty());
        assert!(profile.retained.is_some());
        assert!(root.join(".git").is_dir());
        assert!(!root.join("deadline.py").exists());
        let ledger = hick_observe::prediction::Ledger::open(&profile.ledger_root).unwrap();
        assert!(ledger.history(None).unwrap().ordering.is_empty());
        let prepared = super::super::prepare_with_prediction(
            super::super::ServeOptions {
                target: root.join("deadline-spec.md"),
                port: 0,
                params: vec![],
                executor: crate::ExecutorChoice::Local,
                key_store_path: None,
                ui_settings_path: None,
            },
            profile.binding(&root).unwrap(),
        )
        .await
        .unwrap();
        let bound = prepared.state.private_prediction.unwrap();
        let refused = bound
            .mcp_tool(
                crate::ExecutorChoice::Local,
                &json!({"name":"continue_debugger_prediction","arguments":{}}),
            )
            .await;
        assert_eq!(refused["isError"], true);
        assert!(ledger.history(None).unwrap().attempts.is_empty());
        let sealed=bound.mcp_tool(crate::ExecutorChoice::Local,&json!({"name":"seal_debugger_prediction","arguments":{"expected_source_hash":profile.identity.source,"predicted":{"kind":"equals","value":100},"rationale":"Initial allowance before launch"}})).await;
        assert_eq!(sealed["isError"], false, "{sealed}");
        let receipt = sealed["structuredContent"]["sealed"].as_str().unwrap();
        let launched = bound
            .mcp_tool(
                crate::ExecutorChoice::Local,
                &json!({"name":"continue_debugger_prediction","arguments":{}}),
            )
            .await;
        assert_eq!(launched["isError"], false, "{launched}");
        assert_eq!(
            bound.retained.lock().await.status()["phase"],
            "paused_needs_prediction"
        );
        // Directly challenge the timer callback against an actual paused DAP
        // owner; this does not claim a two-minute wall-clock timing witness.
        bound.expire_retained("retired-source-sequence").await;
        assert_eq!(
            bound.retained.lock().await.status()["phase"],
            "paused_needs_prediction"
        );
        bound.expire_retained(receipt).await;
        assert_eq!(bound.retained.lock().await.status()["phase"], "expired");
        assert_eq!(ledger.history(None).unwrap().attempts.len(), 1);
        assert!(!bound.retained_ready(&profile.identity).unwrap());
        assert!(!root.join("deadline.py").exists());
        let status = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(status.status.success());
        assert!(
            status.stdout.is_empty(),
            "Seeded inputs and instructions must be committed; adapter/cache state stays ignored"
        );
    }
    // Guarantee: prediction success never changes execution permissions.
    #[tokio::test]
    async fn creation_requests_explicit_host_setup_instead_of_downgrading_sandbox() {
        let root = tempfile::tempdir().unwrap();
        let prepared = super::super::prepare(super::super::ServeOptions {
            target: root.path().into(),
            port: 0,
            params: vec![],
            executor: crate::ExecutorChoice::Sandbox,
            key_store_path: None,
            ui_settings_path: None,
        })
        .await
        .unwrap();
        let Json(result) = create(State(prepared.state)).await.unwrap();
        assert_eq!(result["created"], false);
        assert_eq!(
            result["requires_input"]["request"],
            "private-practice-environment"
        );
        assert!(std::fs::read_dir(root.path()).unwrap().next().is_none());
    }
}
</book:entry>

</book:doc>
