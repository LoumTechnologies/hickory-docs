# Hickory Docs

A downloadable **note-taking IDE** whose notes are `.hick` documents:
reproducible, verifiable, executable documents in the hick language, meeting
transcripts and their AI summaries ingested as ordinary notes, and an AI agent
whose output is literate-programming files in git. Notes can run, and an AI
summary in one can be proven to still describe what it summarized.

**Start here (2026-09-05).** The core is small and is meant to stay so:
`hick-lang` parses text plus namespaced tags (and runs in the editor as
WebAssembly, so there is one parser); `hick-blocks` is the element
registry, where a tag is declared once — name, attributes, how it renders
to `{kind, span, …props}`, which children the walk visits, which actions
it answers; `hick-literate` registers the elements over a run's facts; the
server has one route for every element's actions and one that lists the
vocabulary; and `apps/web/src/elements` is the mirror, one folder per
kind. Everything else is an extension over that seam or is parked. The
plan, what is built and what is not, and the bins every crate falls into
are `docs/specs/freeform/the-minimal-core.md`; the rest of this file is
the record of how the product got here, kept because each paragraph is a
decision, and it is long because there were many. A file-length ratchet
(`scripts/check-file-length.sh`) now stops any source file passing a
thousand lines, and the ones already past it may only shrink.

Read three documents before changing anything:
`docs/specs/freeform/notes-ide.md` for what the product is **for** (notes are
documents; meetings are inputs; the phone reads and captures but never
executes), then
`docs/specs/freeform/local-only.md` for what the product **is** (a program you
download; no server, no account, no relay, nothing to buy; `hick up` and a
desktop app over one engine — still true, and widened on purpose only by
`notes-ide.md`), then
`docs/specs/freeform/architecture.md` for how it is **built** — accurate on the
language, crates, execution boundary, and verification, and superseded on
everything hosted.

The surface syntax is settled separately in
`docs/specs/freeform/bare-documents.md`: the `<hick:doc>` wrapper is optional, a
document may begin with markdown or YAML frontmatter, and every document weaves
a `.md` of its own name.

An open question, investigated and deliberately not built, is
`docs/specs/freeform/two-branches-in-one-document.md`: whether feature flags
could replace branching in a `.hick` document. The short answer is that the
branch→feature-set half is worth doing and the branches-as-flags half would
cost the provenance that this product exists for.

**File → New Project is a recipe commit, not a document (2026-09-03,
`lenses.md` step 3, `a-new-project-is-a-recipe-commit.md`).** The
scaffolder runs into a scratch directory and what it wrote is committed **as
one act** through a **temporary index** (`scaffold_commit.rs`: `read-tree
HEAD`, add only the scaffold's files, `write-tree`, `commit-tree`,
`update-ref`), so the person's staged and unstaged work is neither swept in
nor touched, and there is no uncommitted scaffold for an edit to fuse into.
The trailers are `Hick-Recipe` (the command, run from the root, `-o
<folder>`), `Hick-Image`, and `Hick-Output: <tree hash> <folder>` — a **git
tree hash**, so the history lens checks with one `rev-parse` whether the
commit's tree is exactly the scaffolder's and draws one of three states:
*matches its recorded output · upgradeable*, *edited before it was
committed · not upgradeable*, and the replay evidence (still *unrecorded*).
An occupied target folder and the repository root are refused; no
repository is `NotARepository`, a **type** answered as `missing:
"repository"`. A scaffold that fails commits nothing and leaves no folder.
The paragraph below is what it replaced and is kept for `hick ingest --from
'#cell'`, which stays.

**Amended the same day, from dogfooding.** Two of that paragraph's answers
were wrong, and each was wrong for a reason worth keeping. (1) **A project
is made anywhere on this machine**, not in a subfolder of the open one: the
dialog has a **Location** field, and the repository that records the recipe
is **whichever one contains that location** (`resolve_target`), with `-o`
still spelled from *that* repository's root because that is where a replay
runs it. The dialog names the repository while you type, and says when it is
not the open one. (2) **No repository is a checkbox now** —
`init_repository` on the create request (`ensure_repository`), which makes the
folder as well, is ticked by the preview before you press anything, and means
*see to it that there is one* rather than *make one* — so a location already
inside a repository creates nothing. Saying it that way is what keeps it from
ever nesting a repository, and it also removes a race the debounced preview
would otherwise have. The "sentence, never a button" line held for the
.NET SDK and was never right for git: git is already here, it is one command
in one folder, and *you have already chosen the folder by asking for a
project in it*. The SDK stays a sentence and a link. It was briefly a button
on a screen after the refusal, which was the same mistake one step smaller:
**a decision about what pressing the button does belongs beside the button**,
derived correctly and yours to untick, never a stop sign in the middle of an
act. The same reasoning gives the second checkbox — **open the project, and
in a new window or this one**. Windows are the *shell's*, not the server's
(a session is a process here: a new window is a second process, this window
is `remember` + `restart`), so `LocalState::set_shell` is the seam the
desktop app fills after `prepare` and `hick up` leaves empty — and the same
seam carries the **ellipsis beside Location**, which opens the platform's own
folder chooser (`POST /api/pick-folder`, the same `blocking_pick_folder` File
→ Open Folder uses, on a blocking thread because a native modal on the main
thread deadlocks the app). A cancel is `{"path": null}` and a `200`; the
catalogue publishes `can_pick_folder` so a browser tab draws no ellipsis
rather than a button that always fails. The page has no `@tauri-apps/api` and
no `invoke` at all, which is why every one of these is a route. The open
happens **only after the commit**, and **never after a failure** — a restart
that took away the terminal explaining why nothing was committed would
delete the only useful thing on the screen. And (3), the one that generalises: **`dotnet new` runs in a terminal**
(`a-command-the-app-runs-is-watched-in-a-terminal.md`) — `POST /api/scaffold`
answers `202` with a `hick_term` session the way `POST /api/tests/run` does,
a watcher commits **the moment it exits zero** and `inject`s the verdict into
that session's own scrollback, and `GET /api/scaffold/result` is how the
*app* finds out (the person already knows: they watched it). It had been a
subprocess whose stdout was discarded, and the failure that exposed it
reached a person as the two words **"Unprocessable Entity"** — the HTTP
status line — because the web client also read only `{"error": …}` from a
failed response and fell back to `res.statusText` for everything else,
throwing away axum's own `text/plain` rejection that named the exact missing
field. Both halves are fixed; `apiError` now reads the JSON shape, then the
body's own text, and only then the status line.

