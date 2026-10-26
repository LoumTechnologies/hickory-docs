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

Walk `root` and deduce a topology from its code.

<storycode:copy id="f-882b777229b8-generate_topology">
/// Walk `root` and deduce a topology from its code.
pub fn generate_topology(root: &Path, group: Grouping) -> std::io::Result<Topology> {
    let mut files = Vec::new();
    for entry in ignore::WalkBuilder::new(root).build().flatten() {
        let path = entry.path();
        if !path.is_file() {
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
    let mut seen = std::collections::BTreeSet::new();
    for file in &files {
        let id = node_of(&file.path);
        if seen.insert(id.clone()) {
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

<storycode:file path="crates/hickory-cli/src/diagram.rs"><storycode:paste select="#f-882b777229b8-Grouping"/><storycode:paste select="#f-882b777229b8-MAX_FILE_BYTES"/><storycode:paste select="#f-882b777229b8-generate_topology"/><storycode:paste select="#f-882b777229b8-render"/><storycode:paste select="#f-882b777229b8-refresh_fragment"/><storycode:paste select="#f-882b777229b8-tests"/></storycode:file>

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
