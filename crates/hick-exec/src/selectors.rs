//! Selector vocabulary: what `.a.b` requires, and what kind of element an
//! `#id` names.
//!
//! Split out of `state.rs` so the state module holds the run's mutable
//! stores and not the rules for reading a selector.

use std::collections::HashMap;
use std::sync::Mutex;

/// The class names a class selector requires — ALL of them.
///
/// CSS has always read `.a.b` as "carries both", and hick's selectors are
/// CSS-shaped, so this is the compound form rather than a new grammar. It is
/// what replaces CodegenBot's name/value caret tags: matching on two
/// properties at once is the load-bearing half, and bare names carry it.
///
/// Returns `None` for anything that is not a class selector, and for a bare
/// `.` — which names no class and must match nothing rather than everything.
pub(crate) fn required_classes(selector: &str) -> Option<Vec<&str>> {
    let rest = selector.trim().strip_prefix('.')?;
    let names: Vec<&str> = rest.split('.').filter(|s| !s.is_empty()).collect();
    if names.is_empty() { None } else { Some(names) }
}

/// Whether a block carries every class a selector requires.
pub(crate) fn has_all_classes(classes: &[String], required: &[&str]) -> bool {
    required
        .iter()
        .all(|need| classes.iter().any(|have| have == need))
}

/// What kind of element each `id=` in a run's documents names.
///
/// A `<hick:paste select="#x">` that resolves to nothing has two very
/// different causes, and the message used to say only the first: the id does
/// not exist, or it exists and names something that is not text. `cites=`
/// resolves to any element with an id; a paste reproduces bytes and cannot.
/// Consulted only after a paste has already failed, so nothing here is on a
/// resolution path.
#[derive(Default)]
pub struct ElementKinds {
    kinds: Mutex<HashMap<String, String>>,
}

impl ElementKinds {
    /// Record that `id` belongs to an element of kind `kind` (`exec`,
    /// `table`, `file`, …). First registration wins, which is the rule `#id`
    /// resolution follows everywhere else.
    pub fn register(&self, id: &str, kind: &str) {
        if id.is_empty() {
            return;
        }
        self.kinds
            .lock()
            .unwrap()
            .entry(id.to_string())
            .or_insert_with(|| kind.to_string());
    }

    /// What kind of element carries `id`, if the run saw one.
    pub fn kind_of(&self, id: &str) -> Option<String> {
        self.kinds.lock().unwrap().get(id).cloned()
    }
}
