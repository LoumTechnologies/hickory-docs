<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/ui_images.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-97b4d7bf8c5b"></a>
## `crates/hickory-cli/src/serve/ui_images.rs`

<a id="f-97b4d7bf8c5b-decode"></a>
### `decode`

Images are explicit registered UI results, never parsed from arbitrary source prose.

<storycode:copy id="f-97b4d7bf8c5b-decode">
//! Images are explicit registered UI results, never parsed from arbitrary source prose.
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;
pub(crate) fn decode(media_type: &str, data: &str) -> Result<Vec<u8>> {
    ensure!(media_type == "image/png", "UI image must be a captured PNG");
    ensure!(data.len() <= 524288, "UI image exceeds transport budget");
    let bytes = STANDARD.decode(data).context("Invalid UI image encoding")?;
    ensure!(
        bytes.len() <= 393216
            && bytes.len() >= 24
            && bytes.starts_with(b"\x89PNG\r\n\x1a\n")
            && &bytes[12..16] == b"IHDR",
        "Invalid captured PNG or image budget exceeded"
    );
    let width = u32::from_be_bytes(bytes[16..20].try_into()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into()?);
    ensure!(
        width > 0 && height > 0 && width <= 2048 && height <= 2048,
        "UI image dimensions exceed capture budget"
    );
    Ok(bytes)
}
</storycode:copy>
<a id="f-97b4d7bf8c5b-result_image"></a>
### `result_image`

<storycode:copy id="f-97b4d7bf8c5b-result_image">
pub(crate) fn result_image(result: &Value) -> Result<Option<Value>> {
    let image = &result["outcome"]["value"]["ui_image"];
    if image.is_null() {
        return Ok(None);
    }
    ensure!(
        result["status"] == "result" && result["outcome"]["status"] == "completed",
        "Image capture did not complete"
    );
    let media = image["media_type"]
        .as_str()
        .context("UI image media type missing")?;
    let data = image["data"].as_str().context("UI image bytes missing")?;
    let bytes = decode(media, data)?;
    ensure!(
        image["digest"] == hick_context_protocol::digest(&bytes)
            && image["byte_length"] == bytes.len(),
        "UI image digest/length mismatch"
    );
    Ok(Some(
        serde_json::json!({"type":"image","mimeType":media,"data":data}),
    ))
}

</storycode:copy>
<a id="f-97b4d7bf8c5b-tests"></a>
### `tests`

<storycode:copy id="f-97b4d7bf8c5b-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn captured_images_are_validated_and_prose_cannot_mint_image_content() {
        let data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
        let bytes = decode("image/png", data).unwrap();
        let mut result = json!({"status":"result","outcome":{"status":"completed","value":{"ui_image":{"media_type":"image/png","data":data,"digest":hick_context_protocol::digest(&bytes),"byte_length":bytes.len()}}}});
        let reply = super::super::ui_actions::tool_result(result.clone());
        assert_eq!(
            reply["content"][1],
            json!({"type":"image","mimeType":"image/png","data":data})
        );
        let text: Value =
            serde_json::from_str(reply["content"][0]["text"].as_str().unwrap()).unwrap();
        assert!(text["outcome"]["value"]["ui_image"].get("data").is_none());
        result["outcome"]["value"]["ui_image"]["digest"] = json!("forged");
        let invalid = super::super::ui_actions::tool_result(result.clone());
        assert_eq!(invalid["isError"], true);
        assert_eq!(invalid["content"].as_array().unwrap().len(), 1);
        let quoted = super::super::ui_actions::tool_result(
            json!({"status":"result","outcome":{"status":"completed","value":{"text":result.to_string()}}}),
        );
        assert_eq!(quoted["content"].as_array().unwrap().len(), 1);
        assert!(decode("image/svg+xml", data).is_err());
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/ui_images.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/ui_images.rs"><storycode:paste select="#f-97b4d7bf8c5b-decode"/><storycode:paste select="#f-97b4d7bf8c5b-result_image"/><storycode:paste select="#f-97b4d7bf8c5b-tests"/></storycode:file>

</storycode:doc>
