# The marketing site

*For whoever deploys hickorydocs.com. The site is a directory of files — no
server, no database, no runtime configuration.*

The landing page and its interactive demos run entirely in the visitor's
browser: they simulate git, script execution, and an editing room in
TypeScript, and make no API calls. There is no pricing page, because there is
nothing to buy. The whole marketing surface is static, which is what
`docs/specs/freeform/local-only.md` expects — the product is a program people
download, not a workspace they sign into.

## Build it

```sh
just site                          # no analytics
POSTHOG_KEY=phc_… just site        # with browser-side capture
```

Output is `apps/web/dist-site/`. That is the **site** build (`site.html`); the
default `npm run build` produces `dist/`, which is the desktop app's UI and
must never be deployed here. Deploy `dist-site/` anywhere that serves files —
Cloudflare Pages is what this project uses.

## Analytics is the site's, never the product's

The site measures its visitors with PostHog. The downloaded binary sends
nothing, ever: no telemetry, no update check, no first-run ping. The two never
share a key, a build, or a code path. Keep it that way — "we use analytics" and
"the tool phones home" are the kind of pair that collapses into each other by
accident.

Two requirements of the host:

1. **SPA fallback** — unknown paths must serve `index.html`, because the client
   routes on the URL hash and a deep link should not 404.
2. **`/install.sh` served as-is** — `just site` copies `scripts/install.sh`
   into the bundle, because the call to action on every page is
   `curl -fsSL https://hickorydocs.com/install.sh | sh`. Serve it with
   `Content-Type: text/plain` or `application/x-sh`; do not let the host
   rewrite it.

## Analytics

With `POSTHOG_KEY` set, the page posts events straight to PostHog's
`/capture/` endpoint. The key is inlined into the bundle and is public by
construction — that is fine for a **project write key** (`phc_…`), which can
only send events.

It is not fine for a **personal API key** (`phx_…`), which can create and
destroy projects. Two guards exist because that mix-up is easy to make:
`just site` refuses to build with one, and the app refuses to use one at
runtime (logging and disabling capture rather than crashing the page).

Without the key there is no analytics, silently and harmlessly. There is no
server-side fallback: `/api/analytics/capture` existed when the bundle was
served by a hosted app, and that app is gone.

Expect fewer events than a same-origin beacon would collect: a request to
PostHog is among the most-blocked on the web. That is the price of not running
a server, and it is the right price.

## Deploying to Cloudflare Pages

**A push to `master` that touches `apps/web/` publishes the site.** The
**Deploy Site** workflow (`.github/workflows/deploy-site.yml`) builds it and
uploads it; there is no gate and nothing to approve, because a manual gate here
would mean the site silently stops tracking `master` until someone notices a
pending approval.

### One-time setup

Exactly one secret is missing, and it cannot be the one already there. The
`CLOUDFLARE_API_TOKEN` in the `production` environment is scoped to this zone's
DNS and nothing else — that narrowness is deliberate, and it is why the token
cannot see the account, let alone create a Pages project. Replacing it would
break the Terraform DNS stack. So the site deploy gets its own:

1. Mint a Cloudflare token with **Account → Cloudflare Pages → Edit**, scoped
   to this account only (see below for the exact dashboard path).
2. `gh secret set CLOUDFLARE_PAGES_TOKEN --env production`
3. Optionally set `POSTHOG_KEY` — the `phc_…` project write key, which lives
   in the `terraform/posthog` state. `docs/operators/analytics.md` has the
   exact commands (the backend needs its R2 endpoint at init). Without the key
   the site ships with no analytics, which is a working site, not a broken one.

Nothing else. `CLOUDFLARE_ACCOUNT_ID` is already set, and the workflow creates
the Pages project on its first run, so there is no dashboard step beyond
minting the token.

### Minting the token

**dash.cloudflare.com → the account menu (top right) → My Profile → API Tokens
→ Create Token → Create Custom Token.**

| Field | Value |
|---|---|
| Token name | `hickory-docs pages deploy` |
| Permissions | **Account** · **Cloudflare Pages** · **Edit** |
| Account Resources | Include · `Nate@loumtechnologies.com's Account` |
| Client IP / TTL | leave unset |

One permission row is enough. Do **not** add Zone permissions: DNS is a
separate stack with a separate token, and a token that can do both has a blast
radius neither job needs.

The token is shown once. Paste it straight into
`gh secret set CLOUDFLARE_PAGES_TOKEN --env production`, which reads from
stdin, rather than into a file.

### Then point the domain at it

Only after a deploy has succeeded and the `*.pages.dev` URL serves the page:

```sh
cd terraform/dns
export CLOUDFLARE_API_TOKEN=…          # the zone-scoped one, not the Pages token
export TF_VAR_zone_id=90bc4539c95be534f2533ff909949627
export AWS_ACCESS_KEY_ID=…             # R2_ACCESS_KEY_ID
export AWS_SECRET_ACCESS_KEY=…         # R2_SECRET_ACCESS_KEY
terraform init -backend-config='endpoints={s3="https://437ea403f048c8547b4242bf10b891c5.r2.cloudflarestorage.com"}'
terraform apply
```

`pages_hostname` defaults to the project name the workflow creates, so there is
nothing to pass.

Flipping DNS first takes the site down for as long as it takes to notice.
`hickorydocs.com` currently still resolves to the old Fly machine, which is
also still running and billing — destroy it once Pages is serving.

Pages satisfies both host requirements above out of the box: unknown paths
fall back to `index.html`, and `install.sh` is served as uploaded.
