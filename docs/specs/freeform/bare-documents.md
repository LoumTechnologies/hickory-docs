# Bare documents: the wrapper is optional, the markdown is not

*Status: design of record for the surface syntax of a `.hick` document.
Adopted 2026-08-18. Additive to the language — every document that parses today
parses unchanged, with one deliberate behaviour change named under "The weave
default" below. Does not touch the no-escaping invariant, which is what makes
this cheap.*

Today the smallest possible document is:

```
<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="standup.md">
# Standup, Tuesday
</hick:doc>
```

Four lines of ceremony around one line of note. That is defensible for a
literate-programming pipeline, where a document is a build artifact somebody
sat down to write. It is indefensible for a **notes IDE** (`notes-ide.md`),
where the file is a thing you open on a phone in a meeting, and where the first
impression of the format is the first line of the file.

So: **a `.hick` file may begin with markdown.**

```
# Standup, Tuesday

Nothing blocking. <hick:copy id="numbers">42</hick:copy>
```

That is a complete document. It weaves to `standup.md`.

## The rules

**1. The root element is optional.** If the prologue does not open a
`PREFIX:doc` element, the entire file is the body of an implicit one. There is
no root tag to open and therefore none to close — "the little bit of XML at the
top and bottom" goes at both ends or neither.

**2. The prefix defaults to `hick`.** This is already what the parser does —
`detect_prefix` in `crates/hick-lang/src/lib.rs` scans for
`xmlns:PREFIX="http://www.hickorydocs.com/1.0"` and falls back to `"hick"` when
there is none. A bare document simply never declares one.

The consequence is worth stating because `AGENTS.md` depends on it: **rebinding
the prefix requires the explicit root.** Documentation *about* hick uses `h:`
so that `hick:` examples stay literal text, and those documents keep their
`<h:doc xmlns:h="…">` wrapper. That is the right split — the file that needs to
talk about the syntax is exactly the file that can afford four lines of it.

**3. Detection reads the prologue, not the file.** Today the parser searches
the whole input for `<PREFIX:doc`. Once prose can appear before the root, that
search will find the string `<hick:doc` inside a sentence about hick and try to
parse the paragraph as a root tag. So the decision becomes: skip an optional
XML declaration, comments, and frontmatter; if the first construct after them
opens `PREFIX:doc`, the document is wrapped; otherwise it is bare, and
`MissingRoot` stops being reachable from a bare file. A wrapped document that
does not close its root is still an error — the ambiguity is only about whether
a root was ever opened.

**4. `weave` defaults to the document's own name.** `notes/standup.hick` weaves
`notes/standup.md`. An explicit `weave=` still wins, and `weave="none"` opts
out.

## The weave default is the one behaviour change

Today `weave_path` is `Option`, and a document with no `weave=` attribute emits
no markdown. Defaulting it means **existing wrapped documents that deliberately
omit `weave=` will start writing a `.md` beside themselves** — a new file
appearing in someone's repository, and a new file that `hick test` will then
check for drift.

Per `pre-launch.md`, changing a document format's behaviour is allowed but is
"worth a deliberate decision rather than a side effect." This is that decision,
and it goes the way the request asked: *always* output a markdown file of the
same name, because a note that has no readable form is not a note. The escape
hatch is `weave="none"` for a document that is genuinely only a generator of
other files.

## YAML frontmatter: yes, narrowly

Frontmatter earns its place here for a reason specific to notes rather than to
documents in general: meetings have attendees, dates, and tags, and every tool
a user might also point at that folder — Obsidian, a static site generator,
`grep` — already agrees on where that metadata lives.

```
---
date: 2026-08-18
attendees: [nate, sam]
tags: [standup]
weave: notes/standup.md
---

# Standup, Tuesday
```

Three rules keep it from becoming a second configuration system:

- **Bare documents only.** A wrapped document configures itself with attributes
  on its root. Two mechanisms for one thing in one file is how a format rots.
- **Reserved keys are a closed, typed set** — `weave`, `prefix`, `volatile` —
  deserialized and validated, with an error naming the key and what a valid
  value looks like. This is `config-and-environments`' rule applied to the
  document format, and for the same reason: it fails on someone else's machine
  where nobody can debug it for them.
- **Every other key is metadata, and is never interpreted.** It is preserved,
  exposed to search and to selectors, and otherwise left alone. The format does
  not get an opinion about what `attendees` means.

**The block weaves through verbatim**, reserved keys included. The alternative —
stripping `weave:` and `prefix:` on the way out — means the woven markdown is
not a faithful rendering of its source, and it means the reverse-edit path in
`hick up` has to reconstruct keys it deleted in order to carry an edit back. A
stray `weave:` in a note's frontmatter is harmless; a lossy round-trip is not.

## The `---` ambiguity, which is accepted rather than solved

A bare document whose first line is `---` is ambiguous: markdown says
horizontal rule, frontmatter says opening fence. Every tool in this space has
the same problem and resolves it the same way, so this one does too — it is
frontmatter only when the first line is exactly `---`, a closing `---` exists,
and what lies between parses as a YAML mapping. Otherwise it is content.

This is a real, if narrow, way to write a document that means something other
than what it looks like. It is named here so that the bug report, when it
arrives, is met with a decision instead of a surprise.

## What this makes easy downstream

- **`hick adopt` on a markdown file becomes nearly a rename.** Adoption is
  byte-exact or refused (`docs/guarantees/authoring/adoption-is-byte-exact-or-refused.md`);
  with a bare document, the content that has to reproduce byte-exactly is the
  file itself with nothing wrapped around it.
- **A note is a markdown file that gained powers**, which is the sentence the
  notes IDE has to be able to say. Someone can put their existing notes folder
  under `hick up` and have every file still be the file they wrote.
- **Ingested meeting notes look like notes.** A transcript ingested per
  `notes-ide.md` produces frontmatter (date, attendees, source), a heading, a
  `hick:copy` holding the raw transcript, and `hick:transform` passages for the
  summary and the action items — and the first line a human sees is the date,
  not an XML declaration.

## Open edges

- **Round-tripping frontmatter through the reverse-edit path** is unproven.
  `hick up` carries an edit made in `standup.md` back into `standup.hick`; an
  edit made *inside the woven frontmatter block* is a case that has never
  existed before and needs a decision — most likely the same one generated
  regions get.
- **Whether `hick:doc` should be spellable at all in a bare document** is
  unresolved. Rule 3 makes it safe to mention in prose, but a bare document
  that opens a nested `hick:doc` halfway through is a shape nobody has thought
  about.
- **The XML declaration in existing examples becomes noise.** `examples/` all
  carry the full wrapper. They should not all be rewritten reflexively — some
  of them exist to demonstrate the explicit form — but the ones that are simply
  showing a feature should lead with markdown.
