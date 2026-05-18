//! File I/O for hick files and generated output.

use std::collections::HashMap;
use std::path::Path;

use hick_flow::provenance::ProvenanceMap;

use crate::{HickEditFormat, HickEditService, SourceEdit};

/// Load all .hick files from a project directory.
/// Uses _hick.yml if present, otherwise discovers *.hick recursively.
pub fn load_hick_sources(project_dir: &Path) -> anyhow::Result<HashMap<String, String>> {
    let mut sources = HashMap::new();

    let config_path = project_dir.join("_hick.yml");
    let patterns: Vec<String> = if config_path.is_file() {
        let config: HickConfig = serde_yaml::from_str(&std::fs::read_to_string(&config_path)?)?;
        config.files.unwrap_or_else(|| vec!["*.hick".to_string()])
    } else {
        vec!["**/*.hick".to_string()]
    };

    let hick_state_dir = project_dir.join(".hick");

    for pattern in patterns {
        let full_pattern = project_dir.join(&pattern);
        let pattern_str = full_pattern.to_string_lossy();
        if let Ok(entries) = glob::glob(&pattern_str) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.is_file() && !entry.starts_with(&hick_state_dir) {
                    let rel = entry
                        .strip_prefix(project_dir)
                        .unwrap_or(&entry)
                        .to_string_lossy()
                        .replace('\\', "/");
                    let content = std::fs::read_to_string(&entry)?;
                    sources.insert(rel, content);
                }
            }
        }
    }

    if sources.is_empty() {
        for entry in walkdir::WalkDir::new(project_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|e| e == "hick")
                && !path.starts_with(&hick_state_dir)
            {
                let rel = path
                    .strip_prefix(project_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let content = std::fs::read_to_string(path)?;
                sources.insert(rel, content);
            }
        }
    }

    Ok(sources)
}

#[derive(serde::Deserialize)]
struct HickConfig {
    #[serde(default)]
    files: Option<Vec<String>>,
}

/// Write hick sources to disk.
pub fn write_hick_sources(
    project_dir: &Path,
    sources: &HashMap<String, String>,
) -> anyhow::Result<()> {
    for (rel_path, content) in sources {
        let full = project_dir.join(rel_path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full, content)?;
    }
    Ok(())
}

/// Write generated pipeline output to disk.
pub fn write_generated_files(
    project_dir: &Path,
    output_dir: Option<&str>,
    files: &HashMap<String, String>,
) -> anyhow::Result<()> {
    let base = match output_dir {
        Some(d) => project_dir.join(d),
        None => project_dir.to_path_buf(),
    };
    for (rel_path, content) in files {
        let full = base.join(rel_path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full, content)?;
    }
    Ok(())
}

/// Write provenance maps as JSON to `.hick/provenance/`.
pub fn write_provenance_maps(
    project_dir: &Path,
    provenance_maps: &HashMap<String, ProvenanceMap>,
) -> anyhow::Result<()> {
    let prov_dir = project_dir.join(".hick").join("provenance");
    std::fs::create_dir_all(&prov_dir)?;

    for (file_path, prov_map) in provenance_maps {
        let safe_name = file_path.replace('/', "__");
        let json_path = prov_dir.join(format!("{safe_name}.json"));
        let json = serde_json::to_string_pretty(prov_map)?;
        std::fs::write(&json_path, json)?;
    }

    Ok(())
}

/// Apply edits to hick files and write to disk.
pub fn apply_and_write_edits(
    project_dir: &Path,
    sources: &mut HashMap<String, String>,
    edits: &[SourceEdit],
) -> anyhow::Result<()> {
    let updated = <HickEditService as HickEditFormat>::apply_edits(sources, edits);
    *sources = updated;
    write_hick_sources(project_dir, sources)
}
