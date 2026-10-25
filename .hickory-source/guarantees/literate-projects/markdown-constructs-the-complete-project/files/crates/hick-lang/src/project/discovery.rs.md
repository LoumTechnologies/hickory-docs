<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/discovery.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Rebuild the program from its Markdown source](../../../../../contract.md)

Captured SHA-256: `a7ae3e5861f37e07020af4cdd1adc5d1e2e6bf233c7b9a2433e1591d0ab08385`.

<book:entry path="crates/hick-lang/src/project/discovery.rs">
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
            load(&name, read, sources)?;
        }
    }
    Ok(())
}
</book:entry>

</book:doc>
