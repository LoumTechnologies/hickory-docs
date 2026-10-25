use super::*;
use sha2::{Digest, Sha256};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn candidates(root: &Path, scope: &str, file: &str) -> Vec<PathBuf> {
    let path = if let Ok(url) = reqwest::Url::parse(file) {
        if url.scheme() == "file" {
            url.to_file_path().ok()
        } else if ["http", "https"].contains(&url.scheme()) {
            Some(PathBuf::from(url.path().trim_start_matches('/')))
        } else {
            None
        }
    } else {
        Some(PathBuf::from(file))
    };
    let Some(path) = path else { return vec![] };
    let list = if path.is_absolute() {
        vec![path]
    } else {
        vec![root.join(&path), root.join(scope).join(&path)]
    };
    list.into_iter()
        .filter_map(|p| p.canonicalize().ok())
        .filter(|p| p.starts_with(root))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn matched(root: &Path, identity: &Identity, file: &str, content: Option<&str>) -> Option<String> {
    let paths: Vec<_> = candidates(root, &identity.scope, file)
        .into_iter()
        .filter_map(|p| {
            let rel = relative(root, &p);
            let expected = identity.source_hashes.get(&rel)?;
            let bytes = std::fs::read(p).ok()?;
            if hash(&bytes) != *expected || content.is_some_and(|c| c.as_bytes() != bytes) {
                return None;
            }
            Some(rel)
        })
        .collect();
    (paths.len() == 1).then(|| paths[0].clone())
}
fn browser_frame(
    root: &Path,
    identity: &Identity,
    frame: &Frame,
    script: &browser::Script,
) -> Option<Frame> {
    if let Some(map) = &script.map {
        let map = sourcemap::SourceMap::from_slice(&serde_json::to_vec(map).ok()?).ok()?;
        let token = map.lookup_token(
            frame.line?.checked_sub(1)?,
            frame.column.unwrap_or(1).saturating_sub(1),
        )?;
        let source = token.get_source()?;
        let content = map.get_source_contents(token.get_src_id())?;
        let base = if script
            .map_url
            .as_deref()
            .is_some_and(|u| !u.starts_with("data:"))
        {
            reqwest::Url::parse(&script.url)
                .ok()?
                .join(script.map_url.as_deref()?)
                .ok()?
        } else {
            reqwest::Url::parse(&script.url).ok()?
        };
        let source_url = base.join(source).ok().map(|u| u.to_string());
        let path = source_url
            .as_deref()
            .and_then(|p| matched(root, identity, p, Some(content)))
            .or_else(|| matched(root, identity, source, Some(content)))?;
        let mut mapped = frame.clone();
        mapped.file = Some(path);
        mapped.line = Some(token.get_src_line() + 1);
        mapped.column = Some(token.get_src_col() + 1);
        return Some(mapped);
    }
    let path = matched(root, identity, &script.url, Some(script.source.as_deref()?))?;
    let mut mapped = frame.clone();
    mapped.file = Some(path);
    Some(mapped)
}
pub(super) fn bind(
    root: &Path,
    run: &mut Record,
    scripts: Option<&BTreeMap<String, browser::Script>>,
) -> Result<()> {
    let mut locations = BTreeMap::<String, Vec<u32>>::new();
    if let Some(report) = &mut run.report {
        for thread in &mut report.threads {
            for row in &mut thread.rows {
                let resolved = if let Some(scripts) = scripts {
                    row.frame
                        .script_id
                        .as_ref()
                        .and_then(|id| scripts.get(id))
                        .and_then(|script| browser_frame(root, &run.identity, &row.frame, script))
                } else {
                    row.frame
                        .file
                        .as_deref()
                        .and_then(|p| matched(root, &run.identity, p, None))
                        .map(|path| {
                            let mut f = row.frame.clone();
                            f.file = Some(path);
                            f
                        })
                };
                if let Some(frame) = resolved {
                    row.frame = frame;
                }
                if let (Some(file), Some(line)) = (&row.frame.file, row.frame.line)
                    && run.identity.source_hashes.contains_key(file)
                    && matched(root, &run.identity, file, None).as_ref() == Some(file)
                {
                    locations.entry(file.clone()).or_default().push(line);
                }
            }
            thread
                .rows
                .sort_by(|a, b| b.self_weight.total_cmp(&a.self_weight));
        }
    }
    run.mappings = crate::source_mapping::capture(root, &run.identity.source_hashes, &locations)?;
    let paths: std::collections::BTreeSet<_> = locations
        .keys()
        .cloned()
        .chain(run.mappings.values().flat_map(|m| {
            m.values()
                .filter_map(|v| v["path"].as_str().map(str::to_string))
        }))
        .collect();
    for file in paths {
        let source = std::fs::read_to_string(safe_path(root, &file)?)?;
        if run.identity.source_hashes.get(&file) == Some(&hash(source.as_bytes())) {
            run.sources.insert(file, source);
        }
    }
    Ok(())
}
pub(super) fn python(
    root: &Path,
    path: &Path,
    record: &mut Record,
    run: &hick_observe::Run,
) -> Result<()> {
    let mut functions = BTreeMap::<(String, u32, String), Value>::new();
    let mut elapsed = 0f64;
    for event in &run.events {
        if let hick_observe::Observation::Profile {
            metric,
            value,
            function: Some(function),
            interval,
            ..
        } = &event.observation
        {
            elapsed = elapsed.max(interval.end.value - interval.start.value);
            let (file, line) = if let Some(location) = &event.emission {
                let source = run.sources.iter().find(|s| s.id == location.source);
                let offset = location.bytes.as_ref().map(|b| b.start);
                match (source, offset) {
                    (Some(s), Some(offset)) => (
                        relative(root, path),
                        s.bytes[..offset].iter().filter(|b| **b == b'\n').count() as u32 + 1,
                    ),
                    _ => (function.file.clone(), function.line),
                }
            } else {
                (function.file.clone(), function.line)
            };
            let key = (file.clone(), line, function.name.clone());
            let row=functions.entry(key).or_insert_with(||json!({"name":function.name,"file":file,"line":line,"total_calls":function.total_calls,"primitive_calls":function.primitive_calls,"self_seconds":0.,"cumulative_seconds":0.}));
            row[metric] = json!(value);
        }
    }
    let raw=json!({"elapsed_seconds":elapsed,"scope":"main-thread cProfile","functions":functions.into_values().collect::<Vec<_>>()}).to_string();
    record.report = Some(hick_profile::parse("cprofile-json", &raw)?);
    let detail: Value = run
        .detail
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(Value::Null);
    record.identity.collector = "cpython-cprofile-v1".into();
    record.identity.version = detail["provider_version"]
        .as_str()
        .unwrap_or("unknown")
        .into();
    record.identity.configuration =
        "confined saved Python; main-thread cProfile; output hooks active".into();
    record.environment =
        json!({"runtime_run":run.id,"provider":detail,"missing_artifacts":run.missing_artifacts});
    record.status = match run.status {
        hick_observe::Status::Completed => "completed",
        _ => "partial",
    }
    .into();
    record.source_changed = snapshot(root)? != record.identity.source_hashes;
    bind(root, record, None)?;
    Ok(())
}
