# DNS for hickorydocs.com

The zone is served by Cloudflare and described in `terraform/dns/`. Plans are
posted on pull requests; applying is a gated `workflow_dispatch`, because this
zone serves production and `plan-deploy-shared` puts production changes behind
a gate rather than a branch push.

## Read this before the first apply

**The zone is not empty, and Terraform does not know about it yet.**

Cloudflare currently holds apex and `www` records pointing at its own proxy,
Porkbun MX records, an SPF TXT record, and a **wildcard `*.hickorydocs.com`**.
`terraform/dns/main.tf` declares everything except the wildcard.

Cloudflare permits several A records on one name, so an apply against an
un-imported zone would **add** Fly's addresses beside the existing ones rather
than replace them. The domain would then round-robin between Fly and an origin
that is not there, and roughly half of all requests would fail. Import first.

## One-time bootstrap

Done once, by hand, and deliberately not automated — it creates the credentials
everything else depends on.

**1. R2 bucket for state.** In Cloudflare → R2, create `hickory-tfstate`.
Create an R2 API token with Object Read & Write on that bucket. Note the
account id from the R2 overview page.

**2. Cloudflare API token for DNS.** My Profile → API Tokens → Create Token.
Permissions: `Zone → DNS → Edit` **and** `Zone → Zone → Read`, scoped to
`hickorydocs.com` only. A token that can edit every zone in the account is a
far larger blast radius than this stack needs.

**3. Repository secrets** (Settings → Secrets and variables → Actions):

| Secret | Where it comes from |
|---|---|
| `CLOUDFLARE_API_TOKEN` | step 2 |
| `CLOUDFLARE_ZONE_ID` | zone Overview page, right-hand column |
| `R2_ACCOUNT_ID` | R2 overview |
| `R2_ACCESS_KEY_ID` | step 1 |
| `R2_SECRET_ACCESS_KEY` | step 1 |

**4. A `production` GitHub Environment** with whatever reviewers you want. The
apply job targets it, so it cannot run until they approve.

**5. Import what already exists.**

```sh
cd terraform/dns
export CLOUDFLARE_API_TOKEN=...  AWS_ACCESS_KEY_ID=...  AWS_SECRET_ACCESS_KEY=...
export TF_VAR_zone_id=...
terraform init -backend-config="endpoints={s3=\"https://<account-id>.r2.cloudflarestorage.com\"}"

# List the live records and their ids:
curl -s -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
  "https://api.cloudflare.com/client/v4/zones/$TF_VAR_zone_id/dns_records" \
  | python3 -m json.tool | grep -E '"id"|"name"|"type"|"content"'

# Then, for each one (id from above):
terraform import cloudflare_dns_record.apex_v4     "$TF_VAR_zone_id/<record-id>"
terraform import cloudflare_dns_record.apex_v6     "$TF_VAR_zone_id/<record-id>"
terraform import cloudflare_dns_record.www_v4      "$TF_VAR_zone_id/<record-id>"
terraform import cloudflare_dns_record.www_v6      "$TF_VAR_zone_id/<record-id>"
terraform import cloudflare_dns_record.mx_primary  "$TF_VAR_zone_id/<record-id>"
terraform import cloudflare_dns_record.mx_secondary "$TF_VAR_zone_id/<record-id>"
terraform import cloudflare_dns_record.spf         "$TF_VAR_zone_id/<record-id>"

terraform plan   # apex/www should show IN-PLACE UPDATES, never creations
```

**A plan that wants to *create* apex or www records means the import did not
take.** Stop and fix it rather than applying.

## Records not managed here, and why

- **The wildcard `*.hickorydocs.com`.** Every name resolves today
  (`random123.hickorydocs.com` answers), so a wildcard exists. It is not
  declared because its intent is unknown — deleting it could break something
  outside this repo. Decide what it is for, then either bring it in or remove
  it. Explicit records win over a wildcard, so it does not block anything here.
- **SendGrid's authentication records.** Domain authentication generates CNAMEs
  on a per-account subdomain (`em####`, plus two DKIM names). Add whatever
  SendGrid's UI asks for; do not hand-write them, and do not edit the root SPF
  to compensate — SendGrid authenticates its own return-path subdomain.
- **Fly's ACME challenge.** Handled by Fly against the A/AAAA records; nothing
  to add.

## Proxying

`var.proxied` is `false` and must stay false until `fly certs check
hickorydocs.com` reports the certificate issued. A proxied record makes
Cloudflare answer the ACME challenge with its own certificate, so Fly's
issuance never completes. Turning it on afterwards is possible, but it also
places a second TLS terminator in front of the WebSocket connections the
collaborative editor uses — worth being deliberate about rather than leaving on
by default.
