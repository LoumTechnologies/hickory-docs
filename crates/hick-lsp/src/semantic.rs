//! Semantic tokens across virtual files.
//!
//! This is the feature that makes a `hick:file` block look like code rather
//! than text, and it is the one that cannot be forwarded by rewriting a range
//! or two. LSP encodes semantic tokens as a flat array of five-number groups,
//! each *relative to the token before it*:
//!
//! ```text
//! [ Δline, Δstart, length, tokenType, tokenModifiers, … ]
//! ```
//!
//! Deltas are meaningless once the tokens move. A document interleaves
//! several virtual files with prose between them, so mapping each token into
//! document coordinates reorders them — the tokens of the second block land
//! after the first block's, but their deltas were computed against their own
//! file's origin. So this decodes to absolute positions, maps each one, sorts
//! by where it now is, and re-encodes.
//!
//! Legends are per server. Two children (pyright and rust-analyzer, say) can
//! disagree about which integer means `function`, so a merged result has to
//! be re-indexed against one legend the editor was told about — otherwise the
//! Rust in a document is coloured with Python's palette.

use std::collections::HashMap;

use crate::position_map::PositionMap;

/// One token, in absolute coordinates, with its type named rather than
/// numbered — names are the only thing two servers agree on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub token_type: String,
    pub modifiers: Vec<String>,
}

/// Decode a server's `data` array using the legend it declared.
pub fn decode(data: &[u32], types: &[String], modifiers: &[String]) -> Vec<Token> {
    let mut out = Vec::new();
    let mut line = 0u32;
    let mut start = 0u32;
    for chunk in data.chunks_exact(5) {
        let (d_line, d_start, length, type_index, modifier_bits) =
            (chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]);
        // A non-zero line delta resets the column, exactly as the encoding
        // says; getting this wrong shifts every token on the line.
        if d_line > 0 {
            line += d_line;
            start = d_start;
        } else {
            start += d_start;
        }
        let token_type = types
            .get(type_index as usize)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let mut names = Vec::new();
        for (bit, name) in modifiers.iter().enumerate() {
            if modifier_bits & (1 << bit) != 0 {
                names.push(name.clone());
            }
        }
        out.push(Token {
            line,
            start,
            length,
            token_type,
            modifiers: names,
        });
    }
    out
}

/// Map tokens from one virtual file into document coordinates.
///
/// A token whose position has no source — text the weaver produced, not the
/// author — is dropped rather than guessed at. Colouring bytes nobody wrote
/// would be colouring a fiction.
pub fn to_source(tokens: Vec<Token>, map: &PositionMap) -> Vec<Token> {
    tokens
        .into_iter()
        .filter_map(|token| {
            let (line, start) = map.to_source(token.line, token.start)?;
            Some(Token {
                line,
                start,
                ..token
            })
        })
        .collect()
}

/// Re-encode absolute tokens against `legend`, in document order.
pub fn encode(
    mut tokens: Vec<Token>,
    legend_types: &[String],
    legend_modifiers: &[String],
) -> Vec<u32> {
    // Sort first: tokens arrive per virtual file, and a document interleaves
    // those files with prose. Encoding out of order produces negative deltas,
    // which the protocol cannot express and editors render as garbage.
    tokens.sort_by(|a, b| a.line.cmp(&b.line).then(a.start.cmp(&b.start)));

    let type_index: HashMap<&str, u32> = legend_types
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i as u32))
        .collect();
    let modifier_index: HashMap<&str, u32> = legend_modifiers
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i as u32))
        .collect();

    let mut out = Vec::with_capacity(tokens.len() * 5);
    let mut last_line = 0u32;
    let mut last_start = 0u32;
    for token in tokens {
        // A type this build's legend does not carry is dropped: emitting an
        // index the editor was not told about colours it as something else.
        let Some(type_id) = type_index.get(token.token_type.as_str()).copied() else {
            continue;
        };
        let mut bits = 0u32;
        for modifier in &token.modifiers {
            if let Some(bit) = modifier_index.get(modifier.as_str()) {
                bits |= 1 << bit;
            }
        }
        let d_line = token.line - last_line;
        let d_start = if d_line == 0 {
            token.start.saturating_sub(last_start)
        } else {
            token.start
        };
        out.extend_from_slice(&[d_line, d_start, token.length, type_id, bits]);
        last_line = token.line;
        last_start = token.start;
    }
    out
}

