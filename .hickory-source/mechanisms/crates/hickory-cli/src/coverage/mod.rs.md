<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/coverage/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-670d7c0cde38"></a>
## `crates/hickory-cli/src/coverage/mod.rs`

<a id="f-670d7c0cde38-commands"></a>
### `commands`

Project testing and instrumented manual sessions. UI and MCP use this service.

<storycode:copy id="f-670d7c0cde38-commands">
//! Project testing and instrumented manual sessions. UI and MCP use this service.
mod commands;
mod discovery;
</storycode:copy>
<a id="f-670d7c0cde38-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-670d7c0cde38-use_declaration">
pub(crate) use runner::execute;
</storycode:copy>
<a id="f-670d7c0cde38-mapping"></a>
### `mapping`

<storycode:copy id="f-670d7c0cde38-mapping">
mod mapping;
</storycode:copy>
<a id="f-670d7c0cde38-real_tests"></a>
### `real_tests`

<storycode:copy id="f-670d7c0cde38-real_tests">
#[cfg(test)]
mod real_tests;
</storycode:copy>
<a id="f-670d7c0cde38-results"></a>
### `results`

<storycode:copy id="f-670d7c0cde38-results">
mod results;
</storycode:copy>
<a id="f-670d7c0cde38-runner"></a>
### `runner`

<storycode:copy id="f-670d7c0cde38-runner">
mod runner;
</storycode:copy>
<a id="f-670d7c0cde38-store"></a>
### `store`

<storycode:copy id="f-670d7c0cde38-store">
mod store;
</storycode:copy>
<a id="f-670d7c0cde38-tests"></a>
### `tests`

<storycode:copy id="f-670d7c0cde38-tests">
#[cfg(test)]
mod tests;

</storycode:copy>
<a id="f-670d7c0cde38-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-670d7c0cde38-use_declaration-2">
use anyhow::{Context, Result, bail, ensure};
</storycode:copy>
<a id="f-670d7c0cde38-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-670d7c0cde38-use_declaration-3">
use hick_coverage::{Evidence, Identity};
</storycode:copy>
<a id="f-670d7c0cde38-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-670d7c0cde38-use_declaration-4">
use serde::{Deserialize, Serialize};
</storycode:copy>
<a id="f-670d7c0cde38-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-670d7c0cde38-use_declaration-5">
use serde_json::{Value, json};
</storycode:copy>
<a id="f-670d7c0cde38-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-670d7c0cde38-use_declaration-6">
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
</storycode:copy>
<a id="f-670d7c0cde38-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-670d7c0cde38-use_declaration-7">
pub use store::{safe_path, snapshot};

</storycode:copy>
<a id="f-670d7c0cde38-Group"></a>
### `Group`

<storycode:copy id="f-670d7c0cde38-Group">
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Group {
    #[serde(default)]
    pub selectors: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}
</storycode:copy>
<a id="f-670d7c0cde38-ManualProfile"></a>
### `ManualProfile`

<storycode:copy id="f-670d7c0cde38-ManualProfile">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualProfile {
    /// Already-instrumented application command. No shell interpretation.
    pub argv: Vec<String>,
    pub format: String,
    /// Workspace-relative report written by instrumentation.
    pub report: String,
    /// Optional collector flush/export command, run after the application exits.
    #[serde(default)]
    pub finish: Vec<String>,
    #[serde(default)]
    pub checkpoint: Vec<String>,
    pub collector: String,
    pub version: String,
}
</storycode:copy>
<a id="f-670d7c0cde38-Config"></a>
### `Config`

<storycode:copy id="f-670d7c0cde38-Config">
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub groups: BTreeMap<String, Group>,
    #[serde(default)]
    pub tests: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub manual: BTreeMap<String, ManualProfile>,
}
</storycode:copy>
<a id="f-670d7c0cde38-TestResult"></a>
### `TestResult`

<storycode:copy id="f-670d7c0cde38-TestResult">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub id: String,
    pub status: String,
    pub output: String,
    pub path: Option<String>,
    pub line: Option<u32>,
    pub labels: Vec<String>,
}
</storycode:copy>
<a id="f-670d7c0cde38-Record"></a>
### `Record`

