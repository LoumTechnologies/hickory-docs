<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-grove/src/doc_store.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ad6421a9ee647debd9b96d5673124c6a0eba62d2c4bf14a013687f3e91de53fe`.

<book:entry path="crates/hick-grove/src/doc_store.rs">
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

/// Thread-safe store mapping DocId → Yrs document + metadata.
pub struct DocStore {
    docs: Mutex<HashMap<DocId, Arc<DocMeta>>>,
}

impl Default for DocStore {
    fn default() -> Self {
        Self::new()
    }
}

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
</book:entry>

</book:doc>
