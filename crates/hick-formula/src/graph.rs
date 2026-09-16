//! Cell references, dependencies, and the order things are evaluated in.
//!
//! All of this is the HOST's, and deliberately so. It is the part that must
//! be identical whatever language a document's formulas are written in — a
//! table mixing a Python cell and a JavaScript one has to evaluate in one
//! order, and two backends each sorting their own half could not agree on
//! one. It is also the part that is easy to get subtly wrong, which is
//! exactly the work not to ask of every new backend.
//!
//! A1 notation, because it is the notation every person who has used a
//! spreadsheet already knows, and because it is unambiguous in a way
//! `row 3, column 2` is not when somebody inserts a row.

use std::collections::{BTreeMap, BTreeSet, HashMap};

/// One cell, by column and row, both zero-based internally.
///
/// Zero-based inside and one-based on the wire: `A1` is `(0, 0)`. Doing the
/// conversion once, here, is what keeps every other file free of `+ 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellRef {
    pub column: usize,
    pub row: usize,
}

impl CellRef {
    pub fn new(column: usize, row: usize) -> Self {
        Self { column, row }
    }

    /// Parse `A1`, `AB12`. Case-insensitive; `$` is accepted and ignored,
    /// because people paste absolute references out of other spreadsheets and
    /// being pedantic about it helps nobody.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let mut column = 0usize;
        let mut letters = 0usize;
        let mut chars = text.chars().peekable();
        if chars.peek() == Some(&'$') {
            chars.next();
        }
        while let Some(&c) = chars.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            chars.next();
            letters += 1;
            // Bijective base-26: A=1..Z=26, then AA=27. Not plain base-26 —
            // there is no zero digit, which is why `Z` is followed by `AA`.
            column = column * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
        }
        // Two letters, not three, and the reason is the lexical scan in
        // `references_in` rather than arithmetic. `ZZ` is 702 columns, which
        // is far more than any table anybody edits by hand — and stopping
        // there means `myA1` and `sum1` are identifiers rather than
        // references to columns `MYA` and `SUM`. Allowing a third letter
        // buys columns nobody has at the cost of shadowing variables
        // everybody writes.
        if letters == 0 || letters > 2 {
            return None;
        }
        if chars.peek() == Some(&'$') {
            chars.next();
        }
        let digits: String = chars.collect();
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let row: usize = digits.parse().ok()?;
        if row == 0 {
            return None;
        }
        Some(Self::new(column - 1, row - 1))
    }

    /// The A1 spelling, which is what a backend sees as a variable name.
    pub fn label(&self) -> String {
        let mut letters = Vec::new();
        let mut n = self.column + 1;
        while n > 0 {
            let rem = (n - 1) % 26;
            letters.push((b'A' + rem as u8) as char);
            n = (n - 1) / 26;
        }
        letters.reverse();
        format!(
            "{}{}",
            letters.into_iter().collect::<String>(),
            self.row + 1
        )
    }
}

/// A table's cells, by reference. Only the cells that have something in them.
pub type Sheet = BTreeMap<CellRef, String>;

/// One rectangular range in a formula. Its name is deliberately a legal
/// identifier: the host replaces `B2:B4` with it before either backend sees
/// the expression, while the CSV continues to show familiar A1 notation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRange {
    pub cells: Vec<CellRef>,
    pub name: String,
}

/// A formula is a cell whose text starts with `=`, exactly as in every
/// spreadsheet. Everything else is a literal value.
pub fn is_formula(text: &str) -> bool {
    text.trim_start().starts_with('=')
}

/// The expression part of a formula cell — everything after the `=`.
pub fn expression_of(text: &str) -> &str {
    text.trim_start().strip_prefix('=').unwrap_or(text).trim()
}

/// Every cell reference an expression mentions, in the order they appear.
///
/// Deliberately lexical rather than parsed. The expression is in a language
/// this crate does not know — it cannot tell a reference from a variable by
/// understanding the code, and pretending otherwise would mean writing a
/// parser per language. So the rule is a shape: a run of letters followed by
/// digits, not attached to anything else.
///
/// The cost is honest and worth stating: a Python cell using a local variable
/// literally called `A1` would have it treated as a reference. The benefit is
/// that one rule serves every language, and the rule is the one people
/// already expect from a spreadsheet.
pub fn references_in(expression: &str) -> Vec<CellRef> {
    let bytes: Vec<char> = expression.chars().collect();
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut i = 0usize;
    for range in ranges_in(expression) {
        for cell in range.cells {
            if seen.insert(cell) {
                out.push(cell);
            }
        }
    }
    while i < bytes.len() {
        // A reference cannot begin part-way through a word, or `myA1` would
        // contain one.
        let boundary = i == 0 || !(bytes[i - 1].is_alphanumeric() || bytes[i - 1] == '_');
        if !boundary || !bytes[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
            i += 1;
        }
        let digits_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        // ...and it cannot run INTO a word either: `A1b` is an identifier.
        let ends_cleanly = i >= bytes.len() || !(bytes[i].is_alphanumeric() || bytes[i] == '_');
        if digits_start > start && i > digits_start && ends_cleanly {
            let text: String = bytes[start..i].iter().collect();
            if let Some(cell) = CellRef::parse(&text)
                && seen.insert(cell)
            {
                out.push(cell);
            }
        }
    }
    out
}

