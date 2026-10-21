# Unsaved Work Survives Closing The App

Given a buffer that differs from its last acknowledged file write, the tab and the same
file in the Files pane carry an asterisk. Closing its tab offers **Save**, **Discard**,
or **Cancel**. Closing the window offers **Save all**, **Close without saving**,
or **Cancel**. Discard and Close without saving remove unsaved changes and their
recovery drafts, including unnamed documents. Explicitly discarded work does not
return on the next launch, regardless of the recovery setting.

Recovery protects against unexpected interruption. Each edited unnamed document
has its own draft outside the project; previously saved files have recovery only
when **Settings → Editing → Retain unsaved changes** is on (off by default).
When recovered work is opened again, the buffer holds it, still unsaved. Writing
a recovery draft never clears the asterisk; an acknowledged write to the actual
file does. Save names an unnamed document; recovery never creates a project file.

The untouched startup introduction is clean: no asterisk, recovery draft, or save
prompt. Returning its text to its original baseline clears its changes. File → New
starts blank. Restoring drafts replaces the startup fallback instead of adding an
extra introduction alongside recovered notes.

Four rules make it true rather than mostly true:

1. **A draft is written while you type, not at shutdown.** A draft written
   only on the way out is a draft that is not there after the one event most
   likely to lose work. A slow timer writes the buffer whenever it has moved,
   and `pagehide` flushes whatever the timer has not reached yet.
2. **The buffer is read through a callback, never a prop.** An editor pane
   deliberately does not re-render per keystroke — CodeMirror owns the text —
   so there is no prop that changes when the buffer does, and inventing one
   would put a React render on the typing path to power a background save.
3. **Every draft records the bytes it was taken from.** That base is the
   common ancestor of the buffer and whatever the file says now, and it is
   what makes the reopen safe rather than merely possible.
4. **Nothing is written inside the project.** The store lives under the user's
   own data directory, keyed by the project's canonical path, so git cannot
   reach it by construction. A `.gitignore` entry would be a promise this tool
   cannot keep on somebody else's machine — a `git add -A` in a folder where
   `hick init` never ran, a fresh clone, a rewritten ignore file — and what
   would leak is unfinished work its author has not decided to keep.

## What happens when the file moved on

Three outcomes, and only one of them interrupts anybody:

- **The file is as we left it.** The text goes back into the buffer, still
  unsaved, silently. This is the common case by a wide margin, and a dialog
  here would train people to dismiss dialogs.
- **The file already says the same thing.** Somebody saved that text from
  somewhere else. The draft is stale and is discarded — and this is checked
  *first*, so a draft matching the file is never treated as a conflict however
  far the file has moved from the base.
- **The file moved on and so did the buffer.** A merge, and it is worth
  someone's attention.

The merge is diff3, over the recorded base, which means most of it is not a
question: a region only one side changed is that side's edit and is taken
without asking. What is left is shown as a conflict. Auto-resolved regions are
shown too, labelled with who they came from — a merge tool that silently takes
a side is a merge tool nobody trusts twice — and the summary leads with how
much of it is actually the reader's problem, because "11 merged, 2 need you"
is the difference between a merge someone answers and one they abandon.

Accepting with conflicts still unanswered is allowed, and the button says what
it will do. An unanswered conflict already reads as the reader's own text —
which is what the buffer holds at that moment — so accepting is a real choice
("everything I did not answer stays mine"), and blocking it would trap someone
in a dialog over a region they do not care about.

The same merge is offered from the save-conflict banner, where the two
existing answers each throw away one side's work.

## Boundary

**Nothing here is required for the app to run.** A store that cannot be opened
— no data directory, a read-only home — degrades to "the window forgets", says
so on the route, and never blocks a start. `HICKORY_STATE_DIR` names the
directory for a portable install.

This covers `.md` document buffers. Each unnamed buffer has a recovery
key per tab within its workspace/window rather than a disk path. A fresh File → New stays
blank; launch restoration brings back retained work. A document room persists live bytes to the actual document; an acknowledgment
of those exact bytes advances the saved baseline. Recovery drafts stay separate. Plain files retain their existing short debounced-save
window and draft machinery.

A draft is keyed by path. Renaming a file outside the app orphans its draft
rather than following it — the draft is still there, under the old path, and
will be offered if a file reappears at that path.

Two-way merge exists for the case with no ancestor at all (a buffer that never
had a file behind it, where a file now exists). It is honest about being
worse: every differing run is a conflict, because without a base there is no
way to tell an addition on one side from a deletion on the other, and a tool
that guessed would silently throw away work.


> **Amended 2026-09-02.** Found by dogfooding on this repository: switching
> away from a plain file's tab **emptied the file on disk**. The draft
> keeper's final flush runs during unmount, after the effect that destroys
> the editor view (React runs cleanups in declaration order), and reading the
> buffer through the dead view gave `""` — recorded as a draft of nothing,
> restored "silently" on the next mount because the disk still matched the
> base, and then autosaved. The pane now keeps the buffer's last text in a
> ref and reads that when the view is gone; an absent view is not an empty
> buffer. `PlainFilePane.test.tsx` ("closing the pane") fails on the old code.

---

Last LLM verification:

- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified
- Evidence: `useUnsavedLifecycle.ts` discards recovery before approving an
  explicit tab/window close without saving. `useUntitledRecovery.ts` suppresses
  subsequent flushes for discarded buffers and waits for in-flight draft writes
  before deleting their drafts. Save failures and Cancel leave the tab open.
