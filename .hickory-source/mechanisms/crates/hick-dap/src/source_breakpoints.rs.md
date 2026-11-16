<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/source_breakpoints.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-41b419af990c"></a>
## `crates/hick-dap/src/source_breakpoints.rs`

<a id="f-41b419af990c-SourceSet"></a>
### `SourceSet`

<storycode:copy id="f-41b419af990c-SourceSet">
use super::*;
use std::collections::HashSet;

#[derive(Clone)]
struct SourceSet {
    document: Option<PathBuf>,
    breakpoints: Vec<Breakpoint>,
}
</storycode:copy>
<a id="f-41b419af990c-Sources"></a>
### `Sources`

DAP replaces a whole generated file. Keep every source's requested set so
updating one contributor cannot erase another contributor in the same file.

<storycode:copy id="f-41b419af990c-Sources">
/// DAP replaces a whole generated file. Keep every source's requested set so
/// updating one contributor cannot erase another contributor in the same file.
#[derive(Default)]
pub(super) struct Sources {
    serial: tokio::sync::Mutex<()>,
    desired: std::sync::Mutex<HashMap<String, SourceSet>>,
    files: std::sync::Mutex<HashSet<PathBuf>>,
    statuses: std::sync::Mutex<HashMap<String, Vec<BreakpointStatus>>>,
    ids: std::sync::Mutex<HashMap<i64, (String, u32)>>,
}
</storycode:copy>
<a id="f-41b419af990c-impl-Session"></a>
### `impl-Session`

A failed adapter replacement must not become the accepted desired set and then
silently retry at a later stop. Build a proposed set, retain all possibly touched
file addresses for cleanup, and commit desired source/root wishes only after the
whole delivery succeeds. Internal run-to replacements preserve the user's root
wishes, so clearing the temporary breakpoint can restore the accepted set.
Receipt authority is independently suspended by the
engine while delivery can partially affect the adapter.

<storycode:copy id="f-41b419af990c-impl-Session">
impl Session {
    pub async fn set_breakpoints(
        &self,
        breakpoints: &[Breakpoint],
    ) -> Result<Vec<BreakpointStatus>> {
        self.apply_for("", None, breakpoints, true).await
    }

    pub(super) async fn apply_breakpoints(
        &self,
        breakpoints: &[Breakpoint],
    ) -> Result<Vec<BreakpointStatus>> {
        self.apply_for("", None, breakpoints, false).await
    }

    pub async fn set_source_breakpoints(
        &self,
        source: &str,
        document: &std::path::Path,
        breakpoints: &[Breakpoint],
    ) -> Result<Vec<BreakpointStatus>> {
        self.apply_for(source, Some(document.to_path_buf()), breakpoints, false)
            .await
    }

