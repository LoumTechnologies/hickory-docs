use super::validate_project_path;
use crate::parse;
use std::collections::BTreeMap;

/// Discover only explicitly declared Markdown inputs. The caller provides a
/// read-only revision reader; discovery never consults sibling code or config.
pub fn load_project_sources(
    entry: &str,
    read: &impl Fn(&str) -> Result<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut sources = BTreeMap::new();
    load(entry, read, &mut sources)?;
    Ok(sources)
}
fn load(
    path: &str,
    read: &impl Fn(&str) -> Result<String, String>,
    sources: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    validate_project_path(path)?;
    if !path.ends_with(".md") {
        return Err(format!("source closure requires Markdown: {path}"));
    }
    if sources.contains_key(path) {
        return Ok(());
    }
    let source = read(path)?;
    let doc = parse(&source).map_err(|e| e.to_string())?;
    sources.insert(path.into(), source);
    for tag in doc.tags() {
        let names: Vec<_> = match tag.name.as_str() {
            "project" if tag.get_attribute("entry").is_some() => {
                vec![tag.get_attribute("entry").unwrap()]
            }
            "project" => tag
                .get_attribute("sources")
                .unwrap_or("")
                .split_whitespace()
                .collect(),
            "include" => vec![tag.get_attribute("file").ok_or("include requires file")?],
            _ => vec![],
        };
        for name in names {
            validate_project_path(name)?;
            let name = path
                .rsplit_once('/')
                .map_or(name.to_owned(), |(dir, _)| format!("{dir}/{name}"));
            load(&name, read, sources)?;
        }
    }
    Ok(())
}
