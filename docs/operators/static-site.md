# Publish the homepage

For the person editing or deploying hickorydocs.com. Hickory Docs is
**downloadable software**; Cloudflare Pages serves only the static homepage.
The short JavaScript / TypeScript example starts a real interpreter worker
and pauses at a breakpoint. It needs no server or authenticated engine.

## Edit the copy and example

| Change | File |
|---|---|
| Headline, description, download section | `apps/web/src/views/LandingView.tsx` |
| Short example and instructions | `apps/web/src/landing/demos/BrowserDebugDemo.tsx` |
| Page title and link preview descriptions | `apps/web/site.html` |
| Homepage layout | `apps/web/src/landing/homepage.css` |

The current copy is provisional. Keep the download message visible above the
example. The source is a nine-line `.md` document; its pause and variables
come from actual execution. Editing keeps the previous session visible as
stale until Restart. No remote-engine authentication is part of this demo.

```sh
just preview-browser-embedding
```

This builds the acceptance surface and serves it locally. For hot reload,
use the existing `just dev-site` recipe. The desktop app's `index.html` and
homepage's `site.html` are separate entries.

## Build and verify

```sh
just install-test-browsers
just test-browser-embedding --grep homepage
just site
```

`just site` produces `apps/web/dist-site/index.html`, hashed assets and
`install.sh`. The production build excludes the host test fixtures. The
acceptance build includes them to test React embedding, storage and iframes.
Do not publish `apps/web/dist/`: that is the desktop app's UI.

`POSTHOG_KEY=phc_… just site` enables the existing website analytics. With no
key, capture is disabled. A personal `phx_…` key is refused. The downloaded
product never receives this key or sends website analytics.

## Deploy

Push to `master`. **Deploy Site** uses the existing production environment's
`CLOUDFLARE_PAGES_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`, checks the homepage in
three browsers, rebuilds the production bundle, attaches both domain names,
and publishes to the `hickory-docs` Pages project. A failed acceptance check
prevents publication. `workflow_dispatch` can republish the current commit.

Cloudflare Pages [serves static requests for free without a request quota](https://developers.cloudflare.com/pages/functions/pricing/).
There are no Pages Functions or R2 runtime objects in this demo. Its files
fit within the [Free plan's asset limits](https://developers.cloudflare.com/pages/platform/limits/).
R2 is used by the existing Terraform state backend, independently of the site.

If `hickory-docs.pages.dev` works but hickorydocs.com returns 522, check both
custom-domain attachment and the Terraform-managed DNS records. Apex and
`www` must be proxied CNAMEs to `hickory-docs.pages.dev`.

```sh
gh workflow run terraform.yml -f stack=dns -f apply=false
# Read that run's plan before requesting its apply.
gh workflow run terraform.yml -f stack=dns -f apply=true
```

The apply invocation uses the production environment's protection rules.
It prints and applies the same saved plan in one runner, so exhausted GitHub
artifact storage cannot prevent repairing DNS. Email records are part of the
DNS stack; inspect the plan to ensure a site repair does not change them.

## Read the maintained source publicly

Open `https://hickorydocs.com/source.html`. The entry is `project.md`; the document
links, breadcrumbs and a left files pane navigate the maintained `.hickory-source/` pages.
The documents teach how to build Hickory in construction order. The reader shows
the rendered document without an additional introduction, statistics or picker.
It uses Hickory's shared read-only document surface and folds captured
scaffold files by default. It starts no local engine and runs no cells.

`just site` exports only committed Markdown from `project.md` and
`.hickory-source/`, with an index and per-page hashes at `public/source/`.
The displayed revision identifies those exact Git bytes, including during a
dirty local preview. Untracked files, local sessions and credentials are excluded.
`project.md` and maintained-source changes trigger **Deploy Site**, so narrative
changes cannot silently leave the public source mirror stale.

The existing selected executable-spec publications are separate. When their
reviewed input hashes change, inspect the actual difference and rerun the focused
publisher before updating the maintained selection. Never bypass that check merely
to make a site deployment green.

## Public downloads

The source repository is public. The homepage links to its GitHub releases and
the existing GitHub-based installer. Keep the download copy consistent with the
actual published release assets; publishing the static source browser does not
build or release a new downloadable application.

If records were recreated in the Cloudflare dashboard, the ordinary plan may
try to create duplicates. The workflow's explicit recovery option discovers
and imports only the existing apex and `www` records, then applies the same
DNS configuration. It does not delete live records:

```sh
gh workflow run terraform.yml -f stack=dns -f apply=true -f reconcile_site_dns=true
```

After changing DNS back to Pages, an attached domain can remain inactive.
**Deploy Site** retries validation for inactive domains and requires both to
be active. To retry validation without rebuilding or publishing:

```sh
gh workflow run deploy-site.yml -f domains_only=true
```
