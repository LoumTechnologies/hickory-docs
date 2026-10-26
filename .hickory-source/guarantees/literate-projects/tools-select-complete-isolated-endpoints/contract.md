<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep language tools and debug launches tied to their captured project

For: Maintainers reorganizing source while preserving the complete generated program.

Read one side of a comparison while another file is renamed. Its tools must use the captured side and filename; choosing a debugger must not silently change ordinary language-service scope.

## What must happen

Language and debug tools receive explicit captured project and source identities. Ordinary document language services use the completed endpoint. File-block and generated-output debug requests identify the displayed code; the host rejects changed source and chooses the latest supported matching whole tree. Comparisons retain the immutable capture used by their tools, breakpoints and launch. Opening a comparison can register memory-only language context but does not execute the program. Removed or ambiguous mappings remain unavailable.

## Limits of the evidence

A tool context is not an execution receipt. Additional native platforms and adapters require their own witnesses.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Remove the historical global-selector and auto-open-different-endpoint expectations. Retain the later exact-source rule and revision-local filenames.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-86b58a00764b"></a>
## `crates/hick-dap/src/project.rs`

Related contracts: [Keep language tools and debug launches tied to their captured project](contract.md).

<a id="f-86b58a00764b-ProjectLaunch"></a>
### `ProjectLaunch`

Explicit revision-aware debugger preparation. Native builds and adapters use
this complete selected tree; none of these operations run during rendering.

<storycode:copy id="f-86b58a00764b-ProjectLaunch">
//! Explicit revision-aware debugger preparation. Native builds and adapters use
//! this complete selected tree; none of these operations run during rendering.
use anyhow::Result;
use hick_lang::EvaluatedProject;
use std::collections::BTreeMap;

pub struct ProjectLaunch {
    pub materialized: hick_lsp::MaterializedProject,
    pub program: std::path::PathBuf,
    pub language: &'static str,
}
</storycode:copy>
<a id="f-86b58a00764b-prepare"></a>
### `prepare`

<storycode:copy id="f-86b58a00764b-prepare">
pub fn prepare(
    project: &EvaluatedProject,
    side: hick_lsp::ProjectEndpoint,
) -> Result<ProjectLaunch> {
    let materialized = hick_lsp::materialize(project, side).map_err(anyhow::Error::msg)?;
    let files = materialized.files();
    let program = crate::entry_point(&files)?;
    let language = crate::language_of(&program)
        .ok_or_else(|| anyhow::anyhow!("selected project entry has no supported debug language"))?;
    // Adapter discovery does not certify successful building or launching.
    Ok(ProjectLaunch {
        materialized,
        program,
        language,
    })
}
</storycode:copy>
<a id="f-86b58a00764b-prepare_closure"></a>
### `prepare_closure`

<storycode:copy id="f-86b58a00764b-prepare_closure">
pub fn prepare_closure(
    entry: &str,
    sources: &BTreeMap<String, String>,
    input: &hick_lang::ProjectTree,
    side: hick_lsp::ProjectEndpoint,
) -> Result<ProjectLaunch> {
    let project = hick_lang::evaluate_project(entry, sources, input).map_err(anyhow::Error::msg)?;
    prepare(&project, side)
}

</storycode:copy>
<a id="f-86b58a00764b-document_maps"></a>
### `document_maps`

<storycode:copy id="f-86b58a00764b-document_maps">
pub(crate) fn document_maps(
    state: &hick_lsp::document::HickDocumentState,
    workdir: &std::path::Path,
) -> std::collections::HashMap<std::path::PathBuf, hick_lsp::position_map::PositionMap> {
    let workdir = workdir
        .canonicalize()
        .unwrap_or_else(|_| workdir.to_path_buf());
    state
        .virtual_files
        .iter()
        .map(|file| {
            (
                workdir.join(&file.path),
                state
                    .project_maps
                    .get(&file.path)
                    .cloned()
                    .unwrap_or_else(|| {
                        hick_lsp::position_map::PositionMap::build(&file.segments)
                            .without_first_line()
                    }),
            )
        })
        .collect()
}

