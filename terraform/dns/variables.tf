variable "zone_id" {
  description = "Cloudflare zone id for hickorydocs.com (Overview tab, right-hand column)."
  type        = string
}

variable "fly_ipv4" {
  description = <<-EOT
    Fly's shared IPv4 ingress for the app. Shared is fine: Fly routes on SNI,
    and a dedicated v4 costs money for no benefit here.
  EOT
  type        = string
  default     = "66.241.125.95"
}

variable "fly_ipv6" {
  description = "Fly's dedicated IPv6 ingress for the app."
  type        = string
  default     = "2a09:8280:1::163:1979:0"
}

variable "proxied" {
  description = <<-EOT
    Whether Cloudflare proxies (orange cloud) the app records.

    Must be false at least until Fly's certificates validate: a proxied record
    makes Cloudflare answer the ACME challenge with its own certificate, so
    Fly's issuance never completes. Proxying can be reconsidered afterwards,
    but it also puts a second TLS terminator in front of the WebSockets the
    collaborative editor depends on.
  EOT
  type        = bool
  default     = false
}
