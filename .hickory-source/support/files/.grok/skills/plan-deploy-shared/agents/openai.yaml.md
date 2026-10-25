<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .grok/skills/plan-deploy-shared/agents/openai.yaml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f9d9cf41c6d092da70c53ba579ccb5845c23bd53d1ff2501de71a1891e979dbe`.

<book:entry path=".grok/skills/plan-deploy-shared/agents/openai.yaml">
interface:
  display_name: "Plan Deploy — Shared"
  short_description: "The substrate-independent half of staging/production delivery"
  default_prompt: "Use $plan-deploy-shared for the delivery rules that hold no matter who provisions the infrastructure: production and staging as named GitHub Environments (never 'prod'), a long-lived master branch with no production branch, automatic Deploy Staging plus a gated Trigger Promote to Production → Promote to Production path with immutable version tags and ref-based rollback, staging that differs from production in size but never in shape and never holds real customer data, a plans.json-driven billing catalog with append-only prices and a sandbox/live mismatch that fails rather than warns, one analytics project per product per environment with deploy annotations, a just launch-state command that classifies risk before destructive changes, and a traffic-surge runbook at docs/operators/runbooks/traffic-surge.md. Pair it with exactly one substrate skill: $plan-deploy-terraform or $plan-deploy-paas."
</book:entry>

</book:doc>
