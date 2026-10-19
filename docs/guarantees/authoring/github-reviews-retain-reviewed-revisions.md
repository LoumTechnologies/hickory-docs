# GitHub reviews retain reviewed revisions and external identity

## Requirements

The existing Review threads panel explicitly selects owner/repository, PR number
and saved workspace document. The read-only provider uses the person's existing
`gh` credentials, without reading/storing tokens in Hickory. It cursor-paginates
threads, each thread's replies and ordinary PR conversation comments, refusing
cursor loops and exposing bounded/incomplete capture. Loading, permissions,
source-unavailable and refresh failure are visible. A failed refresh leaves the
previous session capture historical; no absence or current resolution is inferred.

Review annotations preserve repository/PR/thread/comment IDs, authors, creation
and edit times, exact GitHub discussion links, resolution/outdated state, paths,
LEFT/RIGHT side, current/original multiline positions and original diff context.
RIGHT source is fetched at the original reviewed commit. Current diff-base source
uses GitHub's reported comparison merge-base, not the moving base branch tip.
Historical LEFT bases that cannot be established remain explicitly unknown.
Review evidence has no fabricated runtime run/stop/frame.

Current-source attachment requires matching repository identity, non-outdated
reviewed head, exact full reviewed-file bytes and a deterministic existing
literate source mapping. Current plain documents and generated fragments use the
same immutable UTF-8 source revision/location handles as other providers. Git's
explicit rename relation is usable when local history is available. Missing
history, changed/deleted/ambiguous files, unprovable bases and discontinuous
fragment mappings stay accessible as unmapped/historical evidence. Neither
nearby line coordinates nor a matching quote alone is sufficient attribution.

Shared source-adjacent widgets and provider readings render comment text as
content without modifying Markdown. Original diff and exact revision links remain
available. Expanded or pinned individual threads select inspectable context;
collapsed threads do not auto-disclose every PR comment. UI and context retain
identical source/comment identities, lineage and changed/historical state. This
selection is not a delivery receipt or permission to execute instructions.

GitHub state is separate from local review state, investigations and executed
verification. Refresh replaces observed replies/edits/resolution only. This
provider issues read operations only; it has no reply/resolve/publish action.
Existing outbound GitHub features are separate explicit user operations.

## Verification and limits

Focused Rust tests cover pagination/replies/edit/resolution fields, cursor loops,
auth/offline refusal, exact multiline generated mapping, changed/deleted/unknown
source refusal, rename mapping, and LEFT/RIGHT/PR-level envelope roundtrips.
Browser fixtures drive the real product panel/widgets/context with deterministic
API responses: selective expansion, original links, external text safety,
reply/resolution refresh, source edits and offline historical captures. These
fixtures are not a claim to modify or resolve a live PR.

Current attachment intentionally uses conservative whole-file equality, so some
unchanged spans in an otherwise edited file remain unmapped. No source is fetched
from a moving branch. Captures are session-local, not a new persistent GitHub
cache. Each connection is bounded to 20 pages; incompleteness is explicit.
Local source bytes are compared locally, not uploaded to GitHub. Existing `gh`
network/credential availability governs remote reads. No human usability study
is claimed. Exact executed results are recorded in the issue handoff.

LLM review: existing GitHub credential integration, shared provider/context model
and literate mapping are reused. Ambiguous historical bases are exposed rather
than labeled with the current base tip. Comments remain external evidence.

Executed focused checks: three Rust pagination/mapping/envelope tests passed.
Chromium, Firefox and WebKit passed the serial real-product browser fixture flow,
including safe external text, selective context, reply/resolution refresh,
source-edit invalidation and offline history. The browser build passed TypeScript.
Connections additionally cap normalized JSON at 8 MiB and selected evidence at
16 MiB, exposing incomplete coverage rather than unbounded disclosure.
The actual temporary-Git rename/missing-history test passed. Two existing local
review behavior tests and the generic external-context contract test passed.
No outbound GitHub action was executed in these checks.
