<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Formula Is An Expression In A Real Language

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a `<hick:table>` that names a `language`, when a cell begins with `=`,
then what follows is evaluated as an expression in that language, with the
cells it refers to bound as ordinary variables; and when it cannot be
evaluated, the cell reports what that language said.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)
- [Requirements 3](requirements/03.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />
- [Requirement detail](requirements/03.md)
<storycode:include file="requirements/03.md" />

## Implementation

<a id="output-3de309f900ec"></a>
## `apps/web/src/lib/cellRef.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Formula Is An Expression In A Real Language](contract.md).

<a id="f-3de309f900ec-columnLabel"></a>
### `columnLabel`

A1 notation, in the browser.
The twin of `CellRef` in crates/hick-formula/src/graph.rs, and deliberately
only the half the UI needs: turning a position into a label, and back. The
interesting work — which cells a formula depends on, what order they
evaluate in, what a cycle is — lives on the host and stays there. Two
implementations of THAT would eventually disagree, and a table that
evaluated differently depending on who asked would be unusable.
The letters of a zero-based column, e.g. 0 → "A", 26 → "AA".
Bijective base-26, which is the one every spreadsheet uses and the one
`CellRef` on the host parses back. Drawn across the top of the grid, so the
reference a formula needs is the thing already on screen rather than
something the author counts out.

<storycode:copy id="f-3de309f900ec-columnLabel">
// A1 notation, in the browser.
//
// The twin of `CellRef` in crates/hick-formula/src/graph.rs, and deliberately
// only the half the UI needs: turning a position into a label, and back. The
// interesting work — which cells a formula depends on, what order they
// evaluate in, what a cycle is — lives on the host and stays there. Two
// implementations of THAT would eventually disagree, and a table that
// evaluated differently depending on who asked would be unusable.

/**
 * The letters of a zero-based column, e.g. 0 → "A", 26 → "AA".
 *
 * Bijective base-26, which is the one every spreadsheet uses and the one
 * `CellRef` on the host parses back. Drawn across the top of the grid, so the
 * reference a formula needs is the thing already on screen rather than
 * something the author counts out.
 */
export function columnLabel(column: number): string {
  let letters = "";
  let n = column + 1;
  while (n > 0) {
    const remainder = (n - 1) % 26;
    letters = String.fromCharCode(65 + remainder) + letters;
    n = Math.floor((n - 1) / 26);
  }
  return letters;
}

</storycode:copy>
<a id="f-3de309f900ec-cellLabel"></a>
### `cellLabel`

<storycode:copy id="f-3de309f900ec-cellLabel">
/** The A1 label of a zero-based position, e.g. (0, 0) → "A1". */
export function cellLabel(column: number, row: number): string {
  return `${columnLabel(column)}${row + 1}`;
}

</storycode:copy>
<a id="f-3de309f900ec-isFormula"></a>
### `isFormula`

Whether a cell's text is a formula: it begins with `=`, as in every
spreadsheet anybody has used.

<storycode:copy id="f-3de309f900ec-isFormula">
/** Whether a cell's text is a formula: it begins with `=`, as in every
 * spreadsheet anybody has used. */
export function isFormula(text: string): boolean {
  return text.trimStart().startsWith("=");
}

</storycode:copy>
<a id="f-3de309f900ec-parseCellLabel"></a>
### `parseCellLabel`

The position an A1 label names, e.g. "B3" → `{ column: 1, row: 2 }`.
The inverse of `cellLabel`, and the only other half of A1 notation the UI
needs: the host answers in labels — a formula's value, a step's bindings —
and the grid has to find the cell those labels mean. Everything harder
than this (which cells a formula depends on, what order they evaluate in)
stays on the host, for the reason at the top of this file.
Deliberately narrow where `CellRef::parse` is forgiving: this reads labels
this product produced, not labels a person pasted, so `$B$4` and lower
case are not its problem.

<storycode:copy id="f-3de309f900ec-parseCellLabel">
/**
 * The position an A1 label names, e.g. "B3" → `{ column: 1, row: 2 }`.
 *
 * The inverse of `cellLabel`, and the only other half of A1 notation the UI
 * needs: the host answers in labels — a formula's value, a step's bindings —
 * and the grid has to find the cell those labels mean. Everything harder
 * than this (which cells a formula depends on, what order they evaluate in)
 * stays on the host, for the reason at the top of this file.
 *
 * Deliberately narrow where `CellRef::parse` is forgiving: this reads labels
 * this product produced, not labels a person pasted, so `$B$4` and lower
 * case are not its problem.
 */
export function parseCellLabel(label: string): { column: number; row: number } | null {
  const match = /^([A-Z]+)([0-9]+)$/.exec(label.trim());
  if (!match) return null;
  let column = 0;
  for (const letter of match[1]) {
    // Bijective base-26: there is no zero digit, which is why Z is followed
    // by AA.
    column = column * 26 + (letter.charCodeAt(0) - 64);
  }
  const row = Number(match[2]);
  if (row < 1) return null;
  return { column: column - 1, row: row - 1 };
}
</storycode:copy>

### Assemble `apps/web/src/lib/cellRef.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/cellRef.ts"><storycode:paste select="#f-3de309f900ec-columnLabel"/><storycode:paste select="#f-3de309f900ec-cellLabel"/><storycode:paste select="#f-3de309f900ec-isFormula"/><storycode:paste select="#f-3de309f900ec-parseCellLabel"/></storycode:file>

<a id="output-932c1cfa22d9"></a>
## `apps/web/src/lib/formulaPoint.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Formula Is An Expression In A Real Language](contract.md).

<a id="f-932c1cfa22d9-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md
— rule 7, "a click is how a reference gets written".

<storycode:copy id="f-932c1cfa22d9-expression_statement">
// Protects docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md
// — rule 7, "a click is how a reference gets written".

import { describe, expect, it } from "vitest";

import { acceptsReference, pointAt } from "./formulaPoint";

describe("where a reference could go", () => {
  it("accepts one after the `=` that starts a formula", () => {
    expect(acceptsReference("=", 1)).toBe(true);
  });

  it("accepts one after an operator, an opener, or a separator", () => {
    for (const text of ["=1+", "=sum(", "=sum([A1,", "=A1 * ", "=f(x, "]) {
      expect(acceptsReference(text, text.length)).toBe(true);
    }
  });

  it("refuses one where the expression already has its operand", () => {
    // After `42` or after a word, a click is a click — the formula is not
    // asking for anything.
    for (const text of ["=42", "=sum(A1)", "=total"]) {
      expect(acceptsReference(text, text.length)).toBe(false);
    }
  });

  it("refuses one in an empty field, which is not a formula at all", () => {
    expect(acceptsReference("", 0)).toBe(false);
  });
});

</storycode:copy>
<a id="f-932c1cfa22d9-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-932c1cfa22d9-expression_statement-2">
describe("writing the reference in", () => {
  it("puts it at the caret", () => {
    const out = pointAt("=1+", 3, "B2")!;
    expect(out.text).toBe("=1+B2");
    expect(out.caret).toBe(5);
    expect(out.pointed).toEqual({ start: 3, end: 5 });
  });

  it("writes into the middle when that is where the caret is", () => {
    const out = pointAt("=(+1)", 2, "A1")!;
    expect(out.text).toBe("=(A1+1)");
  });

  it("REPLACES the reference the last click put there", () => {
    // Clicking around to find the right cell should leave one reference, not
    // five.
    const first = pointAt("=", 1, "B2")!;
    const second = pointAt(first.text, first.caret, "C7", first.pointed)!;
    expect(second.text).toBe("=C7");
    expect(second.pointed).toEqual({ start: 1, end: 3 });
    const third = pointAt(second.text, second.caret, "AA10", second.pointed)!;
    expect(third.text).toBe("=AA10");
  });

  it("stops replacing once the caret has moved off the reference", () => {
    // The person typed `+` after it; the next click is a new operand.
    const first = pointAt("=", 1, "B2")!;
    const typed = `${first.text}+`;
    const next = pointAt(typed, typed.length, "C7", first.pointed)!;
    expect(next.text).toBe("=B2+C7");
  });

  it("says no when the formula is not asking for an operand", () => {
    // The caller then treats the click as the ordinary click it was.
    expect(pointAt("=42", 3, "B2")).toBeNull();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/formulaPoint.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/formulaPoint.test.ts"><storycode:paste select="#f-932c1cfa22d9-expression_statement"/><storycode:paste select="#f-932c1cfa22d9-expression_statement-2"/></storycode:file>

<a id="output-b60b27abb53f"></a>
## `apps/web/src/lib/formulaPoint.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Formula Is An Expression In A Real Language](contract.md).

<a id="f-b60b27abb53f-PointedAt"></a>
### `PointedAt`

Pointing at a cell while you are writing a formula.
In every spreadsheet, `=` followed by a click on another cell writes that
cell's reference into what you are typing. It is the feature that makes
formulas usable without counting rows, and the reason people can write
`=B2+B3` having never learned A1 notation.
The whole difficulty is deciding whether a click MEANS a reference. While
a formula is open, a click is either "put B2 here" or "I am done with this
cell, take me to B2", and the grid has to guess right every time. A
spreadsheet decides by where the caret is:
- Just after a reference this same act inserted → REPLACE it. Clicking
around to find the right cell should leave one reference, not five.
- Somewhere an operand could go — after `=`, `+`, `(`, `,`, a space →
INSERT. There is nothing there yet, so a reference is the only thing
the click could mean.
- Anywhere else, e.g. immediately after `42` or after a word → the
formula does not want an operand, so the click is a click: commit and
move on.
This module is that decision, as a pure function over the text and the
caret. It is deliberately not language-aware: the expression is in Python
or JavaScript or Rust, and the panel cannot parse it (see hick-formula's
lib.rs). The operand rule is a lexical one that holds in all of them.
A click writes one cell; dragging writes an A1 range. The panel still only
decides where text may go — the host expands a range and binds it as one
list value before Python or JavaScript evaluates the expression.
Where the reference this act last inserted sits in the text, so the next
click replaces it rather than piling up beside it.

<storycode:copy id="f-b60b27abb53f-PointedAt">
// Pointing at a cell while you are writing a formula.
//
// In every spreadsheet, `=` followed by a click on another cell writes that
// cell's reference into what you are typing. It is the feature that makes
// formulas usable without counting rows, and the reason people can write
// `=B2+B3` having never learned A1 notation.
//
// The whole difficulty is deciding whether a click MEANS a reference. While
// a formula is open, a click is either "put B2 here" or "I am done with this
// cell, take me to B2", and the grid has to guess right every time. A
// spreadsheet decides by where the caret is:
//
//   - Just after a reference this same act inserted → REPLACE it. Clicking
//     around to find the right cell should leave one reference, not five.
//   - Somewhere an operand could go — after `=`, `+`, `(`, `,`, a space →
//     INSERT. There is nothing there yet, so a reference is the only thing
//     the click could mean.
//   - Anywhere else, e.g. immediately after `42` or after a word → the
//     formula does not want an operand, so the click is a click: commit and
//     move on.
//
// This module is that decision, as a pure function over the text and the
// caret. It is deliberately not language-aware: the expression is in Python
// or JavaScript or Rust, and the panel cannot parse it (see hick-formula's
// lib.rs). The operand rule is a lexical one that holds in all of them.
//
// A click writes one cell; dragging writes an A1 range. The panel still only
// decides where text may go — the host expands a range and binds it as one
// list value before Python or JavaScript evaluates the expression.

/** Where the reference this act last inserted sits in the text, so the next
 * click replaces it rather than piling up beside it. */
export interface PointedAt {
  start: number;
  end: number;
}

</storycode:copy>
<a id="f-b60b27abb53f-Pointing"></a>
### `Pointing`

<storycode:copy id="f-b60b27abb53f-Pointing">
export interface Pointing {
  /** The text with the reference in it. */
  text: string;
  /** Where to leave the caret: just after what was written. */
  caret: number;
  /** Where the reference landed, to be replaced by the next click. */
  pointed: PointedAt;
}

</storycode:copy>
<a id="f-b60b27abb53f-OPERAND_AFTER"></a>
### `OPERAND_AFTER`

The characters after which an operand could begin.
Openers, separators, and operators — the shapes that are punctuation in
every language a formula might be written in. A letter, a digit, `)` or a
closing bracket are deliberately absent: after those the expression already
has its operand, and a click is a click.

<storycode:copy id="f-b60b27abb53f-OPERAND_AFTER">
/**
 * The characters after which an operand could begin.
 *
 * Openers, separators, and operators — the shapes that are punctuation in
 * every language a formula might be written in. A letter, a digit, `)` or a
 * closing bracket are deliberately absent: after those the expression already
 * has its operand, and a click is a click.
 */
const OPERAND_AFTER = new Set([
  "=",
  "+",
  "-",
  "*",
  "/",
  "%",
  "^",
  "(",
  "[",
  "{",
  ",",
  ":",
  ";",
  "<",
  ">",
  "!",
  "&",
  "|",
  "~",
  "?",
  " ",
  "\t",
  "\n",
]);

</storycode:copy>
<a id="f-b60b27abb53f-acceptsReference"></a>
### `acceptsReference`

Whether a reference could go in at `caret` — see the note above.

<storycode:copy id="f-b60b27abb53f-acceptsReference">
/** Whether a reference could go in at `caret` — see the note above. */
export function acceptsReference(text: string, caret: number): boolean {
  const before = text.slice(0, Math.max(0, caret)).trimEnd();
  // Nothing before the caret at all is not an operand position in a formula:
  // a formula's first character is its `=`, and text with no `=` is not a
  // formula in the first place.
  if (before === "") return false;
  return OPERAND_AFTER.has(before[before.length - 1]);
}

</storycode:copy>
<a id="f-b60b27abb53f-pointAt"></a>
### `pointAt`

Write `label` into `text` at `caret`, or say the click did not mean that.
`pointed` is where the previous click in this same edit put a reference;
when the caret is still sitting at the end of it, this one replaces it.
Answers null when the formula is not asking for an operand — the caller
then treats the click as an ordinary click, which is what it was.

<storycode:copy id="f-b60b27abb53f-pointAt">
/**
 * Write `label` into `text` at `caret`, or say the click did not mean that.
 *
 * `pointed` is where the previous click in this same edit put a reference;
 * when the caret is still sitting at the end of it, this one replaces it.
 * Answers null when the formula is not asking for an operand — the caller
 * then treats the click as an ordinary click, which is what it was.
 */
export function pointAt(
  text: string,
  caret: number,
  label: string,
  pointed?: PointedAt | null,
): Pointing | null {
  const at = Math.max(0, Math.min(caret, text.length));
  if (pointed && at === pointed.end && pointed.end > pointed.start) {
    const next = text.slice(0, pointed.start) + label + text.slice(pointed.end);
    return {
      text: next,
      caret: pointed.start + label.length,
      pointed: { start: pointed.start, end: pointed.start + label.length },
    };
  }
  if (!acceptsReference(text, at)) return null;
  const next = text.slice(0, at) + label + text.slice(at);
  return {
    text: next,
    caret: at + label.length,
    pointed: { start: at, end: at + label.length },
  };
}
</storycode:copy>

### Assemble `apps/web/src/lib/formulaPoint.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/formulaPoint.ts"><storycode:paste select="#f-b60b27abb53f-PointedAt"/><storycode:paste select="#f-b60b27abb53f-Pointing"/><storycode:paste select="#f-b60b27abb53f-OPERAND_AFTER"/><storycode:paste select="#f-b60b27abb53f-acceptsReference"/><storycode:paste select="#f-b60b27abb53f-pointAt"/></storycode:file>

<a id="output-806bbf2d45fa"></a>
## `crates/hick-formula/src/backend.rs`

Related contracts: [A Formula Is An Expression In A Real Language](contract.md).

<a id="f-806bbf2d45fa-BACKENDS"></a>
### `BACKENDS`

Which program evaluates a language's formulas, and getting it onto the
machine.
# Why there is nothing to download
`hick lsp install` and `hick dap install` fetch things, because a language
server is a large third-party program nobody would want us to reimplement.
A formula backend is the opposite: sixty lines that read a pipe and call
`eval`. So the backends are **embedded in this binary** and written into
the project's cache the first time they are needed.
That makes "auto-installed" literally true, and true offline. There is no
network path here at all — no URL, no fallback, no "re-run when you are
online". What a machine needs is the *interpreter*, which it either has or
does not, and which we say plainly rather than trying to provide.
# Where they go
`.hick-cache/formula/`, which `hick init` already keeps out of git — the
same place `hick search --install-model` puts its model. A backend is
derived, reproducible from the binary that wrote it, and belongs in a
cache rather than in anybody's history.

<storycode:copy id="f-806bbf2d45fa-BACKENDS">
//! Which program evaluates a language's formulas, and getting it onto the
//! machine.
//!
//! # Why there is nothing to download
//!
//! `hick lsp install` and `hick dap install` fetch things, because a language
//! server is a large third-party program nobody would want us to reimplement.
//! A formula backend is the opposite: sixty lines that read a pipe and call
//! `eval`. So the backends are **embedded in this binary** and written into
//! the project's cache the first time they are needed.
//!
//! That makes "auto-installed" literally true, and true offline. There is no
//! network path here at all — no URL, no fallback, no "re-run when you are
//! online". What a machine needs is the *interpreter*, which it either has or
//! does not, and which we say plainly rather than trying to provide.
//!
//! # Where they go
//!
//! `.hick-cache/formula/`, which `hick init` already keeps out of git — the
//! same place `hick search --install-model` puts its model. A backend is
//! derived, reproducible from the binary that wrote it, and belongs in a
//! cache rather than in anybody's history.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

pub use hick_plugins::FormulaBackend as Backend;

/// Compatibility view of the plugin-owned backends.
pub static BACKENDS: std::sync::LazyLock<Vec<Backend>> = std::sync::LazyLock::new(|| {
    hick_plugins::registry()
        .plugins()
        .flat_map(|p| p.interfaces)
        .filter_map(|i| match i {
            hick_plugins::Interface::Formula(backend) => Some(*backend),
            _ => None,
        })
        .collect()
});

</storycode:copy>
<a id="f-806bbf2d45fa-canonical_language"></a>
### `canonical_language`

<storycode:copy id="f-806bbf2d45fa-canonical_language">
pub fn canonical_language(name: &str) -> Option<&'static str> {
    let name = name.trim().to_ascii_lowercase();
    BACKENDS
        .iter()
        .find(|b| b.aliases.contains(&name.as_str()))
        .map(|b| b.language)
}

</storycode:copy>
<a id="f-806bbf2d45fa-backend_for"></a>
### `backend_for`

The backend for a language, whatever it was called.

<storycode:copy id="f-806bbf2d45fa-backend_for">
/// The backend for a language, whatever it was called.
pub fn backend_for(language: &str) -> Option<&'static Backend> {
    let canonical = canonical_language(language)?;
    BACKENDS.iter().find(|b| b.language == canonical)
}

</storycode:copy>
<a id="f-806bbf2d45fa-backend_dir"></a>
### `backend_dir`

<storycode:copy id="f-806bbf2d45fa-backend_dir">
/// Where backends live for a project.
pub fn backend_dir(root: &Path) -> PathBuf {
    root.join(".hick-cache").join("formula")
}

</storycode:copy>
<a id="f-806bbf2d45fa-find_interpreter"></a>
### `find_interpreter`

The first interpreter on this machine that answers `--version`.
Checked rather than assumed: a `node` on `PATH` that is a broken shim is a
far more confusing failure at evaluation time than at install time.

<storycode:copy id="f-806bbf2d45fa-find_interpreter">
/// The first interpreter on this machine that answers `--version`.
///
/// Checked rather than assumed: a `node` on `PATH` that is a broken shim is a
/// far more confusing failure at evaluation time than at install time.
pub fn find_interpreter(backend: &Backend) -> Option<String> {
    backend
        .interpreters
        .iter()
        .find(|name| {
            std::process::Command::new(name)
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
        .map(|name| name.to_string())
}

</storycode:copy>
<a id="f-806bbf2d45fa-install"></a>
### `install`

Write a language's backend into the project's cache, and check that
something can run it.
Idempotent, and it rewrites rather than skipping: the script is derived
from this binary, so an older copy left by an older version is exactly the
thing to replace. Nothing is downloaded — see the module header.

<storycode:copy id="f-806bbf2d45fa-install">
/// Write a language's backend into the project's cache, and check that
/// something can run it.
///
/// Idempotent, and it rewrites rather than skipping: the script is derived
/// from this binary, so an older copy left by an older version is exactly the
/// thing to replace. Nothing is downloaded — see the module header.
pub fn install(root: &Path, language: &str) -> Result<PathBuf> {
    let backend = backend_for(language).with_context(|| {
        let known: Vec<&str> = BACKENDS.iter().map(|b| b.language).collect();
        format!(
            "no formula backend for `{language}`.\n  \
             This build carries: {}.\n  \
             A backend is a small script that evaluates one expression at a \
             time; adding one is a contribution to hick-formula, not a \
             download.",
            known.join(", ")
        )
    })?;

    let dir = backend_dir(root);
    std::fs::create_dir_all(&dir).with_context(|| format!("could not create {}", dir.display()))?;
    let path = dir.join(backend.file_name);
    std::fs::write(&path, backend.source)
        .with_context(|| format!("could not write {}", path.display()))?;

    if find_interpreter(backend).is_none() {
        bail!(
            "wrote {} — but nothing on this machine can run it.\n  \
             Tried: {}.\n  \
             Install one and formulas in `{}` will work; documents using \
             other languages are unaffected.",
            path.display(),
            backend.interpreters.join(", "),
            backend.language,
        );
    }
    Ok(path)
}

</storycode:copy>
<a id="f-806bbf2d45fa-installed_languages"></a>
### `installed_languages`

Every language this project can evaluate right now: a backend this build
knows, an interpreter this machine has.

<storycode:copy id="f-806bbf2d45fa-installed_languages">
/// Every language this project can evaluate right now: a backend this build
/// knows, an interpreter this machine has.
pub fn installed_languages(root: &Path) -> Vec<&'static str> {
    BACKENDS
        .iter()
        .filter(|b| backend_dir(root).join(b.file_name).is_file() && find_interpreter(b).is_some())
        .map(|b| b.language)
        .collect()
}

</storycode:copy>
<a id="f-806bbf2d45fa-ensure"></a>
### `ensure`

The command that runs a language's backend, installing it if it is not
there yet.
This is the "auto" in auto-installed: the first formula in a document puts
the script in place. Writing sixty lines into a cache is not an act worth
asking permission for, which is exactly why it can be automatic here and
could not be for `hick lsp install`.

<storycode:copy id="f-806bbf2d45fa-ensure">
/// The command that runs a language's backend, installing it if it is not
/// there yet.
///
/// This is the "auto" in auto-installed: the first formula in a document puts
/// the script in place. Writing sixty lines into a cache is not an act worth
/// asking permission for, which is exactly why it can be automatic here and
/// could not be for `hick lsp install`.
pub fn ensure(root: &Path, language: &str) -> Result<(String, PathBuf)> {
    let backend = backend_for(language)
        .with_context(|| format!("no formula backend for `{language}` in this build"))?;
    let path = backend_dir(root).join(backend.file_name);
    // Rewritten when it is missing OR stale: the script is derived from this
    // binary, so a copy left by an older version is exactly what to replace.
    let current = std::fs::read_to_string(&path).ok();
    if current.as_deref() != Some(backend.source) {
        install(root, language)?;
    }
    let interpreter = find_interpreter(backend).with_context(|| {
        format!(
            "formulas in `{}` need one of: {}. None is on this machine.",
            backend.language,
            backend.interpreters.join(", ")
        )
    })?;
    Ok((interpreter, path))
}

</storycode:copy>
<a id="f-806bbf2d45fa-tests"></a>
### `tests`

<storycode:copy id="f-806bbf2d45fa-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_is_found_by_any_name_a_document_would_use() {
        assert_eq!(backend_for("py").map(|b| b.language), Some("python"));
        assert_eq!(backend_for("Python3").map(|b| b.language), Some("python"));
        assert_eq!(backend_for("js").map(|b| b.language), Some("javascript"));
        assert_eq!(
            backend_for("typescript").map(|b| b.language),
            Some("javascript")
        );
        assert_eq!(backend_for("node").map(|b| b.language), Some("javascript"));
    }

    #[test]
    fn an_unknown_language_is_none_rather_than_a_guess() {
        assert!(backend_for("cobol").is_none());
        assert!(backend_for("").is_none());
    }

    #[test]
    fn installing_writes_the_script_into_the_cache_git_ignores() {
        let dir = tempfile::tempdir().unwrap();
        // May fail on a machine with no python; the FILE is written either
        // way, which is the part this asserts.
        let _ = install(dir.path(), "python");
        let written = dir.path().join(".hick-cache/formula/python_backend.py");
        assert!(written.is_file(), "the script is written");
        assert!(
            std::fs::read_to_string(&written)
                .unwrap()
                .contains("def evaluate"),
            "and it is the real script"
        );
    }

    #[test]
    fn installing_an_unknown_language_says_what_this_build_carries() {
        let dir = tempfile::tempdir().unwrap();
        let error = install(dir.path(), "cobol").unwrap_err().to_string();
        assert!(error.contains("python"), "{error}");
        assert!(error.contains("not a download"), "{error}");
    }

    #[test]
    fn installing_replaces_an_older_copy_rather_than_skipping_it() {
        // The script is derived from this binary, so a stale copy left by an
        // older version is exactly the thing to overwrite.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".hick-cache/formula/python_backend.py");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "# stale\n").unwrap();
        let _ = install(dir.path(), "python");
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("def evaluate")
        );
    }

    #[test]
    fn nothing_is_written_outside_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let _ = install(dir.path(), "python");
        let top: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(top, vec![".hick-cache".to_string()]);
    }
}
</storycode:copy>

