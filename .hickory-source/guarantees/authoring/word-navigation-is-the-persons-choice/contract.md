<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Word Navigation Is The Person's Choice

For: Authors editing executable documents and the files those documents produce.

Given any editor in the app — a `.md` document, a plain file, a woven
output — when Ctrl+← or Ctrl+→ is pressed (Option+← / Option+→ on macOS), then
the caret moves by whichever definition of "word" the person picked in
Settings → Editing, and Shift extends the selection by the same unit:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/word-navigation-is-the-persons-choice.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-a5a0660fd481"></a>
## `apps/web/src/editor/wordMotion.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Word Navigation Is The Person's Choice](contract.md).

<a id="f-a5a0660fd481-LEFT-RIGHT"></a>
### `[LEFT, RIGHT]`

@vitest-environment jsdom
Ctrl+←/→ under both definitions of "word". The commands themselves are
CodeMirror's; what is ours — and what these assert — is that the SETTING
picks between them, that it is consulted at keypress rather than captured
at mount, and that the chords are the two `standardKeymap` already owns, so
the override actually shadows the default instead of sitting beside it.

<storycode:copy id="f-a5a0660fd481-LEFT-RIGHT">
// @vitest-environment jsdom
//
// Ctrl+←/→ under both definitions of "word". The commands themselves are
// CodeMirror's; what is ours — and what these assert — is that the SETTING
// picks between them, that it is consulted at keypress rather than captured
// at mount, and that the chords are the two `standardKeymap` already owns, so
// the override actually shadows the default instead of sitting beside it.

