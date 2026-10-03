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
