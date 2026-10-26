---
name: audience-first-literate-docs
description: Write, reorganize, or maintain Hickory Docs literate source with audience-focused explanations interleaved with the actual implementation and tests. Use for guarantee-organized codebases, literate refactors, and reviews of code/prose ownership; use ordinary audience-first-docs for standalone product documentation.
---

# Audience First Literate Docs

Write for a maintainer learning or changing one stakeholder-visible behavior.
Explain the behavior, show the code that implements it, and show how its important
outcomes are checked, in a useful reading order.

## The unit is an explained behavior

Give every substantive code fragment an owning explanation. Code without a
meaningful place in the narrative is a design or documentation smell to inspect.
Explain why the fragment exists, what decision it makes, or which invariant or
failure case it protects. A path, hash, symbol name, or generic "edit this entry"
paragraph does not provide that explanation.

Interleave at meaningful reasoning boundaries: a check, transformation, state
transition, failure path, or coherent group of declarations. Avoid a sentence per
line and a long file dump after a high-level promise. Keep small obvious plumbing
with the behavior it supports; do not manufacture prose to fill a quota.

Navigation pages and shared conceptual explanations may contain no code. Do not
split a behavior into contract, review, construction, and payload pages merely to
satisfy a template. Keep explanation, relevant implementation, and focused tests
readable together. Split when a reader's task or reusable abstraction changes,
rather than at every output-file boundary.

## Start from the reader's change

1. Identify the reader and their next concrete task. Read the current requirement,
   actual implementation, and relevant tests before explaining the behavior.
2. Trace the path from input through decision to observable result. Identify
   errors, units, state ownership, ordering, and side effects that affect the task.
3. Arrange that reasoning as explanation → actual maintained code → next
   explanation. Introduce a failure example before its refusal logic. Place an
   assertion beside the outcome it demonstrates.
4. Keep review status and evidence limits near the claim they qualify. Distinguish
   implementation inspection, executed checks, and human acceptance. An include,
   reference, test name, or passing unrelated test cannot establish correctness.

Explain the surprising part. For example: "Validate before subtracting so an
invalid request cannot become a negative balance" helps a maintainer decide
where to insert a new check; "This function validates the request" adds little.
Do not invent stakeholder guarantees from a convenient implementation detail.

## Use Hickory to maintain the narrative

Inspect the project's entry document, local instructions, and supported Hickory
syntax. Use the installed implementation or its tests to resolve uncertain
semantics; do not invent assembly features or assume a Markdown fence generates
code. Respect the project's namespace binding and execution environment.

Use `copy` fragments beside their explanations and explicit `paste` selections in
a `file` construction when a file needs several explained pieces. Keep narrative
order useful to the reader and assembly order explicit for the compiler. Stable
fragment names describe responsibility, not line positions. Preserve indentation,
inter-fragment whitespace, encoding, final newlines and metadata.

Raw `entry` payloads preserve bytes, including embedded tag examples; do not
assume they expand nested `paste` tags. Check delimiter collisions when choosing a
namespace prefix. Keep exact assets and binary payloads in appropriate literal
constructions with a brief explanation of their role.

Read [the worked literate example](assets/interleaved-example.md) when introducing
fragment assembly or demonstrating the intended reading experience. It constructs
a small Python implementation and its tests. It is a pattern, not a mandated
language, naming scheme, or test framework.

Maintain one definition for each shared fragment and one construction owner for
each output. Explain a shared abstraction at its owner; callers explain why they
use it and link to that owner. Do not duplicate implementations or test bodies to
make every guarantee look self-contained. Do not assign unrelated code to a
guarantee just to remove an ownership warning.

When code lacks a clear owner, inspect its callers and purpose. Place it under an
honest behavior or shared mechanism if supported by the code. Otherwise record
the unresolved ownership and what needs inspection. Do not hide it in an
unexplained support bucket, delete it on that evidence alone, or claim the whole
codebase is explained after processing only some files.

## Preserve the maintained source

Edit owning Markdown constructions, not generated files alone. In a reorganizing
change, hold generated outputs fixed unless the user also authorized behavior
changes. Record the starting inventory and compare the completed reconstruction
for exact bytes, paths, kinds and portable modes. A successful build is not a
byte-equivalence check.

Update include edges, ownership maps and incoming links when moving fragments.
Retain requirement decisions and evidence dependencies; distinguish current
contracts from historical text. Consolidating pages must not silently discard an
exclusion, failure case, or unresolved verification note.

Regenerate through Hickory, then run the project's source/output and ownership
checks plus relevant focused behavior or safety witnesses. If the evaluator or
adapter changed, repeat with the rebuilt implementation. Use a fresh temporary
directory when checking Markdown-only reconstruction. Follow local limits on
tests, commits and status monitoring; this skill grants no publishing permission.

For the Hickory repository that uses `project.md`, `.hickory-source/`, and
`hickory-dogfood/behavior-map.json`, read `AGENTS.md` and
`docs/developers/literate-workflow.md` for the active workflow. These are local
conventions, not required paths for other Hickory projects. Do not rerun
historical capture scripts as an editing workflow.

## Review the result as a reader

- Can the intended maintainer follow one behavior from explanation to actual
  implementation and a meaningful assertion without a chain of wrapper pages?
- Does the prose explain decisions and constraints rather than translate syntax?
- Does each substantive fragment have an honest owner, including shared code and
  tests? Are remaining ownership gaps visible and specific?
- Are the useful path and important failure paths explained, with evidence
  described only as strongly as the checks support?
- Do generated outputs agree with maintained source, and with the starting bytes
  when this was a reorganization? Are intentional differences accounted for?

Report the scope actually improved and what remains. File counts and link
coverage measure organization; they do not establish explanatory quality.