/// Every `A1:B4` range in an expression, in source order.
pub fn ranges_in(expression: &str) -> Vec<CellRange> {
    let bytes = expression.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let boundary = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
        if !boundary || !bytes[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let Some((first, first_end)) = cell_at(bytes, i) else {
            i += 1;
            continue;
        };
        if first_end >= bytes.len() || bytes[first_end] != b':' {
            i = first_end;
            continue;
        }
        let Some((last, end)) = cell_at(bytes, first_end + 1) else {
            i = first_end + 1;
            continue;
        };
        if end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            i = end;
            continue;
        }
        let mut cells = Vec::new();
        for row in first.row.min(last.row)..=first.row.max(last.row) {
            for column in first.column.min(last.column)..=first.column.max(last.column) {
                cells.push(CellRef::new(column, row));
            }
        }
        out.push(CellRange {
            cells,
            name: format!("__hick_range_{}_{}", first.label(), last.label()),
        });
        i = end;
    }
    out
}

fn cell_at(bytes: &[u8], start: usize) -> Option<(CellRef, usize)> {
    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
        end += 1;
    }
    let letters_end = end;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if letters_end == start || end == letters_end {
        return None;
    }
    let text = std::str::from_utf8(&bytes[start..end]).ok()?;
    Some((CellRef::parse(text)?, end))
}

/// Why an order could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    /// The cells caught in it, sorted, so the message is stable.
    pub cells: Vec<CellRef>,
}

impl Cycle {
    /// What a person is told. Names the cells, because "circular reference"
    /// without saying where is the single most useless error a spreadsheet
    /// produces.
    pub fn message(&self) -> String {
        let names: Vec<String> = self.cells.iter().map(CellRef::label).collect();
        format!(
            "these cells depend on each other in a circle: {}",
            names.join(" → ")
        )
    }
}

