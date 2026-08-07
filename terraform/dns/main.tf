# DNS for hickorydocs.com.
#
# One zone, one stack — deliberately NOT per-environment workspaces. A DNS zone
# is a shared singleton: two workspaces both holding the apex record would
# fight over it on every apply. Staging lives in this zone as its own
# subdomain, so it is described here alongside production rather than in a
# parallel copy of the same zone.
#
# The zone predates this configuration and is not empty. Everything already in
# it is declared below and must be IMPORTED before the first apply — see
# docs/operators/dns.md. Applying without importing would append a second set
# of apex records rather than replacing the existing ones, and the domain would
# round-robin between Fly and an origin that is not there.

# --- the app -----------------------------------------------------------------

resource "cloudflare_dns_record" "apex_v4" {
  zone_id = var.zone_id
  name    = "hickorydocs.com"
  type    = "A"
  content = var.fly_ipv4
  ttl     = 1 # 1 = "automatic"; Cloudflare requires this when proxied
  proxied = var.proxied
  comment = "Fly.io ingress — hickory-docs-production"
}

resource "cloudflare_dns_record" "apex_v6" {
  zone_id = var.zone_id
  name    = "hickorydocs.com"
  type    = "AAAA"
  content = var.fly_ipv6
  ttl     = 1
  proxied = var.proxied
  comment = "Fly.io ingress — hickory-docs-production"
}

resource "cloudflare_dns_record" "www_v4" {
  zone_id = var.zone_id
  name    = "www.hickorydocs.com"
  type    = "A"
  content = var.fly_ipv4
  ttl     = 1
  proxied = var.proxied
  comment = "Fly.io ingress — hickory-docs-production"
}

resource "cloudflare_dns_record" "www_v6" {
  zone_id = var.zone_id
  name    = "www.hickorydocs.com"
  type    = "AAAA"
  content = var.fly_ipv6
  ttl     = 1
  proxied = var.proxied
  comment = "Fly.io ingress — hickory-docs-production"
}

# --- inbound mail (pre-existing) ---------------------------------------------
#
# Porkbun email forwarding. Declared here because this file claims to describe
# the zone, and a config that silently omits live records is worse than no
# config: the next person reads it as complete and removes what they cannot
# see. Sending (SendGrid) and receiving (Porkbun) are different directions and
# coexist fine.

resource "cloudflare_dns_record" "mx_primary" {
  zone_id  = var.zone_id
  name     = "hickorydocs.com"
  type     = "MX"
  content  = "fwd1.porkbun.com"
  priority = 10
  ttl      = 1
  comment  = "Porkbun email forwarding — do not remove without moving inbound mail"
}

resource "cloudflare_dns_record" "mx_secondary" {
  zone_id  = var.zone_id
  name     = "hickorydocs.com"
  type     = "MX"
  content  = "fwd2.porkbun.com"
  priority = 20
  ttl      = 1
  comment  = "Porkbun email forwarding — do not remove without moving inbound mail"
}

resource "cloudflare_dns_record" "spf" {
  zone_id = var.zone_id
  name    = "hickorydocs.com"
  type    = "TXT"
  content = "\"v=spf1 include:_spf.porkbun.com ~all\""
  ttl     = 1
  comment = "SPF. SendGrid authenticates its own return-path subdomain, so outbound mail does not need an include here — check what SendGrid asks for rather than editing this by hand."
}
