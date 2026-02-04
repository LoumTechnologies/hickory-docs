//! Built-in tag handlers for hick pipelines.
//!
//! This module provides handlers for standard hick tags like copy, cut, paste,
//! val, exec, etc.

mod copy;
mod paste;
mod val;

pub use copy::{CopyHandler, CutHandler};
pub use paste::PasteHandler;
pub use val::ValHandler;

use crate::TagRegistry;

/// Register all built-in handlers with a registry.
pub fn register_builtins(registry: &mut TagRegistry) {
    registry.register(Box::new(CopyHandler));
    registry.register(Box::new(CutHandler));
    registry.register(Box::new(PasteHandler));
    registry.register(Box::new(ValHandler));
}
