<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/workspace_fs/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6275b6243a9b"></a>
## `crates/hickory-cli/src/serve/workspace_fs/mod.rs`

<a id="f-6275b6243a9b-engine"></a>
### `engine`

Frontend-independent workspace I/O. A mount and protocol clients share these
revision checks, buffered saves, and the language's existing reverse mapping.

<storycode:copy id="f-6275b6243a9b-engine">
//! Frontend-independent workspace I/O. A mount and protocol clients share these
//! revision checks, buffered saves, and the language's existing reverse mapping.
mod engine;
</storycode:copy>
<a id="f-6275b6243a9b-host"></a>
### `host`

<storycode:copy id="f-6275b6243a9b-host">
mod host;
</storycode:copy>
<a id="f-6275b6243a9b-native"></a>
### `native`

<storycode:copy id="f-6275b6243a9b-native">
mod native;
</storycode:copy>
<a id="f-6275b6243a9b-smoke"></a>
### `smoke`

<storycode:copy id="f-6275b6243a9b-smoke">
mod smoke;
</storycode:copy>
<a id="f-6275b6243a9b-view"></a>
### `view`

<storycode:copy id="f-6275b6243a9b-view">
pub(crate) mod view;
</storycode:copy>
<a id="f-6275b6243a9b-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-6275b6243a9b-use_declaration">
pub use engine::Engine;
</storycode:copy>
<a id="f-6275b6243a9b-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-6275b6243a9b-use_declaration-2">
pub use host::Host;
</storycode:copy>
<a id="f-6275b6243a9b-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-6275b6243a9b-use_declaration-3">
pub use native::{Mount, availability};
</storycode:copy>
<a id="f-6275b6243a9b-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-6275b6243a9b-use_declaration-4">
pub use smoke::smoke_argv;
</storycode:copy>
<a id="f-6275b6243a9b-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-6275b6243a9b-use_declaration-5">
pub use view::{Entry, View};
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/workspace_fs/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/workspace_fs/mod.rs"><storycode:paste select="#f-6275b6243a9b-engine"/><storycode:paste select="#f-6275b6243a9b-host"/><storycode:paste select="#f-6275b6243a9b-native"/><storycode:paste select="#f-6275b6243a9b-smoke"/><storycode:paste select="#f-6275b6243a9b-view"/><storycode:paste select="#f-6275b6243a9b-use_declaration"/><storycode:paste select="#f-6275b6243a9b-use_declaration-2"/><storycode:paste select="#f-6275b6243a9b-use_declaration-3"/><storycode:paste select="#f-6275b6243a9b-use_declaration-4"/><storycode:paste select="#f-6275b6243a9b-use_declaration-5"/></storycode:file>

</storycode:doc>
