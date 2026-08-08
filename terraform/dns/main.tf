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

# --- outbound mail (SendGrid) ------------------------------------------------
#
# SendGrid Domain Authentication. Created through SendGrid's UI and imported
# here, because a config that omits live records is one the next reader takes
# as complete and prunes against.
#
# `em4579` is the branded return path: it is what moves the envelope sender
# onto a subdomain SendGrid controls, which is why this zone needs NO root SPF
# record for SendGrid to pass. The two `_domainkey` names carry DKIM, which is
# what makes DMARC align.
#
# There is deliberately no MX record and no SPF record: nothing receives mail
# at this domain. If that changes, note that a zone may only ever have ONE SPF
# record — two TXT records each beginning v=spf1 is an error under the spec,
# not a union, and fails SPF everywhere.

resource "cloudflare_dns_record" "sendgrid_return_path" {
  zone_id = var.zone_id
  name    = "em4579.hickorydocs.com"
  type    = "CNAME"
  content = "u60828890.wl141.sendgrid.net"
  ttl     = 1
  proxied = false # a proxied CNAME would break the return path
  comment = "SendGrid domain authentication — branded return path"
}

resource "cloudflare_dns_record" "sendgrid_dkim1" {
  zone_id = var.zone_id
  name    = "s1._domainkey.hickorydocs.com"
  type    = "CNAME"
  content = "s1.domainkey.u60828890.wl141.sendgrid.net"
  ttl     = 1
  proxied = false
  comment = "SendGrid domain authentication — DKIM"
}

resource "cloudflare_dns_record" "sendgrid_dkim2" {
  zone_id = var.zone_id
  name    = "s2._domainkey.hickorydocs.com"
  type    = "CNAME"
  content = "s2.domainkey.u60828890.wl141.sendgrid.net"
  ttl     = 1
  proxied = false
  comment = "SendGrid domain authentication — DKIM"
}

resource "cloudflare_dns_record" "dmarc" {
  zone_id = var.zone_id
  name    = "_dmarc.hickorydocs.com"
  type    = "TXT"
  content = "\"v=DMARC1; p=none;\""
  ttl     = 1
  comment = "p=none: report only. Tighten to quarantine/reject once reports show DKIM aligning."
}
