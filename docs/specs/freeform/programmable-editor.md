# A programmable development environment: resources, relationships, views, and commands

Status: proposed, October 25, 2026; expanded to include provenance and alternative
development workflows. This is a design grounded in the current
editor implementation. The SDK and wire messages below are illustrative;
they are not available APIs. Implementation tracking belongs in GitHub issues.

Audience: Hickory contributors designing editor extensibility, and developers
who want to write a personal visualization without maintaining an IDE fork.

## The experience we want

A developer asks an agent: “Show allocation pressure beside each function, let
me expand its history, and give me a button to capture another sample.” The
agent writes a small local program. The developer enables it, and its results
appear in the same editor as coverage, language information, and debugging.
They can change its display, inspect its data, disable it, or share its source.
Publishing a package is optional.

Hickory should supply the machinery for that experience. A built-in feature,
a personal script, and a shared extension should use the same editor contribution
contract. An agent can author a program or operate its controls through that
contract. Neither activity requires injecting code into the editor process.

The core model has **resources, relationships, views, and commands**. A resource
can be source text, an agent attempt, a test result, or a structured record.
Relationships connect particular versions and locations within resources. Views
present either; commands interact with them. A source-anchored editor
contribution is the common case explored first below.

Gutter marks, semantic colors, coverage backgrounds, debugger values, charts,
forms, provenance ribbons, and relationship inspectors use this model. Their
producers keep their own domain semantics. Hickory's literate documents and
agent workflow become one application of the framework. Someone can use the
framework with ordinary source files and their own development model, with no
requirement to author Hickory documents or use Hickory's agent harness.

## What is already there

The editor uses CodeMirror extensions. The duplication is in the feature
integration above those primitives:

| Feature | Current integration | Behavior the shared system must preserve |
|---|---|---|
| Coverage | [`coverage/gutter.ts`](../../../apps/web/src/coverage/gutter.ts) owns state, background decorations, gutter markers, and refresh subscriptions | Current-source marks require matching captured source hashes; edits clear them |
| Profiling | [`performance/inline.ts`](../../../apps/web/src/performance/inline.ts) owns block widgets, source checks, disclosure, and agent actions | Function costs retain units, aggregation, run identity, and attribution precision |
| LSP | [`lsp/cmLsp.ts`](../../../apps/web/src/lsp/cmLsp.ts) and [`cmLspFeatures.ts`](../../../apps/web/src/lsp/cmLspFeatures.ts) own diagnostics, semantic marks, hints, and readings | Position mapping and provider refresh semantics remain explicit |
| Debugging | [`debug/cmDebug.ts`](../../../apps/web/src/debug/cmDebug.ts) and [`inlineEval.ts`](../../../apps/web/src/debug/inlineEval.ts) own gutter controls, paused state, and evaluation forms | Commands and replies belong to a particular source, session, stop, and frame |
| Provenance | [`shell/Ribbons.tsx`](../../../apps/web/src/shell/Ribbons.tsx), [`lib/lensSources.ts`](../../../apps/web/src/lib/lensSources.ts), and [`lib/provenanceLayers.ts`](../../../apps/web/src/lib/provenanceLayers.ts) connect registered views using built-in relationship families | Exact anchors, discoverable closed targets, distinct meanings, and navigation survive generalization |

[`editorLanguageActions`](../../../apps/web/src/editor/languageActions.ts)
already exposes editor controls to agents. Runtime/profile readings already
carry evidence and source identity in
[`ObservationReading`](../../../apps/web/src/debug/observations.ts).
[`EditingSurface`](../../../apps/web/src/embed/EditingSurface.tsx) accepts
CodeMirror extensions. These are starting seams, not a finished public SDK.

The Rust registry in `crates/hick-plugins` describes built-in language
contributions. It does not establish a runtime for arbitrary user programs.
Extensibility needs an explicit host; adding another registry entry is insufficient.

## Make provenance a programmable relationship view

Yes: the bands, braces, endpoint highlights, filtering, and click-to-reveal
behavior should be available to a personal program. A developer can supply new
relationships, choose another presentation of existing relationships, or do both.
Drawing a connection and establishing what it means are separate operations.

For example, a developer whose workflow begins with requirements could connect:

```text
requirement → agent attempt → proposed patch → test run → review decision
```

