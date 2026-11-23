<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Diagram The Code Can Regenerate

For: Authors editing executable documents and the files those documents produce.

Given a folder of source code, when `hick diagram <path>` runs — at a
terminal, in CI, or inside a `hick:exec` cell, which is the intended home —
then a scene topology is deduced deterministically —
no model, no API key, no toolchain: tree-sitter compiled into the binary,
walking gitignore-aware — as nodes per file or per top-level directory and
edges from name-resolved references, **never** carrying a `layout`. Node
order is **path order, not walk order**: the same folder yields the same
topology on every machine, which is what makes an `<hick:expect>` around it a
pin rather than a record of one filesystem. And when
`hick diagram <path> --refresh <doc> --fragment <id>` runs, exactly that
`<hick:copy>` fragment's content is rewritten with the fresh topology, so a
derived diagram pasting it updates while the layout in the diagram's own body
is untouched by construction.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-diagram-the-code-can-regenerate.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-882b777229b8"></a>
## `crates/hickory-cli/src/diagram.rs`

Related contracts: [A Diagram The Code Can Regenerate](contract.md).

<a id="f-882b777229b8-Grouping"></a>
### `Grouping`

`hick diagram`: deduce a codebase's shape with no model and no toolchain.
The third way a diagram gets authored (after a person and an agent — see
`docs/specs/freeform/a-diagram-you-can-drag.md`): deterministically, from
the code. This walks a folder gitignore-aware, reads structure with
`hick-structure` (tree-sitter, ships in the binary, works offline), and
aggregates the name-resolved links into a scene topology — nodes and
edges only, NEVER a layout, because arranging boxes is the person's half
and a generator that invents positions destroys them on every run.
Honesty note, inherited from `hick-structure`: resolution is by NAME. The
picture this emits is a good place to start looking, not a compiler's
call graph. When the user has a toolchain, SCIP in a container inside a
`hick:exec` cell is the precise version of this — same output shape, so
the same derived diagram consumes either.

<storycode:copy id="f-882b777229b8-Grouping">
//! `hick diagram`: deduce a codebase's shape with no model and no toolchain.
//!
//! The third way a diagram gets authored (after a person and an agent — see
//! `docs/specs/freeform/a-diagram-you-can-drag.md`): deterministically, from
//! the code. This walks a folder gitignore-aware, reads structure with
//! `hick-structure` (tree-sitter, ships in the binary, works offline), and
//! aggregates the name-resolved links into a scene topology — nodes and
//! edges only, NEVER a layout, because arranging boxes is the person's half
//! and a generator that invents positions destroys them on every run.
//!
//! Honesty note, inherited from `hick-structure`: resolution is by NAME. The
//! picture this emits is a good place to start looking, not a compiler's
//! call graph. When the user has a toolchain, SCIP in a container inside a
//! `hick:exec` cell is the precise version of this — same output shape, so
//! the same derived diagram consumes either.

use std::collections::BTreeMap;
use std::path::Path;

use hick_literate::scene::{Scene, SceneEdge, SceneNode, Topology, to_mermaid};

/// How files collapse into nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grouping {
    /// One node per source file.
    File,
    /// One node per top-level directory (root-level files stand alone).
    Dir,
}

</storycode:copy>
<a id="f-882b777229b8-MAX_FILE_BYTES"></a>
### `MAX_FILE_BYTES`

Files too large to be source worth parsing (generated bundles, data).

<storycode:copy id="f-882b777229b8-MAX_FILE_BYTES">
/// Files too large to be source worth parsing (generated bundles, data).
const MAX_FILE_BYTES: u64 = 512 * 1024;

</storycode:copy>
<a id="f-882b777229b8-generate_topology"></a>
### `generate_topology`

The CLI command resolves identifiers across saved files. A file node carries
the location of its first definition when the analyzer finds one. The
deterministic scan runs without a separate language runtime.

