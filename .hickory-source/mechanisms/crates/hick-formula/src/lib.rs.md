<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-formula/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0d41e38bffcc"></a>
## `crates/hick-formula/src/lib.rs`

<a id="f-0d41e38bffcc-backend"></a>
### `backend`

Formulas in a table, evaluated by whatever language you write them in.
# Why this is a protocol and not an expression language
Every spreadsheet ships its own formula language, and every one of them is
a small, badly-specified programming language that people then have to
learn twice: once for the spreadsheet, once for the language they actually
work in. This product already runs Python, JavaScript, Rust and shell in
the same document. Inventing a fifth, worse language for the tables would
be a strange thing to do.
So a formula is **an expression in a real language**, and evaluating one is
a request to a small program that speaks that language — the same shape
`hick-lsp` uses for intelligence and `hick-dap` uses for debugging, for the
same reason: the thing that knows Python is Python.
# The split, which is the whole design
The **host** owns everything that is language-independent:
* what a cell reference is (`A1`, `B2:B9`),
* which cells a formula depends on,
* what order they have to be evaluated in,
* what a cycle is and how it is reported.
The **backend** owns exactly one thing: given an expression and the values
its references resolved to, produce a value or an error.
That split is not an implementation detail, it is what makes "any
language" affordable. A backend is an evaluator — sixty lines — rather
than a spreadsheet engine, so adding a language is a small, obviously
correct program instead of a second chance to get topological sorting
subtly wrong. It is also what makes the ORDER of evaluation identical
whatever language a document mixes, which a per-backend graph could never
promise.
# No network, ever
A backend is a script this crate writes into the project's cache the first
time it is needed, and runs with an interpreter that is already on the
machine. There is nothing to download: `hick formula install` is a local
`write` and a `--version` check. A machine with Python has Python formulas
and a machine without says so.

<storycode:copy id="f-0d41e38bffcc-backend">
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
</storycode:copy>
<a id="f-0d41e38bffcc-evaluate"></a>
### `evaluate`

<storycode:copy id="f-0d41e38bffcc-evaluate">
pub mod evaluate;
</storycode:copy>
<a id="f-0d41e38bffcc-graph"></a>
### `graph`

<storycode:copy id="f-0d41e38bffcc-graph">
pub mod graph;
</storycode:copy>
<a id="f-0d41e38bffcc-protocol"></a>
### `protocol`

<storycode:copy id="f-0d41e38bffcc-protocol">
pub mod protocol;
</storycode:copy>
<a id="f-0d41e38bffcc-session"></a>
### `session`

<storycode:copy id="f-0d41e38bffcc-session">
pub mod session;

</storycode:copy>
<a id="f-0d41e38bffcc-use_declaration"></a>
### `use_declaration`

Which backend serves a language, and getting one onto the machine.

<storycode:copy id="f-0d41e38bffcc-use_declaration">
/// Which backend serves a language, and getting one onto the machine.
pub use backend::{Backend, backend_for, install, installed_languages};
</storycode:copy>
<a id="f-0d41e38bffcc-use_declaration-2"></a>
### `use_declaration`

Evaluating a whole table, where the host's graph meets a backend.

<storycode:copy id="f-0d41e38bffcc-use_declaration-2">
/// Evaluating a whole table, where the host's graph meets a backend.
pub use evaluate::{Computed, Step, Trace, available, evaluate_sheet, levels, trace_sheet};
</storycode:copy>
<a id="f-0d41e38bffcc-use_declaration-3"></a>
### `use_declaration`

A cell reference and the sheet arithmetic around it.

<storycode:copy id="f-0d41e38bffcc-use_declaration-3">
/// A cell reference and the sheet arithmetic around it.
pub use graph::{CellRange, CellRef, Sheet, evaluation_order, ranges_in, references_in};
</storycode:copy>
<a id="f-0d41e38bffcc-use_declaration-4"></a>
### `use_declaration`

The wire format between the host and a backend.

<storycode:copy id="f-0d41e38bffcc-use_declaration-4">
/// The wire format between the host and a backend.
pub use protocol::{EvalRequest, EvalResponse, Formula, FormulaError, FormulaResult, Value};
</storycode:copy>
<a id="f-0d41e38bffcc-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-0d41e38bffcc-use_declaration-5">
/// A running backend.
pub use session::Session;
</storycode:copy>

### Assemble `crates/hick-formula/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-formula/src/lib.rs"><storycode:paste select="#f-0d41e38bffcc-backend"/><storycode:paste select="#f-0d41e38bffcc-evaluate"/><storycode:paste select="#f-0d41e38bffcc-graph"/><storycode:paste select="#f-0d41e38bffcc-protocol"/><storycode:paste select="#f-0d41e38bffcc-session"/><storycode:paste select="#f-0d41e38bffcc-use_declaration"/><storycode:paste select="#f-0d41e38bffcc-use_declaration-2"/><storycode:paste select="#f-0d41e38bffcc-use_declaration-3"/><storycode:paste select="#f-0d41e38bffcc-use_declaration-4"/><storycode:paste select="#f-0d41e38bffcc-use_declaration-5"/></storycode:file>

</storycode:doc>
