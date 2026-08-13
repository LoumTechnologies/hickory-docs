variable "zone_id" {
  description = "Cloudflare zone id for hickorydocs.com (Overview tab, right-hand column)."
  type        = string
}

variable "pages_hostname" {
  description = <<-EOT
    The Cloudflare Pages project's own hostname.

    Both the apex and `www` are proxied CNAMEs to this. It is a variable rather
    than a hardcoded name because the project is created outside this stack —
    the Deploy Site workflow creates it on its first run, using a token scoped
    to Pages, while the token *this* stack uses is deliberately scoped to one
    zone's DNS and nothing else.
  EOT
  type        = string
  default     = "hickory-docs.pages.dev"
}
