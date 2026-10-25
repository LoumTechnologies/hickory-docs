<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# terraform/dns/variables.tf

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3756a8f99c21f4b8a80a79701d239de22f9876266998f4abf54c0d099ccf55c7`.

<book:entry path="terraform/dns/variables.tf">
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
</book:entry>

</book:doc>
