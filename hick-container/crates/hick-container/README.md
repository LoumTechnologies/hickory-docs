# container

Core WASM sandbox runtime.

Wraps a `wasmtime` module in a background Tokio task, bridging WASI
stdin/stdout/stderr to OS Unix pipe pairs.

## Features

- **Pause/resume** via epoch interruption (a separate epoch-tick task is spawned
  and aborted on pause)
- **Networking** via a `ConnectionFilter`-gated TCP proxy with optional
  inter-container routing through a shared `NetworkBridge`
- **Filesystem** via preopened WASI directories

## Public API

```rust
Container::start(config)    // launch
container.write_stdin(data)
container.read_stdout()
container.pause() / resume()
container.kill()
container.wait()
container.stats()           // CPU/memory via ContainerStats
```
