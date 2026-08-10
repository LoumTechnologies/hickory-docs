# Containers Are Offline By Default

Given the docker executor, when a container starts, then it runs with
`--network none`, a memory cap, and a PID cap unless the deployment
explicitly opts out.

`hick:deny network="*"` was in the language from the beginning and did nothing
for just as long: capabilities were parsed into `ContainerCapabilities`, and
`LocalExecutor` — which has no sandbox — ignored them. A document could
advertise that its report generator never touches the network, and the claim
was decorative.

Under docker the default inverts. A container gets no network at all, so
"this document cannot exfiltrate" is the resting state and connectivity is the
deliberate exception. Memory and PID caps are
there for a different reason: a hosted deployment runs documents written by
strangers, and untrusted code on cheap compute attracts miners. Per-account
execution-minute quotas already exist in the server, but a container with no
PID limit can take down the host before any quota notices.

**This is the floor, and the document decides who rises above it.** A
container is offline unless its own `<hick:allow network="host:port">` says
otherwise — see
[declared-capabilities-are-enforced](declared-capabilities-are-enforced.md),
which is where the per-container half of this lives.
`HICKORY_DOCKER_NETWORK` still sets what "offline" resolves to for a container
that declared nothing, and `HICKORY_DOCKER_ALLOWED_NETWORK` sets what "online"
resolves to for one that did.

---

Last LLM verification:
- Date: 2026-08-09
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-executor-docker/src/lib.rs` — `DockerLimits`
  defaults to `network: none`, `allowed_network: bridge`, `memory: 512m`,
  `cpus: 1.0`, `pids: 256`, each overridable by `HICKORY_DOCKER_*`;
  `start()` passes memory, cpus, pids and the container's
  `network_mode_for(...)` result to `docker run`, which is `limits.network`
  for every container that did not declare network access.
  `fly.toml` sets `HICKORY_DOCKER_NETWORK = "none"`.
- Test coverage: `contract.rs` `containers_have_no_network_by_default` starts
  a container under the default limits and asserts an outbound `wget` fails;
  `a_containers_network_is_whatever_its_document_declared` covers the
  per-container half that this guarantee previously said was impossible.