    pub fn source_breakpoint_hashes(&self) -> HashMap<String, String> {
        self.sources
            .desired
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(key, set)| {
                let document = set.document.as_deref()?;
                self.mapping
                    .source_hash(document)
                    .map(|hash| (key.clone(), hash))
            })
            .collect()
    }

    pub fn source_breakpoint_statuses(&self) -> HashMap<String, Vec<BreakpointStatus>> {
        self.sources.statuses.lock().unwrap().clone()
    }

    async fn apply_for(
        &self,
        key: &str,
        document: Option<PathBuf>,
        breakpoints: &[Breakpoint],
        commit_root: bool,
    ) -> Result<Vec<BreakpointStatus>> {
        let _serial = self.sources.serial.lock().await;
        let sets = {
            let mut desired = self.sources.desired.lock().unwrap().clone();
            desired.insert(
                key.to_string(),
                SourceSet {
                    document,
                    breakpoints: breakpoints.to_vec(),
                },
            );
            desired
        };
        let mut by_file: HashMap<PathBuf, Vec<(&str, &Breakpoint, u32)>> = HashMap::new();
        let mut statuses: HashMap<String, Vec<BreakpointStatus>> = HashMap::new();
        for (source, set) in &sets {
            let held = statuses.entry(source.clone()).or_default();
            for bp in &set.breakpoints {
                let location = set.document.as_ref().map_or_else(
                    || self.mapping.to_generated(bp.line),
                    |document| self.mapping.to_generated_in(document, bp.line),
                );
                if let Some((path, line)) = location {
                    by_file.entry(path).or_default().push((source, bp, line));
                } else {
                    held.push(BreakpointStatus { line:bp.line, moved_to:None, state:BindState::Refused,
                        message:Some("this line has no unique location in the selected complete project — it may be prose, removed code, or an ambiguous contribution".into()) });
                }
            }
        }
        let next_files = by_file.keys().cloned().collect();
        // Sending [] is necessary even after removing the last breakpoint.
        for path in self.sources.files.lock().unwrap().iter() {
            by_file.entry(path.clone()).or_default();
        }
        // Retain every possibly touched file even if delivery fails halfway.
        // A later explicit replacement can clear a partially applied new file.
        self.sources
            .files
            .lock()
            .unwrap()
            .extend(by_file.keys().cloned());
        let mut root_ids = HashMap::new();
        let mut source_ids = HashMap::new();
        for (path, entries) in by_file {
            let payload: Vec<Value> = entries
                .iter()
                .map(|(_, bp, line)| {
                    let mut item = json!({"line":line+1});
                    if self.capabilities.conditional_breakpoints
                        && let Some(value) = &bp.condition
                    {
                        item["condition"] = json!(value);
                    }
                    if self.capabilities.hit_conditional_breakpoints
                        && let Some(value) = &bp.hit_condition
                    {
                        item["hitCondition"] = json!(value);
                    }
                    if self.capabilities.log_points
                        && let Some(value) = &bp.log_message
                    {
                        item["logMessage"] = json!(value);
                    }
                    item
                })
                .collect();
            let body = self
                .adapter
                .request(
                    "setBreakpoints",
                    json!({"source":{"path":path.to_string_lossy()},"breakpoints":payload}),
                )
                .await?;
            let answers = body.get("breakpoints").and_then(Value::as_array);
            for (index, (source, bp, _)) in entries.iter().enumerate() {
                let answer = answers.and_then(|items| items.get(index));
                let moved_to = answer
                    .and_then(|a| a.get("line"))
                    .and_then(Value::as_u64)
                    .map(|line| (line as u32).saturating_sub(1))
                    .and_then(|line| match &sets[*source].document {
                        None => self.mapping.to_document(&path, line),
                        Some(_) if self.mapping.is_identity() => Some(line),
                        Some(document) => self
                            .mapping
                            .source_location(&path, line)
                            .filter(|(origin, _)| crate::mapping::same_file(origin, document))
                            .map(|(_, line)| line),
                    })
                    .filter(|line| *line != bp.line);
                if let Some(id) = answer.and_then(|a| a.get("id")).and_then(Value::as_i64) {
                    if source.is_empty() {
                        root_ids.insert(id, bp.line);
                    } else {
                        source_ids.insert(id, (source.to_string(), bp.line));
                    }
                }
                statuses.get_mut(*source).unwrap().push(BreakpointStatus {
                    line: bp.line,
                    moved_to,
                    state: if answer
                        .and_then(|a| a.get("verified"))
                        .and_then(Value::as_bool)
                        == Some(true)
                    {
                        BindState::Bound
                    } else {
                        BindState::Pending
                    },
                    message: answer
                        .and_then(|a| a.get("message"))
                        .and_then(Value::as_str)
                        .map(str::to_string),
                });
            }
        }
        for held in statuses.values_mut() {
            held.sort_by_key(|bp| bp.line);
        }
        let result = statuses.get(key).cloned().unwrap_or_default();
        if commit_root {
            *self.desired.lock().await = breakpoints.to_vec();
        }
        *self.sources.desired.lock().unwrap() = sets;
        *self.sources.files.lock().unwrap() = next_files;
        *self.bound_ids.lock().unwrap() = root_ids;
        *self.statuses.lock().unwrap() = statuses.remove("").unwrap_or_default();
        *self.sources.ids.lock().unwrap() = source_ids;
        *self.sources.statuses.lock().unwrap() = statuses;
        Ok(result)
    }
}
</storycode:copy>
<a id="f-41b419af990c-impl-Sources"></a>
### `impl-Sources`

