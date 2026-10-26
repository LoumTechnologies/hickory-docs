<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AgentChanges.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b9b904250eee"></a>
## `apps/web/src/components/AgentChanges.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/AgentChanges.css">
.agent-changes {
  flex: 0 0 auto;
  min-height: 0;
  padding: .5rem .75rem;
  border-bottom: 1px solid var(--border);
}
.agent-changes > label { display: flex; align-items: center; gap: .5rem; }
.agent-changes > .agent-change-link { margin-top: .5rem; }
.agent-changes .agent-change-link > span { overflow-wrap: anywhere; }
.agent-changes .btn { margin: .35rem .35rem .35rem 0; }
/* Inline previews own their content height. Shrinking them lets later turns
   paint over pending review controls and intercept the reviewer's pointer. */
.agent-changes .document-reading { height: auto; }
</storycode:entry>

</storycode:doc>
