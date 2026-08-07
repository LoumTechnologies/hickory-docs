# The Declared Image Is the Environment

Given a document that declares `<hick:container image="python:3.12">`, when it
runs under the docker executor, then its cells execute inside that image — not
against whatever toolchain the host happens to have.

Reproducibility is the product's central claim, and it was not true. Two
separate failures stacked:

`LocalExecutor` records `image=` and ignores it, which is documented. But the
**pipeline never carried the image at all**: `container_defs` collected
capabilities from each `<hick:container>` tag and dropped its `image`
attribute on the floor, so the only image an executor ever saw came from an
`image=` on the `<hick:exec>` tag itself. Every shipped example declares its
image on the container — `examples/grand-tour.hick` asks for `python:3.12` and
`duckdb/duckdb:v1.5.5` — so all of them were silently running on the host.
Nothing surfaced it, because the one executor in use ignored images anyway.

Resolution order is now: an `image=` on the exec, then the container's
declaration, then the historical `alpine` default. A fork inherits its
source's image unless it declares its own.

The consequence is that `hickory check` finally means what it says. A document
verified under the docker executor is verified for the environment it names,
so two machines that agree on the image agree on the result.

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-literate/src/lib.rs` — `prepare_pipeline` collects
  `container_images` alongside `container_defs` from the same tag and carries
  it on `PreparedPipeline`; both the live and weave paths resolve
  `exec.image → container_images[container] → DEFAULT_IMAGE`. Fork targets
  inherit the source's image via `container_images.entry(to).or_insert(..)`.
  `crates/hickory-executor-docker` runs each container from the resolved
  image with `docker run`.
- Test coverage: `crates/hickory-executor-docker/tests/contract.rs`
  `the_declared_image_is_the_environment` asserts `python:3.12-alpine` reports
  Python 3.12 on a host running a different version. Driven end to end: a
  document declaring `python:3.12-alpine` and `python:3.10-alpine` in two
  containers passes both pinned expectations under
  `HICKORY_EXECUTOR=docker`, and fails both under `HICKORY_EXECUTOR=local`
  (which reports the host's 3.14 for each).
