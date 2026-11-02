<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Complete the implementation by layer

For engineers extending a component after following the [construction story](../../project.md).

The opening chapters bring the workspace, core evaluator, scaffold ingestion,
editors and persistence into a useful reading order. This reference includes the
remaining maintained modules by their role in that system. A component index
links directly to its code owners and tests; files already included by an opening
chapter retain their links here without a second construction.

The [explanation ledger](../../hickory-dogfood/narrative-map.json) still records
unresolved rationale. Neither a layer assignment nor an include establishes a
product guarantee. Use the related current contracts when changing behavior.

## Parse and construct the document

The language is shared across hosts. Conditional construction, reusable handlers and reactive flow support the core project model.

- [crates/hick-lang](crates/hick-lang/index.md)
<storycode:include file="crates/hick-lang/index.md" />

- [crates/hick-lang-wasm](crates/hick-lang-wasm/index.md)
<storycode:include file="crates/hick-lang-wasm/index.md" />

- [crates/hick-xml](crates/hick-xml/index.md)
<storycode:include file="crates/hick-xml/index.md" />

- [crates/hick-blocks](crates/hick-blocks/index.md)
<storycode:include file="crates/hick-blocks/index.md" />

- [crates/hick-handlers](crates/hick-handlers/index.md)
<storycode:include file="crates/hick-handlers/index.md" />

- [crates/hick-feature](crates/hick-feature/index.md)
<storycode:include file="crates/hick-feature/index.md" />

- [crates/hick-condition](crates/hick-condition/index.md)
<storycode:include file="crates/hick-condition/index.md" />

- [crates/hick-case](crates/hick-case/index.md)
<storycode:include file="crates/hick-case/index.md" />

- [crates/hick-flow](crates/hick-flow/index.md)
<storycode:include file="crates/hick-flow/index.md" />

- [crates/hick-exec](crates/hick-exec/index.md)
<storycode:include file="crates/hick-exec/index.md" />

## Execute with an explicit environment

The pipeline delegates process execution through the executor boundary. Local execution uses host tools; Docker, sandbox and Canopy backends have different enforcement properties. Inspect the selected implementation rather than inferring isolation from a container declaration.

- [crates/hick-literate](crates/hick-literate/index.md)
<storycode:include file="crates/hick-literate/index.md" />

- [crates/hickory-executor](crates/hickory-executor/index.md)
<storycode:include file="crates/hickory-executor/index.md" />

- [crates/hickory-executor-docker](crates/hickory-executor-docker/index.md)
<storycode:include file="crates/hickory-executor-docker/index.md" />

- [crates/hickory-executor-sandbox](crates/hickory-executor-sandbox/index.md)
<storycode:include file="crates/hickory-executor-sandbox/index.md" />

- [crates/hickory-executor-canopy](crates/hickory-executor-canopy/index.md)
<storycode:include file="crates/hickory-executor-canopy/index.md" />

- [crates/hick-project-env](crates/hick-project-env/index.md)
<storycode:include file="crates/hick-project-env/index.md" />

- [crates/hick-plugins](crates/hick-plugins/index.md)
<storycode:include file="crates/hick-plugins/index.md" />

- [crates/hick-classify](crates/hick-classify/index.md)
<storycode:include file="crates/hick-classify/index.md" />

## Trace, inspect and edit results

Byte lineage supports reverse editing. Language servers and structural navigation answer different questions; observation, coverage and profiling retain their own evidence limits.

- [crates/hickory-lineage](crates/hickory-lineage/index.md)
<storycode:include file="crates/hickory-lineage/index.md" />

- [crates/hick-lsp](crates/hick-lsp/index.md)
<storycode:include file="crates/hick-lsp/index.md" />

- [crates/hick-dap](crates/hick-dap/index.md)
<storycode:include file="crates/hick-dap/index.md" />

- [crates/hick-observe](crates/hick-observe/index.md)
<storycode:include file="crates/hick-observe/index.md" />

- [crates/hick-formula](crates/hick-formula/index.md)
<storycode:include file="crates/hick-formula/index.md" />

## Persist sessions and connect machines

Version storage, merging, local workspace state and collaborative rooms preserve different kinds of state. Fleet identity and the peer channel add cross-machine boundaries without changing the document language.

- [crates/hick-store](crates/hick-store/index.md)
<storycode:include file="crates/hick-store/index.md" />

- [crates/hick-merge](crates/hick-merge/index.md)
<storycode:include file="crates/hick-merge/index.md" />

- [crates/hick-grove](crates/hick-grove/index.md)
<storycode:include file="crates/hick-grove/index.md" />

- [crates/hickory-workspace](crates/hickory-workspace/index.md)
<storycode:include file="crates/hickory-workspace/index.md" />

- [crates/hickory-collab](crates/hickory-collab/index.md)
<storycode:include file="crates/hickory-collab/index.md" />

- [crates/hickory-fleet](crates/hickory-fleet/index.md)
<storycode:include file="crates/hickory-fleet/index.md" />

