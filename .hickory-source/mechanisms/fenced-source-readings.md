<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Read ordinary Markdown code without creating a generated file

For maintainers adding language readings to an authored Markdown editor. A static code fence has useful language syntax even though it declares no output path. We reuse canonical parsed regions and send only their literal body to the installed provider. These inspection addresses supply read positions; they grant no edit, generation, execution or ownership authority.

## Prepare literal bodies once

Known complete static pathless fences become bounded inspection documents. Canonical parsing remains the single fence detector. Up to the opening fence's ASCII indentation is removed from each line, and every remaining byte has an exact authored origin. Unknown, dynamic and incomplete regions stay readable without pretending that a provider owns them. One shared source snapshot avoids copying a large document for every fence.

<storycode:copy id="fenced-source-preparation">
//! Literal inspection documents: source bytes and read positions, never outputs.
use hick_lang::{source_context::source_code_regions, ByteOrigin};
use std::{collections::BTreeMap, sync::Arc};
const MAX_SOURCE: usize = 8 * 1024 * 1024;
const MAX_READINGS: usize = 256;
const MAX_ORIGINS: usize = 100_000;
#[derive(Clone, Debug)]
pub(crate) struct FencedReading {
    pub ordinal: usize,
    pub language: String,
    pub content: String,
    pub origins: Vec<ByteOrigin>,
}
pub(crate) struct FencedReadings {
    pub sources: Arc<BTreeMap<String, String>>,
    pub readings: Vec<FencedReading>,
}
impl FencedReadings {
    pub fn prepare(source: &str) -> Result<Self, String> {
        if source.len() > MAX_SOURCE { return Err("Fenced source exceeds the inspection byte limit".into()); }
        let regions = source_code_regions(source)?;
        let mut readings = Vec::new();
        let mut total_origins = 0usize;
        for (ordinal, region) in regions.into_iter().enumerate() {
            if region.kind != "fence" || region.dynamic || region.path.is_some() { continue; }
            let Ok(language) = crate::source_context::region_language(&region) else { continue; };
            if readings.len() == MAX_READINGS { return Err("Too many fenced inspection documents".into()); }
            let prefix = source.get(..region.from).ok_or("Invalid canonical fence boundary")?;
            let opening = prefix.strip_suffix('\n').unwrap_or(prefix);
            let opening = opening.strip_suffix('\r').unwrap_or(opening);
            let opening = opening.rsplit('\n').next().unwrap_or("").trim_start_matches('\u{feff}');
            let indent = opening.bytes().take_while(|byte| *byte == b' ').count();
            if indent > 3 { return Err("Invalid canonical fence indentation".into()); }
            let body = source.get(region.from..region.to).ok_or("Invalid canonical fence body")?;
            let mut content = String::with_capacity(body.len());
            let mut origins = Vec::new();
            let mut offset = region.from;
            for line in body.split_inclusive('\n') {
                let skipped = line.bytes().take(indent).take_while(|byte| *byte == b' ').count();
                let literal = &line[skipped..];
                if !literal.is_empty() {
                    total_origins += 1;
                    if total_origins > MAX_ORIGINS { return Err("Fenced inspection provenance exceeds its limit".into()); }
                    let start = content.len();
                    content.push_str(literal);
                    origins.push(ByteOrigin { output:(start,content.len()),document:"source".into(),source:(offset+skipped,offset+line.len()) });
                }
                offset += line.len();
            }
            if !content.is_empty() { readings.push(FencedReading {ordinal,language,content,origins}); }
        }
        Ok(Self {sources:Arc::new(BTreeMap::from([("source".into(),source.into())])),readings})
    }
}
</storycode:copy>

## Check coordinates and the authority boundary

The same exact provenance arithmetic used by project readings translates UTF-16 columns, Unicode and CRLF. Inspection maps identify the original editor, but reject all edit ranges, including insertions. Tests also refuse fabricated provenance and bound work before any provider starts. These are focused model checks; real provider and host acceptance are recorded separately.

