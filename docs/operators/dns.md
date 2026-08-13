# DNS for hickorydocs.com

The zone is served by Cloudflare and described in `terraform/dns/`. Plans are
posted on pull requests; applying is a gated `workflow_dispatch`, because this
zone serves production and `plan-deploy-shared` puts production changes behind
a gate rather than a branch push.

## Read this before the first apply

**Clear the zone first.** Cloudflare currently holds apex and `www` records
pointing at its own proxy, a wildcard `*.hickorydocs.com`, Porkbun MX records
and an SPF TXT record. None of it is in use, and all of it is going away.

This matters mechanically, not just tidily: Cloudflare permits several records
on one name, so applying over an existing apex would **add** the new target
beside the old one rather than replace it. The domain would then round-robin
between Pages and an origin that is not there, and roughly half of all requests
would fail.

Deleting the old records first — rather than importing them — is what lets
`terraform/dns/main.tf` be the whole truth about this zone. Nothing is
imported, so there is no hidden state to reconcile later.

## One-time bootstrap

Done once, by hand, and deliberately not automated — it creates the credentials
everything else depends on.

**1. R2 bucket for state.** In Cloudflare → R2, create `hickorydocs-terraform-state`.
Create an R2 API token with Object Read & Write on that bucket. Note the
account id from the R2 overview page.

**2. Cloudflare API token for DNS.** My Profile → API Tokens → Create Token.
Permissions: `Zone → DNS → Edit` **and** `Zone → Zone → Read`, scoped to
`hickorydocs.com` only. A token that can edit every zone in the account is a
far larger blast radius than this stack needs.

**3. Environment secrets.** Nothing is stored at the repository level — every
secret lives in a GitHub Environment, so the set of jobs that can read it is
explicit rather than "anything in this repo".

There are two environments, because plan and apply want opposite things:

| Environment | Protection | Used by | Why |
|---|---|---|---|
| `production-plan` | none | `plan` | Runs on every PR. A plan that needs approval before it renders is a plan nobody reads. |
| `production` | required reviewer | `apply` | The gate. Nothing reaches the zone without a human. |

Both hold the same five secrets:

| Secret | Where it comes from |
|---|---|
| `CLOUDFLARE_API_TOKEN` | step 2 |
| `CLOUDFLARE_ZONE_ID` | zone Overview page, right-hand column |
| `R2_ACCOUNT_ID` | R2 overview |
| `R2_ACCESS_KEY_ID` | step 1 |
| `R2_SECRET_ACCESS_KEY` | step 1 |

**Both copies are currently write-capable, which is worth improving.**
`production-plan` only needs to *read* Cloudflare and to read/write the state
lock, so a Cloudflare token scoped to `Zone:Zone:Read` + `Zone:DNS:Read` would
mean an ungated job could never change DNS even if the workflow were
subverted. That is the version to move to; the same token is in both today
only because one existed.

**5. Empty the zone.** In Cloudflare → DNS, delete every record except the
nameservers: the apex A/AAAA, the `www` records, the wildcard, the two Porkbun
MX records, and the SPF TXT. (Check nothing else is using the domain first —
`dig hickorydocs.com ANY` and a look at the dashboard.)

**6. First plan and apply.**

**Run it through the workflow, not locally:**

```sh
gh workflow run terraform.yml -f stack=dns -f apply=false   # plan
gh run watch
# read the plan, then:
gh workflow run terraform.yml -f stack=dns -f apply=true
```

The credentials are already in the `production-plan` and `production`
environments, so there is nothing to fetch, and the workflow pins the
Terraform version.

**That pin is the reason to prefer it.** CI runs `TF_VERSION` (1.9.8); a
workstation is likely to have something much newer. Terraform stamps the
version into state and refuses to read state written by a newer one — so a
single local `apply` from a newer binary locks CI out of the state until
someone upgrades the pin. The state is shared; the binary that writes it
should be too.

### Running it locally anyway

