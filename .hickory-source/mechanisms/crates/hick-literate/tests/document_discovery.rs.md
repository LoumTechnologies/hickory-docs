<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/document_discovery.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1781823434f8"></a>
## `crates/hick-literate/tests/document_discovery.rs`

<a id="f-1781823434f8-default_directory_discovery_prunes_runtime_trees_but_honors_explicit_targets"></a>
### `default_directory_discovery_prunes_runtime_trees_but_honors_explicit_targets`

<storycode:copy id="f-1781823434f8-default_directory_discovery_prunes_runtime_trees_but_honors_explicit_targets">
use hick_literate::expand_path_arg;

#[test]
fn default_directory_discovery_prunes_runtime_trees_but_honors_explicit_targets() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("note.md"), "project note").unwrap();
    let excluded = [
        ".git",
        ".hick-cache",
        "node_modules",
        ".venv",
        "__pycache__",
        "target",
        "dist",
        "dist-site",
        "build",
        "bin",
        "obj",
        ".dev",
    ];
    for name in excluded {
        let child = dir.path().join(name).join("package");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::write(child.join("README.md"), "runtime dependency").unwrap();
    }
    let (files, config) = expand_path_arg(dir.path()).unwrap();
    assert_eq!(files, vec![dir.path().join("note.md")]);
    assert!(config.is_none());
    let selected = dir.path().join(".venv/package/README.md");
    assert_eq!(
        expand_path_arg(&selected).unwrap().0,
        vec![selected.clone()]
    );
    assert_eq!(
        expand_path_arg(&dir.path().join(".venv")).unwrap().0,
        vec![selected]
    );
    std::fs::write(
        dir.path().join("_hick.yml"),
        "files:\n  - .hick-cache/package/README.md\n",
    )
    .unwrap();
    assert_eq!(
        expand_path_arg(dir.path()).unwrap().0,
        vec![dir.path().join(".hick-cache/package/README.md")]
    );
}

</storycode:copy>
<a id="f-1781823434f8-default_discovery_prunes_unreadable_cache_subtrees_before_traversal"></a>
### `default_discovery_prunes_unreadable_cache_subtrees_before_traversal`

<storycode:copy id="f-1781823434f8-default_discovery_prunes_unreadable_cache_subtrees_before_traversal">
#[cfg(unix)]
#[test]
fn default_discovery_prunes_unreadable_cache_subtrees_before_traversal() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("note.md"), "project note").unwrap();
    let cache = dir.path().join(".hick-cache");
    std::fs::create_dir(&cache).unwrap();
    std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0)).unwrap();
    let result = expand_path_arg(dir.path());
    std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(result.unwrap().0, vec![dir.path().join("note.md")]);
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/document_discovery.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/document_discovery.rs"><storycode:paste select="#f-1781823434f8-default_directory_discovery_prunes_runtime_trees_but_honors_explicit_targets"/><storycode:paste select="#f-1781823434f8-default_discovery_prunes_unreadable_cache_subtrees_before_traversal"/></storycode:file>

</storycode:doc>
