# A Denied Mach Lookup Explains Itself

Given `HICKORY_EXECUTOR=sandbox` on macOS, when a cell's command is refused
because the Seatbelt `Profile::Cell` policy grants no `(allow mach-lookup)`,
then the error a person sees names the sandbox and the fix
(`HICKORY_EXECUTOR=local`), not only the raw string the denied system service
returned.

## Why

`SandboxedExecutor::explain` already did this for a denied network — a cell
that cannot resolve a hostname fails with `Temporary failure in name
resolution` or similar, which reads like a wifi problem, and `explain` appends
the real reason plus `<hick:allow>` as the fix. Mach lookups had no equivalent.
`dotnet` is the confirmed case: its crypto/certificate stack calls
`securityd`/`trustd` over Mach IPC for `CSSM_ModuleLoad`, the policy has no
`(allow mach-lookup)` at all (`policy.rs`'s `Profile::Cell` is `(deny
default)` with only `process-exec`, `process-fork`, `sysctl-read`, and scoped
`file-read*`/`file-write*`), and the denial surfaces as
`CSSM_ModuleLoad(): One or more parameters passed to a function were not
valid.` — a string that names neither `dotnet`, the sandbox, nor Mach IPC.

Unlike the network case, there is no per-document escape hatch offered:
`<hick:allow>` grants a host and port, not a Mach service, and mach-lookup
stays denied by design (loosening it per-document would let a sandboxed cell
reach arbitrary system services by name). The only way out is
`HICKORY_EXECUTOR=local`, which the message says plainly.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hickory-executor-sandbox/src/lib.rs`'s
  `looks_like_a_denied_mach_lookup` and the second branch of `explain`;
  confirmed against a real `HICKORY_EXECUTOR=sandbox hick run` of a `.hick`
  document with a `dotnet build` cell on macOS 15.7.7 — the sandboxed run
  fails with `CSSM_ModuleLoad(): One or more parameters passed to a function
  were not valid.` while the identical command outside the sandbox and under
  `HICKORY_EXECUTOR=local` succeeds.
- Test coverage: `crates/hickory-executor-sandbox/src/lib.rs`'s
  `explain_tests` module (3 tests: the confirmed signature is recognized, two
  unrelated failures — a missing file, a denied network — are not mistaken
  for it, and the network detector still works unchanged alongside the new
  one).
- Caveat requiring LLM review: the signature list is one confirmed string.
  Other Mach-IPC-denied failures (a different Security-framework call, a
  different tool than `dotnet`) will not yet be recognized and will fall
  through to the raw, unexplained error — extend the list as real cases are
  found rather than guessing signatures ahead of evidence. This guarantee
  covers macOS/Seatbelt only; Linux/bubblewrap has no Mach IPC and this
  failure mode does not apply there.
