<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep compiler-issued ownership when the main loses its Markdown suffix

For maintainers protecting source writes after an explicit main relocation. The role supplies the entry; its compiler supplies actual output paths. Retained positive claims remain protective when current compilation fails. A plain map cannot create a workspace-root volume claim, and an unknown closure never proves another source writable. The small immutable compiler cache validates every consumed physical source guard; it avoids rerunning the compiler under the global history writer mutex.

Files can also recover the entry's document identity from the retained role
without compiling it. This optional reading observes a confirmed binding and
never initializes one. The tree attaches the identity only to an exact visible
file, so readable paths and source acquisition do not depend on that metadata.

<ui:copy id="workspace-entry-owned-claims">
mod claims {
    use super::*;
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::Path,
        sync::{Arc, LazyLock, Mutex},
        time::Instant,
    };
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(super) struct Inventory {
        version: u32,
        entry: EntryStamp,
        outputs: BTreeSet<String>,
        volumes: BTreeSet<String>,
        root_volume: bool,
    }
    #[derive(Clone)]
    pub(crate) struct PositiveClaims {
        entry: String,
        outputs: BTreeSet<String>,
        volumes: BTreeSet<String>,
        root_volume: bool,
    }
    impl PositiveClaims {
        pub(crate) fn entry(&self) -> &str {
            &self.entry
        }
        pub(crate) fn outputs(&self) -> impl Iterator<Item = &String> {
            self.outputs.iter()
        }
        pub(crate) fn volumes(&self) -> impl Iterator<Item = &String> {
            self.volumes.iter()
        }
        pub(crate) fn root_volume(&self) -> bool {
            self.root_volume
        }
    }
    #[derive(Clone)]
    struct Current {
        root: String,
        entry: EntryStamp,
        inventory: Inventory,
        guards: BTreeMap<String, String>,
        at: Instant,
    }
    static CURRENT: LazyLock<Mutex<BTreeMap<std::path::PathBuf, Arc<Current>>>> =
        LazyLock::new(|| Mutex::new(BTreeMap::new()));
    impl Inventory {
        pub(super) fn validate(&self) -> Result<(), String> {
            self.entry.validate()?;
            if self.version != 1
                || self.entry.path.is_none()
                || self.outputs.len() > 10000
                || self.volumes.len() > 10000
            {
                return Err("Canonical ownership inventory has invalid bounds.".into());
            }
            for path in self.outputs.iter().chain(self.volumes.iter()) {
                super::super::resource_changes::validate_path(path)?;
            }
            Ok(())
        }
        fn projection(&self) -> Result<PositiveClaims, String> {
            self.validate()?;
            Ok(PositiveClaims {
                entry: self
                    .entry
                    .path
                    .clone()
                    .ok_or("Absent main has no output projection")?,
                outputs: self.outputs.clone(),
                volumes: self.volumes.clone(),
                root_volume: self.root_volume,
            })
        }
        fn union(&mut self, other: &Self) {
            self.outputs.extend(other.outputs.clone());
            self.volumes.extend(other.volumes.clone());
            self.root_volume |= other.root_volume;
        }
    }
    fn captured(
        root: &Path,
        entry: &EntryStamp,
        proposed: Option<(&str, &str)>,
    ) -> Result<Current, String> {
        let path = entry
            .path
            .as_ref()
            .ok_or("No canonical main is configured")?;
        let observed = super::entry_observation(root, path)?;
        let source = proposed
            .filter(|(changed, _)| *changed == path)
            .map(|(_, text)| text.to_owned())
            .unwrap_or(observed.source);
        if hick_lang::parse(&source)
            .map_err(|error| error.to_string())?
            .find_tags("project")
            .is_empty()
        {
            return Err("The retained main no longer declares its project; its previous ownership remains protected.".into());
        }
        let sources = proposed
            .filter(|(changed, _)| *changed != path)
            .map(|(path, text)| BTreeMap::from([(path.to_owned(), text.to_owned())]))
            .unwrap_or_default();
        let scan = super::super::source_tests::scan_with_namespace(
            root,
            &super::super::source_tests::Request {
                path: path.clone(),
                source,
                project_entry: Some(path.clone()),
                sources,
                capture: false,
            },
            None,
            &BTreeSet::new(),
        )?;
        let tree = scan
            .tree
            .as_ref()
            .ok_or("Canonical construction has no complete compiler tree")?;
        let base = Path::new(path).parent().unwrap_or(Path::new(""));
        let mut outputs = BTreeSet::new();
        for output in tree.entries().keys() {
            outputs.insert(super::super::source_tests::relative(base, output)?);
        }
        let mut volumes =
            super::super::resource_inputs::declared_volume_folders(root, &scan.source_texts)?;
        let root_volume = volumes.remove("");
        let inventory = Inventory {
            version: 1,
            entry: entry.clone(),
            outputs,
            volumes,
            root_volume,
        };
        inventory.validate()?;
        let mut guards = BTreeMap::new();
        for (path, expected) in &scan.source_texts {
            let observed =
                super::super::source_history::read_observed(root, &root.join(path), path)?;
            if proposed.is_none_or(|(changed, _)| changed != path) && observed.source != *expected {
                return Err("Canonical input changed during ownership capture.".into());
            }
            guards.insert(
                path.clone(),
                observed
                    .save_guard
                    .ok_or("Canonical input has no physical source guard")?,
            );
        }
        let value = Current {
            root: super::super::resource_inputs::RootIdentity::fingerprint(root)?,
            entry: entry.clone(),
            inventory,
            guards,
            at: Instant::now(),
        };
        if !current(root, &value, entry)? {
            return Err("Canonical input changed before ownership capture completed.".into());
        }
        Ok(value)
    }
    fn current(root: &Path, value: &Current, entry: &EntryStamp) -> Result<bool, String> {
        if &value.entry != entry
            || value.root != super::super::resource_inputs::RootIdentity::fingerprint(root)?
        {
            return Ok(false);
        }
        for (path, guard) in &value.guards {
            if !super::super::source_history::cached_source_guard_current(root, path, guard)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn actual(root: &Path, entry: &EntryStamp) -> Result<Arc<Current>, String> {
        let address = root.canonicalize().map_err(|error| error.to_string())?;
        let cached = CURRENT
            .lock()
            .map_err(|_| "Canonical ownership cache is unavailable")?
            .get(&address)
            .cloned();
        if let Some(value) = cached {
            if current(root, &value, entry)? {
                return Ok(value);
            }
        }
        let value = Arc::new(captured(root, entry, None)?);
        let mut values = CURRENT
            .lock()
            .map_err(|_| "Canonical ownership cache is unavailable")?;
        if values.len() >= 32 {
            if let Some(oldest) = values
                .iter()
                .min_by_key(|(_, value)| value.at)
                .map(|(path, _)| path.clone())
            {
                values.remove(&oldest);
            }
        }
        values.insert(address, value.clone());
        Ok(value)
    }
    pub(super) fn inventory(root: &Path, entry: &EntryStamp) -> Result<Inventory, String> {
        Ok(actual(root, entry)?.inventory.clone())
    }
    #[cfg(unix)]
    pub(crate) fn positive(root: &Path) -> Result<Option<PositiveClaims>, String> {
        let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
        let Some(bound) = super::storage::Bound::optional_under(&base, root)? else {
            return Ok(None);
        };
        if bound.payload.current.path.is_none() {
            return Ok(None);
        }
        if bound.payload.history.is_empty()
            && bound.payload.current.path.as_deref() == Some("project.md")
            && bound.payload.ownership.is_none()
        {
            return Ok(None);
        }
        let mut retained = bound.payload.ownership.clone();
        let fresh = actual(root, &bound.payload.current);
        match fresh {
            Ok(value) => {
                let mut inventory = value.inventory.clone();
                if let Some(before) = retained.take() {
                    inventory.union(&before);
                }
                inventory.projection().map(Some)
            }
            Err(error) => match retained {
                Some(inventory) => inventory.projection().map(Some),
                None => Err(error),
            },
        }
    }
    #[cfg(not(unix))]
    pub(crate) fn positive(_root: &Path) -> Result<Option<PositiveClaims>, String> {
        Ok(None)
    }
    pub(super) fn protect_body(root: &Path, path: &Path, source: &str) -> Result<(), String> {
        #[cfg(unix)]
        {
            let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
            let Some(bound) = super::storage::Bound::optional_under(&base, root)? else {
                return Ok(());
            };
            let Some(main) = &bound.payload.current.path else {
                return Ok(());
            };
            if bound.payload.history.is_empty()
                && main == "project.md"
                && bound.payload.ownership.is_none()
            {
                return Ok(());
            }
            let full = if path.exists() {
                path.canonicalize().map_err(|error| error.to_string())?
            } else {
                path.parent()
                    .ok_or("Source target has no parent")?
                    .canonicalize()
                    .map_err(|error| error.to_string())?
                    .join(path.file_name().ok_or("Source target has no name")?)
            };
            let canonical = root.canonicalize().map_err(|error| error.to_string())?;
            let Ok(relative) = full.strip_prefix(&canonical) else {
                return Ok(());
            };
            let relative = relative
                .to_str()
                .ok_or("Source target is not UTF-8")?
                .replace(std::path::MAIN_SEPARATOR, "/");
            if let Some(before) = &bound.payload.ownership {
                let projection = before.projection()?;
                if owns(&projection, &relative) {
                    return Err(
                        "A retained canonical generated claim owns this source target.".into(),
                    );
                }
            }
            let before = actual(root, &bound.payload.current);
            let input = before
                .as_ref()
                .map(|value| value.guards.contains_key(&relative))
                .unwrap_or(relative == *main);
            if let Ok(before) = &before {
                if owns(&before.inventory.projection()?, &relative) {
                    return Err(
                        "The actual canonical compiler owns this generated source target.".into(),
                    );
                }
            }
            if before.is_err() && relative != *main {
                return Err("Canonical ownership is incomplete; repair the retained main before authorizing other source writes. Readable source and drafts are retained.".into());
            }
            if input {
                let after = captured(root, &bound.payload.current, Some((&relative, source)))?;
                if owns(&after.inventory.projection()?, &relative) {
                    return Err("The proposed construction would generate its own edited source; no authored write was granted.".into());
                }
            }
        }
        Ok(())
    }
    fn owns(claims: &PositiveClaims, path: &str) -> bool {
        claims.root_volume
            || claims.outputs.contains(path)
            || claims.volumes.iter().any(|folder| {
                path == folder
                    || path
                        .strip_prefix(folder.as_str())
                        .is_some_and(|suffix| suffix.starts_with('/'))
            })
    }
}
pub(crate) use claims::PositiveClaims;
pub(crate) fn positive_entry_claims(
    root: &std::path::Path,
) -> Result<Option<PositiveClaims>, String> {
    claims::positive(root)
}
pub(crate) fn protect_source_body(
    root: &std::path::Path,
    path: &std::path::Path,
    source: &str,
) -> Result<(), String> {
    claims::protect_body(root, path, source)
}
#[cfg(unix)]
pub(crate) fn retained_tree_entry(root: &std::path::Path) -> Result<Option<String>, String> {
    leases::ordinary_root(root)?;
    let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
    let Some(bound) = storage::Bound::optional_under(&base, root)? else {
        return Ok(None);
    };
    bound.unchanged()?;
    Ok(if bound.payload.pending.is_none() {
        bound.payload.current.path.clone()
    } else {
        None
    })
}
#[cfg(not(unix))]
pub(crate) fn retained_tree_entry(_root: &std::path::Path) -> Result<Option<String>, String> {
    Ok(None)
}
</ui:copy>

</ui:doc>
