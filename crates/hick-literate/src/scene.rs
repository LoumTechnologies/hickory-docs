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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Text colour. Absent means the theme's own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneEdge {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub from: String,
    pub to: String,
    /// Which side of each node the end attaches to (top/right/bottom/left).
    /// A person's choice, recorded by the editor; a generator never sets it,
    /// and the mermaid downgrade ignores it — mermaid routes its own edges.
    #[serde(default, rename = "fromSide", skip_serializing_if = "Option::is_none")]
    pub from_side: Option<String>,
    #[serde(default, rename = "toSide", skip_serializing_if = "Option::is_none")]
    pub to_side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// solid (default), dashed, dotted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// Which end(s) wear an arrowhead: end (default), start, both, none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrow: Option<String>,
    /// Line colour, arrowheads included. Absent means the theme's own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
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
            (false, "start") => "<--",
            (false, _) => "-->",
            (true, "none") => "-.-",
            (true, "both") => "<-.->",
            (true, "start") => "<-.-",
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

// ---------------------------------------------------------------------------
// The SVG the weave draws.
// ---------------------------------------------------------------------------

const DEFAULT_W: i64 = 160;
const DEFAULT_H: i64 = 64;
const SLOT_SPACING: f64 = 18.0;
const PAD: i64 = 24;

/// Default ink for a document asset: neutral, readable on white — the woven
/// file is read outside the app, where no theme exists.
const INK_STROKE: &str = "#64748b";
const INK_FILL: &str = "#f8fafc";
const INK_TEXT: &str = "#1e293b";

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

