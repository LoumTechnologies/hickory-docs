use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
/// One installed source/ownership mapping, independent of provider diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceRevision(u64);
impl SourceRevision {
    pub(crate) fn fresh() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}
/// Content-free identity of the installed mapping graph. Replacement is conservative.
pub(crate) fn source_graph_fingerprint<'a>(
    rows: impl Iterator<Item = (&'a str, SourceRevision)>,
) -> String {
    let mut rows: Vec<_> = rows.collect();
    rows.sort_by(|a, b| a.0.cmp(b.0));
    let mut digest = Sha256::new();
    digest.update(b"hickory-provider-source-graph-v1\0");
    for (uri, revision) in rows {
        digest.update((uri.len() as u64).to_be_bytes());
        digest.update(uri.as_bytes());
        digest.update(revision.0.to_be_bytes());
    }
    format!("{:x}", digest.finalize())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_source_identity_rejects_replacement_removal_and_reordering_only_is_stable() {
        let a = SourceRevision::fresh();
        let b = SourceRevision::fresh();
        let fingerprint =
            |rows: Vec<(&str, SourceRevision)>| source_graph_fingerprint(rows.into_iter());
        let first = fingerprint(vec![("a", a), ("b", b)]);
        assert_eq!(first, fingerprint(vec![("b", b), ("a", a)]));
        assert_ne!(
            first,
            fingerprint(vec![("a", SourceRevision::fresh()), ("b", b)])
        );
        assert_ne!(first, fingerprint(vec![("a", a)]));
        assert_ne!(first, fingerprint(vec![("other-owner", a), ("b", b)]));
    }
}
