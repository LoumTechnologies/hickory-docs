# Programmable Network Stack for Container-to-WASM Runtime

## Problem Statement

We run Linux containers converted to WebAssembly modules using [container2wasm](https://github.com/container2wasm/container2wasm). The WASM module contains a full CPU emulator (x86_64 or RISC-V) running a Linux kernel that boots into a shell. Our Rust runtime (`hick`) executes these modules via [wasmtime](https://github.com/bytecodealliance/wasmtime) and provides programmatic stdin/stdout/stderr I/O.

We need to add **networking** to this runtime, with the ability to:

- **Deep packet inspection** -- see every HTTP request/response in full
- **Block requests** -- prevent specific outbound connections based on rules
- **Modify responses** -- rewrite HTTP response bodies/headers before they reach the container
- **Support HTTP and HTTPS** initially, with extensibility to other protocols later

## How Container2wasm Networking Works

Container2wasm's networking does **not** use WASI's native socket APIs (`wasi:sockets/tcp`, etc.). Instead, the architecture is:

```
Linux kernel (inside WASM)
  has a virtual network interface
    ↓
Emulator exposes raw Ethernet frames via a socket on fd=3
  using QEMU's -netdev socket stream protocol
    ↓
Host-side process receives frames and bridges them to the real network
```

### The QEMU Network Protocol

The QEMU `-netdev socket` stream protocol is simple: raw Ethernet frames are sent over a TCP stream, each prefixed with a **4-byte big-endian length header**.

```
[4 bytes: frame length (big-endian u32)] [N bytes: raw Ethernet frame]
[4 bytes: frame length (big-endian u32)] [N bytes: raw Ethernet frame]
...
```

This is the protocol that gvisor-tap-vsock's `AcceptQemu` function speaks. It's bidirectional -- both sides send and receive length-prefixed Ethernet frames over the same TCP stream.

### The Existing c2w-net Tool (What We're Replacing)

The upstream project provides a Go binary called `c2w-net` that handles the host side. When invoked with `--invoke`, it:

1. Spawns wasmtime with TCP listener flags: `wasmtime run -S preview2=n -S tcplisten=127.0.0.1:1234 --env='LISTEN_FDS=1' -- module.wasm --net=socket sh`
2. Connects to `127.0.0.1:1234` (the TCP socket wasmtime opens via the WASI sock API on fd=3)
3. Runs [gvisor-tap-vsock](https://github.com/containers/gvisor-tap-vsock) over that connection, creating a virtual network: subnet 192.168.127.0/24, gateway 192.168.127.1, VM at 192.168.127.3, with NAT, DHCP, and DNS forwarding

`c2w-net` is an opaque bridge -- it forwards all traffic without inspection. We are building a Rust replacement that sits in the same position but gives us full visibility and control over traffic.

### The `--net=socket` Flag

When the WASM module is started with `--net=socket` as an argument, the emulated Linux kernel is configured to send/receive network packets through WASI fd=3 using the QEMU protocol. Without this flag, networking is disabled entirely inside the container.

### File Descriptor Assignment

By default, the network socket is fd=3. If other pre-opened resources (directories, etc.) are configured, the fd number shifts. The flag `--net=socket=listenfd=N` allows specifying which fd to use. For our use case, fd=3 is sufficient since we don't pre-open directories.

## Our Runtime Architecture (Context for Integration)

Our runtime is a Rust library crate. The key types are:

- **`ContainerConfig`** -- holds a wasmtime `Engine`, compiled `Module`, and pipe buffer size
- **`Container`** -- a running WASM container with async I/O methods (`write_stdin`, `read_stdout`, etc.)
- **`ChannelStdin` / `ChannelStdout`** -- WASI stream adapters that bridge tokio `DuplexStream` pairs to wasmtime-wasi's `StdinStream`/`StdoutStream` traits

The container runs in a background tokio task. The WASI context is built with:

```rust
let wasi = WasiCtxBuilder::new()
    .stdin(channel_stdin)
    .stdout(channel_stdout)
    .stderr(channel_stderr)
    .build_p1();   // WASI Preview 1
```

To add networking, the WasiCtxBuilder needs a pre-opened TCP listener socket passed via `.preopened_socket(...)` or equivalent mechanism so the WASM module gets fd=3 as a listening socket.

**Dependencies:** wasmtime 41.0.0, wasmtime-wasi 41.0.0, tokio 1.x (rt-multi-thread, macros, io-util, sync).

## Design Decisions

### 1. Embedded Rust Network Stack (not shelling out to c2w-net)

**Decision:** Build the network stack in Rust, embedded in the same process as the wasmtime runtime.

**Why:**
- DPI, blocking, and response modification require being in the packet path at L7. A separate opaque bridge process provides no hooks for this.
- In-process means zero IPC overhead and full type safety.
- Shelling out to the Go-based `c2w-net` would mean maintaining a Go dependency, cross-compiling it, and having no programmatic control over traffic.

### 2. smoltcp for the Userspace TCP/IP Stack

**Decision:** Use [smoltcp](https://docs.rs/smoltcp/) as the userspace TCP/IP stack that processes raw Ethernet frames from the QEMU protocol.

**Why:**
- smoltcp is a well-maintained, no-std-compatible Rust TCP/IP stack designed for exactly this use case -- processing raw packets in userspace.
- It accepts raw Ethernet frames via a `Device` trait and provides TCP/UDP socket abstractions.
- It's used in the broader bytecodealliance/Rust embedded ecosystem.
- Alternative (writing raw packet parsing from scratch) is error-prone and unnecessary.

**How it fits:**
- Implement smoltcp's `Device` trait to read/write Ethernet frames from/to the QEMU protocol stream.
- smoltcp manages ARP, IP routing, TCP state machines, DHCP, and DNS.
- TCP connections from the container appear as smoltcp sockets that we can read/write programmatically.

### 3. Transparent Proxy Architecture for L7 Interception

**Decision:** Intercept TCP streams from smoltcp and proxy them through protocol-specific handlers before forwarding to the real network via tokio's TCP/UDP.

**Architecture:**

```
WASM module
  ↕ QEMU protocol (length-prefixed Ethernet frames over TCP socket on fd=3)
QEMU Protocol Parser
  ↕ raw Ethernet frames
smoltcp (userspace TCP/IP stack)
  ↕ TCP streams / UDP datagrams
Protocol Router (dispatches based on port, SNI, content)
  ↕
Protocol Handlers:
  ├── HTTP handler  (inspect/block/modify requests and responses)
  ├── HTTPS handler (TLS MITM: terminate, inspect, re-encrypt)
  ├── DNS handler   (inspect/modify/block DNS queries)
  └── Passthrough   (forward unknown protocols without inspection)
  ↕
Real Network (via tokio::net::TcpStream, UdpSocket)
```

### 4. HTTPS via TLS MITM with rustls + rcgen

**Decision:** For HTTPS interception, perform TLS man-in-the-middle using rustls for TLS termination and rcgen for dynamic certificate generation.

**Why:**
- HTTPS traffic is encrypted. The only way to inspect/modify it is to terminate TLS on our side, inspect the plaintext HTTP, then re-encrypt to the real server.
- rustls is the standard pure-Rust TLS library (no OpenSSL dependency).
- rcgen generates X.509 certificates on the fly, signed by a CA we control.
- The CA certificate must be injected into the container's trust store (in the Alpine image, this means adding it to `/etc/ssl/certs/ca-certificates.crt` before or during container conversion).

**Flow for HTTPS:**
1. Container makes TLS connection to e.g. `api.example.com:443`
2. smoltcp surfaces the TCP stream to our proxy
3. We read the TLS ClientHello, extract the SNI hostname
4. We generate a certificate for `api.example.com` signed by our CA, on the fly
5. We complete TLS handshake with the container using our generated cert
6. We open a real TLS connection to `api.example.com:443`
7. We relay plaintext HTTP between the two, with hooks for inspection/blocking/modification

### 5. Virtual Network Configuration

**Decision:** Mirror the same network layout as c2w-net for compatibility.

| Parameter | Value |
|-----------|-------|
| Subnet | 192.168.127.0/24 |
| Gateway IP | 192.168.127.1 |
| VM (container) IP | 192.168.127.3 |
| VM MAC | 02:00:00:00:00:01 |
| Gateway MAC | 5a:94:ef:e4:0c:dd |
| MTU | 1500 |
| Protocol | QEMU stream (4-byte BE length + Ethernet frame) |

DHCP and DNS must be handled by the network stack (smoltcp or custom handlers) so the container can obtain its IP and resolve hostnames.

### 6. Interceptor Trait for Extensibility

**Decision:** Define a trait-based interceptor API so new protocol handlers can be added without changing the core proxy.

Conceptual shape:

```rust
#[async_trait]
trait Interceptor: Send + Sync {
    /// Decide whether this interceptor handles the connection.
    fn matches(&self, dest_addr: SocketAddr, initial_bytes: &[u8]) -> bool;

    /// Process the connection. The interceptor owns both sides.
    async fn handle(
        &self,
        client_stream: TcpStream,   // from smoltcp (container side)
        dest_addr: SocketAddr,       // where the container wanted to connect
        context: &ConnectionContext, // metadata, rules, etc.
    ) -> Result<()>;
}
```

Initial interceptors: `HttpInterceptor`, `HttpsInterceptor`, `DnsInterceptor`, `PassthroughInterceptor`.

### 7. Async Integration with tokio

**Decision:** The network stack runs as tokio tasks in the same runtime as the container.

- The QEMU protocol reader/writer runs as a dedicated task, shuttling frames between the TCP socket and smoltcp.
- smoltcp's poll loop runs on a configurable interval (e.g., 1ms) in its own task.
- Each intercepted connection spawns a task for proxying.
- This matches the existing architecture where the WASM module already runs as a tokio task.

### 8. WASI P1 Socket Binding

**Decision:** Use wasmtime-wasi's mechanism to pass a pre-opened TCP listener to the WASM module as fd=3.

The WasiCtxBuilder must be configured so the WASM module receives a TCP listener socket. When the emulated kernel starts with `--net=socket`, it connects to this listener and begins sending/receiving QEMU protocol frames. Our network stack accepts this connection and begins processing.

This replaces the upstream approach of passing `-S tcplisten=<addr>` to the wasmtime CLI. Since we embed wasmtime as a library, we do this programmatically via the WasiCtxBuilder API.

## Key Crates

| Crate | Purpose | Version Guidance |
|-------|---------|-----------------|
| `smoltcp` | Userspace TCP/IP stack | Latest stable, with `std` and `medium-ethernet` features |
| `rustls` | TLS termination for HTTPS MITM | Latest stable |
| `rcgen` | Dynamic X.509 certificate generation | Latest stable |
| `hyper` or `httparse` | HTTP/1.1 parsing for request/response inspection | Use `httparse` if you want minimal/zero-copy; `hyper` if you want full HTTP semantics |
| `tokio` | Async runtime (already in use) | 1.x |
| `wasmtime` / `wasmtime-wasi` | WASM runtime (already in use) | 41.0.0 |

## What Needs to Be Built

1. **QEMU protocol codec** -- Parse and emit length-prefixed Ethernet frames over a TCP stream. This is ~30 lines of code (read 4-byte BE u32, then read that many bytes).

2. **smoltcp Device adapter** -- Implement smoltcp's `Device` trait backed by the QEMU protocol codec. Feed received frames into smoltcp, send smoltcp's outbound frames through the codec.

3. **Virtual network services** -- DHCP server (assign 192.168.127.3 to the VM) and DNS forwarder (forward queries to host resolver). These run inside smoltcp.

4. **TCP proxy core** -- Accept TCP connections from smoltcp, route them through interceptors, and forward to the real network via tokio.

5. **HTTP interceptor** -- Parse HTTP/1.1 requests and responses, apply rules (block, modify), forward.

6. **HTTPS interceptor** -- Detect TLS ClientHello, extract SNI, generate cert, terminate TLS, then delegate to HTTP interceptor for the plaintext.

7. **Interceptor trait and registry** -- The extensibility layer for adding new protocol handlers.

8. **Integration with Container** -- Wire the network stack into the existing `ContainerConfig` / `Container::start()` flow. The container start sequence becomes: create TCP listener, pass it to WASI as fd=3, start WASM module with `--net=socket` arg, accept connection on listener, start network stack on that connection.

## Testing Strategy

The existing test suite has a `test_network_request` integration test that runs `wget -q -O - http://example.com/ | head -5` inside the container and checks for HTML content. Once networking is implemented, this test should pass and can be un-ignored.

Additional tests should cover:
- HTTP request interception and blocking
- HTTP response modification
- HTTPS interception (with test CA cert injected into container)
- DNS resolution
- Connection to real external hosts
