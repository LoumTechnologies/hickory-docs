<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/coverage/runner.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6da46651825b"></a>
## `crates/hickory-cli/src/coverage/runner.rs`

<a id="f-6da46651825b-Sessions"></a>
### `Sessions`

The runner opens the original working directory before admission and executes each collector stage from that held directory. Root replacement retires new stages and source mappings, but bounded in-process reservations retain the terminal process output and parsed results even when persistence refuses its original bucket.

<storycode:copy id="f-6da46651825b-Sessions">
use super::*;
use futures::{SinkExt, StreamExt};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tokio::{io::AsyncReadExt, sync::oneshot};

// Reserve retained-result capacity before a process starts. Completed outcomes are
// never evicted merely because their original source directory disappeared.
struct Retained { scope: std::sync::Arc<crate::serve::resource_inputs::RootIdentity>, record:Record }
fn outcomes()->&'static Mutex<HashMap<String,Retained>> {static VALUES:OnceLock<Mutex<HashMap<String,Retained>>>=OnceLock::new();VALUES.get_or_init(||Mutex::new(HashMap::new()))}
fn reserve(root:&Path,record:&Record,scope:std::sync::Arc<crate::serve::resource_inputs::RootIdentity>)->Result<()> {
 let mut values=outcomes().lock().unwrap();ensure!(values.len()<128,"Retained collector capacity reached; restart after exporting retained results before starting another run");
 values.insert(key(root,&record.id),Retained{scope,record:record.clone()});Ok(())
}
fn retain(root:&Path,record:&Record){if let Some(value)=outcomes().lock().unwrap().get_mut(&key(root,&record.id)){value.record=record.clone();}}
pub(super) fn owns_outcome(root:&Path,scope:&crate::serve::resource_inputs::RootIdentity,id:&str)->bool{outcomes().lock().unwrap().get(&key(root,id)).is_some_and(|value|scope.same_physical_binding(&value.scope))}
pub(super) fn retained_records(root:&Path)->Vec<Record>{let prefix=format!("{}:",root.display());outcomes().lock().unwrap().iter().filter(|(key,_)|key.starts_with(&prefix)).map(|(_,value)|value.record.clone()).collect()}
pub(super) fn retained(root:&Path,id:&str)->Option<Record>{outcomes().lock().unwrap().get(&key(root,id)).map(|value|value.record.clone())}
pub(super) fn original_outcomes(root:&Path,scope:&crate::serve::resource_inputs::RootIdentity,id:Option<&str>)->Value{
 let values=outcomes().lock().unwrap();let mut records:Vec<_>=values.values().filter(|value|scope.same_physical_binding(&value.scope)&&id.is_none_or(|id|value.record.id==id)).map(|value|{let mut record=value.record.clone();record.source_changed_during_run=true;record.mappings.clear();let mut value=json!(record);value["source_state"]=json!("historical");value["storage_state"]=json!("retained-in-original-process");value["authority"]=json!("historical-unverified");value}).collect();
 records.sort_by_key(|record|std::cmp::Reverse(record["started_ms"].as_u64().unwrap_or_default()));
 if id.is_some(){records.into_iter().next().unwrap_or(json!({"error":"Original collector result is unavailable in this process"}))}else{json!({"total":records.len(),"runs":records,"limit":128,"source_state":"historical","authority":"historical-unverified","root":root})}
}
type Sessions = Mutex<HashMap<String, Option<oneshot::Sender<()>>>>;
</storycode:copy>
<a id="f-6da46651825b-sessions"></a>
### `sessions`

<storycode:copy id="f-6da46651825b-sessions">
fn sessions() -> &'static Sessions {
    static SESSIONS: OnceLock<Sessions> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}
</storycode:copy>
<a id="f-6da46651825b-key"></a>
### `key`

<storycode:copy id="f-6da46651825b-key">
fn key(root: &Path, id: &str) -> String {
    format!("{}:{id}", root.display())
}
</storycode:copy>
<a id="f-6da46651825b-active"></a>
### `active`

<storycode:copy id="f-6da46651825b-active">
pub(super) fn active(root: &Path, id: &str) -> bool {
    sessions().lock().unwrap().contains_key(&key(root, id))
}
</storycode:copy>
<a id="f-6da46651825b-stop"></a>
### `stop`

<storycode:copy id="f-6da46651825b-stop">
pub(super) fn stop(root: &Path, id: &str) -> bool {
    sessions()
        .lock()
        .unwrap()
        .get_mut(&key(root, id))
        .and_then(Option::take)
        .is_some_and(|s| s.send(()).is_ok())
}
</storycode:copy>
<a id="f-6da46651825b-Output"></a>
### `Output`