<storycode:copy id="fenced-source-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use crate::position_map::PositionMap;
    use tower_lsp::lsp_types::{Position,Range};
    #[test]
    fn canonical_fences_are_literal_readings_not_editable_outputs() {
        let source="Prose 😀\r\n  ```python\r\n  marker = '😀'\r\n  def answer():\r\n      return 42\r\n  answer()\r\n  ```\r\n\n```unknown-language\nignored\n```\n";
        let prepared=FencedReadings::prepare(source).unwrap();
        assert_eq!(prepared.readings.len(),1);
        let reading=&prepared.readings[0];
        assert_eq!(reading.language,"python");
        assert_eq!(reading.content,"marker = '😀'\r\ndef answer():\r\n    return 42\r\nanswer()\r\n");
        let map=PositionMap::inspection(reading,prepared.sources).unwrap();
        assert!(map.is_inspection());assert!(!map.is_project());
        assert_eq!(map.to_virtual(3,6),Some((1,4)));
        assert_eq!(map.source_document(1,4),Some((None,3,6)));
        assert_eq!(map.to_virtual(1,2),None,"opening fence is not code");
        assert_eq!(map.to_virtual(6,2),None,"closing fence is not code");
        assert_eq!(map.to_virtual(2,1),None,"removed indentation is not code");
        assert!(map.source_edit_range(Range::new(Position::new(1,4),Position::new(1,10))).is_none());
        assert!(map.source_edit_range(Range::new(Position::new(0,0),Position::new(0,0))).is_none());
        assert!(map.contributing_documents().is_empty());assert!(map.project_source("source").is_none());
    }
    #[test]
    fn dynamic_and_incomplete_fences_never_gain_a_provider_reading() {
        // The namespace declaration makes this an actual conditional tag,
        // rather than inert prose that happens to contain an XML-shaped label.
        let conditional="\x3ch:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\x3ch:when test=\"false\">\n```python\nvalue=1\n```\n\x3c/h:when>\x3c/h:doc>\n";
        assert!(FencedReadings::prepare(conditional).unwrap().readings.is_empty());
        let incomplete="```typescript\nconst incomplete=1;\n";
        assert!(FencedReadings::prepare(incomplete).unwrap().readings.is_empty());
    }
    #[test]
    fn output_and_reading_limits_refuse_before_provider_work() {
        assert!(FencedReadings::prepare(&"x".repeat(MAX_SOURCE+1)).is_err());
        assert!(FencedReadings::prepare(&"```python\nx=1\n```\n".repeat(MAX_READINGS+1)).is_err());
    }
    #[test]
    fn literal_constructor_rejects_forged_or_unowned_bytes() {
        let sources=Arc::new(BTreeMap::from([("source".into(),"x".into())]));
        let origin=ByteOrigin{output:(0,1),document:"source".into(),source:(0,1)};
        assert!(crate::project_positions::ProjectProjection::literal("y".into(),vec![origin.clone()],sources.clone(),"source").is_err());
        assert!(crate::project_positions::ProjectProjection::literal("xx".into(),vec![origin],sources,"source").is_err());
    }
}
</storycode:copy>


## Executed evidence and limits

The official generated focused group passed all four preparation/provenance tests plus the backend's exact-routing/lifetime regression (five `fenced_` cases in total). Eleven existing position-map tests also passed. The real protocol, rendered browser, mounted macOS editor and Android controller acceptance are maintained in [the host reading](../frontends/fenced-source-reading-host-acceptance.md), with exact counts and retained receipts. Genuine Python and TypeScript providers returned authored locations without code-file materialization or saved-source changes.

The initial conditional test used an undeclared XML prefix, which canonical parsing correctly treated as inert prose. The corrected namespace-declared conditional reports dynamic ownership and remains excluded; the incomplete-fence case is separate. A real provider failure also exposed whole-Markdown identity-map priority, now covered by the backend regression. No edit, generated-output or execution authority is inferred from these read-only projections.

<storycode:file path="crates/hick-lsp/src/fenced_source.rs"><storycode:paste select="#fenced-source-preparation"/><storycode:paste select="#fenced-source-tests"/></storycode:file>
</storycode:doc>
