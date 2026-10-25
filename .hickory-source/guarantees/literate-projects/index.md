<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Literate projects

For: Maintainers reorganizing source while preserving the complete generated program.

Construct the project from an empty tree, compare complete inventory, then review implementation and focused witnesses.

- [Candidate revision checks never borrow working bytes](candidate-revisions-never-borrow-working-bytes/contract.md) — retained-requirement.
- [Check every source file, including assets and permissions](complete-inventory-is-byte-exact/contract.md) — rewritten.
- [Comparison edit permissions do not establish replay evidence](comparison-permissions-and-evidence-are-independent/contract.md) — retained-requirement.
- [Check the agent's transition separately from maintained-source reconstruction](finalization-and-incorporation-check-distinct-equalities/contract.md) — reconciled-requirement.
- [Keep language tools and debug launches tied to their captured project](tools-select-complete-isolated-endpoints/contract.md) — reconciled-requirement.
- [Patch elements use the shared diff renderer](patch-view-uses-the-shared-diff-renderer/contract.md) — retained-requirement.
- [Patch layout exposes endpoint breakpoints](patch-layout-breakpoints/contract.md) — retained-requirement.
- [Keep a patch edit as a draft until the whole construction still works](patch-edits-revalidate-downstream-construction/contract.md) — reconciled-requirement.
- [Rebuild the program from its Markdown source](markdown-constructs-the-complete-project/contract.md) — rewritten.
- [Reject a patch that does not fit its exact input](replay-is-exact-and-atomic/contract.md) — rewritten.

<book:include file="candidate-revisions-never-borrow-working-bytes/contract.md" />
<book:include file="comparison-permissions-and-evidence-are-independent/contract.md" />
<book:include file="complete-inventory-is-byte-exact/contract.md" />
<book:include file="finalization-and-incorporation-check-distinct-equalities/contract.md" />
<book:include file="markdown-constructs-the-complete-project/contract.md" />
<book:include file="patch-edits-revalidate-downstream-construction/contract.md" />
<book:include file="patch-layout-breakpoints/contract.md" />
<book:include file="patch-view-uses-the-shared-diff-renderer/contract.md" />
<book:include file="replay-is-exact-and-atomic/contract.md" />
<book:include file="tools-select-complete-isolated-endpoints/contract.md" />

</book:doc>
