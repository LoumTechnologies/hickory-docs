<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/invocation.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-75082813903f"></a>
## `crates/hickory-agent/src/invocation.rs`

<a id="f-75082813903f-ToolInvocation"></a>
### `ToolInvocation`

Shared tool invocation and historical session XML decoding. No agent policy.
One tool invocation parsed from a `<hick:next>tool</hick:next>` response.

<storycode:copy id="f-75082813903f-ToolInvocation">
//! Shared tool invocation and historical session XML decoding. No agent policy.
/// One tool invocation parsed from a `<hick:next>tool</hick:next>` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolInvocation {
    /// Tool name (`read_doc`, `read_output`, `edit_output`, `edit_doc`,
    /// `verify`).
    pub name: String,
    /// `<hick:arg name="...">value</hick:arg>` children, in order.
    pub args: Vec<(String, String)>,
    /// The `<hick:input>` payload (multi-line, raw), if present.
    pub input: Option<String>,
    /// The verbatim `<hick:tool>...</hick:tool>` XML, for session logging.
    pub raw_xml: String,
}

</storycode:copy>
<a id="f-75082813903f-impl-ToolInvocation"></a>
### `impl-ToolInvocation`

<storycode:copy id="f-75082813903f-impl-ToolInvocation">
impl ToolInvocation {
    /// Build an invocation from values a caller already holds, rendering the
    /// `raw_xml` a parsed one would have carried.
    ///
    /// This is what lets callers such as the `hick
    /// doc` subcommands, and through them any external coding agent — drive
    /// the same tools the built-in agent uses. Rendering the XML rather than
    /// leaving it empty keeps one invariant true: every invocation, whoever
    /// made it, can be written into a `hick:session` document verbatim and
    /// parsed back. A synthetic invocation with no `raw_xml` would produce a
    /// session that says a tool ran but not what it was asked to do.
    ///
    /// No escaping is performed, and none is possible — that is the parser's
    /// no-escaping invariant. A payload containing `</hick:input>` cannot be
    /// represented, and is rejected here rather than silently truncating the
    /// session file at that point.
    pub fn synthetic(
        name: impl Into<String>,
        args: Vec<(String, String)>,
        input: Option<String>,
    ) -> Result<Self, String> {
        let name = name.into();
        let mut raw_xml = format!("<hick:tool name=\"{name}\">");
        for (k, v) in &args {
            if v.contains("</hick:arg>") {
                return Err(format!(
                    "the value of argument '{k}' contains </hick:arg>, which cannot be \
                     represented in a session document"
                ));
            }
            raw_xml.push_str(&format!("\n<hick:arg name=\"{k}\">{v}</hick:arg>"));
        }
        if let Some(payload) = &input {
            if payload.contains("</hick:input>") {
                return Err(
                    "the replacement text contains </hick:input>, which cannot be represented \
                     in a session document"
                        .to_string(),
                );
            }
            raw_xml.push_str(&format!("\n<hick:input>{payload}</hick:input>"));
        }
        raw_xml.push_str("\n</hick:tool>");
        Ok(Self {
            name,
            args,
            input,
            raw_xml,
        })
    }

    /// First value of the named argument, if present.
    pub fn arg(&self, name: &str) -> Option<&str> {
        self.args
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

</storycode:copy>
<a id="f-75082813903f-parse_tool_invocation"></a>
### `parse_tool_invocation`

<storycode:copy id="f-75082813903f-parse_tool_invocation">
pub fn parse_tool_invocation(response: &str) -> Result<ToolInvocation, String> {
    let start = response.find("<hick:tool").ok_or_else(|| {
        "declared <hick:next>tool</hick:next> but included no <hick:tool> element".to_string()
    })?;
    let close = "</hick:tool>";
    let end = response
        .rfind(close)
        .ok_or_else(|| "the <hick:tool> element is not closed with </hick:tool>".to_string())?;
    if end < start {
        return Err("malformed <hick:tool> element".into());
    }
    let raw_xml = response[start..end + close.len()].to_string();

    let wrapped = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{raw_xml}\n</hick:doc>\n"
    );
    let doc = hick_lang::parse(&wrapped)
        .map_err(|e| format!("could not parse the <hick:tool> element: {e}"))?;

    let tool = doc
        .nodes
        .iter()
        .find_map(|n| match n {
            hick_lang::HickNode::Tag(t) if t.name == "tool" => Some(t),
            _ => None,
        })
        .ok_or_else(|| "no <hick:tool> element found".to_string())?;

    let name = tool
        .get_attribute("name")
        .ok_or_else(|| "the <hick:tool> element has no name attribute".to_string())?
        .to_string();

    let mut args = Vec::new();
    let mut input_count = 0usize;
    for child in &tool.children {
        if let hick_lang::HickNode::Tag(t) = child {
            match t.name.as_str() {
                "arg" => {
                    let arg_name = t
                        .get_attribute("name")
                        .ok_or_else(|| "a <hick:arg> element has no name attribute".to_string())?
                        .to_string();
                    args.push((arg_name, t.text_content().trim().to_string()));
                }
                "input" => input_count += 1,
                other => {
                    return Err(format!(
                        "unexpected <hick:{other}> inside <hick:tool> (only hick:arg and hick:input are allowed)"
                    ));
                }
            }
        }
    }
    if input_count > 1 {
        return Err("a <hick:tool> element may carry at most one <hick:input> payload".into());
    }

    // Extract the payload RAW from the verbatim XML (not via text_content),
    // so balanced hick: tags inside it survive byte-for-byte.
    let input = if input_count == 1 {
        let open_at = raw_xml
            .find("<hick:input")
            .ok_or_else(|| "malformed <hick:input> payload".to_string())?;
        let open_end = raw_xml[open_at..]
            .find('>')
            .map(|i| open_at + i + 1)
            .ok_or_else(|| "malformed <hick:input> payload".to_string())?;
        let close_at = raw_xml
            .rfind("</hick:input>")
            .ok_or_else(|| "the <hick:input> payload is not closed".to_string())?;
        if close_at < open_end {
            return Err("malformed <hick:input> payload".into());
        }
        Some(trim_payload(&raw_xml[open_end..close_at]))
    } else {
        None
    };

    Ok(ToolInvocation {
        name,
        args,
        input,
        raw_xml,
    })
}

</storycode:copy>
<a id="f-75082813903f-trim_payload"></a>
### `trim_payload`

Strip exactly one leading and one trailing newline from an
`<hick:input>` payload (the newlines separating the payload from its
tags), preserving all interior whitespace.

<storycode:copy id="f-75082813903f-trim_payload">
/// Strip exactly one leading and one trailing newline from an
/// `<hick:input>` payload (the newlines separating the payload from its
/// tags), preserving all interior whitespace.
fn trim_payload(text: &str) -> String {
    let text = text
        .strip_prefix("\r\n")
        .or_else(|| text.strip_prefix('\n'))
        .unwrap_or(text);
    let text = text.strip_suffix('\n').unwrap_or(text);
    let text = text.strip_suffix('\r').unwrap_or(text);
    text.to_string()
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/invocation.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/invocation.rs"><storycode:paste select="#f-75082813903f-ToolInvocation"/><storycode:paste select="#f-75082813903f-impl-ToolInvocation"/><storycode:paste select="#f-75082813903f-parse_tool_invocation"/><storycode:paste select="#f-75082813903f-trim_payload"/></storycode:file>

</storycode:doc>
