<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Comparison edit permissions do not establish replay evidence

For: Maintainers reorganizing source while preserving the complete generated program.

Given an N-source project comparison, when Old and New endpoints are selected,
then their identities, writable destinations and edit policy are separate from
replay verification. All four policies (neither, old, new, both) are supported
by the model. An immutable source without a writable fork cannot become editable.
An accepted endpoint edit clears equality evidence and leaves the opposite
endpoint unchanged. Destination trees must still equal the captured identity
before a host applies reviewed changes.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-ea722f80e003"></a>
## `crates/hick-lang/src/project/comparison.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Comparison edit permissions do not establish replay evidence](contract.md).

<a id="f-ea722f80e003-RevisionIdentity"></a>
### `RevisionIdentity`

<storycode:copy id="f-ea722f80e003-RevisionIdentity">
use super::{ProjectDifference, ProjectTree, evaluate_project};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RevisionIdentity {
    Directory { identity: String },
    Git { revision: String },
    Mercurial { revision: String },
    Crdt { document: String, snapshot: String },
    Authored { document: String },
}
</storycode:copy>
<a id="f-ea722f80e003-EditPolicy"></a>
### `EditPolicy`

<storycode:copy id="f-ea722f80e003-EditPolicy">
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EditPolicy {
    Neither,
    Old,
    New,
    Both,
}
</storycode:copy>
<a id="f-ea722f80e003-impl-EditPolicy"></a>
### `impl-EditPolicy`

<storycode:copy id="f-ea722f80e003-impl-EditPolicy">
impl EditPolicy {
    pub fn old(self) -> bool {
        matches!(self, Self::Old | Self::Both)
    }
    pub fn new_side(self) -> bool {
        matches!(self, Self::New | Self::Both)
    }
}
</storycode:copy>
<a id="f-ea722f80e003-ComparisonSource"></a>
### `ComparisonSource`

<storycode:copy id="f-ea722f80e003-ComparisonSource">
#[derive(Debug, Clone)]
pub struct ComparisonSource {
    pub identity: RevisionIdentity,
    pub tree: ProjectTree,
    /// None for immutable revision readings. An explicit fork creates a target.
    pub writable_target: Option<String>,
}

</storycode:copy>
<a id="f-ea722f80e003-ProjectComparison"></a>
### `ProjectComparison`

Shared N-source comparison contract. Replay evidence is independent of
importing a provider diff or naming a revision adapter.

<storycode:copy id="f-ea722f80e003-ProjectComparison">
/// Shared N-source comparison contract. Replay evidence is independent of
/// importing a provider diff or naming a revision adapter.
#[derive(Debug, Clone)]
pub struct ProjectComparison {
    sources: Vec<ComparisonSource>,
    old: usize,
    new: usize,
    policy: EditPolicy,
    verified: bool,
    captured_destinations: Vec<ProjectTree>,
}
</storycode:copy>
<a id="f-ea722f80e003-impl-ProjectComparison"></a>
### `impl-ProjectComparison`

