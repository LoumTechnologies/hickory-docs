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

The project is created **once**, by hand, and then never touched: pushes to
`master` publish through the **Deploy Site** workflow
(`.github/workflows/deploy-site.yml`).

Creating it needs an **account-scoped** API token. The token in `.env` is
deliberately scoped to one zone's DNS and nothing else, so it cannot do this —
that narrowness is the point, not an oversight.

1. Mint a token with **Account → Cloudflare Pages → Edit**.
2. Create a Pages project (direct upload, no git integration — the workflow
   uploads the build). Note its `*.pages.dev` hostname.
3. Set in the repository's `production` GitHub Environment:
   - `CLOUDFLARE_API_TOKEN` (secret) — the Pages-scoped token
   - `CLOUDFLARE_ACCOUNT_ID` (secret)
   - `CLOUDFLARE_PAGES_PROJECT` (variable) — the project name
   - `POSTHOG_KEY` (secret) — the `phc_…` project write key
4. Run **Deploy Site** once and confirm the `*.pages.dev` URL serves the page
   and `/install.sh`.
5. **Only then** point DNS at it — `terraform/dns` with `pages_hostname` set to
   the `*.pages.dev` name. Flipping DNS first takes the site down for as long
   as it takes to notice.

Pages satisfies both host requirements above out of the box: unknown paths
fall back to `index.html`, and `install.sh` is served as uploaded.
