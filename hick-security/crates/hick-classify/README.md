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