Another could connect competing hypotheses to experiments, or model-generated
functions to independent review notes. Each supplies its own resource schemas,
relationship types, collectors, and commands. None needs to translate those
objects into `hick:copy`, a Hickory session, or byte lineage.

### Endpoints name resources, not pixels

A resource has a namespaced identity, a version, a schema, and a resolver. A
location identifies a text range, a structured record or field, or the whole
resource with an explicit precision. External resource versions need not be
content hashes; the provider must state their stability and retain the content
needed for historical inspection. Files and unsaved buffers use host identities.

A view registers which resource/version it presents and how supported locations
map to host-managed anchors. For editor text the host measures ranges; for
native tables it measures rows or cells. An isolated renderer requests anchors
within its allocated bounds through its bridge. Extension protocol messages
never carry DOM handles or arbitrary screen coordinates.

The relationship renderer resolves anchors in open views, or a named tab,
inspector entry, or navigation port when the target is closed. Folded and
offscreen endpoints retain explicit fallback states. A missing historical
version remains missing; a newer record cannot silently take its place.
Closed-target resolution does not execute a resource merely to draw its link.

An illustrative relationship publication could be:

```ts
// Proposed protocol shape; references resolve through registered resource providers.
await layer.replaceRelationships({
  generation: 7,
  relationships: [{
    id: "attempt-12-addresses-requirement-8",
    type: "my.workflow/addresses",
    from: { resource: attemptId, version: "12", location: { record: "summary" } },
    to: { resource: requirementId, version: "3", location: { record: "8" } },
    explanation: "The author says this attempt addresses requirement 8",
    evidence: [{ resource: reviewId, version: "1" }],
  }],
});
```

The host stamps producer identity. Relationship type registration provides a
label, direction, schema, explanation, default appearance, and optional commands.
Evidence references are references, not proof. Hyperedges or many-to-many
relationships can use an explicit activity resource with role-labeled edges,
preserving the joint operation instead of implying independent pairwise causes.

### Keep the vocabulary open and the evidence inspectable

Today's fixed `ProvenanceLayer` and `RibbonFamily` unions cannot be the public
extension vocabulary. Replace that boundary with namespaced relationship types
and discoverable layers. Hickory's lineage, context, declared, continuity, and
execution providers register their existing meanings through adapters.

The generic host records who published a relationship, its supplied evidence,
and any independent verifier result. A verifier result names the verifier,
version, checked inputs, result, and actual scope of the check. Providers cannot
assign themselves a host verification badge. Third-party verifiers remain
identified as such, even when the user trusts them.

Hickory's byte correspondence verifier can establish a byte mapping. A different
workflow can verify a test receipt, or leave a relationship as an author assertion.
There is no mandatory universal theory of causation, correctness, or how agents
should work. Schemas and verifiers can be supplied by a workflow package.
Provider attribution, version identity, and access boundaries remain host facts.

Users can customize palettes, strokes, grouping, direction, visibility, and
presentation as ribbons, braces, lists, or graphs. The inspector and accessible
labels always expose relationship meaning, producer, precision, and evidence
status. A styling choice cannot confer verification. The Hickory workflow's
default view preserves its distinctions between byte derivation, supplied
context, citations, and runtime observations.

Relationship reads, filtering, selection, navigation, evidence inspection, and
commands use the same agent interface as editor contributions. Exporting or
sharing a graph requires an explicit data selection: a relationship's existence
does not grant access to private endpoint content or its evidence.

## Let workflow packages augment or replace the development model

Custom relationship types alone are insufficient. A workflow package must also
be able to register resource adapters, native or isolated views, inspectors,
commands, and default workspace arrangements. It can provide subscriptions and
automation under explicit capability grants. For example, a review-centered
package could open an attempts board beside a patch editor, with test receipts
and approval commands, and use no literate-document execution at all.

The host supplies editing, resource/version resolution, view layout, anchor
geometry, rendering, command dispatch, access control, and lifecycle management.
The package supplies domain objects, relationship semantics, orchestration,
storage adapters, and verification policies. A resource's edit command uses its
owner's concurrency and persistence rules; a generic graph does not make
every node writable or create a second universal storage system.

Hickory Docs remains a supported workflow package with its own syntax, engine,
source mappings, provenance rules, and defaults. Generalizing the UI must
preserve its behavior while removing those concepts as prerequisites for other
packages. The wider architectural test is whether another package can replace
or augment the primary working view and workflow without modifying the host or
fabricating Hickory documents to hold its data.