How a document owns files a scaffolder (`dotnet new`) wrote is
`docs/specs/freeform/owning-what-a-scaffolder-wrote.md` (**superseded for
scaffolds** by the paragraph above): **`hick ingest` reads
the exec's output volume** and writes the scaffold into the document as ordinary
`hick:file` bytes carrying the run's fingerprint, so your four lines are
ordinary edits and there is no anchor grammar at all. Nesting is
`exec > ingested > file` — never `exec > file`, because `file > exec` already
means "run this, paste the output here". The bytes need their own origin: they
must not read as `Literal` (they are not yours) or as `Exec` (that is synthetic
and kills the reverse edit). A re-run is a **three-way merge** with the recorded
hash as the base, coarse for a principled reason — two runs of a scaffolder
share no history, so no byte-precise thread exists to record. It supersedes
`docs/specs/freeform/scaffolded-files-and-derived-edits.md` on the mechanism,
which is still where the two refusals are argued: **CRDT edits** cannot express
a derivation from a foreign artifact, and **line-offset patches** are the
fifty-year-old version of the same mistake. What killed that document's `from=`
is worth remembering generally — **it built a durable claim on a gitignored
artifact** (`.hick-cache/`), so the base did not survive a clone. **Built
(2026-08-23):** `hick ingest --from '#cell' <doc>.hick`, the `<hick:ingested>`
element, the gitignore filter, the non-UTF-8 refusal, and the ingested origin —
plus one rule the spec did not name: **a volume a document has ingested is no
longer flushed as a pipeline output** (it arrives in
`PipelineResult::ingested_volume_files` instead), or the next `hick run`
overwrites your four lines. **Re-ingest was built and then retired (2026-09-03, `lenses.md`)** — a second ingest into a cell is now refused by name, pointing at the history lens — but while it existed it settled where
the base comes from: the document holds *ours* and records the run's hash, but
a hash verifies rather than reconstructs — so **the base is this document at
the commit that introduced that fingerprint**, recovered from git. Volatile
regions are deliberately not built until the merge has produced enough false
conflicts to show what they look like; note `volatile` is already a reserved
frontmatter key. **File → New Project is built (2026-09-01)**: a door onto that
same verb rather than a second mechanism — it writes the document, then calls
the same `ingest_from_exec`. Its templates and its form fields are **the
scaffolder's own**, read from `dotnet new list` and `dotnet new <t> --help`, so
a template from a package this product never heard of gets a form too. The
structured source is a trap: the template engine's `templatecache.json` holds
every symbol and default but names them by **symbol**, and `--ExcludeLaunchSettings`
is rejected — the CLI spelling exists only in the help text. Parsing that
hard-wrapped text has exactly one rule, and it is in the bytes: **a line broken
at a space keeps the space, a line broken mid-token does not**, so concatenating
is the whole join. Two things cost real bugs and are worth remembering: the
footer's indented example command (`   dotnet new winformslib -h --language VB`)
sits in the continued-name column and turned `--nullable` into `--language`, and
`dotnet new <unknown> --help` **exits zero** — the `Usage:` line, not the exit
code, is what says a template exists. The command written carries **only what
was changed from the default**, `--no-restore` starts **on** (a restore fills
`obj/` inside the volume the ingest reads, and not generating it beats relying
on a `.gitignore` being right), and the preview is `POST /api/scaffold/preview`
— the same renderer, over the wire, so no second implementation can show a
document different from the one written. A run that fails **still leaves the
document**; no SDK is a `NoDotnetSdk` **type** answered as `missing: "dotnet"`,
and unlike a debug adapter it is **a sentence and a link, never a button** —
this product has no catalogue for a platform install.

How one engineer's several machines see each other's IDE sessions — remote
view and remote control, one identity, no relay and no account — is
`docs/specs/freeform/one-engineer-many-machines.md`. Editing one file across several of them —
and across branches and worktrees — is `docs/specs/freeform/the-merged-view.md`:
one tab synthesizes a file from N sources with `<hick:when>`-shaped variants
that **exist on disk nowhere**, so **the view is the merge, held live** (a
shared region is agreed by construction and cannot conflict later). It is a
**lens, not a document** — no save path, no `.hick` extension — and it closes
`two-branches-in-one-document.md` without adopting branches-as-flags. A
committed variant names a **property, never a machine**: *the distinction lives
in the document, which machine matches lives on the machine.* The invariant that
keeps it honest — **CI weaves with no facts at all, so the shared reading must
stand alone and a variant may only add** — comes from the superseded
`machine-scoped-edits.md`, which is still the place the reasoning is written
down. A machine bought to run an agent on is
`docs/specs/freeform/the-broker-and-the-sealed-machine.md`: a sealed machine
holds no real key and has one road out through a broker the engineer runs,
which allows, denies, asks, or substitutes the real credential. Say "one road
out, with a toll booth" — **never** "airgapped", which a machine that talks to
a model is not. The open question under all three is
`docs/specs/freeform/changes-not-commits.md`: the reverse edit is already
jujutsu's move-a-change-where-it-belongs on the tangle axis, a session should
record which commit its writes *landed* in, and **emission is one-way** — a
session may produce a commit, but nothing may re-produce one that exists, which
is the only form of generated history that survives blame.

Whether a document that emits commits replaces the repository is
`docs/specs/freeform/expression-and-log.md`, and it does not: **a document is an
expression, a repository is the log of its values**, so a document that emits
history needs git *more*, each emitted commit carries the document version that
emitted it, and **the document describes the present while git holds the past**
— it never accumulates corrections, because the record of what generated an old
commit is inside that commit. Machines and repos are **places** (spanning them
is coordination); commits are a **time** (spanning them is rewriting), allowed
only above the publication floor. Across repositories: **read across, write
local.** **Built (2026-08-23):** the floor itself — computed on every read
(`merge-base(HEAD, @{upstream})`, then `origin/HEAD`, then `origin/master`),
carried on `GET /api/git/log`, and marked per row in the git pane. Nothing
emits anything.

Provenance *across* versions of a document is
`docs/specs/freeform/provenance-across-versions.md`. The data is not missing,
the identity is: `(commit, path, line)` addresses published bytes exactly (and
only below the publication floor — re-emission churns hashes above it), replay
recomputes exact lineage at any commit, and neither can **correlate** two
versions. The mechanism is a **recorded correspondence** — refactor mode proves the
outputs unchanged, which makes them a join key, and a merge is the richest
recording site of all because base, ours and theirs are in hand. **There are no
element ids**: an unwatched edit is guessed at by an agent, confirmed by a
person, and recorded as an assertion, or shrugged at. Register `hick-merge` as a
git merge driver rather than writing a team rule — and check it is configured,
since an undefined driver silently falls back to git's line merge. The
correspondence journal is a **record**, not a cache, so it may be committed, and
that choice is the only thing deciding whether CI can check anything here.
Continuity is a **fourth provenance family, off by default**, and the whole of
it — ribbon, journal, and the pre-commit repair — rides that one switch.
**Built (2026-08-23):** replay (`hick lineage --at`/`--history`, the time
slider, the grammar boundary stated in those words) and the merge driver with
its configured-driver check at project open and in `hick test` — never in the
pre-commit hook, since `hick init` installs the hook. Continuity itself stays
**also built:** the correspondence journal (`.hick-journal/`, gitignored by
`hick init` — delete that line to let CI check it), refactor mode as a
byte-precise recording site (the re-ingest was a diff-precise one until it was retired 2026-09-03), and the
pre-commit repair. All of it rides one switch, per-user in
`hickory-workspace`, **off by default**: no continuity, no journal, no check.
Journal entries above the floor are **provisional** — the concession that part
of a record is derived, which is the cost of recording at the moment both
sides are in hand. Nothing draws continuity yet.

