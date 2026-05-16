//! `hick pipeline` subcommand: inspect the pipeline DAG and file ownership.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::{HickConfig, find_config};

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// `hick pipeline show` — print the full pipeline DAG with file ownership.
///
/// For each `.hick` source listed in `_hick.yml`, shows which output files it
/// owns and what named paste slots are available.
pub fn pipeline_show(project_dir: &Path) -> Result<()> {
    let (config_path, config) = load_config(project_dir)?;
    let config_dir = config_path.parent().unwrap_or(project_dir);

    println!("Pipeline: {}", config_path.display());
    println!();

    if config.files.is_empty() {
        println!("  (no .hick files listed — add entries under `files:` in _hick.yml)");
        return Ok(());
    }

    let entries = scan_pipeline(config_dir, &config)?;

    for (hick_rel, owned) in &entries {
        println!("  {hick_rel}");
        if owned.is_empty() {
            println!("    (no <hick:file> declarations)");
        }
        for of in owned {
            if of.paste_slots.is_empty() {
                println!("    -> {}", of.output_path);
            } else {
                let slots = of.paste_slots.join(", ");
                println!("    -> {}  [slots: {}]", of.output_path, slots);
            }
        }
    }

    let total_files: usize = entries.values().map(|v| v.len()).sum();
    let total_slots: usize = entries
        .values()
        .flat_map(|v| v.iter())
        .map(|f| f.paste_slots.len())
        .sum();
    println!();
    println!(
        "  {} hick source(s), {} output file(s), {} paste slot(s)",
        entries.len(),
        total_files,
        total_slots
    );

    Ok(())
}