### Assemble `crates/hick-formula/src/backend.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-formula/src/backend.rs"><storycode:paste select="#f-806bbf2d45fa-BACKENDS"/><storycode:paste select="#f-806bbf2d45fa-canonical_language"/><storycode:paste select="#f-806bbf2d45fa-backend_for"/><storycode:paste select="#f-806bbf2d45fa-backend_dir"/><storycode:paste select="#f-806bbf2d45fa-find_interpreter"/><storycode:paste select="#f-806bbf2d45fa-install"/><storycode:paste select="#f-806bbf2d45fa-installed_languages"/><storycode:paste select="#f-806bbf2d45fa-ensure"/><storycode:paste select="#f-806bbf2d45fa-tests"/></storycode:file>

<a id="output-b35ccb039dc1"></a>
## `crates/hick-formula/src/graph.rs`

Related contracts: [A Formula Is An Expression In A Real Language](contract.md).

<a id="f-b35ccb039dc1-CellRef"></a>
### `CellRef`

Cell references, dependencies, and the order things are evaluated in.
All of this is the HOST's, and deliberately so. It is the part that must
be identical whatever language a document's formulas are written in — a
table mixing a Python cell and a JavaScript one has to evaluate in one
order, and two backends each sorting their own half could not agree on
one. It is also the part that is easy to get subtly wrong, which is
exactly the work not to ask of every new backend.
A1 notation, because it is the notation every person who has used a
spreadsheet already knows, and because it is unambiguous in a way
`row 3, column 2` is not when somebody inserts a row.