Augmentation is a first-class use case. A developer can keep Hickory's literate
documents and agent workflow while adding a requirements layer, an alternative
review process, or relationships from another agent harness. Several packages
can contribute to the same workspace and views. Each owns its namespaced data,
commands, and defaults; enabling one cannot silently take over another's layout,
relationship meanings, or automation. Cross-package data access uses explicit
schema-versioned interfaces and capability grants. The user chooses which
contributions to combine and which defaults to adopt.

## Write one small program

Start with a JSON-producing process and host-rendered components. Any language
can produce the protocol; a TypeScript SDK can make the common path short.
An illustrative program could look like this:

```ts
// Proposed SDK, not executable against today's Hickory.
const layer = await ide.layers.open({ id: "my.allocations", label: "Allocations" });
const command = await layer.command({
  id: "capture",
  label: "Capture another sample",
  input: { type: "object", properties: {}, additionalProperties: false },
}, async (_args, context) => captureThroughGrantedTool(context));

const source = await ide.buffers.snapshot(bufferId);
const sample = await allocationsFor(source); // Developer-supplied collector.
await layer.replace({
  source: source.identity, // Includes buffer revision and serialized-source hash.
  generation: sample.generation,
  scope: { run: sample.id },
  items: sample.functions.map(fn => ({
    id: fn.id,
    anchor: { range: fn.range, unit: "utf16", precision: "function" },
    data: { bytes: fn.bytes, history: fn.history },
    views: [
      { kind: "gutter", icon: "meter", label: `${fn.bytes} allocated bytes` },
      { kind: "below-line", content: {
        type: "disclosure", label: `${fn.bytes} allocated bytes`, children: [
          { type: "sparkline", values: fn.history, unit: "bytes" },
          { type: "button", label: "Capture another sample", command: command.id },
        ],
      } },
    ],
  })),
});
```

The developer explicitly enables the program in a workspace or in personal
settings. The host checks its declared API version and requested capabilities,
starts it outside the UI process, and gives it scoped handles. Its published
batch is validated and projected into each applicable editor. Clicking the
button sends a command to that program through the same dispatcher used by an
agent. The next capture publishes a newer generation.

The first SDK should ship a runnable version of this example, including a fixture
collector, manifest, launch command, and disable path. A proposed API example
alone does not establish that personal customization works.

## Three parts with separate responsibilities

**Providers produce data.** An adapter translates coverage, an LSP response,
a debugger event, or a custom analysis into contributions. It supplies stable
item IDs, source identity, payload schema, scope, and attribution precision.
Run data can refer to retained evidence; transient hints need no invented run.
Providers remain responsible for what their numbers mean.

**Views present data.** A layer chooses one or more placements for each item.
Switching a hotspot from a gutter meter to an expanded chart preserves its item
and evidence IDs. Presentation state such as disclosure belongs to the window;
it is separate from the captured data. Different windows can choose different
views of the same measurement.

**Commands perform interactions.** A command declares input schema, current
availability, required capabilities, and a revision precondition. Native clicks,
keyboard actions, and agent actions call the same implementation. Provider
commands can request existing host operations; access to a view grants no
implicit authority to edit files or evaluate code.

This separates data collection from visualization. A developer should also be
able to write a new view of an existing profile without implementing a profiler.
Reading that data uses a separately granted, schema-versioned subscription.

### A bounded vocabulary that can grow

| Placement | Initial uses |
|---|---|
| Gutter lane | Breakpoints, coverage, heat meters, custom markers |
| Text range | Semantic foreground, underline, outline, background |
| Line | Whole-line background or emphasis |
| Inline at an anchor | Inlay hints, compact values, evaluation input |
| Block above/below a line | Tables, expandable values, charts, forms |
| Hover or inspector | Details and overflow that would crowd the editor |

Host-rendered components include text, badges, disclosure, lists, tables,
trees, buttons, inputs, and charts with accessible summaries. Strings are text;
the protocol accepts no HTML, CSS selectors, JavaScript callbacks, or DOM handles.
Style uses theme roles and scoped, validated color values.

Some developers will need a visualization beyond that vocabulary. Provide a
separate custom-renderer capability using a sandboxed frame with a versioned
message protocol. It receives only explicitly supplied data and emits validated
interaction messages. The host controls placement, size limits, focus entry and
exit, and a text fallback. Isolation must deny parent DOM, ambient credentials,
and ungranted network access. This boundary needs its own implementation and
security verification; the existing execution sandbox is not a browser sandbox.

