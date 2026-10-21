# Review a synthetic authorization patch

For engineers preparing a reproducible security-patch evidence example with the
existing private prediction adapters.

[Vulnerable source](vulnerable.md) checks only that a record exists. The
[patch](patched.md) also checks that the requesting tenant owns it. Both sources
keep R-AUTH-1 (deny cross-tenant access), R-LEGIT-1 (preserve legitimate access),
the synthetic records, and the original output expectation unchanged.

The fixture uses no real targets, network endpoints, credentials or customer
data. It checks one authorization predicate; it does not assess authentication,
storage isolation, concurrency or other security properties.

From the repository root, with Python and a configured Python DAP adapter, run:

```sh
cargo test -p hick-literate --test prediction_authorization --quiet -- --include-ignored
```

This focused host witness seals output and first-hit `allowed` predictions before
each dispatch. The vulnerable revision fails both comparisons. The patched
revision passes the unchanged predictions and keeps the failure in the same
scenario history. Assertions inside both Python sources check each tenant's own
record and a missing record. Another witness correctly predicts the vulnerable
behavior and verifies that it still cannot satisfy the security requirement.

The ordinary executor and scratch DAP capture run the same document revision in
separate processes. The host fixture declares its environment identity; this is
not a fingerprint of an immutable toolchain or a sandbox certification.
Missing debugger evidence stays non-passing. Tests requiring DAP are explicitly
ignored by default rather than treating an unavailable adapter as a pass.

These files are reusable source templates. They are not uploaded reports,
reviewed publication assets or an application strict-mode workflow. To publish
them, use the existing explicit publication selection with exact revision and
source/dependency/environment identities, retain both failed and repaired
results, and label captured stops as recorded. Human-only actual values need an
isolation boundary from predictor context before publication. A static viewer
does not execute these sources or start a live debugger.