struct Box_ {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Where each node sits: its layout entry, or a spot to the right of the
/// placed extent — the same rule the editor uses for an unplaced node.
fn boxes(nodes: &[SceneNode], layout: &BTreeMap<String, NodeLayout>) -> BTreeMap<String, Box_> {
    let mut out = BTreeMap::new();
    let mut right = f64::MIN;
    let mut top = f64::MAX;
    for node in nodes {
        if let Some(at) = layout.get(&node.id) {
            let w = at.w.unwrap_or(DEFAULT_W) as f64;
            right = right.max(at.x as f64 + w);
            top = top.min(at.y as f64);
        }
    }
    if right == f64::MIN {
        right = 0.0;
        top = 0.0;
    }
    let mut unplaced = 0;
    for node in nodes {
        let b = match layout.get(&node.id) {
            Some(at) => Box_ {
                x: at.x as f64,
                y: at.y as f64,
                w: at.w.unwrap_or(DEFAULT_W) as f64,
                h: at.h.unwrap_or(DEFAULT_H) as f64,
            },
            None => {
                let b = Box_ {
                    x: right + 60.0,
                    y: top + unplaced as f64 * (DEFAULT_H as f64 + 24.0),
                    w: DEFAULT_W as f64,
                    h: DEFAULT_H as f64,
                };
                unplaced += 1;
                b
            }
        };
        out.insert(node.id.clone(), b);
    }
    out
}

/// A recorded `left`/`left.N` side ref, split.
fn side_slot(reference: &str) -> Option<(&str, i64)> {
    let mut parts = reference.splitn(2, '.');
    let side = parts.next()?;
    if !matches!(side, "top" | "right" | "bottom" | "left") {
        return None;
    }
    let slot = match parts.next() {
        None => 0,
        Some(n) => n.parse().ok()?,
    };
    Some((side, slot))
}

/// The point a side ref names on a box — mirroring the editor: the occupied
/// slots of that side, sorted, centred on the side's midline, this slot's
/// ordinal deciding its offset.
fn anchor(b: &Box_, side: &str, slot: i64, occupied: &[i64]) -> (f64, f64) {
    let mut slots: Vec<i64> = occupied.to_vec();
    slots.sort_unstable();
    slots.dedup();
    let i = slots.iter().position(|s| *s == slot).unwrap_or(0) as f64;
    let k = slots.len().max(1) as f64;
    let offset = (i - (k - 1.0) / 2.0) * SLOT_SPACING;
    match side {
        "top" => (b.x + b.w / 2.0 + offset, b.y),
        "bottom" => (b.x + b.w / 2.0 + offset, b.y + b.h),
        "left" => (b.x, b.y + b.h / 2.0 + offset),
        _ => (b.x + b.w, b.y + b.h / 2.0 + offset),
    }
}

/// Mirror of the editor's rule: a corner you cannot render whole is a corner
/// you do not draw — short or near-aligned connectors go straight.
fn straight(source_side: &str, target_side: &str, dx: f64, dy: f64) -> bool {
    if dx.hypot(dy) < 96.0 {
        return true;
    }
    let h = |s: &str| s == "left" || s == "right";
    let v = |s: &str| s == "top" || s == "bottom";
    (h(source_side) && h(target_side) && dy.abs() < 24.0)
        || (v(source_side) && v(target_side) && dx.abs() < 24.0)
}

/// A rounded orthogonal path, or a straight line, between two anchors:
/// start, one or two axis-aligned corners, end — each corner rounded with a
/// radius clamped to half its shorter adjacent segment, so a whole corner is
/// drawn or the straightness rule above has already removed it.
fn edge_path(sx: f64, sy: f64, tx: f64, ty: f64, s_side: &str, t_side: &str) -> String {
    if straight(s_side, t_side, tx - sx, ty - sy) {
        return format!("M {sx:.1} {sy:.1} L {tx:.1} {ty:.1}");
    }
    let h = |s: &str| s == "left" || s == "right";
    let corners: Vec<(f64, f64)> = if h(s_side) && h(t_side) {
        let mid = (sx + tx) / 2.0;
        vec![(mid, sy), (mid, ty)]
    } else if !h(s_side) && !h(t_side) {
        let mid = (sy + ty) / 2.0;
        vec![(sx, mid), (tx, mid)]
    } else if h(s_side) {
        vec![(tx, sy)]
    } else {
        vec![(sx, ty)]
    };
    let mut pts = vec![(sx, sy)];
    pts.extend(corners);
    pts.push((tx, ty));
    let mut d = format!("M {sx:.1} {sy:.1}");
    for i in 1..pts.len() - 1 {
        let (px, py) = pts[i - 1];
        let (cx, cy) = pts[i];
        let (nx, ny) = pts[i + 1];
        // NOT f64::signum, whose signum(0.0) is 1.0: a zero-length axis
        // component must contribute zero offset, or every corner grows a
        // phantom 8px jog on the axis it does not travel.
        let sgn = |v: f64| {
            if v > 0.0 {
                1.0
            } else if v < 0.0 {
                -1.0
            } else {
                0.0
            }
        };
        let into = (cx - px).abs() + (cy - py).abs();
        let out = (nx - cx).abs() + (ny - cy).abs();
        let r = 8.0_f64.min(into / 2.0).min(out / 2.0);
        let ax = cx - sgn(cx - px) * r;
        let ay = cy - sgn(cy - py) * r;
        let bx = cx + sgn(nx - cx) * r;
        let by = cy + sgn(ny - cy) * r;
        d.push_str(&format!(
            " L {ax:.1} {ay:.1} Q {cx:.1} {cy:.1} {bx:.1} {by:.1}"
        ));
    }
    d.push_str(&format!(" L {tx:.1} {ty:.1}"));
    d
}

fn node_svg(node: &SceneNode, b: &Box_) -> String {
    let fill = node.fill.as_deref().unwrap_or(INK_FILL);
    let stroke = node.stroke.as_deref().unwrap_or(INK_STROKE);
    let text = node.text.as_deref().unwrap_or(INK_TEXT);
    let (x, y, w, h) = (b.x, b.y, b.w, b.h);
    let shape = match node.shape.as_deref().unwrap_or("rect") {
        "circle" => format!(
            r#"<ellipse cx="{:.1}" cy="{:.1}" rx="{:.1}" ry="{:.1}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#,
            x + w / 2.0,
            y + h / 2.0,
            w / 2.0,
            h / 2.0
        ),
        "round" | "pill" => format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="{:.1}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#,
            h / 2.0
        ),
        "diamond" => format!(
            r#"<polygon points="{:.1},{y:.1} {:.1},{:.1} {:.1},{:.1} {x:.1},{:.1}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#,
            x + w / 2.0,
            x + w,
            y + h / 2.0,
            x + w / 2.0,
            y + h,
            y + h / 2.0
        ),
        "hexagon" => format!(
            r#"<polygon points="{:.1},{y:.1} {:.1},{y:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {x:.1},{:.1}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#,
            x + 0.12 * w,
            x + 0.88 * w,
            x + w,
            y + h / 2.0,
            x + 0.88 * w,
            y + h,
            x + 0.12 * w,
            y + h,
            y + h / 2.0
        ),
        "cylinder" => format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="10" ry="18" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#
        ),
        _ => format!(
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="6" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#
        ),
    };
    format!(
        "{shape}<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"system-ui, sans-serif\" font-size=\"13\" fill=\"{text}\">{}</text>",
        x + w / 2.0,
        y + h / 2.0,
        esc(node.label.as_deref().unwrap_or(&node.id)),
    )
}

