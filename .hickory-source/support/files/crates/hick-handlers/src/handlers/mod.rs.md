<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-handlers/src/handlers/mod.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a89a41ca6a3dd1a1f9cbb2795568cc2152c8c3ce14b70998800e47e9cb2a37b2`.

<book:entry path="crates/hick-handlers/src/handlers/mod.rs">
//! Built-in tag handlers for hick pipelines.
//!
//! This module provides handlers for standard hick tags like copy, cut, paste,
//! val, exec, etc.

mod copy;
mod exclude;
mod exec;
mod paste;
mod script;
mod substitute;
mod transform;
mod val;

pub use copy::{CopyHandler, CutHandler, TranscriptHandler};
pub use exclude::ExcludeHandler;
pub use exec::ExecHandler;
pub use exec::quotable_exec_node;
pub use paste::PasteHandler;
pub use script::ScriptHandler;
pub use substitute::SubstituteHandler;
pub use transform::{CheckHandler, TransformHandler};
pub use val::ValHandler;

use crate::TagRegistry;

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
</book:entry>

</book:doc>