<storycode:copy id="f-b35ccb039dc1-CellRef">
//! Cell references, dependencies, and the order things are evaluated in.
//!
//! All of this is the HOST's, and deliberately so. It is the part that must
//! be identical whatever language a document's formulas are written in — a
//! table mixing a Python cell and a JavaScript one has to evaluate in one
//! order, and two backends each sorting their own half could not agree on
//! one. It is also the part that is easy to get subtly wrong, which is
//! exactly the work not to ask of every new backend.
//!
//! A1 notation, because it is the notation every person who has used a
//! spreadsheet already knows, and because it is unambiguous in a way
//! `row 3, column 2` is not when somebody inserts a row.

use std::collections::{BTreeMap, BTreeSet, HashMap};

/// One cell, by column and row, both zero-based internally.
///
/// Zero-based inside and one-based on the wire: `A1` is `(0, 0)`. Doing the
/// conversion once, here, is what keeps every other file free of `+ 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellRef {
    pub column: usize,
    pub row: usize,
}

</storycode:copy>
<a id="f-b35ccb039dc1-impl-CellRef"></a>
### `impl-CellRef`

<storycode:copy id="f-b35ccb039dc1-impl-CellRef">
impl CellRef {
    pub fn new(column: usize, row: usize) -> Self {
        Self { column, row }
    }

