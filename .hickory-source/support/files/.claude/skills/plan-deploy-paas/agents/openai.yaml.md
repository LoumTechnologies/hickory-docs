<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .claude/skills/plan-deploy-paas/agents/openai.yaml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `2a937754a9b6dddb7e1bc63ae5523000dca1586198d884f87007074a0c52faf0`.

<book:entry path=".claude/skills/plan-deploy-paas/agents/openai.yaml">
interface:
  display_name: "Plan Deploy — PaaS"
  short_description: "Staging/production delivery on a managed platform (Railway, Fly, Render)"
  default_prompt: "Use $plan-deploy-paas to choose a managed platform and plan or review delivery on it, where the platform owns runtime provisioning: pick between platforms on Terraform-provider maturity and pricing across both environments (comparison and the portfolio's current preference are in references/platform-notes.md), staging and production as separate platform environments with separate managed databases, deploys driven only from GitHub Actions behind the production environment gate with the platform's own auto-deploy-on-push turned off, deploy credentials in the matching GitHub Environment, a Stripe catalog still owned by Terraform with price ids flowing into platform variables rather than being copy/pasted, and an explicit written record of intended platform state to compensate for having no reviewable plan. Read $plan-deploy-shared alongside it for the branch policy, promote path, staging rules, launch-state, and surge runbook. A repo enables the paas or the terraform substrate module, never both — though Terraform is still present for the billing catalog either way."
</book:entry>

</book:doc>