<storycode:copy id="f-41b419af990c-impl-Sources">
impl Sources {
    pub(super) fn promote(&self, body: &Value) {
        let Some(bp) = body.get("breakpoint") else {
            return;
        };
        if bp.get("verified").and_then(Value::as_bool) != Some(true) {
            return;
        }
        let Some(id) = bp.get("id").and_then(Value::as_i64) else {
            return;
        };
        let Some((source, line)) = self.ids.lock().unwrap().get(&id).cloned() else {
            return;
        };
        if let Some(status) = self
            .statuses
            .lock()
            .unwrap()
            .get_mut(&source)
            .and_then(|held| held.iter_mut().find(|bp| bp.line == line))
            && !status.state.is_refused()
        {
            status.state = BindState::Bound;
            status.message = None;
        }
    }
}
</storycode:copy>

## Exercise a partial replacement at the real DAP boundary

The software transaction can fail after an adapter has accepted the first file.
This witness starts an ordinary `Session` against a framed stdio Python adapter,
then fails or holds the second request of a two-file replacement. It tests both
a newly introduced contributor and replacement of the accepted root wish.
Cancellation drops the actual waiting Session future after an explicit adapter
checkpoint, rather than simulating an error in a model.

Accepted wishes, statuses and adapter-ID ownership must remain unchanged after
the failed transaction. Every possibly touched file remains in the cleanup set;
a later explicit replacement clears a new partial contributor instead of
retrying its failed wish. An unsolicited promotion for a partial response ID
cannot install a new accepted status. This is a software receipt/desired-set
transaction, not rollback of the adapter's partial effects; retaining addresses
for a later explicit clear is therefore required. The fixture runs no test
program and must not change either source file.

Executed 2026-11-16: `CARGO_TARGET_DIR=target/source-edit-focus cargo test -p
hick-dap --lib session_transaction_faults -- --nocapture` passed both tests,
zero ignored. Each test exercised a new contributor and a root replacement.
This is real Session-to-stdio-DAP fault injection, not a claim about the behavior
of an installed third-party adapter.

<storycode:copy id="source-breakpoints-session-transaction-witness">
#[cfg(all(test, unix))]
mod session_transaction_faults {
    use super::*;

    const ADAPTER: &str = r#"
import json, pathlib, sys, time
trace, fault, held, release, settled = map(pathlib.Path, sys.argv[1:])
seq = 0
sets = 0
mode_seen = None
stage = 0
first_id = None
def emit(value):
    global seq
    seq += 1
    value['seq'] = seq
    body = json.dumps(value, separators=(',', ':')).encode()
    sys.stdout.buffer.write(('Content-Length: %d\r\n\r\n' % len(body)).encode() + body)
    sys.stdout.buffer.flush()
def respond(request, body=None, success=True):
    emit({'type':'response', 'request_seq':request['seq'], 'command':request['command'], 'success':success, 'message':None if success else 'planned second-file replacement failure', 'body':body or {}})
while True:
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            sys.exit(0)
        if line in (b'\r\n', b'\n'):
            break
        key, value = line.decode().split(':', 1)
        headers[key.lower()] = value.strip()
    request = json.loads(sys.stdin.buffer.read(int(headers['content-length'])))
    command = request['command']
    if command == 'initialize':
        respond(request, {'supportsConfigurationDoneRequest':True, 'supportsConditionalBreakpoints':True})
    elif command == 'launch':
        emit({'type':'event', 'event':'initialized', 'body':{}})
        respond(request)
    elif command == 'setBreakpoints':
        sets += 1
        mode = fault.read_text() if fault.exists() else ''
        if mode != mode_seen:
            mode_seen, stage, first_id = mode, 0, None
        if mode:
            stage += 1
        marks = request['arguments']['breakpoints']
        identity = sets * 1000
        with trace.open('a') as output:
            output.write(json.dumps({'mode':mode,'stage':stage,'source':request['arguments']['source']['path'],'breakpoints':marks,'identity':identity}) + '\n')
        if mode and stage == 1:
            first_id = identity
        if mode and stage == 2:
            if mode == 'cancel':
                held.write_text('second request received')
                while not release.exists():
                    time.sleep(0.005)
            fault.unlink()
            if mode == 'fail':
                respond(request, success=False)
            else:
                respond(request, {'breakpoints':[{'id':identity+i,'verified':True,'line':mark['line']} for i,mark in enumerate(marks)]})
            emit({'type':'event','event':'breakpoint','body':{'reason':'changed','breakpoint':{'id':first_id,'verified':True,'line':999}}})
            settled.write_text('partial replacement answered')
        else:
            respond(request, {'breakpoints':[{'id':identity+i,'verified':True,'line':mark['line']} for i,mark in enumerate(marks)]})
    elif command == 'disconnect':
        respond(request)
        break
    else:
        respond(request)
"#;

