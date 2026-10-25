<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# terraform/posthog/main.tf

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1a03b7f4befb892a50296edc6366005e1e3c779584adc16e925ab32b72545cca`.

<book:entry path="terraform/posthog/main.tf">
# PostHog analytics projects for Hickory Docs.
#
# Why this is Terraform and not a click in the PostHog UI: the project's write
# key is what the server sends events with, and a dashboard-created project
# leaves no record of which key belongs to which environment. Two products in
# one organization (this one and portzero) plus a future staging environment
# is exactly the situation where events quietly land in the wrong project and
# nobody notices until the funnel numbers are already wrong.
#
# Deliberately NOT managed here: the `Default project` and `portzero-production`
# projects that already exist in this organization. portzero belongs to another
# repository, and importing another product's project would let an apply in
# this repo destroy it.

resource "posthog_project" "production" {
  name            = var.project_name
  organization_id = var.organization_id
  timezone        = "UTC"

  lifecycle {
    # An analytics project cannot be restored once destroyed — the events go
    # with it. Recreating one is cheap; recovering six months of funnel
    # history is impossible. A rename that Terraform decides to satisfy by
    # replacement would silently do exactly that, so destruction has to be a
    # deliberate act: comment this out, apply, then remove the resource.
    prevent_destroy = true
  }
}
</book:entry>

</book:doc>
