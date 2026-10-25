<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/AgentChanges.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c6748b2230ecdd215ea2909de5f8ef99765e08befc6bf10233a3540a33d0af9f`.

<book:entry path="apps/web/src/components/AgentChanges.css">
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
</book:entry>

</book:doc>
