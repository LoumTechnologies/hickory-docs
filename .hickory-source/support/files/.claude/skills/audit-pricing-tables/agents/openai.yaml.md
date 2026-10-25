<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .claude/skills/audit-pricing-tables/agents/openai.yaml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `0f43dfb2ce57cbbd0a9e0e5dc382574490cf44f24fe9d2b03ae9e59bf1dbf8bd`.

<book:entry path=".claude/skills/audit-pricing-tables/agents/openai.yaml">
interface:
  display_name: "Audit Pricing Tables"
  short_description: "Reconcile plans.json against displayed pricing tables and gates"
  default_prompt: "Use $audit-pricing-tables to audit plans.json against every pricing table, comparison grid, gate, and upgrade prompt in this repo, verify claimed features are actually implemented, confirm gates match plans.json limits, and check PostHog tracks usage of gated features."
</book:entry>

</book:doc>