<storycode:copy id="f-882b777229b8-generate_topology">
/// Walk `root` and deduce a topology from its code.
pub fn generate_topology(root: &Path, group: Grouping) -> std::io::Result<Topology> {
    let mut files = Vec::new();
    for entry in ignore::WalkBuilder::new(root).follow_links(false).build().flatten() {
        let path = entry.path();
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        if entry
            .metadata()
            .map(|m| m.len() > MAX_FILE_BYTES)
            .unwrap_or(true)
        {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        if hick_structure::language_of(&relative).is_none() {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(path) else {
            // Not UTF-8, not source we can read — skipped, not fatal.
            continue;
        };
        if let Some(structure) = hick_structure::analyze(&relative, &source) {
            files.push(structure);
        }
    }
    // Sorted by path, because the walker's order is the FILESYSTEM's and a
    // topology is meant to be pinned. `ignore::WalkBuilder` yields entries in
    // whatever order the directory hands back, which differs between machines
    // and between filesystems — so `examples/architecture-that-draws-itself`
    // pinned `lib` before `app` on the author's machine and produced `app`
    // before `lib` everywhere else, failing an expectation that was correct
    // about the shape and wrong about the order. A generator whose output a
    // document is invited to freeze cannot be one entry ahead of readdir.
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let links = hick_structure::resolve(&files);

    let node_of = |path: &str| -> String {
        match group {
            Grouping::File => path.to_string(),
            Grouping::Dir => match path.split_once('/') {
                Some((dir, _)) => dir.to_string(),
                None => path.to_string(),
            },
        }
    };

    // Nodes in path order (the sort above makes it the same everywhere),
    // edges counted.
    let mut order: Vec<String> = Vec::new();
    let mut destinations = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    for file in &files {
        let id = node_of(&file.path);
        if seen.insert(id.clone()) {
            if id == file.path {
                let path = file.path.replace('%', "%25").replace('#', "%23").replace('?', "%3F");
                let href = file.definitions.iter().map(|definition| definition.start_line).min()
                    .map_or(path.clone(), |line| format!("{path}#L{}", line + 1));
                destinations.insert(id.clone(), href);
            }
            order.push(id);
        }
    }
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for link in &links {
        let from = node_of(&link.from_path);
        let to = node_of(&link.to_path);
        if from == to {
            continue;
        }
        *counts.entry((from, to)).or_default() += 1;
    }

    Ok(Topology {
        nodes: order
            .into_iter()
            .map(|id| SceneNode {
                href: destinations.remove(&id),
                id,
                ..SceneNode::default()
            })
            .collect(),
        edges: counts
            .into_iter()
            .map(|((from, to), n)| SceneEdge {
                from,
                to,
                label: (n > 1).then(|| format!("{n} refs")),
                // Everything else — sides, colour — is a person's choice,
                // recorded by the editor and never invented here.
                ..SceneEdge::default()
            })
            .collect(),
    })
}

</storycode:copy>
<a id="f-882b777229b8-render"></a>
### `render`

The topology as the format the flag asked for.

<storycode:copy id="f-882b777229b8-render">
/// The topology as the format the flag asked for.
pub fn render(topology: &Topology, mermaid: bool) -> String {
    if mermaid {
        let scene = Scene {
            topology: Some(topology.clone()),
            ..Scene::default()
        };
        to_mermaid(&scene)
    } else {
        let mut json = serde_json::to_string_pretty(topology).expect("a topology serializes");
        json.push('\n');
        json
    }
}

</storycode:copy>
<a id="f-882b777229b8-model-diagram"></a>
### Evaluate a code-model diagram

A document script chooses the provider and GraphQL selection in
`codeModelQuery()`. Hickory supplies the query's `data` to
`buildDiagram(data)`, which returns a scene with any file links the author
chose. A fresh, bounded runtime evaluates each function. The provider is
an installed host tool, so simply opening a repository cannot launch an
executable from that repository.

A visible preview asks for updates using its last revision. Include the
script and source folder in the revision so an old preview or edited
script cannot claim the current result. Recheck the inputs before
publishing each fresh scene.

<storycode:copy id="f-882b777229b8-model-diagram">
#[derive(serde::Deserialize)]
pub(crate) struct ModelDiagramTarget {
    uri: String,
    script: String,
    revision: Option<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct ModelDiagramReply {
    revision: String,
    scene: Option<Scene>,
}

pub(crate) async fn workspace_model_diagram(
    axum::extract::State(state): axum::extract::State<crate::serve::LocalState>,
    axum::Json(target): axum::Json<ModelDiagramTarget>,
) -> Result<axum::Json<ModelDiagramReply>, crate::serve::api::ApiError> {
    use crate::serve::api::ApiError;
    static SCANS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(4);

    if !target.uri.starts_with("hick:///") || target.script.len() > 32 * 1024 {
        return Err(ApiError::bad_request(
            "Use a workspace document and a diagram script up to 32 KiB.",
        ));
    }
    let permit = SCANS
        .try_acquire()
        .map_err(|_| ApiError::conflict("The code model is busy. The diagram will retry."))?;
    let workspace = state.index.root().to_path_buf();
    let handle = tokio::runtime::Handle::current();
    let reply = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let lease = state
            .resource_root
            .check(&workspace)
            .map_err(ApiError::conflict)?;
        let path = crate::debug_sessions::source_document(&workspace, &target.uri)
            .map_err(|error| ApiError::bad_request(error.to_string()))?;
        let folder = path
            .parent()
            .ok_or_else(|| ApiError::bad_request("Open a document inside the workspace."))?;
        let revision = model_diagram_revision(folder, &target.script)
            .map_err(|error| ApiError::unprocessable(error.to_string()))?;
        let scene = if target.revision.as_deref() == Some(revision.as_str()) {
            None
        } else {
            let request: ModelQuery = diagram_javascript(&target.script, "codeModelQuery()", None)
                .map_err(|error| ApiError::unprocessable(error.to_string()))?;
            let model = handle
                .block_on(query_diagram_model(&workspace, folder, request))
                .map_err(|error| ApiError::unprocessable(error.to_string()))?;
            let scene: serde_json::Value = diagram_javascript(
                &target.script,
                "buildDiagram(JSON.parse(modelJson))",
                Some(&model),
            )
            .map_err(|error| ApiError::unprocessable(error.to_string()))?;
            let scene = hick_literate::scene::parse_scene(&scene.to_string())
                .map_err(ApiError::unprocessable)?;
            let (nodes, edges) = scene.topology();
            if nodes.len() > 300 || edges.len() > 2_000 {
                return Err(ApiError::unprocessable(
                    "The diagram exceeds 300 nodes or 2,000 edges.",
                ));
            }
            if model_diagram_revision(folder, &target.script)
                .ok()
                .as_deref()
                != Some(revision.as_str())
            {
                return Err(ApiError::conflict(
                    "Files changed during code-model analysis. The diagram will retry.",
                ));
            }
            Some(scene)
        };
        if state
            .resource_root
            .check(&workspace)
            .map_err(ApiError::conflict)?
            != lease
        {
            return Err(ApiError::conflict(
                "The workspace changed during code-model analysis.",
            ));
        }
        Ok(axum::Json(ModelDiagramReply { revision, scene }))
    })
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;
    reply
}

</storycode:copy>

<a id="f-882b777229b8-model-diagram-revision"></a>
### Keep a preview tied to the files it read

Order the file inventory before hashing it so directory traversal order cannot
trigger a refresh. Include the script in the revision because an edit to the
query or graph builder can change the output with the same project files.

<storycode:copy id="f-882b777229b8-model-diagram-revision">
fn model_diagram_revision(folder: &Path, script: &str) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let mut entries = BTreeMap::new();
    let mut bytes = 0;
    for (index, entry) in ignore::WalkBuilder::new(folder)
        .hidden(true)
        .follow_links(false)
        .filter_entry(|entry| entry.file_name() != "node_modules")
        .build()
        .enumerate()
    {
        anyhow::ensure!(
            index < 4_000,
            "The code-model diagram needs a smaller source folder."
        );
        let entry = entry?;
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        let path = entry.path();
        let metadata = entry.metadata()?;
        bytes += metadata.len();
        anyhow::ensure!(
            bytes <= 16 * 1024 * 1024,
            "The code-model diagram source exceeds 16 MiB."
        );
        let modified = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        entries.insert(
            path.strip_prefix(folder)?.to_path_buf(),
            (metadata.len(), modified.as_nanos()),
        );
    }
    let mut digest = Sha256::new();
    digest.update(folder.as_os_str().as_encoded_bytes());
    digest.update([0]);
    digest.update(script);
    for (path, (length, modified)) in entries {
        digest.update([0]);
        digest.update(path.as_os_str().as_encoded_bytes());
        digest.update([0]);
        digest.update(length.to_le_bytes());
        digest.update(modified.to_le_bytes());
    }
    Ok(format!("{:x}", digest.finalize()))
}

