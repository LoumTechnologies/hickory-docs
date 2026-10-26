<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Equation Renders And Shows Its Source

For: Authors editing executable documents and the files those documents produce.

Given LaTeX written in a document this app styles as prose, when the caret is
not in it, then it is typeset where it was written; and when the reader wants
the LaTeX back, there is always a gesture that returns it without retyping
anything.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-equation-renders-and-shows-its-source.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-554c1bc6d72b"></a>
## `apps/web/src/editor/mathRender.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Equation Renders And Shows Its Source](contract.md).

<a id="f-554c1bc6d72b-mount"></a>
### `mount`

<storycode:copy id="f-554c1bc6d72b-mount">
import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { renderedMath } from "./mathRender";
import { parseHickDoc, verbatimRanges } from "./hickDoc";
import { mathSpans } from "../lib/math";
import { cardsOf } from "./cards";
import { actionsFor } from "../lib/railActions";

const mount = (doc: string, cursor: number) =>
  new EditorView({
    state: EditorState.create({
      doc,
      selection: { anchor: cursor },
      extensions: [renderedMath()],
    }),
    parent: document.body,
  });

</storycode:copy>
<a id="f-554c1bc6d72b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-554c1bc6d72b-expression_statement">
describe("maths in the editor", () => {
  it("replaces an inline equation with a rendered one", () => {
    const view = mount("the value $x^2$ is squared\n", 27);
    const drawn = view.dom.querySelectorAll(".cm-math");
    expect(drawn).toHaveLength(1);
    expect(drawn[0].classList.contains("cm-math--display")).toBe(false);
    view.destroy();
  });

  it("leaves the LaTeX as source on the line holding the caret", () => {
    // The reveal IS the source/rendered switch for inline maths.
    const view = mount("the value $x^2$ is squared\n", 3);
    expect(view.dom.querySelectorAll(".cm-math")).toHaveLength(0);
    expect(view.state.doc.toString()).toContain("$x^2$");
    view.destroy();
  });

  it("sets display maths that owns its lines as a block", () => {
    const doc = "before\n$$\na^2 + b^2 = c^2\n$$\nafter\n";
    const view = mount(doc, 0);
    const drawn = view.dom.querySelector(".cm-math");
    expect(drawn).not.toBeNull();
    expect(drawn!.classList.contains("cm-math--display")).toBe(true);
    // A block replacement is a DIV; an inline one is a span, because a span
    // is all that can sit inside a line without adding a row to it.
    expect(drawn!.tagName).toBe("DIV");
    view.destroy();
  });

  it("shows the source until the engine arrives, never an empty box", () => {
    // KaTeX is loaded lazily, so the first paint must still say something
    // true. jsdom never resolves the import here, which is exactly the case
    // this asserts.
    const view = mount("$e = mc^2$\n", 11);
    expect(view.dom.querySelector(".cm-math")!.textContent).toBe("e = mc^2");
    view.destroy();
  });

  it("draws no maths where a document says there is none", () => {
    const view = mount("it cost $5 and $6\n", 0);
    expect(view.dom.querySelectorAll(".cm-math")).toHaveLength(0);
    view.destroy();
  });
});

