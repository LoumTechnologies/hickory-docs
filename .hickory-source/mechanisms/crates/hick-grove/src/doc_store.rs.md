<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/doc_store.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7a3132652045"></a>
## `crates/hick-grove/src/doc_store.rs`

<a id="f-7a3132652045-DocMeta"></a>
### `DocMeta`

<storycode:copy id="f-7a3132652045-DocMeta">
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use yrs::Doc;

use crate::namespace::{DocId, NamespaceUri};
use crate::yrs_bridge::ElementCache;

/// Metadata about a document's namespace declarations.
pub struct DocMeta {
    pub doc: Doc,
    /// Map from prefix → namespace URI declared when the document was created.
    pub namespaces: HashMap<String, NamespaceUri>,
    /// Shared element cache for removal detection across bridge instances.
    pub element_cache: ElementCache,
}

</storycode:copy>
<a id="f-7a3132652045-DocStore"></a>
### `DocStore`

Thread-safe store mapping DocId → Yrs document + metadata.

<storycode:copy id="f-7a3132652045-DocStore">
/// Thread-safe store mapping DocId → Yrs document + metadata.
pub struct DocStore {
    docs: Mutex<HashMap<DocId, Arc<DocMeta>>>,
}

</storycode:copy>
<a id="f-7a3132652045-impl-Default-for-DocStore"></a>
### `impl-Default for DocStore`

<storycode:copy id="f-7a3132652045-impl-Default-for-DocStore">
impl Default for DocStore {
    fn default() -> Self {
        Self::new()
    }
}

</storycode:copy>
<a id="f-7a3132652045-impl-DocStore"></a>
### `impl-DocStore`

<storycode:copy id="f-7a3132652045-impl-DocStore">
impl DocStore {
    pub fn new() -> Self {
        Self {
            docs: Mutex::new(HashMap::new()),
        }
    }

    pub fn create(&self, id: DocId, namespaces: HashMap<String, NamespaceUri>) -> Arc<DocMeta> {
        let doc = Doc::new();
        let element_cache = ElementCache::new();
        let meta = Arc::new(DocMeta {
            doc,
            namespaces,
            element_cache,
        });
        self.docs.lock().unwrap().insert(id, meta.clone());
        meta
    }

    pub fn get(&self, id: &DocId) -> Option<Arc<DocMeta>> {
        self.docs.lock().unwrap().get(id).cloned()
    }

    pub fn contains(&self, id: &DocId) -> bool {
        self.docs.lock().unwrap().contains_key(id)
    }
}
</storycode:copy>

### Assemble `crates/hick-grove/src/doc_store.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/doc_store.rs"><storycode:paste select="#f-7a3132652045-DocMeta"/><storycode:paste select="#f-7a3132652045-DocStore"/><storycode:paste select="#f-7a3132652045-impl-Default-for-DocStore"/><storycode:paste select="#f-7a3132652045-impl-DocStore"/></storycode:file>

</storycode:doc>