</storycode:copy>
<a id="f-86b58a00764b-generated_line"></a>
### `generated_line`

<storycode:copy id="f-86b58a00764b-generated_line">
pub(crate) fn generated_line(
    files: &std::collections::HashMap<std::path::PathBuf, hick_lsp::position_map::PositionMap>,
    line: u32,
) -> Option<(std::path::PathBuf, u32)> {
    let mut found = None;
    for (path, map) in files {
        for mapping in &map.mappings {
            if mapping.source_line != line {
                continue;
            }
            if found.is_some() {
                return None;
            } // A line breakpoint must have one destination.
            found = Some((path.clone(), mapping.virtual_line));
        }
    }
    found
}
</storycode:copy>

### Assemble `crates/hick-dap/src/project.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/project.rs"><storycode:paste select="#f-86b58a00764b-ProjectLaunch"/><storycode:paste select="#f-86b58a00764b-prepare"/><storycode:paste select="#f-86b58a00764b-prepare_closure"/><storycode:paste select="#f-86b58a00764b-document_maps"/><storycode:paste select="#f-86b58a00764b-generated_line"/></storycode:file>

## Test the behavior: `dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines`

This is the maintained test fragment from `crates/hick-dap/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-b360a6d46b8d-dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines"></a>
### `dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines`

<storycode:copy id="f-b360a6d46b8d-dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines">
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines() {
    let source = "<hick:project/>\n<hick:file id=\"base\" path=\"program.py\">\nprint('before')\n</hick:file>\n<hick:patch after=\"base\">\n--- a/program.py\n+++ b/program.py\n@@ -1 +1 @@\n-print('before')\n+print('after')\n</hick:patch>";
    let sources = BTreeMap::from([("p.md".into(), source.into())]);
    let state = hick_lsp::document::HickDocumentState::from_project("p.md", &sources).unwrap();
    let mapping =
        hick_dap::Mapping::for_state(Path::new("p.md"), &state, Path::new("scratch")).unwrap();
    assert_eq!(
        mapping.to_generated(9),
        Some((Path::new("scratch/program.py").to_path_buf(), 0))
    );
    assert_eq!(mapping.to_generated(8), None);
    assert!(hick_lang::evaluate_project("p.md", &sources, &ProjectTree::default()).is_ok());
}

</storycode:copy>

## Test the behavior: `explicit_nested_scopes_finish_all_their_acts_without_ordering_independent_writes`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-explicit_nested_scopes_finish_all_their_acts_without_ordering_independent_writes"></a>
### `explicit_nested_scopes_finish_all_their_acts_without_ordering_independent_writes`

<storycode:copy id="f-005c1e9344c7-explicit_nested_scopes_finish_all_their_acts_without_ordering_independent_writes">
// Guarantees: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn explicit_nested_scopes_finish_all_their_acts_without_ordering_independent_writes() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"app\">a\n</hick:file>
<hick:scope id=\"feature\" after=\"base\">
<hick:patch id=\"first\">\n--- a/app\n+++ b/app\n@@ -1 +1 @@\n-a\n+b\n</hick:patch>
<hick:scope id=\"nested\" after=\"first\">
<hick:patch id=\"last\">\n--- a/app\n+++ b/app\n@@ -1 +1 @@\n-b\n+c\n</hick:patch>
</hick:scope></hick:scope>
<hick:patch after=\"feature\">\n--- a/app\n+++ b/app\n@@ -1 +1 @@\n-c\n+d\n</hick:patch>";
    let project = generate(source).unwrap();
    assert_eq!(
        &*project.scopes["feature"].0.get("app").unwrap().bytes,
        b"a\n"
    );
    assert_eq!(
        &*project.scopes["feature"].1.get("app").unwrap().bytes,
        b"c\n"
    );
    assert_eq!(
        &*project.scopes["nested"].0.get("app").unwrap().bytes,
        b"b\n"
    );
    assert_eq!(
        &*project.scopes["nested"].1.get("app").unwrap().bytes,
        b"c\n"
    );
    assert_eq!(&*project.new.get("app").unwrap().bytes, b"d\n");
    assert!(
        generate(&source.replace("id=\"nested\" after=\"first\"", "id=\"nested\""))
            .unwrap_err()
            .contains("context")
    );
}

</storycode:copy>

## Test the behavior: `local_bytes_map_to_renamed_endpoint_by_origin_and_removed_bytes_have_no_endpoint`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-local_bytes_map_to_renamed_endpoint_by_origin_and_removed_bytes_have_no_endpoint"></a>
### `local_bytes_map_to_renamed_endpoint_by_origin_and_removed_bytes_have_no_endpoint`

<storycode:copy id="f-005c1e9344c7-local_bytes_map_to_renamed_endpoint_by_origin_and_removed_bytes_have_no_endpoint">
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn local_bytes_map_to_renamed_endpoint_by_origin_and_removed_bytes_have_no_endpoint() {
    let source = "<hick:project/><hick:file id=\"a\" path=\"a.ts\">🦀a\nb\n</hick:file><hick:patch id=\"rename\" after=\"a\">\n--- a/a.ts\n+++ b/b.ts\n@@ -1,2 +1 @@\n 🦀a\n-b\n</hick:patch>";
    let p = generate(source).unwrap();
    let local = &p.local_states["a"].1;
    assert_eq!(
        hick_lang::project_byte_location(local, &p.new, "a.ts", 4).unwrap(),
        Some(("b.ts".into(), 4))
    );
    assert_eq!(
        hick_lang::project_byte_location(local, &p.new, "a.ts", 6).unwrap(),
        None
    );
}
</storycode:copy>

## Test the behavior: `services_use_completed_tree_and_separate_native_identities`

This is the maintained test fragment from `crates/hick-lsp/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d2279aae5a8a-services_use_completed_tree_and_separate_native_identities"></a>
### `services_use_completed_tree_and_separate_native_identities`

