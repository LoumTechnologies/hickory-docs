terraform {
  required_version = ">= 1.6"

  required_providers {
    posthog = {
      # PostHog's own provider. Registry tier is "community" because it lives
      # outside the HashiCorp partner programme, not because it is a
      # third-party effort — the source repo is github.com/PostHog.
      source  = "PostHog/posthog"
      version = "~> 1.0"
    }
  }

  # Same remote, locked, encrypted R2 state as the DNS stack, under its own
  # key. A separate stack rather than a second module in `dns/` because the two
  # share no resources and have completely different blast radii: a botched
  # apply here loses analytics history, a botched apply there takes the domain
  # offline. See terraform/dns/versions.tf for why the s3 backend needs the
  # skips below to work against R2.
  #
  # This state holds the project's write key and is therefore secret.
  backend "s3" {
    bucket = "hickorydocs-terraform-state"
    key    = "posthog/terraform.tfstate"
    region = "auto"

    # R2 is path-style only; see terraform/dns/versions.tf for what the
    # default virtual-hosted addressing fails with.
    use_path_style = true

    use_lockfile                = true
    skip_credentials_validation = true
    skip_metadata_api_check     = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    # `skip_s3_checksum` is the knob that makes R2 work: R2 does not implement
    # the AWS checksum semantics the SDK assumes, and without this every
    # operation fails on a checksum mismatch rather than on anything
    # meaningful.
    #
    # `request_checksum_calculation` / `response_checksum_validation` used to
    # sit here too. They were added to the S3 backend for a middle range of
    # Terraform versions and are rejected by newer ones ("An argument named
    # ... is not expected here"), so they are gone; `skip_s3_checksum` was
    # always the load-bearing one.
    skip_s3_checksum = true
  }
}

provider "posthog" {
  # Passed explicitly rather than through the provider's `POSTHOG_API_KEY`
  # environment variable, which collides with the SERVER's variable of the
  # same name. They are different secrets with very different power:
  #
  #   POSTHOG_API_KEY (server, `phc_…`)  — project write key; can send events.
  #   this one        (Terraform, `phx_…`) — personal API key; can create and
  #                                          destroy projects.
  #
  # Letting the provider pick the name up from the environment would mean a
  # runner that happens to have the server's key exported silently
  # authenticates Terraform with a key that cannot manage anything, and the
  # error would point at permissions rather than at the mix-up.
  api_key = var.posthog_personal_api_key
  host    = var.posthog_host
}
