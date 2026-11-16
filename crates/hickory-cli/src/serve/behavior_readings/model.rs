use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub(super) fn id(kind: &str, path: &str) -> String {
    format!("{kind}:{}", hash(path.as_bytes()))
}
pub(super) fn synthetic_key(path:&str,fragment:&str)->String {serde_json::to_string(&(path,fragment)).expect("string tuples serialize")}
#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub source_path: Option<String>,
    pub fragment_id: Option<String>,
    pub text_origin:Option<String>,
    pub reading_notice:Option<String>,
    pub label: String,
    pub status: String,
    pub hash: Option<String>,
    pub preview: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub relationship: String,
    pub basis: String,
    pub reason: String,
    pub authority: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub label: String,
    pub path: String,
    pub available: bool,
    pub reason: String,
    pub candidate_id: Option<String>,
    pub analysis_fingerprint: Option<String>,
}
#[derive(Default, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: BTreeMap<String, Node>,
    pub edges: Vec<Edge>,
    pub checks: Vec<Check>,
    pub notices: Vec<String>,
    pub truncated: bool,
    pub metadata_hashes: BTreeMap<String, String>,
}
impl Graph {
    pub(super) fn node(&mut self, kind: &str, path: &str, label: &str, status: &str) -> String {
        let key = id(kind, path);
        if self.nodes.contains_key(&key) {
            return key;
        }
        if self.nodes.len() >= 2048 {
            self.truncated = true;
            return key;
        }
        self.nodes.insert(
            key.clone(),
            Node {
                id: key.clone(),
                kind: kind.into(),
                path: path.into(),
                source_path: Some(path.into()),
                fragment_id:None,
                text_origin:None,reading_notice:None,
                label: label.chars().take(512).collect(),
                status: status.into(),
                hash: None,
                preview: None,
            },
        );
        key
    }
    pub(super) fn edge(
        &mut self,
        from: &str,
        to: &str,
        relation: &str,
        basis: &str,
        reason: &str,
        authority: &str,
    ) {
        if self.edges.len() >= 8192 {
            self.truncated = true;
            return;
        }
        if !self.nodes.contains_key(from) || !self.nodes.contains_key(to) {
            return;
        }
        let next = Edge {
            from: from.into(),
            to: to.into(),
            relationship: relation.into(),
            basis: basis.into(),
            reason: reason.into(),
            authority: authority.into(),
        };
        if !self
            .edges
            .iter()
            .any(|e| e.from == next.from && e.to == next.to && e.relationship == next.relationship)
        {
            self.edges.push(next)
        }
    }
    pub(super) fn metadata(&mut self, map: &Value, narrative: &Value, paths: &BTreeSet<String>) {
        if let Some(contracts) = map["contracts"].as_object() {
            for (key, c) in contracts {
                let related = c["references"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .chain(c["owned"].as_array().into_iter().flatten())
                    .any(|p| p.as_str().is_some_and(|p| paths.contains(p)));
                if !related {
                    continue;
                }
                let page = c["page"].as_str().unwrap_or("");
                let status = match c["disposition"].as_str() {
                    Some("historical") => "historical",
                    Some("retained-requirement" | "reconciled-requirement" | "rewritten") => {
                        "canonical-declared"
                    }
                    _ => "declared-status-unknown",
                };
                let contract = self.node("contract", &synthetic_key(page,key), c["title"].as_str().unwrap_or(key), status);
                if let Some(node)=self.nodes.get_mut(&contract){node.path=page.into();node.source_path=Some(page.into());}
                for p in c["references"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .chain(c["owned"].as_array().into_iter().flatten())
                    .filter_map(Value::as_str)
                {
                    if paths.contains(p) {
                        let target = self.node("source", p, p, "captured");
                        self.edge(&contract,&target,"declares-related-source","declared","Contract metadata references this source; this is not semantic coverage.","hickory-dogfood/behavior-map.json");
                    }
                }
                for test in c["tests"].as_array().into_iter().flatten() {
                    if self.checks.len() >= 1024 {
                        self.truncated = true;
                        break;
                    }
                    if let Some(file) = test["file"].as_str() {
                        let key = serde_json::to_string(&(file,&test["runner"],&test["name"])).expect("JSON metadata serializes");
                        let check = id("check", &key);
                        let node = check.clone();
                        if self.nodes.len() < 2048 {
                            self.nodes.insert(
                                node.clone(),
                                Node {
                                    id: node.clone(),
                                    kind: "check".into(),
                                    path: file.into(),
                                    source_path: Some(file.into()),
                                    fragment_id:None,
                text_origin:None,reading_notice:None,
                                    label: test["name"].as_str().unwrap_or(file).into(),
                                    status: "declared".into(),
                                    hash: None,
                                    preview: None,
                                },
                            );
                        } else {
                            self.truncated = true;
                        }
                        self.edge(&contract,&node,"declares-check","declared","This exact runner/file/name was declared by contract metadata; execution support is checked separately.","hickory-dogfood/behavior-map.json");
                        if !self.checks.iter().any(|c| c.id == check) {
                            self.checks.push(Check{id:check,label:test["name"].as_str().unwrap_or(file).into(),path:file.into(),available:false,reason:format!("Declared runner {} has no exact admitted source candidate yet.",test["runner"].as_str().unwrap_or("unknown")),candidate_id:None,analysis_fingerprint:None});
                        }
                    }
                }
            }
        }
        if let Some(pairs) = narrative["paired_test_modules"].as_array() {
            for pair in pairs {
                let Some(implementation) = pair["implementation"].as_str() else {
                    continue;
                };
                if !paths.contains(implementation) {
                    continue;
                }
                if let Some(test) = pair["tests"].as_str() {
                    let a = self.node("source", implementation, implementation, "captured");
                    let b = self.node("test-module", test, test, "declared");
                    self.edge(&a,&b,"paired-test-module","declared","Narrative metadata explicitly pairs these modules; this does not establish execution or assertion quality.","hickory-dogfood/narrative-map.json");
                }
            }
        }
        if let Some(fragments) = narrative["fragments"].as_array() {
            for fragment in fragments {
                let Some(output) = fragment["output"].as_str() else {
                    continue;
                };
                if !paths.contains(output) {
                    continue;
                }
                let page = fragment["page"].as_str().unwrap_or("");
                let symbol = fragment["symbol"].as_str().unwrap_or("");
                let fragment_id=fragment["id"].as_str().unwrap_or(symbol);
                let key = synthetic_key(page,fragment_id);
                let basis = fragment["basis"].as_str().unwrap_or("needs-explanation");
                let status = if basis == "needs-explanation" {
                    "needs-explanation"
                } else {
                    "explanation-declared"
                };
                let a = self.node("explanation", &key, symbol, status);
                if let Some(node) = self.nodes.get_mut(&a) {
                    node.path=format!("{page}#{fragment_id}");node.fragment_id=Some(fragment_id.into());
                    node.source_path = Some(page.into());
                }
                let b = self.node("source", output, output, "captured");
                self.edge(
                    &a,
                    &b,
                    "explains",
                    "declared",
                    basis,
                    "hickory-dogfood/narrative-map.json",
                );
            }
        }
    }
    pub(super) fn impact(&self, focus: &str) -> Value {
        let mut reached: BTreeSet<String> = self
            .nodes
            .values()
            .filter(|n| n.path == focus)
            .map(|n| n.id.clone())
            .collect();
        let mut links = Vec::new();
        let mut complete = false;
        for _ in 0..8 {
            let before = reached.len();
            for edge in &self.edges {
                if reached.contains(&edge.from) || reached.contains(&edge.to) {
                    reached.insert(edge.from.clone());
                    reached.insert(edge.to.clone());
                    if !links.iter().any(|e: &&Edge| {
                        e.from == edge.from
                            && e.to == edge.to
                            && e.relationship == edge.relationship
                    }) {
                        links.push(edge)
                    }
                }
            }
            if reached.len() == before {
                complete = true;
                break;
            }
        }
        json!({"basis":"inferred","reason":"Related through the displayed compiler or declared-metadata edges; not a claim of executed reachability.","nodes":reached,"via":links,"depth_limit":8,"truncated":!complete})
    }
}
impl Graph {
    /// Add observation links after computing the input/admission fingerprint.
    /// An independently arriving receipt must not change source authority.
    pub(super) fn observations(&mut self,evidence:&Value,receipts:&Value,fingerprint:&str,budget:&mut usize){
        for record in evidence["records"].as_array().into_iter().flatten(){
            let Some(record_id)=record["id"].as_str() else {continue};
            let related=record["matching_inputs"].as_array().into_iter().flatten().filter_map(Value::as_str).any(|path|self.nodes.contains_key(&id("source",path)));
            if !related {continue}
            let key=self.node("evidence",&format!("coverage:{record_id}"),record["collector"].as_str().unwrap_or("Retained coverage record"),record["freshness"].as_str().unwrap_or("unknown"));
            self.observation_preview(&key,record,budget);
            for path in record["matching_inputs"].as_array().into_iter().flatten().filter_map(Value::as_str){let source=id("source",path);self.edge(&source,&key,"recorded-input",if record["origin"]=="current-workspace"{"observed"}else{"declared"},"The retained record names this input. Workspace origin and byte freshness remain separate; this does not prove current-source execution or frozen inputs.","coverage record");}
            for reach in record["reachability"].as_array().into_iter().flatten(){let Some(path)=reach["path"].as_str()else{continue};let source=id("source",path);let automatic=record["evidence_kind"]=="automated-collector"&&record["origin"]=="current-workspace";self.edge(&source,&key,"reported-line-reachability",if automatic{"observed"}else{"declared"},if automatic{"The collector reports line reachability for this file. Attribution and source freshness remain separate; no guarantee or assertion-quality relationship is inferred."}else{"This manual, imported or origin-unconfirmed report declares line reachability. It is not current-workspace automated verification or a collected test verdict."},"retained collector report");}
        }
        for receipt in receipts["records"].as_array().into_iter().flatten(){let (Some(request),Some(check))=(receipt["request_id"].as_str(),receipt["check_id"].as_str())else{continue};if !self.nodes.contains_key(check){continue}let owned=receipt["origin"]=="current-workspace";let current=owned&&receipt["reading_fingerprint"]==fingerprint;let key=self.node("verification",&format!("request:{request}"),receipt["label"].as_str().unwrap_or(request),if current{"current-reading-receipt"}else if owned{"historical-reading-receipt"}else{"historical-origin-unknown"});self.observation_preview(&key,receipt,budget);self.edge(check,&key,"verification-request",if owned{"observed"}else{"declared"},if owned{"The owned retained request explicitly selected this check. A process outcome is not a collected test-case verdict."}else{"This retained receipt declares a historical check association, but its workspace origin and live process are unconfirmed. This is not a collected test-case verdict."},"guarded verification receipt");}
    }
    fn observation_preview(&mut self,key:&str,value:&Value,budget:&mut usize){if let Some(node)=self.nodes.get_mut(key){node.source_path=None;node.text_origin=Some("retained-summary".into());let raw=value.to_string();node.hash=Some(hash(raw.as_bytes()));let mut preview=String::new();for character in raw.chars().take(2000){if preview.len()+character.len_utf8()>*budget{break}preview.push(character)}*budget=budget.saturating_sub(preview.len());node.preview=Some(preview);}}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_declared_relationships_create_checks() {
        let mut graph = Graph::default();
        graph.metadata(&json!({"contracts":{"c":{"page":"c.md","references":["a.py"],"tests":[{"runner":"pytest","file":"test_a.py","name":"test_a"}]}}}),&json!({}),&BTreeSet::from(["a.py".into(),"test_unrelated.py".into()]));
        assert_eq!(graph.checks.len(), 1);
        assert!(!graph.checks[0].available);
        assert!(!graph.nodes.values().any(|n| n.path == "test_unrelated.py"));
        assert!(graph.edges.iter().all(|e| e.basis == "declared"));
    }
    #[test]
    fn literal_hash_names_do_not_alias_synthetic_fragment_or_shared_contract_identities(){let mut graph=Graph::default();graph.metadata(&json!({"contracts":{"one":{"page":"same.md","references":["a.py"]},"two":{"page":"same.md","references":["a.py"]}}}),&json!({"fragments":[{"page":"a#b","id":"c","output":"a.py"},{"page":"a","id":"b#c","output":"a.py"}]}),&BTreeSet::from(["a.py".into()]));assert_eq!(graph.nodes.values().filter(|node|node.kind=="explanation").count(),2);assert_eq!(graph.nodes.values().filter(|node|node.kind=="contract").count(),2);assert_ne!(id("fragment",&synthetic_key("a#b","c")),id("fragment",&synthetic_key("a","b#c")));}

    #[test]
    fn observations_keep_manual_reachability_and_process_outcomes_distinct(){let mut graph=Graph::default();let source=graph.node("source","a#literal.py","a#literal.py","captured");assert_eq!(graph.nodes[&source].source_path.as_deref(),Some("a#literal.py"));let check=graph.node("check","selected","selected","declared");let evidence=json!({"records":[{"id":"manual","collector":"import","freshness":"origin-unknown","evidence_kind":"manual-or-imported","matching_inputs":["a#literal.py"],"reachability":[{"path":"a#literal.py","hit_lines":1}]}]});let receipts=json!({"records":[{"request_id":"owned","origin":"current-workspace","check_id":check,"reading_fingerprint":"old","label":"selected","status":"process-finished","outcome":"process-only"}]});let mut budget=1024;graph.observations(&evidence,&receipts,"new",&mut budget);assert!(graph.edges.iter().any(|edge|edge.relationship=="reported-line-reachability"&&edge.basis=="declared"));assert!(graph.nodes.values().any(|node|node.status=="historical-reading-receipt"));assert!(graph.edges.iter().filter(|edge|edge.relationship=="verification-request").all(|edge|edge.reason.contains("not a collected test-case verdict")));}

    #[test]
    fn collector_mode_without_confirmed_origin_does_not_become_current_execution(){for origin in ["unknown","current-workspace"]{let mut graph=Graph::default();graph.node("source","a.py","a.py","captured");let evidence=json!({"records":[{"id":"report","origin":origin,"collector":"pytest-cov","freshness":"origin-unknown","evidence_kind":"automated-collector","matching_inputs":["a.py"],"reachability":[{"path":"a.py","hit_lines":1}]}]});let mut budget=4096;graph.observations(&evidence,&json!({"records":[]}),"reading",&mut budget);assert!(graph.edges.iter().filter(|edge|edge.relationship=="reported-line-reachability").all(|edge|edge.basis==if origin=="current-workspace"{"observed"}else{"declared"}));}}

    #[test]
    fn gaps_and_history_do_not_become_verified() {
        let mut graph = Graph::default();
        graph.metadata(&json!({"contracts":{"c":{"page":"old.md","disposition":"historical","references":["a.py"]}}}),&json!({"fragments":[{"id":"a","page":"a.md","output":"a.py","basis":"needs-explanation"}]}),&BTreeSet::from(["a.py".into()]));
        assert!(graph.nodes.values().any(|n| n.status == "historical"));
        assert!(
            graph
                .nodes
                .values()
                .any(|n| n.status == "needs-explanation")
        );
    }
}
