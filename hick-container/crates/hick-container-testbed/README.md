# container-testbed

Interactive terminal binary for manually exercising the container system.

Presents a raw-mode REPL where meta-commands (`:from <image>`, `:switch`,
`:kill`, `:ls`) launch and manage multiple named WASM containers through a
`ContainerManager`.

Also implements an `InteractiveFilter` that intercepts outbound network
connections in real time, prompting the operator to allow or block each
destination before the connection is established.

Primary use: manual QA and exploratory testing of the `container` crate's
networking and I/O plumbing.