</storycode:copy>

<a id="f-882b777229b8-model-diagram-javascript"></a>
### Evaluate the document’s script with data only

Evaluate the query and scene functions in separate JavaScript contexts. The
second context gets the model response as a string. The script has a deadline
and memory limit, and the preview accepts a bounded JSON value from it.

<storycode:copy id="f-882b777229b8-model-diagram-javascript">
#[derive(serde::Deserialize)]
struct ModelQuery {
    language: String,
    query: String,
}

fn diagram_javascript<T: serde::de::DeserializeOwned>(
    script: &str,
    expression: &str,
    model: Option<&str>,
) -> anyhow::Result<T> {
    use rquickjs::{CatchResultExt, Context, Runtime, Value};
    let engine = Runtime::new()?;
    engine.set_memory_limit(32 * 1024 * 1024);
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
    engine.set_interrupt_handler(Some(Box::new(move || {
        std::time::Instant::now() >= deadline
    })));
    Context::full(&engine)?.with(|context| {
        context
            .globals()
            .set("modelJson", model.unwrap_or("null"))?;
        let source =
            format!("(function () {{ 'use strict';\n{script}\nreturn ({expression});\n}})()");
        let result: Value = context
            .eval(source)
            .catch(&context)
            .map_err(|error| anyhow::anyhow!("Diagram script: {error}"))?;
        let json = context
            .json_stringify(result)
            .catch(&context)
            .map_err(|error| anyhow::anyhow!("Diagram output: {error}"))?
            .ok_or_else(|| anyhow::anyhow!("The diagram script must return JSON data."))?
            .to_string()?;
        anyhow::ensure!(json.len() <= 512 * 1024, "Diagram output exceeds 512 KiB.");
        Ok(serde_json::from_str(&json)?)
    })
}

