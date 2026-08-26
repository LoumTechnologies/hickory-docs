# A terminal that writes the document

`launching-what-a-document-builds.md` says **a terminal you can type into is a
second input the document does not have**. That rule is right and it is stated
too narrowly. The rule is not *no typing*. It is **no unrecorded input** — and
typing that becomes the document is recorded by construction.

So there is a third thing a terminal can be, and it is the one that makes
writing a cell feel like using a shell: **you type, and what you typed is the
cell.**

## Three bindings, one component

A terminal is not three components. It is one, with a binding, and the binding
decides everything:

| binding | input | what it writes |
|---|---|---|
| **nothing** — *ephemeral* | typed | nothing, ever |
| **a container in a document** — *persistent* | typed | the cell, as you type it |
| **a cell that is running** — *watching* | none | nothing |

Ephemeral is a scratch shell: poke at the machine, close it, nothing follows
you. Watching is the read-only view of a run in progress. Persistent is the
new one, and it is the reason the other two need names.

## One surface, because the characters are the same

Typing `dotnet build app` at a prompt and typing it into a cell body is the
same keystrokes. What differs is not the text, it is what surrounds it: what
Enter means, whether the caret may go backwards, and where the output lands.

**Emacs unified this in 1988 and the mechanism is three things.** A comint
buffer — `M-x shell`, `ielm`, the SLIME REPL — *is* a text buffer, with every
ordinary editing command. It differs from one only by:

1. a **process mark**: text after it is the current input, text before it is
   history;
2. **RET rebound** to submit rather than insert;
3. the region before the mark being **read-only**.

Not a different interface. The same interface, plus a boundary and one
rebound key.

Hick already has all three under other names, which is why this is finishing
something rather than starting it:

| comint | hick |
|---|---|
| the process mark | **the cell** — a `hick:exec` body is the input region |
| RET rebound to submit | the run gesture on a cell |
| read-only history | `docs/guarantees/authoring/a-generated-file-refuses-an-edit.md` |

So the modal rule is one line: **Enter inserts a newline; a modifier
submits.** Every notebook has already taught it, and SLIME's in-buffer
`C-c C-c` is the same gesture from the other tradition. The three bindings
above are then Emacs's three kinds of buffer — `*scratch*`, a file-visiting
buffer, and a read-only one.

**The rendering falls out of the model that exists.** A cell's transcript is
`Cmd`/`Out`/`Err`/`Exit` events in order, which is exactly a REPL scrollback:
a command, its output, the next command. The same bytes therefore have two
honest renderings — **as a script**, which is what you edit, and **as a
session**, which is what you read. Nothing new is stored to get the second.

**Two things this has that comint never did:**

- **The history is editable and re-derivable.** In comint you may edit an old
  input line, but it is inert — resubmitting is all it can do. Here, editing
  the third line of a session changes the derivation: the key changes, and
  what no longer matches can be said. A REPL whose past is live.
- **Editing output means something.** Comint's answer to "do not edit
  history" is read-only everywhere. Hick's is narrower and better, and is
  already specified: an output edit **lands back in the document it came
  from**, byte-exactly, inside the block it came from
  (`docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`). The
  wall stands only where bytes map back to nothing.

### The boundary Emacs never crossed, and neither should this

Emacs did not unify everything. `M-x shell` is a buffer; `M-x term` is a
terminal emulator; nobody runs `top` in comint. Forty years, and the line held:
**line-oriented interaction unifies with editing, screen-oriented interaction
does not.** A buffer cannot be `vim`, because `vim` does not produce lines —
it paints a screen.

That is the same boundary the suspend rule already found from the other side.
`tcgetpgrp(master) != shell_pgid` marks exactly where the buffer model stops
applying, so **one detector serves twice**: it suspends the recording, and it
is the moment the surface must stop pretending to be an editor and be a
terminal. The unification is real up to a line that is *detected* rather than
argued about.

## A persistent terminal is a container

The anchor does not need a new element, and this is the part worth getting
right before any code exists.

