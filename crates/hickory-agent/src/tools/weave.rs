use super::*;

/// Weave (no execution) one document source, mirroring `hick weave`:
/// cached transcripts are used when the project has a transcript cache.
pub(super) async fn weave_source(
    doc_path: &Path,
    doc_name: &str,
    source: &str,
    params: &[(String, String)],
) -> Result<WeaveState> {
    // Fail early with the parse error rather than a pipeline error.
    hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error in {doc_name}: {e}"))?;

    let project_dir = doc_path.parent().unwrap_or(Path::new("."));
    let cc =
        hick_literate::cache::CacheConfig::new(project_dir, hick_literate::cache::CacheMode::Reuse);
    let cache_config = cc.cache_dir.is_dir().then_some(&cc);

    let declared = declared_sources(doc_path, Some(source))?;
    let sources: Vec<_> = if declared.is_empty() {
        vec![(doc_name, source)]
    } else {
        declared
            .iter()
            .map(|(path, text)| (path.to_str().unwrap(), text.as_str()))
            .collect()
    };
    let result = hick_literate::run_pipeline_weave(&sources, params, cache_config).await?;

    let mut files = HashMap::new();
    for (path, content) in &result.files {
        if let FileContent::Text(s) = content {
            files.insert(path.clone(), s.clone());
        }
    }
    let provenance = result
        .provenance_maps
        .iter()
        .map(|(p, m)| (p.clone(), hickory_lineage::from_provenance_map(m)))
        .collect();
    Ok(WeaveState { files, provenance })
}

pub(super) fn declared_sources(
    doc_path: &Path,
    primary: Option<&str>,
) -> Result<std::collections::BTreeMap<PathBuf, String>> {
    let source = primary
        .map(str::to_owned)
        .map(Ok)
        .unwrap_or_else(|| std::fs::read_to_string(doc_path))?;
    if hick_lang::parse(&source)?.find_tags("project").is_empty() {
        return Ok(Default::default());
    }
    let root = doc_path.parent().unwrap_or(Path::new("."));
    let entry = doc_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Project requires a UTF-8 entry"))?;
    let sources = hick_lang::load_project_sources(entry, &|name| {
        if name == entry {
            return Ok(source.clone());
        }
        let path = root.join(name);
        if !std::fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .is_file()
            || !path
                .canonicalize()
                .map_err(|e| e.to_string())?
                .starts_with(root.canonicalize().map_err(|e| e.to_string())?)
        {
            return Err(format!("Project source escapes its root: {name}"));
        }
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    })
    .map_err(anyhow::Error::msg)?;
    Ok(sources
        .into_iter()
        .map(|(name, text)| (root.join(name), text))
        .collect())
}
