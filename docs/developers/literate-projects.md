# Constructing and checking a literate project

Use a declared project when Markdown must construct the complete program without
Git history, generated source, caches or configuration sidecars. This is the
portable construction subset of `docs/specs/freeform/literate-changesets.md`.
Cell execution remains a separate operation; unsupported cells and workspace
contributions produce a named error during pure project generation.

The entry Markdown declares discovery and inventory:

```xml
<h:project sources="library.md assets/schema.md"
           exclude=".git/ .hg/ .hick-cache/ node_modules/ bin/ obj/" />
```

These examples use the documentation prefix `h:`. Actual bare notes use `hick:`;
rebind the namespace explicitly with a document root to use another prefix.
The source closure contains its own discovery and exclusion rules. Paths name
files relative to the declaring Markdown. Existing include elements use their
existing `file` attribute. Every source must be `.md`. Inputs cannot escape the
project; the initial portable spelling supports ASCII normalized relative paths
and refuses Unicode path normalization ambiguities explicitly. File **contents**
support Unicode, BOM, CRLF and final-newline distinctions.

A contributing sibling document can declare `<h:project entry="project.md" />`.
Opening or testing it then selects the root's completed closure rather than its
initial file construction. The root must list that contributor; entry-route
cycles and routes outside the declared closure fail. The route carries no
inventory or source settings of its own.

Inventory is every expected file except the source Markdown closure, declared
woven reading targets, the language-defined root metadata directories `.git/`,
`.hg/`, `.hick-cache/`, `.hick-journal/`, and explicit excluded paths. Exclusions ending in `/` select directory prefixes; other
exclusions select exact paths. There is no extension filter, glob, filename-order
rule or Git-ignore fallback. Include manifests, lockfiles, migrations and assets
in the construction. An unexplained file causes complete-tree verification to
fail. Declare transient build/dependency directories before executing tools.

Construct initial files, then order transformations explicitly:

```xml
<h:file id="app" path="app.ts">
import { greet } from './library';
console.log(greet());
</h:file>

<h:patch id="caller" after="app library">
--- a/app.ts
+++ b/app.ts
@@ -1,2 +1,2 @@
-import { greet } from './library';
-console.log(greet());
+import { welcome } from './library';
+console.log(welcome('Hickory'));
</h:patch>
```

`id` names construction acts across the closure. `after` lists explicit
predecessors, accepting either `app` or `#app`. Every later transformation of a
path must transitively depend on its previous writer. Independent acts may
construct disjoint paths. Duplicate paths, unordered writes, missing predecessors
and cycles fail. IDs are optional for acts nobody references; automatic identities
use the document and opening byte offset, not a persisted element ID.

A patch's local predecessor tree is its captured input plus its declared
ancestors; unrelated acts do not appear merely because their IDs sort first.
The completed New tree combines independent contributions. This also makes
explicit local-state inspection stable when independent acts are renamed.

A patch body is verbatim until its bound namespace's closing patch delimiter.
Tags, comments and Markdown-looking bytes inside it are data. A plain Markdown
`diff` fence is display-only. Headers use `--- a/path`, `+++ b/path`, or
`/dev/null`; hunks use ordinary exact unified-diff offsets/counts and no-newline
markers. No fuzzy application, structural merge or process occurs. Git extended
headers and binary/mode-only patch operations currently fail explicitly.

One patch can create, modify, delete or rename several UTF-8 files in one act.
BOMs and line endings inside payload lines are data; use CRLF payloads when
patching CRLF input. All unchanged bytes retain their origins. Diff-added bytes
map to the patch source, and deleted bytes remain available in local Old state.

For assets and metadata, construct explicit entries:

```xml
<h:entry path="assets/logo.bin" encoding="hex">00ff0142</h:entry>
<h:entry path="assets/current" kind="symlink">logo.bin</h:entry>
<h:file path="scripts/run.sh" executable="true">echo hello
</h:file>
```

Hex bodies allow ASCII whitespace and hold binary bytes entirely in Markdown.
Encoded bytes have no byte-equal reverse edit map. A symlink body is its exact
UTF-8 target. Native materialization currently supports confined relative symlink
spelling on Unix, and refuses unsupported symlink/mode operations on other
platforms. Virtual generation and equality retain that metadata everywhere.

Run the existing commands for a declared entry:

```sh
hick weave project.md       # construct and write the complete result
hick test project.md        # check complete directory inventory
hick project generate project.md --root .
hick project verify project.md --root .
hick project verify project.md --root . --revision HEAD
hick project verify project.md --root . --revision :index
```

The `project` commands generate and compare in memory. Generate prints inventory;
it does not materialize files. Commit and index verification obtain both Markdown
and expected outputs from captured object IDs. Staged documents cannot borrow
unstaged source or outputs. These checks establish source equality, not runtime
correctness or explanatory truth.

`New` is the completed project closure for native LSP and document debug staging.
It includes later changes to every file, even if the displayed patch describes
an incomplete local state. `Old` is the scope input; it is empty for maintained
source and cannot be launched as a program. Explicit Old/local state selection
is available through `hick_lsp::ProjectEndpoint`, not yet through a toolbar
selector. Materializations get independent temporary identities. Endpoint maps
refuse removed or ambiguous bytes; the older single-document protocol omits
foreign-document origins rather than returning an unrelated line.

The portable APIs also provide `ProjectComparison` with N sources and four edit
policies, exact patch regeneration with downstream validation,
`AgentProjectCapture` for verified textual A-to-B reconstruction, and
`IncorporationCandidate` for the separate empty-tree-to-B check and destination
recheck. These are service contracts. Automatic harness capture, AI incorporation,
editable comparison UI and multi-worktree undo are still pending integration.
The Mercurial adapter has a real provider test in `just test-literate-projects-hg`.
Automatic commit-hook selection of project entry checks remains pending; the
root recipe `just verify-literate-project-index project.md <project-directory>`
performs exact staged verification explicitly.

The runnable two-file fixture is
`crates/hick-lang/tests/fixtures/literate-project/`. Its caller patch introduces
an absent function until a later library patch completes the project. It can be
copied into a new Git-free directory containing only those Markdown files.
Its committed `.ts` outputs are the expected tree, regenerated with
`just regenerate-literate-project-fixtures`. The EFCore fixture beside it keeps
its complete program, final migration and snapshot in the Markdown closure;
`just test-literate-projects-csharp` executes and debugs that final tree separately
from pure equality checks.

Repository checks are `just test-literate-projects`,
`just test-literate-projects-web`, `just test-literate-projects-regressions` and
`just lint-literate-projects`. Evidence and current boundaries live under
`docs/guarantees/literate-projects/`.