</storycode:copy>

<a id="f-882b777229b8-model-diagram-provider"></a>
### Read the model through its GraphQL protocol

Resolve the provider from a host-controlled installation before starting it.
Send one query using the same stdio protocol as the CLI. Bound the response and
stop the provider when the request times out.

<storycode:copy id="f-882b777229b8-model-diagram-provider">
// Preview runs can start automatically. Only a provider installed outside the
// workspace is trusted to execute. The project supplies a query and a scene
// script, and receives neither host handles nor the provider process.
fn diagram_model_provider(workspace: &Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    anyhow::ensure!(
        !language.is_empty()
            && language.len() <= 32
            && language
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
        "Choose a registered code-model language."
    );
    let name = format!("hick-model-{language}");
    let path = std::env::var_os("PATH").unwrap_or_default();
    let provider = std::env::split_paths(&path)
        .filter(|entry| entry.is_absolute())
        .map(|entry| entry.join(&name))
        .find_map(|candidate| {
            candidate
                .canonicalize()
                .ok()
                .filter(|file| file.is_file() && !file.starts_with(workspace))
        });
    #[cfg(debug_assertions)]
    let provider = provider.or_else(|| {
        let candidate = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../code-models")
            .join(language)
            .join(&name);
        candidate
            .canonicalize()
            .ok()
            .filter(|file| file.is_file() && !file.starts_with(workspace))
    });
    provider
        .ok_or_else(|| anyhow::anyhow!("Install {name} on Hickory's PATH to render this diagram."))
}

