# Declared Capabilities Are Enforced

Given a document that declares container capabilities with `hick:allow` /
`hick:deny`, when the pipeline runs it, then those declarations decide what
each container actually gets:

- **Network.** A container whose `<hick:container>` carries
  `<hick:allow network="host:port">` starts on a connected network; a
  container that declares nothing, or only `<hick:deny network="*">`, starts
  with `--network none`. Silence means no network.
- **Volume read.** A container that mounts a volume the volume's
  `<hick:allow>` rules never name is refused, with an error naming the
  container, the volume, and the tag to add. A container granted part of a
  volume (`read="public/**"`) is handed that part and nothing else.
- **Volume write.** A read-only container's writes never leave it. A partial
  writer (`write="Controllers/**"`) has exactly its granted paths merged back
  into the volume; its other writes are dropped.
- **No rules, no change.** A volume with no `<hick:allow>` children stays
  unrestricted, so documents written before access rules existed keep
  working. Rules bind everyone only once anyone is named.

`hick:allow` had been parsed since the beginning and enforced nowhere. The
volume rules reached the DAG builder — where they decided writer→reader edge
order — and the network rules reached a `ContainerCapabilities` struct that
was minted into a token nobody presented. A document could advertise that its
report generator never touches the network and that its linter cannot modify
the source tree; both claims were decorative.

This matters more than tidiness because of where the language is going:
`hick:agent` (`docs/specs/freeform/agent-cells.md`) makes running someone
else's document spend the reader's tokens and lets a model author the
commands. A capability that is described but not imposed is exactly the wrong
substrate for that.

## Where each half is enforced, and why they differ

**Volumes are the pipeline's.** Data enters and leaves a container as a tar
archive that `hick-literate` moves, so the grant is enforced by narrowing what
crosses that boundary — on every backend identically, including the
deliberately unsandboxed `LocalExecutor`. Filtering happens only for partial
grants; a full grant passes the archive through untouched, so nothing is
re-encoded in the common case.

**The network is the executor's.** `Executor::declare_capabilities` hands a
container's declaration to the backend before anything starts, because a
sandbox confines a container when it is created. `DockerExecutor` turns that
into `--network`; `LocalExecutor` records it and imposes nothing, which is
the honest behavior for an executor with no sandbox at all — never point it at
an untrusted document.

## What this deliberately does not do

- **Host and port in a network grant are not enforced.** `--network` is a
  switch: a container granted `github.com:443` can reach whatever the network
  named by `HICKORY_DOCKER_ALLOWED_NETWORK` (default `bridge`) reaches.
  Narrowing to the declared allowlist needs an egress proxy.
- **`ContainerCapabilities::allows_network` is stricter than
  `check_network`.** The latter treats a document with no `DenyAll` as
  unconstrained — a policy reading. At the enforcement boundary silence must
  mean no, or every existing document would gain connectivity the moment
  capabilities started driving the sandbox.
- **Deletions do not propagate through a partial write grant.** Absence of a
  path in what came back out of a container is indistinguishable from "never
  wrote it", and treating it as a delete would let a restricted writer erase
  what it cannot overwrite.
- **No policy evaluator, sinks, or taint propagation.** Scope here is
  `network` plus volume read/write; the rest of the capability vocabulary
  (`file-read`/`file-write`, secrets, sinks, purposes) is still parsed and
  recorded only.

---

Last LLM verification:
- Date: 2026-08-09
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `crates/hick-token/src/lib.rs` — `ContainerCapabilities::allows_network`
    (deny-by-default; only an explicit `Allow` grants).
  - `crates/hick-exec/src/volume.rs` — `AccessScope`,
    `VolumeDeclaration::read_scope` / `write_scope`; empty `access_rules`
    yields `AccessScope::All`.
  - `crates/hickory-executor/src/lib.rs` — `Executor::declare_capabilities`
    (default: drop); `LocalExecutor` records into `LocalState::declared` and
    exposes `declared_capabilities`.
  - `crates/hickory-executor-docker/src/lib.rs` — `network_mode_for` picks
    `DockerLimits::allowed_network` (`HICKORY_DOCKER_ALLOWED_NETWORK`,
    default `bridge`) for a granted container and `DockerLimits::network`
    (default `none`) otherwise; `start()` passes it to `docker run`.
    `declare_capabilities` accepts a repeat declaration for a container that
    is already running (one executor serves several documents in a row) but
    refuses one that would need a different `--network`, since `docker run`
    flags are fixed at creation.
  - `crates/hick-literate/src/lib.rs` — `prepare_pipeline` folds each
    volume's access rules into the named container's capabilities before
    tokens are minted; `run_pipeline_live` calls `declare_capabilities` for
    every container before the first exec, refuses an unnamed container's
    mount, filters injected archives to a partial read grant, and merges
    extractions through `volume_state::merge_permitted_writes`.
- Test coverage:
  - `crates/hick-literate/tests/capability_enforcement_tests.rs` — all four
    volume clauses in both directions (granted and denied), plus the
    declaration reaching the executor.
  - `crates/hickory-executor-docker/tests/contract.rs`
    `a_containers_network_is_whatever_its_document_declared` — asks the
    daemon for `HostConfig.NetworkMode` of a granted and a denied container.
    Skips when Docker is absent or `HICKORY_DOCKER_NETWORK` is overridden.
    `a_running_container_cannot_be_re_declared_into_a_different_confinement`
    covers the re-declaration rule.
  - `crates/hickory-executor-docker/src/lib.rs`
    `the_document_decides_whether_a_container_has_a_network`,
    `crates/hick-token/src/lib.rs` `allows_network_is_deny_by_default`,
    `crates/hick-exec/src/volume.rs` `access_scopes_…` /
    `a_volume_with_no_rules_grants_everything_to_everyone`.
  - **Not covered by tests:** that a granted container can actually open a
    connection (asserting real egress would make the suite depend on the
    internet); the contract test asserts the daemon-side network mode
    instead. Host/port narrowing is out of scope, not untested.