A machine bought to run an agent on is sealed and reaches the network through
`hick broker` — a CONNECT proxy with a per-host policy and a log
(`hickory-broker`, built 2026-08-23; `allow`/`deny` only, no TLS termination,
no keys, no CA). What a re-emission would produce is `hick emit`
(`crates/hickory-cli/src/emission.rs`, read-only): one stage, one commit,
refusing to rewrite anything below the floor.

How a session is refined is `docs/specs/freeform/sessions-you-run-again.md`.
A **re-run is not a reenactment**: it is a second real session that stands
without the first, which is a draft the gitignore already discards. Three kinds
of session must never be mistaken for each other — **run** (harness-written,
evidence), **edited** (a declared layer over a frozen base, marked *in the
bytes*, not merely in the rendering), and **staged** (authored, never executed,
carried by the existing never-run marking). **Built (2026-08-23):** rewind and
re-run named apart in the dock, and `hick ingest --from carry` — which settled the carry's
home by inventing nothing: it is an ordinary `.hick` document, written outside
the gitignored `sessions/`. Equivalence between two attempts is
an **instrument, never a gate** — the person is the judge, and their
understanding is allowed to move. The constraint that does the work is
legibility, not provenance: **a session must be followable by a reader who has
only the repository**, which is also the real argument for the interview form —
a prescient opening prompt concentrates everything you learned into one
unexplained monolith, while a dialogue lets each requirement arrive attached to
the question that provoked it.

How a sentence someone posts proves itself — meeting → analysis → message,
with ribbons across documents — is walked through in
`docs/specs/freeform/receipts-for-a-message.md`, including what is still
clunky. The three kinds of provenance that answer "why is this here?" —
lineage (the weave), context (what the model was shown, from the session),
declared (`cites=`) — and why they must never look alike, are
`docs/specs/freeform/three-provenances.md`. Sessions are the user's own
record and are not checked in; `hick init` ignores `sessions/`. A session
file is the conversation: one file per dock conversation, each turn naming
its parent, drawn in the app by the same cards as the chat
(`docs/guarantees/agent/a-session-is-the-conversation.md`); the model's
reasoning, when a provider streams it, is kept apart and folded
(`docs/guarantees/agent/reasoning-is-shown-apart-from-the-answer.md`). A Claude Code transcript becomes a session document with
`hick ingest --from claude-code` — what maps to what, what is dropped and counted,
and how the no-escaping invariant is honoured are
`docs/specs/freeform/claude-code-sessions.md`. How outside material enters a notes folder is `docs/specs/freeform/ingest.md`;
how the result is marked is `docs/specs/freeform/provenance-and-standing.md`.
The rule that governs both: **provenance is derived and checkable, standing is
declared and unverifiable, and the two must never render alike.** Say
"AI-touched" or "no evidence of AI" — **never** "human-written" or
"human-verified", which nothing can prove.

A proposal, not built, is `docs/specs/freeform/local-history.md`: git's
resolution is a commit, but in a `.hick` folder **you are not the only
writer** — runs, weaves, reverse edits, ingests, the agent, find-and-replace
and the merge driver all write between two commits, and six of those have no
way back. **The unit is the act, not the file**, because almost every writer
here is a batch writer. It lives in `hickory-workspace` beside drafts, under
the user's own data directory, and the constraint that keeps it safe is
learned from `from=`: **nothing may cite it** — it is a cache, not a record,
it never crosses git or the peer channel, and it is not a fifth provenance.
Say "local history", **never** "version history", "backup", or "snapshots".

Debugging a compiled language is
`docs/specs/freeform/launching-what-a-document-builds.md`, investigated and
not built. The finding is not about C#: **`hick-dap` assumes the generated
file IS the program**, which is true of Python, Node and Go and false of
everything compiled — `Program.cs` is not a program, `bin/Debug/…/app.dll`
is. The missing concept is **a build between the weave and the launch**, which
is exactly VS Code's `preLaunchTask`, so the shape is copied rather than
invented. Two things do not carry over: **there is no `launch.json` and there
must not be** (the whole editor-intelligence family is *found without being
configured*), and the program is woven into a **scratch directory**, so a
document with no project file has nothing to build and must be refused
clearly rather than have one written for it. The open question is where the
build command comes from — a built-in per-language recipe is shippable, and
**deriving it from the document's own cell that already builds the thing** is
right. **Never claim a language is debuggable before it is**: adding
netcoredbg to discovery alone would offer C#, pick `Program.cs`, and fail at
launch. A build is watched in a **read-only terminal**, not reported by a
spinner — a failing build says why in its own output, and ANSI colour and
carriage-return rewriting are why an emulator rather than the text card that
exists. **A terminal you can type into is a second input the document does not
have**, so a *watching* cell terminal has no input path at all; that is what separates it
from a `hick-term` **session** terminal, which is a person's own shell. The
transcript is the record and the terminal is the run happening — **nothing is
ever verified against what a terminal showed**. It generalises: watching a
build is just watching a cell.