<storycode:copy id="f-d2279aae5a8a-services_use_completed_tree_and_separate_native_identities">
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn services_use_completed_tree_and_separate_native_identities() {
    let (_, project) = fixture();
    assert!(
        materialize(&project, ProjectEndpoint::Old)
            .unwrap_err()
            .contains("empty")
    );
    let a = materialize(&project, ProjectEndpoint::New).unwrap();
    let b = materialize(&project, ProjectEndpoint::New).unwrap();
    assert_ne!(a.root(), b.root());
    assert_eq!(
        std::fs::read(a.root().join("library.ts")).unwrap(),
        b"export function welcome(name: string) { return 'Hello ' + name; }\n"
    );
    let local = materialize(&project, ProjectEndpoint::LocalAfter("caller".into())).unwrap();
    assert!(
        std::fs::read_to_string(local.root().join("library.ts"))
            .unwrap()
            .contains("greet")
    );
    assert!(
        endpoint(&project, &ProjectEndpoint::New)
            .unwrap()
            .get("app.ts")
            .is_some()
    );
}
</storycode:copy>

## Test the behavior: `removed_patch_lines_cannot_bind_to_unrelated_final_code`

This is the maintained test fragment from `crates/hick-lsp/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d2279aae5a8a-removed_patch_lines_cannot_bind_to_unrelated_final_code"></a>
### `removed_patch_lines_cannot_bind_to_unrelated_final_code`

<storycode:copy id="f-d2279aae5a8a-removed_patch_lines_cannot_bind_to_unrelated_final_code">
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn removed_patch_lines_cannot_bind_to_unrelated_final_code() {
    let (sources, project) = fixture();
    let source = &sources["project.md"];
    let removed = source.find("-import { greet }").unwrap() + 1;
    let inserted = source.find("+import { welcome }").unwrap() + 1;
    assert_eq!(
        to_endpoint(&project.new, "project.md", removed).unwrap(),
        None
    );
    assert_eq!(
        to_endpoint(&project.new, "project.md", inserted).unwrap(),
        Some(("app.ts".into(), 0))
    );
    assert_eq!(
        to_source(&project.new, "app.ts", 0).unwrap(),
        Some(("project.md".into(), inserted))
    );
    let positions = BTreeMap::from([("unicode.md".into(), "🦀é\nabc".into())]);
    assert_eq!(utf16_position(&positions, "unicode.md", 6).unwrap(), (0, 3));
    assert!(utf16_position(&positions, "unicode.md", 1).is_err());
    let state =
        hick_lsp::document::HickDocumentState::from_project("project.md", &sources).unwrap();
    let file = state
        .virtual_files
        .iter()
        .find(|f| f.path == "app.ts")
        .unwrap();
    assert_eq!(
        file.written_content().as_bytes(),
        project.new.get("app.ts").unwrap().bytes.as_ref()
    );
    let map = hick_lsp::position_map::PositionMap::build(&file.segments).without_first_line();
    let source_line = source[..inserted].bytes().filter(|b| *b == b'\n').count() as u32;
    assert_eq!(map.to_virtual(source_line, 1), Some((0, 0)));
    let removed_line = source[..removed].bytes().filter(|b| *b == b'\n').count() as u32;
    assert_eq!(map.to_virtual(removed_line, 1), None);
}
</storycode:copy>

## Test the behavior: `native_writer_does_not_follow_existing_symlinks`

This is the maintained test fragment from `crates/hick-lsp/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d2279aae5a8a-native_writer_does_not_follow_existing_symlinks"></a>
### `native_writer_does_not_follow_existing_symlinks`