<storycode:copy id="f-670d7c0cde38-Record">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Namespace of newly captured records; absent legacy records have unknown origin.
    #[serde(default)]
    pub workspace_id: Option<String>,
    /// Only physical-root-v1 establishes current physical workspace origin.
    #[serde(default)]
    pub workspace_identity_kind: Option<String>,
    pub id: String,
    pub mode: String,
    pub status: String,
    pub started_ms: u128,
    pub project: String,
    pub language: String,
    pub group: Option<String>,
    pub metadata: Config,
    pub labels: Vec<String>,
    pub identity: Identity,
    pub argv: Vec<String>,
    pub tests: Vec<TestResult>,
    pub evidence: Option<Evidence>,
    pub output: String,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub source_changed_during_run: bool,
    pub manual_verdict: Option<String>,
    pub notes: String,
    pub checkpoints: Vec<String>,
    pub baseline: Option<String>,
    pub profile: Option<String>,
    pub parent: Option<String>,
    #[serde(default)]
    pub mappings: BTreeMap<String, BTreeMap<u32, Value>>,
}
</storycode:copy>
<a id="f-670d7c0cde38-Request"></a>
### `Request`

<storycode:copy id="f-670d7c0cde38-Request">
#[derive(Debug, Default, Deserialize)]
pub struct Request {
    pub action: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub language: String,
    pub id: Option<String>,
    pub other: Option<String>,
    pub group: Option<String>,
    pub profile: Option<String>,
    pub format: Option<String>,
    pub raw: Option<String>,
    pub identity: Option<Identity>,
    #[serde(default)]
    pub labels: Vec<String>,
    pub verdict: Option<String>,
    #[serde(default)]
    pub notes: String,
    pub metric: Option<String>,
    pub url: Option<String>,
    pub selector: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

</storycode:copy>
<a id="f-670d7c0cde38-config"></a>
### `config`

<storycode:copy id="f-670d7c0cde38-config">
fn config(project: &Path) -> Result<Config> {
    let file = project.join(".hick-tests.json");
    if !file.exists() {
        return Ok(Config::default());
    }
    serde_json::from_str(&std::fs::read_to_string(file)?).context("Invalid .hick-tests.json")
}
</storycode:copy>
<a id="f-670d7c0cde38-project"></a>
### `project`

<storycode:copy id="f-670d7c0cde38-project">
fn project(root: &Path, request: &Request) -> Result<PathBuf> {
    let p = safe_path(
        root,
        if request.path.is_empty() {
            "."
        } else {
            &request.path
        },
    )?;
    if p.is_dir() {
        return Ok(p);
    }
    Ok(
        crate::serve::test_run::test_command(root, &p, "", &request.language)
            .map_err(anyhow::Error::msg)?
            .cwd,
    )
}
</storycode:copy>
<a id="f-670d7c0cde38-id"></a>
### `id`

<storycode:copy id="f-670d7c0cde38-id">
fn id(request: &Request) -> Result<&str> {
    request.id.as_deref().context("Run id required")
}

</storycode:copy>
<a id="f-670d7c0cde38-dispatch"></a>
### `dispatch`

<storycode:copy id="f-670d7c0cde38-dispatch">
/// An engine's original physical root remains authoritative across API requests.
/// A newly created directory at the same spelling cannot authorize collector work.
pub async fn dispatch_scoped(root: &Path, scope: std::sync::Arc<crate::serve::resource_inputs::RootIdentity>, request: Request) -> Result<Value> {
    if let Err(error)=scope.check(root) {
        return match request.action.as_str(){
          "get" if runner::owns_outcome(root,&scope,id(&request)?)=>Ok(runner::original_outcomes(root,&scope,Some(id(&request)?))),
          "list"=>Ok(runner::original_outcomes(root,&scope,None)),
          "stop" if runner::owns_outcome(root,&scope,id(&request)?)&&runner::stop(root,id(&request)?)=>Ok(json!({"id":request.id,"status":"stopping","authority":"historical-unverified"})),
          _=>Err(anyhow::Error::msg(error)),
        };
    }
    dispatch(root, request).await
}

pub async fn dispatch(root: &Path, request: Request) -> Result<Value> {
    let root = root.canonicalize()?;
    match request.action.as_str() {
        "list" => store::list(&root),
        "get" => store::view(&root, id(&request)?),
        "locations" => locations(&root, &request),
        "inspect" => {
            let project = project(&root, &request)?;
            Ok(
                json!({"project":store::relative(&root,&project),"metadata":config(&project)?,"languages":["python","csharp","typescript","rust"],"formats":["coverage-py","cobertura","lcov","istanbul"],"manual":{"browser":"Istanbul instrumentation and loopback Chrome CDP; counters since baseline","native":"Project-owned instrumented app profiles; checkpoints require collector flush support"},"coverage_is_not":"test success, assertion quality or completed workflow verification"}),
            )
        }
        "discover" => {
            let project = project(&root, &request)?;
            discovery::inspect(&root,&project,&request.language).await
        }
        "run" | "manual-start" => {
            let project = project(&root, &request)?;
            let mut record = store::new_record(&root, &project, &request)?;
            if request.action == "manual-start" && request.profile.is_none() {
                // Browser collection is deliberately explicit: instrumentation must be present.
                record.identity.collector = "istanbul-browser".into();
                record.identity.version =
                    "Istanbul map schema v1; instrumenter version unavailable".into();
                record.identity.configuration =
                    "Istanbul counters; source maps resolved by instrumentation".into();
                let baseline = if let Some(url) = &request.url {
                    runner::browser_checkpoint(url, request.selector.as_deref()).await?
                } else {
                    request.raw.clone().context("Browser session requires a CDP URL or an initial Istanbul coverage map as baseline")?
                };
                hick_coverage::parse("istanbul", &baseline)?;
                record.baseline = Some(baseline);
                record.status = "collecting".into();
                store::save(&root, &record)?;
            } else {
                let plan = commands::plan(&root, &project, &request, &mut record)?;
                store::save(&root, &record)?;
                runner::start(root.clone(), project, record.clone(), plan)?;
            }
            Ok(json!(record))
        }
        "stop" => {
            let mut record = store::load(&root, id(&request)?)?;
            if runner::stop(&root, &record.id) {
                return Ok(json!({"id":record.id,"status":"stopping"}));
            }
            ensure!(
                record.mode == "manual" && record.argv.is_empty() && record.status == "collecting",
                "Session is not active in this process"
            );
            record.status = if record.evidence.is_some() {
                "completed"
            } else {
                "unavailable"
            }
            .into();
            record.error = record
                .evidence
                .is_none()
                .then(|| "No instrumented coverage checkpoint was received".into());
            store::save(&root, &record)?;
            store::view(&root, &record.id)
        }
        "browser-checkpoint" => {
            let record = store::load(&root, id(&request)?)?;
            ensure!(
                record.mode == "manual" && record.status == "collecting" && record.argv.is_empty(),
                "Not an active browser session"
            );
            let url = request.url.as_deref().context("Browser CDP URL required")?;
            let raw = runner::browser_checkpoint(url, request.selector.as_deref()).await?;
            checkpoint(&root, request, raw, "istanbul")
        }
        "checkpoint" => {
            let (raw, format) = if let Some(raw) = &request.raw {
                (
                    raw.clone(),
                    request.format.clone().context("Report format required")?,
                )
            } else {
                let record = store::load(&root, id(&request)?)?;
                let profile = record
                    .profile
                    .as_ref()
                    .and_then(|p| record.metadata.manual.get(p))
                    .context("Report data or a native manual profile required")?;
                let project = safe_path(
                    &root,
                    if record.project.is_empty() {
                        "."
                    } else {
                        &record.project
                    },
                )?;
                let profile = commands::manual_profile(&root, &project, &record.id, profile)?;
                if !profile.checkpoint.is_empty() {
                    let output = runner::execute(&profile.checkpoint, &project, None).await?;
                    ensure!(
                        output.code == Some(0),
                        "Collector checkpoint failed: {}",
                        output.text
                    );
                }
                let report = safe_path(&root, &profile.report)?;
                ensure!(
                    std::fs::metadata(&report)?.len() <= 16 * 1024 * 1024,
                    "Report too large"
                );
                (std::fs::read_to_string(report)?, profile.format.clone())
            };
            checkpoint(&root, request, raw, &format)
        }
        "import" => {
            let project = project(&root, &request)?;
            let mut record = store::new_record(&root, &project, &request)?;
            // Never bind an arbitrary imported report to today's source implicitly.
            record.identity = request.identity.clone().unwrap_or(Identity {
                collector: "import".into(),
                version: "unknown".into(),
                configuration: "unknown".into(),
                scope: record.project.clone(),
                source_hashes: BTreeMap::new(),
            });
            let evidence = evidence(
                &root,
                &record,
                request.format.as_deref().context("Format required")?,
                request.raw.as_deref().context("Report required")?,
                "unavailable",
                request.labels.clone(),
            )?;
            record.evidence = Some(evidence);
            record.mappings = mapping::capture(&root, &record)?;
            record.status = "completed".into();
            store::save(&root, &record)?;
            store::view(&root, &record.id)
        }
        "compare" => {
            let a = store::load(&root, id(&request)?)?;
            let b = store::load(
                &root,
                request.other.as_deref().context("Other run required")?,
            )?;
            ensure!(
                !a.source_changed_during_run && !b.source_changed_during_run,
                "Sources changed during collection"
            );
            let a = a.evidence.context("Left report is unavailable")?;
            let b = b.evidence.context("Right report is unavailable")?;
            Ok(serde_json::to_value(hick_coverage::compare(
                &a,
                &b,
                request.metric.as_deref().unwrap_or("line"),
            )?)?)
        }
        "annotate" => {
            let mut record = store::load(&root, id(&request)?)?;
            ensure!(
                record.mode == "manual",
                "Only manual sessions have manual verdicts"
            );
            ensure!(
                !["running", "collecting"].contains(&record.status.as_str()),
                "Stop collection before recording its final verdict"
            );
            ensure!(
                request
                    .verdict
                    .as_deref()
                    .is_none_or(
                        |v| ["passed", "failed", "partial", "blocked", "untested"].contains(&v)
                    ),
                "Invalid manual verdict"
            );
            record.manual_verdict = request.verdict;
            record.notes = request.notes;
            store::save(&root, &record)?;
            store::view(&root, &record.id)
        }
        "rerun-failed" => {
            let previous = store::load(&root, id(&request)?)?;
            ensure!(
                previous.mode == "automated",
                "Manual sessions have no failed test identities"
            );
            let failed: Vec<_> = previous
                .tests
                .iter()
                .filter(|t| t.status == "failed")
                .map(|t| t.id.clone())
                .collect();
            ensure!(!failed.is_empty(), "No failed tests with runner identities");
            let project = safe_path(
                &root,
                if previous.project.is_empty() {
                    "."
                } else {
                    &previous.project
                },
            )?;
            let mut record = store::new_record(
                &root,
                &project,
                &Request {
                    language: previous.language,
                    action: "run".into(),
                    ..Default::default()
                },
            )?;
            let plan = commands::automated(&root, &project, &mut record, &failed)?;
            store::save(&root, &record)?;
            runner::start(root.clone(), project, record.clone(), plan)?;
            Ok(json!(record))
        }
        _ => bail!("Unknown coverage action"),
    }
}

</storycode:copy>
<a id="f-670d7c0cde38-evidence"></a>
### `evidence`

<storycode:copy id="f-670d7c0cde38-evidence">
pub(super) fn evidence(
    root: &Path,
    record: &Record,
    format: &str,
    raw: &str,
    attribution: &str,
    labels: Vec<String>,
) -> Result<Evidence> {
    let mut report = hick_coverage::parse(format, raw)?;
    let project = root.join(&record.project);
    let mut files = BTreeMap::new();
    for (path, data) in report.files {
        let p = Path::new(&path);
        let absolute = if p.is_absolute() {
            p.to_path_buf()
        } else {
            project.join(p)
        };
        let absolute = absolute.canonicalize().with_context(|| {
            format!("Unresolved collector source {path}; source-map or path remapping required")
        })?;
        ensure!(
            absolute.starts_with(root),
            "Collector source is outside workspace: {path}"
        );
        let rel = store::relative(root, &absolute);
        ensure!(
            !files.contains_key(&rel),
            "Ambiguous collector source aliases: {rel}"
        );
        files.insert(rel, data);
    }
    report.files = files;
    let mut identity = record.identity.clone();
    if format == "istanbul" {
        use sha2::{Digest, Sha256};
        let map: Value = serde_json::from_str(raw)?;
        let layout:BTreeMap<_,_>=map.as_object().context("Invalid Istanbul map")?.iter().map(|(path,file)|(path,json!({"statements":file["statementMap"],"branches":file["branchMap"],"functions":file["fnMap"],"schema":file["_coverageSchema"]}))).collect();
        identity.configuration.push_str(&format!(
            "; instrumentation-layout-sha256={:x}",
            Sha256::digest(serde_json::to_vec(&layout)?)
        ));
    }
    Ok(Evidence {
        identity,
        report,
        labels,
        tests: record.tests.iter().map(|t| t.id.clone()).collect(),
        attribution: attribution.into(),
    })
}
</storycode:copy>
<a id="f-670d7c0cde38-checkpoint"></a>
### `checkpoint`

<storycode:copy id="f-670d7c0cde38-checkpoint">
fn checkpoint(root: &Path, request: Request, raw: String, format: &str) -> Result<Value> {
    let mut parent = store::load(root, id(&request)?)?;
    ensure!(
        parent.mode == "manual" && ["collecting", "running"].contains(&parent.status.as_str()),
        "Manual session is not collecting"
    );
    let mut checkpoint = parent.clone();
    checkpoint.id = store::fresh_id();
    checkpoint.parent = Some(parent.id.clone());
    checkpoint.checkpoints.clear();
    checkpoint.status = "completed".into();
    let labels = parent.labels.clone();
    let raw = if let Some(baseline) = &parent.baseline {
        ensure!(
            format == "istanbul",
            "Browser checkpoints require Istanbul coverage"
        );
        runner::browser_delta(baseline, &raw)?
    } else {
        raw
    };
    checkpoint.evidence = Some(evidence(
        root,
        &checkpoint,
        format,
        &raw,
        "group",
        if request.labels.is_empty() {
            labels
        } else {
            request.labels.clone()
        },
    )?);
    checkpoint.mappings = mapping::capture(root, &checkpoint)?;
    checkpoint.source_changed_during_run = snapshot(root)? != parent.identity.source_hashes;
    store::save(root, &checkpoint)?;
    parent.evidence = checkpoint.evidence.clone();
    parent.mappings = checkpoint.mappings.clone();
    parent.source_changed_during_run |= checkpoint.source_changed_during_run;
    parent.checkpoints.push(checkpoint.id.clone());
    store::save(root, &parent)?;
    store::view(root, &checkpoint.id)
}

</storycode:copy>
<a id="f-670d7c0cde38-catalogue"></a>
### `catalogue`

<storycode:copy id="f-670d7c0cde38-catalogue">
pub fn catalogue() -> Value {
    json!({"name":"code_coverage","description":"Discover/run project tests, rerun failures, import/compare coverage, or collect human/AI manual application coverage. Start manual browser testing with a loopback Chrome CDP url and optional page URL selector; browser-checkpoint reads Istanbul counters and subtracts pre-session hits. Native manual profiles in .hick-tests.json launch instrumented apps; checkpoint flushes configured collectors, stop ends collection. Inspect locations for a bounded page of reached/missed executable lines and branches with attribution and source mappings. get retrieves the full report and output; prefer locations to avoid large raw reports. annotate records a separate manual verdict. Missing instrumentation is unavailable, never zero. Coverage does not prove assertions or workflow success.","inputSchema":{"type":"object","required":["action"],"properties":{
        "action":{"type":"string","enum":["list","get","locations","inspect","discover","run","rerun-failed","manual-start","browser-checkpoint","checkpoint","stop","import","compare","annotate"]},
        "offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":500},"path":{"type":"string","description":"Workspace-relative source file or project directory; locations filters this measured file"},"language":{"type":"string","enum":["python","csharp","typescript","rust"]},"id":{"type":"string"},"other":{"type":"string"},"group":{"type":"string"},"profile":{"type":"string"},"url":{"type":"string","description":"Loopback Chrome CDP HTTP URL, e.g. http://127.0.0.1:9222"},"selector":{"type":"string","description":"Substring selecting exactly one page URL"},"labels":{"type":"array","items":{"type":"string"}},"format":{"type":"string","enum":["lcov","coverage-py","cobertura","istanbul"]},"raw":{"type":"string"},"identity":{"type":"object","description":"Explicit original collector/version/configuration/scope/source_hashes for an import; omitted identity remains unknown"},"metric":{"type":"string","enum":["line","branch"]},"verdict":{"type":"string","enum":["passed","failed","partial","blocked","untested"]},"notes":{"type":"string"}
    }}})
}

