<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# 3. Prepare a document before executing it

The pure tree gives us deterministic reconstruction. To run cells we must add
native source loading, container declarations and execution authority. We will
prepare that information first so verification can inspect the same document
without starting a process.

## Collect the environment we will need

Keep the parsed documents and shared construction state together with the
container's declared image, capabilities, required tools and volumes. The image
name alone does not confine a process; the chosen executor must enforce the
[declared capabilities](guarantees/execution/declared-capabilities-are-enforced/contract.md).

<a id="f-e099e80d2a99-PreparedPipeline"></a>
<book:copy id="f-e099e80d2a99-PreparedPipeline">
// Pipeline setup (shared between dry-run and live)

/// Intermediate state after parsing, filtering, and collecting definitions.
struct PreparedPipeline<'a> {
    documents: Vec<(&'a str, HickDocument)>,
    state: Arc<MultiDocumentState>,
    container_defs: HashMap<String, ContainerCapabilities>,
    /// Image each container declares, so the executor is told which
    /// environment to provide. Capabilities and images are collected from the
    /// same tag but were not carried together.
    container_images: HashMap<String, String>,
    /// What each container's cells expect to find installed
    /// (`<hick:needs bin="…" />`), checked before anything runs.
    container_needs: std::collections::BTreeMap<String, Vec<crate::needs::Need>>,
    /// Fork registrations: (from, to, additional_caps).
    /// Needed by live pipeline to register forks with the executor.
    fork_registrations: Vec<(String, String, Option<ContainerCapabilities>)>,
    /// Volume declarations across every document, by volume name. Collected
    /// here rather than in the live pipeline because their `<hick:allow>`
    /// children are part of a container's capabilities, and capabilities are
    /// minted into tokens here.
    volumes: HashMap<String, hick_exec::volume::VolumeDeclaration>,
}

</book:copy>

Build the preparation and run-mode pipeline around that state. Validate retained
captures during preparation and re-preparation, including included documents.
<book:include file="mechanisms/crates/hick-literate/src/lib.rs.md" />

## Keep retained bytes literal through the native adapter

The native adapter also handles variables and substitutions. Those authored
features must not rewrite a captured path or body. Use a path containing variable
syntax and a body containing substitution text, a BOM, Unicode and CRLF. Replay
must preserve all of them, and a changed capture must fail its digest.

<a id="immutable-scaffold-literal-replay-test"></a>
<book:copy id="immutable-scaffold-literal-replay-test">
#[tokio::test]
async fn standalone_scaffold_replay_keeps_literal_paths_and_bytes() {
    let path = "{{name}}.txt";
    let content = "\u{feff}FavoriteApp λ\r\n";
    let hash = hick_lang::scaffold_fingerprint(&std::collections::BTreeMap::from([(
        path.into(),
        content.into(),
    )]));
    let source = format!(
        r#"<hick:var name="name">changed</hick:var>
<hick:substitute name="app" pattern="FavoriteApp" value="Modified" />
<hick:container name="sdk" />
<hick:exec container="sdk">
<hick:copy>echo retained</hick:copy>
<hick:ingested sha256="{hash}">
<hick:file path="{path}">
{content}</hick:file>
</hick:ingested>
</hick:exec>
"#
    );
    let result = hick_literate::run_pipeline_weave(&[("standalone.md", &source)], &[], None)
        .await
        .unwrap();
    assert_eq!(result.files.len(), 1);
    assert_eq!(result.files[path].as_text(), Some(content));
    let tampered = source.replace("FavoriteApp", "WrongBase");
    let error = hick_literate::run_pipeline_weave(&[("standalone.md", &tampered)], &[], None)
        .await
        .err()
        .unwrap();
    assert!(error.to_string().contains("hash mismatch"));
}
</book:copy>

The remaining native project witnesses complete the construction of this test file.
<book:include file="mechanisms/crates/hick-literate/tests/project.rs.md" />

```sh
cargo test --locked -p hick-literate --test project \
  standalone_scaffold_replay_keeps_literal_paths_and_bytes -- --exact
```

We can now prepare and replay a native document while retaining the boundary
between inspection and execution. Next, [map editor actions through byte origins](04-editing.md).
</book:doc>