    /// Parse `A1`, `AB12`. Case-insensitive; `$` is accepted and ignored,
    /// because people paste absolute references out of other spreadsheets and
    /// being pedantic about it helps nobody.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let mut column = 0usize;
        let mut letters = 0usize;
        let mut chars = text.chars().peekable();
        if chars.peek() == Some(&'$') {
            chars.next();
        }
        while let Some(&c) = chars.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            chars.next();
            letters += 1;
            // Bijective base-26: A=1..Z=26, then AA=27. Not plain base-26 —
            // there is no zero digit, which is why `Z` is followed by `AA`.
            column = column * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
        }
        // Two letters, not three, and the reason is the lexical scan in
        // `references_in` rather than arithmetic. `ZZ` is 702 columns, which
        // is far more than any table anybody edits by hand — and stopping
        // there means `myA1` and `sum1` are identifiers rather than
        // references to columns `MYA` and `SUM`. Allowing a third letter
        // buys columns nobody has at the cost of shadowing variables
        // everybody writes.
        if letters == 0 || letters > 2 {
            return None;
        }
        if chars.peek() == Some(&'$') {
            chars.next();
        }
        let digits: String = chars.collect();
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let row: usize = digits.parse().ok()?;
        if row == 0 {
            return None;
        }
        Some(Self::new(column - 1, row - 1))
    }

    /// The A1 spelling, which is what a backend sees as a variable name.
    pub fn label(&self) -> String {
        let mut letters = Vec::new();
        let mut n = self.column + 1;
        while n > 0 {
            let rem = (n - 1) % 26;
            letters.push((b'A' + rem as u8) as char);
            n = (n - 1) / 26;
        }
        letters.reverse();
        format!(
            "{}{}",
            letters.into_iter().collect::<String>(),
            self.row + 1
        )
    }
}

</storycode:copy>
<a id="f-b35ccb039dc1-Sheet"></a>
### `Sheet`

A table's cells, by reference. Only the cells that have something in them.

<storycode:copy id="f-b35ccb039dc1-Sheet">
/// A table's cells, by reference. Only the cells that have something in them.
pub type Sheet = BTreeMap<CellRef, String>;

</storycode:copy>
<a id="f-b35ccb039dc1-CellRange"></a>
### `CellRange`

One rectangular range in a formula. Its name is deliberately a legal
identifier: the host replaces `B2:B4` with it before either backend sees
the expression, while the CSV continues to show familiar A1 notation.

<storycode:copy id="f-b35ccb039dc1-CellRange">
/// One rectangular range in a formula. Its name is deliberately a legal
/// identifier: the host replaces `B2:B4` with it before either backend sees
/// the expression, while the CSV continues to show familiar A1 notation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRange {
    pub cells: Vec<CellRef>,
    pub name: String,
}

</storycode:copy>
<a id="f-b35ccb039dc1-is_formula"></a>
### `is_formula`

A formula is a cell whose text starts with `=`, exactly as in every
spreadsheet. Everything else is a literal value.

<storycode:copy id="f-b35ccb039dc1-is_formula">
/// A formula is a cell whose text starts with `=`, exactly as in every
/// spreadsheet. Everything else is a literal value.
pub fn is_formula(text: &str) -> bool {
    text.trim_start().starts_with('=')
}

</storycode:copy>
<a id="f-b35ccb039dc1-expression_of"></a>
### `expression_of`

The expression part of a formula cell — everything after the `=`.

<storycode:copy id="f-b35ccb039dc1-expression_of">
/// The expression part of a formula cell — everything after the `=`.
pub fn expression_of(text: &str) -> &str {
    text.trim_start().strip_prefix('=').unwrap_or(text).trim()
}

</storycode:copy>
<a id="f-b35ccb039dc1-references_in"></a>
### `references_in`

Every cell reference an expression mentions, in the order they appear.
Deliberately lexical rather than parsed. The expression is in a language
this crate does not know — it cannot tell a reference from a variable by
understanding the code, and pretending otherwise would mean writing a
parser per language. So the rule is a shape: a run of letters followed by
digits, not attached to anything else.
The cost is honest and worth stating: a Python cell using a local variable
literally called `A1` would have it treated as a reference. The benefit is
that one rule serves every language, and the rule is the one people
already expect from a spreadsheet.

<storycode:copy id="f-b35ccb039dc1-references_in">
/// Every cell reference an expression mentions, in the order they appear.
///
/// Deliberately lexical rather than parsed. The expression is in a language
/// this crate does not know — it cannot tell a reference from a variable by
/// understanding the code, and pretending otherwise would mean writing a
/// parser per language. So the rule is a shape: a run of letters followed by
/// digits, not attached to anything else.
///
/// The cost is honest and worth stating: a Python cell using a local variable
/// literally called `A1` would have it treated as a reference. The benefit is
/// that one rule serves every language, and the rule is the one people
/// already expect from a spreadsheet.
pub fn references_in(expression: &str) -> Vec<CellRef> {
    let bytes: Vec<char> = expression.chars().collect();
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut i = 0usize;
    for range in ranges_in(expression) {
        for cell in range.cells {
            if seen.insert(cell) {
                out.push(cell);
            }
        }
    }
    while i < bytes.len() {
        // A reference cannot begin part-way through a word, or `myA1` would
        // contain one.
        let boundary = i == 0 || !(bytes[i - 1].is_alphanumeric() || bytes[i - 1] == '_');
        if !boundary || !bytes[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
            i += 1;
        }
        let digits_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        // ...and it cannot run INTO a word either: `A1b` is an identifier.
        let ends_cleanly = i >= bytes.len() || !(bytes[i].is_alphanumeric() || bytes[i] == '_');
        if digits_start > start && i > digits_start && ends_cleanly {
            let text: String = bytes[start..i].iter().collect();
            if let Some(cell) = CellRef::parse(&text)
                && seen.insert(cell)
            {
                out.push(cell);
            }
        }
    }
    out
}