<storycode:copy id="f-6da46651825b-Output">
pub(crate) struct Output {
    pub code: Option<i32>,
    pub text: String,
}
</storycode:copy>
<a id="f-6da46651825b-read"></a>
### `read`

<storycode:copy id="f-6da46651825b-read">
async fn read(mut pipe: impl tokio::io::AsyncRead + Unpin) -> Result<String> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    let mut truncated = false;
    loop {
        let n = pipe.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        if bytes.len() + n <= 2 * 1024 * 1024 {
            bytes.extend_from_slice(&chunk[..n]);
        } else {
            truncated = true;
        }
    }
    let mut text = String::from_utf8_lossy(&bytes).into_owned();
    if truncated {
        text.push_str("\n[output truncated at 2 MiB]\n");
    }
    Ok(text)
}
</storycode:copy>
<a id="f-6da46651825b-execute"></a>
### `execute`

<storycode:copy id="f-6da46651825b-execute">
pub(crate) async fn execute(
    argv: &[String],
    cwd: &Path,
    stop: Option<oneshot::Receiver<()>>,
) -> Result<Output> {
    execute_held(argv,cwd,stop,None,None).await
}
pub(super) async fn execute_held(argv:&[String],cwd:&Path,stop:Option<oneshot::Receiver<()>>,held:Option<&std::fs::File>,artifacts:Option<&Path>)->Result<Output>{
    ensure!(!argv.is_empty(), "Empty collector command");
    let mut cmd = tokio::process::Command::new(&argv[0]);
    cmd.args(&argv[1..])
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    if let Some(artifacts)=artifacts { cmd.env("COVERAGE_FILE",artifacts.join("coverage-data")).env("PYTHONDONTWRITEBYTECODE","1"); }
    #[cfg(unix)] {
        use std::os::fd::AsRawFd;
        cmd.process_group(0);
        if let Some(held)=held { let fd=held.as_raw_fd(); unsafe {cmd.pre_exec(move || {if libc::fchdir(fd)==0 {Ok(())}else{Err(std::io::Error::last_os_error())}});} }
    }
    #[cfg(not(unix))] ensure!(held.is_none(),"Held-directory collector execution is unavailable on this host");
    let mut child = cmd.spawn().with_context(|| {
        format!(
            "Cannot start {}. Install the project collector/toolchain",
            argv[0]
        )
    })?;
    let pid = child.id();
    let stdout = tokio::spawn(read(child.stdout.take().unwrap()));
    let stderr = tokio::spawn(read(child.stderr.take().unwrap()));
    let stopped = async {
        match stop {
            Some(r) => {
                let _ = r.await;
            }
            None => std::future::pending::<()>().await,
        }
    };
    let status = tokio::select! {
        status=child.wait()=>status?,
        _=stopped=>{
            #[cfg(unix)] if let Some(pid)=pid { unsafe {libc::kill(-(pid as i32),libc::SIGINT);} }
            #[cfg(not(unix))] {child.start_kill()?;}
            match tokio::time::timeout(Duration::from_secs(15),child.wait()).await {Ok(s)=>s?,Err(_)=>{
                #[cfg(unix)] if let Some(pid)=pid {unsafe{libc::kill(-(pid as i32),libc::SIGKILL);}}
                child.kill().await?; child.wait().await?
            }}
        },
        _=tokio::time::sleep(Duration::from_secs(3600))=>{
            #[cfg(unix)] if let Some(pid)=pid {unsafe{libc::kill(-(pid as i32),libc::SIGKILL);}}
            child.kill().await?; child.wait().await?
        }
    };
    // Descendant pipes must not hold the API indefinitely after the runner exits.
    let text = tokio::time::timeout(Duration::from_secs(5), async {
        Ok::<_, anyhow::Error>(format!("{}\n{}", stdout.await??, stderr.await??))
    })
    .await
    .context("Collector descendants retained output pipes")??;
    Ok(Output {
        code: status.code(),
        text,
    })
}
</storycode:copy>
<a id="f-6da46651825b-start"></a>
### `start`