</storycode:copy>
<a id="f-554c1bc6d72b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-554c1bc6d72b-expression_statement-2">
describe("maths in a .md document", () => {
  it("leaves a shell variable in a cell alone", () => {
    const src = '<hick:exec container="sh">\necho $PATH and $HOME\n</hick:exec>\n';
    const structure = parseHickDoc(src);
    expect(mathSpans(src, verbatimRanges(structure.blocks))).toEqual([]);
  });

  it("gives a <hick:math> block a rail card whose only verb is its source", () => {
    const src = "# Notes\n\n<hick:math>\ne = mc^2\n</hick:math>\n";
    const cards = cardsOf(parseHickDoc(src), { text: src });
    const math = cards.filter((c) => c.kind === "math");
    expect(math).toHaveLength(1);
    expect(math[0].label).toBe("Equation 1");
    expect(actionsFor("math")).toEqual(["source"]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/mathRender.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/mathRender.test.ts"><storycode:paste select="#f-554c1bc6d72b-mount"/><storycode:paste select="#f-554c1bc6d72b-expression_statement"/><storycode:paste select="#f-554c1bc6d72b-expression_statement-2"/></storycode:file>

<a id="output-8cee42215a1b"></a>
## `apps/web/src/lib/math.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Equation Renders And Shows Its Source](contract.md).

<a id="f-8cee42215a1b-found"></a>
### `found`

<storycode:copy id="f-8cee42215a1b-found">
import { describe, expect, it } from "vitest";
import { mathSpans } from "./math";

const found = (src: string) =>
  mathSpans(src).map((s) => ({ text: src.slice(s.from, s.to), display: s.display }));

</storycode:copy>
<a id="f-8cee42215a1b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-8cee42215a1b-expression_statement">
describe("finding LaTeX in prose", () => {
  it("finds display maths on its own lines", () => {
    const src = "before\n$$\na^2 + b^2 = c^2\n$$\nafter\n";
    const spans = mathSpans(src);
    expect(spans).toHaveLength(1);
    expect(spans[0].display).toBe(true);
    expect(spans[0].source.trim()).toBe("a^2 + b^2 = c^2");
  });

  it("finds inline maths inside a sentence", () => {
    expect(found("the value $x^2$ is squared")).toEqual([
      { text: "$x^2$", display: false },
    ]);
  });

  it("reads a `$` pair inside display maths as part of the display block", () => {
    const spans = mathSpans("$$ a $ b $$");
    expect(spans).toHaveLength(1);
    expect(spans[0].display).toBe(true);
  });
});

</storycode:copy>
<a id="f-8cee42215a1b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-8cee42215a1b-expression_statement-2">
describe("not turning money and shell variables into equations", () => {
  it("leaves a pair of prices alone", () => {
    // "$5 and $6" is the classic false positive: two dollars with tight
    // content between them looks exactly like inline maths.
    expect(found("it cost $5 and $6 in total")).toEqual([]);
  });

  it("leaves `$ 5` alone, because maths never opens on a space", () => {
    expect(found("costs $ 5 today $")).toEqual([]);
  });

  it("leaves a trailing space before the closer alone", () => {
    expect(found("$x + y $")).toEqual([]);
  });

  it("does not let an unclosed `$` swallow the rest of the document", () => {
    expect(found("a lone $ sign\nand a later $ one\n")).toEqual([]);
  });

  it("respects an escaped delimiter", () => {
    expect(found("costs \\$5 and \\$6")).toEqual([]);
  });

  it("skips maths inside a verbatim range", () => {
    // `$PATH` in a shell cell is a variable, and the cell's payload is
    // verbatim by definition.
    const src = "echo $PATH$HOME done";
    expect(mathSpans(src, [[0, src.length]])).toEqual([]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/math.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/math.test.ts"><storycode:paste select="#f-8cee42215a1b-found"/><storycode:paste select="#f-8cee42215a1b-expression_statement"/><storycode:paste select="#f-8cee42215a1b-expression_statement-2"/></storycode:file>

<a id="output-7fce937ed3a3"></a>
## `apps/web/src/lib/math.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Equation Renders And Shows Its Source](contract.md).

<a id="f-7fce937ed3a3-MathSpan"></a>
### `MathSpan`

Where the LaTeX is in a piece of prose.
Two notations, and they are genuinely different things rather than one
thing with a size setting:
- `$$…$$` is DISPLAY maths: an equation that is a paragraph, set on its
own lines and centred. It may span lines.
- `$…$` is INLINE maths: a symbol inside a sentence, which must stay on
the sentence's line and share its baseline.
The hard part is not the delimiters, it is the false positives. `$5 and $6`
is money, not an equation, and a document about shell scripting is full of
`$PATH` and `$1`. TeX's own rule is the one that works: an inline `$` opens
maths only when the character after it is not a space, and closes only when
the character before the closer is not a space. That alone throws out most
prices — "$5 and $6" has "5 and 6" between two tight delimiters, so it
needs the second rule too: a closing `$` may not be followed immediately by
a digit, which is what "$6" looks like from the inside.
Nothing here renders anything. This module is pure text-in / spans-out so
the false-positive rules can be argued with in a test rather than in a
browser.
One run of maths found in prose.

<storycode:copy id="f-7fce937ed3a3-MathSpan">
// Where the LaTeX is in a piece of prose.
//
// Two notations, and they are genuinely different things rather than one
// thing with a size setting:
//
//  - `$$…$$` is DISPLAY maths: an equation that is a paragraph, set on its
//    own lines and centred. It may span lines.
//  - `$…$` is INLINE maths: a symbol inside a sentence, which must stay on
//    the sentence's line and share its baseline.
//
// The hard part is not the delimiters, it is the false positives. `$5 and $6`
// is money, not an equation, and a document about shell scripting is full of
// `$PATH` and `$1`. TeX's own rule is the one that works: an inline `$` opens
// maths only when the character after it is not a space, and closes only when
// the character before the closer is not a space. That alone throws out most
// prices — "$5 and $6" has "5 and 6" between two tight delimiters, so it
// needs the second rule too: a closing `$` may not be followed immediately by
// a digit, which is what "$6" looks like from the inside.
//
// Nothing here renders anything. This module is pure text-in / spans-out so
// the false-positive rules can be argued with in a test rather than in a
// browser.

/** One run of maths found in prose. */
export interface MathSpan {
  /** Whole span, both delimiters included. */
  from: number;
  to: number;
  /** Display maths (`$$`) sets on its own lines; inline shares the line. */
  display: boolean;
  /** The LaTeX between the delimiters, untrimmed. */
  source: string;
}

</storycode:copy>
<a id="f-7fce937ed3a3-escapedAt"></a>
### `escapedAt`

Escaped delimiters (`\$`) are dollars a reader wants to see.

<storycode:copy id="f-7fce937ed3a3-escapedAt">
/** Escaped delimiters (`\$`) are dollars a reader wants to see. */
function escapedAt(text: string, index: number): boolean {
  let backslashes = 0;
  for (let i = index - 1; i >= 0 && text[i] === "\\"; i--) backslashes++;
  return backslashes % 2 === 1;
}

</storycode:copy>
<a id="f-7fce937ed3a3-inRanges"></a>
### `inRanges`

<storycode:copy id="f-7fce937ed3a3-inRanges">
function inRanges(pos: number, ranges: readonly [number, number][]): boolean {
  return ranges.some(([from, to]) => pos >= from && pos < to);
}

</storycode:copy>
<a id="f-7fce937ed3a3-mathSpans"></a>
### `mathSpans`

Every maths span in `text`, in document order, skipping anything inside
`verbatim` (fenced code, a cell's payload, a generated file's body).
Display maths is matched first and greedily consumes its region, so the
`$` pair inside `$$ a $ b $$` is part of the display block rather than two
inline spans nested in it.

<storycode:copy id="f-7fce937ed3a3-mathSpans">
/**
 * Every maths span in `text`, in document order, skipping anything inside
 * `verbatim` (fenced code, a cell's payload, a generated file's body).
 *
 * Display maths is matched first and greedily consumes its region, so the
 * `$` pair inside `$$ a $ b $$` is part of the display block rather than two
 * inline spans nested in it.
 */
export function mathSpans(
  text: string,
  verbatim: readonly [number, number][] = [],
): MathSpan[] {
  const spans: MathSpan[] = [];
  let i = 0;
  while (i < text.length) {
    if (text[i] !== "$" || escapedAt(text, i) || inRanges(i, verbatim)) {
      i++;
      continue;
    }
    if (text.startsWith("$$", i)) {
      const close = findClose(text, i + 2, "$$");
      if (close < 0) {
        i += 2;
        continue;
      }
      spans.push({
        from: i,
        to: close + 2,
        display: true,
        source: text.slice(i + 2, close),
      });
      i = close + 2;
      continue;
    }
    const close = findInlineClose(text, i + 1);
    if (close < 0) {
      i++;
      continue;
    }
    spans.push({ from: i, to: close + 1, display: false, source: text.slice(i + 1, close) });
    i = close + 1;
  }
  return spans;
}

</storycode:copy>
<a id="f-7fce937ed3a3-findClose"></a>
### `findClose`

<storycode:copy id="f-7fce937ed3a3-findClose">
function findClose(text: string, from: number, delimiter: string): number {
  for (let i = from; i <= text.length - delimiter.length; i++) {
    if (text.startsWith(delimiter, i) && !escapedAt(text, i)) return i;
  }
  return -1;
}

</storycode:copy>
<a id="f-7fce937ed3a3-findInlineClose"></a>
### `findInlineClose`

The closing `$` of an inline span opened at `from`, or -1.
The rules, all of them about not turning money and shell variables into
equations:
- the opener must be followed by a non-space, so `$ 5` is never maths;
- the closer must be preceded by a non-space, so `5 $` is never maths;
- the closer may not be followed by a digit, which is what the SECOND
price in "$5 and $6" looks like from inside the pair;
- inline maths does not cross a blank line, and in practice does not
cross a line at all — an unclosed `$` should give up at the end of its
line rather than swallow the rest of the document.

<storycode:copy id="f-7fce937ed3a3-findInlineClose">
/**
 * The closing `$` of an inline span opened at `from`, or -1.
 *
 * The rules, all of them about not turning money and shell variables into
 * equations:
 *  - the opener must be followed by a non-space, so `$ 5` is never maths;
 *  - the closer must be preceded by a non-space, so `5 $` is never maths;
 *  - the closer may not be followed by a digit, which is what the SECOND
 *    price in "$5 and $6" looks like from inside the pair;
 *  - inline maths does not cross a blank line, and in practice does not
 *    cross a line at all — an unclosed `$` should give up at the end of its
 *    line rather than swallow the rest of the document.
 */
function findInlineClose(text: string, from: number): number {
  if (from >= text.length) return -1;
  const opener = text[from];
  if (opener === " " || opener === "\t" || opener === "\n") return -1;
  for (let i = from; i < text.length; i++) {
    const ch = text[i];
    if (ch === "\n") return -1;
    if (ch !== "$" || escapedAt(text, i)) continue;
    const before = text[i - 1];
    if (before === " " || before === "\t") return -1;
    const after = text[i + 1];
    if (after !== undefined && after >= "0" && after <= "9") return -1;
    return i;
  }
  return -1;
}
</storycode:copy>

### Assemble `apps/web/src/lib/math.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/math.ts"><storycode:paste select="#f-7fce937ed3a3-MathSpan"/><storycode:paste select="#f-7fce937ed3a3-escapedAt"/><storycode:paste select="#f-7fce937ed3a3-inRanges"/><storycode:paste select="#f-7fce937ed3a3-mathSpans"/><storycode:paste select="#f-7fce937ed3a3-findClose"/><storycode:paste select="#f-7fce937ed3a3-findInlineClose"/></storycode:file>

<a id="output-1d7d3ed04488"></a>
## `crates/hickory-cli/tests/math.rs`

Related contracts: [An Equation Renders And Shows Its Source](contract.md).

<a id="f-1d7d3ed04488-hick"></a>
### `hick`

Display maths, and what a reader with no hick installed sees.
Protects docs/guarantees/authoring/an-equation-renders-and-shows-its-source.md
The app typesets a `<hick:math>` block in place. The woven markdown is the
other half of that promise: an equation that only existed inside our own
editor would be worth less than the image someone would otherwise paste.

<storycode:copy id="f-1d7d3ed04488-hick">
//! Display maths, and what a reader with no hick installed sees.
//!
//! Protects docs/guarantees/authoring/an-equation-renders-and-shows-its-source.md
//!
//! The app typesets a `<hick:math>` block in place. The woven markdown is the
//! other half of that promise: an equation that only existed inside our own
//! editor would be worth less than the image someone would otherwise paste.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-1d7d3ed04488-DOC"></a>
### `DOC`

<storycode:copy id="f-1d7d3ed04488-DOC">
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="physics.md">
# Energy

The relation itself:

<hick:math>
e = mc^2
</hick:math>
</hick:doc>
"##;

</storycode:copy>
<a id="f-1d7d3ed04488-weave"></a>
### `weave`

<storycode:copy id="f-1d7d3ed04488-weave">
fn weave(source: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("physics.md");
    std::fs::write(&doc, source).unwrap();
    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::read_to_string(dir.path().join("physics.md")).unwrap()
}

</storycode:copy>
<a id="f-1d7d3ed04488-maths_weaves_into_the_dollar_block_every_renderer_understands"></a>
### `maths_weaves_into_the_dollar_block_every_renderer_understands`

<storycode:copy id="f-1d7d3ed04488-maths_weaves_into_the_dollar_block_every_renderer_understands">
#[test]
fn maths_weaves_into_the_dollar_block_every_renderer_understands() {
    // `$$…$$` is what GitHub, an editor preview, and every static-site
    // generator already draw. A ```latex fence would show a reader the
    // SOURCE of an equation where they expected the equation.
    let woven = weave(DOC);
    assert!(woven.contains("$$"), "no display-maths block in:\n{woven}");
    assert!(woven.contains("e = mc^2"), "maths body missing:\n{woven}");
}

</storycode:copy>
<a id="f-1d7d3ed04488-the_tag_is_source_and_does_not_survive_the_weave"></a>
### `the_tag_is_source_and_does_not_survive_the_weave`

<storycode:copy id="f-1d7d3ed04488-the_tag_is_source_and_does_not_survive_the_weave">
#[test]
fn the_tag_is_source_and_does_not_survive_the_weave() {
    let woven = weave(DOC);
    assert!(
        !woven.contains("hick:math"),
        "tag leaked into the weave:\n{woven}"
    );
}

</storycode:copy>
<a id="f-1d7d3ed04488-the_prose_around_an_equation_is_untouched"></a>
### `the_prose_around_an_equation_is_untouched`

<storycode:copy id="f-1d7d3ed04488-the_prose_around_an_equation_is_untouched">
#[test]
fn the_prose_around_an_equation_is_untouched() {
    // The equation is a paragraph among paragraphs, not a section break.
    let woven = weave(DOC);
    assert!(woven.contains("# Energy"), "heading missing:\n{woven}");
    assert!(
        woven.contains("The relation itself:"),
        "prose missing:\n{woven}"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/math.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/math.rs"><storycode:paste select="#f-1d7d3ed04488-hick"/><storycode:paste select="#f-1d7d3ed04488-DOC"/><storycode:paste select="#f-1d7d3ed04488-weave"/><storycode:paste select="#f-1d7d3ed04488-maths_weaves_into_the_dollar_block_every_renderer_understands"/><storycode:paste select="#f-1d7d3ed04488-the_tag_is_source_and_does_not_survive_the_weave"/><storycode:paste select="#f-1d7d3ed04488-the_prose_around_an_equation_is_untouched"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../mechanisms/apps/desktop/src-tauri/src/lib.rs.md#output-42caba993099)
- [apps/web/src/components/MathPanel.tsx](../../../mechanisms/apps/web/src/components/MathPanel.tsx.md#output-79c748e2d9c5)
- [apps/web/src/editor/CardRail.tsx](../../../mechanisms/apps/web/src/editor/CardRail.tsx.md#output-f6019c4774f8)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/cards.ts](../../../mechanisms/apps/web/src/editor/cards.ts.md#output-84385185ee44)
- [apps/web/src/editor/mathRender.ts](../../../mechanisms/apps/web/src/editor/mathRender.ts.md#output-b432fcb15a24)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [apps/web/src/lib/insertCatalog.ts](../../../mechanisms/apps/web/src/lib/insertCatalog.ts.md#output-8620a55486dc)
- [apps/web/src/lib/math.ts](contract.md#output-7fce937ed3a3)
- [apps/web/src/lib/railActions.ts](../../../mechanisms/apps/web/src/lib/railActions.ts.md#output-499155452cf6)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)

### Tests and fixtures

- [apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs](../../../mechanisms/apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs.md#output-ffb5698079c9)
- [apps/web/src/editor/mathRender.test.ts](contract.md#output-554c1bc6d72b)
- [apps/web/src/lib/math.test.ts](contract.md#output-8cee42215a1b)
- [crates/hickory-cli/tests/math.rs](contract.md#output-1d7d3ed04488)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