import { afterEach, describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { standardKeymap } from "@codemirror/commands";

import { wordMotionBindings } from "./wordMotion";
import { saveWordMotion, WORD_MOTION_KEY } from "../lib/wordMotion";

const [LEFT, RIGHT] = wordMotionBindings;

</storycode:copy>
<a id="f-a5a0660fd481-views"></a>
### `views`

<storycode:copy id="f-a5a0660fd481-views">
let views: EditorView[] = [];
</storycode:copy>
<a id="f-a5a0660fd481-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a5a0660fd481-expression_statement">
afterEach(() => {
  for (const v of views.splice(0)) v.destroy();
  localStorage.removeItem(WORD_MOTION_KEY);
});

</storycode:copy>
<a id="f-a5a0660fd481-editor"></a>
### `editor`

A view with the caret at `at`, tracked for teardown.

<storycode:copy id="f-a5a0660fd481-editor">
/** A view with the caret at `at`, tracked for teardown. */
function editor(doc: string, at: number) {
  const view = new EditorView({
    parent: document.body,
    state: EditorState.create({ doc, selection: { anchor: at } }),
  });
  views.push(view);
  return view;
}

</storycode:copy>
<a id="f-a5a0660fd481-press"></a>
### `press`

<storycode:copy id="f-a5a0660fd481-press">
/** Where the caret ends up after one press. */
const press = (view: EditorView, binding: typeof LEFT, shift = false): number => {
  (shift ? binding.shift! : binding.run!)(view);
  return view.state.selection.main.head;
};

</storycode:copy>
<a id="f-a5a0660fd481-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a5a0660fd481-expression_statement-2">
describe("what Ctrl+arrow counts as a word", () => {
  it("crosses a whole identifier under the default", () => {
    saveWordMotion("word");
    const view = editor("XMLHttpRequest done", 0);
    expect(press(view, RIGHT)).toBe("XMLHttpRequest".length);
  });

  it("stops on the case humps under subwords", () => {
    saveWordMotion("subword");
    const view = editor("XMLHttpRequest done", 0);
    // XML | Http | Request — the run of capitals belongs to the word that
    // follows it, which is what makes this readable on an acronym.
    expect(press(view, RIGHT)).toBe("XML".length);
    expect(press(view, RIGHT)).toBe("XMLHttp".length);
    expect(press(view, RIGHT)).toBe("XMLHttpRequest".length);
  });

  it("stops on PascalCase and snake_case going both ways", () => {
    saveWordMotion("subword");
    const forward = editor("ParseResult parse_result", 0);
    expect(forward.state.doc.sliceString(0, press(forward, RIGHT))).toBe("Parse");

    const back = editor("ParseResult", "ParseResult".length);
    expect(press(back, LEFT)).toBe("Parse".length);

    const snake = editor("parse_result", 0);
    expect(press(snake, RIGHT)).toBe("parse".length);
  });

  it("extends the selection by the same unit when Shift is held", () => {
    saveWordMotion("subword");
    const view = editor("ParseResult", 0);
    press(view, RIGHT, true);
    const { from, to } = view.state.selection.main;
    expect(view.state.doc.sliceString(from, to)).toBe("Parse");
  });

  // The whole reason the choice is read inside the command rather than baked
  // into the extension list: a person changes this BECAUSE the arrow key just
  // did the wrong thing, and presses it again to check — in the editor that
  // is already open.
  it("follows a change of setting without rebuilding the editor", () => {
    saveWordMotion("word");
    const view = editor("ParseResult", 0);
    expect(press(view, RIGHT)).toBe("ParseResult".length);

    view.dispatch({ selection: { anchor: 0 } });
    saveWordMotion("subword");
    expect(press(view, RIGHT)).toBe("Parse".length);
  });

  it("binds exactly the chords the default keymap uses for this", () => {
    // If these ever drift apart the override silently stops overriding —
    // both bindings would be live on different keys.
    const defaults = standardKeymap.filter((b) => b.key?.endsWith("-ArrowLeft") || b.key?.endsWith("-ArrowRight"));
    for (const ours of wordMotionBindings) {
      const match = defaults.find((b) => b.key === ours.key);
      expect(match, `standardKeymap still binds ${ours.key}`).toBeTruthy();
      expect(match!.mac).toBe(ours.mac);
    }
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/wordMotion.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/wordMotion.test.ts"><storycode:paste select="#f-a5a0660fd481-LEFT-RIGHT"/><storycode:paste select="#f-a5a0660fd481-views"/><storycode:paste select="#f-a5a0660fd481-expression_statement"/><storycode:paste select="#f-a5a0660fd481-editor"/><storycode:paste select="#f-a5a0660fd481-press"/><storycode:paste select="#f-a5a0660fd481-expression_statement-2"/></storycode:file>

<a id="output-12355592dce2"></a>
## `apps/web/src/editor/wordMotion.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Word Navigation Is The Person's Choice](contract.md).

<a id="f-12355592dce2-byMotion"></a>
### `byMotion`

Ctrl+← and Ctrl+→, under whichever definition of "word" the person chose.
These are the two bindings `standardKeymap` already owns — same keys, same
mac spelling (Alt+arrow there, because Cmd+arrow is line start), same
`preventDefault`. They are copied rather than referred to so that the pair
stays readable as a pair; if CodeMirror ever changes the chord, this file is
where the divergence shows up.
The choice is read AT KEYPRESS, not at mount. Every other display
preference in this app is read when the workspace mounts and takes effect
when it next does — which is fine for tab placement, and would be a
surprise here: a person changes what an arrow key does precisely because
the arrow key just did the wrong thing, and would press it again to check.
One localStorage read per Ctrl+arrow is not a cost worth plumbing a
compartment to avoid.
There is no `cursorSubwordLeft`, only `cursorSubwordBackward`: CodeMirror's
subword motion is logical rather than visual, so in a right-to-left run
these keys move by document order where the whole-word motion moves by
direction. Nothing in this product writes RTL code, and the honest note is
cheaper than a wrapper that pretends otherwise.

<storycode:copy id="f-12355592dce2-byMotion">
// Ctrl+← and Ctrl+→, under whichever definition of "word" the person chose.
//
// These are the two bindings `standardKeymap` already owns — same keys, same
// mac spelling (Alt+arrow there, because Cmd+arrow is line start), same
// `preventDefault`. They are copied rather than referred to so that the pair
// stays readable as a pair; if CodeMirror ever changes the chord, this file is
// where the divergence shows up.
//
// The choice is read AT KEYPRESS, not at mount. Every other display
// preference in this app is read when the workspace mounts and takes effect
// when it next does — which is fine for tab placement, and would be a
// surprise here: a person changes what an arrow key does precisely because
// the arrow key just did the wrong thing, and would press it again to check.
// One localStorage read per Ctrl+arrow is not a cost worth plumbing a
// compartment to avoid.
//
// There is no `cursorSubwordLeft`, only `cursorSubwordBackward`: CodeMirror's
// subword motion is logical rather than visual, so in a right-to-left run
// these keys move by document order where the whole-word motion moves by
// direction. Nothing in this product writes RTL code, and the honest note is
// cheaper than a wrapper that pretends otherwise.

import { keymap } from "@codemirror/view";
import type { Command, KeyBinding } from "@codemirror/view";
import {
  cursorGroupLeft,
  cursorGroupRight,
  cursorSubwordBackward,
  cursorSubwordForward,
  selectGroupLeft,
  selectGroupRight,
  selectSubwordBackward,
  selectSubwordForward,
} from "@codemirror/commands";
import type { Extension } from "@codemirror/state";

import { loadWordMotion } from "../lib/wordMotion";

/** The same gesture under both definitions, decided when it happens. */
const byMotion =
  (word: Command, subword: Command): Command =>
  (view) =>
    (loadWordMotion() === "subword" ? subword : word)(view);

</storycode:copy>
<a id="f-12355592dce2-wordMotionBindings"></a>
### `wordMotionBindings`

The two bindings, for anyone assembling a keymap by hand (the tests, and
the editors below).

<storycode:copy id="f-12355592dce2-wordMotionBindings">
/**
 * The two bindings, for anyone assembling a keymap by hand (the tests, and
 * the editors below).
 */
export const wordMotionBindings: readonly KeyBinding[] = [
  {
    key: "Mod-ArrowLeft",
    mac: "Alt-ArrowLeft",
    run: byMotion(cursorGroupLeft, cursorSubwordBackward),
    shift: byMotion(selectGroupLeft, selectSubwordBackward),
    preventDefault: true,
  },
  {
    key: "Mod-ArrowRight",
    mac: "Alt-ArrowRight",
    run: byMotion(cursorGroupRight, cursorSubwordForward),
    shift: byMotion(selectGroupRight, selectSubwordForward),
    preventDefault: true,
  },
];

</storycode:copy>
<a id="f-12355592dce2-wordMotionKeymap"></a>
### `wordMotionKeymap`

The extension every editor loads.
Must come BEFORE `defaultKeymap` in the array: bindings for one key run in
registration order and the first to return true wins, so an override that
arrives second never runs at all.

<storycode:copy id="f-12355592dce2-wordMotionKeymap">
/**
 * The extension every editor loads.
 *
 * Must come BEFORE `defaultKeymap` in the array: bindings for one key run in
 * registration order and the first to return true wins, so an override that
 * arrives second never runs at all.
 */
export function wordMotionKeymap(): Extension {
  return keymap.of([...wordMotionBindings]);
}
</storycode:copy>

### Assemble `apps/web/src/editor/wordMotion.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/wordMotion.ts"><storycode:paste select="#f-12355592dce2-byMotion"/><storycode:paste select="#f-12355592dce2-wordMotionBindings"/><storycode:paste select="#f-12355592dce2-wordMotionKeymap"/></storycode:file>

<a id="output-cd81eaa11e7e"></a>
## `apps/web/src/lib/wordMotion.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Word Navigation Is The Person's Choice](contract.md).

<a id="f-cd81eaa11e7e-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-cd81eaa11e7e-expression_statement">
import { describe, expect, it } from "vitest";

import { loadWordMotion, saveWordMotion, WORD_MOTION_KEY } from "./wordMotion";

describe("word motion persistence", () => {
  it("defaults to whole words with nothing stored", () => {
    localStorage.removeItem(WORD_MOTION_KEY);
    expect(loadWordMotion()).toBe("word");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveWordMotion("subword");
    expect(localStorage.getItem(WORD_MOTION_KEY)).toBe("subword");
    expect(loadWordMotion()).toBe("subword");
    saveWordMotion("word");
    expect(loadWordMotion()).toBe("word");
  });

  it("falls back to whole words on a value nothing ever wrote", () => {
    localStorage.setItem(WORD_MOTION_KEY, "camelhumps");
    expect(loadWordMotion()).toBe("word");
  });

  it("works without any storage at all", () => {
    expect(loadWordMotion(null)).toBe("word");
    expect(() => saveWordMotion("subword", null)).not.toThrow();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/wordMotion.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/wordMotion.test.ts"><storycode:paste select="#f-cd81eaa11e7e-expression_statement"/></storycode:file>

<a id="output-7812eaa9926e"></a>
## `apps/web/src/lib/wordMotion.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Word Navigation Is The Person's Choice](contract.md).

<a id="f-7812eaa9926e-WordMotion"></a>
### `WordMotion`

What Ctrl+← and Ctrl+→ count as one word.
"Whole words" is what every editor does by default: a run of word
characters, stopping at space and punctuation. "Subwords" also stops inside
an identifier — `PascalCase`, `camelCase`, `snake_case`, `XMLHttpRequest` —
which is the motion you want in code and the wrong one in prose. Since a
`.md` document is both, this cannot be inferred from the file; it is a
preference about the person, so it is one.
An editing preference rather than a presentation one, but it persists the
same way for the same reason: there is no account to hang it on, so it is
localStorage, per browser, with a hard default for anything unset or
unrecognisable. Mirrors lib/tabStyle.ts.

<storycode:copy id="f-7812eaa9926e-WordMotion">
// What Ctrl+← and Ctrl+→ count as one word.
//
// "Whole words" is what every editor does by default: a run of word
// characters, stopping at space and punctuation. "Subwords" also stops inside
// an identifier — `PascalCase`, `camelCase`, `snake_case`, `XMLHttpRequest` —
// which is the motion you want in code and the wrong one in prose. Since a
// `.md` document is both, this cannot be inferred from the file; it is a
// preference about the person, so it is one.
//
// An editing preference rather than a presentation one, but it persists the
// same way for the same reason: there is no account to hang it on, so it is
// localStorage, per browser, with a hard default for anything unset or
// unrecognisable. Mirrors lib/tabStyle.ts.

export type WordMotion = "word" | "subword";

</storycode:copy>
<a id="f-7812eaa9926e-WORD_MOTION_KEY"></a>
### `WORD_MOTION_KEY`

<storycode:copy id="f-7812eaa9926e-WORD_MOTION_KEY">
export const WORD_MOTION_KEY = "hickory.wordMotion";

</storycode:copy>
<a id="f-7812eaa9926e-isMotion"></a>
### `isMotion`

<storycode:copy id="f-7812eaa9926e-isMotion">
const isMotion = (value: unknown): value is WordMotion =>
  value === "word" || value === "subword";

</storycode:copy>
<a id="f-7812eaa9926e-loadWordMotion"></a>
### `loadWordMotion`

The persisted motion, defaulting to "word" on anything unset or invalid.

<storycode:copy id="f-7812eaa9926e-loadWordMotion">
/** The persisted motion, defaulting to "word" on anything unset or invalid. */
export function loadWordMotion(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): WordMotion {
  const stored = storage?.getItem(WORD_MOTION_KEY);
  return isMotion(stored) ? stored : "word";
}

</storycode:copy>
<a id="f-7812eaa9926e-saveWordMotion"></a>
### `saveWordMotion`

<storycode:copy id="f-7812eaa9926e-saveWordMotion">
export function saveWordMotion(
  motion: WordMotion,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): void {
  storage?.setItem(WORD_MOTION_KEY, motion);
}
</storycode:copy>

### Assemble `apps/web/src/lib/wordMotion.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/wordMotion.ts"><storycode:paste select="#f-7812eaa9926e-WordMotion"/><storycode:paste select="#f-7812eaa9926e-WORD_MOTION_KEY"/><storycode:paste select="#f-7812eaa9926e-isMotion"/><storycode:paste select="#f-7812eaa9926e-loadWordMotion"/><storycode:paste select="#f-7812eaa9926e-saveWordMotion"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/OutputEditorPane.tsx](../../../mechanisms/apps/web/src/components/OutputEditorPane.tsx.md#output-2571b0682164)
- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/wordMotion.ts](contract.md#output-12355592dce2)
- [apps/web/src/lib/channelWidth.ts](../../../mechanisms/apps/web/src/lib/channelWidth.ts.md#output-1bd88b546d8a)
- [apps/web/src/lib/ribbonStyle.ts](../../../mechanisms/apps/web/src/lib/ribbonStyle.ts.md#output-5fe1be43bb6f)
- [apps/web/src/lib/tabStyle.ts](../../../mechanisms/apps/web/src/lib/tabStyle.ts.md#output-0732de414ed2)
- [apps/web/src/lib/wordMotion.ts](contract.md#output-7812eaa9926e)
- [apps/web/src/views/SettingsView.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-38894484eb8f)

### Tests and fixtures

- [apps/web/src/editor/wordMotion.test.ts](contract.md#output-a5a0660fd481)
- [apps/web/src/lib/wordMotion.test.ts](contract.md#output-cd81eaa11e7e)
- [apps/web/src/views/SettingsView.test.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-5bfac230ae5d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
