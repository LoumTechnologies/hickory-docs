//! CSV, and the markdown table it weaves to.
//!
//! A `<hick:table>` is a dataset that is also prose. Its content is CSV —
//! which is what a spreadsheet exports, what a query writes, and what a
//! script reads — and the woven document shows a markdown table, which is
//! what GitHub and every editor preview already draw.
//!
//! Both halves matter and neither replaces the other. Weaving CSV as a fenced
//! block would show a reader the delimiters instead of the data; storing
//! markdown and converting the other way would make the file no longer a file
//! anything else can read.
//!
//! The parser is deliberately forgiving. A ragged row, an unterminated quote,
//! a bare quote in the middle of a field: all of these occur in real exports,
//! and a weave that failed on one would refuse to render a document because
//! one cell was odd.

/// Split CSV text into rows of fields. Never fails.
///
/// RFC 4180 with the two concessions every real file needs: a quote is only
/// special at the START of a field (so `5" pipe` is a five-inch pipe rather
/// than a parse error), and an unterminated quoted field ends with the text
/// rather than swallowing it.
pub fn parse_csv(text: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            } else {
                field.push(ch);
            }
            continue;
        }
        if ch == '"' && field.is_empty() {
            quoted = true;
        } else if ch == delimiter {
            row.push(std::mem::take(&mut field));
        } else if ch == '\r' {
            // A lone \r or the \r of a \r\n: either way the row ends here.
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            row.push(std::mem::take(&mut field));
            rows.push(std::mem::take(&mut row));
        } else if ch == '\n' {
            row.push(std::mem::take(&mut field));
            rows.push(std::mem::take(&mut row));
        } else {
            field.push(ch);
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// The delimiter an attribute names, defaulting to a comma.
///
/// `tab` is spelled out because a tab character cannot be typed into an XML
/// attribute in any way a reader would recognise.
pub fn delimiter_of(value: Option<&str>) -> char {
    match value {
        Some("tab") | Some("\\t") => '\t',
        Some(other) => other.chars().next().unwrap_or(','),
        None => ',',
    }
}

/// A markdown table.
///
/// Pipes and backslashes are escaped: markdown tables have no quoting of
/// their own, so a cell containing `|` would silently become two cells. A
/// short row is padded out to the table's width, because a markdown table
/// with a ragged row renders as a mess rather than as missing data.
pub fn to_markdown(rows: &[Vec<String>], header: bool) -> String {
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if width == 0 || rows.is_empty() {
        return String::new();
    }
    let escape = |value: &str| value.replace('\\', "\\\\").replace('|', "\\|");
    let line = |row: &Vec<String>| {
        let cells: Vec<String> = (0..width)
            .map(|i| escape(row.get(i).map(String::as_str).unwrap_or("")))
            .collect();
        format!("| {} |", cells.join(" | "))
    };
    let rule = format!("|{}", " --- |".repeat(width));

    let mut out = Vec::new();
    if header {
        out.push(line(&rows[0]));
        out.push(rule);
        for row in &rows[1..] {
            out.push(line(row));
        }
    } else {
        // Markdown has no table without a header row, and inventing column
        // names would be inventing content — so the header is left empty.
        out.push(format!("|{}", "  |".repeat(width)));
        out.push(rule);
        for row in rows {
            out.push(line(row));
        }
    }
    out.join("\n")
}

/// Whether a `header` attribute means "the first row names the columns".
/// Absent means yes: a CSV with a header row is the overwhelmingly common
/// case, and a document that has to say so every time is a document full of
/// noise.
pub fn header_of(value: Option<&str>) -> bool {
    !matches!(value, Some("false") | Some("0") | Some("no"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_plain_rows() {
        assert_eq!(
            parse_csv("a,b\n1,2\n", ','),
            vec![vec!["a", "b"], vec!["1", "2"]]
        );
    }

    #[test]
    fn reads_a_quoted_field_holding_the_delimiter() {
        assert_eq!(
            parse_csv("name,note\nAda,\"one, two\"\n", ','),
            vec![vec!["name", "note"], vec!["Ada", "one, two"]]
        );
    }

    #[test]
    fn reads_a_doubled_quote_as_one_quote() {
        assert_eq!(
            parse_csv("a\n\"she said \"\"hi\"\"\"\n", ','),
            vec![vec!["a"], vec!["she said \"hi\""]]
        );
    }

    #[test]
    fn reads_a_newline_inside_a_quoted_field() {
        assert_eq!(
            parse_csv("a,b\n\"one\ntwo\",x\n", ','),
            vec![vec!["a", "b"], vec!["one\ntwo", "x"]]
        );
    }

    #[test]
    fn reads_a_bare_quote_mid_field_literally() {
        // What a spreadsheet export produces. Refusing it would refuse a real
        // file, and a weave that fails because one cell is odd is a weave
        // nobody can rely on.
        assert_eq!(
            parse_csv("a\n5\" pipe\n", ','),
            vec![vec!["a"], vec!["5\" pipe"]]
        );
    }

    #[test]
    fn an_unterminated_quote_ends_with_the_text() {
        assert_eq!(
            parse_csv("a,\"unterminated\n", ','),
            vec![vec!["a", "unterminated\n"]]
        );
    }

    #[test]
    fn handles_crlf() {
        assert_eq!(
            parse_csv("a,b\r\n1,2\r\n", ','),
            vec![vec!["a", "b"], vec!["1", "2"]]
        );
    }

    #[test]
    fn keeps_a_ragged_row_ragged() {
        assert_eq!(
            parse_csv("a,b,c\n1\n", ','),
            vec![vec!["a", "b", "c"], vec!["1"]]
        );
    }

    #[test]
    fn tabs_are_a_delimiter_you_can_name() {
        assert_eq!(delimiter_of(Some("tab")), '\t');
        assert_eq!(delimiter_of(Some(";")), ';');
        assert_eq!(delimiter_of(None), ',');
    }

    #[test]
    fn writes_a_markdown_table_with_a_rule() {
        let rows = parse_csv("name,age\nAda,36\n", ',');
        assert_eq!(
            to_markdown(&rows, true),
            "| name | age |\n| --- | --- |\n| Ada | 36 |"
        );
    }

    #[test]
    fn escapes_a_pipe_which_markdown_cannot_quote() {
        let rows = parse_csv("a\n\"x|y\"\n", ',');
        assert!(
            to_markdown(&rows, true).contains("x\\|y"),
            "{:?}",
            to_markdown(&rows, true)
        );
    }

    #[test]
    fn pads_a_ragged_row_out_to_the_width() {
        let rows = parse_csv("a,b\n1\n", ',');
        assert_eq!(
            to_markdown(&rows, true),
            "| a | b |\n| --- | --- |\n| 1 |  |"
        );
    }

    #[test]
    fn a_headerless_table_gets_an_empty_header_rather_than_invented_names() {
        let rows = parse_csv("1,2\n", ',');
        let out = to_markdown(&rows, false);
        assert!(out.starts_with("|  |  |"), "{out}");
        assert!(out.contains("| 1 | 2 |"), "{out}");
    }

    #[test]
    fn an_empty_table_weaves_to_nothing() {
        assert_eq!(to_markdown(&[], true), "");
    }

    #[test]
    fn a_header_is_assumed_unless_denied() {
        assert!(header_of(None));
        assert!(header_of(Some("true")));
        assert!(!header_of(Some("false")));
        assert!(!header_of(Some("no")));
    }
}
