# DNS for hickorydocs.com.
#
# One zone, one stack — deliberately NOT per-environment workspaces. A DNS zone
# is a shared singleton: two workspaces both holding the apex record would
# fight over it on every apply. Staging lives in this zone as its own
# subdomain, so it is described here alongside production rather than in a
# parallel copy of the same zone.
#
# The zone's previous contents are disposable and are cleared by hand before
# the first apply — see docs/operators/dns.md. That is why nothing here is
# imported: starting from an empty zone means this file is the whole truth
# about it, which is the only state in which "the config describes the zone" is
# actually a true sentence.

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

# --- mail --------------------------------------------------------------------
#
# Deliberately empty, and both halves belong here when they exist:
#
#   receiving  @hickorydocs.com  -> MyMangoMail, via MX records
#   sending    verification/reset -> SendGrid, via three Domain Authentication
#                                    CNAMEs (em####, s1._domainkey, s2._domainkey)
#
# The zone previously carried Porkbun forwarding; that is being replaced and is
# not re-declared here, because declaring it would recreate exactly what is
# meant to go away.
#
# SendGrid needs no root SPF include: domain authentication moves the return
# path onto its own subdomain, so SPF is evaluated there. If MyMangoMail asks
# for an SPF record, that is the ONLY one this zone may ever have — two TXT
# records each beginning v=spf1 is an error under the spec, not a union, and
# fails SPF everywhere. See docs/operators/dns.md.
