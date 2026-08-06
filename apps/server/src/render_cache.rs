//! In-process cache for `GET /api/docs/:id/render`.
//!
//! Weaving a document is pure: the same source + the same includes + the same
//! last-run overlay always produce the same block model. The render route
//! paid that cost on every call — after every save, every run event and every
//! page load — so the same bytes were rewoven dozens of times per session.
//!
//! ## Why in-process, and why bounded
//!
//! The rendered block model is *derived* state: losing it costs one re-weave,
//! never correctness. That rules out the operational weight of Redis or a
//! Postgres table; a process-local map is the cheapest thing that works, and
//! a multi-instance deploy simply gets one warm cache per instance.
//!
//! Unboundedness is the real hazard, so this is a fixed-capacity LRU
//! (`CAPACITY` entries) with an additional per-entry byte cap: a block model
//! larger than `MAX_ENTRY_BYTES` is served but never stored, so one
//! pathological document cannot pin the process's memory.
//!
//! ## What is cached, and under what key
//!
//! Only the *weave* is cached — the expensive, pure part. The last-run
//! status/transcript overlay is re-applied from Postgres on every request
//! (two indexed reads and a hash-map merge), so run results and the `stale`
//! flag are never served from a stale cache.
//!
//! The weave is a function of exactly three things:
//!
//! - the document's own source → `sha256(doc.source)`, which covers edits
//!   arriving from any path (save, CRDT persist, lineage edit);
//! - the other documents in the project, which the weave may pull in through
//!   `<hick:include>` → `docs_revision`, the project's newest `docs.updated_at`;
//! - the non-document files in the project checkout, which only ever change
//!   when a run commits its outputs → `outputs_revision`, the newest
//!   `runs.finished_at` in the project.
//!
//! A change to any of those yields a different key, so there is no explicit
//! invalidation call to forget: superseded entries simply age out of the LRU.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

/// Maximum number of cached renders held per process.
const CAPACITY: usize = 256;

/// Renders bigger than this are computed and served but never stored.
const MAX_ENTRY_BYTES: usize = 4 * 1024 * 1024;

/// Identity of a weave: everything the woven block model depends on.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RenderKey {
    pub doc_id: Uuid,
    pub source_hash: [u8; 32],
    /// Newest `docs.updated_at` in the project (includes may have changed).
    pub docs_revision: Option<DateTime<Utc>>,
    /// Newest `runs.finished_at` in the project (committed outputs changed).
    pub outputs_revision: Option<DateTime<Utc>>,
}

impl RenderKey {
    pub fn new(
        doc_id: Uuid,
        source: &str,
        docs_revision: Option<DateTime<Utc>>,
        outputs_revision: Option<DateTime<Utc>>,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        let source_hash: [u8; 32] = hasher.finalize().into();
        RenderKey {
            doc_id,
            source_hash,
            docs_revision,
            outputs_revision,
        }
    }
}

#[derive(Default)]
struct Inner {
    entries: HashMap<RenderKey, (u64, Value)>,
    /// Monotonic tick; the smallest tick is the least recently used entry.
    clock: u64,
}

/// Bounded LRU of rendered block arrays.
#[derive(Default)]
pub struct RenderCache {
    inner: Mutex<Inner>,
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
}

impl RenderCache {
    pub fn get(&self, key: &RenderKey) -> Option<Value> {
        let mut inner = self.inner.lock().expect("render cache poisoned");
        inner.clock += 1;
        let clock = inner.clock;
        let hit = inner.entries.get_mut(key).map(|slot| {
            slot.0 = clock;
            slot.1.clone()
        });
        let counter = if hit.is_some() {
            &self.hits
        } else {
            &self.misses
        };
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        hit
    }

    pub fn insert(&self, key: RenderKey, blocks: Value) {
        // Rough size guard: serialized length is proportional to what the
        // entry actually costs in memory, and is cheap next to the weave we
        // just paid for.
        if serde_json::to_string(&blocks).map(|s| s.len()).unwrap_or(0) > MAX_ENTRY_BYTES {
            return;
        }
        let mut inner = self.inner.lock().expect("render cache poisoned");
        inner.clock += 1;
        let clock = inner.clock;
        inner.entries.insert(key, (clock, blocks));
        while inner.entries.len() > CAPACITY {
            let Some(victim) = inner
                .entries
                .iter()
                .min_by_key(|(_, (tick, _))| *tick)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            inner.entries.remove(&victim);
        }
    }

    /// (hits, misses) since process start — observability for the cache.
    pub fn stats(&self) -> (u64, u64) {
        use std::sync::atomic::Ordering::Relaxed;
        (self.hits.load(Relaxed), self.misses.load(Relaxed))
    }

    pub fn len(&self) -> usize {
        self.inner
            .lock()
            .expect("render cache poisoned")
            .entries
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn same_inputs_hit_changed_inputs_miss() {
        let cache = RenderCache::default();
        let doc = Uuid::new_v4();
        let t0 = Utc::now();
        let key = RenderKey::new(doc, "source v1", Some(t0), None);
        assert!(cache.get(&key).is_none());
        cache.insert(key.clone(), json!([{"kind": "prose"}]));
        assert_eq!(cache.get(&key), Some(json!([{"kind": "prose"}])));

        // Edited source → different weave.
        assert!(
            cache
                .get(&RenderKey::new(doc, "source v2", Some(t0), None))
                .is_none()
        );
        // A sibling doc was saved (an include may have changed).
        let t1 = t0 + chrono::Duration::seconds(1);
        assert!(
            cache
                .get(&RenderKey::new(doc, "source v1", Some(t1), None))
                .is_none()
        );
        // A run committed new outputs into the project checkout.
        assert!(
            cache
                .get(&RenderKey::new(doc, "source v1", Some(t0), Some(t1)))
                .is_none()
        );
    }

    #[test]
    fn stays_bounded_and_evicts_least_recently_used() {
        let cache = RenderCache::default();
        let first = RenderKey::new(Uuid::new_v4(), "keep-me", None, None);
        cache.insert(first.clone(), json!([]));
        for i in 0..CAPACITY * 2 {
            // Touch `first` so it stays the most recently used entry.
            assert!(cache.get(&first).is_some());
            cache.insert(
                RenderKey::new(Uuid::new_v4(), &format!("doc {i}"), None, None),
                json!([]),
            );
            assert!(cache.len() <= CAPACITY);
        }
        assert!(cache.get(&first).is_some(), "LRU evicted a hot entry");
    }

    #[test]
    fn oversized_renders_are_not_stored() {
        let cache = RenderCache::default();
        let key = RenderKey::new(Uuid::new_v4(), "huge", None, None);
        cache.insert(
            key.clone(),
            json!([{ "text": "x".repeat(MAX_ENTRY_BYTES + 1) }]),
        );
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.len(), 0);
    }
}
