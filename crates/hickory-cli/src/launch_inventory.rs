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
