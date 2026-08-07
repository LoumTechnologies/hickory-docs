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
    bucket = "hickory-tfstate"
    key    = "dns/terraform.tfstate"
    region = "auto"

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
