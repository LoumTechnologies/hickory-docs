<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-classify/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `8b1a549e22891f0bb8d010ddd199e30be7654ee6ea40dc58d1ba76a1f8d82e66`.

<book:entry path="crates/hick-classify/README.md">
# hick-classify

Data classification labels and masking specifications used across the security
stack.

A pure data-definition crate with no I/O, depended on by `hick-policy`,
`hick-token`, and `hick-sink`.

## Key types

- `Classification` — a `Vec<serde_json::Value>` wrapper supporting union and
  deduplication of free-form label values
- `ColumnClassifications` — per-column labels with projection support
- `MaskSpec` — describes a masking operation for a named column:
  - `Mask` — partial redaction
  - `Redact` — full removal
  - `Hash` — BLAKE3 hash (via optional `host-mask` feature on Arrow/Parquet data)
  - `Bucket` — value bucketing
</book:entry>

</book:doc>