async fn query_diagram_model(
    workspace: &Path,
    folder: &Path,
    request: ModelQuery,
) -> anyhow::Result<String> {
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    anyhow::ensure!(
        request.query.len() <= 16 * 1024,
        "The code-model query exceeds 16 KiB."
    );
    let executable = diagram_model_provider(workspace, &request.language)?;
    let mut child = tokio::process::Command::new(&executable)
        .current_dir(
            executable
                .parent()
                .expect("canonical executable has a parent"),
        )
        .arg(folder)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let mut input = child.stdin.take().expect("piped stdin");
    let mut output = BufReader::new(
        child
            .stdout
            .take()
            .expect("piped stdout")
            .take(8 * 1024 * 1024),
    );
    let mut query = serde_json::to_vec(&serde_json::json!({"query": request.query}))?;
    query.push(b'\n');
    let mut response = String::new();
    tokio::time::timeout(std::time::Duration::from_secs(12), async {
        input.write_all(&query).await?;
        drop(input);
        output.read_line(&mut response).await?;
        Ok::<_, std::io::Error>(())
    })
    .await
    .map_err(|_| anyhow::anyhow!("The code-model query timed out."))??;
    anyhow::ensure!(
        response.ends_with('\n'),
        "The {} model returned no complete response. Check its installation and source folder.",
        request.language
    );
    let mut value: serde_json::Value = serde_json::from_str(&response)?;
    if let Some(errors) = value
        .get("errors")
        .filter(|errors| errors.as_array().is_some_and(|items| !items.is_empty()))
    {
        anyhow::bail!("Code-model query failed: {errors}");
    }
    let data = value
        .get_mut("data")
        .map(serde_json::Value::take)
        .ok_or_else(|| anyhow::anyhow!("The code model returned no query data."))?;
    Ok(serde_json::to_string(&data)?)
}
</storycode:copy>

<a id="f-882b777229b8-refresh_fragment"></a>
### `refresh_fragment`

Rewrite one named `<hick:copy>` fragment's content in place with a fresh
topology — the deterministic sibling of `hick refresh`. The layout of any
diagram pasting this fragment is untouched by construction: it lives in
the diagram's own body, keyed by node id, and this never goes near it.
Returns the new document text, or a message naming what was not found.

<storycode:copy id="f-882b777229b8-refresh_fragment">
/// Rewrite one named `<hick:copy>` fragment's content in place with a fresh
/// topology — the deterministic sibling of `hick refresh`. The layout of any
/// diagram pasting this fragment is untouched by construction: it lives in
/// the diagram's own body, keyed by node id, and this never goes near it.
///
/// Returns the new document text, or a message naming what was not found.
pub fn refresh_fragment(
    document: &str,
    fragment_id: &str,
    topology: &Topology,
) -> Result<String, String> {
    let needle_double = format!("id=\"{fragment_id}\"");
    let needle_single = format!("id='{fragment_id}'");
    let mut at = 0;
    while let Some(open_rel) = document[at..].find("<hick:copy") {
        let open = at + open_rel;
        let Some(gt_rel) = document[open..].find('>') else {
            break;
        };
        let gt = open + gt_rel;
        let head = &document[open..gt];
        if head.contains(&needle_double) || head.contains(&needle_single) {
            let Some(close_rel) = document[gt..].find("</hick:copy>") else {
                return Err(format!(
                    "the fragment `#{fragment_id}` never closes — nothing was rewritten"
                ));
            };
            let close = gt + close_rel;
            let body = serde_json::to_string(topology).expect("a topology serializes");
            return Ok(format!(
                "{}{}{}",
                &document[..gt + 1],
                body,
                &document[close..]
            ));
        }
        at = gt;
    }
    Err(format!(
        "no `<hick:copy id=\"{fragment_id}\">` in this document. The fragment a \
         derived diagram pastes must exist before it can be refreshed — add \
         one, or check the id against the diagram's `select`."
    ))
}

</storycode:copy>
<a id="f-882b777229b8-tests"></a>
### `tests`