<storycode:copy id="f-d2279aae5a8a-native_writer_does_not_follow_existing_symlinks">
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn native_writer_does_not_follow_existing_symlinks() {
    #[cfg(unix)]
    {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("escape")).unwrap();
        let mut tree = ProjectTree::default();
        tree.insert("escape/write", ProjectEntry::file(b"no".to_vec()))
            .unwrap();
        assert!(hick_lsp::write_tree(dir.path(), &tree).is_err());
        assert!(!outside.path().join("write").exists());
    }
}
</storycode:copy>

## Test the behavior: `complete_project_positions_keep_foreign_documents_and_utf16_boundaries`

This is the maintained test fragment from `crates/hick-lsp/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d2279aae5a8a-complete_project_positions_keep_foreign_documents_and_utf16_boundaries"></a>
### `complete_project_positions_keep_foreign_documents_and_utf16_boundaries`

<storycode:copy id="f-d2279aae5a8a-complete_project_positions_keep_foreign_documents_and_utf16_boundaries">
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn complete_project_positions_keep_foreign_documents_and_utf16_boundaries() {
    let sources = std::collections::BTreeMap::from([
        ("project.md".into(),"<hick:project sources=\"member.md\"/><hick:file id=\"a\" path=\"a.ts\">🦀x\n</hick:file>".into()),
        ("member.md".into(),"<hick:project entry=\"project.md\"/><hick:file id=\"b\" path=\"b.ts\">éy\n</hick:file>".into()),
    ]);
    let state =
        hick_lsp::document::HickDocumentState::from_project("project.md", &sources).unwrap();
    let foreign = &state.project_maps["b.ts"];
    let (document, line, column) = foreign.source_document(0, 1).unwrap();
    assert_eq!(document.as_deref(), Some("member.md"));
    assert_eq!(line, 0);
    assert!(column > 1);
    assert_eq!(foreign.to_source(0, 1), None);
    let local = &state.project_maps["a.ts"];
    assert!(
        local.to_source(0, 1).is_none(),
        "half a surrogate pair has no byte coordinate"
    );
    assert!(local.to_source(0, 2).is_some());
    let old = hick_lsp::document::HickDocumentState::from_project_at(
        "project.md",
        &sources,
        &hick_lang::ProjectTree::default(),
        &hick_lsp::ProjectEndpoint::Old,
    )
    .unwrap();
    assert!(
        old.virtual_files.is_empty(),
        "Old is genuinely empty, never borrowed working code"
    );
}

</storycode:copy>

## Test the behavior: `captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files`

This is the maintained test fragment from `crates/hick-lsp/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d2279aae5a8a-captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files"></a>
### `captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files`

