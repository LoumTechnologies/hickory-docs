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
