# c2w

Container-to-WASM conversion library (the VM strategy backend).

Converts an OCI container image into a self-contained WASM module that can run
inside the hick container runtime.

## Two pipelines

- **Docker-based** — invokes `docker buildx` to build the WASM image
- **No-Docker** — `assembler::build_without_docker` uses `oci-pull` to fetch
  the image directly and assembles the WASM without a Docker daemon

Supports optional Wizer pre-initialization post-build. The `wizer-lib` feature
swaps the CLI `wizer` invocation for library-based pre-initialization via
`wasmtime`.

## Key modules

`builder`, `assembler`, `image`, `rootfs`, `spec`, `template`, `wizer`, `iso`
