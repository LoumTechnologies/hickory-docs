# The workspace tree: files, work, and windows in their places

*Status: design of record, adopted 2026-09-14 and corrected by dogfooding
2026-09-15. Visible terminal nodes placed by cwd, a literal Files editor
buffer with filesystem name/indent edits, GitHub issues, GitHub
PRs, and GitHub unread projection are built. Window discovery, Jira, and GitLab
are not built.*

**Audience:** contributors extending the Files pane with something that belongs
to a folder but is not a file.

The Files pane answers too small a question. A directory contains files, but a
person working in it also has terminals, another Hickory Docs window, tickets,
issues, and a code review for the branch checked out there. Those things should
not each invent a pane. The tree answers one question: **what is here, and what
is happening here?**

This is the **workspace tree**. “Files” remains a useful short label in the UI
until the contents make the wider meaning obvious.

## The first useful reading

Given this machine:

```text
notes/
  meetings/
    launch.hick
  terminal: summarize meetings                    working
  JIRA-142: Correct the launch summary             2 unread
feature-retry/                                     worktree
  src/
    retry.rs
  PR #318: Add retry policy                        checks failing
    checks/
      unit                                          passed
      integration                                   failed
        log
    reviews/
      Ada                                            approved
      Lin                                             changes requested
    current-head conflicts/
      PR #321: Replace transport                    conflict in retry.rs
window: Release notes
```

clicking a file opens the file, clicking the terminal opens its session,
clicking the other window focuses that native window, and expanding a remote
object fetches only the children needed to draw it.

The tree is not a copy of any of those sources. It is a lens over them.

## One node contract

Every visible row has the same structural facts even though its source differs:

```text
key          stable identity including source/provider
kind         folder | file | terminal | window | work-item | review | field |
             comment | check | log
parent       the visible node it belongs beneath
label        the text a person recognizes
state        optional provider-neutral presentation state
unread       unread events rooted at this node
expand       absent, already loaded, or a lazy request
capabilities
             the verbs this exact node can answer
source       enough information to route a verb home, never a credential
freshness    live, cached-at <time>, refreshing, unavailable, or stale
```

`key` is not the label. A renamed Jira summary keeps the same key. A Jira key is
part of provider identity and is not presented as renameable. GitHub and GitLab
repository identity is the normalized host plus repository id when the provider
answers one, not whichever remote spelling happened to be cloned.

Capabilities are data. A file may answer `rename`, a window `rename-title` and
`focus`, a ticket `rename-title`, `edit-body`, and `comment`, and a check log
only `expand` and `copy`. The UI never guesses a verb from `kind` and never
offers a mutation the provider did not advertise.

## Placement is separate from rendering

A source adapter reports an immutable identity and a location. A pure projection
joins those locations to the directories the filesystem listing contains.

- A terminal's location is its live cwd when its shell reports OSC 7, otherwise
  the directory it started in with that limitation in its tooltip.
- A Hickory window's location is the folder its process has locked and opened.
- A manually associated ticket or issue names a root-relative folder.
- A code review is placed beneath the clone or worktree whose normalized remote
  is its repository and whose checked-out branch is its current head branch.
- An object outside the open roots is not shown.
- If a listing is truncated, an object climbs to the deepest listed ancestor.

Placement does not call a provider. Rendering does not decide placement. This
is what lets terminal, ticket, and review fixtures exercise the same sharp path
cases without mounting the whole app.

## Expansion is a request, not preloading

Filesystem children are already returned by the file walk. Remote children are
not.

1. The first tree response contains the ticket or review summary and whether it
   can expand.
2. Expanding it asks its adapter for the next level.
3. Expanding comments, checks, or a failed check asks for that collection or
   log only then.
4. The node says when the result was fetched and whether refresh failed. Cached
   content remains readable after a failed refresh and is marked stale.

Check logs must be bounded and paged. Opening a tree must never download every
log from every pull request.

## Associations survive; credentials do not travel

A folder association contains the provider, immutable remote object identity,
and root-relative folder. It belongs to the user's repository so another clone
can reconstruct the same workspace tree. It contains no token, cookie, server
password, or provider response.

Credentials stay in the platform credential store on the machine that makes the
request. A clone without credentials still shows the association and says that
its provider is unavailable; it does not make the whole tree fail.

This is compatible with `local-only.md`: Hickory operates no account or relay.
A provider adapter talks directly to a service the person configured, using
their credential, and only for a feature they enabled.

## Other windows are local presence

A session is a process here, so several windows are several processes. They
need a machine-local registry, not a server:

1. On launch, the desktop process registers a random process identity, opened
   folder, display title, process id, focus endpoint, and liveness timestamp in
   the user's application-data directory.
2. It refreshes liveness and removes its entry on a clean exit.
3. Readers verify process liveness before drawing an entry and prune stale
   registrations.
4. Activating a window node sends `focus` over local IPC. The target process
   asks its own native shell to unminimize, raise, and focus its window.

The focus endpoint is not a remotely reachable API. `hick up` has no native
window and registers none. A title belongs to this registered window/workspace;
the existing single global `ui.json` title cannot represent two differently
named windows and must be replaced without losing the fallback to folder and
focused file.

## Unread state lives on the event; its badge lives where it can be seen

Collapsing a folder must not hide a new Jira comment. Moving the badge into a
copy of the unread state would make expansion lose or duplicate it. Instead:

1. Each provider event has a stable event id and read/unread state.
2. Each entity's unread count is derived from those events.
3. The rendering projection places that count on the nearest visible ancestor
   of the entity.
4. When expansion makes the entity visible, the next projection removes the
   ancestor badge and draws it on the entity.
5. Expansion is not acknowledgement. Viewing the relevant comment or choosing
   Mark read is.

Several hidden descendants aggregate on the same ancestor. The tooltip breaks
the total down by source so a count never becomes an unexplained red number.

## The Files pane is text

The first implementation copied dired's commands onto a button tree and put a
small editor over a name after F2. Dogfooding showed that this missed the
requirement. A `.hick` pane does not need a shortcut before its bytes become
editable, and neither does Files.

The filesystem is therefore a CodeMirror buffer. Each line is one entry,
folders end in `/`, and two spaces of significant indentation mean “inside the
folder above.” Ordinary caret movement, selection, multiple cursors, column
editing, history, and folding are editor operations because there are actual
bytes to operate on. Saving changed names or indentation computes filesystem
rename/move operations. A click places point; double-click selects the whole
line and activates the file or non-file object named there. It never leaves the
filename's one word selected as ordinary text double-click would.

Line order carries no filesystem meaning. Reordering unchanged entries is a
local presentation edit, stays clean, and produces no Dry Run or Apply
operation; identity matching uses the unchanged root-relative path and
file/folder kind before interpreting the remaining rows as renames or moves.
This matters for sorting by hand and for column or multi-cursor edits that touch
several names at once.

Unsaved edits reveal a toolbar across the buffer's top. **Dry Run and Apply
must share one plan**: Dry Run lists the ordered, full-path operations without
writing anything, while Apply executes those same operations. Invalid text is
a dry-run refusal, never an empty plan. Destructive removals still require the
second no-trash confirmation after Apply.

The text is still a lens, not a file stored on disk. External filesystem
changes regenerate it when there are no unsaved edits. Unsupported meanings
must be refused explicitly. Inserted filesystem lines create files or
trailing-slash folders. Removed lines stage topmost paths behind the no-trash
confirmation. Create/delete is a separate save from rename/move so the text
does not need invisible identity markers. When live provider/terminal lines
are interspersed, line-count changes are currently refused until those lines
can be tracked through arbitrary edits. Associated GitHub issue titles are
editable through their adapter; other provider fields remain read-only until
their adapter advertises the corresponding write capability.

Remote bodies and comments expand as editable regions of this same conceptual
tree. A Jira key or provider id remains immutable; “rename ticket” means edit
its summary, and a comment is append-only unless its provider says otherwise.

A filesystem rename, native-window title change, Jira mutation, and GitHub
mutation have no shared transaction. A multi-edit across sources therefore
shows a preview, executes one source operation per node, and reports success or
failure per node. It never says “renamed” when only some rows changed and never
rolls back a successful remote mutation by guessing an inverse operation.

Jira issue keys and provider ids are immutable. “Rename ticket” means edit its
summary. A new comment is append-only unless the provider explicitly advertises
editing that particular comment.

## Work-item adapters

The initial work-item adapters are Jira tickets and GitHub issues. Both answer
the node contract; neither leaks its wire response into React components.

The useful common fields are title, body, status, author/assignee, labels,
comments, unread events, URL, and freshness. Provider-only fields remain child
nodes when useful rather than bloating the common contract.

Adding another tracker means implementing discovery/read/expand/mutate and
mapping its errors and capabilities. It must not add a new branch throughout
`FolderTreePane`.

## Code-review adapters

GitHub pull requests and GitLab merge requests are the initial code-review
adapters. The UI may use the provider's own noun in the label while the internal
kind remains `review`.

The summary carries title, number, author, draft/open/closed/merged state,
review decision, check summary, provider mergeability, head/base refs and head
SHA. Expansion exposes comments, individual reviews, checks, bounded logs, and
conflict analysis.

“Mergeable” and “current-head conflict” are different claims:

- Provider mergeability reports whether this review's current head merges into
  its current base according to the provider.
- Cross-review analysis fetches current head refs and performs a local merge
  calculation for relevant open reviews. Its wording is **“these current head
  commits conflict if merged together.”** It does not predict edits nobody has
  made and does not call a developer's future work safe.

The developers shown for another review come from provider authorship and the
current commits' authors, named separately when they differ. A missing ref or
shallow history produces `unavailable`, not “no conflict.” Pairwise work is
bounded to reviews sharing a base and touching overlapping paths before the
more expensive merge calculation.

## Status and sequence

| Slice | State |
|---|---|
| Visible terminal child nodes placed by cwd | built |
| Provider-neutral node/capability client contract | built |
| Machine-local window registry and focus | not built |
| Per-window/workspace titles | not built |
| Folder associations without credentials | built for GitHub issues |
| Jira ticket adapter | not built |
| GitHub issue adapter | built |
| Nearest-visible-ancestor unread projection | built for GitHub notifications |
| GitHub PR adapter | built |
| GitLab MR adapter | not built |
| Lazy checks and bounded GitHub Actions logs | built |
| Current-head cross-review conflict analysis | built for GitHub |
| Literal Files editor; filesystem name and indentation edits | built |
| Text-line filesystem create/reviewed-delete | built without interspersed live nodes |
| Associated GitHub issue title editing | built |
| Provider-backed body/comment editing | not built |

Build in that order except that the generic unread projection belongs in the
node contract before either remote provider. Each slice adds or updates a strict
guarantee and provider fixtures before another provider is added.