- Test coverage: `App.test.tsx` exercises discard and reopen with and without
  a folder, window discard, pagehide recovery, and repeated restoration without
  duplicating the introduction. `useUnsavedLifecycle.test.tsx` checks close
  choices, named-file discard with recovery on/off, and failed Save.
  `drafts.lifecycle.test.tsx` checks in-flight write ordering and suppression of
  later flushes. `just test-web` passed all 2,021 tests.
- Caveats: native window-close handshake and filesystem recovery are reviewed,
  not driven by these frontend tests. Crash recovery retains the last periodic
  write, so edits after that write may be lost on abrupt termination.

> **Amended 2026-10-08:** the coherence implementation supersedes the former
> explicit-Save checkpoint for document dirty state. Automatic writes to the
> actual file now count as saved; recovery drafts never do. See
> `saved-means-the-displayed-bytes-reached-the-file.md`. The earlier verification
> account above is historical and does not validate this amended contract.

## Source-bound tab close and resumed recovery — 2026-10-20

A tab-close decision belongs to its exact tab lifetime and live source. Ordinary
Save/Discard/Cancel choices are agent-accessible under that source revision; they
are not human attestations. A changed source or identical-ID replacement before
the decision resumes refuses saving/discarding. Save refusal leaves the tab open.
Newer pending edits after save/discard/recovery cleanup also keep the tab open;
completed earlier effects are not rolled back or described as untouched. Layout
updates check the final tab identity again instead of closing a replacement.

Spatial close actions await that actual result, including cancellation, with a
separate operation subject so reading remains addressable while a prompt waits.
An owning AbortSignal cancels its exact prompt, including cleanup of listeners;
it cannot dismiss a replacement. Cancellation after a save/discard starts leaves
the tab open and reports possible effects rather than rollback.

A generated discard checks expected text before and after waiting for earlier
saves. Newer edits stay pending. Recovery discard suppression applies to the
observed bytes, rather than every future edit in that mounted writer. Named and
unnamed recovery writes/deletes are ordered; edits after discard or undo-to-base
cleanup queue behind that cleanup. Unnamed suppression also binds tab identity.
Old-path save cleanup cannot suppress the replacement path's recovery. A changed
unnamed source during deletion is retained before metadata is forgotten.

Retrospective correction: waiting for a generated save used to clear newer
`latest` text. Recovery writers also suppressed later work for an entire mounted
lifetime, and some deletes were not chained with earlier/later writes. Those
paths could undermine the newer-edit refusal during tab close. This continuation
corrects that interaction; it does not erase the earlier historical reviews.

LLM implementation review: partially verified. Inspected `useUnsavedLifecycle`,
`SpatialShellView`, PromptPanel, `viewBuffers`, PlainFilePane, RepresentationPane,
generated `shell/views`, outputSave, drafts and useUntitledRecovery. Production
buffer owners supply live revision/dirty getters; third-party owners without them
retain conservative fallback behavior and are not certified for exact races.
Whole-window closure's separate save/discard policy and OS shutdown/response-loss
paths remain unverified and are not represented as completed #72 parity.

Executed focused checks: five existing and four new lifecycle cases; five prompt
cases; one spatial pending-close/read/cancel case; two generated saver race cases;
two existing generated remount/discard cases; four named recovery-writer cases;
and two unnamed recovery-writer cases. Paused fixture replies distinguish actual
old cleanup effects from later retained input. Checks include same-ID/same-source
replacement, duplicate close, source changes during decision/cleanup, cancellation
after save start, abort-owned prompt replacement and serialized old-write/delete/
new-write ordering. Targeted TypeScript included the containing WorkspaceView;
whitespace/file-length checks passed. Broad regressions and post-commit monitoring
were not run. UI/API callbacks are fixtures; no native/provider/release/deployed
witness or human acceptance is claimed.

Evidence dependencies (SHA-256; changed inputs invalidate this checkpoint):
- `apps/web/src/views/useUnsavedLifecycle.ts`: `df6bedc5b85ef5f1cdaf35d97ec38a288ffbe24456b0a31a518d9be67dc08939`
- `apps/web/src/shell/SpatialShellView.tsx`: `cf8ba7f420ef3fe9a9e8043375bc03f49c0a6121c32c27ec5eb53ba42de4c740`
- `apps/web/src/components/PromptPanel.tsx`: `317c2924c369f672d0f77fa9c447fd183a88ec305ce43e7b0fdca146e4a918de`
- `apps/web/src/lib/viewBuffers.ts`: `5c9aa309e8d994f51ee0ba6b84fa7fa92eaed310a3f96b532457473fdfc75c39`
- `apps/web/src/components/PlainFilePane.tsx`: `fda5a8e2ec87ca2f4019e612494f05d360c80c248582c64bbb2affb703052336`
- `apps/web/src/views/RepresentationPane.tsx`: `f2921c1c26e154bc52df0f38ff96ecc6da48780337ea16b2f8b08f3885ada0ff`
- `apps/web/src/shell/views.tsx`: `555147f13e5f0c6944ea92bd8c18cdae54939c669c1187bf9f31404134ef5267`
- `apps/web/src/lib/outputSave.ts`: `8f758f5833ecb52926857dfd701e3b3be9488916b256c3c127f422be91196604`
- `apps/web/src/lib/drafts.ts`: `0b7b8f235b2ab1d6598053e38e4c08003102d9986f71919d683d504d648226ff`
- `apps/web/src/views/useUntitledRecovery.ts`: `c5aaf657a3c801a6c52c7fc238936330d7512425224ab29f955a1db9b9b13641`
