# Word Navigation Is The Person's Choice

Given any editor in the app — a `.hick` document, a plain file, a woven
output — when Ctrl+← or Ctrl+→ is pressed (Option+← / Option+→ on macOS), then
the caret moves by whichever definition of "word" the person picked in
Settings → Editing, and Shift extends the selection by the same unit:

- **Whole words** (the default) is the ordinary motion: a run of word
  characters, stopping at space and punctuation.
- **Subwords** also stops inside an identifier — `PascalCase`, `camelCase`,
  `snake_case`. A run of capitals belongs to the word that follows it, so
  `XMLHttpRequest` reads as `XML | Http | Request` rather than as one token or
  as one stop per capital.

The choice cannot be inferred from the file, and that is the whole reason it
is a setting. Subword motion is what you want in code and the wrong thing in
prose, and a `.hick` document **is both** — a paragraph explaining a program,
then the program. Deciding per block would mean the same key does two
different things a few lines apart, which is worse than either.

**It is read when the key is pressed, not when the editor mounts.** Every
other display preference in this app (`lib/tabStyle.ts`, `lib/ribbonStyle.ts`,
`lib/channelWidth.ts`) is read at mount and takes effect when the workspace
next mounts, which is honest for tab placement and would be a surprise here: a
person changes what an arrow key does *because* the arrow key just did the
wrong thing, and their next act is to press it again in the document that is
already open. One `localStorage` read per Ctrl+arrow is cheaper than the
compartment that would avoid it.

## Boundary

**Only the two navigation chords.** `Ctrl+Backspace` and `Ctrl+Delete` still
delete a whole word under both settings. That is a different verb, and
deleting more than you meant to is not symmetric with moving further than you
meant to.

There is no `cursorSubwordLeft`, only `cursorSubwordBackward`: CodeMirror's
subword motion is logical rather than visual, so in a right-to-left run these
keys move by document order where the whole-word motion moves by direction.
Nothing in this product writes RTL code, and the note is cheaper than a
wrapper that pretends otherwise.

The preference is per browser profile, like the rest of them. There is no
account for it to follow, and syncing it would need a server —
`local-only.md`.

The landing page's demo editors are deliberately not wired to it: the
marketing site has no settings page, so the choice would have nowhere to come
from and the default is the only honest answer there.

---

Last LLM verification:
- Date: 2026-08-27
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - The preference: `apps/web/src/lib/wordMotion.ts` — `loadWordMotion` /
    `saveWordMotion` over `hickory.wordMotion`, defaulting to `"word"` for
    unset, unrecognisable, or absent storage. No CodeMirror import, matching
    the rest of `lib/`.
  - The keymap: `apps/web/src/editor/wordMotion.ts` — the two bindings
    `standardKeymap` owns (`Mod-ArrowLeft` / `Mod-ArrowRight`, `mac:
    Alt-Arrow*`), each choosing between CodeMirror's `cursorGroupLeft` /
    `cursorGroupRight` and `cursorSubwordBackward` / `cursorSubwordForward`
    inside the command, plus the matching `select*` pair on `shift`.
  - Loaded by every editor that has a keymap, **first** in the array:
    `editor/DocumentEditor.tsx`, `components/PlainFilePane.tsx`,
    `components/OutputEditorPane.tsx`. Order matters and is not cosmetic —
    `buildKeymap` in `@codemirror/view` pushes commands for one key into an
    array and runs them in registration order until one returns true, so an
    override registered after `defaultKeymap` never runs.
  - The setting: `apps/web/src/views/SettingsView.tsx::EditingSection`, a new
    "Editing" section rather than a row under Appearance — this changes what a
    key *does*, and someone hunting for it after Ctrl+→ overshot is not
    looking under "Appearance".
  - Seen running, in the app on `.dev/project/scaffolding.hick`: from the
    start of `Console.WriteLine("Hello, World!");`, three Ctrl+→ presses land
    at `Console.Write|Line` under Subwords and at `Console.WriteLine|` under
    Whole words — **same editor, not remounted between the two**, which is the
    read-at-keypress claim above.
- Test coverage:
  - `apps/web/src/editor/wordMotion.test.ts` — both definitions over
    `XMLHttpRequest`, `ParseResult` and `parse_result`, backwards as well as
    forwards, Shift extending by the same unit, a setting change taking effect
    in an editor that is not rebuilt, and a drift check that the two chords
    are still the ones `standardKeymap` binds (if they diverge the override
    silently stops overriding).
  - `apps/web/src/lib/wordMotion.test.ts` — default, round trip, an
    unrecognisable stored value, and no storage at all.
  - `apps/web/src/views/SettingsView.test.tsx` — the row renders, defaults to
    Whole words, and writes through the lib.
- Caveat requiring review: the tests invoke the bindings' commands directly
  rather than dispatching a key event, so what they prove is that the setting
  selects the right command — not that the chord reaches it. The chord itself
  was checked by hand in the browser (above) and by the drift assertion, not
  by an automated key dispatch.