That rule is stated too narrowly, and
`docs/specs/freeform/a-terminal-that-writes-the-document.md` says how: **the
rule is not *no typing*, it is *no unrecorded input*** — typing that becomes
the document is recorded by construction. **The characters are the same in a REPL and an editor**, and comint unified
them in 1988 with three things hick already has under other names: a process
mark (**the cell**), RET rebound to submit (the run gesture), and a read-only
history (`a-generated-file-refuses-an-edit`). So the modal rule is one line —
**Enter inserts, a modifier submits** — and a cell's transcript
(`Cmd`/`Out`/`Err`/`Exit`, in order) is already a REPL scrollback, so the same
bytes render **as a script** (what you edit) and **as a session** (what you
read). Two things comint never had: the history is **editable and
re-derivable**, and an output edit **lands back in its document** rather than
being walled off. The line Emacs never crossed holds here too: **line-oriented
interaction unifies with editing, screen-oriented does not** — `M-x shell` is a
buffer, `M-x term` is not, and nobody runs `top` in comint — which is the same
boundary the pgrp test already detects, so **one detector serves twice**. One
terminal component, three bindings: **nothing** (ephemeral, writes nothing), **a container in a
document** (persistent — the typing becomes the cell), **a running cell**
(watching, no input path). The anchor needs no new element, because **a
persistent terminal is a container**: several `hick:exec` blocks naming one
container is already how the language says "commands that share state", and
the DAG already chains them. Say "**anchored to a container**", never
"attached to a document". It writes **one cell that grows**, not one per line
(`commands: Vec<String>` is already plural), so **the unit of re-run is the
cell** and splitting a session is what a sentence of prose is for. The
sharpest edges both have one answer — **the anchor suspends**: the terminal
keeps working, the document stops receiving, and the invariant holds that **an
anchored cell is always a prefix of the session that reproduces**. **Suspend,
never filter** — a cell with a line cut from the middle claims a run it cannot
reproduce, while a prefix is true. A secret typed or **pasted** in suspends
recording, and the scan **gates the write, never undoes one**, because the
document syncs and autosaves; say "**a line that looks like a secret stops the
recording**" — **never** "your secrets are safe here", which no scanner can
promise. A program that is not the shell suspends it too, and that is **exact
rather than heuristic**: `tcgetpgrp(master) != shell_pgid` means the keystrokes
belong to a child, catching a REPL, `cat` and `read` as well as `vi` — verified
on Linux 2026-08-26, where a full-screen detector alone would have recorded
`python3` then `print(1)` as two shell commands. The alternate-screen sequence
(`ESC [ ? 1049 h`) decides only how the warning **reads**, never whether to
record. **ConPTY has no process groups**, so Windows keeps only the weaker
signal and the rest is undesigned. `hick_literate::transcript::TranscriptBuilder`
is **deleted** (2026-08-26): it emitted one `hick:exec` per command, which is
the cell-per-line model that spec argues against, and could only write a whole
document from scratch. Where the typed line comes from is **measured**: a
`DEBUG` trap reports executed commands not typed lines, `PROMPT_COMMAND` +
`history 1` leaks a line from `~/.bash_history` at the first prompt, and
**`PS0` is the one that works** — expanded once per submitted line, in a
**subshell** (so state lives in Rust), with `HISTCONTROL` able to keep a line
out of history and make `history 1` re-report the previous one, which the
history *number* detects and which is a **suspension**. A program that is not
the shell is excluded **by construction** (its PS0 never fires), so `tcgetpgrp`
explains rather than gates. **Both shells are verified (2026-08-27)** and they
disagree: zsh's `$HISTCMD` in `preexec` is the slot a line *would* take, so a
line hidden by `HIST_IGNORE_SPACE` advances it and gives it back and the NEXT
command reuses the number — the opposite of bash, where the hidden line is
never reported and the next one arrives stale. So **a leading space means "do
not record" as a hick rule**, applied in Rust for every shell, rather than
inferred from whichever shell this is; zsh's history options are deliberately
not detected, because re-implementing another program's rules in a hook is how
a hook drifts.
**Built (2026-08-27):** the persistent binding — `POST
/api/terminals/{id}/anchor` binds a terminal to a **container** (never "to a
document"), typing grows one cell, and the suspension rules hold: a secret
suspends, a line the shell kept out of history suspends, suspension is
**sticky**, and resuming starts a **new cell**. A shell with no hook is
**refused by name** rather than left looking anchored — the sharpest
consequence of "never anchor silently", and only obvious once the mechanism
turned out to need the shell's cooperation. Not built: the *message* for a
foreground program (the gate is unreachable anyway — a REPL's PS0 never
fires), and any scanning of **output**.

Settled: **netcoredbg** (Samsung, MIT) is the adapter (`--interpreter=vscode`);
Microsoft's `vsdbg` is licensed to its own editors and is unavailable to this
product. **Built (2026-08-26):** `hick lsp install csharp` fetches csharp-ls
(MIT) with `DOTNET_CLI_HOME`/`NUGET_PACKAGES` redirected into the prefix and
telemetry opted out, plus C# and XML highlighting. **Built (2026-08-27):** the
whole C# path — a `Build` step between the weave and the launch (the table
version; it is scaffolding, and the signal to stop extending it is a second
compiled language needing a fourth field), `hick dap install csharp` via a new
**archive installer shape** (per-platform URL + pinned SHA-256; a pin, never a
signature), and a real debug session verified end to end. One correction that
cost nothing to find and would have cost a lot to ship: **`lang_detect` had no
`cs` row**, so the C# language server had never been reachable at all. Another:
a breakpoint has **three** states, not two — netcoredbg reports `pending` at
set time and binds on module load, so `verified: false` means "not yet", never
"never", and the only certain refusal is the one hick makes itself. **The
default sandbox cannot run `dotnet` at all**, on any platform an `hick:exec`
cell might use it from — its crypto/certificate stack needs a system-service
call the sandbox denies by design, and there is no per-document way to grant
it. `HICKORY_EXECUTOR=local` is required on every `hick run`/`hick
ingest`/`hick test` invocation that touches a C# cell.

What the app does when a tool is missing, and what its toolbar is allowed to
say, are `docs/guarantees/debugging/a-missing-debugger-is-a-button.md` and
`docs/guarantees/editor-intelligence/the-toolbar-uses-words-an-ide-user-knows.md`
(both 2026-08-28). **A fixable failure is a button, never a command to go and
type**: a missing debug adapter offers to install itself through the same
catalogue and confinement `hick dap install` uses, and the decision is made by
downcasting `hick_dap::MissingAdapter` — a **type**, so a reworded sentence
cannot silently take the button away — and only when `installable` says a
catalogue can serve it. The offer is cleared whenever a session starts, or an
install that worked leaves "No Python debugger on this machine" beside
"finished — exit code 0". Say `POST /api/install` **adds an affordance, never
a second mechanism**. On vocabulary: **never use a word another editor has
already given a different meaning to.** The toolbar's `Verify` is now `Test`,
matching `hick test` so the button and the command find each other, and
`Refactor` — which named a *mode* while meaning rename/extract everywhere else
— is `Pin outputs as a baseline` in an overflow menu, named for what it
produces. A **count must be expandable**: the status bar's problem count opens
a list (F8 keeps the jump), because clicking it used to reach only the focused
document and could do nothing at all.

A generator is written in **the team's language, not the model server's**
(`docs/guarantees/languages/a-generator-is-written-in-the-team-s-language.md`,
2026-08-30). A code model server speaks GraphQL over a pipe and has no opinion
about what is on the other end, so `--client --target <lang>` emits the types
and `--runtime --target <lang>` emits the transport; **python and csharp ship
a runtime, go and typescript do not and say so**. The warehouse demo used to
make a virtue of the generator being Python while the code it modelled was C#
— a true capability and a bad default, because a C# shop maintains C#. It is
now a C# generator beside a **Python** layering check against one C# server,
which demonstrates the same thing honestly. The rewrite is also the best
evidence available that the rules and not the implementation are what matter:
the C# generator emits both files **byte-for-byte identically** to the Python
one. It costs 22 lines (163 → 185) and moves the demo's break-even from eight
endpoints to nine — worth paying, not worth paying unknowingly.