/// Draw the scene as a self-contained SVG — the woven form of a graph
/// diagram. Positions, sizes, shapes, and colours are the author's own, so
/// the picture in the woven markdown IS the picture in the editor.
pub fn to_svg(scene: &Scene) -> String {
    let (nodes, edges) = scene.topology();
    let boxes = boxes(nodes, &scene.layout);

    // Slot occupancy per (node, side), for anchor placement.
    let mut occupied: BTreeMap<(String, String), Vec<i64>> = BTreeMap::new();
    for edge in edges {
        for (node, side_ref) in [(&edge.from, &edge.from_side), (&edge.to, &edge.to_side)] {
            if let Some(reference) = side_ref
                && let Some((side, slot)) = side_slot(reference)
            {
                occupied
                    .entry((node.clone(), side.to_string()))
                    .or_default()
                    .push(slot);
            }
        }
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for b in boxes.values() {
        min_x = min_x.min(b.x);
        min_y = min_y.min(b.y);
        max_x = max_x.max(b.x + b.w);
        max_y = max_y.max(b.y + b.h);
    }
    if boxes.is_empty() {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 200.0;
        max_y = 100.0;
    }
    let width = max_x - min_x + 2.0 * PAD as f64;
    let height = max_y - min_y + 2.0 * PAD as f64;

    // One arrowhead marker per colour actually used.
    let mut colors: Vec<String> = edges
        .iter()
        .map(|e| e.color.clone().unwrap_or_else(|| INK_STROKE.to_string()))
        .collect();
    colors.sort();
    colors.dedup();
    let mut defs = String::new();
    for color in &colors {
        let id = color.trim_start_matches('#');
        defs.push_str(&format!(
            r#"<marker id="a-{id}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="{color}"/></marker>"#
        ));
    }

    let mut body = String::new();
    for edge in edges {
        let (Some(fb), Some(tb)) = (boxes.get(&edge.from), boxes.get(&edge.to)) else {
            continue;
        };
        // Recorded sides win; an unrecorded end faces the other box.
        let face = |from: &Box_, to: &Box_| -> String {
            let dx = (to.x + to.w / 2.0) - (from.x + from.w / 2.0);
            let dy = (to.y + to.h / 2.0) - (from.y + from.h / 2.0);
            if dx.abs() >= dy.abs() {
                if dx >= 0.0 { "right" } else { "left" }
            } else if dy >= 0.0 {
                "bottom"
            } else {
                "top"
            }
            .to_string()
        };
        let (s_side, s_slot) = edge
            .from_side
            .as_deref()
            .and_then(side_slot)
            .map(|(s, n)| (s.to_string(), n))
            .unwrap_or_else(|| (face(fb, tb), 0));
        let (t_side, t_slot) = edge
            .to_side
            .as_deref()
            .and_then(side_slot)
            .map(|(s, n)| (s.to_string(), n))
            .unwrap_or_else(|| (face(tb, fb), 0));
        let empty: Vec<i64> = Vec::new();
        let s_occ = occupied
            .get(&(edge.from.clone(), s_side.clone()))
            .unwrap_or(&empty);
        let t_occ = occupied
            .get(&(edge.to.clone(), t_side.clone()))
            .unwrap_or(&empty);
        let (sx, sy) = anchor(fb, &s_side, s_slot, s_occ);
        let (tx, ty) = anchor(tb, &t_side, t_slot, t_occ);
        let color = edge.color.as_deref().unwrap_or(INK_STROKE);
        let marker = color.trim_start_matches('#');
        let arrow = edge.arrow.as_deref().unwrap_or("end");
        let dash = match edge.style.as_deref() {
            Some("dashed") => r#" stroke-dasharray="8 5""#,
            Some("dotted") => r#" stroke-dasharray="2 5""#,
            _ => "",
        };
        let marker_end = if arrow == "end" || arrow == "both" {
            format!(r#" marker-end="url(#a-{marker})""#)
        } else {
            String::new()
        };
        let marker_start = if arrow == "start" || arrow == "both" {
            format!(r#" marker-start="url(#a-{marker})""#)
        } else {
            String::new()
        };
        body.push_str(&format!(
            r#"<path d="{}" fill="none" stroke="{color}" stroke-width="1.5"{dash}{marker_end}{marker_start}/>"#,
            edge_path(sx, sy, tx, ty, &s_side, &t_side)
        ));
        if let Some(label) = edge.label.as_deref().filter(|l| !l.is_empty()) {
            let (lx, ly) = ((sx + tx) / 2.0, (sy + ty) / 2.0);
            let w = label.chars().count() as f64 * 6.5 + 10.0;
            body.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"16\" rx=\"3\" fill=\"#ffffff\" fill-opacity=\"0.85\"/><text x=\"{lx:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"system-ui, sans-serif\" font-size=\"11\" fill=\"{color}\">{}</text>",
                lx - w / 2.0,
                ly - 8.0,
                ly + 3.5,
                esc(label),
            ));
        }
    }
    for node in nodes {
        if let Some(b) = boxes.get(&node.id) {
            body.push_str(&node_svg(node, b));
        }
    }

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.1} {:.1} {width:.1} {height:.1}\" width=\"{width:.0}\" height=\"{height:.0}\">\n<defs>{defs}</defs>\n{body}\n</svg>\n",
        min_x - PAD as f64,
        min_y - PAD as f64,
    )
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
    fn the_svg_draws_the_authors_layout_with_whole_corners() {
        let s = scene(
            r##"{
              "nodes": [
                {"id": "a", "label": "Alpha", "stroke": "#F8766D"},
                {"id": "b", "shape": "cylinder"}
              ],
              "edges": [
                {"from": "a", "fromSide": "top", "to": "b", "toSide": "left", "label": "SQL", "arrow": "both"}
              ],
              "layout": {
                "a": {"x": 100, "y": 0, "w": 100, "h": 50},
                "b": {"x": 300, "y": 200, "w": 100, "h": 50}
              }
            }"##,
        );
        let svg = to_svg(&s);
        assert!(svg.starts_with("<svg"), "{svg}");
        assert!(svg.contains(">Alpha</text>"), "{svg}");
        assert!(svg.contains(">SQL</text>"), "{svg}");
        assert!(svg.contains("marker-start"), "{svg}");
        // The author's layout is in the picture…
        assert!(svg.contains("300"), "{svg}");
        // …and the corner is WHOLE: from a's top (150,0) to b's left
        // (300,225), the elbow turns at (150,225) with no phantom jog on the
        // axis it does not travel (f64::signum(0.0) is 1.0 — the regression
        // this pins).
        assert!(
            svg.contains("L 150.0 217.0 Q 150.0 225.0 158.0 225.0"),
            "{svg}"
        );
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
