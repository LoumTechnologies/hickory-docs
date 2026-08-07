# DNS for hickorydocs.com

The zone is served by Cloudflare and described in `terraform/dns/`. Plans are
posted on pull requests; applying is a gated `workflow_dispatch`, because this
zone serves production and `plan-deploy-shared` puts production changes behind
a gate rather than a branch push.

## Read this before the first apply

**Clear the zone first.** Cloudflare currently holds apex and `www` records
pointing at its own proxy, a wildcard `*.hickorydocs.com`, Porkbun MX records
and an SPF TXT record. None of it is in use, and all of it is going away.

This matters mechanically, not just tidily: Cloudflare permits several A
records on one name, so applying over the existing apex records would **add**
Fly's addresses beside them rather than replace them. The domain would then
round-robin between Fly and an origin that is not there, and roughly half of
all requests would fail.

Deleting the old records first — rather than importing them — is what lets
`terraform/dns/main.tf` be the whole truth about this zone. Nothing is
imported, so there is no hidden state to reconcile later.

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

**5. Empty the zone.** In Cloudflare → DNS, delete every record except the
nameservers: the apex A/AAAA, the `www` records, the wildcard, the two Porkbun
MX records, and the SPF TXT. (Check nothing else is using the domain first —
`dig hickorydocs.com ANY` and a look at the dashboard.)

**6. First plan and apply.**

```sh
cd terraform/dns
export CLOUDFLARE_API_TOKEN=...  AWS_ACCESS_KEY_ID=...  AWS_SECRET_ACCESS_KEY=...
export TF_VAR_zone_id=...
terraform init -backend-config="endpoints={s3=\"https://<account-id>.r2.cloudflarestorage.com\"}"
terraform plan    # four creations: apex A/AAAA and www A/AAAA. Nothing else.
```

Then apply through the workflow (Actions → Terraform → Run workflow → apply)
so the first change goes through the same gate every later one does.

**A plan showing anything other than those four creations means the zone was
not emptied.** Stop and look rather than applying.

## Mail records

The zone carries no mail records at all right now — the Porkbun forwarding that
used to be there is being replaced and was not carried over.

When the provider is configured, **put its records in `terraform/dns/main.tf`**
rather than the dashboard, under the "inbound mail" heading that is waiting for
them. Two things to get right:

- **Take the records from the provider's setup screen.** Do not hand-write SPF
  or DKIM.
- **One SPF record, ever.** A domain with two TXT records that each begin
  `v=spf1` is a domain that fails SPF everywhere — if two services send as
  `@hickorydocs.com`, their includes merge into a single record.

## Records not managed here, and why

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