</storycode:copy>
<a id="f-b35ccb039dc1-ranges_in"></a>
### `ranges_in`

Every `A1:B4` range in an expression, in source order.

<storycode:copy id="f-b35ccb039dc1-ranges_in">
/// Every `A1:B4` range in an expression, in source order.
pub fn ranges_in(expression: &str) -> Vec<CellRange> {
    let bytes = expression.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let boundary = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
        if !boundary || !bytes[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let Some((first, first_end)) = cell_at(bytes, i) else {
            i += 1;
            continue;
        };
        if first_end >= bytes.len() || bytes[first_end] != b':' {
            i = first_end;
            continue;
        }
        let Some((last, end)) = cell_at(bytes, first_end + 1) else {
            i = first_end + 1;
            continue;
        };
        if end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            i = end;
            continue;
        }
        let mut cells = Vec::new();
        for row in first.row.min(last.row)..=first.row.max(last.row) {
            for column in first.column.min(last.column)..=first.column.max(last.column) {
                cells.push(CellRef::new(column, row));
            }
        }
        out.push(CellRange {
            cells,
            name: format!("__hick_range_{}_{}", first.label(), last.label()),
        });
        i = end;
    }
    out
}

</storycode:copy>
<a id="f-b35ccb039dc1-cell_at"></a>
### `cell_at`

<storycode:copy id="f-b35ccb039dc1-cell_at">
fn cell_at(bytes: &[u8], start: usize) -> Option<(CellRef, usize)> {
    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
        end += 1;
    }
    let letters_end = end;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if letters_end == start || end == letters_end {
        return None;
    }
    let text = std::str::from_utf8(&bytes[start..end]).ok()?;
    Some((CellRef::parse(text)?, end))
}

</storycode:copy>
<a id="f-b35ccb039dc1-Cycle"></a>
### `Cycle`

<storycode:copy id="f-b35ccb039dc1-Cycle">
/// Why an order could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    /// The cells caught in it, sorted, so the message is stable.
    pub cells: Vec<CellRef>,
}

</storycode:copy>
<a id="f-b35ccb039dc1-impl-Cycle"></a>
### `impl-Cycle`

<storycode:copy id="f-b35ccb039dc1-impl-Cycle">
impl Cycle {
    /// What a person is told. Names the cells, because "circular reference"
    /// without saying where is the single most useless error a spreadsheet
    /// produces.
    pub fn message(&self) -> String {
        let names: Vec<String> = self.cells.iter().map(CellRef::label).collect();
        format!(
            "these cells depend on each other in a circle: {}",
            names.join(" → ")
        )
    }
}

</storycode:copy>
<a id="f-b35ccb039dc1-evaluation_order"></a>
### `evaluation_order`

The order formula cells must be evaluated in, or the cycle that prevents
one.
A depth-first topological sort over the formula cells only: a literal has
no dependencies and never needs ordering. References to cells outside the
sheet, or to literals, are fine — they resolve to a value directly.

<storycode:copy id="f-b35ccb039dc1-evaluation_order">
/// The order formula cells must be evaluated in, or the cycle that prevents
/// one.
///
/// A depth-first topological sort over the formula cells only: a literal has
/// no dependencies and never needs ordering. References to cells outside the
/// sheet, or to literals, are fine — they resolve to a value directly.
pub fn evaluation_order(sheet: &Sheet) -> Result<Vec<CellRef>, Cycle> {
    let formulas: BTreeMap<CellRef, Vec<CellRef>> = sheet
        .iter()
        .filter(|(_, text)| is_formula(text))
        .map(|(cell, text)| (*cell, references_in(expression_of(text))))
        .collect();

    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        Open,
        Done,
    }
    let mut marks: HashMap<CellRef, Mark> = HashMap::new();
    let mut order = Vec::new();
    // An explicit stack rather than recursion: a column of ten thousand
    // formulas each referring to the one above is a legitimate sheet, and it
    // must not blow the stack.
    for &root in formulas.keys() {
        if marks.contains_key(&root) {
            continue;
        }
        let mut stack = vec![(root, 0usize)];
        while let Some((cell, index)) = stack.pop() {
            if index == 0 {
                match marks.get(&cell) {
                    Some(Mark::Done) => continue,
                    Some(Mark::Open) => {
                        // Reached a cell already on this path: the cycle is
                        // everything still open.
                        let mut cells: Vec<CellRef> = stack
                            .iter()
                            .map(|(c, _)| *c)
                            .filter(|c| marks.get(c) == Some(&Mark::Open))
                            .collect();
                        cells.push(cell);
                        cells.sort();
                        cells.dedup();
                        return Err(Cycle { cells });
                    }
                    None => {
                        marks.insert(cell, Mark::Open);
                    }
                }
            }
            let deps = formulas.get(&cell).map(Vec::as_slice).unwrap_or(&[]);
            // Only a dependency that is ITSELF a formula needs ordering.
            let next = deps
                .iter()
                .skip(index)
                .position(|d| formulas.contains_key(d));
            match next {
                Some(offset) => {
                    let at = index + offset;
                    stack.push((cell, at + 1));
                    stack.push((deps[at], 0));
                }
                None => {
                    marks.insert(cell, Mark::Done);
                    order.push(cell);
                }
            }
        }
    }
    Ok(order)
}

</storycode:copy>
<a id="f-b35ccb039dc1-tests"></a>
### `tests`

<storycode:copy id="f-b35ccb039dc1-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn cell(text: &str) -> CellRef {
        CellRef::parse(text).unwrap_or_else(|| panic!("{text} parses"))
    }

    fn sheet(entries: &[(&str, &str)]) -> Sheet {
        entries
            .iter()
            .map(|(at, text)| (cell(at), text.to_string()))
            .collect()
    }

    #[test]
    fn a1_notation_round_trips() {
        for text in ["A1", "B2", "Z9", "AA1", "AZ26", "BA100", "ZZ702"] {
            assert_eq!(cell(text).label(), text, "{text}");
        }
    }

    #[test]
    fn columns_are_bijective_base_26_not_base_26() {
        // There is no zero digit, which is why Z is followed by AA. Plain
        // base-26 would make AA the 27th column only by accident and Aa the
        // 27th too.
        assert_eq!(cell("A1").column, 0);
        assert_eq!(cell("Z1").column, 25);
        assert_eq!(cell("AA1").column, 26);
        assert_eq!(cell("AB1").column, 27);
    }

    #[test]
    fn absolute_references_pasted_from_elsewhere_are_accepted() {
        // People paste `$B$4` out of other spreadsheets; being pedantic about
        // it helps nobody.
        assert_eq!(CellRef::parse("$B$4"), CellRef::parse("B4"));
        assert_eq!(CellRef::parse("b4"), CellRef::parse("B4"));
    }

    #[test]
    fn nonsense_is_not_a_reference() {
        for text in ["", "1", "A", "A0", "1A", "AAA1", "AAAA1", "A1.5"] {
            assert!(CellRef::parse(text).is_none(), "{text} must not parse");
        }
    }

    #[test]
    fn a_formula_is_a_cell_beginning_with_equals() {
        assert!(is_formula("=A1+1"));
        assert!(is_formula("  =A1"));
        assert!(!is_formula("A1+1"));
        // `==` IS a formula; its expression happens to be `=`, which the
        // backend will reject in its own words.
        assert!(is_formula("=="));
        assert_eq!(expression_of("=="), "=");
        assert_eq!(expression_of("  = A1 + 1 "), "A1 + 1");
    }

    #[test]
    fn references_are_found_in_document_order_without_duplicates() {
        assert_eq!(references_in("A1 + B2 * A1"), vec![cell("A1"), cell("B2")]);
    }

    #[test]
    fn an_identifier_that_merely_contains_a_reference_is_not_one() {
        // `myA1` is excluded by the two-letter column cap: three letters is
        // an identifier, not a column nobody has. `A1b` and `_A1` are
        // excluded by the word boundaries.
        assert!(references_in("myA1").is_empty());
        assert!(references_in("A1b").is_empty());
        assert!(references_in("_A1").is_empty());
    }

    #[test]
    fn references_are_found_around_real_syntax() {
        assert_eq!(
            references_in("sum([A1, A2]) / (B1 - 1)"),
            vec![cell("A1"), cell("A2"), cell("B1")]
        );
    }

    #[test]
    fn a_range_depends_on_every_cell_inside_it() {
        assert_eq!(
            references_in("sum(B2:C3)"),
            vec![cell("B2"), cell("C2"), cell("B3"), cell("C3")]
        );
        assert_eq!(ranges_in("sum(B2:C3)")[0].name, "__hick_range_B2_C3");
    }

    #[test]
    fn a_sheet_of_literals_needs_no_order() {
        assert_eq!(
            evaluation_order(&sheet(&[("A1", "1"), ("B1", "2")])),
            Ok(vec![])
        );
    }

    #[test]
    fn dependencies_come_before_the_cells_that_use_them() {
        let order = evaluation_order(&sheet(&[("A1", "1"), ("B1", "=A1*2"), ("C1", "=B1+A1")]))
            .expect("no cycle");
        assert_eq!(order, vec![cell("B1"), cell("C1")]);
    }

    #[test]
    fn a_long_chain_is_ordered_without_recursing() {
        // A column of formulas each referring to the one above is a real
        // sheet, and it must not blow the stack.
        let mut entries: Vec<(String, String)> = vec![("A1".into(), "1".into())];
        for row in 2..=5_000 {
            entries.push((format!("A{row}"), format!("=A{}+1", row - 1)));
        }
        let s: Sheet = entries
            .iter()
            .map(|(at, text)| (cell(at), text.clone()))
            .collect();
        let order = evaluation_order(&s).expect("no cycle");
        assert_eq!(order.len(), 4_999);
        assert_eq!(order.first(), Some(&cell("A2")));
        assert_eq!(order.last(), Some(&cell("A5000")));
    }

    #[test]
    fn a_cycle_names_the_cells_in_it() {
        // "Circular reference" without saying where is the single most
        // useless error a spreadsheet produces.
        let err = evaluation_order(&sheet(&[("A1", "=B1"), ("B1", "=A1")])).unwrap_err();
        assert_eq!(err.cells, vec![cell("A1"), cell("B1")]);
        assert!(err.message().contains("A1"), "{}", err.message());
        assert!(err.message().contains("B1"), "{}", err.message());
    }

    #[test]
    fn a_cell_referring_to_itself_is_a_cycle() {
        let err = evaluation_order(&sheet(&[("A1", "=A1+1")])).unwrap_err();
        assert_eq!(err.cells, vec![cell("A1")]);
    }

    #[test]
    fn a_reference_to_a_literal_or_to_nothing_needs_no_ordering() {
        // Both resolve to a value directly — an empty cell is empty, which is
        // a value like any other.
        let order = evaluation_order(&sheet(&[("A1", "1"), ("B1", "=A1+Z99")])).expect("no cycle");
        assert_eq!(order, vec![cell("B1")]);
    }
}
</storycode:copy>