/// The order formula cells must be evaluated in, or the cycle that prevents
/// one.
///
/// A depth-first topological sort over the formula cells only: a literal has
/// no dependencies and never needs ordering. References to cells outside the
/// sheet, or to literals, are fine — they resolve to a value directly.
pub fn evaluation_order(sheet: &Sheet) -> Result<Vec<CellRef>, Cycle> {
    let formulas: BTreeMap<CellRef, Vec<CellRef>> = sheet
        .iter()
        .filter(|(_, text)| is_formula(text))
        .map(|(cell, text)| (*cell, references_in(expression_of(text))))
        .collect();

    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        Open,
        Done,
    }
    let mut marks: HashMap<CellRef, Mark> = HashMap::new();
    let mut order = Vec::new();
    // An explicit stack rather than recursion: a column of ten thousand
    // formulas each referring to the one above is a legitimate sheet, and it
    // must not blow the stack.
    for &root in formulas.keys() {
        if marks.contains_key(&root) {
            continue;
        }
        let mut stack = vec![(root, 0usize)];
        while let Some((cell, index)) = stack.pop() {
            if index == 0 {
                match marks.get(&cell) {
                    Some(Mark::Done) => continue,
                    Some(Mark::Open) => {
                        // Reached a cell already on this path: the cycle is
                        // everything still open.
                        let mut cells: Vec<CellRef> = stack
                            .iter()
                            .map(|(c, _)| *c)
                            .filter(|c| marks.get(c) == Some(&Mark::Open))
                            .collect();
                        cells.push(cell);
                        cells.sort();
                        cells.dedup();
                        return Err(Cycle { cells });
                    }
                    None => {
                        marks.insert(cell, Mark::Open);
                    }
                }
            }
            let deps = formulas.get(&cell).map(Vec::as_slice).unwrap_or(&[]);
            // Only a dependency that is ITSELF a formula needs ordering.
            let next = deps
                .iter()
                .skip(index)
                .position(|d| formulas.contains_key(d));
            match next {
                Some(offset) => {
                    let at = index + offset;
                    stack.push((cell, at + 1));
                    stack.push((deps[at], 0));
                }
                None => {
                    marks.insert(cell, Mark::Done);
                    order.push(cell);
                }
            }
        }
    }
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(text: &str) -> CellRef {
        CellRef::parse(text).unwrap_or_else(|| panic!("{text} parses"))
    }

    fn sheet(entries: &[(&str, &str)]) -> Sheet {
        entries
            .iter()
            .map(|(at, text)| (cell(at), text.to_string()))
            .collect()
    }

    #[test]
    fn a1_notation_round_trips() {
        for text in ["A1", "B2", "Z9", "AA1", "AZ26", "BA100", "ZZ702"] {
            assert_eq!(cell(text).label(), text, "{text}");
        }
    }

    #[test]
    fn columns_are_bijective_base_26_not_base_26() {
        // There is no zero digit, which is why Z is followed by AA. Plain
        // base-26 would make AA the 27th column only by accident and Aa the
        // 27th too.
        assert_eq!(cell("A1").column, 0);
        assert_eq!(cell("Z1").column, 25);
        assert_eq!(cell("AA1").column, 26);
        assert_eq!(cell("AB1").column, 27);
    }

    #[test]
    fn absolute_references_pasted_from_elsewhere_are_accepted() {
        // People paste `$B$4` out of other spreadsheets; being pedantic about
        // it helps nobody.
        assert_eq!(CellRef::parse("$B$4"), CellRef::parse("B4"));
        assert_eq!(CellRef::parse("b4"), CellRef::parse("B4"));
    }

    #[test]
    fn nonsense_is_not_a_reference() {
        for text in ["", "1", "A", "A0", "1A", "AAA1", "AAAA1", "A1.5"] {
            assert!(CellRef::parse(text).is_none(), "{text} must not parse");
        }
    }

    #[test]
    fn a_formula_is_a_cell_beginning_with_equals() {
        assert!(is_formula("=A1+1"));
        assert!(is_formula("  =A1"));
        assert!(!is_formula("A1+1"));
        // `==` IS a formula; its expression happens to be `=`, which the
        // backend will reject in its own words.
        assert!(is_formula("=="));
        assert_eq!(expression_of("=="), "=");
        assert_eq!(expression_of("  = A1 + 1 "), "A1 + 1");
    }

    #[test]
    fn references_are_found_in_document_order_without_duplicates() {
        assert_eq!(references_in("A1 + B2 * A1"), vec![cell("A1"), cell("B2")]);
    }

    #[test]
    fn an_identifier_that_merely_contains_a_reference_is_not_one() {
        // `myA1` is excluded by the two-letter column cap: three letters is
        // an identifier, not a column nobody has. `A1b` and `_A1` are
        // excluded by the word boundaries.
        assert!(references_in("myA1").is_empty());
        assert!(references_in("A1b").is_empty());
        assert!(references_in("_A1").is_empty());
    }

    #[test]
    fn references_are_found_around_real_syntax() {
        assert_eq!(
            references_in("sum([A1, A2]) / (B1 - 1)"),
            vec![cell("A1"), cell("A2"), cell("B1")]
        );
    }

    #[test]
    fn a_range_depends_on_every_cell_inside_it() {
        assert_eq!(
            references_in("sum(B2:C3)"),
            vec![cell("B2"), cell("C2"), cell("B3"), cell("C3")]
        );
        assert_eq!(ranges_in("sum(B2:C3)")[0].name, "__hick_range_B2_C3");
    }

    #[test]
    fn a_sheet_of_literals_needs_no_order() {
        assert_eq!(
            evaluation_order(&sheet(&[("A1", "1"), ("B1", "2")])),
            Ok(vec![])
        );
    }

    #[test]
    fn dependencies_come_before_the_cells_that_use_them() {
        let order = evaluation_order(&sheet(&[("A1", "1"), ("B1", "=A1*2"), ("C1", "=B1+A1")]))
            .expect("no cycle");
        assert_eq!(order, vec![cell("B1"), cell("C1")]);
    }

    #[test]
    fn a_long_chain_is_ordered_without_recursing() {
        // A column of formulas each referring to the one above is a real
        // sheet, and it must not blow the stack.
        let mut entries: Vec<(String, String)> = vec![("A1".into(), "1".into())];
        for row in 2..=5_000 {
            entries.push((format!("A{row}"), format!("=A{}+1", row - 1)));
        }
        let s: Sheet = entries
            .iter()
            .map(|(at, text)| (cell(at), text.clone()))
            .collect();
        let order = evaluation_order(&s).expect("no cycle");
        assert_eq!(order.len(), 4_999);
        assert_eq!(order.first(), Some(&cell("A2")));
        assert_eq!(order.last(), Some(&cell("A5000")));
    }

    #[test]
    fn a_cycle_names_the_cells_in_it() {
        // "Circular reference" without saying where is the single most
        // useless error a spreadsheet produces.
        let err = evaluation_order(&sheet(&[("A1", "=B1"), ("B1", "=A1")])).unwrap_err();
        assert_eq!(err.cells, vec![cell("A1"), cell("B1")]);
        assert!(err.message().contains("A1"), "{}", err.message());
        assert!(err.message().contains("B1"), "{}", err.message());
    }

    #[test]
    fn a_cell_referring_to_itself_is_a_cycle() {
        let err = evaluation_order(&sheet(&[("A1", "=A1+1")])).unwrap_err();
        assert_eq!(err.cells, vec![cell("A1")]);
    }

    #[test]
    fn a_reference_to_a_literal_or_to_nothing_needs_no_ordering() {
        // Both resolve to a value directly — an empty cell is empty, which is
        // a value like any other.
        let order = evaluation_order(&sheet(&[("A1", "1"), ("B1", "=A1+Z99")])).expect("no cycle");
        assert_eq!(order, vec![cell("B1")]);
    }
}