A document already models *a sequence of commands that share state*: several
`<hick:exec container="sdk">` blocks naming the same container. The DAG
already treats that state as an edge, and the cache key already folds each
cell's upstream keys in, so cell five cannot be replayed without one through
four — which is exactly what a shell session is.

**So the anchor is the container name.** A terminal bound to `sdk` in this
document is that shell; a terminal bound to nothing is ephemeral. Nothing new
in the grammar, and the session's history is already replayable, already
cacheable, already a readable diff, already `hick lineage`-able. Say
"**anchored to a container**" — never "attached to a document", which does not
say the thing that makes it work.

There is a serializer for this already, and it is dead code:
`hick_literate::transcript::TranscriptBuilder` — "accumulates container
declarations, capability rules, volumes, forks, and executed commands, then
serializes them to valid `.hick` XML" — has **no caller anywhere in the
workspace**. It was built for a REPL that no longer exists. This design is
what would revive it; if this is not built, it should be deleted, because a
`pub` module nothing calls is a claim the product does not honour.

## One cell that grows, not one cell per line

The tempting reading is a cell per command: each gets its own key, its own
recording, its own reverse edit. It is also wrong here, for two reasons.

A shell session of forty exploratory commands would become forty cells, and a
document is something a person reads. And the unit of *re-run* for a shell
session is not a line — you cannot replay `cd build && make` from the middle,
because the state that made it meaningful is gone.

So **a persistent terminal writes one cell, and the cell grows.** Its body is
the lines you typed; its transcript is what came back. `ExecTranscriptEntry`
already carries `commands: Vec<String>` — plural, and always has — so the
model is already there.

Starting a new cell is a deliberate act, and the natural one: stop typing in
the terminal and write a sentence in the document. Prose between two cells is
what a literate document is made of.

The honest cost: **the unit of re-run is the cell**, so a session you want to
re-run in parts is a session you should have split with a sentence. That is
the same trade every notebook makes, and unlike a notebook the split is
visible in the file.

## Suspending the anchor

Both hazards below have the same answer, and finding that they do is what
makes them cheap: **the anchor suspends.** The terminal keeps working — every
keystroke reaches the shell, every byte comes back, nothing about using it
changes — and the document simply stops receiving.

That yields the invariant the whole feature rests on:

> **An anchored cell is always a prefix of the session that reproduces.**

A prefix is honest. A cell with a hole in it is not: if line 3 of 10 is
dropped and 4 through 10 are kept, the document claims a run that no longer
reproduces, because the thing line 3 did is missing. **Skipping a line is
worse than stopping**, and that is the whole argument for suspending rather
than filtering.

Resuming is deliberate, and usually means a new cell — after a suspension the
shell's state contains something the document does not describe, so a cell
that continues would be claiming that state came from the lines above it.

### A secret stops the recording, and does not stop the shell

`export ANTHROPIC_API_KEY=sk-…` typed into an anchored terminal would write
that key into a `.hick` file in somebody's git repository. So input is scanned
for known secret shapes, and a match suspends the anchor.

Three rules make the mechanism honest rather than reassuring:

- **The scan gates the write; it never undoes one.** The document is a live
  CRDT that autosaves and syncs, so anything written may already have been
  persisted and sent. There is no such thing as taking it back. The keystroke
  reaches the shell first and the anchor second, and the second may be
  refused.
- **Paste is the dangerous path, and is the same path.** People type
  passwords rarely and paste keys constantly.
- **Never say secrets cannot be committed.** A scanner is a heuristic:
  high-entropy strings and known prefixes (`sk-`, `ghp_`, `AKIA`, a PEM
  header) are catchable; a short password is not. Say "**a line that looks
  like a secret stops the recording**" — never "your secrets are safe here",
  which is a promise nothing can keep. The same restraint the product already
  practises about "AI-touched" versus "human-written".

False positives cost a suspension the person can see and resume from. False
negatives cost a key in a repository. The asymmetry is why the rule is stop,
not skip — and why a base64 blob tripping it is an acceptable price.

**Output is the same exposure and is not new here.** A command that *prints* a
token has always been recorded by the transcript, in every cell. The same
suspend belongs on the output path; fixing it is a change to how every cell is
recorded, not a thing this feature invented.

