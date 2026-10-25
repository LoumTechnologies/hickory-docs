//! Project profiling shared by UI and agents; existing confined Python capture stays intact.
mod browser;
mod native;
#[cfg(test)]
mod real_tests;
mod resolve;
mod store;
#[cfg(test)]
mod tests;
use crate::coverage::{safe_path, snapshot};
use anyhow::{Context, Result, bail, ensure};
use hick_profile::{Frame, Report};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub collector: String,
    pub version: String,
    pub configuration: String,
    pub scope: String,
    pub source_hashes: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub status: String,
    pub mode: String,
    pub started_ms: u128,
    pub identity: Identity,
    pub labels: Vec<String>,
    pub report: Option<Report>,
    pub mappings: BTreeMap<String, BTreeMap<u32, Value>>,
    pub sources: BTreeMap<String, String>,
    pub output: String,
    pub error: Option<String>,
    pub source_changed: bool,
    pub environment: Value,
}
#[derive(Default, Deserialize)]
pub struct Request {
    pub action: String,
    #[serde(default)]
    pub path: String,
    pub id: Option<String>,
    pub other: Option<String>,
    pub profile: Option<String>,
    pub format: Option<String>,
    pub raw: Option<String>,
    pub identity: Option<Identity>,
    #[serde(default)]
    pub labels: Vec<String>,
    pub url: Option<String>,
    pub selector: Option<String>,
    pub seconds: Option<u64>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub thread: Option<usize>,
    pub program: Option<String>,
}
fn id(request: &Request) -> Result<&str> {
    request.id.as_deref().context("Profile run id required")
}
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
fn project(root: &Path, request: &Request) -> Result<PathBuf> {
    let p = safe_path(
        root,
        if request.path.is_empty() {
            "."
        } else {
            &request.path
        },
    )?;
    Ok(if p.is_dir() {
        p
    } else {
        p.parent()
            .context("Project directory required")?
            .to_path_buf()
    })
}
fn fresh(root: &Path, project: &Path, request: &Request, mode: &str) -> Result<Record> {
    let mut random = [0u8; 16];
    getrandom::fill(&mut random)?;
    Ok(Record {
        id: hex::encode(random),
        status: "running".into(),
        mode: mode.into(),
        started_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis(),
        identity: Identity {
            collector: "pending".into(),
            version: "unknown".into(),
            configuration: String::new(),
            scope: relative(root, project),
            source_hashes: snapshot(root)?,
        },
        labels: request
            .labels
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect(),
        report: None,
        mappings: BTreeMap::new(),
        sources: BTreeMap::new(),
        output: String::new(),
        error: None,
        source_changed: false,
        environment: json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH}),
    })
}
pub async fn dispatch(root: &Path, request: Request) -> Result<Value> {
    let root = root.canonicalize()?;
    match request.action.as_str() {
        "list" => store::list(&root),
        "get" => store::view(&root, id(&request)?, true),
        "locations" => locations(&root, &request),
        "inspect" => {
            let project = project(&root, &request)?;
            Ok(
                json!({"profiles":native::config(&project)?,"formats":["chrome-cpu","speedscope","cprofile-json","firefox"],"browser":"V8 CPU sampling via loopback CDP; source maps or byte-identical JavaScript required for inline anchors","python":"Existing explicit confined saved-Python provider via capture-python","native":"Project-owned argv profiles; process runs with local permissions"}),
            )
        }
        "capture-python" => {
            let path = safe_path(&root, &request.path)?;
            let mut record = fresh(&root, path.parent().unwrap(), &request, "confined-python")?;
            store::save(&root, &record)?;
            let result = async {
                let run = crate::observation::capture_mode(
                    &path,
                    request.program.as_deref(),
                    request.seconds.unwrap_or(10),
                    true,
                )
                .await?;
                hick_observe::LocalStore::open(path.parent().unwrap())?.save(&run)?;
                resolve::python(&root, &path, &mut record, &run)
            }
            .await;
            store::finish(&root, &mut record, result);
            store::view(&root, &record.id, true)
        }
        "import" => {
            let project = project(&root, &request)?;
            let mut record = fresh(&root, &project, &request, "import")?;
            record.identity = request.identity.unwrap_or(Identity {
                collector: "import".into(),
                version: "unknown".into(),
                configuration: "unknown".into(),
                scope: relative(&root, &project),
                source_hashes: BTreeMap::new(),
            });
            let format = request.format.as_deref().context("Format required")?;
            let raw = request.raw.as_deref().context("Report required")?;
            record.report = Some(hick_profile::parse(format, raw)?);
            record.status = "completed".into();
            resolve::bind(&root, &mut record, None)?;
            store::save(&root, &record)?;
            store::view(&root, &record.id, true)
        }
        "browser-start" => {
            let project = project(&root, &request)?;
            let mut record = fresh(&root, &project, &request, "manual-browser")?;
            let connection = browser::connect(
                request.url.as_deref().context("CDP URL required")?,
                request.selector.as_deref(),
            )
            .await?;
            record.identity.collector = "V8 CPU profiler/CDP".into();
            record.identity.version = connection.version.clone();
            record.identity.configuration =
                "1000 microsecond sampling interval; one page V8 isolate; function anchors".into();
            store::save(&root, &record)?;
            browser::start(root.clone(), record.clone(), connection)?;
            store::view(&root, &record.id, true)
        }
        "run" => {
            let project = project(&root, &request)?;
            let mut record = fresh(&root, &project, &request, "native-profile")?;
            let plan = native::plan(&root, &project, &request, &mut record)?;
            store::save(&root, &record)?;
            native::start(root.clone(), project, record.clone(), plan)?;
            store::view(&root, &record.id, true)
        }
        "stop" => {
            let run = id(&request)?;
            ensure!(
                browser::stop(&root, run) || native::stop(&root, run),
                "Session is not active in this process"
            );
            Ok(json!({"id":run,"status":"stopping"}))
        }
        "compare" => compare(&root, &request),
        _ => bail!("Unknown profiling action"),
    }
}
fn locations(root: &Path, request: &Request) -> Result<Value> {
    let record = store::load(root, id(request)?)?;
    let report = record
        .report
        .as_ref()
        .context("Profile unavailable; no costs can be inferred")?;
    let scope = request.thread.unwrap_or(0);
    let thread = report
        .threads
        .get(scope)
        .context("Profile scope index unavailable")?;
    let rows: Vec<_> = thread
        .rows
        .iter()
        .filter(|r| request.path.is_empty() || r.frame.file.as_deref() == Some(&request.path))
        .collect();
    let offset = request.offset.unwrap_or(0);
    let limit = request.limit.unwrap_or(100).clamp(1, 500);
    ensure!(offset <= rows.len(), "Offset exceeds profile rows");
    Ok(
        json!({"run":record.id,"source_state":store::source_state(root,&record),"collector":record.identity.collector,"version":record.identity.version,"scope":thread.name,"unit":thread.unit,"aggregation":thread.aggregation,"measured_weight":thread.measured_weight,"interval":[thread.interval_start,thread.interval_end],"interval_unit":thread.interval_unit,"clock":thread.clock,"labels":record.labels,"rows":rows.iter().skip(offset).take(limit).map(|r|{let mut row=json!(r);row["document"]=r.frame.file.as_ref().zip(r.frame.line).and_then(|(file,line)|record.mappings.get(file).and_then(|m|m.get(&line))).cloned().unwrap_or(Value::Null);row}).collect::<Vec<_>>(),"total":rows.len(),"next":(offset+limit<rows.len()).then_some(offset+limit),"limitations":report.limitations}),
    )
}
fn compare(root: &Path, request: &Request) -> Result<Value> {
    let a = store::load(root, id(request)?)?;
    let b = store::load(
        root,
        request.other.as_deref().context("Other run required")?,
    )?;
    ensure!(
        !a.source_changed && !b.source_changed,
        "Sources changed during collection"
    );
    ensure!(
        !a.identity.source_hashes.is_empty() && !b.identity.source_hashes.is_empty(),
        "Original source identity required"
    );
    ensure!(
        a.identity.collector == b.identity.collector
            && a.identity.version == b.identity.version
            && a.identity.configuration == b.identity.configuration
            && a.identity.scope == b.identity.scope,
        "Collector/version, configuration or measured scope differ"
    );
    ensure!(
        !["unknown", ""].contains(&a.identity.version.as_str())
            && a.identity.configuration != "unknown",
        "Collector version and instrumentation configuration must be known for comparison"
    );
    let index = request.thread.unwrap_or(0);
    let left = a
        .report
        .as_ref()
        .context("Left profile missing")?
        .threads
        .get(index)
        .context("Left scope missing")?;
    let right = b
        .report
        .as_ref()
        .context("Right profile missing")?
        .threads
        .get(index)
        .context("Right scope missing")?;
    ensure!(
        left.unit == right.unit
            && left.interval_unit == right.interval_unit
            && left.clock == right.clock
            && left.aggregation == right.aggregation
            && left.name == right.name,
        "Metric or thread semantics differ"
    );
    let left_rows: BTreeMap<_, _> = left
        .rows
        .iter()
        .map(|r| {
            let mut frame = r.frame.clone();
            frame.script_id = None;
            (frame, r)
        })
        .collect();
    let right_rows: BTreeMap<_, _> = right
        .rows
        .iter()
        .map(|r| {
            let mut frame = r.frame.clone();
            frame.script_id = None;
            (frame, r)
        })
        .collect();
    let keys: std::collections::BTreeSet<_> =
        left_rows.keys().chain(right_rows.keys()).cloned().collect();
    let mut rows:Vec<_>=keys.into_iter().map(|frame|json!({"frame":frame,"left":left_rows.get(&frame),"right":right_rows.get(&frame),"self_delta":left_rows.get(&frame).zip(right_rows.get(&frame)).map(|(l,r)|r.self_weight-l.self_weight)})).collect();
    rows.sort_by(|a, b| {
        b["self_delta"]
            .as_f64()
            .unwrap_or(0.)
            .abs()
            .total_cmp(&a["self_delta"].as_f64().unwrap_or(0.).abs())
    });
    let total = rows.len();
    let offset = request.offset.unwrap_or(0);
    let limit = request.limit.unwrap_or(100).clamp(1, 500);
    ensure!(offset <= total, "Comparison offset exceeds rows");
    let rows: Vec<_> = rows.into_iter().skip(offset).take(limit).collect();
    Ok(
        json!({"left":a.id,"right":b.id,"left_source_state":store::source_state(root,&a),"right_source_state":store::source_state(root,&b),"same_sources":a.identity.source_hashes==b.identity.source_hashes,"unit":left.unit,"rows":rows,"total":total,"next":(offset+limit<total).then_some(offset+limit),"scope":"Recorded workload comparison; changes in input/environment, profiler overhead and sampling noise can affect cost. A missing function is unavailable, not zero."}),
    )
}
pub fn catalogue() -> Value {
    json!({"name":"performance_profile","description":"Collect and inspect performance evidence for human/AI workflows. browser-start uses V8 CPU sampling in an existing loopback Chrome tab; drive the UI separately, then stop and inspect locations. run launches a project-owned .hick-profile.json collector profile. capture-python reuses the explicit confined saved-Python provider. Import chrome-cpu, Speedscope or cprofile-json. Inspect bounded per-scope rows, source mappings, sample weights and measured intervals; select evidence inline in literate documents. Sampling counts are not calls; inclusive time overlaps. Compare retains before/after source identities and never infers speedups from missing samples.","inputSchema":{"type":"object","required":["action"],"properties":{"action":{"type":"string","enum":["list","get","locations","inspect","capture-python","import","browser-start","run","stop","compare"]},"path":{"type":"string"},"id":{"type":"string"},"other":{"type":"string"},"profile":{"type":"string"},"format":{"type":"string","enum":["chrome-cpu","speedscope","cprofile-json","firefox"]},"raw":{"type":"string"},"identity":{"type":"object"},"labels":{"type":"array","items":{"type":"string"}},"url":{"type":"string"},"selector":{"type":"string"},"seconds":{"type":"integer","minimum":1,"maximum":120},"program":{"type":"string"},"thread":{"type":"integer","minimum":0},"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":500}}}})
}
