<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A `hick:file` Is A Generated File However Deeply It Is Nested

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a `<hick:file path="…">` anywhere in a document — at the top level, or
inside a `hick:exec`, a `hick:ingested`, or anything else — when the document
is read by the language server, the debugger, or a `<hick:capture>`, then that
block is one of the document's generated files: it has an embedded language
server, a breakpoint gutter, and a place in the debugger's weave. The rule is
one rule, stated once: **the file blocks the engine's own weave writes**
(`all_tags`), never a subset of them.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-file-block-is-a-file-however-deeply-it-is-nested.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-0281c6fee9f5"></a>
## `crates/hick-literate/src/pipeline.rs`

Related contracts: [A `hick:file` Is A Generated File However Deeply It Is Nested](contract.md).

<a id="f-0281c6fee9f5-pipeline_show"></a>
### `pipeline_show`

`hick pipeline` subcommand: inspect the pipeline DAG and file ownership.

<storycode:copy id="f-0281c6fee9f5-pipeline_show">
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
/// For each `.md` source listed in `_hick.yml`, shows which output files it
/// owns and what named paste slots are available.
pub fn pipeline_show(project_dir: &Path) -> Result<()> {
    let (config_path, config) = load_config(project_dir)?;
    let config_dir = config_path.parent().unwrap_or(project_dir);

    println!("Pipeline: {}", config_path.display());
    println!();

    if config.files.is_empty() {
        println!("  (no .md files listed — add entries under `files:` in _hick.yml)");
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

</storycode:copy>
<a id="f-0281c6fee9f5-pipeline_status"></a>
### `pipeline_status`

`hick pipeline status` — show which output files are pipeline-owned vs untracked.
Pipeline-owned files declared in `.md` sources are marked with their
ownership source. Files present on disk that are not declared by any
pipeline source are listed as untracked.

<storycode:copy id="f-0281c6fee9f5-pipeline_status">
/// `hick pipeline status` — show which output files are pipeline-owned vs untracked.
///
/// Pipeline-owned files declared in `.md` sources are marked with their
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
        if !exists
            && !pipeline_present
                .iter()
                .any(|(p, _)| *p == output_path.as_str())
        {
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

</storycode:copy>
<a id="f-0281c6fee9f5-OwnedFile"></a>
### `OwnedFile`

Internal helpers
One output file declared by a `<hick:file>` element.

<storycode:copy id="f-0281c6fee9f5-OwnedFile">
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

</storycode:copy>
<a id="f-0281c6fee9f5-load_config"></a>
### `load_config`

Load `_hick.yml` from `project_dir` (or by walking up to a `.git` boundary).

<storycode:copy id="f-0281c6fee9f5-load_config">
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

</storycode:copy>
<a id="f-0281c6fee9f5-scan_pipeline"></a>
### `scan_pipeline`

Scan each `.md` file listed in `config` and return an ordered map of
hick_relative_path → Vec<OwnedFile>.

<storycode:copy id="f-0281c6fee9f5-scan_pipeline">
/// Scan each `.md` file listed in `config` and return an ordered map of
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

</storycode:copy>
<a id="f-0281c6fee9f5-extract_owned_files"></a>
### `extract_owned_files`

Parse a `.md` source and collect `<hick:file>` declarations with their paste slots.

<storycode:copy id="f-0281c6fee9f5-extract_owned_files">
/// Parse a `.md` source and collect `<hick:file>` declarations with their paste slots.
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

</storycode:copy>
<a id="f-0281c6fee9f5-walk_project_files"></a>
### `walk_project_files`

Walk project files, skipping internal/build directories.
Returns relative paths (forward-slash separated).

<storycode:copy id="f-0281c6fee9f5-walk_project_files">
/// Walk project files, skipping internal/build directories.
///
/// Returns relative paths (forward-slash separated).
fn walk_project_files(project_dir: &Path) -> Vec<String> {
    let skip_dirs = [".git", "target", ".md", "node_modules"];

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
        // Skip the pipeline config itself and .md source files
        if rel == "_hick.yml" || rel.ends_with(".md") {
            continue;
        }
        files.push(rel);
    }

    files.sort();
    files
}

</storycode:copy>
<a id="f-0281c6fee9f5-tests"></a>
### `tests`

<storycode:copy id="f-0281c6fee9f5-tests">
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
        setup_project(dir.path(), &[("main.md", &src)]);
        pipeline_show(dir.path()).unwrap();
    }

    #[test]
    fn pipeline_status_owned_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let src = hick_doc(r#"<hick:file path="out.txt">hello</hick:file>"#);
        setup_project(dir.path(), &[("main.md", &src)]);
        // Write the output file so it appears as present
        fs::write(dir.path().join("out.txt"), "hello").unwrap();
        pipeline_status(dir.path()).unwrap();
    }

    #[test]
    fn pipeline_status_missing_output() {
        let dir = tempfile::tempdir().unwrap();
        let src = hick_doc(r#"<hick:file path="out.txt">hello</hick:file>"#);
        setup_project(dir.path(), &[("main.md", &src)]);
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
        fs::write(dir.path().join("main.md"), "<hick:doc/>").unwrap();
        fs::write(dir.path().join("out.txt"), "data").unwrap();

        let files = walk_project_files(dir.path());
        assert!(files.contains(&"out.txt".to_string()));
        assert!(!files.contains(&"_hick.yml".to_string()));
        assert!(!files.contains(&"main.md".to_string()));
    }
}
</storycode:copy>

### Assemble `crates/hick-literate/src/pipeline.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/pipeline.rs"><storycode:paste select="#f-0281c6fee9f5-pipeline_show"/><storycode:paste select="#f-0281c6fee9f5-pipeline_status"/><storycode:paste select="#f-0281c6fee9f5-OwnedFile"/><storycode:paste select="#f-0281c6fee9f5-load_config"/><storycode:paste select="#f-0281c6fee9f5-scan_pipeline"/><storycode:paste select="#f-0281c6fee9f5-extract_owned_files"/><storycode:paste select="#f-0281c6fee9f5-walk_project_files"/><storycode:paste select="#f-0281c6fee9f5-tests"/></storycode:file>

<a id="output-a254cbbf439c"></a>
## `crates/hick-lsp/src/virtual_file.rs`

Related contracts: [A `hick:file` Is A Generated File However Deeply It Is Nested](contract.md).

<a id="f-a254cbbf439c-VirtualSegment"></a>
### `VirtualSegment`

Virtual file generation from `hick:file` blocks with copy/paste resolution.

<storycode:copy id="f-a254cbbf439c-VirtualSegment">
//! Virtual file generation from `hick:file` blocks with copy/paste resolution.

use std::collections::HashMap;

use hick_lang::{HickDocument, HickNode, HickTag, SourceSpan};

use crate::lang_detect;

/// A segment of a virtual file, tracking its origin in the .md source.
#[derive(Debug, Clone)]
pub struct VirtualSegment {
    /// The text content of this segment.
    pub text: String,
    /// Source span in the .md file where this text originated.
    /// None for synthesized content (e.g., newlines between segments).
    pub source_span: Option<SourceSpan>,
    /// 1-based line in the .md file where this segment's tag starts.
    pub source_line: usize,
    /// 0-based column for dedenting.
    pub source_column: usize,
}

</storycode:copy>
<a id="f-a254cbbf439c-VirtualFile"></a>
### `VirtualFile`

A virtual file generated from a hick:file block.

<storycode:copy id="f-a254cbbf439c-VirtualFile">
/// A virtual file generated from a hick:file block.
#[derive(Debug, Clone)]
pub struct VirtualFile {
    /// Output file path (from the path="" attribute).
    pub path: String,
    /// Language identifier for this file.
    pub language_id: Option<&'static str>,
    /// Ordered segments that compose this file's content.
    pub segments: Vec<VirtualSegment>,
}

</storycode:copy>
<a id="f-a254cbbf439c-impl-VirtualFile"></a>
### `impl-VirtualFile`

<storycode:copy id="f-a254cbbf439c-impl-VirtualFile">
impl VirtualFile {
    /// Get the full text content of the virtual file.
    pub fn content(&self) -> String {
        let mut out = String::new();
        for seg in &self.segments {
            out.push_str(&seg.text);
        }
        out
    }

    /// The bytes to **write** for this file, as the engine's weave writes
    /// them: the content without the newline that opens it.
    ///
    /// [`VirtualFile::content`] begins with the newline after the
    /// `<hick:file>` tag, so line 0 is the tag's line and the rest lines up
    /// with the document. A language server, which only ever reads, wants
    /// exactly that. Anything that writes the file down wants the other
    /// thing, because a leading blank line moves whatever must be at byte 0
    /// — a `#!` line, a UTF-8 BOM, an XML declaration. `dotnet new` writes
    /// its `.csproj` with a BOM, and a scaffold a document had ingested
    /// could not be built by the debugger for exactly this reason:
    /// `MSB4025: Data at the root level is invalid. Line 2, position 1.`
    ///
    /// Pair it with [`crate::position_map::PositionMap::without_first_line`],
    /// which is the same removal on the other side.
    pub fn written_content(&self) -> String {
        let content = self.content();
        content
            .strip_prefix('\n')
            .map(str::to_string)
            .unwrap_or(content)
    }

    /// Total number of lines.
    pub fn line_count(&self) -> usize {
        let content = self.content();
        if content.is_empty() {
            return 0;
        }
        content.lines().count()
    }
}

</storycode:copy>
<a id="f-a254cbbf439c-build_virtual_files"></a>
### `build_virtual_files`

Build virtual files from all `hick:file` tags in the document.
`copy_registry` maps copy/paste IDs to their resolved text content.
**At any depth**, which is the whole of a bug that was live for a week.
This used to ask `find_tags`, which is the top level only, while the
engine's own weave has always asked `all_tags` — so a `hick:file` nested
inside anything was written to disk by `hick run` and invisible to
everything built on this. The nesting is not exotic: `exec > ingested >
file` is exactly what `hick ingest` writes, so every scaffold a document
owned had no language server inside it, and the debugger — which weaves
through this function — reported "this document does not generate
app/Program.cs. It generates:" and then nothing, about a file sitting in
the document it was reading.
The rule is one rule: **the same file blocks the weave writes**. A
`hick:file` inside another is not skipped, for the same reason — matching
the weave is the point, and it does not skip one either.

<storycode:copy id="f-a254cbbf439c-build_virtual_files">
/// Build virtual files from all `hick:file` tags in the document.
///
/// `copy_registry` maps copy/paste IDs to their resolved text content.
///
/// **At any depth**, which is the whole of a bug that was live for a week.
/// This used to ask `find_tags`, which is the top level only, while the
/// engine's own weave has always asked `all_tags` — so a `hick:file` nested
/// inside anything was written to disk by `hick run` and invisible to
/// everything built on this. The nesting is not exotic: `exec > ingested >
/// file` is exactly what `hick ingest` writes, so every scaffold a document
/// owned had no language server inside it, and the debugger — which weaves
/// through this function — reported "this document does not generate
/// app/Program.cs. It generates:" and then nothing, about a file sitting in
/// the document it was reading.
///
/// The rule is one rule: **the same file blocks the weave writes**. A
/// `hick:file` inside another is not skipped, for the same reason — matching
/// the weave is the point, and it does not skip one either.
pub fn build_virtual_files(
    doc: &HickDocument,
    copy_registry: &HashMap<String, String>,
) -> Vec<VirtualFile> {
    let mut files = Vec::new();

    for tag in doc.all_tags().into_iter().filter(|t| t.name == "file") {
        let path = match tag.get_attribute("path") {
            Some(p) => p.to_string(),
            None => continue,
        };

        let language_id = lang_detect::language_id(&path);
        let mut segments = Vec::new();

        collect_segments(tag, copy_registry, &mut segments);

        files.push(VirtualFile {
            path,
            language_id,
            segments,
        });
    }

    files
}

</storycode:copy>
<a id="f-a254cbbf439c-collect_segments"></a>
### `collect_segments`

<storycode:copy id="f-a254cbbf439c-collect_segments">
fn collect_segments(
    tag: &HickTag,
    copy_registry: &HashMap<String, String>,
    segments: &mut Vec<VirtualSegment>,
) {
    for child in &tag.children {
        match child {
            HickNode::Text(text, span) => {
                let dedented = hick_lang::dedent(text, tag.source_column);
                segments.push(VirtualSegment {
                    text: dedented,
                    source_span: *span,
                    source_line: tag.source_line,
                    source_column: tag.source_column,
                });
            }
            HickNode::Tag(child_tag) => {
                if child_tag.name == "copy" || child_tag.name == "paste" {
                    if let Some(select) = child_tag.get_attribute("select") {
                        let id = select.strip_prefix('#').unwrap_or(select);
                        if let Some(content) = copy_registry.get(id) {
                            segments.push(VirtualSegment {
                                text: content.clone(),
                                source_span: child_tag.source_span,
                                source_line: child_tag.source_line,
                                source_column: child_tag.source_column,
                            });
                        }
                    }
                } else {
                    // Recurse into other tags
                    collect_segments(child_tag, copy_registry, segments);
                }
            }
        }
    }
}

</storycode:copy>
<a id="f-a254cbbf439c-tests"></a>
### `tests`

<storycode:copy id="f-a254cbbf439c-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use hick_lang::{HickDocument, HickNode, HickTag, SourceSpan};

    fn make_doc(nodes: Vec<HickNode>) -> HickDocument {
        HickDocument {
            nodes,
            source: String::new(),
            prefix: "hick".to_string(),
            weave_path: None,
            frontmatter: None,
            volatile: false,
            span_files: Vec::new(),
            root_tag: None,
        }
    }

    fn make_file_tag(path: &str, children: Vec<HickNode>) -> HickTag {
        HickTag {
            name: "file".to_string(),
            attributes: vec![("path".to_string(), path.to_string())],
            children,
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: Some(SourceSpan::new(0, 10, 1, 0)),
            close_span: None,
        }
    }

    #[test]
    fn simple_text_file() {
        let doc = make_doc(vec![HickNode::Tag(make_file_tag(
            "src/main.rs",
            vec![HickNode::Text(
                "fn main() {}\n".to_string(),
                Some(SourceSpan::new(20, 33, 2, 0)),
            )],
        ))]);

        let registry = HashMap::new();
        let files = build_virtual_files(&doc, &registry);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[0].language_id, Some("rust"));
        assert_eq!(files[0].content(), "fn main() {}\n");
        assert_eq!(files[0].line_count(), 1);
    }

    #[test]
    fn file_with_copy_paste() {
        let copy_tag = HickTag {
            name: "paste".to_string(),
            attributes: vec![("select".to_string(), "#imports".to_string())],
            children: vec![],
            self_closing: true,
            source_line: 3,
            source_column: 4,
            source_span: Some(SourceSpan::new(40, 70, 3, 4)),
            close_span: None,
        };

        let doc = make_doc(vec![HickNode::Tag(make_file_tag(
            "app.py",
            vec![
                HickNode::Tag(copy_tag),
                HickNode::Text(
                    "\ndef main():\n    pass\n".to_string(),
                    Some(SourceSpan::new(70, 92, 4, 0)),
                ),
            ],
        ))]);

        let mut registry = HashMap::new();
        registry.insert("imports".to_string(), "import os\n".to_string());

        let files = build_virtual_files(&doc, &registry);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].content(), "import os\n\ndef main():\n    pass\n");
        assert_eq!(files[0].language_id, Some("python"));
    }

    #[test]
    fn file_without_path_is_skipped() {
        let tag = HickTag {
            name: "file".to_string(),
            attributes: vec![],
            children: vec![HickNode::Text("content".to_string(), None)],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
            close_span: None,
        };
        let doc = make_doc(vec![HickNode::Tag(tag)]);
        let files = build_virtual_files(&doc, &HashMap::new());
        assert!(files.is_empty());
    }

    #[test]
    fn multiple_files() {
        let doc = make_doc(vec![
            HickNode::Tag(make_file_tag(
                "a.rs",
                vec![HickNode::Text("a\n".to_string(), None)],
            )),
            HickNode::Tag(make_file_tag(
                "b.py",
                vec![HickNode::Text("b\n".to_string(), None)],
            )),
        ]);

        let files = build_virtual_files(&doc, &HashMap::new());
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "a.rs");
        assert_eq!(files[1].path, "b.py");
    }

    #[test]
    fn empty_file_line_count() {
        let doc = make_doc(vec![HickNode::Tag(make_file_tag("empty.txt", vec![]))]);
        let files = build_virtual_files(&doc, &HashMap::new());
        assert_eq!(files[0].line_count(), 0);
    }

    #[test]
    fn copy_select_without_hash() {
        let copy_tag = HickTag {
            name: "copy".to_string(),
            attributes: vec![("select".to_string(), "no_hash".to_string())],
            children: vec![],
            self_closing: true,
            source_line: 2,
            source_column: 0,
            source_span: None,
            close_span: None,
        };

        let doc = make_doc(vec![HickNode::Tag(make_file_tag(
            "out.txt",
            vec![HickNode::Tag(copy_tag)],
        ))]);

        let mut registry = HashMap::new();
        registry.insert("no_hash".to_string(), "resolved\n".to_string());

        let files = build_virtual_files(&doc, &registry);
        assert_eq!(files[0].content(), "resolved\n");
    }
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/virtual_file.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/virtual_file.rs"><storycode:paste select="#f-a254cbbf439c-VirtualSegment"/><storycode:paste select="#f-a254cbbf439c-VirtualFile"/><storycode:paste select="#f-a254cbbf439c-impl-VirtualFile"/><storycode:paste select="#f-a254cbbf439c-build_virtual_files"/><storycode:paste select="#f-a254cbbf439c-collect_segments"/><storycode:paste select="#f-a254cbbf439c-tests"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-dap/src/program.rs](../../../mechanisms/crates/hick-dap/src/program.rs.md#output-dd5fefdc2d90)
- [crates/hick-dap/src/session.rs](../../../mechanisms/crates/hick-dap/src/session.rs.md#output-3c41c22e0551)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)
- [crates/hick-literate/src/pipeline.rs](contract.md#output-0281c6fee9f5)
- [crates/hick-lsp/src/document.rs](../../../mechanisms/crates/hick-lsp/src/document.rs.md#output-7017743f2a99)
- [crates/hick-lsp/src/position_map.rs](../../../mechanisms/crates/hick-lsp/src/position_map.rs.md#output-c77393891cd2)
- [crates/hick-lsp/src/virtual_file.rs](contract.md#output-a254cbbf439c)

### Tests and fixtures

- [crates/hick-dap/tests/live_capture.rs](../../../mechanisms/crates/hick-dap/tests/live_capture.rs.md#output-74b67dfe72b0)
- [crates/hick-dap/tests/live_session.rs](../../../mechanisms/crates/hick-dap/tests/live_session.rs.md#output-7cb62902a4d7)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