CodeMirror stays a host implementation detail. Host-rendered contributions work
in every supported editor surface; custom frames can advertise platform limits
and fall back to the inspector. Direct CodeMirror extensions remain an internal
integration technique rather than the portable user API.

## Coordinates and time are part of the contract

A batch identifies workspace, source URI or unsaved buffer identity, source
revision, provider instance, and generation. Each anchor declares its coordinate
unit and precision. Accept UTF-16 ranges for editor snapshots and UTF-8 byte
ranges for captured source; convert through the host's checked source mapping.
Line/column adapters must declare their encoding and base. Reject invalid ranges
instead of silently clamping them onto unrelated code.

Generated files and literate fragments use existing source mappings. One source
location can appear in several views. Unmapped or synthetic text has no invented
origin: it can appear in the inspector with its original location. A function
hotspot anchored at a line must continue to say it measures a function.

Separate **where an anchor moved** from **whether the data is still valid**:

- Exact-source measurements, including coverage and profiles, lose current-source
  placement after an edit. Their historical data remains inspectable against the
  captured revision. Matching bytes alone do not establish a still-current run.
- Provider-refresh annotations, including semantic tokens, may be mapped across
  edits as provisional where their adapter supports it. The host exposes that
  state and replaces it only with a response for the correct revision.
- User bookmarks can follow edits with declared anchor affinity. That behavior
  cannot upgrade a measurement into evidence about the edited program.
- Paused debugger controls additionally require the same live session, stop,
  thread, and frame. Resume retires the control even when source bytes match.

Replacement is atomic within a provider's layer and scope. Generations increase
within a host-issued provider-instance identity; restart gets a new identity.
Late batches and replies are rejected after replacement, source retirement, or
workspace switch. Disposal removes subscriptions, views, and command handles.
Visibility virtualization does not discard the underlying data or change which
expanded evidence the user selected for agent context.

## Make several layers readable together

The host owns composition. Providers cannot win collisions by choosing an
arbitrarily large priority or z-index.

User preferences select foreground ownership, background heatmaps, gutter order,
and which blocks expand by default. Semantic color can coexist with diagnostic
underlines and a coverage background. Competing foregrounds use the user's
selected owner; competing backgrounds use an explicit selected layer or a
documented blend. All contributing items remain available in the inspector.
Producer identity, verification status, and diagnostic severity retain
recognizable host-owned indicators. Workflow-specific relationship styles
come from the selected package and the user's preferences.

Gutter lanes have a width budget and an overflow control with named entries.
Several inline values collapse into an inspectable summary. Block views have
height limits and expand deliberately. Layer controls let the user mute, isolate,
reorder, inspect, and disable a provider, with its identity always discoverable.
Source selection, copying, search, and undo continue to operate on source text;
rendered values enter source only through an explicit edit command.

## Programmable by humans and agents

Expose layer discovery, data schemas, item readings, display preferences, and
available commands through the existing UI action/read bridge. Agent reads
carry the same IDs, source revision, stale state, units, and truncation as the
visible presentation. Large payloads support bounded reads rather than dumping
every chart point into context. View data remains inspectable even when a custom
renderer supplies pixels rather than native controls.

Commands check availability and revision at dispatch, and check ownership again
before adopting a reply. Debug evaluation may execute user code and have side
effects. A cancelled request reports whether dispatch occurred; cancellation
does not promise rollback. A view cannot label evaluation as verification.
Expansion can select evidence through the existing observation/context path;
automatic agent access to unrelated layers is a separate grant.

An agent can create or revise a local provider, reload it, and inspect errors.
Activation and capability grants follow the person's existing authorization.
Generated code and repository configuration do not grant themselves access.
The same applies when a developer imports someone else's extension.

## Local first, shareable when useful

A proposed manifest declares identity, API version, entry point, data schemas,
supported environments, and requested capabilities. Personal programs can live
outside the repository; workspace programs can be versioned with the project.
Opening a repository never runs its programs automatically. An explicitly
enabled provider can reload during development, retiring its old handles.

Capabilities distinguish buffer reads, selected evidence reads, debugger access,
source edits, process execution, filesystem scope, and network destinations.
Host brokers enforce these boundaries. A subprocess with unrestricted inherited
credentials is not isolated merely because its manifest lists fewer rights.
Start with declarative data imports or a genuinely restricted worker/process;
an intentionally trusted local executable must be described as such.

