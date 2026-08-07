# Containers Are Offline By Default

Given the docker executor, when a container starts, then it runs with
`--network none`, a memory cap, and a PID cap unless the deployment
explicitly opts out.

`hick:deny network="*"` has been in the language since the beginning and has
never done anything: capabilities are parsed from the document into
`ContainerCapabilities`, and `LocalExecutor` ignores them entirely. A document
could advertise that its report generator never touches the network, and the
claim was decorative.

Under docker the default inverts. A container gets no network at all, so
"this document cannot exfiltrate" is the resting state and connectivity is the
deliberate exception (`HICKORY_DOCKER_NETWORK=bridge`). Memory and PID caps are
there for a different reason: a hosted deployment runs documents written by
strangers, and untrusted code on cheap compute attracts miners. Per-account
execution-minute quotas already exist in the server, but a container with no
PID limit can take down the host before any quota notices.

**This is executor-wide, not per-container.** The document's own
`hick:allow network=` still cannot drive it, because `Executor::ensure_started`
takes only a container name and an image — there is no parameter to carry
capabilities. Making each container's declared capabilities enforceable is a
trait change, and until then the honest description is "the executor is
offline", not "the document's network policy is enforced".

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: partially verified
- Evidence: `crates/hickory-executor-docker/src/lib.rs` — `DockerLimits`
  defaults to `network: none`, `memory: 512m`, `cpus: 1.0`, `pids: 256`, each
  overridable by `HICKORY_DOCKER_*`; `start()` passes all four to
  `docker run`. `fly.toml` sets `HICKORY_DOCKER_NETWORK = "none"`.
- Test coverage: `contract.rs` `containers_have_no_network_by_default` starts
  a container under the default limits and asserts an outbound `wget` fails.
  **Not covered:** per-container enforcement of `hick:allow`/`hick:deny`,
  which the trait cannot express yet — that is the "partially" above.
