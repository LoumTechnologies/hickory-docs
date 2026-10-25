<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Release

For: Maintainers publishing installers and people checking what a download contains.

Choose the platform and channel, then inspect the build provenance and installation witness.

- [A Download Runs Without A Rust Toolchain](a-download-runs-without-a-rust-toolchain/contract.md) — retained-requirement.
- [A Local Install Is a Launchable macOS App](a-local-install-is-a-launchable-macos-app/contract.md) — retained-requirement.
- [Desktop collection excludes unchanged cached installers](desktop-artifacts-come-from-the-current-build/contract.md) — retained-requirement.
- [Publish the committed Unstable build chosen by the maintainer](only-a-green-commit-becomes-downloadable/contract.md) — reconciled-requirement.
- [The App Installs Its Command For The User](the-app-installs-its-command-for-the-user/contract.md) — retained-requirement.
- [The Desktop App Is Its Own Download](the-desktop-app-is-its-own-download/contract.md) — retained-requirement.
- [Unstable Updates Require Consent](unstable-updates-require-consent/contract.md) — retained-requirement.

<book:include file="a-download-runs-without-a-rust-toolchain/contract.md" />
<book:include file="a-local-install-is-a-launchable-macos-app/contract.md" />
<book:include file="desktop-artifacts-come-from-the-current-build/contract.md" />
<book:include file="only-a-green-commit-becomes-downloadable/contract.md" />
<book:include file="the-app-installs-its-command-for-the-user/contract.md" />
<book:include file="the-desktop-app-is-its-own-download/contract.md" />
<book:include file="unstable-updates-require-consent/contract.md" />

</book:doc>
