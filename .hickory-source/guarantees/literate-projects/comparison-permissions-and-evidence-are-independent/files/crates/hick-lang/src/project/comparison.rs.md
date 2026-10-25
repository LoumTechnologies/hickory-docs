<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/comparison.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Comparison edit permissions do not establish replay evidence](../../../../../contract.md)

Captured SHA-256: `aff5564b515ff4b834f8b288d2eec725c2b8891b46b8dc00ffc5daced72d4ebf`.

<book:entry path="crates/hick-lang/src/project/comparison.rs">
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EditPolicy {
    Neither,
    Old,
    New,
    Both,
}
impl EditPolicy {
    pub fn old(self) -> bool {
        matches!(self, Self::Old | Self::Both)
    }
    pub fn new_side(self) -> bool {
        matches!(self, Self::New | Self::Both)
    }
}
#[derive(Debug, Clone)]
pub struct ComparisonSource {
    pub identity: RevisionIdentity,
    pub tree: ProjectTree,
    /// None for immutable revision readings. An explicit fork creates a target.
    pub writable_target: Option<String>,
}

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
</book:entry>

</book:doc>