Showing what a generator makes, without storing it, is
`<hick:sample path from to caption>` — a window inside the `hick:exec` that
produces the file, whose lines appear **only in the weave**
(`docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md`,
built 2026-08-28). The document gains one line, the bytes cannot be edited
into a lie because there are none there, and the drift check catches
staleness for free. Three things it needed that were not obvious: it is
excluded from the DAG **twice** (command text and stdin — a cell fed its own
output is the opposite of the point); it reads **this run's** bytes via
`MultiDocumentState::register_produced_file`, because an output volume is not
on disk until after the weave and reading the file made a first run report
that a file it had just written did not exist; and a range past the end of
the file **says how long the file is**, never an empty block. Picking one is
a gesture — select lines in a generated file and `POST /api/samples` finds
the owning document and cell. A volume is declared at document level and
reaches a cell by being **mounted**, so that search runs forward to the mount,
not outward from the declaration.

A volume seeded from a directory carries **what the repository would carry**
(`docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`,
2026-08-28). This is not tidiness: a cell's recording is keyed by the digest
of what it mounts, so `obj/` or a `__pycache__` beside a generator makes every
recording unfindable and the next weave writes `[never run]` over a run that
really happened. The filter is the project's own `.gitignore` — never a list
of build-output names we maintain — and hidden files are kept, because
`.editorconfig` is an input a build reads. The other half no ignore rule can
fix: a cell that mounts its own **unstable** output — the weave target, or a
`hick:file` a cell fills — is warned about. Only those two; the first version
warned on any output and fired on three of five demo documents, every one of
them correct, because *a document writing a script from literal text and
mounting the directory so a cell can run it is the central move of literate
programming*.

Code intelligence wider than one file is
`docs/specs/freeform/an-index-beside-the-language-server.md`, not built.
**SCIP is in addition to LSP, never in place of it**: SCIP is an *index
format* (Apache-2.0) with no completions, no diagnostics, no rename and no
view of an unsaved buffer, while a language server has no view of a project it
has not opened. JetBrains feels the way it does because it has both. This
product wants an index more than most editors do, because its code lives in
**documents** and the files appear only when something weaves them — so the
interesting question ("where else is this used") spans documents and generated
files no server has open. **A reference that cannot be mapped back through
lineage to a document span is not shown**, or a person is sent to edit a file
that regenerates over them. An index is a **cache, never a record**: it may
speed an answer and may never *be* the answer, the live server wins any
disagreement, and staleness is the same input-digest signal recordings already
use — not a second mechanism. Indexers are **spawned** like language servers
(`hick index install`, same sandboxed catalogue as `hick lsp install`); the
`scip` crate would be **linked**, which is where the MIT-only heading and the
copyleft rule used to disagree — **resolved 2026-08-27 by amending the
heading**, because the rule was always the policy. Apache-2.0 is permissive
and allowed.
**Built (2026-08-27):** `hick index install|build|find`. The heading was
amended (**MIT only** → **No copyleft**), so `scip` (Apache-2.0, with MIT
`protobuf` beneath it) is linked and the index is read. The rule that earns
it: a reference in a **generated** file is reported as the line of the
**document** that wrote it, through the same lineage the reverse edit and the
debugger use — and one that cannot be mapped back is **not shown**, with the
number dropped reported rather than swallowed. Every answer says when the
index was built and how many covered files have changed since. Not built: any
use of it inside the app.

**A `hick:file` is a generated file however deeply it is nested
(2026-09-03, `a-file-block-is-a-file-however-deeply-it-is-nested.md`).**
Debugging a scaffolded C# program said *"this document does not generate
app/Program.cs. It generates:"* — and then nothing, about a file in the
document it was reading. Two disagreements between the engine and the view
built on `hick_lsp`, the first hiding the second. (1) **Which blocks are
files:** `hick-literate` has always asked `all_tags`, `build_virtual_files`
asked `find_tags` (top level only), and `hick ingest` writes
`exec > ingested > file` — so every scaffold a document owned was written by
`hick run` and invisible to the language server and the debugger. (2) **What
the bytes are:** a virtual file's `content()` opens with the newline after
its tag, which is what makes its line 0 the tag's line and is exactly right
for a reader that never writes the file down; anything that **writes** it
must drop that line (`written_content`) and shift the map with it
(`PositionMap::without_first_line`), because a leading blank line moves
whatever has to be at byte 0 — a `#!`, a **BOM**, an XML declaration.
`dotnet new` writes a BOM, so MSBuild refused the woven `.csproj` with
`MSB4025` at *line 2, position 1*. The fixture in `live_session.rs` wrote the
files itself and claimed in a comment that it therefore could not drift from
the mapping; it now weaves through `weave_into`, which is the one place that
decides.

A plain file in the repository — `src/main.rs`, `app.py` — has **the same
language server a document has**
(`docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md`,
2026-09-02): `hick-lsp` hands a file that is not a `.hick` document to its
child **as itself**, at its real path, with `PositionMap::identity`, rooted at
the folder the app opened. Until then the app was an editor for documents and
a viewer for the repository around them, and the whole language server sat
one pane over. What it forced: **one language-server session per workspace**
(`one-language-server-per-workspace.md`), because the session used to be per
WebSocket and a document has a socket each — invisible while only staged code
asked, a machine on its knees the moment three plain files each got their
own rust-analyzer. `LspHub` remaps request ids per connection, broadcasts
notifications, reference-counts open files, and `?doc=workspace` is a socket
with no room, carrying only the language channel. Two things worth
remembering: the system test spawns the **binary**, so `cargo test` alone
runs the last build's `hick-lsp` and answers nothing; and two `LspClient`s on
one channel each take the other's replies, which is why plain files share
one client.