- [crates/hickory-peer](crates/hickory-peer/index.md)
<storycode:include file="crates/hickory-peer/index.md" />

## Compose applications and agent tools

The CLI hosts shared native behavior; the web and desktop applications compose the editor. Agent orchestration lives in the agent application, while Rust provider transports and document tools are shared mechanisms. The broker is a separate credential and egress boundary.

- [crates/hickory-cli](crates/hickory-cli/index.md)
<storycode:include file="crates/hickory-cli/index.md" />

- [apps/web](apps/web/index.md)
<storycode:include file="apps/web/index.md" />

- [apps/desktop](apps/desktop/index.md)
<storycode:include file="apps/desktop/index.md" />

- [crates/hick-context-store](crates/hick-context-store/index.md)
<storycode:include file="crates/hick-context-store/index.md" />

- [crates/hickory-agent](crates/hickory-agent/index.md)
<storycode:include file="crates/hickory-agent/index.md" />

- [apps/agent](apps/agent/index.md)
<storycode:include file="apps/agent/index.md" />

- [crates/hickory-broker](crates/hickory-broker/index.md)
<storycode:include file="crates/hickory-broker/index.md" />

## Integrate external languages and editors

Language model clients and editor adapters translate between existing ecosystems and the shared document engine. Keep their implementations at those boundaries.

- [code-models/clients](code-models/clients/index.md)
<storycode:include file="code-models/clients/index.md" />

- [code-models/csharp](code-models/csharp/index.md)
<storycode:include file="code-models/csharp/index.md" />

- [code-models/go](code-models/go/index.md)
<storycode:include file="code-models/go/index.md" />

- [code-models/typescript](code-models/typescript/index.md)
<storycode:include file="code-models/typescript/index.md" />

- [editors](editors/index.md)
<storycode:include file="editors/index.md" />

## Exercise and deliver this repository

Examples demonstrate authored documents; experiments retain their own scope. Dogfooding, scripts and VM checks maintain this repository and its delivery workflow.

- [examples](examples/index.md)
<storycode:include file="examples/index.md" />

- [experiments](experiments/index.md)
<storycode:include file="experiments/index.md" />

- [hickory-dogfood](hickory-dogfood/index.md)
<storycode:include file="hickory-dogfood/index.md" />

- [scripts](scripts/index.md)
<storycode:include file="scripts/index.md" />

- [vmtest](vmtest/index.md)
<storycode:include file="vmtest/index.md" />

## Keep generation operations shared across hosts

The project environment provider, native serving adapter and browser generation
control meet at the same explicit project boundary. Their focused component test
is included with that control; opening a document is not generation authority.

<storycode:include file="crates/hick-profile/index.md" />
<storycode:include file="crates/hick-secrets/index.md" />
<storycode:include file="crates/hick-sink/index.md" />
<storycode:include file="crates/hick-term/index.md" />
<storycode:include file="crates/hick-project-env/src/generation.rs.md" />
<storycode:include file="crates/hickory-cli/src/serve/generation.rs.md" />
<storycode:include file="apps/web/src/components/ProjectGeneration.tsx.md" />
<storycode:include file="apps/web/src/components/ProjectGeneration.test.tsx.md" />

<storycode:include file="scripts/compile-source-highlights.mjs.md" />

<storycode:include file="scripts/capture-source-language.py.md" />

<storycode:include file="apps/web/src/embed/staticLanguage.ts.md" />

<storycode:include file="apps/web/src/embed/staticLanguage.test.ts.md" />

<storycode:include file="scripts/test-capture-source-language.py.md" />

<storycode:include file="scripts/capture-source-models.py.md" />

<storycode:include file="apps/web/e2e/staticLanguage.spec.ts.md" />

<storycode:include file="code-models/typescript/source-index.js.md" />

<storycode:include file="code-models/typescript/source-index-schema.js.md" />

<storycode:include file="code-models/typescript/source-index.test.js.md" />

<storycode:include file="crates/hickory-cli/src/project_export.rs.md" />

<storycode:include file="scripts/map-source-index.py.md" />

<storycode:include file="scripts/static-highlights.mjs.md" />

<storycode:include file="scripts/static-highlights-entry.mjs.md" />

<storycode:include file="scripts/build-static-highlights.mjs.md" />

<storycode:include file="scripts/test-map-source-index.py.md" />

<storycode:include file="docs/users/static-export.md.md" />

<storycode:include file="apps/web/src/embed/staticArtifacts.ts.md" />

<storycode:include file="apps/web/src/embed/staticArtifacts.test.ts.md" />

<storycode:include file="apps/web/src/lsp/symbolInspection.ts.md" />

<storycode:include file="apps/web/src/lsp/symbolInspection.test.ts.md" />

<storycode:include file="apps/web/src/editor/paragraphReading.ts.md" />

<storycode:include file="apps/web/src/editor/paragraphReading.test.ts.md" />

</storycode:doc>