    fn breakpoint(line: u32, condition: &str) -> Breakpoint {
        Breakpoint {
            line,
            condition: Some(condition.into()),
            hit_condition: None,
            log_message: None,
        }
    }
    fn calls(path: &std::path::Path) -> Vec<Value> {
        std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    async fn wait_file(path: &std::path::Path) {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !path.exists() {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("Fake adapter did not reach the explicit transaction checkpoint");
    }
    fn source_wishes(session: &Session) -> Value {
        Value::Object(
            session
                .sources
                .desired
                .lock()
                .unwrap()
                .iter()
                .map(|(key, set)| {
                    (
                        key.clone(),
                        json!({"document":set.document,"breakpoints":set.breakpoints}),
                    )
                })
                .collect(),
        )
    }
    async fn transaction_fault(mode: &str, replace_root: bool) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root.py");
        let side = directory.path().join("side.py");
        let source = "value = 1\nvalue += 1\nprint(value)\n";
        std::fs::write(&root, source).unwrap();
        std::fs::write(&side, source).unwrap();
        let trace = directory.path().join("requests.jsonl");
        let fault = directory.path().join("fault");
        let held = directory.path().join("held");
        let release = directory.path().join("release");
        let settled = directory.path().join("settled");
        let command = vec![
            "python3".into(),
            "-u".into(),
            "-c".into(),
            ADAPTER.into(),
            trace.to_string_lossy().into_owned(),
            fault.to_string_lossy().into_owned(),
            held.to_string_lossy().into_owned(),
            release.to_string_lossy().into_owned(),
            settled.to_string_lossy().into_owned(),
        ];
        let mapping = Arc::new(Mapping::identity(&root).with_plain_sources(
            directory.path(),
            [
                ("root.py", source.as_bytes()),
                ("side.py", source.as_bytes()),
            ],
        ));
        let original = vec![breakpoint(0, "kept-root")];
        let (session, initial) = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            Session::start(
                Launch {
                    transport: crate::Transport::Stdio,
                    multi_session: false,
                    adapter_extra: json!({}),
                    adapter: command,
                    program: root.clone(),
                    cwd: directory.path().to_path_buf(),
                    extra: json!({}),
                },
                mapping,
                &original,
            ),
        )
        .await
        .expect("Fake adapter initialization stalled")
        .unwrap();
        assert!(
            initial
                .iter()
                .all(|mark| matches!(mark.state, BindState::Bound))
        );
        if replace_root {
            session
                .set_source_breakpoints("side", &side, &[breakpoint(0, "kept-side")])
                .await
                .unwrap();
        }
        let root_before = serde_json::to_value(session.desired.lock().await.clone()).unwrap();
        let sources_before = source_wishes(&session);
        let status_before = serde_json::to_value(session.breakpoint_statuses()).unwrap();
        let source_status_before =
            serde_json::to_value(session.source_breakpoint_statuses()).unwrap();
        let hashes_before = session.source_breakpoint_hashes();
        let root_ids_before = session.bound_ids.lock().unwrap().clone();
        let source_ids_before = session.sources.ids.lock().unwrap().clone();
        let baseline = calls(&trace).len();
        std::fs::write(&fault, mode).unwrap();
        let replacement = vec![breakpoint(1, "failed-wish")];
        {
            let update = async {
                if replace_root {
                    session.set_breakpoints(&replacement).await
                } else {
                    session
                        .set_source_breakpoints("side", &side, &replacement)
                        .await
                }
            };
            if mode == "fail" {
                let error = tokio::time::timeout(std::time::Duration::from_secs(5), update)
                    .await
                    .unwrap()
                    .unwrap_err();
                assert!(error.to_string().contains("planned second-file"), "{error}");
            } else {
                tokio::pin!(update);
                tokio::select! {
                    result=&mut update=>panic!("Update completed before its second request was canceled: {result:?}"),
                    _=wait_file(&held)=>{}
                }
                // Dropping the actual Session future releases the serialization
                // guard without committing either partially delivered result.
            }
        }
        if mode == "cancel" {
            std::fs::write(&release, "release the canceled request").unwrap();
        }
        wait_file(&settled).await;
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        let partial = calls(&trace);
        assert_eq!(
            partial.len(),
            baseline + 2,
            "A failed wish retried without another explicit replacement"
        );
        assert_eq!(partial[baseline]["stage"], 1);
        assert_eq!(partial[baseline + 1]["stage"], 2);
        assert_ne!(partial[baseline]["source"], partial[baseline + 1]["source"]);
        assert_eq!(
            serde_json::to_value(session.desired.lock().await.clone()).unwrap(),
            root_before
        );
        assert_eq!(source_wishes(&session), sources_before);
        assert_eq!(
            serde_json::to_value(session.breakpoint_statuses()).unwrap(),
            status_before
        );
        assert_eq!(
            serde_json::to_value(session.source_breakpoint_statuses()).unwrap(),
            source_status_before
        );
        assert_eq!(session.source_breakpoint_hashes(), hashes_before);
        assert_eq!(*session.bound_ids.lock().unwrap(), root_ids_before);
        assert_eq!(*session.sources.ids.lock().unwrap(), source_ids_before);
        assert!(
            session
                .sources
                .files
                .lock()
                .unwrap()
                .contains(&root.canonicalize().unwrap())
        );
        assert!(
            session
                .sources
                .files
                .lock()
                .unwrap()
                .contains(&side.canonicalize().unwrap()),
            "A possibly touched new file was forgotten after partial delivery"
        );
        session.set_breakpoints(&original).await.unwrap();
        let recovery = calls(&trace);
        let explicit = &recovery[partial.len()..];
        assert_eq!(explicit.len(), 2);
        assert!(
            explicit
                .iter()
                .flat_map(|call| call["breakpoints"].as_array().unwrap())
                .all(|mark| mark["condition"] != "failed-wish")
        );
        let side_call = explicit
            .iter()
            .find(|call| call["source"].as_str() == side.canonicalize().unwrap().to_str())
            .unwrap();
        if replace_root {
            assert_eq!(side_call["breakpoints"][0]["condition"], "kept-side");
        } else {
            assert!(
                side_call["breakpoints"].as_array().unwrap().is_empty(),
                "The partially applied source was not explicitly cleared"
            );
            assert!(!session.sources.desired.lock().unwrap().contains_key("side"));
        }
        // Run-to delivery is temporary, even though it replaces the same root
        // file. It must not overwrite the user's committed wish set.
        let mut temporary = original.clone();
        temporary.push(breakpoint(2, "temporary-run-to"));
        session.apply_breakpoints(&temporary).await.unwrap();
        assert_eq!(
            serde_json::to_value(&*session.desired.lock().await).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        session.apply_breakpoints(&original).await.unwrap();
        assert!(
            !source_wishes(&session)
                .to_string()
                .contains("temporary-run-to")
        );
        assert_eq!(std::fs::read_to_string(&root).unwrap(), source);
        assert_eq!(std::fs::read_to_string(&side).unwrap(), source);
        session.shutdown().await;
    }

    #[tokio::test]
    async fn actual_partial_set_failure_preserves_accepted_wishes_and_explicitly_clears_touched_files()
     {
        transaction_fault("fail", false).await;
        transaction_fault("fail", true).await;
    }
    #[tokio::test]
    async fn canceled_partial_set_preserves_accepted_wishes_and_explicitly_clears_touched_files() {
        transaction_fault("cancel", false).await;
        transaction_fault("cancel", true).await;
    }
}
</storycode:copy>

### Assemble `crates/hick-dap/src/source_breakpoints.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/source_breakpoints.rs"><storycode:paste select="#f-41b419af990c-SourceSet"/><storycode:paste select="#f-41b419af990c-Sources"/><storycode:paste select="#f-41b419af990c-impl-Session"/><storycode:paste select="#f-41b419af990c-impl-Sources"/><storycode:paste select="#source-breakpoints-session-transaction-witness"/></storycode:file>

</storycode:doc>
