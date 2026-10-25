<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Execution

For: Authors running documents on their own machine and reviewers assessing their side effects.

Read the execution boundary before running an unfamiliar document; inspect the refusal and cleanup paths.

- [A Cell Cannot Hang A Run](a-cell-cannot-hang-a-run/contract.md) — retained-requirement.
- [A Cell's Own Path Belongs To Its Project, Not To Your Shell](a-cells-own-path-belongs-to-its-project/contract.md) — retained-requirement.
- [A Collaborative Session Never Duplicates the Document](collab-sessions-never-duplicate-the-document/contract.md) — retained-requirement.
- [A Command The App Runs On Your Behalf Is Watched In A Terminal](a-command-the-app-runs-is-watched-in-a-terminal/contract.md) — retained-requirement.
- [A Denied Mach Lookup Explains Itself](a-denied-mach-lookup-explains-itself/contract.md) — retained-requirement.
- [A Document Says What It Needs](a-document-says-what-it-needs/contract.md) — retained-requirement.
- [A Document's First Run Behaves Like Every Later Run](first-run-behaves-like-every-later-run/contract.md) — retained-requirement.
- [A Formula Is An Expression In A Real Language](a-formula-is-an-expression-in-a-real-language/contract.md) — retained-requirement.
- [A Sandboxed Cell Cannot Reach Past Its Workdir](a-sandboxed-cell-cannot-reach-past-its-workdir/contract.md) — retained-requirement.
- [A Test Runs From The Line It Is Written On](a-test-runs-from-the-line-it-is-written-on/contract.md) — retained-requirement.
- [Apply repository exclusions when seeding execution inputs](a-volume-carries-what-the-repository-carries/contract.md) — reconciled-requirement.
- [A failed action keeps its editor](a-failed-action-keeps-its-editor/contract.md) — retained-requirement.
- [A run result is distinct from what was checked](a-run-result-is-distinct-from-what-was-checked/contract.md) — retained-requirement.
- [An Earlier Cell's Output Survives A Later Cell's Failure](an-earlier-cells-output-survives-a-later-cells-failure/contract.md) — retained-requirement.
- [An Output Path Declared Twice Warns Before It Runs](an-output-path-declared-twice-warns-before-it-runs/contract.md) — retained-requirement.
- [An Output Path Never Escapes The Document's Directory](output-paths-stay-inside-the-project/contract.md) — retained-requirement.
- [An Output Volume Flushes Only What The Run Changed](an-output-volume-flushes-only-what-the-run-changed/contract.md) — retained-requirement.
- [Browser output carries actual runtime source and occurrence evidence](browser-output-has-runtime-source-evidence/contract.md) — retained-requirement.
- [Carry a generated-text edit back to its local source](output-lineage-round-trips-byte-for-byte/contract.md) — rewritten.
- [Containers Are Offline By Default](containers-are-offline-by-default/contract.md) — retained-requirement.
- [Declared Capabilities Are Enforced](declared-capabilities-are-enforced/contract.md) — retained-requirement.
- [Every Desktop Platform Can Confine A Cell](every-desktop-platform-can-confine-a-cell/contract.md) — retained-requirement.
- [Every Doc Save Is a Commit in the Project's Git Repository](doc-saves-commit-to-project-git/contract.md) — retained-requirement.
- [Execution logs preserve contributor relationships and historical event identity](runtime-logs-preserve-contributor-relationships/contract.md) — retained-requirement.
- [Only `hickory-executor-canopy` Knows Cloud Canopy's API](canopy-api-isolated-to-one-crate/contract.md) — retained-requirement.
- [Profiles keep their measured scope](profiles-keep-their-measured-scope/contract.md) — retained-requirement.
- [Python observations stay confined and source bound](python-observations-stay-confined/contract.md) — retained-requirement.
- [Python profiles keep function scope and measurement semantics](python-profile-keeps-function-semantics/contract.md) — retained-requirement.
- [Rendering a Document Never Blocks the Server](render-never-blocks-the-server/contract.md) — retained-requirement.
- [Runtime history keeps runs and invocation occurrences separate](runtime-history-keeps-runs-and-occurrences-separate/contract.md) — retained-requirement.
- [Runtime observations retain their producing revision and occurrence](revision-bound-runtime-observations/contract.md) — retained-requirement.
- [Stepping A Table Replays The Order The Host Chose](stepping-a-table-replays-the-order-the-host-chose/contract.md) — retained-requirement.
- [The Declared Image Is the Environment](the-declared-image-is-the-environment/contract.md) — retained-requirement.

<book:include file="a-cell-cannot-hang-a-run/contract.md" />
<book:include file="a-cells-own-path-belongs-to-its-project/contract.md" />
<book:include file="a-command-the-app-runs-is-watched-in-a-terminal/contract.md" />
<book:include file="a-denied-mach-lookup-explains-itself/contract.md" />
<book:include file="a-document-says-what-it-needs/contract.md" />
<book:include file="a-failed-action-keeps-its-editor/contract.md" />
<book:include file="a-formula-is-an-expression-in-a-real-language/contract.md" />
<book:include file="a-run-result-is-distinct-from-what-was-checked/contract.md" />
<book:include file="a-sandboxed-cell-cannot-reach-past-its-workdir/contract.md" />
<book:include file="a-test-runs-from-the-line-it-is-written-on/contract.md" />
<book:include file="a-volume-carries-what-the-repository-carries/contract.md" />
<book:include file="an-earlier-cells-output-survives-a-later-cells-failure/contract.md" />
<book:include file="an-output-path-declared-twice-warns-before-it-runs/contract.md" />
<book:include file="an-output-volume-flushes-only-what-the-run-changed/contract.md" />
<book:include file="browser-output-has-runtime-source-evidence/contract.md" />
<book:include file="canopy-api-isolated-to-one-crate/contract.md" />
<book:include file="collab-sessions-never-duplicate-the-document/contract.md" />
<book:include file="containers-are-offline-by-default/contract.md" />
<book:include file="declared-capabilities-are-enforced/contract.md" />
<book:include file="doc-saves-commit-to-project-git/contract.md" />
<book:include file="every-desktop-platform-can-confine-a-cell/contract.md" />
<book:include file="first-run-behaves-like-every-later-run/contract.md" />
<book:include file="output-lineage-round-trips-byte-for-byte/contract.md" />
<book:include file="output-paths-stay-inside-the-project/contract.md" />
<book:include file="profiles-keep-their-measured-scope/contract.md" />
<book:include file="python-observations-stay-confined/contract.md" />
<book:include file="python-profile-keeps-function-semantics/contract.md" />
<book:include file="render-never-blocks-the-server/contract.md" />
<book:include file="revision-bound-runtime-observations/contract.md" />
<book:include file="runtime-history-keeps-runs-and-occurrences-separate/contract.md" />
<book:include file="runtime-logs-preserve-contributor-relationships/contract.md" />
<book:include file="stepping-a-table-replays-the-order-the-host-chose/contract.md" />
<book:include file="the-declared-image-is-the-environment/contract.md" />

</book:doc>
