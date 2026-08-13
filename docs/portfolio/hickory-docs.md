# Hickory Docs — portfolio write-up

Source material for nloum.github.io and a resume. Written to be pasted and
trimmed, not published as-is.

---

## One-liner

**Hickory Docs** — a document format and toolchain where the documentation is
the test suite. Every command in a `.hick` file actually runs, every output
shown is the output produced, and drift between the docs and reality fails the
build.

## Short paragraph (portfolio card)

Hickory Docs is an executable-document toolchain written in Rust. A `.hick`
file is prose, a program, and a test suite in one artifact: commands run in
declared containers, their real transcripts are woven into the published
Markdown, and `hick test` fails with a line number and a diff the moment
the tool's behaviour stops matching what the document claims. It ships with a
byte-precise provenance system that maps every character of generated output
back to the prose that produced it, a language server that multiplexes real
language servers into embedded code blocks, and an AI agent whose entire
session is saved as a replayable document rather than an evaporating chat log.

## Longer write-up (project page)

### The problem

Documentation rots silently. A flag is renamed, an output format changes, and
nobody notices until a user does. The usual answer is discipline; the actual
answer is verification.

Hickory treats a document as a build artifact with a dependency graph.
Containers, volumes, and copy/paste references define the order — not the
order the prose happens to be written in. Commands execute, transcripts are
captured, expectations are checked, and the woven Markdown contains the output
that really happened.

### What's interesting technically

**A parser with a no-escaping invariant.** Only tags carrying the `hick:`
namespace prefix are structure; every other byte is raw text. Shell heredocs,
Rust generics, and regexes paste in verbatim — no CDATA, no entity escaping.
That constraint is what makes the format usable for real code, and it is
enforced throughout the stack.

**Byte-precise provenance.** Generated files carry a lineage map from every
output byte back to the source span that produced it, which makes reverse
editing possible: edit the generated code in the web editor and the change is
routed back to the prose fragment it came from, or refused with a structured
explanation when the range is synthetic.

**Two different verification claims, deliberately separated.** `hick:expect`
asks "did this behave as claimed" — enforced always. Drift checking asks "do
these bytes reproduce" — meaningful only when the inputs are fixed, so a
document over live data can mark its report `volatile` without weakening its
behavioural assertions.

**LLM-written prose that can still be verified.** A `hick:transform` block
records a fingerprint over its exact input and instruction. The document does
not claim the passage reproduces — an LLM wrote it — only that it was written
from exactly those bytes under exactly that instruction. Checking that claim is
a hash comparison: free, offline, and deterministic in CI, which is what lets
generated prose live inside a verified document at all.

**A document pipeline.** `hick:upstream` lets a document declare its place in a
chain. A requirements document declares one edge to a domain model and reaches
decisions recorded in a meeting note two hops up without naming it. Change a
sentence in the meeting and every document downstream fails together.

### Scale

Rust workspace, 23 crates, ~870 tests, zero clippy warnings. React + TypeScript
front end shared between web and Tauri (iOS/Android). Yrs/Yjs CRDTs over
WebSocket for live collaboration; git for durable history.

### Licence and shape

GPL-3.0-or-later. A hosted, collaborative platform — teams share documents,
execution, and their own LLM keys — that deploys to Fly.io from the repository
and can be self-hosted in full, since the whole stack is in the same repo under
the same licence.

---

## Resume bullets

Pick two or three; they are written to stand alone.

- Designed and built **Hickory Docs**, an executable-document toolchain in Rust
  (23-crate workspace, ~870 tests): documentation whose examples are verified
  by execution, with byte-precise provenance from generated output back to
  source prose.

- Implemented a document pipeline in which requirements, domain models, and
  meeting notes form a dependency graph — editing one recorded decision fails
  every downstream document that depends on it, turning documentation drift
  into a build failure.

- Built a provider-agnostic AI agent harness (Anthropic, OpenAI, DeepSeek,
  Grok) whose sessions are saved as replayable literate-programming documents,
  with prompt-cache-stable request construction and four-way token accounting.

- Diagnosed and fixed a CRDT corruption bug that inflated a 15 KB document to
  50 MB: yrs indexes text in **bytes** while browsers index in **UTF-16**, so
  the reconcile path silently duplicated content on every non-ASCII edit.

- Shipped a language server that multiplexes real language servers (rust-analyzer,
  pyright) into code blocks embedded in documents, mapping positions between the
  document and per-language virtual files.

---

## Demo to show

`docs/todo-app/` is a complete lifecycle in five documents:

```
meetings/2026-08-07-kickoff.hick   decisions, written down once
        ↓ hick:upstream
domain.hick                        glossary + the decisions, by reference
        ↓ hick:upstream
requirements.hick                  requirements, tangled into work/*.task.md
        ↓ hick:upstream
implementation.hick                the code, plus acceptance cells that run it
        ↓ hick:file
todo.py                            the shipped program
```

Change one sentence in the meeting note and every document below it fails
`hick test` — including the code, because the acceptance cells stop
matching. That is the demo: it takes ten seconds and it is not a mockup.

An AI agent working in this tree can read and edit the whole chain, so asked
to add a task state it amends the *decision* in the meeting note rather than
patching the requirement in front of it — then propagates down to the Python
and reruns the acceptance cells.
