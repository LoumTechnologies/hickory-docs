# hick-agent-core

Thin re-export shim that consolidates the public API surface used by binary
crates.

Re-exports `CommandExecutor`, `WasmSandboxConfig`, and `WasmSandboxExecutor`
from its own `command` module (which has a slightly different API shape than the
SDK's counterparts), plus Hickory-aware edit types (`HickEditFormat`,
`HickEditService`, `PipelineOutput`, `QueryMatch`, `SourceEdit`) from
`hick-agent-hick-edit`.

The distinction from `hick-agent-sdk` is that this crate knows about the hick
pipeline. It exists so binaries can import from one place rather than
two separate crates.
