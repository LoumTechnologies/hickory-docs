# A Fence Becomes A Cell That Runs

Given a `.hick` document whose prose contains a closed fenced code block, when
its rail icon is clicked, then a panel shows the exact `<hick:exec>` that
would replace it — the container it will run in, and a body derived from the
fence's language — and confirming replaces exactly the fence, leaving every
byte around it untouched.

A markdown fence is a claim about a command that nobody checked. Closing the
distance between "the code is written down" and "the code is verified" is the
whole product, and for a fence that distance is one mechanical step: the text
is already there, it is simply not wired to anything.

Three properties make the conversion trustworthy:

1. **It is shown before it happens.** The panel previews the bytes, the same
   rule the Insert menu follows: this is a text format a person owns, and a
   conversion you cannot see is one you cannot check.
2. **The language decides the body, and an unknown language is admitted.**
   `<hick:exec>` runs a shell. A fence tagged `bash` (or untagged) transfers
   verbatim. A fence tagged with a language whose interpreter reads standard
   input becomes `python3 - <<'EOF' … EOF` — the idiom the bundled examples
   already use — with the delimiter QUOTED, so the shell cannot expand `$` and
   backticks in the program before the interpreter sees it, and stepped aside
   to `EOF2` when the program itself contains a bare `EOF` line. A language
   with no known interpreter is carried across unchanged with a note saying
   so, because a wrong guess that looks right is worse than an honest
   hand-off.
3. **Only prose fences are offered.** Three backticks inside a `hick:file`
   body are content of a generated file; inside an exec they are part of a
   command. Neither gets an icon. An unterminated fence gets none either —
   guessing where it ends would rewrite text nobody pointed at.

The container chooser lists every container the document uses, not only the
ones `<hick:container>` declares: naming one on an exec (with `image=` on the
first) creates it, and telling a document with an implicit container that it
has none is both wrong and unhelpful when the name is right there in the
source.

## Boundary

The conversion is a text substitution, and it is deliberately not clever. It
does not check that the interpreter exists (that is `<hick:needs>`), does not
add a container declaration when you name one that does not exist yet, and
does not add an `<hick:expect>` — capturing what the cell should print is a
judgement about the output, which nobody has seen yet.

The whole replacement is a single transaction carrying a `userEvent`, so one
Ctrl+Z takes it back. Nothing is written to disk by the conversion itself; the
document's normal save path does that.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change; driven end to
  end in a browser — a `python` fence became a heredoc exec cell with the
  prose around it unchanged)
- Evidence: `apps/web/src/lib/fenceToExec.ts` (`convertFence`,
  `heredocDelimiter`, the shell/interpreter tables and the unknown-language
  note), `apps/web/src/editor/hickDoc.ts::proseFences` (closed fences outside
  verbatim block content) and `::containerNamesOf` (declared, implicit, and
  forked containers), `apps/web/src/components/FenceConvert.tsx` (the panel;
  the preview is built with `renderElement` from the Insert catalogue, so the
  two features cannot disagree about how an exec is spelled),
  `apps/web/src/editor/DocumentEditor.tsx::convertFenceCard` (the single
  `userEvent` transaction over `[card.from, card.to]`).
- Test coverage: `apps/web/src/lib/fenceToExec.test.ts` (11 tests: language
  parsing, verbatim shell, heredoc form, quoted delimiter, delimiter
  collision including a program containing `EOF`, unknown-language note);
  `apps/web/src/editor/cards.test.ts` (which fences are offered — prose only,
  closed only, tilde vs backtick, empty fence; and the container list);
  `apps/web/src/components/FenceConvert.test.tsx` (7 tests: preview matches
  what is handed back, container choice drives it, the no-container case
  blocks Convert, the unknown-language note, cancel);
  `apps/web/src/editor/DocumentEditor.test.tsx` — "turns a prose fence into an
  exec cell" asserts the fence is gone, the program survived, the surrounding
  prose is byte-identical, and the rail swapped a fence icon for a cell icon.
- Caveat requiring review: the interpreter table is a judgement call about
  which commands genuinely read a program on stdin. It was written from the
  documented behaviour of each interpreter, not verified by running them —
  an entry that actually opened a REPL would hang the cell until its timeout.
  Only the `python` path has been executed end to end, and only in the mock
  environment, where no cell really runs.
