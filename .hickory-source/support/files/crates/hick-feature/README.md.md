<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-feature/README.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `aa063d6fba8c6d6730d1274e4f0bef1f167a09a3f763b7b2803c3449982e0707`.

<book:entry path="crates/hick-feature/README.md">
# hick-feature

DAG-based feature flag system for conditional code generation.

`FeatureDef` describes a feature with its name, description, `requires`
dependencies, `conflicts_with` exclusions, and optional `exclusive_group`.
`FeatureRegistry` validates the full registry via DFS cycle detection.

`FeatureSet` holds the currently-enabled features and implements
`validate_and_expand`, which computes the transitive closure of all required
dependencies and checks for conflicts before returning the resolved set.

The pipeline uses this crate to honour `<hick:feature>` declarations and
`--features` CLI flags, ensuring that feature combinations are coherent before
any code is generated.
</book:entry>

</book:doc>
