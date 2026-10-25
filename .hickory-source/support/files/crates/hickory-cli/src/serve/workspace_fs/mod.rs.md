<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/workspace_fs/mod.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `474526e81919a7658a756098adc0007be4a8f64c0d36de9a6e34987f7340e947`.

<book:entry path="crates/hickory-cli/src/serve/workspace_fs/mod.rs">
//! Frontend-independent workspace I/O. A mount and protocol clients share these
//! revision checks, buffered saves, and the language's existing reverse mapping.
mod engine;
mod host;
mod native;
mod smoke;
pub(crate) mod view;
pub use engine::Engine;
pub use host::Host;
pub use native::{Mount, availability};
pub use smoke::smoke_argv;
pub use view::{Entry, View};
</book:entry>

</book:doc>
