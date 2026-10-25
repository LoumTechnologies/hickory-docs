/// Use the same plugin-owned routing as plain files, LSP, debugging and codegen.
pub(in crate::serve) fn language_of(path: &str) -> String {
    hick_plugins::routing::language_id(path)
        .unwrap_or_else(|| std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or(""))
        .to_string()
}
