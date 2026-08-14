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
pub mod discovery;
pub mod protocol;
pub mod session;

pub use adapter::Adapter;
pub use discovery::{Discovered, discover, known_languages};
pub use session::{
    Breakpoint, BreakpointStatus, Capabilities, Frame, Launch, Mapping, Session, Step, Stopped,
    Variable,
};