<storycode:copy id="f-6da46651825b-start">
pub(super) fn start(
    root: PathBuf,
    project: PathBuf,
    mut record: Record,
    plan: commands::Plan,
) -> Result<()> {
    let scope=std::sync::Arc::new(crate::serve::resource_inputs::RootIdentity::bind(&root));
    scope.check(&root).map_err(anyhow::Error::msg)?;
    let held=std::fs::File::open(&project)?;
    ensure!(same_file::Handle::from_file(held.try_clone()?)?==same_file::Handle::from_path(&project)?,"Collector directory changed");
    scope.check(&root).map_err(anyhow::Error::msg)?;
    ensure!(record.workspace_id==crate::serve::resource_inputs::RootIdentity::fingerprint(&root).ok(),"Collector workspace changed before process admission");
    reserve(&root,&record,scope.clone())?;
    let (sender, stop) = oneshot::channel();
    sessions()
        .lock()
        .unwrap()
        .insert(key(&root, &record.id), Some(sender));
    tokio::spawn(async move {
        let result = collect(&root, &project, &mut record, plan, stop, &scope, &held).await;
        if let Err(e) = result {
            record.error = Some(format!("{e:#}"));
            record.status = "unavailable".into();
        }
        if scope.check(&root).is_ok() { if let Ok(saved) = store::load(&root, &record.id) {
            record.checkpoints = saved.checkpoints;
        }}
        record.source_changed_during_run = if scope.check(&root).is_err() {true} else {snapshot(&root)
            .map(|s| s != record.identity.source_hashes)
            .unwrap_or(true)};
        retain(&root,&record);
        if let Err(e) = scope.check(&root).map_err(anyhow::Error::msg).and_then(|_|store::save(&root, &record)) {
            let notice=format!("Original collector outcome is retained in this process; durable storage unavailable: {e:#}");
            record.error=Some(match record.error.take(){Some(error)=>format!("{error}\n{notice}"),None=>notice});
            retain(&root,&record);
            log::error!("Saving coverage {}: {e}", record.id);
        }
        sessions().lock().unwrap().remove(&key(&root, &record.id));
    });
    Ok(())
}
</storycode:copy>
<a id="f-6da46651825b-collect"></a>
### `collect`

<storycode:copy id="f-6da46651825b-collect">
async fn collect(
    root: &Path,
    project: &Path,
    record: &mut Record,
    plan: commands::Plan,
    stop: oneshot::Receiver<()>,
    scope:&crate::serve::resource_inputs::RootIdentity,
    held:&std::fs::File,
) -> Result<()> {
    if let Some(artifacts)=&plan.artifacts{artifacts.check()?;}
    let artifact_path=plan.artifacts.as_ref().map(|dir|dir.path());
    if !plan.version.is_empty() {
        scope.check(root).map_err(anyhow::Error::msg)?;
        let version = execute_held(&plan.version, project, None,Some(held),artifact_path).await?;
        ensure!(
            version.code == Some(0),
            "Collector version unavailable: {}",
            version.text
        );
        record.identity.version = version.text.trim().into();
        scope.check(root).map_err(anyhow::Error::msg)?;
        store::save(root, record)?;
    }
    scope.check(root).map_err(anyhow::Error::msg)?;
    let output = execute_held(&plan.argv, project, Some(stop),Some(held),artifact_path).await?;
    record.output = output.text;
    record.exit_code = output.code;
    if !plan.finish.is_empty() {
        scope.check(root).map_err(anyhow::Error::msg)?;
        let output = execute_held(&plan.finish, project, None,Some(held),artifact_path).await?;
        record.output.push_str(&output.text);
        ensure!(output.code == Some(0), "Collector export failed");
    }
    if let Some(artifacts)=&plan.artifacts{artifacts.check()?;}
    if record.mode == "automated" {
        let result_path = if record.language == "csharp" {
            find_report(plan.report.parent().unwrap(), "results")?.unwrap_or(plan.result.clone())
        } else {
            plan.result.clone()
        };
        let raw = std::fs::read_to_string(&result_path).unwrap_or_default();
        match results::parse(
            &record.language,
            &raw,
            &record.output,
            &record.metadata,
            project,
        ) {
            Ok(mut tests) => {
                for test in &mut tests {
                    if let Some(path) = &test.path {
                        let file = if Path::new(path).is_absolute() {
                            PathBuf::from(path)
                        } else {
                            project.join(path)
                        };
                        test.path = file
                            .canonicalize()
                            .ok()
                            .filter(|p| p.starts_with(root))
                            .map(|p| store::relative(root, &p));
                    }
                }
                record.tests = tests;
            }
            Err(e) => record.error = Some(format!("Test result import incomplete: {e:#}")),
        }
    }
    let report = if record.language == "csharp" && record.mode == "automated" {
        let all = find_reports(plan.report.parent().unwrap(), "coverage.cobertura.xml")?;
        let mut unique = std::collections::BTreeMap::new();
        for path in all {
            ensure!(
                std::fs::metadata(&path)?.len() <= 16 * 1024 * 1024,
                "Coverage report exceeds 16 MiB"
            );
            unique.entry(std::fs::read(&path)?).or_insert(path);
        }
        let paths: Vec<_> = unique.into_values().collect();
        ensure!(
            paths.len() == 1,
            "Expected one target's Coverlet report, found {}. Select one project/framework or import reports separately",
            paths.len()
        );
        paths[0].clone()
    } else {
        plan.report
    };
    let report = if let Some(dir)=artifact_path {
        let canonical=report.canonicalize().context("Collector did not write coverage; install/configure its coverage plugin")?;
        ensure!(canonical.starts_with(dir.canonicalize()?),"Collector report escaped private artifacts");canonical
    } else {scope.check(root).map_err(anyhow::Error::msg)?;safe_path(root,&store::relative(root,&report))?};
    ensure!(
        std::fs::metadata(&report)?.len() <= 16 * 1024 * 1024,
        "Coverage report exceeds 16 MiB"
    );
    let raw = std::fs::read_to_string(report)?;
    record.evidence = Some(if scope.check(root).is_err() {Evidence{identity:record.identity.clone(),report:hick_coverage::parse(&plan.format,&raw)?,labels:record.labels.clone(),tests:record.tests.iter().map(|test|test.id.clone()).collect(),attribution:"group".into()}} else {evidence(
        root,
        record,
        &plan.format,
        &raw,
        "group",
        record.labels.clone(),
    )?});
    if scope.check(root).is_ok() {record.mappings = super::mapping::capture(root, record)?;} else {record.source_changed_during_run=true;record.mappings.clear();}
    record.status = if record.mode == "manual" || output.code == Some(0) {
        "completed"
    } else {
        "failed"
    }
    .into();
    Ok(())
}
</storycode:copy>
<a id="f-6da46651825b-find_reports"></a>
### `find_reports`