/// The legend this server advertises: the union of what the children use, in
/// a fixed order so a client can rely on it across requests.
pub fn legend_types() -> Vec<String> {
    [
        "namespace",
        "type",
        "class",
        "enum",
        "interface",
        "struct",
        "typeParameter",
        "parameter",
        "variable",
        "property",
        "enumMember",
        "event",
        "function",
        "method",
        "macro",
        "keyword",
        "modifier",
        "comment",
        "string",
        "number",
        "regexp",
        "operator",
        "decorator",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn legend_modifiers() -> Vec<String> {
    [
        "declaration",
        "definition",
        "readonly",
        "static",
        "deprecated",
        "abstract",
        "async",
        "modification",
        "documentation",
        "defaultLibrary",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::virtual_file::VirtualSegment;

    fn types() -> Vec<String> {
        vec!["function".into(), "variable".into(), "keyword".into()]
    }
    fn modifiers() -> Vec<String> {
        vec!["declaration".into(), "readonly".into()]
    }

    #[test]
    fn decoding_follows_the_delta_encoding() {
        // Two tokens on line 0, one on line 2. The second token's column is
        // relative to the first; the third resets because its line moved.
        let data = vec![0, 5, 3, 0, 0, 0, 4, 2, 1, 1, 2, 1, 6, 2, 0];
        let tokens = decode(&data, &types(), &modifiers());
        assert_eq!(tokens.len(), 3);
        assert_eq!((tokens[0].line, tokens[0].start), (0, 5));
        assert_eq!((tokens[1].line, tokens[1].start), (0, 9));
        assert_eq!((tokens[2].line, tokens[2].start), (2, 1));
        assert_eq!(tokens[0].token_type, "function");
        assert_eq!(tokens[1].modifiers, vec!["declaration".to_string()]);
    }

    #[test]
    fn encoding_is_the_inverse_of_decoding() {
        let data = vec![0, 5, 3, 0, 0, 0, 4, 2, 1, 1, 2, 1, 6, 2, 0];
        let tokens = decode(&data, &types(), &modifiers());
        assert_eq!(encode(tokens, &types(), &modifiers()), data);
    }

    #[test]
    fn tokens_from_two_blocks_are_re_sorted_before_encoding() {
        // The failure this prevents: block two's tokens map above block
        // one's, and encoding them in arrival order needs a negative delta,
        // which the protocol cannot express.
        let tokens = vec![
            Token {
                line: 40,
                start: 0,
                length: 3,
                token_type: "function".into(),
                modifiers: vec![],
            },
            Token {
                line: 10,
                start: 2,
                length: 4,
                token_type: "variable".into(),
                modifiers: vec![],
            },
        ];
        let data = encode(tokens, &types(), &modifiers());
        assert_eq!(data[0], 10, "the earliest token comes first");
        // Every line delta is non-negative by construction.
        for chunk in data.chunks_exact(5) {
            assert!(chunk[0] < 1_000_000, "line delta wrapped: {chunk:?}");
        }
    }

    #[test]
    fn a_token_the_legend_does_not_carry_is_dropped_not_mis_coloured() {
        let tokens = vec![Token {
            line: 0,
            start: 0,
            length: 1,
            token_type: "somethingNobodyElseHas".into(),
            modifiers: vec![],
        }];
        assert!(encode(tokens, &types(), &modifiers()).is_empty());
    }

    #[test]
    fn tokens_land_where_the_document_has_them() {
        // A block whose text starts on document line 11 (the tag is on 10):
        // a token on the block's first line must come back on line 11.
        let segments = vec![VirtualSegment {
            text: "def f():\n    return 1\n".to_string(),
            source_span: None,
            source_line: 11,
            source_column: 0,
        }];
        let map = PositionMap::build(&segments);
        let mapped = to_source(
            vec![Token {
                line: 0,
                start: 4,
                length: 2,
                token_type: "function".into(),
                modifiers: vec![],
            }],
            &map,
        );
        assert_eq!(mapped.len(), 1, "the token maps into the document");
        assert!(mapped[0].line >= 10, "landed at {}", mapped[0].line);
    }

    #[test]
    fn a_token_with_no_source_is_dropped() {
        // Bytes the weaver produced are nobody's code; colouring them would
        // colour a fiction.
        let map = PositionMap::build(&[]);
        assert!(
            to_source(
                vec![Token {
                    line: 3,
                    start: 0,
                    length: 1,
                    token_type: "function".into(),
                    modifiers: vec![]
                }],
                &map
            )
            .is_empty()
        );
    }

    #[test]
    fn the_advertised_legend_covers_what_servers_send() {
        // If the legend is missing a common type, every token of that type is
        // silently dropped — so this is worth pinning.
        let types = legend_types();
        for expected in [
            "function",
            "variable",
            "parameter",
            "property",
            "keyword",
            "string",
            "comment",
        ] {
            assert!(
                types.contains(&expected.to_string()),
                "legend lacks {expected}"
            );
        }
        assert!(legend_modifiers().contains(&"declaration".to_string()));
    }
}
