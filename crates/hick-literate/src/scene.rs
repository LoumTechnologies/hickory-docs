//! The `renderer="graph"` scene: a diagram you can drag.
//!
//! A mermaid diagram is text, which is what makes it checkable — and what
//! makes it impossible to lay out by hand. The graph renderer's body is a
//! JSON scene instead, split on one line: **topology is what a generator can
//! deduce, layout is what only a person decides.** `nodes` and `edges` (or a
//! whole `topology` object arriving through a `<hick:paste>`) carry ids,
//! labels, and shapes; `layout` maps node id → position, and is the only part
//! the interactive editor rewrites for a derived scene.
//!
//! Ids are semantic (`"api"`, never `n_1700000000000_a1b2`): they are the join
//! key that lets a re-run of the generator keep your layout, and they keep the
//! canonical serialization line-stable under git.
//!
//! The schema is written fresh here; its shape is informed by the pure-data
//! format documented in grafly's `GRAFLY_DIAGRAM_FORMAT.md` (grafly itself is
//! AGPL and none of its code was read or used — see
//! `docs/specs/freeform/a-diagram-you-can-drag.md` for the provenance note).
//!
//! The weave downgrades a scene to a mermaid fence: positions are dropped,
//! which is honest — markdown has nowhere to keep them — and the picture
//! still renders anywhere markdown renders.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The `renderer` value this module owns.
pub const GRAPH_RENDERER: &str = "graph";

/// A whole scene: inline topology (`nodes`/`edges`) or a derived one
/// (`topology`, usually pasted from a fragment a generator wrote), plus the
/// human-owned `layout`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Scene {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<SceneNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<SceneEdge>,
    /// A derived scene's topology, arriving whole. Mutually exclusive with
    /// inline `nodes`/`edges` — a scene with both has two sources of truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topology: Option<Topology>,
    /// Node id → where the person put it. BTreeMap so serialization is
    /// deterministic whatever order the editor visited nodes in.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub layout: BTreeMap<String, NodeLayout>,
}

/// What a generator can deduce, as one value — so a paste can supply it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Topology {
    #[serde(default)]
    pub nodes: Vec<SceneNode>,
    #[serde(default)]
    pub edges: Vec<SceneEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// rect (default), round, pill, circle, diamond, hexagon, cylinder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEdge {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// solid (default), dashed, dotted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// end (default), none, both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrow: Option<String>,
}

/// Where a node sits, integer pixels in the scene's own plane.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NodeLayout {
    pub x: i64,
    pub y: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub w: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<i64>,
}

impl Scene {
    /// The topology, wherever it lives.
    pub fn topology(&self) -> (&[SceneNode], &[SceneEdge]) {
        match &self.topology {
            Some(t) => (&t.nodes, &t.edges),
            None => (&self.nodes, &self.edges),
        }
    }

    /// Whether the topology arrived whole (derived) rather than inline.
    pub fn is_derived(&self) -> bool {
        self.topology.is_some()
    }
}

/// Parse a scene body, refusing the shapes that make a document lie.
pub fn parse_scene(body: &str) -> Result<Scene, String> {
    let scene: Scene = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if scene.topology.is_some() && (!scene.nodes.is_empty() || !scene.edges.is_empty()) {
        return Err(
            "a scene carries either inline nodes/edges or a derived topology, never both — \
             two topologies would be two sources of truth"
                .to_string(),
        );
    }
    Ok(scene)
}

/// Human-readable problems with a scene that still parses: duplicate ids,
/// edges naming nodes that do not exist. Warnings, never errors — a
/// half-drawn diagram is the common case while someone is working.
pub fn scene_warnings(scene: &Scene) -> Vec<String> {
    let (nodes, edges) = scene.topology();
    let mut warnings = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for node in nodes {
        if !seen.insert(node.id.as_str()) {
            warnings.push(format!("duplicate node id '{}'", node.id));
        }
    }
    for edge in edges {
        for end in [&edge.from, &edge.to] {
            if !seen.contains(end.as_str()) {
                warnings.push(format!("edge names a node that does not exist: '{end}'"));
            }
        }
    }
    for id in scene.layout.keys() {
        if !seen.contains(id.as_str()) {
            warnings.push(format!("layout for a node that does not exist: '{id}'"));
        }
    }
    warnings
}

/// A mermaid identifier for a scene node id: alphanumerics and `_` survive,
/// everything else becomes `_`, and a collision or an empty result gets the
/// node's ordinal appended so two nodes never merge in the picture.
fn mermaid_ids(nodes: &[SceneNode]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut taken = std::collections::BTreeSet::new();
    for (ordinal, node) in nodes.iter().enumerate() {
        let mut id: String = node
            .id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        if id.is_empty() || taken.contains(&id) {
            id = format!("{id}_{ordinal}");
        }
        taken.insert(id.clone());
        out.insert(node.id.clone(), id);
    }
    out
}

fn mermaid_label(text: &str) -> String {
    // Inside mermaid's double quotes, a double quote is `#quot;`.
    text.replace('"', "#quot;")
}

