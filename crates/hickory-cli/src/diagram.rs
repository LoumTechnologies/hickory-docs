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

/// Files too large to be source worth parsing (generated bundles, data).
const MAX_FILE_BYTES: u64 = 512 * 1024;

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

    // Nodes in walk order (stable for a given tree), edges counted.
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
                label: None,
                shape: None,
                fill: None,
                stroke: None,
            })
            .collect(),
        edges: counts
            .into_iter()
            .map(|((from, to), n)| SceneEdge {
                id: None,
                from,
                to,
                // Sides are a person's choice, recorded by the editor —
                // never invented here.
                from_side: None,
                to_side: None,
                label: (n > 1).then(|| format!("{n} refs")),
                style: None,
                arrow: None,
            })
            .collect(),
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn topo() -> Topology {
        Topology {
            nodes: vec![SceneNode {
                id: "api".into(),
                label: None,
                shape: None,
                fill: None,
                stroke: None,
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
