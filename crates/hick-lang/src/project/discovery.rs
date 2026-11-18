use super::validate_project_path;
use crate::parse;
use std::collections::BTreeMap;

/// Read an exact entry address as Markdown and discover its declared inputs.
/// The caller provides a read-only revision reader. Discovered inputs must
/// carry the Markdown extension.
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
            "include" | "upstream" => {
                vec![tag.get_attribute("file").ok_or("include requires file")?]
            }
            _ => vec![],
        };
        for name in names {
            validate_project_path(name)?;
            let name = path
                .rsplit_once('/')
                .map_or(name.to_owned(), |(dir, _)| format!("{dir}/{name}"));
            if !name.ends_with(".md") {
                return Err(format!("source closure requires Markdown: {name}"));
            }
            load(&name, read, sources)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_entry_extension_does_not_extend_discovered_contributor_authority() {
        let entry = "nested/literal #entry.spec";
        let sources = load_project_sources(entry, &|path| match path {
            name if name == entry => Ok(r#"<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="body.md"/></h:doc>"#.into()),
            "nested/body.md" => Ok("Explanation".into()),
            other => panic!("Unexpected source read: {other}"),
        }).unwrap();
        assert!(sources.contains_key(entry));
        assert_eq!(sources["nested/body.md"], "Explanation");

        let refused = load_project_sources(entry, &|path| {
            assert_eq!(path, entry);
            Ok(r#"<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="body.spec"/></h:doc>"#.into())
        }).unwrap_err();
        assert_eq!(refused, "source closure requires Markdown: nested/body.spec");
    }
}