/// The weave's downgrade: a deterministic `flowchart TD`. Positions and
/// colours are dropped — markdown has nowhere honest to keep them — and the
/// spec says so plainly rather than letting the fence imply fidelity.
pub fn to_mermaid(scene: &Scene) -> String {
    let (nodes, edges) = scene.topology();
    let ids = mermaid_ids(nodes);
    let mut out = String::from("flowchart TD\n");
    for node in nodes {
        let id = &ids[&node.id];
        let label = mermaid_label(node.label.as_deref().unwrap_or(&node.id));
        let drawn = match node.shape.as_deref().unwrap_or("rect") {
            "round" | "pill" => format!("{id}(\"{label}\")"),
            "circle" => format!("{id}((\"{label}\"))"),
            "diamond" => format!("{id}{{\"{label}\"}}"),
            "hexagon" => format!("{id}{{{{\"{label}\"}}}}"),
            "cylinder" => format!("{id}[(\"{label}\")]"),
            _ => format!("{id}[\"{label}\"]"),
        };
        out.push_str("  ");
        out.push_str(&drawn);
        out.push('\n');
    }
    for edge in edges {
        let from = ids
            .get(&edge.from)
            .cloned()
            .unwrap_or_else(|| edge.from.clone());
        let to = ids
            .get(&edge.to)
            .cloned()
            .unwrap_or_else(|| edge.to.clone());
        let dashed = matches!(edge.style.as_deref(), Some("dashed") | Some("dotted"));
        let arrow = edge.arrow.as_deref().unwrap_or("end");
        let connector = match (dashed, arrow) {
            (false, "none") => "---",
            (false, "both") => "<-->",
            (false, _) => "-->",
            (true, "none") => "-.-",
            (true, "both") => "<-.->",
            (true, _) => "-.->",
        };
        out.push_str("  ");
        out.push_str(&from);
        out.push(' ');
        out.push_str(connector);
        match &edge.label {
            Some(label) if !label.is_empty() => {
                out.push_str(&format!("|\"{}\"| ", mermaid_label(label)));
            }
            _ => out.push(' '),
        }
        out.push_str(&to);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(json: &str) -> Scene {
        parse_scene(json).expect("scene parses")
    }

    #[test]
    fn downgrade_is_deterministic_and_complete() {
        let s = scene(
            r#"{
              "nodes": [
                {"id": "api", "label": "API server"},
                {"id": "db", "label": "Postgres", "shape": "cylinder"},
                {"id": "ok?", "shape": "diamond"}
              ],
              "edges": [
                {"from": "api", "to": "db", "label": "SQL"},
                {"from": "api", "to": "ok?", "style": "dashed"}
              ],
              "layout": {"api": {"x": 0, "y": 0, "w": 160, "h": 64}}
            }"#,
        );
        let mermaid = to_mermaid(&s);
        assert_eq!(
            mermaid,
            "flowchart TD\n  api[\"API server\"]\n  db[(\"Postgres\")]\n  ok_{\"ok?\"}\n  api -->|\"SQL\"| db\n  api -.-> ok_\n"
        );
        // Positions never leak into the fence.
        assert!(!mermaid.contains("160"));
    }

    #[test]
    fn two_topologies_are_refused() {
        let err = parse_scene(
            r#"{"nodes": [{"id": "a"}], "topology": {"nodes": [{"id": "a"}], "edges": []}}"#,
        )
        .unwrap_err();
        assert!(err.contains("never both"), "{err}");
    }

    #[test]
    fn warnings_name_the_broken_reference() {
        let s = scene(
            r#"{
              "nodes": [{"id": "a"}, {"id": "a"}],
              "edges": [{"from": "a", "to": "ghost"}],
              "layout": {"gone": {"x": 1, "y": 2}}
            }"#,
        );
        let warnings = scene_warnings(&s);
        assert!(warnings.iter().any(|w| w.contains("duplicate node id 'a'")));
        assert!(warnings.iter().any(|w| w.contains("'ghost'")));
        assert!(warnings.iter().any(|w| w.contains("'gone'")));
    }

    #[test]
    fn derived_topology_reads_like_inline() {
        let s = scene(
            r#"{
              "topology": {"nodes": [{"id": "a"}, {"id": "b"}], "edges": [{"from": "a", "to": "b"}]},
              "layout": {"a": {"x": 10, "y": 10}}
            }"#,
        );
        assert!(s.is_derived());
        let (nodes, edges) = s.topology();
        assert_eq!(nodes.len(), 2);
        assert_eq!(edges.len(), 1);
        assert!(to_mermaid(&s).contains("a --> b"));
    }

    #[test]
    fn colliding_sanitized_ids_stay_apart() {
        let s = scene(
            r#"{"nodes": [{"id": "a b"}, {"id": "a-b"}], "edges": [{"from": "a b", "to": "a-b"}]}"#,
        );
        let mermaid = to_mermaid(&s);
        assert!(mermaid.contains("a_b[\"a b\"]"));
        assert!(mermaid.contains("a_b_1[\"a-b\"]"));
        assert!(mermaid.contains("a_b --> a_b_1"));
    }
}
