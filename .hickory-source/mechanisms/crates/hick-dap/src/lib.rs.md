<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b12ed8d1197a"></a>
## `crates/hick-dap/src/lib.rs`

<a id="f-b12ed8d1197a-adapter"></a>
### `adapter`

Debugging a document's cells, over the Debug Adapter Protocol.
The sibling of `hick-lsp`, and deliberately so: a per-language adapter
found the way language servers are found, positions mapped between the
document and the files it generates by the mapping that already exists,
and one session API serving all three surfaces — the app's debugger, a
document's `<hick:capture>`, and the MCP tools an agent drives.
See `docs/specs/freeform/literate-debugging.md`.

<storycode:copy id="f-b12ed8d1197a-adapter">
//! Debugging a document's cells, over the Debug Adapter Protocol.
//!
//! The sibling of `hick-lsp`, and deliberately so: a per-language adapter
//! found the way language servers are found, positions mapped between the
//! document and the files it generates by the mapping that already exists,
//! and one session API serving all three surfaces — the app's debugger, a
//! document's `<hick:capture>`, and the MCP tools an agent drives.
//!
//! See `docs/specs/freeform/literate-debugging.md`.

pub mod adapter;
</storycode:copy>
<a id="f-b12ed8d1197a-build"></a>
### `build`

<storycode:copy id="f-b12ed8d1197a-build">
pub mod build;
</storycode:copy>
<a id="f-b12ed8d1197a-capture"></a>
### `capture`

<storycode:copy id="f-b12ed8d1197a-capture">
pub mod capture;
</storycode:copy>
<a id="f-b12ed8d1197a-discovery"></a>
### `discovery`

<storycode:copy id="f-b12ed8d1197a-discovery">
pub mod discovery;
</storycode:copy>
<a id="f-b12ed8d1197a-java"></a>
### `java`

<storycode:copy id="f-b12ed8d1197a-java">
pub mod java;
</storycode:copy>
<a id="f-b12ed8d1197a-mapping"></a>
### `mapping`

<storycode:copy id="f-b12ed8d1197a-mapping">
mod mapping;
</storycode:copy>
<a id="f-b12ed8d1197a-program"></a>
### `program`

<storycode:copy id="f-b12ed8d1197a-program">
pub mod program;
</storycode:copy>
<a id="f-b12ed8d1197a-project"></a>
### `project`

<storycode:copy id="f-b12ed8d1197a-project">
pub mod project;
</storycode:copy>
<a id="f-b12ed8d1197a-protocol"></a>
### `protocol`

<storycode:copy id="f-b12ed8d1197a-protocol">
pub mod protocol;
</storycode:copy>
<a id="f-b12ed8d1197a-session"></a>
### `session`

<storycode:copy id="f-b12ed8d1197a-session">
pub mod session;
</storycode:copy>
<a id="f-b12ed8d1197a-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-b12ed8d1197a-use_declaration">
pub use mapping::Mapping;

</storycode:copy>
<a id="f-b12ed8d1197a-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-b12ed8d1197a-use_declaration-2">
pub use adapter::{Adapter, Transport};
</storycode:copy>
<a id="f-b12ed8d1197a-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-b12ed8d1197a-use_declaration-3">
pub use build::{BuildOutput, build, build_plain, is_compiled};
</storycode:copy>
<a id="f-b12ed8d1197a-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-b12ed8d1197a-use_declaration-4">
pub use capture::{CaptureSpec, Captured, Hit};
</storycode:copy>
<a id="f-b12ed8d1197a-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-b12ed8d1197a-use_declaration-5">
pub use discovery::{Discovered, discover, how_to_get, known_languages, suggests_hick_install};
</storycode:copy>
<a id="f-b12ed8d1197a-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-b12ed8d1197a-use_declaration-6">
pub use program::{MissingAdapter, adapter_for, entry_point, language_of, weave_into};
</storycode:copy>
<a id="f-b12ed8d1197a-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-b12ed8d1197a-use_declaration-7">
pub use session::{
    BindState, Breakpoint, BreakpointStatus, Capabilities, Exit, Frame, Launch, Session,
    SourceBreakpoints, Step, Stopped, Variable,
};

</storycode:copy>
<a id="f-b12ed8d1197a-use_declaration-8"></a>
### `use_declaration`

<storycode:copy id="f-b12ed8d1197a-use_declaration-8">
pub use project::{ProjectLaunch, prepare as prepare_project, prepare_closure};
</storycode:copy>

### Assemble `crates/hick-dap/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/lib.rs"><storycode:paste select="#f-b12ed8d1197a-adapter"/><storycode:paste select="#f-b12ed8d1197a-build"/><storycode:paste select="#f-b12ed8d1197a-capture"/><storycode:paste select="#f-b12ed8d1197a-discovery"/><storycode:paste select="#f-b12ed8d1197a-java"/><storycode:paste select="#f-b12ed8d1197a-mapping"/><storycode:paste select="#f-b12ed8d1197a-program"/><storycode:paste select="#f-b12ed8d1197a-project"/><storycode:paste select="#f-b12ed8d1197a-protocol"/><storycode:paste select="#f-b12ed8d1197a-session"/><storycode:paste select="#f-b12ed8d1197a-use_declaration"/><storycode:paste select="#f-b12ed8d1197a-use_declaration-2"/><storycode:paste select="#f-b12ed8d1197a-use_declaration-3"/><storycode:paste select="#f-b12ed8d1197a-use_declaration-4"/><storycode:paste select="#f-b12ed8d1197a-use_declaration-5"/><storycode:paste select="#f-b12ed8d1197a-use_declaration-6"/><storycode:paste select="#f-b12ed8d1197a-use_declaration-7"/><storycode:paste select="#f-b12ed8d1197a-use_declaration-8"/></storycode:file>

</storycode:doc>
