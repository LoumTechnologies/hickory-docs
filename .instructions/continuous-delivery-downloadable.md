# Continuous Delivery — downloadable product

> **Amended 2026-08-18** by `docs/specs/freeform/shipping-mobile-and-desktop.md`:
> mobile ships through the App Store and Play Store, which adds a gate this
> module assumed did not exist. Desktop delivery — `.dmg`, `AppImage`, `.msi`,
> GitHub releases, the one-line installer — is unchanged. Where this file says
> "the artifact is the deliverable and CI verifies it", read that as true of
> desktop and *weaker* for mobile: CI builds and bundles, a simulator smoke
> test stands in for a real install, and publication is App Review's verb.

This repository ships a **program people download**, not a service we operate.
There is no staging environment, no production environment, no promote gate,
and no infrastructure to provision — because there is no server. See
`docs/specs/freeform/local-only.md`.

This module **replaces** `continuous-delivery-shared`,
`continuous-delivery-paas`, and `start-with-production`, all of which describe
delivering a cloud service. Do not re-enable them. If a change seems to need
one of them, the change is reintroducing a backend, which is out of scope by
decision rather than by sequencing.

## Branch policy

- There is **always** a long-lived **`master`** branch.
- There is **never** a `production`, `main`, or `staging` branch.
- Never enable GitHub Actions jobs on arbitrary branches; they exist on
  `master` or on PRs into `master`.

## Channels

| Workflow `name:` (Actions UI) | Trigger | What it does |
|-------------------------------|---------|--------------|
| **Unstable Release** | Push to `master` | Builds every target and publishes a prerelease. Automatic — this is the channel that proves the build works. |
| **Trigger Stable Release** | `workflow_dispatch` | Human-chosen version bump (`patch`/`minor`/`major`), starts Stable Release. |
| **Stable Release** | Started by the trigger workflow | Builds, verifies the artifacts, publishes the release, stamps an immutable `vX.Y.Z` tag. |

Use the word **unstable** — not "edge", not "prerelease" — for the non-stable
channel in anything a user reads. GitHub's `prerelease: true` is a mechanic,
not the name.

**Release language, not promote language.** Promote/deploy naming belongs to
cloud products; this repo has neither.

## Rules

- **The artifact is the deliverable, so the artifact is what CI verifies.**
  A release build must be unpacked and exercised on each target before it is
  published — at minimum `hick --version`, `hick --help`, and executing a real
  document from the bundled `examples/`. A binary that builds but cannot run a
  document is a failed release, and the only place to catch that is here.
  Guarantee: `docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md`.
- **A download must work with no toolchain.** Never assume the user has Rust,
  Node, or a package manager. The archive carries the binary, the licence, and
  `examples/`.
- **Every platform we advertise is built and smoke-tested in CI.** If a target
  is not verified by a workflow, it is not a supported platform and must not
  appear in the installer or the docs. **Mobile is the one exception, and it is
  a weaker promise, not a waived one:** CI builds and bundles, a simulator
  smoke test stands in for installing to a device, and a human verifies before
  submission. Say that plainly rather than implying the same coverage.
- **The installer and the release assets are one contract.** `scripts/install.sh`
  resolves an asset name that `scripts/dist.sh` produces. Changing either name
  breaks installation for everyone; change them together, and keep the version
  the binary reports equal to the version in the asset name.
- **Nothing in the product phones home.** No telemetry, no update check, no
  licence check, no crash reporting. A tool that runs on private repositories
  earns that by not talking to anyone. Being *in* a store does not change this:
  the store reports installs to us as a property of the store, and the app
  still says nothing to anyone. Keep those two sentences apart — they are the
  kind of pair that collapses into each other by accident.
- **The site is static.** hickorydocs.com is files and a link to GitHub
  releases. It has no backend, no signup, and no pricing. If a change to the
  site needs a server, it is out of scope.
