variable "posthog_personal_api_key" {
  description = <<-EOT
    PostHog **personal** API key (`phx_…`), not the project write key.

    Create it at Settings → Personal API keys with the narrowest scopes that
    let this stack work: Project (write) to create the project, and
    Organization (read) to resolve the organization. A key with global
    all-scopes access can delete every project in the organization, which is
    far more power than one Terraform stack needs.

    Supplied as TF_VAR_posthog_personal_api_key from the GitHub Environment,
    never through the provider's own POSTHOG_API_KEY environment variable —
    see the comment in versions.tf for why that name is booby-trapped here.
  EOT
  type        = string
  sensitive   = true
}

variable "posthog_host" {
  description = "PostHog API host. US cloud unless the account is on EU cloud."
  type        = string
  default     = "https://us.posthog.com"
}

variable "organization_id" {
  description = <<-EOT
    PostHog organization that owns the project. `@current` resolves to the
    authenticated key's organization, which is correct for a single-org
    account and avoids committing an opaque UUID.
  EOT
  type        = string
  default     = "@current"
}

variable "project_name" {
  description = <<-EOT
    Name of the analytics project.

    One project per product per environment is the portfolio rule. This
    product currently has exactly one environment (production is continuously
    deployed from master; there is no staging), so there is one project, and
    the name says which environment it is so a future staging project cannot
    be confused for it.
  EOT
  type        = string
  default     = "hickory-docs-production"
}