### Assemble `crates/hick-formula/src/graph.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-formula/src/graph.rs"><storycode:paste select="#f-b35ccb039dc1-CellRef"/><storycode:paste select="#f-b35ccb039dc1-impl-CellRef"/><storycode:paste select="#f-b35ccb039dc1-Sheet"/><storycode:paste select="#f-b35ccb039dc1-CellRange"/><storycode:paste select="#f-b35ccb039dc1-is_formula"/><storycode:paste select="#f-b35ccb039dc1-expression_of"/><storycode:paste select="#f-b35ccb039dc1-references_in"/><storycode:paste select="#f-b35ccb039dc1-ranges_in"/><storycode:paste select="#f-b35ccb039dc1-cell_at"/><storycode:paste select="#f-b35ccb039dc1-Cycle"/><storycode:paste select="#f-b35ccb039dc1-impl-Cycle"/><storycode:paste select="#f-b35ccb039dc1-evaluation_order"/><storycode:paste select="#f-b35ccb039dc1-tests"/></storycode:file>

<a id="output-95a1af1a8af6"></a>
## `crates/hick-formula/src/protocol.rs`

Related contracts: [A Formula Is An Expression In A Real Language](contract.md).

<a id="f-95a1af1a8af6-Value"></a>
### `Value`

The Formula Compute Protocol, on the wire.
Framed exactly like LSP and DAP — `Content-Length`, a blank line, one JSON
object — because a backend author almost certainly has an LSP or DAP
implementation to crib from, and because the framing is the one part of
those protocols nobody has ever wished were different.
Correlated by `id`, LSP-style, because a backend answers requests and
never starts a conversation of its own. There are no events here at all:
evaluating an expression either produces a value or fails, and a protocol
that left room for a backend to volunteer something would be leaving room
for a backend to hang the host.
# What is deliberately NOT in this protocol
No cell references, no sheet, no dependency order, no notion of "the table
changed". The host resolves references to values before it asks (see
`graph.rs`), so a backend never learns what a cell is. That is what keeps
a backend to sixty lines and what makes two languages in one table
evaluate in one order.

<storycode:copy id="f-95a1af1a8af6-Value">
//! The Formula Compute Protocol, on the wire.
//!
//! Framed exactly like LSP and DAP — `Content-Length`, a blank line, one JSON
//! object — because a backend author almost certainly has an LSP or DAP
//! implementation to crib from, and because the framing is the one part of
//! those protocols nobody has ever wished were different.
//!
//! Correlated by `id`, LSP-style, because a backend answers requests and
//! never starts a conversation of its own. There are no events here at all:
//! evaluating an expression either produces a value or fails, and a protocol
//! that left room for a backend to volunteer something would be leaving room
//! for a backend to hang the host.
//!
//! # What is deliberately NOT in this protocol
//!
//! No cell references, no sheet, no dependency order, no notion of "the table
//! changed". The host resolves references to values before it asks (see
//! `graph.rs`), so a backend never learns what a cell is. That is what keeps
//! a backend to sixty lines and what makes two languages in one table
//! evaluate in one order.

use serde::{Deserialize, Serialize};

/// A value that crosses the wire.
///
/// Deliberately narrow. A formula cell lives in a CSV file, and a CSV file
/// holds text — so anything a backend returns has to be renderable as text
/// without losing what it was. Numbers and booleans are called out separately
/// from strings only so a backend can format them the way its language would,
/// and so `=1+1` is `2` rather than `2.0` in one language and `2` in another.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Value {
    Number {
        value: f64,
    },
    Text {
        value: String,
    },
    Bool {
        value: bool,
    },
    /// A rectangular A1 range, in reading order. A range is a value because
    /// `sum(B2:B4)` is one operand in Python and JavaScript, not a bit of
    /// syntax either backend has to understand.
    List {
        value: Vec<Value>,
    },
    /// The cell is empty. Distinct from an empty string, because summing a
    /// column should skip blanks rather than treat them as zero-length text.
    Empty,
}

</storycode:copy>
<a id="f-95a1af1a8af6-impl-Value"></a>
### `impl-Value`

<storycode:copy id="f-95a1af1a8af6-impl-Value">
impl Value {
    /// How this value is written into the CSV.
    ///
    /// A whole number loses its `.0`: a table of counts full of `3.0` is a
    /// table that has been through a computer, and the point of writing back
    /// into a CSV is that a person still recognises their data.
    pub fn to_cell(&self) -> String {
        match self {
            Value::Number { value } => {
                if value.is_finite() && *value == value.trunc() && value.abs() < 1e15 {
                    format!("{}", *value as i64)
                } else {
                    format!("{value}")
                }
            }
            Value::Text { value } => value.clone(),
            Value::Bool { value } => value.to_string(),
            Value::List { value } => value
                .iter()
                .map(Value::to_cell)
                .collect::<Vec<_>>()
                .join(", "),
            Value::Empty => String::new(),
        }
    }

    /// Read a cell's text as a value, the way a spreadsheet does: a number if
    /// it looks like one, otherwise text.
    pub fn from_cell(text: &str) -> Self {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Value::Empty;
        }
        if let Ok(value) = trimmed.parse::<f64>() {
            return Value::Number { value };
        }
        match trimmed {
            "true" | "TRUE" => Value::Bool { value: true },
            "false" | "FALSE" => Value::Bool { value: false },
            _ => Value::Text {
                value: text.to_string(),
            },
        }
    }
}

</storycode:copy>
<a id="f-95a1af1a8af6-Formula"></a>
### `Formula`

One expression to evaluate, with its references already resolved.

<storycode:copy id="f-95a1af1a8af6-Formula">
/// One expression to evaluate, with its references already resolved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Formula {
    /// Opaque to the backend; the host uses it to match answers to cells.
    pub id: String,
    /// The expression, without the leading `=`.
    pub expression: String,
    /// The names the expression may use, and what they are worth. The host
    /// resolved these; a backend never resolves anything.
    pub bindings: Vec<(String, Value)>,
}