</storycode:copy>
<a id="f-670d7c0cde38-locations"></a>
### `locations`

<storycode:copy id="f-670d7c0cde38-locations">
fn locations(root: &Path, request: &Request) -> Result<Value> {
    let record = store::load(root, id(request)?)?;
    let evidence = record
        .evidence
        .as_ref()
        .context("Coverage unavailable; no locations can be inferred")?;
    let mut rows = Vec::new();
    let branch = request.metric.as_deref() == Some("branch");
    for (path, file) in &evidence.report.files {
        if !request.path.is_empty() && request.path != *path {
            continue;
        }
        if branch {
            let branches=file.branches.as_ref().context("Identifiable branch metric unavailable; aggregate fractions remain in original report")?;
            for b in branches {
                rows.push(json!({"path":path,"line":b.line,"branch":b.id,"covered":b.covered,"count":b.count,"document":record.mappings.get(path).and_then(|m|m.get(&b.line))}));
            }
        } else {
            for (line, count) in &file.lines {
                rows.push(json!({"path":path,"line":line,"covered":*count>0,"count":if file.line_counts{Some(count)}else{None},"document":record.mappings.get(path).and_then(|m|m.get(line))}));
            }
        }
    }
    let offset = request.offset.unwrap_or(0);
    let limit = request.limit.unwrap_or(100).clamp(1, 500);
    ensure!(offset <= rows.len(), "Location offset exceeds denominator");
    let total = rows.len();
    let page: Vec<_> = rows.into_iter().skip(offset).take(limit).collect();
    let source_state = store::view(root, &record.id)?["source_state"].clone();
    Ok(
        json!({"id":record.id,"source_state":source_state,"metric":if branch{"branch"}else{"line"},"scope":evidence.identity.scope,"collector":evidence.identity.collector,"version":evidence.identity.version,"labels":evidence.labels,"attribution":evidence.attribution,"tests":if evidence.attribution=="individual"{Some(&evidence.tests)}else{None},"run_tests":evidence.tests,"locations":page,"denominator":total,"next":if offset+limit<total{Some(offset+limit)}else{None},"manual_verdict":record.manual_verdict,"status":record.status}),
    )
}
</storycode:copy>

