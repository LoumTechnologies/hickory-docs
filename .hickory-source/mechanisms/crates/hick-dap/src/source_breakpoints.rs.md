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

<storycode:copy id="f-41b419af990c-impl-Session">
impl Session {
    pub async fn set_breakpoints(
        &self,
        breakpoints: &[Breakpoint],
    ) -> Result<Vec<BreakpointStatus>> {
        *self.desired.lock().await = breakpoints.to_vec();
        self.apply_breakpoints(breakpoints).await
    }

    pub(super) async fn apply_breakpoints(
        &self,
        breakpoints: &[Breakpoint],
    ) -> Result<Vec<BreakpointStatus>> {
        self.apply_for("", None, breakpoints).await
    }

    pub async fn set_source_breakpoints(
        &self,
        source: &str,
        document: &std::path::Path,
        breakpoints: &[Breakpoint],
    ) -> Result<Vec<BreakpointStatus>> {
        self.apply_for(source, Some(document.to_path_buf()), breakpoints)
            .await
    }

    pub fn source_breakpoint_statuses(&self) -> HashMap<String, Vec<BreakpointStatus>> {
        self.sources.statuses.lock().unwrap().clone()
    }

    async fn apply_for(
        &self,
        key: &str,
        document: Option<PathBuf>,
        breakpoints: &[Breakpoint],
    ) -> Result<Vec<BreakpointStatus>> {
        let _serial = self.sources.serial.lock().await;
        let sets = {
            let mut desired = self.sources.desired.lock().unwrap();
            desired.insert(
                key.to_string(),
                SourceSet {
                    document,
                    breakpoints: breakpoints.to_vec(),
                },
            );
            desired.clone()
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

### Assemble `crates/hick-dap/src/source_breakpoints.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/source_breakpoints.rs"><storycode:paste select="#f-41b419af990c-SourceSet"/><storycode:paste select="#f-41b419af990c-Sources"/><storycode:paste select="#f-41b419af990c-impl-Session"/><storycode:paste select="#f-41b419af990c-impl-Sources"/></storycode:file>

</storycode:doc>
