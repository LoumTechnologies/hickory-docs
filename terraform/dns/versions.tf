terraform {
  required_version = ">= 1.6"

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.0"
    }
  }

  # Remote, locked, encrypted state. It holds a Cloudflare token, so it is
  # never committed and never local-only.
  #
  # Cloudflare R2 is S3-compatible, so the standard `s3` backend works — and it
  # avoids adding a cloud account this project otherwise has no use for.
  # (`plan-deploy-terraform` names DigitalOcean Spaces; that is the portfolio
  # default because DigitalOcean is the portfolio's default substrate, which
  # this project does not use.)
  #
  # R2 has no DynamoDB, so `use_lockfile` provides locking via a lock object in
  # the bucket itself. R2 also does not implement the AWS checksum and region
  # semantics the provider assumes, hence the skips below — without them every
  # operation fails on a checksum mismatch rather than anything meaningful.
  backend "s3" {
    bucket = "hickorydocs-terraform-state"
    key    = "dns/terraform.tfstate"
    region = "auto"

    # R2 serves only path-style URLs (endpoint/bucket/key). The AWS SDK
    # defaults to virtual-hosted style (bucket.endpoint/key), whose hostname
    # simply does not resolve on R2 — the failure surfaces as
    # "dial tcp: lookup <bucket>.<account>.r2.cloudflarestorage.com: no such
    # host", which reads like a network fault rather than an addressing mode.
    use_path_style = true

    use_lockfile                 = true
    skip_credentials_validation  = true
    skip_metadata_api_check      = true
    skip_region_validation       = true
    skip_requesting_account_id   = true
    skip_s3_checksum             = true
    request_checksum_calculation = "when_required"
    response_checksum_validation = "when_required"

    # endpoints.s3 comes from -backend-config at init time: it embeds the
    # Cloudflare account id, which is not a secret but is account-specific and
    # does not belong hard-coded in a repo that may be forked.
  }
}

provider "cloudflare" {
  # CLOUDFLARE_API_TOKEN from the environment. Scope it to exactly
  # Zone:DNS:Edit + Zone:Zone:Read on hickorydocs.com — a token that can edit
  # every zone in the account is a much larger blast radius than this stack
  # needs.
}