/// `hick pipeline status` — show which output files are pipeline-owned vs untracked.
///
/// Pipeline-owned files declared in `.hick` sources are marked with their
/// ownership source. Files present on disk that are not declared by any
/// pipeline source are listed as untracked.
pub fn pipeline_status(project_dir: &Path) -> Result<()> {
    let (config_path, config) = load_config(project_dir)?;
    let config_dir = config_path.parent().unwrap_or(project_dir);

    println!("Pipeline root: {}", config_path.display());
    println!();

    let entries = scan_pipeline(config_dir, &config)?;

    // Build flat map: output_path -> hick_source
    let mut owned: HashMap<String, String> = HashMap::new();
    for (hick_rel, files) in &entries {
        for f in files {
            owned.insert(f.output_path.clone(), hick_rel.clone());
        }
    }

    // Walk project directory for actual files
    let disk_files = walk_project_files(config_dir);

    let mut pipeline_present: Vec<(&str, &str)> = Vec::new();
    let mut pipeline_missing: Vec<(&str, &str)> = Vec::new();
    let mut untracked: Vec<&str> = Vec::new();

    for disk_path in &disk_files {
        if owned.contains_key(disk_path.as_str()) {
            let src = owned.get(disk_path.as_str()).unwrap();
            pipeline_present.push((disk_path.as_str(), src.as_str()));
        } else {
            untracked.push(disk_path.as_str());
        }
    }

    for (output_path, src) in &owned {
        let exists = config_dir.join(output_path).is_file();
        if !exists && !pipeline_present.iter().any(|(p, _)| *p == output_path.as_str()) {
            pipeline_missing.push((output_path.as_str(), src.as_str()));
        }
    }

    if !pipeline_present.is_empty() {
        println!("Pipeline-owned (on disk):");
        for (path, src) in &pipeline_present {
            println!("  [owned]     {path}  (from {src})");
        }
        println!();
    }

    if !pipeline_missing.is_empty() {
        println!("Pipeline-owned (not yet generated — run `hick run`):");
        for (path, src) in &pipeline_missing {
            println!("  [missing]   {path}  (from {src})");
        }
        println!();
    }

    if !untracked.is_empty() {
        println!("Untracked (not owned by any pipeline source):");
        for path in &untracked {
            println!("  [untracked] {path}");
        }
        println!();
    }

    println!(
        "  {} owned, {} missing, {} untracked",
        pipeline_present.len(),
        pipeline_missing.len(),
        untracked.len()
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// One output file declared by a `<hick:file>` element.
#[derive(Debug, Clone)]
pub struct OwnedFile {
    /// Output path declared by `path="…"`.
    pub output_path: String,
    /// Named `<hick:paste name="…"/>` slots inside this file element.
    pub paste_slots: Vec<String>,
}

/// Load `_hick.yml` from `project_dir` (or by walking up to a `.git` boundary).
fn load_config(project_dir: &Path) -> Result<(PathBuf, HickConfig)> {
    let config_path = find_config(project_dir)
        .or_else(|| {
            let p = project_dir.join("_hick.yml");
            p.is_file().then_some(p)
        })
        .with_context(|| {
            format!(
                "no _hick.yml found in {} or any parent directory (up to .git boundary)",
                project_dir.display()
            )
        })?;
    let config = HickConfig::load(&config_path)?;
    Ok((config_path, config))
}

/// Scan each `.hick` file listed in `config` and return an ordered map of
/// hick_relative_path → Vec<OwnedFile>.
fn scan_pipeline(
    config_dir: &Path,
    config: &HickConfig,
) -> Result<indexmap::IndexMap<String, Vec<OwnedFile>>> {
    let resolved = config.resolve_files(config_dir)?;
    let mut result = indexmap::IndexMap::new();

    for abs_path in resolved {
        let rel = abs_path
            .strip_prefix(config_dir)
            .unwrap_or(&abs_path)
            .to_string_lossy()
            .replace('\\', "/");

        let content = std::fs::read_to_string(&abs_path)
            .with_context(|| format!("failed to read {}", abs_path.display()))?;

        let owned_files = extract_owned_files(&content);
        result.insert(rel, owned_files);
    }

    Ok(result)
}

/// Parse a `.hick` source and collect `<hick:file>` declarations with their paste slots.
fn extract_owned_files(source: &str) -> Vec<OwnedFile> {
    let Ok(doc) = hick_lang::parse(source) else {
        return Vec::new();
    };

    doc.find_tags("file")
        .into_iter()
        .filter_map(|tag: &hick_lang::HickTag| {
            let output_path = tag.get_attribute("path")?.to_string();
            let paste_slots: Vec<String> = tag
                .child_tags()
                .filter(|t: &&hick_lang::HickTag| t.name == "paste")
                .filter_map(|t: &hick_lang::HickTag| t.get_attribute("name"))
                .map(|s: &str| s.to_string())
                .collect();
            Some(OwnedFile {
                output_path,
                paste_slots,
            })
        })
        .collect()
}

/// Walk project files, skipping internal/build directories.
///
/// Returns relative paths (forward-slash separated).
fn walk_project_files(project_dir: &Path) -> Vec<String> {
    let skip_dirs = [".git", "target", ".hick", "node_modules"];

    let mut files = Vec::new();

    let walker = walkdir::WalkDir::new(project_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            if e.file_type().is_dir() {
                return !skip_dirs.contains(&name.as_ref());
            }
            true
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(project_dir)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        // Skip the pipeline config itself and .hick source files
        if rel == "_hick.yml" || rel.ends_with(".hick") {
            continue;
        }
        files.push(rel);
    }

    files.sort();
    files
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_project(dir: &Path, hick_files: &[(&str, &str)]) {
        fs::write(
            dir.join("_hick.yml"),
            format!(
                "files:\n{}\n",
                hick_files
                    .iter()
                    .map(|(name, _)| format!("  - {name}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        )
        .unwrap();
        for (name, content) in hick_files {
            fs::write(dir.join(name), content).unwrap();
        }
    }

    fn hick_doc(body: &str) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
             {body}\n\
             </hick:doc>\n"
        )
    }

    #[test]
    fn extract_owned_files_empty_doc() {
        let src = hick_doc("");
        let files = extract_owned_files(&src);
        assert!(files.is_empty());
    }

    #[test]
    fn extract_owned_files_single() {
        let src = hick_doc(r#"<hick:file path="src/main.rs">fn main() {}</hick:file>"#);
        let files = extract_owned_files(&src);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].output_path, "src/main.rs");
        assert!(files[0].paste_slots.is_empty());
    }

    #[test]
    fn extract_owned_files_with_paste_slots() {
        let src = hick_doc(
            r#"<hick:file path="out.py">
def foo(): pass
<hick:paste name="after-fn-foo"/>
</hick:file>"#,
        );
        let files = extract_owned_files(&src);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].paste_slots, vec!["after-fn-foo"]);
    }

    #[test]
    fn extract_owned_files_multiple() {
        let src = hick_doc(
            r#"<hick:file path="a.py">pass</hick:file>
<hick:file path="b.py">pass</hick:file>"#,
        );
        let files = extract_owned_files(&src);
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn pipeline_show_no_hick_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("_hick.yml"), "files: []\n").unwrap();
        pipeline_show(dir.path()).unwrap();
    }

    #[test]
    fn pipeline_show_with_files() {
        let dir = tempfile::tempdir().unwrap();
        let src = hick_doc(r#"<hick:file path="src/lib.rs">fn lib() {}</hick:file>"#);
        setup_project(dir.path(), &[("main.hick", &src)]);
        pipeline_show(dir.path()).unwrap();
    }

    #[test]
    fn pipeline_status_owned_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let src = hick_doc(r#"<hick:file path="out.txt">hello</hick:file>"#);
        setup_project(dir.path(), &[("main.hick", &src)]);
        // Write the output file so it appears as present
        fs::write(dir.path().join("out.txt"), "hello").unwrap();
        pipeline_status(dir.path()).unwrap();
    }

    #[test]
    fn pipeline_status_missing_output() {
        let dir = tempfile::tempdir().unwrap();
        let src = hick_doc(r#"<hick:file path="out.txt">hello</hick:file>"#);
        setup_project(dir.path(), &[("main.hick", &src)]);
        // out.txt not on disk
        pipeline_status(dir.path()).unwrap();
    }

    #[test]
    fn pipeline_status_untracked_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("_hick.yml"), "files: []\n").unwrap();
        fs::write(dir.path().join("untracked.txt"), "data").unwrap();
        pipeline_status(dir.path()).unwrap();
    }

    #[test]
    fn walk_project_files_skips_git_and_target() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();
        fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main").unwrap();
        fs::write(dir.path().join("target/foo.so"), "binary").unwrap();
        fs::write(dir.path().join("src.txt"), "source").unwrap();

        let files = walk_project_files(dir.path());
        assert!(files.contains(&"src.txt".to_string()));
        assert!(!files.iter().any(|f| f.contains(".git")));
        assert!(!files.iter().any(|f| f.contains("target")));
    }

    #[test]
    fn walk_project_files_skips_hick_yml_and_hick_sources() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("_hick.yml"), "files: []").unwrap();
        fs::write(dir.path().join("main.hick"), "<hick:doc/>").unwrap();
        fs::write(dir.path().join("out.txt"), "data").unwrap();

        let files = walk_project_files(dir.path());
        assert!(files.contains(&"out.txt".to_string()));
        assert!(!files.contains(&"_hick.yml".to_string()));
        assert!(!files.contains(&"main.hick".to_string()));
    }
}
