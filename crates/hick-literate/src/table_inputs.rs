//! Explicit code inputs use the same cell graph as interactive formulas.
use anyhow::{Result, ensure};
use hick_formula::workbook::{Workbook, document_tables};
use hickory_executor::{TranscriptEvent, Transcripts};
use std::collections::BTreeMap;

pub fn recorded_outputs(
    doc: &hick_lang::HickDocument,
    transcripts: &Transcripts,
) -> BTreeMap<usize, String> {
    let mut outputs = BTreeMap::new();
    for tag in doc.find_tags("exec") {
        if tag.get_attribute("table").is_none() {
            continue;
        }
        let container = tag.get_attribute("container").unwrap_or("default");
        if let Some(entry) = transcripts.get(container).and_then(|entries| {
            entries
                .iter()
                .rev()
                .find(|entry| entry.source_line == Some(tag.source_line))
        }) {
            if !entry
                .events
                .iter()
                .any(|event| matches!(event,TranscriptEvent::Exit { code, .. } if *code != 0))
            {
                outputs.insert(tag.source_line, entry.output.clone());
            }
        }
    }
    outputs
}
/// Runs only after cache lookup, inside an explicit permitted code run. Weave never evaluates inputs.
pub async fn program(
    doc: &hick_lang::HickDocument,
    line: usize,
    program: String,
    transcripts: &Transcripts,
    root: &std::path::Path,
) -> Result<String> {
    let tag = doc
        .find_tags("exec")
        .into_iter()
        .find(|tag| tag.source_line == line);
    let Some(selectors) = tag.and_then(|tag| tag.get_attribute("tables")) else {
        return Ok(program);
    };
    let selectors: Vec<String> = selectors
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let outputs = recorded_outputs(doc, transcripts);
    let workbook = Workbook::new(document_tables(doc, &outputs)?)?;
    let values = workbook.evaluate(root, &selectors).await?;
    let mut computed = BTreeMap::new();
    let mut raw = BTreeMap::new();
    for selector in &selectors {
        computed.insert(selector, workbook.data(selector, &values, false)?);
        raw.insert(selector, workbook.data(selector, &values, true)?);
    }
    let computed = serde_json::to_vec(&computed)?;
    let raw = serde_json::to_vec(&raw)?;
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let setup = format!(
        "\n_HICKORY_TABLES = _h_json.loads(bytes.fromhex('{}').decode('utf-8'))\n_HICKORY_RAW_TABLES = _h_json.loads(bytes.fromhex('{}').decode('utf-8'))\n# HICKORY_CELL_SOURCE\n",
        hex(&computed),
        hex(&raw)
    );
    ensure!(
        program.contains("\n# HICKORY_CELL_SOURCE\n"),
        "Table inputs require a Python notebook code block"
    );
    Ok(program.replacen("\n# HICKORY_CELL_SOURCE\n", &setup, 1))
}
