<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Verification

For: Reviewers deciding what an output, recording or prediction actually establishes.

Find the expected outcome and its falsifier before using a passing result as evidence.

- [Run explicitly selected literate CI checks](literate-ci/contract.md) — experimental repository-local runner.

- [A Drift Report Names What Differs](a-drift-report-names-what-differs/contract.md) — retained-requirement.
- [A Recording A Document Keeps Lives In The Document](a-recording-a-document-keeps-lives-in-the-document/contract.md) — retained-requirement.
- [Reuse a recording only for the inputs that produced it](a-recording-is-keyed-by-the-cells-inputs/contract.md) — reconciled-requirement.
- [A Report Is Not a Reproducible Artifact](a-report-is-not-a-reproducible-artifact/contract.md) — retained-requirement.
- [A Transform Is Checked Against The Bytes It Read, Wherever They Live](a-transform-is-checked-against-the-bytes-it-read/contract.md) — retained-requirement.
- [Keep an existing artifact when no recording can construct it](a-weave-without-a-recording-keeps-the-artifact/contract.md) — reconciled-requirement.
- [A Woven Fenced Block Has No Leading BOM](a-woven-fenced-block-has-no-leading-bom/contract.md) — retained-requirement.
- [An Expectation Means The Same Thing On Every Platform](an-expectation-means-the-same-on-every-platform/contract.md) — retained-requirement.
- [Browser-local prediction practice uses the shared gate](browser-predictions-use-the-shared-gate/contract.md) — retained-requirement.
- [Coverage retains its measured scope](coverage-retains-its-measured-scope/contract.md) — retained-requirement.
- [Freeze Is Declared Per Cell, With The Run-Wide Flag As Its Default](freeze-is-declared-per-cell/contract.md) — retained-requirement.
- [Keep a prediction separate from the answer it is tested against](predictions-precede-managed-observation/contract.md) — rewritten.
- [One Comparison Behind Test, Equiv, The Pin, And The Merge Check](one-comparison-behind-test-equiv-pin-and-merge/contract.md) — retained-requirement.
- [Private edit proposals open review before writing](private-proposals-wait-for-review/contract.md) — retained-requirement.
- [Recordings Are Written Only When A Run Is Asked To Write One, And Never By `test`](recordings-are-written-only-when-asked-for/contract.md) — retained-requirement.
- [The Pre-Commit Hook `hick init` Installs Names Which Outcome Blocked The Commit](the-installed-hook-names-the-outcome/contract.md) — retained-requirement.
- [`hick test` Fails When Documented Output Drifts From Real Output](test-fails-on-drift/contract.md) — retained-requirement.
- [Report whether verification found drift, no baseline or a false expectation](test-separates-unverifiable-from-drifted/contract.md) — reconciled-requirement.

<book:include file="a-drift-report-names-what-differs/contract.md" />
<book:include file="a-recording-a-document-keeps-lives-in-the-document/contract.md" />
<book:include file="a-recording-is-keyed-by-the-cells-inputs/contract.md" />
<book:include file="a-report-is-not-a-reproducible-artifact/contract.md" />
<book:include file="a-transform-is-checked-against-the-bytes-it-read/contract.md" />
<book:include file="a-weave-without-a-recording-keeps-the-artifact/contract.md" />
<book:include file="a-woven-fenced-block-has-no-leading-bom/contract.md" />
<book:include file="an-expectation-means-the-same-on-every-platform/contract.md" />
<book:include file="browser-predictions-use-the-shared-gate/contract.md" />
<book:include file="coverage-retains-its-measured-scope/contract.md" />
<book:include file="freeze-is-declared-per-cell/contract.md" />
<book:include file="one-comparison-behind-test-equiv-pin-and-merge/contract.md" />
<book:include file="predictions-precede-managed-observation/contract.md" />
<book:include file="private-proposals-wait-for-review/contract.md" />
<book:include file="recordings-are-written-only-when-asked-for/contract.md" />
<book:include file="test-fails-on-drift/contract.md" />
<book:include file="test-separates-unverifiable-from-drifted/contract.md" />
<book:include file="the-installed-hook-names-the-outcome/contract.md" />

<book:include file="literate-ci/contract.md" />
</book:doc>
