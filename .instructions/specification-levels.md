---
skills:
  - guarantee-maintainer
---

# Maintain the current behavioral contracts

For agents changing a Hickory requirement or its implementation.

Start from `project.md` and open its owning contract under
`.hickory-source/guarantees/`. Edit the contract and the linked implementation/test
entries together. Regenerate the ordinary working files before focused tests.

The generated `docs/guarantees/` and `docs/specs/freeform/` files preserve the
pre-migration specifications. Their amendments and verification notes are
historical inputs; the linked contract pages record current dispositions and
remaining review work. New product promises belong in the canonical contract
pages, with independent evidence and unresolved caveats kept visible.

Keep LLM review, executed checks and human acceptance separate. Name the relevant
contract in tests. Follow `docs/developers/literate-workflow.md` for generation,
source/output checks and recovery.
