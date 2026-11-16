<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Discover launches over physical sources and constructed projects

For maintainers extending [launch profiles](../../../../guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project/contract.md).
Keep filesystem observation separate from portable generated-output publication.

<a id="launch-inventory-interface"></a>
## Read source facts without promising publication

A generated project has portable output names. An existing workspace has
physical source names. Providers need the same read-only facts from both, while
path validation remains owned by the correct domain. This interface grants no
write, build or execution permission.

<storycode:copy id="launch-inventory-interface">
use hick_lang::{ProjectEntry, ProjectTree};
use std::{collections::BTreeMap, sync::Arc};

/// Read-only launch inputs. A physical inventory is never a project to publish.
pub trait SourceInventory {
    fn entries(&self) -> &BTreeMap<String, Arc<ProjectEntry>>;
    fn validate_path(&self, path: &str) -> Result<(), String>;
    fn notices(&self) -> &[String] {
        &[]
    }
    fn get(&self, path: &str) -> Option<&ProjectEntry> {
        self.entries().get(path).map(Arc::as_ref)
    }
}
impl SourceInventory for ProjectTree {
    fn entries(&self) -> &BTreeMap<String, Arc<ProjectEntry>> {
        self.entries()
    }
    fn validate_path(&self, path: &str) -> Result<(), String> {
        hick_lang::validate_project_path(path)
    }
}
</storycode:copy>

<a id="physical-launch-source-inventory"></a>
## Retain exact observed filesystem source names

Store physical source bytes and names without constructing an output tree.
Unicode, percent signs and URI punctuation must not reject another source’s
launch. Unsupported names retain explicit notices. File admission and namespace
confinement remain the caller’s responsibility before provider effects.

<storycode:copy id="physical-launch-source-inventory">
/// Exact UTF-8 physical names observed under the open workspace. This type has
/// no materialization API and does not relax portable constructed-output rules.
#[derive(Default)]
pub struct PhysicalSources {
    entries: BTreeMap<String, Arc<ProjectEntry>>,
    notices: Vec<String>,
}
impl PhysicalSources {
    pub fn insert(&mut self, path: &str, entry: ProjectEntry) -> Result<(), String> {
        validate_workspace_path(path)?;
        self.entries.insert(path.to_owned(), Arc::new(entry));
        Ok(())
    }
    pub fn skip(&mut self, notice: String) {
        self.notices.push(notice);
    }
}
impl SourceInventory for PhysicalSources {
    fn entries(&self) -> &BTreeMap<String, Arc<ProjectEntry>> {
        &self.entries
    }
    fn validate_path(&self, path: &str) -> Result<(), String> {
        validate_workspace_path(path)
    }
    fn notices(&self) -> &[String] {
        &self.notices
    }
}
</storycode:copy>

<a id="physical-launch-path-policy"></a>
## Validate source addresses without applying output portability

Keep relative paths free of traversal, control characters and ambiguous
separators. Physical Unicode and question-mark names are valid on hosts that
actually contain them; this does not permit those paths in a constructed tree.

<storycode:copy id="physical-launch-path-policy">
pub fn validate_workspace_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!(
            "Use an exact relative workspace source path without traversal or ambiguous separators: {path:?}"
        ));
    }
    Ok(())
}

/// A workspace spelling is not a pre-encoded URI. Encode each UTF-8 byte once.
pub fn workspace_source_uri(path: &str) -> Result<String, String> {
    validate_workspace_path(path)?;
    let mut uri = String::from("hick:///");
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-_.!~*'()".contains(&byte) {
            uri.push(byte as char);
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    Ok(uri)
}
</storycode:copy>

<a id="launch-input-domains"></a>
## Preserve the input domain through launch preparation

Only the constructed branch can supply a ProjectTree to materialization.
A workspace inventory can supply provider facts and captured hashes, then run
its existing program in place. Do not coerce observed files into generated
outputs merely to reuse discovery.

<storycode:copy id="launch-input-domains">
/// Keep the two domains distinct while sharing read-only provider discovery.
pub enum LaunchInputs {
    Constructed(ProjectTree),
    Workspace(PhysicalSources),
}
impl LaunchInputs {
    pub fn inventory(&self) -> &dyn SourceInventory {
        match self {
            Self::Constructed(tree) => tree,
            Self::Workspace(sources) => sources,
        }
    }
    pub fn project(&self) -> Option<&ProjectTree> {
        match self {
            Self::Constructed(tree) => Some(tree),
            Self::Workspace(_) => None,
        }
    }
}
</storycode:copy>

<a id="launch-inventory-domain-tests"></a>
## Check exact identities and preserve output safety

These checks exercise the mismatch directly: a literal physical filename is
admitted while the generated-output tree still rejects it. Canonically distinct
UTF-8 names remain distinct values; native adapters must refuse identities they
cannot represent without aliasing.

<storycode:copy id="launch-inventory-domain-tests">
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_names_do_not_relax_generated_output_rules() {
        let name = "literal %2F λ?#.py";
        let mut physical = PhysicalSources::default();
        physical
            .insert(name, ProjectEntry::file(b"print(41)\n".to_vec()))
            .unwrap();
        assert!(physical.get(name).is_some());
        assert!(physical.validate_path(name).is_ok());
        let mut constructed = ProjectTree::default();
        assert!(
            constructed
                .insert(name, ProjectEntry::file(b"print(41)\n".to_vec()))
                .is_err()
        );
        for invalid in [
            "../outside.py",
            "a/../outside.py",
            "/absolute.py",
            "a\\b.py",
            "a:b.py",
            "a//b.py",
            "a\n.py",
        ] {
            assert!(
                physical
                    .insert(invalid, ProjectEntry::file(Vec::new()))
                    .is_err()
            );
        }
    }
    #[test]
    fn canonical_distinct_physical_names_remain_exact() {
        let mut physical = PhysicalSources::default();
        physical
            .insert("é.py", ProjectEntry::file(b"first".to_vec()))
            .unwrap();
        physical
            .insert("e\u{301}.py", ProjectEntry::file(b"second".to_vec()))
            .unwrap();
        assert_eq!(physical.entries().len(), 2);
        assert_eq!(physical.get("é.py").unwrap().bytes.as_ref(), b"first");
        assert_eq!(
            physical.get("e\u{301}.py").unwrap().bytes.as_ref(),
            b"second"
        );
    }
}
</storycode:copy>

<storycode:file path="crates/hickory-cli/src/launch_inventory.rs"><storycode:paste select="#launch-inventory-interface"/><storycode:paste select="#physical-launch-source-inventory"/><storycode:paste select="#physical-launch-path-policy"/><storycode:paste select="#launch-input-domains"/><storycode:paste select="#launch-inventory-domain-tests"/></storycode:file>
</storycode:doc>
