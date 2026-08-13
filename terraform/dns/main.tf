# DNS for hickorydocs.com.
#
# One zone, one stack. There is one deployment — a static site — so there are
# no per-environment workspaces to keep apart.
#
# The site is **static files on Cloudflare Pages**. There is no server, no
# relay, and no app to route to; the whole zone is a marketing page, a download
# link, and the records SendGrid needs. See
# `docs/specs/freeform/local-only.md`.

# --- the site ----------------------------------------------------------------
#
# CNAME to the Pages project rather than A/AAAA to an origin, because Pages has
# no fixed address: Cloudflare resolves the project name onto whichever edge is
# nearest. `proxied = true` is not optional here — an unproxied CNAME to
# `*.pages.dev` resolves, but serves that hostname's certificate rather than
# ours.
#
# The apex is a CNAME, which is only legal because Cloudflare flattens it at
# the edge. On a registrar without CNAME flattening this would have to be an
# A record to an address Pages does not publish.

resource "cloudflare_dns_record" "apex" {
  zone_id = var.zone_id
  name    = "hickorydocs.com"
  type    = "CNAME"
  content = var.pages_hostname
  ttl     = 1 # 1 = "automatic"; Cloudflare requires this when proxied
  proxied = true
  comment = "Cloudflare Pages — the static site"
}

resource "cloudflare_dns_record" "www" {
  zone_id = var.zone_id
  name    = "www.hickorydocs.com"
  type    = "CNAME"
  content = var.pages_hostname
  ttl     = 1
  proxied = true
  comment = "Cloudflare Pages — the static site"
}

# --- email -------------------------------------------------------------------
#
# SendGrid's sending records. Kept because the domain still sends mail (release
# announcements, replies to people who write in) even though the product has no
# accounts and sends nothing itself.

# Renamed from `sendgrid_return_path` when this file was rewritten. Without
# this block Terraform reads a rename as "destroy the old, create the new",
# which for a live DNS record is a window where mail links resolve to nothing
# — and a window that lasts indefinitely if the create then fails.
moved {
  from = cloudflare_dns_record.sendgrid_return_path
  to   = cloudflare_dns_record.sendgrid_bounce
}

resource "cloudflare_dns_record" "sendgrid_bounce" {
  zone_id = var.zone_id
  name    = "em4579.hickorydocs.com"
  type    = "CNAME"
  content = "u60828890.wl141.sendgrid.net"
  ttl     = 1
  proxied = false
  comment = "SendGrid — bounce/link domain"
}

resource "cloudflare_dns_record" "sendgrid_dkim1" {
  zone_id = var.zone_id
  name    = "s1._domainkey.hickorydocs.com"
  type    = "CNAME"
  content = "s1.domainkey.u60828890.wl141.sendgrid.net"
  ttl     = 1
  proxied = false
  comment = "SendGrid — DKIM"
}

resource "cloudflare_dns_record" "sendgrid_dkim2" {
  zone_id = var.zone_id
  name    = "s2._domainkey.hickorydocs.com"
  type    = "CNAME"
  content = "s2.domainkey.u60828890.wl141.sendgrid.net"
  ttl     = 1
  proxied = false
  comment = "SendGrid — DKIM"
}

resource "cloudflare_dns_record" "dmarc" {
  zone_id = var.zone_id
  name    = "_dmarc.hickorydocs.com"
  type    = "TXT"
  content = "\"v=DMARC1; p=none;\""
  ttl     = 1
  proxied = false
  comment = "DMARC — monitor only"
}
