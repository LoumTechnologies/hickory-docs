//! Formulas in a table, evaluated by whatever language you write them in.
//!
//! # Why this is a protocol and not an expression language
//!
//! Every spreadsheet ships its own formula language, and every one of them is
//! a small, badly-specified programming language that people then have to
//! learn twice: once for the spreadsheet, once for the language they actually
//! work in. This product already runs Python, JavaScript, Rust and shell in
//! the same document. Inventing a fifth, worse language for the tables would
//! be a strange thing to do.
//!
//! So a formula is **an expression in a real language**, and evaluating one is
//! a request to a small program that speaks that language — the same shape
//! `hick-lsp` uses for intelligence and `hick-dap` uses for debugging, for the
//! same reason: the thing that knows Python is Python.
//!
//! # The split, which is the whole design
//!
//! The **host** owns everything that is language-independent:
//!
//! * what a cell reference is (`A1`, `B2:B9`),
//! * which cells a formula depends on,
//! * what order they have to be evaluated in,
//! * what a cycle is and how it is reported.
//!
//! The **backend** owns exactly one thing: given an expression and the values
//! its references resolved to, produce a value or an error.
//!
//! That split is not an implementation detail, it is what makes "any
//! language" affordable. A backend is an evaluator — sixty lines — rather
//! than a spreadsheet engine, so adding a language is a small, obviously
//! correct program instead of a second chance to get topological sorting
//! subtly wrong. It is also what makes the ORDER of evaluation identical
//! whatever language a document mixes, which a per-backend graph could never
//! promise.
//!
//! # No network, ever
//!
//! A backend is a script this crate writes into the project's cache the first
//! time it is needed, and runs with an interpreter that is already on the
//! machine. There is nothing to download: `hick formula install` is a local
//! `write` and a `--version` check. A machine with Python has Python formulas
//! and a machine without says so.

pub mod backend;
pub mod evaluate;
pub mod graph;
pub mod protocol;
pub mod session;

/// Which backend serves a language, and getting one onto the machine.
pub use backend::{Backend, backend_for, install, installed_languages};
/// Evaluating a whole table, where the host's graph meets a backend.
pub use evaluate::{Computed, Step, Trace, available, evaluate_sheet, levels, trace_sheet};
/// A cell reference and the sheet arithmetic around it.
pub use graph::{CellRef, Sheet, evaluation_order, references_in};
/// The wire format between the host and a backend.
pub use protocol::{EvalRequest, EvalResponse, Formula, FormulaError, FormulaResult, Value};
/// A running backend.
pub use session::Session;
