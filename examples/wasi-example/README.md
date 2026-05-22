# wasi-example

Minimal WASI binary used as an integration fixture.

Reads a line from stdin in the form `<number> <op> <number>`, evaluates the
arithmetic operation (`+`, `-`, `*`, `/`), and prints the result.

Used in tests to verify that the hick container execution and WASI plumbing work
end-to-end with a simple, known-correct WASM workload.
