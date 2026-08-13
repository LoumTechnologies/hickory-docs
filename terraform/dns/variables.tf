variable "zone_id" {
  description = "Cloudflare zone id for hickorydocs.com (Overview tab, right-hand column)."
  type        = string
}

variable "pages_hostname" {
  description = <<-EOT
    The Cloudflare Pages project's own hostname, e.g. `hickory-docs.pages.dev`.

    Both the apex and `www` are proxied CNAMEs to this. It is a variable rather
    than a hardcoded name because the project is created outside this stack —
    creating a Pages project needs an account-scoped API token, and the token
    this stack uses is deliberately scoped to one zone's DNS and nothing else.
  EOT
  type        = string
}