<storycode:copy id="f-6da46651825b-find_reports">
fn find_reports(dir: &Path, name: &str) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        if e.file_type()?.is_dir() {
            paths.extend(find_reports(&e.path(), name)?);
        } else if e.file_type()?.is_file() && e.file_name() == name {
            paths.push(e.path());
        }
    }
    Ok(paths)
}
</storycode:copy>
<a id="f-6da46651825b-find_report"></a>
### `find_report`

<storycode:copy id="f-6da46651825b-find_report">
fn find_report(dir: &Path, name: &str) -> Result<Option<PathBuf>> {
    Ok(find_reports(dir, name)?.into_iter().next())
}

</storycode:copy>
<a id="f-6da46651825b-browser_checkpoint"></a>
### `browser_checkpoint`

Read Istanbul counters from a user's existing Chromium tab. No page mutation.
The caller supplies a loopback CDP endpoint, never an arbitrary remote URL.

<storycode:copy id="f-6da46651825b-browser_checkpoint">
/// Read Istanbul counters from a user's existing Chromium tab. No page mutation.
/// The caller supplies a loopback CDP endpoint, never an arbitrary remote URL.
pub(super) async fn browser_checkpoint(endpoint: &str, selector: Option<&str>) -> Result<String> {
    let url = reqwest::Url::parse(endpoint)?;
    ensure!(
        url.scheme() == "http"
            && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")),
        "Use a loopback Chrome CDP HTTP endpoint"
    );
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()?;
    let pages: Value = client
        .get(url.join("/json/list")?)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let pages: Vec<_> = pages
        .as_array()
        .context("Invalid Chrome target list")?
        .iter()
        .filter(|p| {
            p["type"] == "page"
                && selector.is_none_or(|s| p["url"].as_str().is_some_and(|u| u.contains(s)))
        })
        .collect();
    ensure!(
        pages.len() == 1,
        "Select exactly one browser page by URL; found {}",
        pages.len()
    );
    let ws = pages[0]["webSocketDebuggerUrl"]
        .as_str()
        .context("Page has no debugger socket")?;
    let ws_url = reqwest::Url::parse(ws)?;
    ensure!(
        ws_url.scheme() == "ws"
            && ws_url.host_str() == url.host_str()
            && ws_url.port_or_known_default() == url.port_or_known_default(),
        "Chrome debugger socket is not on the requested loopback endpoint"
    );
    let (mut socket, _) = tokio_tungstenite::connect_async(ws).await?;
    socket.send(tokio_tungstenite::tungstenite::Message::Text(json!({"id":1,"method":"Runtime.evaluate","params":{"expression":"globalThis.__coverage__ ? JSON.stringify(Object.fromEntries(Object.entries(globalThis.__coverage__).map(([p,c]) => [p,{...c,__hickoryTimeOrigin:performance.timeOrigin}]))) : 'null'","returnByValue":true}}).to_string())).await?;
    let raw=tokio::time::timeout(Duration::from_secs(10),async {
        while let Some(message)=socket.next().await {
            let message=message?;if !message.is_text(){continue;}
            let v:Value=serde_json::from_str(message.to_text()?)?;
            if v["id"]==1 {let raw=v["result"]["result"]["value"].as_str().context("Browser refused coverage read")?;ensure!(raw!="null","This page is not instrumented: globalThis.__coverage__ is unavailable. Configure Istanbul in the tested app");return Ok::<_,anyhow::Error>(raw.to_string());}
        }
        bail!("Chrome closed the debugger connection")
    }).await??;
    let _ = socket.close(None).await;
    Ok(raw)
}