Sharing initially means sharing inspectable source and a manifest. Dependencies
and shared versions should be pinned, and changed requested capabilities need
an updated grant. A marketplace, account, signing service, and automatic updates
are independent distribution choices. A personal program needs none of them.

Provider failure stays local to its layer. Validate message sizes, item counts,
update frequency, chart points, nesting, and rendering height. Coalesce updates,
virtualize visible content, and stop a producer that exceeds its budget. Show
its error and a restart/disable control without replacing the editor. The
keyboard edit path must never wait for a provider process.

## A migration that proves the abstraction

The first implementation slice should expose the contribution store, checked
anchors, layer ownership, and CodeMirror renderer. Move coverage and profiling
through it while retaining their existing collectors, source checks, readings,
and controls. This exercises both gutter/background composition and interactive
block content, including simultaneous display.

Then run a personal provider through a public, versioned protocol using the same
renderer. The allocation example should work in an ordinary file, a generated
file, and a literate fragment. That is the first proof that this is programmable
outside the application source tree.

LSP marks/hints and debugger controls follow through adapters. Preserve their
different refresh and execution lifetimes; do not make `ObservationReading`'s
runtime scope mandatory for a semantic token. Keep parsing, execution, retained
evidence storage, and source mappings with their existing owners. Replacing
those backends is unnecessary to unify their presentation.

Provenance migration should extract a resource-location resolver and relationship
renderer from the existing ribbon overlay. Keep Hickory-specific block expansion
(currently in `shell/ribbonMeasurements.ts`) in its adapter. Move built-in family
labels, meanings, and defaults into registered relationship definitions. Existing
lineage derivation continues to produce the same evidence; the new host draws
it through the public relationship contract.

Prove the broader boundary with a small alternative workflow over ordinary
files, requirement records, and test receipts. It must register its own
relationships, open its own primary view, navigate to versioned endpoints,
and expose a command to both keyboard and agent, with Hickory document/session
providers disabled. Passing only the existing literate-document flows would
not demonstrate independence from Hickory's development model.

Also prove augmentation: enable a requirements/review package alongside Hickory's
existing document, provenance, and agent providers. Both must contribute to the
same views, remain independently configurable, and expose their own commands.
Disabling the added package must preserve Hickory's data and working experience.

Custom sandboxed renderers follow when a real visualization exceeds native
components. The public protocol should already leave room for a renderer kind
with a declared fallback, without promising arbitrary UI execution in v1.

Focused acceptance witnesses should cover:

- Coverage and profiling together, preserving counts, units, attribution, and
  agent readings; disabling one removes only its contributions.
- Unicode and CRLF through unsaved buffers, generated mappings, and fragmented
  literate views; stale or unmappable data never attaches to guessed source.
- A delayed publish after edit, restart, close, or workspace switch, and a debug
  evaluation arriving after resume or frame change.
- A personal interactive view operated by keyboard and agent using the same
  command, including revision refusal and cancellation after dispatch.
- Custom relationships between text and structured records, including closed,
  folded, missing, and historical targets; built-in provenance keeps its meaning.
- An alternative workflow with no Hickory document syntax or agent harness;
  asserted relationships cannot acquire host verification through styling.
- An added workflow alongside Hickory's existing workflow, with independent
  configuration, namespaced commands, and removal of only its own contributions.
- Malformed data, a provider crash, a flood of updates, denied capabilities,
  and a custom renderer attempting to access parent DOM or ungranted network.

No runtime migration or executable SDK is implemented by this proposal. The
first slice must demonstrate these contracts before the remaining features
are moved onto it.

## Relationship to existing designs

This proposes a broader UI boundary than [the minimal core](the-minimal-core.md):
registered resources can exist independently of Hickory line-numbered documents.
It preserves authored document syntax and element storage for the Hickory workflow.
It follows
[coherent working surfaces](coherent-working-surfaces.md) for scope and errors,
and [execution observation provenance](execution-observation-provenance.md)
for captured measurements and historical readings. [Lenses](lenses.md) remain
views over substrates; a contribution decorates those views and acquires no
independent save destination. [Three provenances](three-provenances.md) continues
to define the Hickory package's relationship semantics. Its fixed family list
does not restrict what other packages can represent. These are proposed
architectural changes, not a claim that the current application is already
independent of its document model.
