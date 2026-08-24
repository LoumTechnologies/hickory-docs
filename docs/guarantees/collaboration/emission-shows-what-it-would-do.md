# A Re-Emission Is Shown Before It Is Done, And Refuses To Rewrite What Is Published

Given a folder of stage documents in a git repository, when `hick emit` runs,
then it reports the commits a re-emission **would** produce — one stage, one
commit — and emits nothing; and when producing them would rewrite a commit
below the publication floor, then it refuses and names it.

**Commit boundaries come from stages.** A stage is one document plus the files
it generates, and `stages-write-forward.md` already establishes it as a real
boundary that never writes upstream — the same shape as a stack of dependent
changes. So one stage is one commit by default: the shape is structural rather
than an artifact of the order you typed, and a re-emission is deterministic
given the document.

**Publication is what makes emission one-way.** Below the floor a commit is a
record, and someone else may be holding it; above it, it is a draft that a
re-emission replaces — commits rebuilt, not patched, which is `jj squash --into`
with the destination computed rather than chosen.

Corollaries that are part of the guarantee:

- **Nothing is emitted.** The summary says so in those words. The difference
  between "what it would do" and "what it did" is the only thing standing
  between a person and a rewritten history.
- **A planned commit names the draft it would replace**, or says it would be
  new.
- **Planning never executes.** Documents are woven weave-only: a plan with side
  effects is not a plan.
- **A document that will not weave is still a stage**, with its files reported
  as unknown rather than being dropped from the picture.
- **Files are in path order**, so two runs plan the same commit.
- **A subject comes from the document's own first heading**, and a document
  without one still gets an honest subject rather than an empty one.
- **Refusing exits non-zero and says where the floor is**, per
  `user-facing-errors`.

What this does NOT claim: nothing emits. Steps 2, 4 and 5 of that design — each
emitted commit carrying the document version that emitted it, frontier
re-emission, and cross-repository reads pinned by hash — are not built. The
first of those is named in the module as a property to build in from the first
day, because retrofitting it means a generation of commits that cannot explain
themselves.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/emission.rs` — `plan`, `plan_for` (weave-only),
    `subject_of`, `commits_touching`, `EmissionPlan::allowed`.
  - `crates/hickory-cli/src/floor.rs` — the floor it reads.
  - `crates/hickory-cli/src/main.rs` — `hick emit` and its refusal.
  - Tests: `crates/hickory-cli/src/emission.rs` unit tests (3).
- Caveat requiring LLM review: the mapping from a planned commit to the
  frontier commit it would replace is "the newest draft commit touching that
  document". A stage whose document was touched by several drafts maps onto one
  of them, and which one is not something the design settles — it names
  stage-shaped commits as possibly the wrong grain and leaves the override
  undesigned.
