use hick_lang::ProjectTree;
use std::{collections::BTreeMap, path::Path};
// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[test]
fn dap_line_breakpoints_ignore_diff_prefix_columns_and_refuse_deleted_lines() {
    let source = "<hick:project/>\n<hick:file id=\"base\" path=\"program.py\">\nprint('before')\n</hick:file>\n<hick:patch after=\"base\">\n--- a/program.py\n+++ b/program.py\n@@ -1 +1 @@\n-print('before')\n+print('after')\n</hick:patch>";
    let sources = BTreeMap::from([("p.md".into(), source.into())]);
    let state = hick_lsp::document::HickDocumentState::from_project("p.md", &sources).unwrap();
    let mapping =
        hick_dap::Mapping::for_state(Path::new("p.md"), &state, Path::new("scratch")).unwrap();
    assert_eq!(
        mapping.to_generated(9),
        Some((Path::new("scratch/program.py").to_path_buf(), 0))
    );
    assert_eq!(mapping.to_generated(8), None);
    assert!(hick_lang::evaluate_project("p.md", &sources, &ProjectTree::default()).is_ok());
}
