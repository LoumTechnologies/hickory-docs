//! Language observations over canonical parser regions and captured compiler origins.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLanguageContext {
    pub language: Option<String>,
    pub scope: &'static str,
    pub languages: Vec<String>,
    pub targets: Vec<String>,
    pub reason: Option<String>,
}
fn known(hint: &str) -> Option<String> {
    let hint = hint.trim().to_ascii_lowercase();
    if let Some(value) = crate::lang_detect::known_language_ids()
        .into_iter()
        .find(|value| *value == hint)
    {
        return Some(value.into());
    }
    let hint = match hint.as_str() {
        "c++" => "cpp",
        "c#" => "cs",
        other => other,
    };
    crate::lang_detect::language_id(&format!("source.{hint}")).map(str::to_owned)
}
fn region_language(region: &hick_lang::source_context::SourceCodeRegion) -> Result<String, String> {
    let hint = region
        .language
        .as_deref()
        .map(|value| {
            known(value).ok_or_else(|| format!("Unknown declared source language: {value}"))
        })
        .transpose()?;
    let path = region
        .path
        .as_deref()
        .and_then(crate::lang_detect::language_id)
        .map(str::to_owned);
    match (hint, path) {
        (Some(a), Some(b)) if a != b => Err(format!(
            "Declared language {a} conflicts with target language {b}"
        )),
        (Some(a), _) | (_, Some(a)) => Ok(a),
        _ => Err("The embedded source has no known language".into()),
    }
}
fn reading(
    scope: &'static str,
    languages: BTreeSet<String>,
    targets: BTreeSet<String>,
    reason: Option<String>,
) -> SourceLanguageContext {
    let language = if languages.len() == 1 && reason.is_none() {
        languages.iter().next().cloned()
    } else {
        None
    };
    SourceLanguageContext {
        language,
        scope,
        languages: languages.into_iter().collect(),
        targets: targets.into_iter().collect(),
        reason,
    }
}
fn refused(scope: &'static str, reason: impl Into<String>) -> SourceLanguageContext {
    reading(scope, BTreeSet::new(), BTreeSet::new(), Some(reason.into()))
}
fn contains(from: usize, to: usize, a: usize, b: usize) -> bool {
    a <= from && to <= b
}
fn intersects(from: usize, to: usize, a: usize, b: usize) -> bool {
    if from == to {
        a <= from && from < b
    } else {
        from < b && a < to
    }
}
/// Frozen parsed source and compiler endpoint, reusable for an ordered selection batch.
pub struct PreparedSourceLanguageContext {
    path: String,
    source: String,
    project: Option<(hick_lang::ProjectTree, String)>,
    regions: Vec<hick_lang::source_context::SourceCodeRegion>,
    parsed: Option<hick_lang::HickDocument>,
}
impl PreparedSourceLanguageContext {
    pub fn new(
        path: &str,
        source: &str,
        project: Option<(&hick_lang::ProjectTree, &str)>,
    ) -> Result<Self, String> {
        if source.len() > 8 * 1024 * 1024 {
            return Err("Source language input exceeds8MiB".into());
        }
        let markdown = crate::lang_detect::language_id(path) == Some("markdown");
        let (regions, parsed) = if markdown {
            (
                hick_lang::source_context::source_code_regions(source)?,
                Some(hick_lang::parse(source).map_err(|error| error.to_string())?),
            )
        } else {
            (Vec::new(), None)
        };
        Ok(Self {
            path: path.into(),
            source: source.into(),
            project: project.map(|(tree, document)| (tree.clone(), document.into())),
            regions,
            parsed,
        })
    }
    pub fn language(&self, from: usize, to: usize) -> Result<SourceLanguageContext, String> {
        let path = self.path.as_str();
        let source = self.source.as_str();
        let project = self
            .project
            .as_ref()
            .map(|(tree, document)| (tree, document.as_str()));
        if source.len() > 8 * 1024 * 1024 {
            return Err("Source language input exceeds8MiB".into());
        }
        if from > to || source.get(from..to).is_none() {
            return Err("Source language selection is outside exact UTF-8 boundaries".into());
        }
        let outer = crate::lang_detect::language_id(path);
        if outer != Some("markdown") {
            return Ok(match outer {
                Some(language) => reading(
                    "plain",
                    BTreeSet::from([language.into()]),
                    BTreeSet::new(),
                    None,
                ),
                None => refused("unknown", "The source filename has no registered language"),
            });
        }
        let regions = &self.regions;
        let parsed = self
            .parsed
            .as_ref()
            .ok_or("Prepared Markdown structure is unavailable")?;
        let shift = if source.starts_with('\u{feff}') { 3 } else { 0 };
        for tag in parsed.all_tags() {
            for span in [tag.source_span, tag.close_span].into_iter().flatten() {
                let (a, b) = (span.start + shift, span.end + shift);
                let touches = if from == to {
                    a < from && from < b
                } else {
                    intersects(from, to, a, b)
                };
                if touches {
                    return Ok(if contains(from, to, a, b) {
                        reading(
                            "markup",
                            BTreeSet::from(["markdown".into()]),
                            BTreeSet::new(),
                            None,
                        )
                    } else {
                        refused(
                            "mixed",
                            "Selection crosses structured markup and code boundaries",
                        )
                    });
                }
            }
        }

        let matching: Vec<_> = regions
            .iter()
            .filter(|region| contains(from, to, region.from, region.to))
            .collect();
        let direct = matching
            .iter()
            .copied()
            .filter(|region| region.kind != "fragment")
            .min_by_key(|region| (region.to - region.from, region.from));
        if let Some(region) = direct {
            if region.dynamic {
                return Ok(refused(
                    "dynamic",
                    "Conditional or incomplete source cannot establish one language",
                ));
            }
            if region.kind == "file" {
                if let Some((tree, document)) = project {
                    let mut observed = project_language(tree, document, source, from, to)?;
                    observed.scope = "project-file";
                    if let (Some(hint), Some(actual)) =
                        (region.language.as_deref(), observed.language.as_deref())
                    {
                        if known(hint).as_deref() != Some(actual) {
                            observed.reason = Some(
                                "Declared language conflicts with the selected compiler occurrence"
                                    .into(),
                            );
                            observed.language = None;
                        }
                    }
                    return Ok(observed);
                }
            }
            return Ok(match region_language(region) {
                Ok(language) => reading(
                    "embedded",
                    BTreeSet::from([language]),
                    region.path.clone().into_iter().collect(),
                    None,
                ),
                Err(reason) => refused("unknown", reason),
            });
        }
        if let Some(fragment) = matching
            .iter()
            .copied()
            .filter(|region| region.kind == "fragment")
            .min_by_key(|region| region.to - region.from)
        {
            if fragment.dynamic {
                return Ok(refused("dynamic", "The fragment is conditional"));
            }
            if let Some((tree, document)) = project {
                return project_language(tree, document, source, from, to);
            }
            return local_fragment_language(source, regions, fragment, parsed);
        }
        if regions
            .iter()
            .any(|region| intersects(from, to, region.from, region.to))
        {
            return Ok(refused(
                "mixed",
                "The selection crosses code and document boundaries",
            ));
        }
        Ok(reading(
            "prose",
            BTreeSet::from(["markdown".into()]),
            BTreeSet::new(),
            None,
        ))
    }
}
/// A convenience single-selection read; batch callers retain PreparedSourceLanguageContext.
pub fn source_language(
    path: &str,
    source: &str,
    from: usize,
    to: usize,
    project: Option<(&hick_lang::ProjectTree, &str)>,
) -> Result<SourceLanguageContext, String> {
    PreparedSourceLanguageContext::new(path, source, project)?.language(from, to)
}
fn covers(mut spans: Vec<(usize, usize)>, from: usize, to: usize) -> bool {
    spans.sort_unstable();
    if from == to {
        return spans.iter().any(|(a, b)| *a <= from && from <= *b);
    }
    let mut end = from;
    for (a, b) in spans {
        if a > end {
            return false;
        }
        end = end.max(b);
        if end >= to {
            return true;
        }
    }
    false
}
fn project_language(
    tree: &hick_lang::ProjectTree,
    document: &str,
    source: &str,
    from: usize,
    to: usize,
) -> Result<SourceLanguageContext, String> {
    let mut languages = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut reason = None;
    let mut work = 0;
    for (path, entry) in tree.entries() {
        let content = std::str::from_utf8(&entry.bytes).ok();
        let mut spans = Vec::new();
        for origin in &entry.origins {
            work += 1;
            if work > 1000000 {
                return Err("Source language origin budget exceeded".into());
            }
            if origin.document != document
                || !intersects(from, to, origin.source.0, origin.source.1)
                    && !(from == to && origin.source.1 == from)
            {
                continue;
            }
            if !matches!((source.get(origin.source.0..origin.source.1), content.and_then(|text| text.get(origin.output.0..origin.output.1))), (Some(left),Some(right)) if left==right)
            {
                reason =
                    Some("Compiler occurrence is not an exact literal source projection".into())
            }
            spans.push(origin.source);
        }
        if spans.is_empty() {
            continue;
        }
        targets.insert(path.clone());
        if !covers(spans, from, to) {
            reason = Some("An occurrence maps only part of the selected source".into())
        }
        match crate::lang_detect::language_id(path) {
            Some(language) => {
                languages.insert(language.into());
            }
            None => reason = Some(format!("An occurrence has unknown target language: {path}")),
        }
    }
    if targets.is_empty() {
        reason = Some("No compiler occurrence covers this fragment selection".into())
    } else if languages.len() > 1 {
        reason = Some("The fragment contributes to multiple source languages".into())
    }
    Ok(reading("project-fragment", languages, targets, reason))
}
fn local_fragment_language(
    source: &str,
    regions: &[hick_lang::source_context::SourceCodeRegion],
    selected: &hick_lang::source_context::SourceCodeRegion,
    doc: &hick_lang::HickDocument,
) -> Result<SourceLanguageContext, String> {
    let Some(id) = selected.fragment.as_deref() else {
        return Ok(refused("fragment", "Fragment has no stable identity"));
    };
    let shift = if source.starts_with('\u{feff}') { 3 } else { 0 };
    let tags = doc.all_tags();
    if regions.len().saturating_mul(tags.len()) > 1_000_000 {
        return Err("Fragment language relationship budget exceeded".into());
    }
    let mut definitions: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, region) in regions.iter().enumerate() {
        if region.kind == "fragment" {
            if let Some(id) = &region.fragment {
                definitions.entry(id.clone()).or_default().push(index);
            }
        }
    }
    if definitions.get(id).map(Vec::len) != Some(1) {
        return Ok(refused(
            "ambiguous",
            "Fragment identity is missing or duplicated",
        ));
    }
    let selected_index = definitions[id][0];
    let mut edges: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let mut unresolved = false;
    for tag in tags.iter().copied().filter(|tag| tag.name == "paste") {
        let Some(span) = tag.source_span else {
            unresolved = true;
            continue;
        };
        let at = span.start + shift;
        let Some(parent) = regions
            .iter()
            .enumerate()
            .filter(|(_, region)| region.from <= at && at < region.to)
            .min_by_key(|(_, region)| region.to - region.from)
            .map(|(index, _)| index)
        else {
            continue;
        };
        let selector = tag.get_attribute("select").unwrap_or("");
        let targets: Vec<_> = if let Some(id) = selector.strip_prefix('#') {
            definitions.get(id).cloned().unwrap_or_default()
        } else if let Some(class) = selector.strip_prefix('.') {
            tags.iter()
                .copied()
                .filter(|copy| {
                    copy.name == "copy"
                        && copy
                            .get_attribute("class")
                            .is_some_and(|names| names.split_whitespace().any(|name| name == class))
                })
                .filter_map(|copy| copy.get_attribute("id"))
                .flat_map(|id| definitions.get(id).into_iter().flatten().copied())
                .collect()
        } else {
            unresolved = true;
            Vec::new()
        };
        if targets.is_empty() {
            unresolved = true
        }
        for target in targets {
            edges.entry(parent).or_default().push(target)
        }
    }
    if unresolved {
        return Ok(refused(
            "unknown",
            "An unresolved paste prevents complete local fragment language association; select its construction entry",
        ));
    }
    let mut pending = VecDeque::new();
    for (index, region) in regions
        .iter()
        .enumerate()
        .filter(|(_, region)| region.kind != "fragment")
    {
        let language = region_language(region).ok();
        pending.push_back((index, language, region.dynamic, region.path.clone()))
    }
    let mut seen = BTreeSet::new();
    let mut languages = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut reason = None;
    while let Some((index, language, dynamic, path)) = pending.pop_front() {
        if !seen.insert((index, language.clone(), dynamic, path.clone())) {
            continue;
        }
        if seen.len() > 100000 {
            return Err("Fragment language dependency budget exceeded".into());
        }
        if index == selected_index {
            if dynamic {
                reason = Some("Fragment usage is conditional".into())
            }
            match &language {
                Some(language) => {
                    languages.insert(language.clone());
                }
                None => reason = Some("Fragment usage has an unknown language".into()),
            }
            targets.extend(path.clone())
        }
        if let Some(next) = edges.get(&index) {
            for target in next {
                pending.push_back((
                    *target,
                    language.clone(),
                    dynamic || regions[*target].dynamic,
                    path.clone(),
                ))
            }
        }
    }
    if languages.len() > 1 {
        reason = Some("The fragment is reused by multiple source languages".into())
    }
    if languages.is_empty() && reason.is_none() {
        reason = Some(
            "No known usage establishes this fragment's language; select its construction entry"
                .into(),
        )
    }
    Ok(reading("local-fragment", languages, targets, reason))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn doc(body: &str) -> String {
        format!("\x3ch:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">{body}\x3c/h:doc>")
    }
    fn selected(path: &str, source: &str, needle: &str) -> SourceLanguageContext {
        let from = source.find(needle).unwrap();
        source_language(path, source, from, from + needle.len(), None).unwrap()
    }
    #[test]
    fn plain_prose_fences_and_static_nested_bodies_use_actual_language() {
        assert_eq!(
            selected("a.py", "old", "old").language.as_deref(),
            Some("python")
        );
        assert_eq!(
            selected("notes.md", "ordinary old prose", "old")
                .language
                .as_deref(),
            Some("markdown")
        );
        let fenced = "```py\r\nold\r\n```\r\n";
        assert_eq!(
            selected("notes.md", fenced, "old").language.as_deref(),
            Some("python")
        );
        let source = doc(
            "\x3ch:copy id=\"outer\">\x3ch:file path=\"nested.ts\">old\x3c/h:file>\x3c/h:copy>",
        );
        assert_eq!(
            selected("notes.md", &source, "old").language.as_deref(),
            Some("typescript")
        );
    }
    #[test]
    fn reused_fragments_allow_same_language_and_refuse_cross_language_or_unknown() {
        let source = doc(
            "\x3ch:copy id=\"body\" class=\"shared\">old\x3c/h:copy>\x3ch:copy id=\"nested\">\x3ch:paste select=\"#body\"/>\x3c/h:copy>\x3ch:file path=\"one.py\">\x3ch:paste select=\"#nested\"/>\x3c/h:file>\x3ch:file path=\"two.py\">\x3ch:paste select=\".shared\"/>\x3c/h:file>",
        );
        let known = selected("notes.md", &source, "old");
        assert_eq!(known.language.as_deref(), Some("python"));
        assert_eq!(known.targets, vec!["one.py", "two.py"]);
        let mixed = source.replace("two.py", "two.js");
        let mixed = selected("notes.md", &mixed, "old");
        assert!(mixed.language.is_none());
        assert_eq!(mixed.languages, vec!["javascript", "python"]);
        assert!(
            selected("notes.md", "```unknown-language\nold\n```\n", "old")
                .language
                .is_none()
        );
    }
    #[test]
    fn dynamic_conflicting_and_cross_boundary_selections_refuse_before_planning() {
        let source = doc(
            "\x3ch:when test=\"flag\">\x3ch:file path=\"maybe.py\">old\x3c/h:file>\x3c/h:when>",
        );
        assert_eq!(selected("notes.md", &source, "old").scope, "dynamic");
        let source = doc("\x3ch:file path=\"a.py\" language=\"javascript\">old\x3c/h:file>");
        assert!(
            selected("notes.md", &source, "old")
                .reason
                .unwrap()
                .contains("conflicts")
        );
        let source = "Prose\n```python\nold\n```\n";
        assert_eq!(
            selected("notes.md", source, "Prose\n```python\nold").scope,
            "mixed"
        );
        assert_eq!(
            selected("notes.md", "```python\nold", "old").scope,
            "dynamic"
        );
        assert!(source_language("notes.md", "😀", 1, 2, None).is_err());
        let tag = doc("\x3ch:file path=\"a.py\">old\x3c/h:file>");
        assert_eq!(selected("notes.md", &tag, "path").scope, "markup");
    }
    #[test]
    fn external_fragment_languages_come_from_complete_selected_compiler_endpoint() {
        let part = doc("\x3ch:copy id=\"body\">old\x3c/h:copy>");
        let entry = doc(
            "\x3ch:project sources=\"part.md\"/>\x3ch:file path=\"a.py\">\x3ch:paste select=\"#body\"/>\x3c/h:file>",
        );
        let sources = BTreeMap::from([
            ("entry.md".into(), entry.clone()),
            ("part.md".into(), part.clone()),
        ]);
        let evaluated =
            hick_lang::evaluate_project("entry.md", &sources, &hick_lang::ProjectTree::default())
                .unwrap();
        let from = part.find("old").unwrap();
        let known = source_language(
            "part.md",
            &part,
            from,
            from + 3,
            Some((&evaluated.new, "part.md")),
        )
        .unwrap();
        assert_eq!(known.language.as_deref(), Some("python"));
        assert_eq!(known.scope, "project-fragment");
        let mut sources = sources;
        sources.insert(
            "entry.md".into(),
            entry.replace(
                "\x3c/h:doc>",
                "\x3ch:file path=\"b.js\">\x3ch:paste select=\"#body\"/>\x3c/h:file>\x3c/h:doc>",
            ),
        );
        let mixed =
            hick_lang::evaluate_project("entry.md", &sources, &hick_lang::ProjectTree::default())
                .unwrap();
        assert!(
            source_language(
                "part.md",
                &part,
                from,
                from + 3,
                Some((&mixed.new, "part.md"))
            )
            .unwrap()
            .language
            .is_none()
        );
        assert!(
            source_language(
                "part.md",
                &part,
                from,
                from + 3,
                Some((&hick_lang::ProjectTree::default(), "part.md"))
            )
            .unwrap()
            .language
            .is_none()
        );
    }
}
#[cfg(test)]
mod endpoint_language_tests {
    use super::*;
    #[test]
    fn selected_compiler_occurrence_wins_over_a_construction_filename_hint() {
        let entry = "\x3ch:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\x3ch:project/>\x3ch:file path=\"a.py\">old\x3c/h:file>\x3c/h:doc>";
        let evaluated = hick_lang::evaluate_project(
            "entry.md",
            &BTreeMap::from([("entry.md".into(), entry.into())]),
            &hick_lang::ProjectTree::default(),
        )
        .unwrap();
        let frozen = PreparedSourceLanguageContext::new(
            "entry.md",
            entry,
            Some((&evaluated.new, "entry.md")),
        )
        .unwrap();
        let mut changed = evaluated.new.clone();
        changed
            .insert(
                "additional.js",
                evaluated.new.get("a.py").unwrap().as_ref().clone(),
            )
            .unwrap();
        let at = entry.find("old").unwrap();
        assert_eq!(
            frozen.language(at, at + 3).unwrap().language.as_deref(),
            Some("python")
        );
        assert!(
            source_language("entry.md", entry, at, at + 3, Some((&changed, "entry.md")))
                .unwrap()
                .language
                .is_none()
        );
        let mut selected = hick_lang::ProjectTree::default();
        selected
            .insert(
                "moved.js",
                evaluated.new.get("a.py").unwrap().as_ref().clone(),
            )
            .unwrap();
        let from = entry.find("old").unwrap();
        let observed = source_language(
            "entry.md",
            entry,
            from,
            from + 3,
            Some((&selected, "entry.md")),
        )
        .unwrap();
        assert_eq!(observed.language.as_deref(), Some("javascript"));
        assert_eq!(observed.scope, "project-file");
    }
}