</storycode:copy>
<a id="f-6da46651825b-browser_delta"></a>
### `browser_delta`

A session reports increments since its baseline, not code reached by an older visit.
Reload/reset or instrumentation changes invalidate the session rather than inventing hits.

<storycode:copy id="f-6da46651825b-browser_delta">
/// A session reports increments since its baseline, not code reached by an older visit.
/// Reload/reset or instrumentation changes invalidate the session rather than inventing hits.
pub(super) fn browser_delta(baseline: &str, current: &str) -> Result<String> {
    let baseline: Value = serde_json::from_str(baseline)?;
    let mut current: Value = serde_json::from_str(current)?;
    for (path, file) in current.as_object_mut().context("Invalid coverage map")? {
        if let Some(before) = baseline.get(path) {
            ensure!(
                file["__hickoryTimeOrigin"] == before["__hickoryTimeOrigin"],
                "Browser document reloaded; start a new session"
            );
            ensure!(
                file["statementMap"] == before["statementMap"]
                    && file["branchMap"] == before["branchMap"],
                "Browser instrumentation changed; start a new manual session"
            );
            for kind in ["s", "f"] {
                for (id, value) in file[kind]
                    .as_object_mut()
                    .context("Missing Istanbul counters")?
                {
                    let count = value.as_u64().context("Invalid count")?;
                    let old = before[kind][id]
                        .as_u64()
                        .context("Counter layout changed")?;
                    ensure!(
                        count >= old,
                        "Browser counters reset/reloaded; start a new manual session"
                    );
                    *value = json!(count - old);
                }
            }
            for (id, branches) in file["b"]
                .as_object_mut()
                .context("Missing branch counters")?
            {
                for (index, value) in branches
                    .as_array_mut()
                    .context("Invalid branch counters")?
                    .iter_mut()
                    .enumerate()
                {
                    let count = value.as_u64().context("Invalid branch count")?;
                    let old = before["b"][id][index]
                        .as_u64()
                        .context("Branch layout changed")?;
                    ensure!(
                        count >= old,
                        "Browser counters reset/reloaded; start a new session"
                    );
                    *value = json!(count - old);
                }
            }
        }
    }
    ensure!(
        baseline
            .as_object()
            .context("Invalid baseline")?
            .keys()
            .all(|k| current.get(k).is_some()),
        "Browser modules disappeared/reloaded; start a new session"
    );
    Ok(serde_json::to_string(&current)?)
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/coverage/runner.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/coverage/runner.rs"><storycode:paste select="#f-6da46651825b-Sessions"/><storycode:paste select="#f-6da46651825b-sessions"/><storycode:paste select="#f-6da46651825b-key"/><storycode:paste select="#f-6da46651825b-active"/><storycode:paste select="#f-6da46651825b-stop"/><storycode:paste select="#f-6da46651825b-Output"/><storycode:paste select="#f-6da46651825b-read"/><storycode:paste select="#f-6da46651825b-execute"/><storycode:paste select="#f-6da46651825b-start"/><storycode:paste select="#f-6da46651825b-collect"/><storycode:paste select="#f-6da46651825b-find_reports"/><storycode:paste select="#f-6da46651825b-find_report"/><storycode:paste select="#f-6da46651825b-browser_checkpoint"/><storycode:paste select="#f-6da46651825b-browser_delta"/></storycode:file>

</storycode:doc>
