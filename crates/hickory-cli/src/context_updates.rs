//! Explicit user updates. Queueing never certifies model delivery.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct Exposure {
    pub reference: String,
    #[serde(default)]
    pub unique: bool,
    pub source: hick_context_protocol::Source,
    pub text: String,
    pub start: u64,
    pub role: String,
    pub scope: String,
    pub request: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Update {
    pub id: String,
    pub old: String,
    pub source: hick_context_protocol::Source,
    pub text: Option<String>,
    pub role: String,
    pub scope: String,
    pub state: String,
    pub request: Option<String>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Branch {
    pub exposures: Vec<Exposure>,
    pub updates: Vec<Update>,
}
#[derive(Default)]
pub struct Inbox(
    pub BTreeMap<String, Branch>,
    BTreeMap<String, std::path::PathBuf>,
    Option<std::path::PathBuf>,
);
impl Inbox {
    pub fn open(&mut self, session: &std::path::Path) -> anyhow::Result<()> {
        let path = session.with_extension("context-updates.json");
        if self.2.as_ref() == Some(&path) {
            return Ok(());
        }
        let mut branches: BTreeMap<String, Branch> = match std::fs::read(&path) {
            Ok(raw) => serde_json::from_slice(&raw)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(e) => return Err(e.into()),
        };
        for branch in branches.values_mut() {
            for update in &mut branch.updates {
                if matches!(
                    update.state.as_str(),
                    "preparing" | "prepared" | "transport_started"
                ) {
                    update.state = "uncertain".into();
                }
            }
        }
        for (id, branch) in branches {
            if !self.0.contains_key(&id) {
                self.1.insert(id.clone(), path.clone());
                self.0.insert(id, branch);
            }
        }
        self.2 = Some(path);
        Ok(())
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let mut files: BTreeMap<&std::path::PathBuf, BTreeMap<&String, &Branch>> = BTreeMap::new();
        for (id, branch) in &self.0 {
            if let Some(path) = self.1.get(id) {
                files.entry(path).or_default().insert(id, branch);
            }
        }
        for (path, branches) in files {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let temporary = path.with_extension("json.tmp");
            std::fs::write(&temporary, serde_json::to_vec(&branches)?)?;
            std::fs::rename(&temporary, path)?;
        }
        Ok(())
    }
    pub fn fork(&mut self, branch: &str, parent: Option<&str>) {
        let prior = parent
            .and_then(|p| self.0.get(p))
            .cloned()
            .unwrap_or_default();
        self.0.entry(branch.into()).or_insert(prior);
        if let Some(path) = &self.2 {
            self.1.entry(branch.into()).or_insert(path.clone());
        }
    }
    pub fn take(&mut self, branch: &str) -> Vec<Update> {
        self.0
            .entry(branch.into())
            .or_default()
            .updates
            .iter_mut()
            .filter(|u| u.state == "queued")
            .map(|u| {
                u.state = "preparing".into();
                u.clone()
            })
            .collect()
    }
    pub fn delivery(&mut self, branch: &str, request: &str, state: &str, references: &[String]) {
        for update in &mut self.0.entry(branch.into()).or_default().updates {
            if references.contains(&update.id) && update.state != "provider_confirmed" {
                update.state = state.into();
                update.request = Some(request.into());
            }
        }
    }
    pub fn publish(&mut self, branch: &str, exposures: Vec<Exposure>) {
        let current = &mut self.0.entry(branch.into()).or_default().exposures;
        for exposure in exposures {
            // The last supplied passage wins only for that exact original range.
            current.retain(|old| {
                old.source.id != exposure.source.id
                    || !(old.start >= exposure.start
                        && old.start + old.text.len() as u64
                            <= exposure.start + exposure.text.len() as u64)
            });
            current.push(exposure);
        }
    }
}
/// Preserve request order while walking derived inputs; captures alone are not exposure.
pub fn exposures(manifest: &Value) -> Vec<Exposure> {
    fn visit(
        id: &str,
        nodes: &[Value],
        role: &str,
        scope: &str,
        request: &str,
        seen: &mut Vec<String>,
        out: &mut Vec<Exposure>,
    ) {
        if seen.iter().any(|s| s == id) {
            return;
        }
        seen.push(id.into());
        let Some(node) = nodes.iter().find(|n| n["id"] == id) else {
            return;
        };
        if let (Ok(source), Some(text), Some(start)) = (
            serde_json::from_value(node["source"].clone()),
            node["text"].as_str(),
            node["source_span"]["start"].as_u64(),
        ) {
            out.push(Exposure {
                reference: id.into(),
                unique: false,
                source,
                text: text.into(),
                start,
                role: role.into(),
                scope: scope.into(),
                request: request.into(),
            });
        }
        if node["derivation"] == "model_output" {
            return;
        }
        if let Some(parents) = node["parents"].as_array() {
            for parent in parents {
                if let Some(id) = parent.as_str() {
                    visit(id, nodes, role, scope, request, seen, out);
                }
            }
        }
    }
    let mut out = vec![];
    let Some(nodes) = manifest["nodes"].as_array() else {
        return out;
    };
    if let Some(parts) = manifest["request"]["parts"].as_array() {
        for part in parts {
            if let Some(segments) = part["segments"].as_array() {
                for segment in segments {
                    if let Some(id) = segment["node"].as_str() {
                        visit(
                            id,
                            nodes,
                            part["role"].as_str().unwrap_or("user"),
                            part["scope"].as_str().unwrap_or("conversation"),
                            manifest["request"]["id"].as_str().unwrap_or(""),
                            &mut vec![],
                            &mut out,
                        );
                    }
                }
            }
        }
    }
    out
}

/// Conservative mapping: never infer which duplicate passage the user meant.
pub fn compare(old: &Exposure, current: Option<&str>) -> (&'static str, String) {
    let Some(current) = current else {
        return ("deleted_or_unavailable", String::new());
    };
    let revision = hick_context_protocol::digest(current.as_bytes());
    if revision == old.source.revision {
        return ("unchanged", revision);
    }
    let positions: Vec<_> = current
        .char_indices()
        .filter_map(|(i, _)| current[i..].starts_with(&old.text).then_some(i))
        .take(2)
        .collect();
    let status = if old.text.is_empty() || (!positions.is_empty() && !old.unique) {
        "ambiguous"
    } else {
        match positions.as_slice() {
            [] => "changed_observed_passage",
            [position] if *position as u64 == old.start => "changed_elsewhere",
            [_] => "moved",
            _ => "ambiguous",
        }
    };
    (status, revision)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/explicit-context-updates.md
    #[test]
    fn queue_and_failure_do_not_advance_baseline_or_cross_branches() {
        let mut inbox = Inbox::default();
        inbox.fork("a", None);
        let source = hick_context_protocol::Source {
            id: "file".into(),
            kind: "saved_file".into(),
            locator: "AGENTS.md".into(),
            revision: "old".into(),
        };
        inbox.publish(
            "a",
            vec![Exposure {
                source: source.clone(),
                text: "old".into(),
                reference: "n".into(),
                unique: true,
                start: 0,
                role: "user".into(),
                scope: "child".into(),
                request: "r0".into(),
            }],
        );
        inbox.0.get_mut("a").unwrap().updates.push(Update {
            id: "u".into(),
            old: "n".into(),
            source,
            text: Some("second".into()),
            role: "user".into(),
            scope: "child".into(),
            state: "queued".into(),
            request: None,
        });
        assert_eq!(inbox.take("a").len(), 1);
        assert!(inbox.take("a").is_empty());
        inbox.delivery("a", "r", "failed", &["u".into()]);
        assert_eq!(inbox.0["a"].exposures[0].text, "old");
        inbox.fork("b", Some("a"));
        inbox.delivery("a", "r2", "included", &["u".into()]);
        assert_eq!(inbox.0["b"].updates[0].state, "failed");
    }
    // Guarantee: docs/guarantees/agent/explicit-context-updates.md
    #[test]
    fn comparison_distinguishes_movement_duplicates_and_edits_elsewhere() {
        let old = Exposure {
            reference: "n".into(),
            unique: true,
            source: hick_context_protocol::Source {
                id: "f".into(),
                kind: "saved_file".into(),
                locator: "f".into(),
                revision: hick_context_protocol::digest(b"read\nother"),
            },
            text: "read".into(),
            start: 0,
            role: "user".into(),
            scope: "conversation".into(),
            request: "r".into(),
        };
        assert_eq!(compare(&old, Some("read\nchanged")).0, "changed_elsewhere");
        assert_eq!(compare(&old, Some("prefix\nread")).0, "moved");
        assert_eq!(compare(&old, Some("read\nread")).0, "ambiguous");
        assert_eq!(
            compare(&old, Some("replacement")).0,
            "changed_observed_passage"
        );
        assert_eq!(compare(&old, None).0, "deleted_or_unavailable");
        let mut uncertain = old.clone();
        uncertain.unique = false;
        assert_eq!(compare(&uncertain, Some("prefix\nread")).0, "ambiguous");
    }
    // Guarantee: docs/guarantees/agent/explicit-context-updates.md
    #[test]
    fn two_sessions_persist_separately_and_do_not_clear_active_branches() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.md");
        let b = dir.path().join("b.md");
        let mut inbox = Inbox::default();
        inbox.open(&a).unwrap();
        inbox.fork("a", None);
        inbox.open(&b).unwrap();
        inbox.fork("b", None);
        inbox.save().unwrap();
        assert!(inbox.0.contains_key("a"));
        let mut restarted = Inbox::default();
        restarted.open(&a).unwrap();
        assert!(restarted.0.contains_key("a"));
        assert!(!restarted.0.contains_key("b"));
    }
}