**A plain file has the same debugger a document has**
(`docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md`,
2026-09-03): `src/main.rs` or `tools/app.py` opened in its own pane gets the
breakpoint gutter, a Debug button and the debugger's strip, and is debugged
**as itself** — `Mapping::identity`, so line *n* is line *n*; built by
`build_plain` in the **nearest project above the file** (never above the
folder the app opened) into that project's **own** output, asking `cargo
metadata` where the target directory is because a workspace member's is the
workspace's; run **in place**, with the project directory as its working
directory and no scratch copy, since a plain file has nothing woven to
protect. Until then the app could debug a document's generated Python and
could not put a breakpoint in the repository it was open on. Two things it
forced: the **workspace socket now carries the debug channel**, shared by
every plain-file pane through one `DebugClient`, so the server names the
file on `started`, `build` and a failed `start` and each pane keeps only its
own events (`eventIsOurs`); and a frame in *another* file of the folder is
reported **root-relative with its own line** (`source_line`), so the pane can
open it as a tab rather than call it external. The isolation guarantee's
boundary now says a plain file runs in place: the pane still cannot write
the file while stepping, but the program does whatever the program does.

**Format on save** (`save-can-format-first.md`, 2026-09-02) is a per-user
setting, off by default, persisted in `ui.json` beside the window title;
Shift+Alt+F formats at any time. The edits come from **the file's own
formatter through its language server** — rustfmt via rust-analyzer — and a
`hick:file` block is formatted in the block's own indentation:
`translate_edits` maps the ranges and puts the indentation back after every
inserted newline, bare only for a trailing newline at column 0. rustfmt
answers with the **smallest edits that get there**, not a whole-file
replacement, so a test that greps one edit's text proves nothing; apply them.
**Several cursors** (`several-cursors-edit-at-once.md`) needed three parts
at once — allow many, draw them, rectangular selection — and the document
editor had been getting **no language-server completions at all**
(`completions({ project })` alone); it does now.

**The Git pane does the daily loop** (`the-git-pane-does-the-daily-loop.md`,
2026-09-02): stage, unstage, discard (two clicks), diff, commit, amend, push,
pull, branch switch and create, stash and pop — each **one git command run as
itself**, with git's own words shown when it refuses. It had been read-only on
the argument that a *half*-built git UI teaches a workflow it cannot finish;
the answer was to finish the loop, not to keep the refusal. Three lines it
does not cross: **pull is `--ff-only`**, nothing is ever forced, and **amend
is refused below the publication floor** — the same line `hick emit` draws.
`GIT_TERMINAL_PROMPT=0` and a batch-mode ssh mean a push that needs a
credential **fails saying so** instead of hanging a request on a password
nobody can type.

**Rust debugging is proven end to end (2026-09-02)**: `hick dap install rust`
fetches **codelldb** (MIT, bundles its own lldb, DAP over stdio) through the
archive shape — a `.vsix` is a zip with no top-level directory, so an asset
now says what it unpacks `into` — and the `Build` table gained a `cargo
build` row **without a fourth field**: where the artifact lands
(`artifact_root`, the app's own `.hick-cache/cargo-target` so a session's
build outlives the scratch copy) and what it is called (`artifact_stem`,
read from `[package] name`) are both answered from the project file the table
already has. The spec's stop signal has not fired. cargo's `.d` notes share
the binary's stem and are not programs.

**A test runs from the line it is written on**
(`docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md`,
2026-09-02): a run mark beside every `#[test]`, `it("…")`, `def test_…`,
`[Fact]` and `func TestX`, found **by shape, textually** — a language server
has no opinion about what is a test, and a runner's own discovery would mean
running it to draw a gutter. A click runs that one test in **the ecosystem's
own runner, in the nearest directory that owns the file** (the closest
`Cargo.toml`, `package.json`, `pyproject.toml`, `.csproj`, `go.mod`), as a
**terminal session named after the test** — a terminal, not a summary, for
the reason a build is watched: a failing test says why in its own words.
Found without being configured; there is no `launch.json` and there must not
be. In jsdom a gutter's `domEventHandlers` cannot resolve a line from a
pointer's height, so **the mark answers its own click**.

**Dogfooded on this repository (2026-09-02)** with the app opened on
hickory-docs itself, and three things came out of it. **Switching tabs
emptied a 159-line Rust file on disk**: the draft keeper's unmount flush ran
after the view was destroyed, read `""`, and the next mount restored and
saved it — fixed by never reading the buffer through the view alone
(`unsaved-work-survives-closing-the-app.md`). **The up-loop fights git in a
checkout whose transcript cache is gitignored**: opening the repo re-wove
fifteen committed outputs to `[never run]`, and a `git checkout` of one of
them was seen as an external edit, refused, and re-woven over — so the Git
pane's Discard "worked" and the file stayed modified; this is a design
question about committed woven outputs versus gitignored transcripts, not a
pane bug. **The command bar ranks a typed full path below a fuzzier hit**
(`crates/hick-lsp/src/lang_detect.rs` opened `default.json`), and the LSP
hover tooltip shows raw markdown fences and overflows the window.

**The round trip must never undo work or leave two things disagreeing**, and
2026-09-02's dogfooding found three ways it did. Each is now a rule with a
guarantee. (1) **A document's own unstable products are not in its cells'
cache key** (`a-recording-is-keyed-by-the-cells-inputs.md`, amended): a cell
mounting `.` keyed its recording on its own weave, so no recording was ever
found again and the next weave wrote `[never run]` over recorded output; the
old warning telling authors to mount narrower is deleted, since the hazard is
gone. (2) **A weave never writes `[never run]` over a rendering that exists**
(`a-weave-without-a-recording-keeps-the-artifact.md`, amended): the weave
target is no longer exempt as "the weave's own report". (3) **An output that
cannot be carried back is held, never restored**
(`an-output-that-cannot-be-carried-back-is-held.md`): the loop used to put
the file back a second after git or a person wrote it; now the bytes stay,
the tree and pane say *held* and why, and the hold lifts when the document
catches up or on an explicit *Regenerate from document*. And **transcripts
are committed** (`a-recording-a-document-keeps-lives-in-the-document.md`): `hick init`
writes `.hick-cache/*` and `!.hick-cache/transcripts/` — a bare
`.hick-cache/` cannot be re-included under, which is why the older line is
widened rather than kept. A recording stored under an older key formula is
simply not found; one `hick run` re-records it. A fourth, found while
writing the round-trip test: **an output volume flushes only what the run
changed** (`an-output-volume-flushes-only-what-the-run-changed.md`) — a
volume seeded from `.` with `output="."` used to flush every seeded file
back, the document included, and a placeholder staged before the run for a
file a cell fills then overwrote the cell's real product, with the run
reporting nothing failed.