</storycode:copy>
<a id="f-95a1af1a8af6-EvalRequest"></a>
### `EvalRequest`

Everything to evaluate, in the order the host worked out.
Sent as one request rather than one per cell: a table with four hundred
formulas would otherwise be four hundred round trips, and the ORDER —
which is the host's whole contribution — would have to be re-established
on every one.

<storycode:copy id="f-95a1af1a8af6-EvalRequest">
/// Everything to evaluate, in the order the host worked out.
///
/// Sent as one request rather than one per cell: a table with four hundred
/// formulas would otherwise be four hundred round trips, and the ORDER —
/// which is the host's whole contribution — would have to be re-established
/// on every one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalRequest {
    pub formulas: Vec<Formula>,
}

</storycode:copy>
<a id="f-95a1af1a8af6-FormulaResult"></a>
### `FormulaResult`

<storycode:copy id="f-95a1af1a8af6-FormulaResult">
/// What one expression came to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaResult {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<FormulaError>,
}

</storycode:copy>
<a id="f-95a1af1a8af6-FormulaError"></a>
### `FormulaError`

Why an expression did not produce a value.
A message and nothing else. A backend reports what its own language said —
a Python `NameError`, a JavaScript `TypeError` — because that message is
the one the author can act on, and translating it into a spreadsheet's
`#VALUE!` would throw away the only useful part.

<storycode:copy id="f-95a1af1a8af6-FormulaError">
/// Why an expression did not produce a value.
///
/// A message and nothing else. A backend reports what its own language said —
/// a Python `NameError`, a JavaScript `TypeError` — because that message is
/// the one the author can act on, and translating it into a spreadsheet's
/// `#VALUE!` would throw away the only useful part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaError {
    pub message: String,
}

</storycode:copy>
<a id="f-95a1af1a8af6-EvalResponse"></a>
### `EvalResponse`

The answers, in any order; the host matches on `id`.

<storycode:copy id="f-95a1af1a8af6-EvalResponse">
/// The answers, in any order; the host matches on `id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalResponse {
    pub results: Vec<FormulaResult>,
}

</storycode:copy>
<a id="f-95a1af1a8af6-Request"></a>
### `Request`

A JSON-RPC envelope, as narrow as this protocol needs.

<storycode:copy id="f-95a1af1a8af6-Request">
/// A JSON-RPC envelope, as narrow as this protocol needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    pub params: serde_json::Value,
}

</storycode:copy>
<a id="f-95a1af1a8af6-Response"></a>
### `Response`

<storycode:copy id="f-95a1af1a8af6-Response">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<RpcError>,
}

</storycode:copy>
<a id="f-95a1af1a8af6-RpcError"></a>
### `RpcError`

<storycode:copy id="f-95a1af1a8af6-RpcError">
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

</storycode:copy>
<a id="f-95a1af1a8af6-frame"></a>
### `frame`

<storycode:copy id="f-95a1af1a8af6-frame">
/// Frame a message the way LSP does.
pub fn frame(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

</storycode:copy>
<a id="f-95a1af1a8af6-tests"></a>
### `tests`

<storycode:copy id="f-95a1af1a8af6-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whole_number_loses_its_decimal_point() {
        // A table of counts full of `3.0` is a table that has been through a
        // computer; the point of writing back into a CSV is that a person
        // still recognises their data.
        assert_eq!(Value::Number { value: 3.0 }.to_cell(), "3");
        assert_eq!(Value::Number { value: -12.0 }.to_cell(), "-12");
        assert_eq!(Value::Number { value: 2.5 }.to_cell(), "2.5");
    }

    #[test]
    fn a_huge_number_keeps_its_float_spelling_rather_than_wrapping() {
        let big = Value::Number { value: 1e18 };
        assert!(
            big.to_cell().contains('e') || big.to_cell().len() > 15,
            "{}",
            big.to_cell()
        );
    }

    #[test]
    fn an_empty_cell_is_not_an_empty_string() {
        // Summing a column should skip blanks rather than treat them as
        // zero-length text.
        assert_eq!(Value::from_cell(""), Value::Empty);
        assert_eq!(Value::from_cell("   "), Value::Empty);
        assert_eq!(Value::Empty.to_cell(), "");
    }

    #[test]
    fn a_cell_that_looks_like_a_number_is_one() {
        assert_eq!(Value::from_cell("42"), Value::Number { value: 42.0 });
        assert_eq!(Value::from_cell(" -3.5 "), Value::Number { value: -3.5 });
        assert_eq!(
            Value::from_cell("007"),
            Value::Number { value: 7.0 },
            "a spreadsheet reads a leading zero as a number too"
        );
    }

    #[test]
    fn anything_else_is_text_with_its_spacing_intact() {
        assert_eq!(
            Value::from_cell(" hello "),
            Value::Text {
                value: " hello ".to_string()
            }
        );
    }

    #[test]
    fn framing_counts_bytes_not_characters() {
        // A formula holding an em dash is one character and three bytes; a
        // reader counting characters would truncate the message.
        let body = "{\"a\":\"—\"}";
        assert!(frame(body).starts_with(&format!("Content-Length: {}", body.len())));
        assert_eq!(body.len(), 11, "nine characters, eleven bytes");
    }

    #[test]
    fn a_request_round_trips_through_json() {
        let request = EvalRequest {
            formulas: vec![Formula {
                id: "A1".into(),
                expression: "B1 * 2".into(),
                bindings: vec![("B1".into(), Value::Number { value: 21.0 })],
            }],
        };
        let text = serde_json::to_string(&request).unwrap();
        assert_eq!(serde_json::from_str::<EvalRequest>(&text).unwrap(), request);
    }

    #[test]
    fn a_result_carries_a_value_or_an_error_and_omits_the_other() {
        let ok = FormulaResult {
            id: "A1".into(),
            value: Some(Value::Number { value: 1.0 }),
            error: None,
        };
        let text = serde_json::to_string(&ok).unwrap();
        assert!(!text.contains("error"), "{text}");
    }
}
</storycode:copy>

### Assemble `crates/hick-formula/src/protocol.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-formula/src/protocol.rs"><storycode:paste select="#f-95a1af1a8af6-Value"/><storycode:paste select="#f-95a1af1a8af6-impl-Value"/><storycode:paste select="#f-95a1af1a8af6-Formula"/><storycode:paste select="#f-95a1af1a8af6-EvalRequest"/><storycode:paste select="#f-95a1af1a8af6-FormulaResult"/><storycode:paste select="#f-95a1af1a8af6-FormulaError"/><storycode:paste select="#f-95a1af1a8af6-EvalResponse"/><storycode:paste select="#f-95a1af1a8af6-Request"/><storycode:paste select="#f-95a1af1a8af6-Response"/><storycode:paste select="#f-95a1af1a8af6-RpcError"/><storycode:paste select="#f-95a1af1a8af6-frame"/><storycode:paste select="#f-95a1af1a8af6-tests"/></storycode:file>

<a id="output-17c37978c7de"></a>
## `crates/hick-formula/src/session.rs`

Related contracts: [A Formula Is An Expression In A Real Language](contract.md).

<a id="f-17c37978c7de-EVAL_TIMEOUT"></a>
### `EVAL_TIMEOUT`

A running backend: spawn it, ask it, stop it.
Short, because the protocol is. There are no events, no reverse requests,
and no long-lived conversation — a request goes out, an answer comes back,
and the only interesting engineering is making sure a backend that hangs,
crashes, or writes nonsense does not take the app with it.
Three rules do that:
* **Every request has a deadline.** A formula that never returns is a
plausible thing to write (`=while True: pass`, near enough), and a table
that stops responding because one cell looped is a table nobody trusts.
* **A crash is an answer.** If the process dies, every formula in the
batch gets an error saying so, rather than the caller waiting forever.
* **stderr is captured, not inherited.** A backend printing a warning must
not scribble on the terminal the app is running in.

<storycode:copy id="f-17c37978c7de-EVAL_TIMEOUT">
//! A running backend: spawn it, ask it, stop it.
//!
//! Short, because the protocol is. There are no events, no reverse requests,
//! and no long-lived conversation — a request goes out, an answer comes back,
//! and the only interesting engineering is making sure a backend that hangs,
//! crashes, or writes nonsense does not take the app with it.
//!
//! Three rules do that:
//!
//! * **Every request has a deadline.** A formula that never returns is a
//!   plausible thing to write (`=while True: pass`, near enough), and a table
//!   that stops responding because one cell looped is a table nobody trusts.
//! * **A crash is an answer.** If the process dies, every formula in the
//!   batch gets an error saying so, rather than the caller waiting forever.
//! * **stderr is captured, not inherited.** A backend printing a warning must
//!   not scribble on the terminal the app is running in.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::protocol::{
    EvalRequest, EvalResponse, Formula, FormulaError, FormulaResult, Request, Response, frame,
};

