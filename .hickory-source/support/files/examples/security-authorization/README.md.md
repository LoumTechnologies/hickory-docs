<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/security-authorization/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `70ee7a955d06555140b442869a1416b1dac5f27a7618a1b38b41f11942fce47e`.

<book:entry path="examples/security-authorization/README.md">
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
separate processes. Missing debugger evidence stays non-passing. DAP tests are
ignored by default; the command above explicitly runs them.

Inspect the recorded pair through the existing publication pipeline:

```sh
cargo build -p hickory-cli --bin hick
python3 scripts/publish-spec-results.py --hick target/debug/hick --output /tmp/hickory-selected-reports
```

Serve the generated directory alongside the snapshot viewer, then open
`snapshot.html?index=publications/index.json` using that index's served URL.
The index keeps the vulnerable report visibly failed and the patch passed. Each
report retains both attempts, exact source identities, sealed expectations and
recorded debugger coordinates. Revision navigation requires the selected index;
importing one report alone cannot navigate to another revision. Missing settlement
is unavailable. The viewer does not run a live debugger or grant current certification.

The checked-in `predictions.json` is a selected, answer-withheld host recording.
To make a new recording with Python and the configured debugpy adapter, choose a
new output path:

```sh
cargo run -p hick-literate --example authorization_evidence -- /tmp/authorization-predictions.json
```

The exporter fingerprints the configured interpreter/debugpy code before and
after both runs. This does not freeze the host or fingerprint every OS dependency.
Predictions are synthetic fixture declarations, not model decisions. Review the
new metadata before replacing the selected export and its SHA-256 pins in
`publications/selected.json`. The publisher refuses changed selected bytes.

No upload occurs in these commands. Actual values and private diagnostics stay
out of the selected recording, reports and predictor feedback. Human-only answer
publication needs a separate isolation boundary. This example is not yet an
application strict-mode workflow or a security audit.
</book:entry>

</book:doc>
