<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/ui_verification.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `064b36c0343776aa45c0afb2cb9e15a5be072365cdcf63d2824e9961d0f70ea4`.

<book:entry path="crates/hickory-cli/src/serve/ui_verification.rs">
//! Turn-owned shared-UI check jobs, validated against host execution receipts.
//! UI requests establish attribution, never the execution result.
use super::RunRecord;
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub(super) struct Candidate {
    pub job: String,
    pub document: String,
    pub requested_source_hash: String,
    pub turn: String,
}
impl Candidate {
    pub fn from_reply(reply: &Value, turn: &str) -> Option<Self> {
        if reply["status"] != "result"
            || reply["outcome"]["status"] != "queued"
            || reply["outcome"]["value"]["purpose"] != "document-check"
        {
            return None;
        }
        let value = &reply["outcome"]["value"];
        Some(Self {
            job: reply["outcome"]["job"].as_str()?.into(),
            document: value["document"].as_str()?.into(),
            requested_source_hash: value["requested_source_hash"].as_str()?.into(),
            turn: turn.into(),
        })
    }
    pub fn evidence(&self, turn: &str, run: &RunRecord, path: &std::path::Path) -> Option<Value> {
        let hash = |value: &str| value.strip_prefix("sha256:").unwrap_or(value).to_owned();
        if self.turn != turn
            || !run.check_only
            || self.document != run.document
            || run.requested_source_hash.as_deref().map(hash).as_deref()
                != Some(hash(&self.requested_source_hash).as_str())
            || run.source_hash.as_deref().map(hash).as_deref()
                != Some(hash(&self.requested_source_hash).as_str())
            || !matches!(run.status.as_str(), "ok" | "failed" | "stopped")
        {
            return None;
        }
        let state = match run.status.as_str() {
            "ok" if run.error.is_none() => "passed",
            "stopped" => "stopped",
            _ => "failed",
        };
        let source = hash(&self.requested_source_hash);
        Some(
            json!({"state":state,"revision":source,"run_id":self.job,"document":self.document,
            "source_snapshots":[{"path":path.display().to_string(),"sha256":source}],
            "actor":"harness","tool":"shared-ui-verify",
            "scope":"Shared UI document execution and expectations at this root source revision. Dependency revisions, other edited documents, semantic truth and completeness are not certified"}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Candidate, RunRecord) {
        let reply = json!({"status":"result","outcome":{"status":"queued","job":"check-one","value":{"purpose":"document-check","document":"doc-one","requested_source_hash":"abc"}}});
        (
            Candidate::from_reply(&reply, "turn-one").unwrap(),
            RunRecord {
                document: "doc-one".into(),
                check_only: true,
                requested_source_hash: Some("abc".into()),
                source_hash: Some("abc".into()),
                status: "ok".into(),
                error: None,
                started_at: "now".into(),
                blocks: json!([]),
            },
        )
    }
    #[test]
    fn matching_shared_ui_check_is_bound_to_its_turn_document_and_source() {
        let (candidate, run) = fixture();
        let evidence = candidate
            .evidence("turn-one", &run, std::path::Path::new("doc.md"))
            .unwrap();
        assert_eq!(evidence["state"], "passed");
        assert_eq!(evidence["revision"], "abc");
        assert_eq!(evidence["run_id"], "check-one");
        assert_eq!(evidence["source_snapshots"][0]["sha256"], "abc");
        assert!(
            evidence["scope"]
                .as_str()
                .unwrap()
                .contains("Dependency revisions")
        );
    }
    #[test]
    fn unrelated_turn_document_hash_run_or_incomplete_receipt_cannot_establish_verification() {
        let (candidate, run) = fixture();
        let path = std::path::Path::new("doc.md");
        assert!(candidate.evidence("other-turn", &run, path).is_none());
        for altered in [
            RunRecord {
                document: "other-doc".into(),
                ..run.clone()
            },
            RunRecord {
                source_hash: Some("different".into()),
                ..run.clone()
            },
            RunRecord {
                requested_source_hash: Some("different".into()),
                ..run.clone()
            },
            RunRecord {
                check_only: false,
                ..run.clone()
            },
            RunRecord {
                status: "running".into(),
                ..run.clone()
            },
            RunRecord {
                source_hash: None,
                ..run.clone()
            },
        ] {
            assert!(candidate.evidence("turn-one", &altered, path).is_none());
        }
        assert!(Candidate::from_reply(&json!({"status":"result","outcome":{"status":"completed","value":{"receipt":{"status":"ok"}}}}), "turn-one").is_none());
    }
    #[test]
    fn a_failed_check_or_host_error_is_never_displayed_as_passed() {
        let (candidate, mut run) = fixture();
        run.error = Some("execution failed".into());
        assert_eq!(
            candidate
                .evidence("turn-one", &run, std::path::Path::new("doc.md"))
                .unwrap()["state"],
            "failed"
        );
        run.status = "failed".into();
        assert_eq!(
            candidate
                .evidence("turn-one", &run, std::path::Path::new("doc.md"))
                .unwrap()["state"],
            "failed"
        );
    }
}
</book:entry>

</book:doc>
