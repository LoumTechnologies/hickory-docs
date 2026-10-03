# A reading is a view over durable code

An engineer can read and edit an ordinary Git repository through Hickory without
asking teammates to adopt literate files. Source-backed views are part of the
editor's representation layer. They use the existing parser, file/copy/paste
assembly, provenance, ACP bridge, and editor intelligence. An extension can
supply another organizer; it does not own the persistence mechanism.

The repository's source files remain authoritative. A view has its own identity,
source text, exact output bytes, provenance, and revision token. Creating or
rearranging it verifies byte-for-byte reconstruction without writing a document
or running cells. ACP's organization tool accepts ranges of the original UTF-8
bytes and prose; Hickory supplies the code slices. Reading order can differ from
assembly order. Saving code edits compares the backing files with the recorded
bytes before writing. Rearrangement and prose stay personal.

A persistent literate document is another backing for the same representation.
Its edits update the document and its live room; the existing weave loop owns
publishing its outputs. The comparison editor never becomes a second output
publisher.

Views are disposable by default. **Keep view locally** writes a `.md` reading and
its reconstruction metadata to the user's workspace storage, outside the
repository. Kept readings reopen after restart. An external source edit is
carried through provenance into the reading, without changing repository bytes.
An ambiguous correspondence is refused and offers rebuilding the reading.
Explanations are marked stale after code changes; reconstruction verifies code
bytes, never the truth of prose. ACP session records for views also live outside
the checkout.

## Comparison in the main editor

A comparison supplies a fixed baseline and an optional historical target. The
current side is the ordinary editable buffer. Removed baseline lines are
expandable decorations; they are never part of the text saved, parsed, or sent
to a language server. Added lines update as the current buffer changes. A person
can restore removed lines explicitly or fold unchanged regions.

Comparisons accept commits, branches, tags, and `INDEX`. A historical target is
read-only. Source-backed historical code is projected into the current reading
arrangement, using exact correspondence; its explanations belong to the current
reading and are not evidence from that commit. Persistent documents compare
their actual historical document bytes. Revision reads never check out anything.

Current code keeps language-server requests and debug positions through exact
UTF-8 provenance mapped to editor/LSP UTF-16 coordinates. Prose, computed bytes,
and ambiguous shared origins have no guessed location. Debugging starts only
from saved code. Historical execution belongs in an isolated worktree, where
all the repository's tests and debugger tools can inspect that exact checkout.

## Visual bisect

Git owns the search and its Good/Bad/Skip verdicts. A private control worktree
uses `git bisect --no-checkout`. Every candidate gets a fresh detached inspection
worktree, with source-backed reading templates carried across. The original
checkout, index, and branch stay in place. Candidate windows never change commit
when the search advances, so retained buffers, language servers, debuggers, and
weave state cannot write into another candidate.

The Git pane shows the candidate, tested verdicts, commit graph, remaining
candidates, and Git's outcome. A changed candidate or stale verdict is refused.
**Keep experiment as patch and restore candidate** records a binary patch and
opens a fresh clean inspection of the same commit; the edited inspection stays
intact. Untracked experiments must first be moved out. Skips can leave an
explicitly ambiguous result. Sessions survive restart in personal storage.

Ending a search removes its control worktree and retains inspection worktrees,
so an open window or experiment is never deleted. They can be removed later
with `git worktree remove` after their windows are closed.

## Boundaries

Source-backed readings currently accept UTF-8 text, at most 32 files of 10 MiB
each, and literal file/copy/cut/paste elements. The settled language's reserved
markers still apply; byte reconstruction refuses code that cannot be represented
exactly. They do not execute cells, change file ownership, stage edits, or write
Git configuration. Multi-file publication rolls back completed writes on an IO
failure; it is not a crash-atomic filesystem transaction. Git histories whose
changes cross unrelated fragments may need a fresh reading for alignment.

Language/debug adapters remain the app's existing discoverable tools, with their
existing language and build limits. ACP generation uses a chosen installed agent;
protocol tests do not establish the quality of a model's explanations.
