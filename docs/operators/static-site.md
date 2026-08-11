# The marketing site

*For whoever deploys hickorydocs.com. The site is a directory of files — no
server, no database, no runtime configuration.*

The landing page and its three interactive demos run entirely in the visitor's
browser: they simulate git, script execution, and a collaborative room in
TypeScript, and make no API calls. Pricing is generated from `plans.json` at
build time. So the whole marketing surface is static, which is what
`docs/specs/freeform/local-first.md` expects — the product is a program people
install, not a workspace they sign into.

## Build it

```sh
just site                          # no analytics
POSTHOG_KEY=phc_… just site        # with browser-side capture
```

Output is `apps/web/dist/`. Deploy that directory anywhere that serves files
(Cloudflare Pages, Netlify, S3 + CloudFront, a static bucket, nginx).

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

Without the key, the page falls back to posting to `/api/analytics/capture` —
which only exists when the bundle is served by the hosted app. On a static
deployment, no key means no analytics, silently and harmlessly.

Expect fewer events than the server-side beacon collected: a request to
PostHog is among the most-blocked on the web, and a same-origin `/api` POST is
not. That is the price of not running a server.

## The same bundle, three ways

| Deployment | Build | What the visitor gets |
|---|---|---|
| Static site | `just site` | Landing, demos, pricing, install command. No accounts. |
| Hosted app | `Dockerfile` (`VITE_HOSTED=1`) | The above, plus sign-in, workspaces, and Stripe checkout |
| `hickory serve` | any build | The document editor, opened straight into a document with a session token |

`VITE_HOSTED` is the only switch: it adds the sign-in link and the subscribe
buttons. The default is the deployment with no server, so a build that forgets
to set anything is the safe one.

## Before the install command works for strangers

`scripts/install.sh` downloads release assets from a **private** GitHub
repository, so today it needs `HICKORY_GITHUB_TOKEN`. Publishing the site
before the repository is public ships a call to action that fails for everyone
who is not us. Either flip the repository (the plan of record — MIT, per
`architecture.md`) or host the release archives somewhere public and point the
script at them.
