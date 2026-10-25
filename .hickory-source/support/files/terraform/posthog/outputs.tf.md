<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# terraform/posthog/outputs.tf

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `44d83b57ffc3627ae1c9114bed694d187a0549e51c33a3d22a736af7b5293f81`.

<book:entry path="terraform/posthog/outputs.tf">
# The write key the server needs. Terraform creates the project and knows the
# key; Fly needs it as POSTHOG_API_KEY. The PostHog provider cannot set a Fly
# secret and the Fly provider is too immature to trust with this stack, so the
# hand-off is an explicit step rather than a hidden one:
#
#     just posthog-sync-key
#
# which reads the output below and runs `fly secrets set`. That is the
# documented fallback in .instructions/continuous-delivery-paas.md for exactly
# this case — a value Terraform owns that the platform needs.

output "project_id" {
  description = "Numeric PostHog project id (appears in app.posthog.com URLs)."
  value       = posthog_project.production.id
}

output "project_api_key" {
  description = <<-EOT
    Project write key (`phc_…`) for POSTHOG_API_KEY on the Fly app.

    Read it with `terraform output -raw project_api_key`. It is marked
    sensitive so a plan or apply log never prints it — note that it IS stored
    in the state file, which is why that state lives in encrypted R2 and not
    in the repository.
  EOT
  value       = posthog_project.production.api_token
  sensitive   = true
}
</book:entry>

</book:doc>