/// How long one batch may take before it is abandoned.
///
/// Generous, because a formula that reads a file or fits a model is a
/// legitimate thing to write in a real language; finite, because an infinite
/// loop is a legitimate thing to write by accident.
pub const EVAL_TIMEOUT: Duration = Duration::from_secs(30);

</storycode:copy>
<a id="f-17c37978c7de-Session"></a>
### `Session`

<storycode:copy id="f-17c37978c7de-Session">
/// A backend process, mid-conversation.
pub struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
    language: String,
}

</storycode:copy>
<a id="f-17c37978c7de-impl-Session"></a>
### `impl-Session`

<storycode:copy id="f-17c37978c7de-impl-Session">
impl Session {
    /// Start the backend for `language`, writing it into the project's cache
    /// first if it is not there. See `backend::ensure`.
    pub async fn start(root: &Path, language: &str) -> Result<Self> {
        let (interpreter, script) = crate::backend::ensure(root, language)?;
        let mut child = Command::new(&interpreter)
            .arg(&script)
            // The project root, so a formula that reads a relative path means
            // the same thing a cell in the same document would.
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Captured rather than inherited: a backend printing a warning
            // must not scribble on the terminal the app is running in.
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("could not start {interpreter} {}", script.display()))?;

        let stdin = child.stdin.take().context("the backend has no stdin")?;
        let stdout = BufReader::new(child.stdout.take().context("the backend has no stdout")?);
        let mut session = Self {
            child,
            stdin,
            stdout,
            next_id: 1,
            language: language.to_string(),
        };
        session
            .request("initialize", serde_json::json!({}))
            .await
            .with_context(|| format!("the {language} formula backend did not start"))?;
        Ok(session)
    }

    /// Evaluate a batch, in the order given.
    ///
    /// Order matters and is the host's: `graph.rs` worked it out, and a
    /// backend that reordered would break a table mixing two languages.
    pub async fn evaluate(&mut self, formulas: Vec<Formula>) -> Result<EvalResponse> {
        if formulas.is_empty() {
            return Ok(EvalResponse {
                results: Vec::new(),
            });
        }
        let ids: Vec<String> = formulas.iter().map(|f| f.id.clone()).collect();
        let params = serde_json::to_value(EvalRequest { formulas })?;
        match tokio::time::timeout(EVAL_TIMEOUT, self.request("evaluate", params)).await {
            Ok(Ok(value)) => Ok(serde_json::from_value(value)?),
            // A crash or a hang is an ANSWER: every formula in the batch gets
            // an error naming what happened, rather than the caller waiting
            // for a process that is never going to reply.
            Ok(Err(error)) => Ok(Self::all_failed(&ids, &format!("{error:#}"))),
            Err(_) => {
                let _ = self.child.start_kill();
                Ok(Self::all_failed(
                    &ids,
                    &format!(
                        "the {} formula backend did not answer within {}s and was stopped — \
                         a formula is probably looping",
                        self.language,
                        EVAL_TIMEOUT.as_secs()
                    ),
                ))
            }
        }
    }

    fn all_failed(ids: &[String], message: &str) -> EvalResponse {
        EvalResponse {
            results: ids
                .iter()
                .map(|id| FormulaResult {
                    id: id.clone(),
                    value: None,
                    error: Some(FormulaError {
                        message: message.to_string(),
                    }),
                })
                .collect(),
        }
    }

    /// Ask the backend to stop, then make sure it did.
    pub async fn shutdown(mut self) {
        let _ = self.request("shutdown", serde_json::json!({})).await;
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }

    async fn request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let id = self.next_id;
        self.next_id += 1;
        let body = serde_json::to_string(&Request {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        })?;
        self.stdin.write_all(frame(&body).as_bytes()).await?;
        self.stdin.flush().await?;

        loop {
            let response = self.read_response().await?;
            // Ignore anything that is not the answer to this request. There
            // should be nothing else, but a backend that logs a stray object
            // should be untidy rather than fatal.
            if response.id != Some(id) {
                continue;
            }
            if let Some(error) = response.error {
                bail!("{} (code {})", error.message, error.code);
            }
            return Ok(response.result.unwrap_or(serde_json::Value::Null));
        }
    }

    async fn read_response(&mut self) -> Result<Response> {
        let mut length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line).await?;
            if read == 0 {
                bail!("the {} formula backend closed its output", self.language);
            }
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }
            if let Some(rest) = trimmed.to_ascii_lowercase().strip_prefix("content-length:") {
                length = rest.trim().parse().ok();
            }
        }
        let length = length.context("a backend message arrived with no Content-Length")?;
        let mut body = vec![0u8; length];
        self.stdout.read_exact(&mut body).await?;
        Ok(serde_json::from_slice(&body)?)
    }
}
</storycode:copy>

### Assemble `crates/hick-formula/src/session.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-formula/src/session.rs"><storycode:paste select="#f-17c37978c7de-EVAL_TIMEOUT"/><storycode:paste select="#f-17c37978c7de-Session"/><storycode:paste select="#f-17c37978c7de-impl-Session"/></storycode:file>

## Test the behavior: `have`

This is the maintained test fragment from `crates/hick-formula/tests/backends.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-6d63eb1fec85-have"></a>
### `have`

The backends, actually spawned.
Protects docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md
Everything else in this crate is testable without a process. This file is
the part that is not: that the script we embed really is a program the
machine's own interpreter can run, that it speaks the framing, and that it
answers in this protocol's shapes. A backend that parses in principle and
deadlocks in practice is the failure worth catching here.
Skipped, loudly, on a machine without the interpreter. A CI runner with no
node must not turn a missing dependency into a red build for a feature
that is designed to be absent gracefully.

<storycode:copy id="f-6d63eb1fec85-have">
//! The backends, actually spawned.
//!
//! Protects docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md
//!
//! Everything else in this crate is testable without a process. This file is
//! the part that is not: that the script we embed really is a program the
//! machine's own interpreter can run, that it speaks the framing, and that it
//! answers in this protocol's shapes. A backend that parses in principle and
//! deadlocks in practice is the failure worth catching here.
//!
//! Skipped, loudly, on a machine without the interpreter. A CI runner with no
//! node must not turn a missing dependency into a red build for a feature
//! that is designed to be absent gracefully.

use hick_formula::{Formula, Session, Value};

fn have(interpreters: &[&str]) -> bool {
    interpreters.iter().any(|name| {
        std::process::Command::new(name)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/FormulaDebugger.tsx](../../../mechanisms/apps/web/src/components/FormulaDebugger.tsx.md#output-3a3b3fc26b23)
- [apps/web/src/components/TablePanel.tsx](../../../mechanisms/apps/web/src/components/TablePanel.tsx.md#output-38d5395d7736)
- [apps/web/src/lib/cellRef.ts](contract.md#output-3de309f900ec)
- [apps/web/src/lib/formulaPoint.ts](contract.md#output-b60b27abb53f)
- [crates/hick-formula/src/backend.rs](contract.md#output-806bbf2d45fa)
- [crates/hick-formula/src/evaluate.rs](../../../mechanisms/crates/hick-formula/src/evaluate.rs.md#output-61e8dc986852)
- [crates/hick-formula/src/graph.rs](contract.md#output-b35ccb039dc1)
- [crates/hick-formula/src/protocol.rs](contract.md#output-95a1af1a8af6)
- [crates/hick-formula/src/session.rs](contract.md#output-17c37978c7de)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/serve/formula.rs](../../../mechanisms/crates/hickory-cli/src/serve/formula.rs.md#output-c63d0db349cf)

### Tests and fixtures

- [apps/web/src/components/FormulaDebugger.test.tsx](../../../mechanisms/apps/web/src/components/FormulaDebugger.tsx.md#output-1acac206a244)
- [apps/web/src/components/TablePanel.test.tsx](../../../mechanisms/apps/web/src/components/TablePanel.test.tsx.md#output-694220ace684)
- [apps/web/src/lib/formulaPoint.test.ts](contract.md#output-932c1cfa22d9)
- [crates/hick-formula/tests/backends.rs](../../../mechanisms/crates/hick-formula/tests/backends.rs.md#output-6d63eb1fec85)
- [crates/hickory-cli/tests/serve_formula.rs](../../../mechanisms/crates/hickory-cli/tests/serve_formula.rs.md#output-6af46f552231)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
