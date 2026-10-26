<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-handlers/src/handlers/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d08d3401eedf"></a>
## `crates/hick-handlers/src/handlers/mod.rs`

<a id="f-d08d3401eedf-copy"></a>
### `copy`

Built-in tag handlers for hick pipelines.
This module provides handlers for standard hick tags like copy, cut, paste,
val, exec, etc.

<storycode:copy id="f-d08d3401eedf-copy">
//! Built-in tag handlers for hick pipelines.
//!
//! This module provides handlers for standard hick tags like copy, cut, paste,
//! val, exec, etc.

mod copy;
</storycode:copy>
<a id="f-d08d3401eedf-exclude"></a>
### `exclude`

<storycode:copy id="f-d08d3401eedf-exclude">
mod exclude;
</storycode:copy>
<a id="f-d08d3401eedf-exec"></a>
### `exec`

<storycode:copy id="f-d08d3401eedf-exec">
mod exec;
</storycode:copy>
<a id="f-d08d3401eedf-paste"></a>
### `paste`

<storycode:copy id="f-d08d3401eedf-paste">
mod paste;
</storycode:copy>
<a id="f-d08d3401eedf-script"></a>
### `script`

<storycode:copy id="f-d08d3401eedf-script">
mod script;
</storycode:copy>
<a id="f-d08d3401eedf-substitute"></a>
### `substitute`

<storycode:copy id="f-d08d3401eedf-substitute">
mod substitute;
</storycode:copy>
<a id="f-d08d3401eedf-transform"></a>
### `transform`

<storycode:copy id="f-d08d3401eedf-transform">
mod transform;
</storycode:copy>
<a id="f-d08d3401eedf-val"></a>
### `val`

<storycode:copy id="f-d08d3401eedf-val">
mod val;

</storycode:copy>
<a id="f-d08d3401eedf-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration">
pub use copy::{CopyHandler, CutHandler, TranscriptHandler};
</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-2">
pub use exclude::ExcludeHandler;
</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-3">
pub use exec::ExecHandler;
</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-4">
pub use exec::quotable_exec_node;
</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-5">
pub use paste::PasteHandler;
</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-6">
pub use script::ScriptHandler;
</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-7">
pub use substitute::SubstituteHandler;
</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-8"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-8">
pub use transform::{CheckHandler, TransformHandler};
</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-9"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-9">
pub use val::ValHandler;

</storycode:copy>
<a id="f-d08d3401eedf-use_declaration-10"></a>
### `use_declaration`

<storycode:copy id="f-d08d3401eedf-use_declaration-10">
use crate::TagRegistry;

</storycode:copy>
<a id="f-d08d3401eedf-register_builtins"></a>
### `register_builtins`

Register all built-in handlers with a registry.

<storycode:copy id="f-d08d3401eedf-register_builtins">
/// Register all built-in handlers with a registry.
pub fn register_builtins(registry: &mut TagRegistry) {
    registry.register(Box::new(CopyHandler));
    registry.register(Box::new(CutHandler));
    registry.register(Box::new(TranscriptHandler));
    registry.register(Box::new(ExcludeHandler));
    registry.register(Box::new(ExecHandler));
    registry.register(Box::new(PasteHandler));
    registry.register(Box::new(ScriptHandler));
    registry.register(Box::new(SubstituteHandler));
    registry.register(Box::new(TransformHandler));
    registry.register(Box::new(CheckHandler));
    registry.register(Box::new(ValHandler));
}
</storycode:copy>

### Assemble `crates/hick-handlers/src/handlers/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-handlers/src/handlers/mod.rs"><storycode:paste select="#f-d08d3401eedf-copy"/><storycode:paste select="#f-d08d3401eedf-exclude"/><storycode:paste select="#f-d08d3401eedf-exec"/><storycode:paste select="#f-d08d3401eedf-paste"/><storycode:paste select="#f-d08d3401eedf-script"/><storycode:paste select="#f-d08d3401eedf-substitute"/><storycode:paste select="#f-d08d3401eedf-transform"/><storycode:paste select="#f-d08d3401eedf-val"/><storycode:paste select="#f-d08d3401eedf-use_declaration"/><storycode:paste select="#f-d08d3401eedf-use_declaration-2"/><storycode:paste select="#f-d08d3401eedf-use_declaration-3"/><storycode:paste select="#f-d08d3401eedf-use_declaration-4"/><storycode:paste select="#f-d08d3401eedf-use_declaration-5"/><storycode:paste select="#f-d08d3401eedf-use_declaration-6"/><storycode:paste select="#f-d08d3401eedf-use_declaration-7"/><storycode:paste select="#f-d08d3401eedf-use_declaration-8"/><storycode:paste select="#f-d08d3401eedf-use_declaration-9"/><storycode:paste select="#f-d08d3401eedf-use_declaration-10"/><storycode:paste select="#f-d08d3401eedf-register_builtins"/></storycode:file>

</storycode:doc>
