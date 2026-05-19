# wasm-test-tool

Minimal WASI binary for testing hick-shell WASM command dispatch.

Prints all command-line arguments (excluding `argv[0]`) separated by spaces and
exits with code 0. Used as a stand-in command in the WASM toolchain to verify
that argument passing through the shell interpreter works correctly.