Sometimes you need `terraform output`, or to debug a plan interactively. Then
you need the R2 credentials — which cannot be read back out of GitHub, so
either you have them saved or you mint a fresh pair (Cloudflare → **R2** →
**Manage R2 API Tokens** → **Create API Token**, Object Read & Write on
`hickorydocs-terraform-state`). Creating a new pair does not invalidate the
old one.

```sh
cd terraform/dns
export CLOUDFLARE_API_TOKEN=…          # Zone:DNS:Edit + Zone:Zone:Read
export TF_VAR_zone_id=90bc4539c95be534f2533ff909949627
export AWS_ACCESS_KEY_ID=…             # the R2 access key id
export AWS_SECRET_ACCESS_KEY=…         # the R2 secret

terraform init -backend-config='endpoints={s3="https://437ea403f048c8547b4242bf10b891c5.r2.cloudflarestorage.com"}'
terraform plan
```

Match the pinned version if you intend to `apply` (`tfenv install 1.9.8`, or
bump `TF_VERSION` in the workflow and let CI move first). `plan` and `output`
are read-only and safe from any version.

The same `init` line works for `terraform/posthog`, which shares the bucket
under a different key.

**Read the plan before applying.** It should only ever touch the records this
stack owns; anything else means the zone holds something the config does not
know about.

## Mail records

Two providers, opposite directions, and they do not overlap:

| | Provider | Contributes |
|---|---|---|
| **Receiving** `@hickorydocs.com` | MyMangoMail | MX records (and whatever else its setup screen asks for) |
| **Sending** (verification, password reset) | SendGrid | Three CNAMEs from Domain Authentication |

The zone carries no mail records at all right now — the Porkbun forwarding that
used to be there is being replaced and was not carried over. Put both sets in
`terraform/dns/main.tf`, under the "inbound mail" heading that is waiting for
them, rather than in the dashboard.

### SendGrid

Settings → Sender Authentication → **Authenticate Your Domain**. It generates
three CNAMEs on subdomains — roughly `em####`, `s1._domainkey`, `s2._domainkey`
— pointing into `sendgrid.net`. Take the exact names from that screen; they
embed your account id.

**This does not need a root SPF include.** Domain authentication moves the
envelope sender (the return path) to the `em####.hickorydocs.com` subdomain, so
SPF is evaluated against *that* name, whose record SendGrid controls. It also
means DKIM signs as `hickorydocs.com`, so DMARC aligns.

Until this is done, **every send fails** — SendGrid refuses to send from an
address on a domain it has not authenticated, regardless of a valid API key.
The failure is visible rather than silent: the banner says "We could not send
that email" and the server log carries SendGrid's exact reason.

### The one thing that bites

**One SPF record, ever.** A domain with two TXT records that each begin
`v=spf1` fails SPF everywhere — the spec treats it as an error, not as a union.
If MyMangoMail asks for an SPF include and something else already has one, the
includes merge into a *single* record. Never add a second.

## Records not managed here, and why

- **The Pages project itself.** Creating it needs an account-scoped API token;
  the token this stack uses is scoped to one zone's DNS and nothing else. See
  `static-site.md` for the one-time setup.
- **Pages' certificate.** Cloudflare issues and renews it for a custom domain
  attached to the project. Nothing to add, and nothing to remember to renew.

## Proxying

The site records are **proxied** (`proxied = true`), and this is not optional.
Cloudflare Pages has no fixed origin address, so the records are CNAMEs to the
project's `*.pages.dev` hostname; unproxied, that resolves but serves *that*
hostname's certificate rather than ours.

This is a reversal of the previous arrangement, which kept proxying off so an
origin server could complete its own ACME challenge and so no second TLS
terminator sat in front of the editor's WebSockets. Both reasons are gone:
there is no origin server, and the editor's WebSocket now runs on the user's
own machine over loopback, which never crosses a network at all.

The apex being a CNAME is only legal because Cloudflare flattens it at the
edge. Moving this zone to a registrar without CNAME flattening would mean
finding an A record for a service that does not publish one.
