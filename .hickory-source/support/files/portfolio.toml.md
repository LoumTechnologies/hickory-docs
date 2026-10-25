<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# portfolio.toml

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `20ed26cfbd1900b12c453428cab35f65c65e9f4f6666b3ee0d2a670a09e5269e`.

<book:entry path="portfolio.toml">
[project]
slug = "hickory-docs"
name = "Hickory Docs"
status = "building"
tier = "full"
created = "2026-08-05"
decision_deadline = "2026-11-05"
budget_monthly_usd = 40

[domains]
production = "hickorydocs.com"
# No staging environment — production is continuously deployed from master.
# See .instructions/continuous-delivery-shared.md (pre-launch exception).
staging = ""

# The product is downloaded. Only the static homepage runs on Cloudflare Pages.
# Terraform owns DNS; GitHub Actions publishes the Pages project.
[cloudflare]
pages_project = "hickory-docs"

[stripe]
mode = "sandbox"

[vendors]
sendgrid = true
posthog = true
</book:entry>

</book:doc>