<storycode:copy id="f-882b777229b8-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn topo() -> Topology {
        Topology {
            nodes: vec![SceneNode {
                id: "api".into(),
                ..SceneNode::default()
            }],
            edges: vec![],
        }
    }

    #[test]
    fn refresh_rewrites_exactly_the_named_fragment() {
        let doc = "prose\n<hick:copy id=\"other\">keep me</hick:copy>\n\
                   <hick:copy id=\"arch\">{\"nodes\":[]}</hick:copy>\ntail\n";
        let out = refresh_fragment(doc, "arch", &topo()).expect("rewrites");
        assert!(out.contains("keep me"), "{out}");
        assert!(out.contains(r#"{"nodes":[{"id":"api"}]"#), "{out}");
        assert!(out.starts_with("prose\n"), "{out}");
        assert!(out.ends_with("tail\n"), "{out}");
    }

    #[test]
    fn refresh_names_a_missing_fragment_instead_of_guessing() {
        let err = refresh_fragment("no fragments here", "arch", &topo()).unwrap_err();
        assert!(
            err.contains("#arch") || err.contains("id=\"arch\""),
            "{err}"
        );
    }

    #[test]
    fn a_generated_topology_never_carries_a_layout() {
        // The generator's half is topology; positions are the person's. The
        // serialized form must not even have the key.
        let json = render(&topo(), false);
        assert!(!json.contains("layout"), "{json}");
        assert!(json.contains("\"api\""), "{json}");
    }

    #[test]
    fn walking_a_small_tree_links_caller_to_callee() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("app")).unwrap();
        std::fs::create_dir_all(dir.path().join("lib")).unwrap();
        std::fs::write(dir.path().join("lib/util.py"), "def load():\n    pass\n").unwrap();
        std::fs::write(dir.path().join("app/main.py"), "load()\n").unwrap();
        let by_dir = generate_topology(dir.path(), Grouping::Dir).unwrap();
        let ids: Vec<_> = by_dir.nodes.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"app") && ids.contains(&"lib"), "{ids:?}");
        assert!(
            by_dir
                .edges
                .iter()
                .any(|e| e.from == "app" && e.to == "lib"),
            "{:?}",
            by_dir.edges
        );
        let by_file = generate_topology(dir.path(), Grouping::File).unwrap();
        assert!(
            by_file
                .edges
                .iter()
                .any(|e| e.from == "app/main.py" && e.to == "lib/util.py"),
            "{:?}",
            by_file.edges
        );
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/diagram.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/diagram.rs"><storycode:paste select="#f-882b777229b8-Grouping"/><storycode:paste select="#f-882b777229b8-MAX_FILE_BYTES"/><storycode:paste select="#f-882b777229b8-generate_topology"/><storycode:paste select="#f-882b777229b8-render"/><storycode:paste select="#f-882b777229b8-model-diagram"/><storycode:paste select="#f-882b777229b8-model-diagram-revision"/><storycode:paste select="#f-882b777229b8-model-diagram-javascript"/><storycode:paste select="#f-882b777229b8-model-diagram-provider"/><storycode:paste select="#f-882b777229b8-refresh_fragment"/><storycode:paste select="#f-882b777229b8-tests"/></storycode:file>

<a id="output-856fa0835344"></a>
## `crates/hickory-cli/tests/diagram_cli.rs`

Related contracts: [A Diagram The Code Can Regenerate](contract.md).

<a id="f-856fa0835344-hick"></a>
### `hick`

`hick diagram`: the deterministic, AI-free way a diagram gets authored.
Protects docs/guarantees/authoring/a-diagram-the-code-can-regenerate.md.

<storycode:copy id="f-856fa0835344-hick">
//! `hick diagram`: the deterministic, AI-free way a diagram gets authored.
//!
//! Protects docs/guarantees/authoring/a-diagram-the-code-can-regenerate.md.

use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-856fa0835344-fixture"></a>
### `fixture`

A two-directory fixture with one cross-directory call.

<storycode:copy id="f-856fa0835344-fixture">
/// A two-directory fixture with one cross-directory call.
fn fixture(dir: &Path) {
    std::fs::create_dir_all(dir.join("app")).unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("lib/util.py"), "def load():\n    pass\n").unwrap();
    std::fs::write(dir.join("app/main.py"), "load()\n").unwrap();
}

</storycode:copy>
<a id="f-856fa0835344-emits_a_scene_topology_with_no_layout"></a>
### `emits_a_scene_topology_with_no_layout`

<storycode:copy id="f-856fa0835344-emits_a_scene_topology_with_no_layout">
#[test]
fn emits_a_scene_topology_with_no_layout() {
    let dir = tempfile::tempdir().unwrap();
    fixture(dir.path());
    let out = hick().arg("diagram").arg(dir.path()).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Nodes and the cross-directory edge; NEVER a layout — positions are the
    // person's half, and a generator that writes them destroys arrangements.
    assert!(stdout.contains("\"app\""), "{stdout}");
    assert!(stdout.contains("\"lib\""), "{stdout}");
    assert!(stdout.contains("\"edges\""), "{stdout}");
    assert!(!stdout.contains("layout"), "{stdout}");
}

