# Analytics (PostHog)

The product analytics project is **Terraform-owned**, not created by hand in
the PostHog dashboard. It lives in `terraform/posthog`.

## Why Terraform owns it

The project's write key is what the server sends events with. A
dashboard-created project leaves no record of which key belongs to which
environment — and this PostHog organization already holds more than one
product (`portzero-production`) with a future staging environment likely. That
is exactly the situation where events quietly land in the wrong project and
nobody notices until the funnel numbers are already wrong and unfixable.

`terraform/posthog` deliberately does **not** manage the pre-existing
`Default project` or `portzero-production`. portzero belongs to another
repository; importing it here would let an apply in this repo destroy it.

## One-time setup

The stack needs a PostHog **personal** API key. This is not the same thing as
the key the server uses, and the difference matters:

| Key | Looks like | Can do | Lives in |
|---|---|---|---|
| Project write key | `phc_…` | Send events | GitHub secret `POSTHOG_KEY`, in `production` |
| Personal API key | `phx_…` | Create/destroy projects | GitHub secret `POSTHOG_PERSONAL_API_KEY` |

The PostHog Terraform provider reads a `POSTHOG_API_KEY` environment variable
by default — the same name this repo already uses for the *server's* key. The
stack therefore takes it as `TF_VAR_posthog_personal_api_key` instead and never
relies on the ambient name. Do not "simplify" that back.

1. In PostHog: **Settings → Personal API keys → Create**. Scope it as narrowly
   as it will go — Project (write) and Organization (read). A global
   all-scopes key can delete every project in the organization.
2. Add it as `POSTHOG_PERSONAL_API_KEY` to **both** the `production-plan` and
   `production` GitHub Environments (plan and apply are separate jobs drawing
   from separate environments):

   ```sh
   gh secret set POSTHOG_PERSONAL_API_KEY --env production-plan
   gh secret set POSTHOG_PERSONAL_API_KEY --env production
   ```

## Creating or changing the project

Actions → **Terraform** → Run workflow → stack `posthog`, apply `true`.

The plan renders first and ungated; the apply waits on the required reviewer
of the `production` environment. That gate is intentional and is *not* the
same environment the app deploy uses — see "Environments" below.

## Getting the key to the site

Terraform knows the write key; the **Deploy Site** workflow needs it to build
the bundle. No provider bridges the two, so the hand-off is explicit:

```sh
cd terraform/posthog
export AWS_ACCESS_KEY_ID=…      # R2_ACCESS_KEY_ID
export AWS_SECRET_ACCESS_KEY=…  # R2_SECRET_ACCESS_KEY
terraform init -backend-config='endpoints={s3="https://437ea403f048c8547b4242bf10b891c5.r2.cloudflarestorage.com"}'

terraform output -raw project_api_key
gh secret set POSTHOG_KEY --env production   # paste it
```

`terraform output` needs the backend initialised, and the backend needs that
endpoint: the state is in R2, and R2's endpoint embeds the account id, so it is
passed at init rather than committed.

Check it is a `phc_…` **project** key before pasting. A `phx_…` personal key
can create and destroy projects and must never reach a browser bundle; `just
site` refuses to build with one, and the page refuses to use one at runtime,
but neither guard helps if the wrong value is stored and nobody reads the
build log.

**Until this is set, analytics is a no-op** — and visibly so, which is a change
for the better. There is no server to accept events and silently drop them:
with no key the page simply does not capture, and the absence is obvious in
PostHog rather than indistinguishable from real traffic.

The key affects **only the marketing site**. The downloaded product sends
nothing, ever, and shares no key, build, or code path with the site.

## Environments

Two GitHub Environments:

| Environment | Gated? | Holds | Used by |
|---|---|---|---|
| `production` | **Yes** — required reviewer | Cloudflare (zone + Pages) + PostHog personal + R2 state creds, `POSTHOG_KEY`, `CLOUDFLARE_PAGES_PROJECT` | Terraform **apply**, **Deploy Site** |
| `production-plan` | No | Same, read-oriented | Terraform **plan** |

There is no `production-deploy`. It existed to hold `FLY_API_TOKEN` for a
server that no longer exists; the only thing deployed now is a directory of
files.

## Only one environment's worth of analytics

There is one project because there is one environment — production is
continuously deployed from `master` and there is no staging (see
`.instructions/continuous-delivery-shared.md`). The portfolio rule is one
project per product *per environment*, so **adding a staging environment means
adding a second `posthog_project` resource here**, not pointing staging at this
one.

Local development must leave `POSTHOG_API_KEY` unset. It is optional
everywhere, and dev traffic in the production project is indistinguishable
from real visitors after the fact.

## Destroying the project

`posthog_project.production` carries `prevent_destroy = true`. Events cannot be
recovered once the project is gone, and a rename that Terraform chose to
satisfy by replacement would take the history with it. Removing it is a
deliberate two-step: comment out the lifecycle block, apply, then remove the
resource.