<storycode:copy id="f-d2279aae5a8a-captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files">
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn captured_native_contexts_are_lazy_isolated_and_never_borrow_working_files() {
    let contexts = hick_lsp::ProjectContexts::default();
    let mut old = ProjectTree::default();
    old.insert("app.ts", ProjectEntry::file(b"old".to_vec()))
        .unwrap();
    let mut new = old.clone();
    new.insert("app.ts", ProjectEntry::file(b"new".to_vec()))
        .unwrap();
    contexts.register("1111111111111111", old).unwrap();
    contexts.register("2222222222222222", new).unwrap();
    let (_, _, a) = contexts
        .get(&"hick-project:///1111111111111111/app.ts".parse().unwrap())
        .unwrap();
    let (_, _, b) = contexts
        .get(&"hick-project:///2222222222222222/app.ts".parse().unwrap())
        .unwrap();
    assert!(!a.is_materialized());
    assert!(!b.is_materialized());
    let first = a.materialize().unwrap();
    let second = b.materialize().unwrap();
    assert_ne!(first, second);
    assert_eq!(std::fs::read(first.join("app.ts")).unwrap(), b"old");
    assert_eq!(std::fs::read(second.join("app.ts")).unwrap(), b"new");
    assert!(
        contexts
            .get(
                &"hick-project:///1111111111111111/missing.ts"
                    .parse()
                    .unwrap()
            )
            .is_err()
    );
    contexts.remove("1111111111111111");
    assert!(
        contexts
            .get(&"hick-project:///1111111111111111/app.ts".parse().unwrap())
            .is_err()
    );
    assert!(
        contexts
            .register("3333333333333333", ProjectTree::default())
            .is_err()
    );
}

</storycode:copy>

## Test the behavior: `native_context_registration_freezes_memory_without_borrowing_working_code`