<storycode:copy id="f-ea722f80e003-impl-ProjectComparison">
impl ProjectComparison {
    pub fn new(
        sources: Vec<ComparisonSource>,
        old: usize,
        new: usize,
        policy: EditPolicy,
    ) -> Result<Self, String> {
        if old >= sources.len() || new >= sources.len() || old == new {
            return Err("comparison requires two distinct valid endpoints".into());
        }
        for (side, editable) in [(old, policy.old()), (new, policy.new_side())] {
            if editable
                && matches!(
                    sources[side].identity,
                    RevisionIdentity::Git { .. }
                        | RevisionIdentity::Mercurial { .. }
                        | RevisionIdentity::Crdt { .. }
                )
            {
                return Err(
                    "immutable revision must be forked into an authored comparison before editing"
                        .into(),
                );
            }
        }
        if (policy.old() && sources[old].writable_target.is_none())
            || (policy.new_side() && sources[new].writable_target.is_none())
        {
            return Err("immutable revision must be forked before editing".into());
        }
        let captured_destinations = sources.iter().map(|s| s.tree.clone()).collect();
        Ok(Self {
            sources,
            captured_destinations,
            old,
            new,
            policy,
            verified: false,
        })
    }
    pub fn sources(&self) -> &[ComparisonSource] {
        &self.sources
    }
    pub fn endpoints(&self) -> (usize, usize) {
        (self.old, self.new)
    }
    pub fn policy(&self) -> EditPolicy {
        self.policy
    }
    pub fn verified(&self) -> bool {
        self.verified
    }
    pub fn verify_transition(
        &mut self,
        entry: &str,
        sources: &BTreeMap<String, String>,
    ) -> Result<Vec<ProjectDifference>, String> {
        self.verified = false;
        let evaluated = evaluate_project(entry, sources, &self.sources[self.old].tree)?;
        let differences = evaluated.new.compare(&self.sources[self.new].tree);
        self.verified = differences.is_empty();
        Ok(differences)
    }
    pub fn edit_endpoint(&mut self, side: usize, replacement: ProjectTree) -> Result<(), String> {
        if !((side == self.old && self.policy.old())
            || (side == self.new && self.policy.new_side()))
        {
            return Err("comparison side is read-only".into());
        }
        if self.sources[side].writable_target.is_none() {
            return Err("comparison has no writable destination".into());
        }
        self.sources[side].tree = replacement;
        self.verified = false;
        Ok(())
    }
    /// Before applying reviewed edits, compare exact captured destinations.
    pub fn check_destination(&self, side: usize, current: &ProjectTree) -> Result<(), String> {
        self.sources.get(side).ok_or("invalid comparison source")?;
        if !self.captured_destinations[side].compare(current).is_empty() {
            return Err("destination changed; generate and review a new candidate".into());
        }
        Ok(())
    }
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/comparison.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/comparison.rs"><storycode:paste select="#f-ea722f80e003-RevisionIdentity"/><storycode:paste select="#f-ea722f80e003-EditPolicy"/><storycode:paste select="#f-ea722f80e003-impl-EditPolicy"/><storycode:paste select="#f-ea722f80e003-ComparisonSource"/><storycode:paste select="#f-ea722f80e003-ProjectComparison"/><storycode:paste select="#f-ea722f80e003-impl-ProjectComparison"/></storycode:file>

## Test the behavior: `permissions_replay_and_destination_identity`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-permissions_replay_and_destination_identity"></a>
### `permissions_replay_and_destination_identity`

<storycode:copy id="f-005c1e9344c7-permissions_replay_and_destination_identity">
// Guarantee: docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
#[test]
fn permissions_replay_and_destination_identity() {
    for policy in [
        EditPolicy::Neither,
        EditPolicy::Old,
        EditPolicy::New,
        EditPolicy::Both,
    ] {
        let source = |name: &str| ComparisonSource {
            identity: RevisionIdentity::Authored {
                document: name.into(),
            },
            tree: ProjectTree::default(),
            writable_target: Some(name.into()),
        };
        let mut comparison = ProjectComparison::new(
            vec![source("old"), source("new"), source("other")],
            0,
            1,
            policy,
        )
        .unwrap();
        assert!(
            comparison
                .verify_transition(
                    "p.md",
                    &BTreeMap::from([("p.md".into(), "<hick:project/>".into())])
                )
                .unwrap()
                .is_empty()
        );
        assert!(comparison.verified());
        assert_eq!(
            comparison.edit_endpoint(0, ProjectTree::default()).is_ok(),
            policy.old()
        );
        assert_eq!(
            comparison.edit_endpoint(1, ProjectTree::default()).is_ok(),
            policy.new_side()
        );
        if policy != EditPolicy::Neither {
            assert!(!comparison.verified());
        }
        assert!(
            comparison
                .check_destination(1, &ProjectTree::default())
                .is_ok()
        );
        assert!(
            comparison
                .check_destination(1, &tree(&[("concurrent", b"x")]))
                .is_err()
        );
    }
    let immutable = ComparisonSource {
        identity: RevisionIdentity::Git {
            revision: "abc".into(),
        },
        tree: ProjectTree::default(),
        writable_target: None,
    };
    assert!(
        ProjectComparison::new(vec![immutable.clone(), immutable], 0, 1, EditPolicy::Both).is_err()
    );
}
</storycode:copy>

## Test the behavior: `editing_a_proposal_does_not_replace_its_destination_guard`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-editing_a_proposal_does_not_replace_its_destination_guard"></a>
### `editing_a_proposal_does_not_replace_its_destination_guard`

<storycode:copy id="f-005c1e9344c7-editing_a_proposal_does_not_replace_its_destination_guard">
// Guarantee: docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
#[test]
fn editing_a_proposal_does_not_replace_its_destination_guard() {
    let source = |name: &str| ComparisonSource {
        identity: RevisionIdentity::Authored {
            document: name.into(),
        },
        tree: ProjectTree::default(),
        writable_target: Some(name.into()),
    };
    let mut comparison =
        ProjectComparison::new(vec![source("old"), source("new")], 0, 1, EditPolicy::New).unwrap();
    comparison
        .edit_endpoint(1, tree(&[("proposed", b"new")]))
        .unwrap();
    assert!(
        comparison
            .check_destination(1, &ProjectTree::default())
            .is_ok()
    );
    assert!(
        comparison
            .check_destination(1, &tree(&[("concurrent", b"write")]))
            .is_err()
    );
}
</storycode:copy>

## Test the behavior: `displayed_snapshots_permissions_and_undo_never_overwrite_newer_target_edits`

This is the maintained test fragment from `crates/hickory-cli/tests/merged_view.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-87756f3e699e-displayed_snapshots_permissions_and_undo_never_overwrite_newer_target_edits"></a>
### `displayed_snapshots_permissions_and_undo_never_overwrite_newer_target_edits`

<storycode:copy id="f-87756f3e699e-displayed_snapshots_permissions_and_undo_never_overwrite_newer_target_edits">
// Guarantee: docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
#[tokio::test]
async fn displayed_snapshots_permissions_and_undo_never_overwrite_newer_target_edits() {
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let view = get(&session, "/api/merged?path=shared.txt").await;
    let (_,written)=post(&session,"/api/merged/write",serde_json::json!({
        "path":"shared.txt","targets":["repo","side-tree"],"writable":["repo"],"expected":view["snapshot"],"region":shared,"text":"edited\n","route":"shared"
    })).await;
    assert_eq!(written["landed"], 1);
    assert_eq!(written["partial"], true);
    assert_eq!(written["undo"].as_array().unwrap().len(), 2);
    std::fs::write(session.root.join("shared.txt"), "later user edit\n").unwrap();
    let (_, undo) = post(
        &session,
        "/api/merged/undo",
        serde_json::json!({"path":"shared.txt","undo":written["undo"]}),
    )
    .await;
    // The unreached side is already restored without a write. The newer main
    // edit still refuses undo, so this is an explicitly partial result.
    assert_eq!(undo["restored"], 1);
    assert_eq!(undo["partial"], true);
    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        "later user edit\n"
    );
    let (status,_)=post(&session,"/api/merged/write",serde_json::json!({
        "path":"shared.txt","targets":["repo","side-tree"],"writable":["repo","side-tree"],"expected":view["snapshot"],"region":shared,"text":"stale\n","route":"shared"
    })).await;
    assert_eq!(status, 409);
    assert!(
        std::fs::read_to_string(side_path(&session).join("shared.txt"))
            .unwrap()
            .starts_with("head\n")
    );
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/lib/projectComparison.ts](../../../mechanisms/apps/web/src/lib/projectComparison.ts.md#output-7ddde77909ef)
- [crates/hick-lang/src/project/comparison.rs](contract.md#output-ea722f80e003)
- [crates/hickory-cli/src/serve/merged_write.rs](../../../mechanisms/crates/hickory-cli/src/serve/merged_write.rs.md#output-1635aa80d82b)

### Tests and fixtures

- [apps/web/src/components/ComparisonEditor.test.tsx](../../../mechanisms/apps/web/src/components/ComparisonEditor.test.tsx.md#output-3e23ac42e4a1)
- [apps/web/src/lib/projectComparison.test.ts](../../../mechanisms/apps/web/src/lib/projectComparison.ts.md#output-a9f1fc73a39f)
- [crates/hick-lang/tests/project.rs](../../../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7)
- [crates/hickory-cli/tests/merged_view.rs](../../../mechanisms/crates/hickory-cli/tests/merged_view.rs.md#output-87756f3e699e)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
