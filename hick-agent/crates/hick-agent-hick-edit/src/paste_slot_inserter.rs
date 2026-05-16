//! Inserts `<hick:paste name="…"/>` markers into `hick:file` content.

use hick_lens::paste_slots::compute_insertions;

/// Insert `<hick:paste name="…"/>` slots into source content at structural positions.
///
/// Uses Tree-sitter (via `hick-lens`) to find language-aware insertion points. The
/// resulting slots are invisible to `hick run`: a paste tag with no `select` attribute
/// renders as empty string, so the pipeline output is byte-for-byte identical.
///
/// Idempotent: if `content` already contains `<hick:paste name=`, it is returned
/// unchanged so callers can call this on existing `hick:file` content safely.
pub fn insert_paste_slots(file_path: &str, content: &str) -> String {
    if content.contains("<hick:paste name=") {
        return content.to_string();
    }

    let insertions = compute_insertions(file_path, content);
    if insertions.is_empty() {
        return content.to_string();
    }

    // Apply insertions in reverse byte order so earlier offsets stay valid.
    let mut result = content.to_string();
    for ins in insertions.iter().rev() {
        let pos = ins.byte_offset.min(result.len());
        let tag = format!("<hick:paste name=\"{}\"/>", ins.name);
        result.insert_str(pos, &tag);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_slots_into_rust_content() {
        let content = "fn main() {}\n";
        let result = insert_paste_slots("src/main.rs", content);
        assert!(result.contains("<hick:paste name="), "no paste slots inserted");
        assert!(result.contains("fn main()"), "original content lost");
    }

    #[test]
    fn rendered_output_unchanged_for_rust() {
        // Remove all hick:paste tags from result — should equal original content.
        let content = "fn foo() {}\nfn bar() {}\n";
        let result = insert_paste_slots("src/lib.rs", content);
        let stripped = strip_paste_tags(&result);
        assert_eq!(
            stripped, content,
            "rendered output differs after slot insertion"
        );
    }

    #[test]
    fn rendered_output_unchanged_for_python() {
        let content = "def greet():\n    pass\n\nclass Config:\n    pass\n";
        let result = insert_paste_slots("app.py", content);
        let stripped = strip_paste_tags(&result);
        assert_eq!(stripped, content);
    }

    #[test]
    fn rendered_output_unchanged_for_typescript() {
        let content = "function greet(): void {}\n\nclass App {\n    run() {}\n}\n";
        let result = insert_paste_slots("src/app.ts", content);
        let stripped = strip_paste_tags(&result);
        assert_eq!(stripped, content);
    }

    #[test]
    fn idempotent_when_slots_already_present() {
        let content = "fn main() {}\n<hick:paste name=\"module-footer\"/>";
        let first = insert_paste_slots("a.rs", content);
        let second = insert_paste_slots("a.rs", &first);
        assert_eq!(first, second, "second call changed content");
    }

    #[test]
    fn unknown_extension_gets_header_and_footer() {
        let content = "some plain text\n";
        let result = insert_paste_slots("README.txt", content);
        assert!(result.contains("file-header"));
        assert!(result.contains("module-footer"));
        let stripped = strip_paste_tags(&result);
        assert_eq!(stripped, content);
    }

    /// Remove all `<hick:paste name="…"/>` tags from a string, simulating how
    /// `hick run` renders content (paste tags with no `select` → empty string).
    fn strip_paste_tags(s: &str) -> String {
        let mut result = s.to_string();
        loop {
            if let Some(start) = result.find("<hick:paste name=") {
                if let Some(end) = result[start..].find("/>") {
                    result.replace_range(start..start + end + 2, "");
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        result
    }
}
