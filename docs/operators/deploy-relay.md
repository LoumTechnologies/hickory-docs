# Deploying the relay

*For whoever operates hickorydocs.com. The relay is a second Fly app that
forwards bytes between a guest's browser and a `hickory serve` session on
someone's machine. It shares nothing with the workspace app and is expected to
outlive it (`docs/specs/freeform/local-first.md`).*

## What it needs

| | Why |
|---|---|
| A Fly app + 1 GB volume | Accounts are SQLite on the volume — the only thing the relay persists |
| Wildcard DNS | Every session gets `<slug>.relay.hickorydocs.com`, minted per session |
| Wildcard TLS | Same reason |
| `RELAY_TOKEN_SECRET` | Signs the tokens the relay issues. **Stable** — changing it signs everyone out |
| `GH_OAUTH_CLIENT_ID` | Optional. Without it the relay offers email/password only, and the CLI hides the GitHub option |

## First deploy

From the repository root.

```sh
# 1. The app and its volume. The volume must exist before the first deploy:
#    a relay that boots without one writes accounts to the container filesystem
#    and forgets them on the next restart.
fly apps create hickory-relay-production
fly volumes create relay_data --size 1 --region iad -a hickory-relay-production

# 2. The token secret. Generate it once and keep it — this is the value that
#    signs every sign-in, and a new one invalidates all of them.
fly secrets set RELAY_TOKEN_SECRET="$(openssl rand -base64 48)" -a hickory-relay-production

# 3. Ship it.
fly deploy --config apps/relay/fly.toml --dockerfile apps/relay/Dockerfile

# 4. The addresses. IPv6 is free and dedicated; the shared IPv4 is enough
#    because Fly routes on SNI.
fly ips allocate-v6 -a hickory-relay-production
fly ips list -a hickory-relay-production
```

Put that IPv6 address into Terraform and apply — the zone is Terraform-owned,
and a record added by hand disappears on the next apply:

```sh
cd terraform/dns
terraform apply -var="relay_fly_ipv6=<the v6 address from above>"
```

Then the certificate. Fly validates a wildcard by DNS, so this waits on the
records above having propagated:

```sh
fly certs add "relay.hickorydocs.com"   -a hickory-relay-production
fly certs add "*.relay.hickorydocs.com" -a hickory-relay-production
fly certs check "*.relay.hickorydocs.com" -a hickory-relay-production
```

`fly certs check` prints the `_acme-challenge` record Fly wants if it cannot
validate on its own. Add it to `terraform/dns/main.tf` rather than to
Cloudflare directly, for the same reason as above.

## Check it

```sh
curl https://relay.hickorydocs.com/_relay/health
# {"ok":true,"tunnels":0}

curl https://relay.hickorydocs.com/_relay/auth/methods
# {"password":true}                 ← no GitHub app configured
# {"password":true,"github":{...}}  ← one is
```

Then end to end, from a machine with the CLI:

```sh
hickory login --signup
hickory serve docs/tour.hick --share --public
```

The banner prints a link. Open it from a phone on cellular — not from the same
network — because that is the only test that proves the relay is doing
anything.

## Adding GitHub sign-in later

The relay works without it. To add it:

1. Register a GitHub OAuth app (Settings → Developer settings → OAuth Apps).
   **Enable Device Flow.** No callback URL matters and the client secret is
   never used — device flow for a public client does not have one.
2. `fly secrets set GH_OAUTH_CLIENT_ID=Iv1.xxxxx -a hickory-relay-production`

The CLI picks it up from `/_relay/auth/methods` with no rebuild, so a
self-hosted relay can use its own app.

## Cost, and the one deliberate difference from the workspace app

The workspace app scales to zero. **The relay does not**, and must not: a
machine that stopped when idle would drop every open tunnel the moment traffic
paused, and the guest's next request would arrive at a process that had
forgotten the session. `min_machines_running = 1` on a `shared-cpu-1x` is a
few dollars a month, and it buys a link that keeps working while two people
think about a document.

## What is not automated yet

There is no `Deploy Relay` workflow. The first deploy has to be by hand anyway
(volume, addresses, certificates), and a workflow that shipped to an app whose
volume did not exist would fail in a way nobody would understand. Add one once
this has run for real — it is the same shape as `deploy-production.yml`, with
`--config apps/relay/fly.toml`.

## Backups

The accounts table is the only durable state. It is small and it is the one
thing a restart cannot rebuild:

```sh
fly ssh console -a hickory-relay-production -C "sqlite3 /data/relay.db .dump" > relay-accounts.sql
```

Everything else — tunnels, streams, quotas — is memory-only by design and
recovers by clients reconnecting.
