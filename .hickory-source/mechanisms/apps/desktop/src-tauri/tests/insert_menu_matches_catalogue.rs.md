<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ffb5698079c9"></a>
## `apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

Related contracts: [An Equation Renders And Shows Its Source](../../../../../guarantees/authoring/an-equation-renders-and-shows-its-source/contract.md), [Inserting An Element Writes Hick You Could Have Typed Yourself](../../../../../guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md).

[Read the test `catalogue_path` beside its requirement](../../../../../guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md#f-ffb5698079c9-catalogue_path).
<a id="f-ffb5698079c9-catalogue_ids"></a>
### `catalogue_ids`

Every `id:` of an element entry, in catalogue order.

<storycode:copy id="f-ffb5698079c9-catalogue_ids">
/// Every `id:` of an element entry, in catalogue order.
fn catalogue_ids(source: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        // Element entries are `id: "name",` at the top of each object; a
        // field's `name:` and the tag's `tag:` are spelled differently.
        if let Some(rest) = line.strip_prefix("id: \"")
            && let Some(id) = rest.split('"').next()
        {
            ids.push(id.to_string());
        }
    }
    ids
}

</storycode:copy>
<a id="f-ffb5698079c9-every_menu_item_names_a_real_element"></a>
### `every_menu_item_names_a_real_element`

<storycode:copy id="f-ffb5698079c9-every_menu_item_names_a_real_element">
#[test]
fn every_menu_item_names_a_real_element() {
    let source = std::fs::read_to_string(catalogue_path()).expect("read the catalogue");
    let known: BTreeSet<String> = catalogue_ids(&source).into_iter().collect();
    assert!(
        known.len() > 20,
        "the catalogue parse found only {} elements — the file's shape changed",
        known.len()
    );

    let mut missing = Vec::new();
    for (_, elements) in hickory_desktop_lib::insert_menu_elements() {
        for (id, _) in elements.iter() {
            if !known.contains(*id) {
                missing.push(*id);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "the Insert menu names elements the page does not have: {missing:?}"
    );
}

</storycode:copy>
<a id="f-ffb5698079c9-every_element_is_reachable_from_the_menu"></a>
### `every_element_is_reachable_from_the_menu`

<storycode:copy id="f-ffb5698079c9-every_element_is_reachable_from_the_menu">
#[test]
fn every_element_is_reachable_from_the_menu() {
    let source = std::fs::read_to_string(catalogue_path()).expect("read the catalogue");
    let listed: BTreeSet<&str> = hickory_desktop_lib::insert_menu_elements()
        .iter()
        .flat_map(|(_, elements)| elements.iter().map(|(id, _)| *id))
        .collect();

    let unreachable: Vec<String> = catalogue_ids(&source)
        .into_iter()
        .filter(|id| !listed.contains(id.as_str()))
        .collect();
    assert!(
        unreachable.is_empty(),
        "elements the page offers but the native menu does not: {unreachable:?}"
    );
}
</storycode:copy>

### Assemble `apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/tests/insert_menu_matches_catalogue.rs"><storycode:paste select="#f-ffb5698079c9-catalogue_path"/><storycode:paste select="#f-ffb5698079c9-catalogue_ids"/><storycode:paste select="#f-ffb5698079c9-every_menu_item_names_a_real_element"/><storycode:paste select="#f-ffb5698079c9-every_element_is_reachable_from_the_menu"/></storycode:file>

</storycode:doc>