### A program that is not the shell stops the recording too

A persistent terminal records what you typed **as shell commands**. The
moment the keystrokes belong to something else, they are not commands and
recording them is a lie.

**This is detectable exactly, not heuristically**, and the test is better than
"is it full-screen": compare the terminal's foreground process group against
the shell's.

```
tcgetpgrp(master) == shell_pgid   →  typing goes to the SHELL   →  record
tcgetpgrp(master) != shell_pgid   →  typing goes to a CHILD     →  suspend
```

Measured on this machine (2026-08-26, Linux) rather than assumed:

| state | `tcgetpgrp(master)` | equal to shell pgid |
|---|---|---|
| at the prompt | 2732361 | **yes** — record |
| during `cat` | 2732362 | no — suspend |
| after `cat` exits | 2732361 | **yes** — record |
| inside a Python REPL | 2732433 | no — suspend |
| after the REPL exits | 2732361 | **yes** — record |

The REPL row is the point. A REPL is not full-screen and draws no alternate
screen, so a full-screen detector would happily record `python3`, then
`print(1)`, then `quit()` as three shell commands — a cell that on re-run
would hand `print(1)` to `sh`. The process-group test catches it, along with
`vi`, `top`, `less`, `read`, and `git rebase -i`, without knowing anything
about any of them.

**The alternate screen is still worth watching, for the wording only.** A
program that emits `ESC [ ? 1049 h` (verified: `less` emits it on entry and
`ESC [ ? 1049 l` on exit) is one a person would call full-screen, and the
message can say so instead of "a program". It decides what the warning reads
like; it never decides whether to record.

**Windows is the caveat.** ConPTY has no process groups, so this exact test is
POSIX-only. The alternate-screen signal survives there and catches the visible
cases; the rest of the answer for Windows is not designed, and pretending
otherwise would be the same mistake as claiming the scanner catches every
secret.

### What the person sees

Both suspensions say the same three things, in the terminal, at the moment
they happen — never in a diff afterwards:

```
⏸ recording paused — this line looks like a secret
   the shell ran it; the document did not record it
   resume recording ⏎ (starts a new cell)
```

```
⏸ recording paused — less is not a shell command
   the terminal is yours; the document resumes when it exits
```

A person who does not know recording stopped will assume the cell holds what
they did, and that assumption is the failure this whole design exists to
prevent.

## Refusals

- **Suspend, never filter.** A cell with a line removed from the middle claims
  a run it cannot reproduce. Stopping leaves a prefix, and a prefix is true.
- **Never undo a write to take something back.** The document syncs and
  autosaves; the scan gates the write or it does nothing worth having.
- **Never claim the scanner is a guarantee.** "A line that looks like a secret
  stops the recording" is true. "Your secrets are safe" is not, and this
  product does not say things it cannot prove.
- **Never write an unrecorded input.** That is the rule the read-only
  watching terminal follows by having no input at all, and the rule the
  persistent one follows by writing down everything it takes.
- **Never anchor silently.** A terminal that is writing into a document says
  which document and which container, always visibly. The difference between
  "this disappears" and "this is being committed" is the most important thing
  on the screen.
- **Never let an anchored terminal be the only record.** The cell is the
  record; the terminal's own scrollback is a view, discarded with the window
  like any other.
- **Never invent a second grammar for a shell session.** If a persistent
  terminal needs something a `hick:exec` cannot say, that is a finding about
  `hick:exec`, not a reason for a `hick:shell`.

## What this unlocks

Writing a cell stops being a thing you do *to* a document and becomes a thing
you do *in* one. The scaffolding case is the example: `dotnet new console`,
`ls out`, `dotnet build` — three commands somebody ran in a terminal before
they ever wrote a document about them. With an anchored terminal, running them
IS writing the document, and `hick ingest` then owns what they produced.

It also removes the last reason to leave the app: an ephemeral terminal for
poking, an anchored one for work that should survive, and a read-only one for
watching a run — with the difference between them stated rather than implied
by which pane you happen to be in.