</storycode:copy>
<a id="f-856fa0835344-mermaid_output_is_the_same_downgrade_the_weave_uses"></a>
### `mermaid_output_is_the_same_downgrade_the_weave_uses`

<storycode:copy id="f-856fa0835344-mermaid_output_is_the_same_downgrade_the_weave_uses">
#[test]
fn mermaid_output_is_the_same_downgrade_the_weave_uses() {
    let dir = tempfile::tempdir().unwrap();
    fixture(dir.path());
    let out = hick()
        .arg("diagram")
        .arg(dir.path())
        .arg("--format")
        .arg("mermaid")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("flowchart TD"), "{stdout}");
    assert!(stdout.contains("app --> lib"), "{stdout}");
}

</storycode:copy>
<a id="f-856fa0835344-refresh_rewrites_the_fragment_and_leaves_the_layout_alone"></a>
### `refresh_rewrites_the_fragment_and_leaves_the_layout_alone`

<storycode:copy id="f-856fa0835344-refresh_rewrites_the_fragment_and_leaves_the_layout_alone">
#[test]
fn refresh_rewrites_the_fragment_and_leaves_the_layout_alone() {
    let dir = tempfile::tempdir().unwrap();
    fixture(dir.path());
    let doc = dir.path().join("arch.md");
    std::fs::write(
        &doc,
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"arch.md\">\n\
         <hick:copy id=\"arch-topology\">{\"nodes\":[],\"edges\":[]}</hick:copy>\n\n\
         <hick:diagram renderer=\"graph\">\n\
         {\n  \"topology\": <hick:paste select=\"#arch-topology\" />,\n  \"layout\": {\"app\": {\"x\": 7, \"y\": 9}}\n}\n\
         </hick:diagram>\n\
         </hick:doc>\n",
    )
    .unwrap();

    let out = hick()
        .arg("diagram")
        .arg(dir.path())
        .arg("--refresh")
        .arg(&doc)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let rewritten = std::fs::read_to_string(&doc).unwrap();
    // The fragment now holds the real topology…
    assert!(rewritten.contains(r#""id":"app""#), "{rewritten}");
    // …and the person's layout was not touched.
    assert!(
        rewritten.contains(r#""app": {"x": 7, "y": 9}"#),
        "{rewritten}"
    );

    // The refreshed document weaves: the derived diagram draws the new
    // nodes into its SVG.
    let weave = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        weave.status.success(),
        "{}",
        String::from_utf8_lossy(&weave.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("arch.md")).unwrap();
    assert!(woven.contains("![diagram](diagram-"), "{woven}");
    let name = woven
        .split("![diagram](")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .expect("an image reference");
    let svg = std::fs::read_to_string(dir.path().join(name)).expect("the SVG exists");
    assert!(
        svg.contains(">app</text>") && svg.contains(">lib</text>"),
        "{svg}"
    );
}

</storycode:copy>
<a id="f-856fa0835344-a_folder_with_nothing_to_draw_says_so_and_fails"></a>
### `a_folder_with_nothing_to_draw_says_so_and_fails`

<storycode:copy id="f-856fa0835344-a_folder_with_nothing_to_draw_says_so_and_fails">
#[test]
fn a_folder_with_nothing_to_draw_says_so_and_fails() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.txt"), "not source\n").unwrap();
    let out = hick().arg("diagram").arg(dir.path()).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("nothing to draw"), "{stderr}");
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/diagram_cli.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/diagram_cli.rs"><storycode:paste select="#f-856fa0835344-hick"/><storycode:paste select="#f-856fa0835344-fixture"/><storycode:paste select="#f-856fa0835344-emits_a_scene_topology_with_no_layout"/><storycode:paste select="#f-856fa0835344-mermaid_output_is_the_same_downgrade_the_weave_uses"/><storycode:paste select="#f-856fa0835344-refresh_rewrites_the_fragment_and_leaves_the_layout_alone"/><storycode:paste select="#f-856fa0835344-a_folder_with_nothing_to_draw_says_so_and_fails"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/diagram.rs](contract.md#output-882b777229b8)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hickory-cli/tests/diagram_cli.rs](contract.md#output-856fa0835344)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
