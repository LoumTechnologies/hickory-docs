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
| Project write key | `phc_…` | Send events | Fly secret `POSTHOG_API_KEY` |
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

## Getting the key to the server

Terraform knows the write key; Fly needs it. No provider bridges the two, so
the hand-off is explicit:

```sh
just posthog-sync-key
```

It reads `terraform output -raw project_api_key`, refuses to proceed if the
value is empty or is not a `phc_…` project key, and sets the Fly secret —
which restarts the machine, so it asks before overwriting an existing value.

**Until this is run, analytics is a no-op.** The server accepts events and
drops them: `POST /api/analytics/capture` answers `{"accepted":true}` whether
or not PostHog is configured, deliberately, so a browser never retries
forever against a healthy server. The cost of that design is that silent
success looks identical to real capture from the outside. Verify with the
smoke test the script prints, then confirm the event appears in PostHog's
Activity view.

## Environments

Three GitHub Environments, and the distinction between them is load-bearing:

| Environment | Gated? | Holds | Used by |
|---|---|---|---|
| `production` | **Yes** — required reviewer | Cloudflare + PostHog personal + R2 state creds | Terraform **apply** |
| `production-plan` | No | Same, read-oriented | Terraform **plan** |
| `production-deploy` | No | `FLY_API_TOKEN`, `APP_BASE_URL` | **Deploy Production** |

The app deploys continuously and ungated; infrastructure does not. Those are
opposite policies about the same running system, which is why they cannot
share one environment. Removing the reviewer from `production` to make app
deploys automatic would silently ungate DNS and analytics applies too.

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
