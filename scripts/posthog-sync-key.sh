#!/usr/bin/env bash
# Push the Terraform-owned PostHog project write key to the Fly app.
#
# Terraform owns the analytics project (terraform/posthog) and therefore knows
# its write key; the server reads that key as POSTHOG_API_KEY. No provider
# bridges the two, so this script is the hand-off — the explicit fallback
# .instructions/continuous-delivery-paas.md prescribes for a value Terraform
# owns that the platform needs.
#
# Written in bash to match the other thin orchestration shims in this
# directory (codegen.sh, dev.sh); it only sequences two CLIs.
#
# Usage: scripts/posthog-sync-key.sh   (via `just posthog-sync-key`)
set -euo pipefail
cd "$(dirname "$0")/.."

STACK_DIR=terraform/posthog
FLY_APP=hickory-docs-production

die() {
  echo "error: $1" >&2
  [ $# -gt 1 ] && { echo >&2; echo "$2" >&2; }
  exit 1
}

command -v terraform >/dev/null || die "terraform is not installed." \
  "Install it from https://developer.hashicorp.com/terraform/install, then re-run."
command -v flyctl >/dev/null || die "flyctl is not installed." \
  "Install it with 'curl -L https://fly.io/install.sh | sh', then re-run."

# Reading an output requires initialized state. Initializing needs the R2
# backend endpoint, which embeds the Cloudflare account id, so this is a
# check rather than something the script can do for you.
[ -d "$STACK_DIR/.terraform" ] || die \
  "$STACK_DIR is not initialized." \
  "Run terraform init there first (see docs/operators/analytics.md for the
backend-config incantation), or apply the stack through the Terraform
workflow and run this afterwards."

echo "Reading the project key from Terraform state..."
if ! KEY=$(terraform -chdir="$STACK_DIR" output -raw project_api_key 2>/dev/null); then
  die "terraform has no 'project_api_key' output." \
    "The posthog stack has probably not been applied yet. Run the Terraform
workflow (workflow_dispatch → stack: posthog, apply: true), then re-run this.
Applying needs POSTHOG_PERSONAL_API_KEY in the production-plan and production
GitHub Environments — see docs/operators/analytics.md."
fi

# A key that is empty or does not look like a project key means we are about to
# configure the server with something that will never ingest an event. Fail
# here rather than let analytics silently no-op in production.
case "$KEY" in
  phc_*) ;;
  "") die "terraform returned an empty project key." \
       "That usually means the resource exists in state but the apply did not
complete. Re-run the Terraform workflow for the posthog stack." ;;
  phx_*) die "that is a PERSONAL api key (phx_…), not a project key (phc_…)." \
       "The server needs the project write key. Check that outputs.tf reads
posthog_project.production.api_token." ;;
  *) die "unexpected key format (expected it to start with 'phc_')." \
       "Refusing to set it rather than leave production quietly unable to
send events." ;;
esac

# Idempotence: `fly secrets set` restarts the machine, and a needless
# production restart to write a value that is already there is exactly the
# kind of avoidable blip that makes people stop running commands.
CURRENT_DIGEST=$(flyctl secrets list --app "$FLY_APP" --json 2>/dev/null \
  | python3 -c 'import json,sys
try:
    print(next(s["Digest"] for s in json.load(sys.stdin) if s["Name"] == "POSTHOG_API_KEY"))
except (StopIteration, ValueError):
    print("")' || echo "")

if [ -n "$CURRENT_DIGEST" ]; then
  echo "POSTHOG_API_KEY is already set on $FLY_APP (digest ${CURRENT_DIGEST})."
  echo "Fly does not expose secret values, so this cannot tell whether it"
  echo "matches Terraform's. Re-setting it is safe but restarts the machine."
  printf 'Overwrite it? [y/N] '
  read -r reply
  case "$reply" in
    [yY]*) ;;
    *) echo "Left unchanged."; exit 0 ;;
  esac
fi

echo "Setting POSTHOG_API_KEY on $FLY_APP (this restarts the machine)..."
printf 'POSTHOG_API_KEY=%s' "$KEY" | flyctl secrets import --app "$FLY_APP"

echo
echo "Done. Verify events are landing:"
echo "  curl -X POST https://hickorydocs.com/api/analytics/capture \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"distinct_id\":\"smoke\",\"event\":\"landing_viewed\",\"properties\":{}}'"
echo "then check the PostHog project's Activity view for the event."