### Read observations without borrowing legacy origin authority

Connected readings inspect a bounded subset of retained collector records without creating a cache directory, starting a process, or asking the regression monitor for status. A hash match describes bytes, while namespace, manual origin and collector attribution remain separate. Skipped/malformed records and enumeration limits remain explicit unknowns.

<storycode:copy id="coverage-reading-evidence">
/// Read-only bounded projection for connected readings. Never starts a collector.
pub(crate) fn reading_evidence(root:&Path,workspace:&str,sources:&BTreeMap<String,String>)->Result<Value>{
 let mut dir=root.canonicalize()?;for part in [".hick-cache","coverage"]{dir.push(part);match std::fs::symlink_metadata(&dir){Ok(meta)=>ensure!(meta.is_dir()&&!meta.file_type().is_symlink(),"Coverage evidence storage is unsafe"),Err(e)if e.kind()==std::io::ErrorKind::NotFound=>return Ok(json!({"records":[],"status":"missing","truncated":false})),Err(e)=>return Err(e.into())}}
 let mut entries=Vec::new();let mut truncated=false;for entry in std::fs::read_dir(&dir)?.take(257){let entry=entry?;if entries.len()>=256{truncated=true;break}let name=entry.file_name().to_string_lossy().into_owned();if name.ends_with(".json"){entries.push((entry.path(),name));}}entries.sort_by(|a,b|a.1.cmp(&b.1));let mut rows=Vec::new();let mut budget=16*1024*1024u64;let mut unknown=0;
 for (path,_) in entries {let meta=std::fs::symlink_metadata(&path)?;if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>budget||meta.len()>4*1024*1024{truncated=true;continue}budget-=meta.len();let record:Record=match serde_json::from_slice(&std::fs::read(path)?){Ok(record)=>record,Err(_)=>{unknown+=1;continue}};let matching:Vec<_>=sources.keys().filter(|path|record.identity.source_hashes.contains_key(*path)).cloned().collect();if matching.is_empty(){continue}let origin=match (record.workspace_identity_kind.as_deref(),record.workspace_id.as_deref()){(Some("physical-root-v1"),Some(id))if id==workspace=>"current-workspace",(Some("physical-root-v1"),Some(_))=>"foreign-workspace",_=>"unknown"};let bytes_current=matching.iter().all(|path|record.identity.source_hashes.get(path)==sources.get(path))&&!record.source_changed_during_run;
 let manual=record.mode!="automated";let freshness=if origin!="current-workspace"{"origin-unknown"}else if !bytes_current{"stale"}else{"current-captured-bytes"};let reach:Vec<_>=record.evidence.as_ref().map(|evidence|evidence.report.files.iter().filter(|(path,_)|matching.contains(path)).map(|(path,file)|json!({"path":path,"observed_lines":file.lines.len(),"hit_lines":file.lines.values().filter(|hits|**hits>0).count(),"basis":if manual{"manual-or-imported-observation"}else{"observed-collector"}})).collect()).unwrap_or_default();
 rows.push(json!({"id":record.id,"mode":record.mode,"status":record.status,"origin":origin,"freshness":freshness,"evidence_kind":if manual{"manual-or-imported"}else if record.evidence.is_some(){"automated-collector"}else{"process-record"},"manual_verdict":record.manual_verdict,"test_results":record.tests.iter().take(100).map(|test|json!({"id":test.id,"status":test.status,"path":test.path})).collect::<Vec<_>>(),"test_results_truncated":record.tests.len()>100,"collector":record.identity.collector,"collector_version":record.identity.version,"attribution":record.evidence.as_ref().map(|e|&e.attribution),"freshness_scope":"matching_inputs_only","uncompared_inputs":record.identity.source_hashes.len().saturating_sub(matching.len()),"scope":record.identity.scope,"configuration":record.identity.configuration,"matching_inputs":matching,"source_changed_during_run":record.source_changed_during_run,"reachability":reach,"exit_code":record.exit_code}));if rows.len()>=100{truncated=true;break}
 }
 Ok(json!({"status":if rows.is_empty(){"missing"}else{"retained"},"records":rows,"truncated":truncated,"unreadable_records":unknown,"notice":"Matching captured input hashes are not proof of frozen execution. Collector reachability is not a guarantee/test assertion relationship. Legacy and foreign origins remain unknown; manual/imported records are not automated verification."}))
}
</storycode:copy>