**The viewer is a block editor, and a document is one thing it views**
(`docs/specs/freeform/lenses.md`, adopted 2026-09-03; **all six steps built
the same day**: the merged-view guarantee names the rule; the history lens
— `Read as a story` on the Git pane, `GET /api/git/commit?sha=` for a
card's diff and *edited since*, `recipe` on a log row with
`output_matches` checked by git; New Project as a recipe commit; **replay**
(`recipe.rs`: the command runs in a **detached worktree**, never the
working tree, and S2 is a sibling rebased onto above the floor or a child
merged below it, with `Hick-Replay-Of`/`Hick-Replay-Same` as **evidence**);
the **tail** (`POST /api/git/recipe`, `hick emit` with a place to type it);
and **reword/move/drop** as `git rebase -i` with `GIT_SEQUENCE_EDITOR=cp
<todo>` (`story.rs`), drafts only. A join that stops is `409` with git's
words and the repository is left where git left it. The re-ingest merge is
retired: a second `hick ingest --from '#cell'` is refused by name.) A **lens** is a synthesized document — the same cards — over
something that is not a `.hick` file, and every block declares three things:
what it is a view of, how a change gets home, and whether a change is
allowed right now. The third column is entirely rules already decided
elsewhere (synthetic bytes refuse the reverse edit, a merged-view variant may
only add, the publication floor), asked **per block instead of per tab**.
Four lenses: the plain file, the generated file, the merged view, and the
new **history lens** — commits drawn oldest-first as cards, a recipe-bearing
commit (`Hick-Recipe`/`Hick-Image`/`Hick-Output` trailers) drawn as a cell
with its diff as output, the working tree as the tail. It takes scaffolding
over from `owning-what-a-scaffolder-wrote.md`: a scaffold is an act, so File
→ New Project will **write a commit, not a document**, and **replay makes a
sibling commit, never remakes the old one** — rebase onto it above the floor,
merge below. Say "lens" and "no evidence of drift"; **never** save a lens,
and never say "reproducible" of a commit nobody has replayed. It is Emacs's
buffers with blocks instead of text, a floor instead of trust, provenance
instead of nothing, and no save.

Where all of this is going is `docs/specs/freeform/three-axes.md` (adopted
2026-09-03, nothing built): a document, a cell and a produced file answer
**three questions** — *evidence* (recorded / stale / unrecorded), *ownership*
(does the document own these bytes or point at them), *agreement* (does the
disk hold what the document produces) — and the thirty named modes,
sub-kinds, verbs and banners that exist today are answers to those three.
`adopt`, `promote`, `carry`, `import` become `ingest --from`; refactor mode
becomes a pin on the agreement axis; kept / held / preserved / `409` become
one *diverged* surface with three ways out; and **recordings the document
keeps are ingested into it as `hick:ingested key=…`**, which reverses the
2026-09-02 decision to commit `.hick-cache/transcripts/` — a durable claim
must not point into a cache, which `from=` already taught.

**Three-axes, step 1 built (2026-09-03):** every surface says *recorded /
stale / unrecorded*. A miss is classified by `cache::stale_lookup` — the
newest recording with the same command text — so a cell whose inputs moved
shows its last output marked stale instead of `[never run]`, and the weave
target is written again in that case; `hick test` reports `STALE` as drift
with `hick run` as the fix. The web status value is `unrecorded`.

**Step 2 built:** one *diverged* surface. Held, kept and the plain-file
`409` are one state with one banner (`DivergedBanner`) and three ways out —
keep mine, take theirs, merge — and the server publishes base and theirs so
the merge is a real three-way one; a merge's result goes back through
`POST /api/outputs/resolve` and is then an ordinary save.

**Step 3 built:** one verb. `adopt`, `promote`, `carry` and `import` are
gone; they are `hick ingest --from file|session|carry|claude-code`, beside
the inbox (no `--from`) and `--from '#cell'`
(`one-verb-brings-bytes-into-a-document.md`). `--from recording` is named
and refused until step 4.

**Step 4 built:** a recording a document keeps lives in the document
(`a-recording-a-document-keeps-lives-in-the-document.md`): `hick ingest
--from recording` writes `<hick:ingested key=…>` into the cell behind an
equivalence gate, the weave reads the document first and the cache second,
`hick run` refreshes what a document already keeps, and `.hick-cache/` is
a cache again. Migrating this repository's recordings found four more
things a key must leave out — the mutated volume, an output directory's
contents, a sibling document's kept recordings, a nested cache — all in
`a-recording-is-keyed-by-the-cells-inputs.md`. Two lessons that cost hours:
**the gate pairs cells by position**, because a recording written into an
earlier cell moves every later cell's line, and **the element goes exactly
where the closing tag was**, because a newline added there is command text
and command text is in the key.

**Step 5 built:** one comparison. `hick test`'s drift check and the ingest
gate now go through `compare_outputs`, which `equiv` and the refactor pin
already used, with one report sentence (`describe_difference`) for all of
them (`one-comparison-behind-test-equiv-pin-and-merge.md`). All five steps
of `three-axes.md` are built.

**The editor reads with the parser the server uses (2026-09-05,
`the-editor-reads-with-the-parser-the-server-uses.md`).** The web app
carried a second hick parser — one regular expression — and a differential
run over this repository's documents found it disagreeing with `hick-lang`
on 22 of 29: a rebound prefix gave a document no structure, the guide to
hick showed its examples as live tags, and session files grew phantom blocks
from tool results. `crates/hick-lang-wasm` is `hick-lang` compiled to
WebAssembly (the parser as a library, not a runtime; the rule against a wasm
*container* stands), loaded once at boot; `hick_lang::parse_lenient` and
`structure` are the never-failing parse an editor needs on every keystroke,
reporting the first strict error beside what they drew. The built parser is
a generated file under `just codegen`, and byte-for-byte reproducible.
Spans stay bytes; provenance is byte-precise and nothing here changes that.
**Step 2 the same day (`an-element-is-declared-once.md`):** `hick-blocks`
is the element registry — one `Element<Cx>` per tag declaring name,
attributes, render, descent and actions; `Registry::blocks` is the walk —
and `hick-literate`'s block model is four elements and a prose renderer
registered over the run's facts. The wire shape is unchanged. **Step 3's
routes (`an-action-is-asked-of-the-element.md`):** `GET /api/elements` is
the vocabulary as data and `POST /api/docs/:id/blocks/:at/:action` is one
route for every element's verbs — the element answers with an
`ActionOutcome` (answer / run / edit) and the server carries it out with
the machinery the older routes use; an element never runs anything. **Step
4 (`an-element-is-drawn-by-its-view.md`):** `apps/web/src/elements` is the
mirror — one folder per block kind, a `Record<SlotKind, ElementView>`
keyed by the wire's `kind`, the editor's branch chain replaced by a lookup
and one `SlotContext`; a rendered cell's Run goes through the action route.
The two registries are not yet one list (`math`/`table` are editor-only),
and the Insert menu is still hand-written. The plan is
`docs/specs/freeform/the-minimal-core.md`.

**The tree is a dired, and ingest is a verb of the tree (2026-09-05,
`the-tree-is-a-dired.md`, `a-file-is-ingested-from-the-tree.md`).** Marks
by Ctrl+click or `m`/`u`/`U`; `D`, `R`, `C`, `M`, `+`, `n` and the same
verbs on the row's menu, each one filesystem call on `POST /api/files/op`
with the refusals said plainly and no trash, so a delete asks once. A plain
text file's menu offers *Make literate* and *Ingest into <focused
document>*, both the adoption `hick ingest --from file` performs.

