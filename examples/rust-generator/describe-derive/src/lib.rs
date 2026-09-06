//! `#[derive(Describe)]` — a marker and nothing else.
//!
//! It expands to nothing at all. That is the point: the mark is data for a
//! GENERATOR, not behaviour for the compiler, and a derive that wrote the
//! `describe` function itself would put the interesting decision inside a
//! macro nobody can read the output of. The generator writes an ordinary
//! file instead, the document owns its bytes, and the drift gate checks it.

use proc_macro::TokenStream;

/// Mark a type for `describegen`.
#[proc_macro_derive(Describe)]
pub fn describe(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