### Test origin and evidence authority using an owned record store

A source-hash match cannot supply a missing namespace or turn a manual verdict into an automated collector receipt. This test creates only disposable records and verifies that reading an empty evidence store does not create directories.

<storycode:copy id="coverage-reading-evidence-tests">
#[cfg(test)] mod reading_tests {
 use super::*;
 #[test] fn reading_evidence_separates_origin_manual_and_stale_inputs_without_creating_storage(){
  use sha2::{Digest,Sha256};let temporary=tempfile::tempdir().unwrap();let root=temporary.path().canonicalize().unwrap();std::fs::write(root.join("a.py"),"value=41\n").unwrap();let workspace=crate::serve::resource_inputs::RootIdentity::fingerprint(&root).unwrap();let sources=BTreeMap::from([("a.py".to_string(),hex::encode(Sha256::digest(b"value=41\n")))]);
  assert_eq!(reading_evidence(&root,&workspace,&sources).unwrap()["status"],"missing");assert!(!root.join(".hick-cache").exists());
  let request=Request{action:"run".into(),path:".".into(),language:"python".into(),..Default::default()};
  let mut current=store::new_record(&root,&root,&request).unwrap();current.status="finished".into();current.identity.source_hashes=sources.clone();store::save(&root,&current).unwrap();
  let mut legacy=current.clone();legacy.id="11111111111111111111111111111111".into();legacy.workspace_id=Some(hex::encode(Sha256::digest(root.as_os_str().as_encoded_bytes())));legacy.workspace_identity_kind=None;store::save(&root,&legacy).unwrap();
  let mut stale=current.clone();stale.id="22222222222222222222222222222222".into();stale.identity.source_hashes.insert("a.py".into(),"0".repeat(64));store::save(&root,&stale).unwrap();
  let mut manual=current.clone();manual.id="33333333333333333333333333333333".into();manual.mode="manual".into();manual.manual_verdict=Some("pass".into());store::save(&root,&manual).unwrap();
  let value=reading_evidence(&root,&workspace,&sources).unwrap();let rows=value["records"].as_array().unwrap();assert_eq!(rows.len(),4);assert!(rows.iter().any(|row|row["origin"]=="unknown"&&row["freshness"]=="origin-unknown"));assert!(rows.iter().any(|row|row["freshness"]=="stale"));assert!(rows.iter().any(|row|row["evidence_kind"]=="manual-or-imported"&&row["manual_verdict"]=="pass"));assert!(rows.iter().all(|row|row["evidence_kind"]!="automated-collector"));
 }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/coverage/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/coverage/mod.rs"><storycode:paste select="#f-670d7c0cde38-commands"/><storycode:paste select="#f-670d7c0cde38-use_declaration"/><storycode:paste select="#f-670d7c0cde38-mapping"/><storycode:paste select="#f-670d7c0cde38-real_tests"/><storycode:paste select="#f-670d7c0cde38-results"/><storycode:paste select="#f-670d7c0cde38-runner"/><storycode:paste select="#f-670d7c0cde38-store"/><storycode:paste select="#f-670d7c0cde38-tests"/><storycode:paste select="#f-670d7c0cde38-use_declaration-2"/><storycode:paste select="#f-670d7c0cde38-use_declaration-3"/><storycode:paste select="#f-670d7c0cde38-use_declaration-4"/><storycode:paste select="#f-670d7c0cde38-use_declaration-5"/><storycode:paste select="#f-670d7c0cde38-use_declaration-6"/><storycode:paste select="#f-670d7c0cde38-use_declaration-7"/><storycode:paste select="#f-670d7c0cde38-Group"/><storycode:paste select="#f-670d7c0cde38-ManualProfile"/><storycode:paste select="#f-670d7c0cde38-Config"/><storycode:paste select="#f-670d7c0cde38-TestResult"/><storycode:paste select="#f-670d7c0cde38-Record"/><storycode:paste select="#f-670d7c0cde38-Request"/><storycode:paste select="#f-670d7c0cde38-config"/><storycode:paste select="#f-670d7c0cde38-project"/><storycode:paste select="#f-670d7c0cde38-id"/><storycode:paste select="#f-670d7c0cde38-dispatch"/><storycode:paste select="#f-670d7c0cde38-evidence"/><storycode:paste select="#f-670d7c0cde38-checkpoint"/><storycode:paste select="#f-670d7c0cde38-catalogue"/><storycode:paste select="#f-670d7c0cde38-locations"/><storycode:paste select="#coverage-reading-evidence"/><storycode:paste select="#coverage-reading-evidence-tests"/></storycode:file>

</storycode:doc>