**Every shortcut is a setting (2026-09-05,
`every-shortcut-is-a-setting.md`).** One catalogue in
`apps/web/src/lib/keymap.ts` — menu bar, editor, workspace, dired — with
profiles (Hickory = VS Code keys, VS Code, JetBrains, Visual Studio) and
per-action overrides, persisted in ui.json as `keymap` plus the resolved
`native_accelerators` the desktop shell reads at launch. Consumers ask
`isAction`/`cmKeyOf`, never a literal; chords work in the page, never in
the menu bar; menu changes land at the next launch and Settings says so.

## Stack (settled — do not relitigate)

- CLI + language + local server: Rust (edition 2024), axum, tokio. No database.
- Frontend: React + TypeScript + Vite, shipped as the **desktop and mobile
  apps'** UI via Tauri v2 — one package, three entry points (`index.html` the
  editor, `site.html` the marketing page, mobile its own). No other frontend
  framework. It is not a client for any server we run — the only server it
  talks to is the one in the same process.
- Mobile (`notes-ide.md`): read and capture only. **No executor on iOS** — it
  cannot spawn a subprocess, so no cells, no terminal, no LSP, no DAP. Notes
  render via `hick weave` from cached transcripts. Devices meet through the
  **user's own git remote**, never through anything we run.
- Distribution (`shipping-mobile-and-desktop.md`): **App Store and Play Store**
  for mobile; **`.dmg`, `AppImage`, `.msi`** for desktop, from GitHub releases.
  Two thin Tauri shells over one core — never one shell threaded with
  `#[cfg(mobile)]`. The portable half is the language, documents, weave, and
  transcripts (`hick-lang` and `hick-transcript` verified compiling for
  `aarch64-linux-android`); the host-process half never goes near a phone.
- Live sync: Yrs (Yjs) CRDTs (`hick-grove`, `hickory-collab`). Durable state:
  the user's git repository. The CRDT survives the removal of collaboration
  because the app's editor buffer and the file on disk are still two writers.
- Sharing (`one-engineer-many-machines.md`): a machine is a **keypair**, a
  fleet is a mutual list of public keys, and reachability comes from the
  engineer's own transport — LAN, their overlay, their SSH host — behind the
  same provider seam `--public` already uses. **Never add a relay we operate**;
  `local-only.md`'s four reasons still hold when both endpoints belong to one
  person. Durable state crosses git; only liveness crosses the peer channel.
  **Built:** `hickory-fleet` — identity, the mutual key list, per-verb grants
  (`execute` off by default; a phone can never have it), `hick fleet` — and
  `hickory-peer`, the channel itself: **iroh 1.0, QUIC dialled by public key**,
  whose endpoint identity IS an ed25519 key, so the fleet list is the
  allowlist directly. Requests are gated by a **deny-by-default** path table;
  settings are unreachable over it at all. The spec's 60-second pairing code
  assumed a rendezvous we would have to run, so enrolment is a self-contained
  invitation instead. **On relays (decided 2026-08-24):** the default is
  number0's relays *and* number0's address publishing — neither is a server
  **we** run, so the refusal and the sentence hold, but that sentence now does
  more work, so the product says on every connection whether it was direct or
  relayed and `HICKORY_FLEET_RELAY` takes `direct` or your own relay. Never
  add a relay **we** operate; using somebody else's is a different decision
  and is recorded as one. **`iroh` is pinned to `default-features = false,
  features = ["tls-ring"]`** and must stay that way: the default `portmapper`
  feature pulls the MPL-2.0 `attohttpc` (copyleft, which this workspace
  forbids) and asks the router to open a port without being asked.
- Execution: `Executor` trait; `LocalExecutor` (default) and the Docker
  executor. NEVER reintroduce the wasm container runtime, and NEVER integrate
  third-party CLIs (cram, VHS, etc.) — verification and transcript capture are
  first-party.
- Product shape: **local-only** (`local-only.md`). There is no backend, no
  account, no relay, and no monetization. Do not add one. Anything that would
  need a server we operate is out of scope, not a later phase. Say "nothing
  talks to a server we run" — **never** "your notes never leave your machine",
  which sync to the user's own git remote makes untrue.
- Delivery: a **downloadable product**. `master` only. The release channels
  (`unstable-release.yml`, `stable-release.yml`) and the one-line installer are
  the delivery path; hickorydocs.com is static files and a download link.
  See `.instructions/continuous-delivery-downloadable.md`, as amended by
  `shipping-mobile-and-desktop.md`: mobile publication is App Review's verb, not
  a green CI run, and signing is part of delivery rather than a later polish
  step — an unsigned installer reads to a new user as malware.

## Rules

- `just` is the only task-runner entry point; recipes live at repo root.
- The hick parser's no-escaping invariant is sacred: only namespace-prefixed
  tags are structured; all other text is raw, byte-for-byte. No CDATA, no
  entity escaping. Docs about hick use a different prefix (`h:`) so `hick:`
  examples stay literal — and because rebinding the prefix requires the explicit
  `<h:doc xmlns:h="…">` root, those documents keep their wrapper while ordinary
  notes drop it (`bare-documents.md`).
- Public API of each crate is its explicit `pub use` facade in `lib.rs`.
- Guarantees live in `docs/guarantees/` (one file per guarantee, Given/When/Then
  + verification block); update them in the same change as the implementation.
- Vendored crates keep their names; upstream repos are frozen references —
  fixes happen here, not there.
- cloud-canopy is being modified concurrently by another agent — only
  `hickory-executor-canopy` may know its API; never edit the cloud-canopy repo.
  It stays an **optional** executor pointed at a node the *user* runs; it is
  not a service we operate, and nothing may require it.
- **No copyleft.** A dependency whose licence is GPL or otherwise copyleft
  cannot be linked into this product. Permissive licences — MIT, Apache-2.0,
  BSD, ISC — are fine. The live example: `grit` splits its licence —
  `grit-lib` is MIT and usable, `grit-cli` is GPL-2.0 and is not. Check the
  crate, not the project.

  *Amended 2026-08-27.* This said **MIT only**, which contradicted its own
  rule: the sentence beneath it has always been about copyleft, and Apache-2.0
  is not copyleft. The shorthand had drifted from the thing it was shorthand
  for, and `an-index-beside-the-language-server.md` found the drift by needing
  the Apache-2.0 `scip` crate and being unable to tell which of the two
  sentences was the policy. The rule is the policy; the heading now says so.
- No server, no account, no payment, no telemetry. A change that needs any of
  them is out of scope by decision — see `local-only.md`.

@.instructions/config-and-environments.md
@.instructions/continuous-delivery-downloadable.md
@.instructions/continuous-integration.md
@.instructions/dev-environment.md
@.instructions/documentation-layout.md
@.instructions/framework-agnostic-system-tests.md
@.instructions/github-issues.md
@.instructions/just.md
@.instructions/one-man-team.md
@.instructions/pre-commit-ci-parity.md
@.instructions/pre-launch.md
@.instructions/semble.md
@.instructions/specification-levels.md
@.instructions/third-party-integration-mocking.md
@.instructions/user-facing-errors.md