This is the maintained test fragment from `crates/hickory-cli/tests/literate_agent_projects.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c0298ec2aa35-native_context_registration_freezes_memory_without_borrowing_working_code"></a>
### `native_context_registration_freezes_memory_without_borrowing_working_code`

<storycode:copy id="f-c0298ec2aa35-native_context_registration_freezes_memory_without_borrowing_working_code">
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[tokio::test]
async fn native_context_registration_freezes_memory_without_borrowing_working_code() {
    let host = host().await;
    let mut tree = hick_lang::ProjectTree::default();
    tree.insert(
        "app.ts",
        hick_lang::ProjectEntry::file(b"captured B\n".to_vec()),
    )
    .unwrap();
    tree.insert(
        "library.ts",
        hick_lang::ProjectEntry::file(b"later library\n".to_vec()),
    )
    .unwrap();
    let (status, registered) = request(
        &host,
        "/api/project-contexts",
        Some(json!({"snapshot":hick_lang::ProjectSnapshot::capture(&tree)})),
    )
    .await;
    assert_eq!(status, 200, "{registered}");
    assert_eq!(registered["materialized"], false);
    assert!(!host.root.join("library.ts").exists());
    std::fs::write(host.root.join("app.ts"), "concurrent working code\n").unwrap();
    let mut url = reqwest::Url::parse(&format!("{}/api/project-contexts/file", host.base)).unwrap();
    url.query_pairs_mut().append_pair(
        "uri",
        &format!("{}app.ts", registered["uri"].as_str().unwrap()),
    );
    let response = reqwest::get(url.clone()).await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let file: Value = response.json().await.unwrap();
    assert_eq!(file["bytes"], json!(b"captured B\n".to_vec()));
    assert_eq!(file["files"].as_array().unwrap().len(), 2);
    let removed = reqwest::Client::new()
        .delete(format!(
            "{}/api/project-contexts/{}",
            host.base,
            registered["id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(removed.status(), reqwest::StatusCode::OK);
    assert_eq!(
        reqwest::get(url).await.unwrap().status(),
        reqwest::StatusCode::NOT_FOUND
    );
}

</storycode:copy>

<a id="output-b2544b4d3019"></a>
## `crates/hickory-cli/tests/literate_project_csharp.rs`

Related contracts: [Keep language tools and debug launches tied to their captured project](contract.md).

<a id="f-b2544b4d3019-efcore_final_project_applies_migration_and_debugs_inserted_code"></a>
### `efcore_final_project_applies_migration_and_debugs_inserted_code`

Explicit native acceptance, separated from pure equality tests.

<storycode:copy id="f-b2544b4d3019-efcore_final_project_applies_migration_and_debugs_inserted_code">
//! Explicit native acceptance, separated from pure equality tests.
use hickory_cli::{ExecutorChoice, RunMode};
use std::{path::Path, process::Command, time::Duration};

// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires .NET, NuGet access and an installed pinned netcoredbg"]
async fn efcore_final_project_applies_migration_and_debugs_inserted_code() {
    let source = include_str!("../../hick-lang/tests/fixtures/efcore-project/project.md");
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("project.md"), source).unwrap();
    std::fs::write(
        root.join("persistence.md"),
        include_str!("../../hick-lang/tests/fixtures/efcore-project/persistence.md"),
    )
    .unwrap();
    let run = hickory_cli::run_doc(
        &root.join("project.md"),
        &[],
        RunMode::Weave,
        ExecutorChoice::Local,
    )
    .await
    .unwrap();
    assert_eq!(
        run.result
            .evaluated_project
            .as_ref()
            .unwrap()
            .new
            .entries()
            .len(),
        6
    );
    // Pure equality must complete before any tool materialization/build.
    assert!(!root.join("app").exists());
    hickory_cli::write_outputs(&run, None).unwrap();
    assert!(hickory_cli::check_failures(&run, None).unwrap().is_empty());
    let native = Command::new("dotnet")
        .args(["run", "--project", "app/app.csproj"])
        .current_dir(root)
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .output()
        .unwrap();
    assert!(
        native.status.success(),
        "native EFCore run failed:\n{}\n{}",
        String::from_utf8_lossy(&native.stdout),
        String::from_utf8_lossy(&native.stderr)
    );
    let out = String::from_utf8(native.stdout).unwrap();
    assert!(out.lines().any(|l| l == "verified"), "{out}");
    assert!(out.lines().any(|l| l == "202610120001_Initial"), "{out}");
    assert!(hickory_cli::check_failures(&run, None).unwrap().is_empty());
    // Borrow only the installed adapter. No generated source/config contributes
    // to pure construction or verification.
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.hick-cache/adapters")
        .canonicalize()
        .unwrap();
    std::fs::create_dir_all(root.join(".hick-cache")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&cache, root.join(".hick-cache/adapters")).unwrap();
    #[cfg(not(unix))]
    copy_adapter_directory(&cache, &root.join(".hick-cache/adapters")).unwrap();
    let inserted = source.find("+db.SaveChanges();").unwrap() + 1;
    let line = source[..inserted].bytes().filter(|b| *b == b'\n').count() as u32;
    let registry = hickory_cli::debug_sessions::Registry::new();
    let mut messages = vec![];
    let (id, live, statuses) = registry
        .start(
            &root.join("project.md"),
            &[hick_dap::Breakpoint {
                line,
                condition: None,
                hit_condition: None,
                log_message: None,
            }],
            Some("app/Program.cs"),
            &mut |message| messages.push(format!("{message:?}")),
        )
        .await
        .unwrap_or_else(|e| panic!("native launch failed: {e:#}\n{}", messages.join("\n")));
    assert_eq!(statuses.len(), 1);
    assert!(!statuses[0].state.is_refused(), "{statuses:?}");
    let mut events = live.session.events();
    let stopped = match live
        .session
        .wait_for_stop(Duration::from_secs(60))
        .await
        .unwrap()
    {
        Some(stopped) => stopped,
        None => {
            let mut observed = vec![];
            while let Ok(event) = events.try_recv() {
                if event.event != "module" {
                    observed.push(format!("{event:?}"));
                }
            }
            panic!(
                "inserted source breakpoint did not stop; statuses={:?}; exit={:?}; events={:?}; build={:?}",
                live.session.breakpoint_statuses(),
                live.session.exit(),
                observed,
                messages
            );
        }
    };
    let frames = live.session.stack(stopped.thread_id).await.unwrap();
    let top = frames.first().unwrap();
    assert_eq!(top.line, Some(line));
    assert_eq!(
        live.session
            .evaluate("note.Text", Some(top.id), "watch")
            .await
            .unwrap()
            .value,
        "\"verified\""
    );
    let staged =
        std::fs::read_to_string(live.scratch_path().unwrap().join("app/Program.cs")).unwrap();
    assert!(
        staged.contains("db.Database.GetAppliedMigrations()"),
        "later complete code must be present"
    );
    registry.stop(&id).await.unwrap();
    assert!(hickory_cli::check_failures(&run, None).unwrap().is_empty());
}

</storycode:copy>
<a id="f-b2544b4d3019-copy_adapter_directory"></a>
### `copy_adapter_directory`

<storycode:copy id="f-b2544b4d3019-copy_adapter_directory">
#[cfg(not(unix))]
fn copy_adapter_directory(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(destination)?;
    for item in std::fs::read_dir(source)? {
        let item = item?;
        if item.file_type()?.is_symlink() {
            return Err(std::io::Error::other(
                "adapter fixture does not copy symlinks",
            ));
        }
        let target = destination.join(item.file_name());
        if item.file_type()?.is_dir() {
            copy_adapter_directory(&item.path(), &target)?;
        } else {
            std::fs::copy(item.path(), target)?;
        }
    }
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/literate_project_csharp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/literate_project_csharp.rs"><storycode:paste select="#f-b2544b4d3019-efcore_final_project_applies_migration_and_debugs_inserted_code"/><storycode:paste select="#f-b2544b4d3019-copy_adapter_directory"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-dap/src/project.rs](contract.md#output-86b58a00764b)
- [crates/hick-lsp/src/document.rs](../../../mechanisms/crates/hick-lsp/src/document.rs.md#output-7017743f2a99)
- [crates/hick-lsp/src/location_map.rs](../../../mechanisms/crates/hick-lsp/src/location_map.rs.md#output-f60f4855ed0c)
- [crates/hick-lsp/src/position_map.rs](../../../mechanisms/crates/hick-lsp/src/position_map.rs.md#output-c77393891cd2)
- [crates/hick-lsp/src/project.rs](../../../mechanisms/crates/hick-lsp/src/project.rs.md#output-144d2ea0027d)
- [crates/hick-lsp/src/project_contexts.rs](../../../mechanisms/crates/hick-lsp/src/project_contexts.rs.md#output-3058901b5b2e)
- [crates/hick-lsp/src/project_positions.rs](../../../mechanisms/crates/hick-lsp/src/project_positions.rs.md#output-6f96a89d99eb)
- [crates/hickory-cli/src/debug_sessions.rs](../../../mechanisms/crates/hickory-cli/src/debug_sessions.rs.md#output-24a9413a429b)
- [crates/hickory-cli/src/project_documents.rs](../../../mechanisms/crates/hickory-cli/src/project_documents.rs.md#output-7cd7736e0d32)

### Tests and fixtures

- [apps/web/src/components/ComparisonTools.test.tsx](../../../mechanisms/apps/web/src/components/ComparisonTools.test.tsx.md#output-d1cfc2ed25eb)
- [crates/hick-dap/tests/project.rs](../../../mechanisms/crates/hick-dap/tests/project.rs.md#output-b360a6d46b8d)
- [crates/hick-lang/tests/project.rs](../../../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7)
- [crates/hick-lsp/tests/project.rs](../../../mechanisms/crates/hick-lsp/tests/project.rs.md#output-d2279aae5a8a)
- [crates/hickory-cli/tests/literate_agent_projects.rs](../../../mechanisms/crates/hickory-cli/tests/literate_agent_projects.rs.md#output-c0298ec2aa35)
- [crates/hickory-cli/tests/literate_project_csharp.rs](contract.md#output-b2544b4d3019)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
