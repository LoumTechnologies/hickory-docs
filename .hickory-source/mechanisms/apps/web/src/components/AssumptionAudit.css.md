<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AssumptionAudit.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a9656d08b410"></a>
## `apps/web/src/components/AssumptionAudit.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/AssumptionAudit.css">
.assumption-audit { margin: .75rem; padding: .6rem; border: 1px solid var(--border); border-radius: .35rem; font-size: .9rem; }
.assumption-audit > summary { cursor: pointer; font-weight: 600; }
.assumption-audit label { display: flex; gap: .5rem; align-items: center; margin: .5rem 0; }
.assumption-audit input, .assumption-audit select, .assumption-audit textarea { min-width: 0; max-width: 100%; }
.assumption-audit section { border-top: 1px solid var(--border); padding: .5rem 0; }
.assumption-audit p { overflow-wrap: anywhere; }
.assumption-audit pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 24rem; overflow: auto; }
.assumption-audit form label { display: grid; align-items: start; }
.assumption-audit button { margin-left: .5rem; }

.verification-standing { margin: .5rem .75rem; font-size: .85rem; overflow-wrap: anywhere; }
</storycode:entry>

</storycode:doc>