- **No monetization surface.** No `plans.json`, no payment provider, no
  entitlements, no feature gates, no upgrade prompts. This is a product
  decision recorded in `local-only.md`, not an unimplemented feature.

## Configuration

Because there is no deployment, there are no GitHub Environments, no
staging/production secret parity, and no `env-parity` check. The secrets are what
publishing a release needs: the default `GITHUB_TOKEN`, plus signing. An unsigned
installer reads to a new user as malware, which makes signing part of delivery
rather than a later polish step.

**What signing actually needs**, measured on a MacBook Pro (macOS 15.7.7, Xcode
26.3) on 2026-08-18 rather than assumed:

| Target | Identity | How it is submitted | What lives in CI |
|---|---|---|---|
| macOS `.dmg` | **Developer ID Application** certificate; signed with hardened runtime (`--options runtime`) and a secure timestamp, which notarization requires | `xcrun notarytool submit`, then `xcrun stapler staple` | the certificate as a base64 `.p12` + its password, and **either** an App Store Connect API key (`.p8` + Key ID + Issuer ID) **or** an Apple ID + app-specific password + Team ID |
| iOS App Store | **Apple Distribution** certificate **and** an App Store provisioning profile bound to an explicit App ID | `xcrun altool --upload-app` with an App Store Connect API key | the certificate `.p12` + password, the `.mobileprovision`, and the API key |
| Windows `.msi` | an OV or EV code-signing certificate whose private key **cannot be a file** — see below | — | a *service* credential, not a key |
| Play Store | an upload keystore | Play Developer API | the keystore + a service-account JSON |

Four things worth stating plainly, because each contradicts an assumption that is
easy to carry:

- **A free Apple ID is not enough.** It issues an "Apple Development"
  certificate, which runs a build on a device you own and can neither notarize
  nor submit. Both of those need Developer Program membership. This machine has
  exactly one identity — `Apple Development: nate@loumtechnologies.com`, team
  `6GN937Z3WZ`, valid to 2027-07-24 — and no `Developer ID Application`, no
  `Apple Distribution`, and no provisioning profiles installed.
- **Notarization is `notarytool` only.** `altool` on Xcode 26.3 (version
  26.10.1) has no notarization verb left at all; it is now only an App Store
  uploader. Anything written against `altool notarize-app` is dead.
- **Prefer the App Store Connect API key over an Apple ID.** It is the same
  credential for notarizing and for uploading, it has no 2FA to work around on a
  runner, and it can be revoked without touching the account's password.
- **A macOS runner is required.** `notarytool`, `stapler`, and `altool` all ship
  inside Xcode; none has a Linux equivalent.

**The Windows key is the one that cannot be a secret.** Since 2023-06-01 the
CA/Browser Forum requires the private key of every new or renewed code-signing
certificate — OV *and* EV — to be generated in and never leave a hardware module
certified to FIPS 140-2 Level 2 or Common Criteria EAL4+. A `.p12` in a GitHub
secret is therefore not an option for a certificate obtained now: the choices are
a USB token, which a hosted runner cannot reach, or a cloud signing service
(Azure Trusted Signing, SSL.com eSigner, DigiCert KeyLocker), where what CI holds
is an account credential that authorizes a signing call. Plan the Windows channel
around a signing *service*, not around a key file.

**Simulator builds need no identity.** `codesign --sign -` (ad-hoc) is enough to
install and launch on a simulator, which is what makes the mobile smoke test
runnable in CI before any of the above exists.

Runtime configuration is the **user's**, read from their environment at
startup, and must still be strongly typed and validated at boot per
`config-and-environments` — that rule survives the removal of the deployment,
because a badly-typed `HICKORY_EXECUTOR` fails on a user's machine where nobody
can debug it for them.

## Versioning

- Semantic versioning, starting from `0.x` — major and minor may be zero.
- A stable release stamps an immutable tag. Never move a published tag.
- **Rolling back means publishing a newer version**, not deleting an old one.
  Someone has already downloaded it.
