# An Adapter That Ships As An Archive Installs Like Any Other

Given a debug adapter published as per-platform release archives rather than
through any package manager, when a person types `hick dap install <language>`,
then it installs the same way every other adapter does — confined, into
`.hick-cache/adapters`, found afterwards without configuration — and:

- **the archive is pinned.** Each platform's asset carries a SHA-256 checked
  before anything is unpacked, in one `&&` chain, so a mismatch stops before
  the archive is opened rather than after;
- **a pin is not a signature, and is not called one.** It says these are the
  bytes somebody looked at when the line was written. Upstream publishes no
  signatures, so that is the strongest available statement and the code says
  which one it is;
- **a platform with no build is told so by name**, rather than reported as a
  missing `curl` on a machine that has curl;
- **the unpacker is named per asset** (`tar` for a `.tar.gz`, `unzip` for a
  `.zip`) and its absence is its own message;
- **nothing here is a second install path.** Same sandbox, same prefix, same
  "discovery prefers what you installed yourself" rule, same refusal to
  install without a sandbox.

The first adapter to need this is **netcoredbg**, which is why C# had no
install command until now.

---

Last LLM verification:
- Date: 2026-08-27
- Reviewer: Claude (Opus 5)
- Result: verified by running the whole loop
- Evidence:
  - `Asset` and `asset_for` in `crates/hickory-cli/src/tool_install.rs`; the
    command an asset builds is fetch → verify → unpack → remove, chained on
    `&&`. `sha256sum` on Linux, `shasum -a 256` on macOS, chosen at compile
    time because only the program name differs.
  - The `csharp` entry and the four pinned `NETCOREDBG` assets in
    `crates/hickory-cli/src/dap_install.rs`.
  - `.hick-cache/adapters/netcoredbg` added to `project_dirs` in
    `crates/hick-dap/src/discovery.rs` — the archive unpacks to a directory
    holding the binary beside its managed DLLs, so the binary *is* that
    directory's name and there is no `bin/` to point at.
- Verified by running, on Linux x86_64 (2026-08-27):
  - `hick dap install csharp` in a clean project fetched
    netcoredbg 3.2.0-1092 under bubblewrap with only the prefix writable,
    printed the checksum line as `OK`, and left
    `.hick-cache/adapters/netcoredbg/netcoredbg`.
  - `crates/hick-dap/tests/live_session_csharp.rs` then reported
    **`using adapter netcoredbg (project)`** and ran a full debug session
    against it — so the install, the discovery and the debugger are one loop
    that has been walked end to end, not three parts assumed to meet.
- Test coverage:
  - `every_installer_writes_only_into_its_prefix` now checks the command
    built for **every platform's** asset, not just this machine's.
  - `every_archive_is_pinned_to_bytes_somebody_looked_at` — a 64-hex
    checksum, an `https://` URL, and an unpacker something checks for, on
    every asset; plus all of one installer's assets naming the same release,
    which is what catches a half-finished version bump.
  - `nothing_is_offered_that_cannot_be_installed_and_nothing_installable_is_hidden`
    (`dap_install.rs`) caught this change's own contradiction the moment the
    catalogue grew the entry — `how_to_get("csharp")` still said hick could
    not install it — which is exactly the drift it exists for.
- Caveat requiring review:
  - **Only `linux-x86_64` has been executed.** The other three checksums are
    bytes that were fetched and hashed here, and the archives were never
    unpacked or run. `unzip` on macOS and Windows is untried, and the macOS
    zip carries `__MACOSX/` entries that will be extracted alongside the real
    tree — harmless, untidy, and unverified.
  - **Windows is pinned but almost certainly incomplete.** The asset is
    listed and `unzip` is not something a Windows machine reliably has;
    nothing has run there.
  - **The version is bumped by hand.** Four URLs and four checksums, and the
    test only catches them disagreeing about the release — not a checksum
    that was mistyped for the right release. That failure is loud (the
    install stops at verification) rather than dangerous.
  - **No `.hick-cache/adapters/netcoredbg` cleanup.** Re-running the install
    unpacks over the existing tree rather than replacing it, so a version
    downgrade could leave a newer release's DLLs behind. Not observed, not
    handled.
