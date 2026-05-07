// Phase 1: Pipe wiring between WASM instances in a pipeline.
//
// For `cmd1 | cmd2 | cmd3`:
// - Create N-1 pipe pairs for N commands
// - Wire stdout of each command to stdin of the next
// - Spawn all concurrently via `tokio::JoinSet`
// - Return exit code of last command
