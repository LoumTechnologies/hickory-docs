//! MCP XML extension: `<hick:tool>` element dispatcher.
//!
//! After each LLM response is assembled, [`McpXmlExtension::process_response`]
//! scans for `<hick:tool>` elements, dispatches each to the configured MCP
//! server, and returns the tool results as `<hick:tool-result>` observation
//! messages. No script execution is required — the tool call is declarative XML.
//!
//! ## Element schemas
//!
//! ```xml
//! <!-- hick:arg children -->
//! <hick:tool server="github" name="create_issue">
//!   <hick:arg name="owner">myorg</hick:arg>
//!   <hick:arg name="repo">myrepo</hick:arg>
//! </hick:tool>
//!
//! <!-- hick:input CDATA for complex inputs -->
//! <hick:tool server="filesystem" name="write_file">
//!   <hick:input><![CDATA[{"path": "/workspace/out.txt", "content": "hello"}]]></hick:input>
//! </hick:tool>
//! ```
//!
//! ## Auth isolation
//!
//! Auth credentials never appear in XML elements or conversation history.
//! The extension calls [`McpClient`] directly; the client resolves credentials
//! from [`McpCredentialStore`] on the host.
//!
//! ## Compatibility with hick-agent-hick-edit
//!
//! `<hick:tool>` and `<hick:tool-result>` elements are not `<hick:copy>`,
//! `<hick:paste>`, or any other pipeline-affecting element. They share the `hick:`
//! prefix only — the hick-edit extension ignores unknown element names.

use std::collections::HashMap;
use std::sync::Arc;

use hick_agent_sdk::{McpClient, McpToolDef, ShapeFormatConfig, ToolGate, ToolVerdict, format_mcp_result};
use hick_classify::Classification;

// ---------------------------------------------------------------------------
// XML scanning
// ---------------------------------------------------------------------------

/// A parsed `<hick:tool>` element.
#[derive(Debug, Clone)]
pub struct HickToolElement {
    /// Value of the `server` attribute.
    pub server: String,
    /// Value of the `name` attribute.
    pub tool: String,
    /// Parsed JSON input object.
    pub input: serde_json::Value,
    /// Byte range of the element in the source text (for stripping).
    pub span: std::ops::Range<usize>,
}

/// Scan `text` for all `<hick:tool ...>...</hick:tool>` elements.
///
/// Uses a simple string scanner rather than a full XML parser so it is
/// resilient to surrounding free text. CDATA sections in `<hick:input>` are
/// preserved correctly — their content is never entity-unescaped.
pub fn scan_tool_elements(text: &str) -> Vec<HickToolElement> {
    let mut elements = Vec::new();
    let mut pos = 0;

    while let Some(start) = text[pos..].find("<hick:tool").map(|i| pos + i) {
        // Skip if this is actually <hick:tool-result (result tag, not call tag).
        if text[start..].starts_with("<hick:tool-result") {
            pos = start + 1;
            continue;
        }

        let tag_end = match text[start..].find('>') {
            Some(i) => start + i + 1,
            None => break,
        };
        let tag_text = &text[start..tag_end];

        let server = extract_attr(tag_text, "server");
        let tool = extract_attr(tag_text, "name");

        let (server, tool) = match (server, tool) {
            (Some(s), Some(t)) => (s, t),
            _ => {
                pos = tag_end;
                continue;
            }
        };

        // Find the matching </hick:tool>.
        let close_tag = "</hick:tool>";
        let end = match text[tag_end..].find(close_tag).map(|i| tag_end + i) {
            Some(e) => e,
            None => {
                pos = tag_end;
                continue;
            }
        };
        let body = &text[tag_end..end];
        let span = start..end + close_tag.len();

        let input = parse_tool_body(body);
        elements.push(HickToolElement {
            server,
            tool,
            input,
            span,
        });

        pos = end + close_tag.len();
    }

    elements
}

/// Extract an XML attribute value from a tag string.
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')?;
    Some(tag[start..start + end].to_string())
}

/// Parse `<hick:arg>` children or `<hick:input>` CDATA into a JSON object.
fn parse_tool_body(body: &str) -> serde_json::Value {
    // Try <hick:input> CDATA first.
    if let Some(start) = body.find("<hick:input>") {
        let after_open = start + "<hick:input>".len();
        if let Some(end) = body[after_open..].find("</hick:input>").map(|i| after_open + i) {
            let content = body[after_open..end].trim();
            // Strip CDATA wrapper if present.
            let json_str = if let Some(inner) = content
                .strip_prefix("<![CDATA[")
                .and_then(|s| s.strip_suffix("]]>"))
            {
                inner
            } else {
                content
            };
            if let Ok(v) = serde_json::from_str(json_str) {
                return v;
            }
        }
    }

    // Parse <hick:arg name="key">value</hick:arg> children.
    let mut map = serde_json::Map::new();
    let mut search = body;
    while let Some(arg_start) = search.find("<hick:arg") {
        let after_tag_start = &search[arg_start..];
        let tag_close = match after_tag_start.find('>') {
            Some(i) => i,
            None => break,
        };
        let tag = &after_tag_start[..tag_close + 1];
        let name = match extract_attr(tag, "name") {
            Some(n) => n,
            None => {
                search = &search[arg_start + 1..];
                continue;
            }
        };

        let after_tag = &after_tag_start[tag_close + 1..];
        if let Some(close_pos) = after_tag.find("</hick:arg>") {
            let value_str = after_tag[..close_pos].trim();
            let value: serde_json::Value = serde_json::from_str(value_str)
                .unwrap_or_else(|_| serde_json::Value::String(value_str.to_string()));
            map.insert(name, value);
            search = &after_tag[close_pos + "</hick:arg>".len()..];
        } else {
            break;
        }
    }

    serde_json::Value::Object(map)
}

// ---------------------------------------------------------------------------
// McpXmlExtension
// ---------------------------------------------------------------------------

/// Extension that scans LLM responses for `<hick:tool>` elements and dispatches them.
pub struct McpXmlExtension {
    tool_gate: Arc<ToolGate>,
    mcp_clients: HashMap<String, Arc<dyn McpClient>>,
    format_config: ShapeFormatConfig,
}

impl McpXmlExtension {
    pub fn new(
        tool_gate: Arc<ToolGate>,
        mcp_clients: HashMap<String, Arc<dyn McpClient>>,
        format_config: ShapeFormatConfig,
    ) -> Self {
        Self {
            tool_gate,
            mcp_clients,
            format_config,
        }
    }

    /// Process an LLM response: scan for `<hick:tool>` elements, dispatch each,
    /// and return the observation string to inject (or `None` if no tools found).
    ///
    /// The raw `response` is stored in conversation history unchanged. The
    /// returned observation is a separate user-role message.
    pub async fn process_response(
        &self,
        response: &str,
        scope: &[Classification],
    ) -> Option<String> {
        let elements = scan_tool_elements(response);
        if elements.is_empty() {
            return None;
        }

        let mut results = Vec::new();
        for elem in &elements {
            let result_xml = self.dispatch_element(elem, scope).await;
            results.push(result_xml);
        }

        Some(results.join("\n"))
    }

    async fn dispatch_element(
        &self,
        elem: &HickToolElement,
        scope: &[Classification],
    ) -> String {
        let verdict = self
            .tool_gate
            .check(&elem.server, &elem.tool, scope)
            .await;

        if verdict == ToolVerdict::Block {
            return format!(
                "<hick:tool-result server=\"{}\" name=\"{}\" status=\"blocked\">\
                 <![CDATA[blocked: ToolGate denied call to {}/{}]]>\
                 </hick:tool-result>",
                elem.server, elem.tool, elem.server, elem.tool
            );
        }

        let client = match self.mcp_clients.get(&elem.server) {
            Some(c) => c.clone(),
            None => {
                return format!(
                    "<hick:tool-result server=\"{}\" name=\"{}\" status=\"error\">\
                     <![CDATA[no MCP client configured for server '{}']]>\
                     </hick:tool-result>",
                    elem.server, elem.tool, elem.server
                );
            }
        };

        let params = serde_json::json!({
            "name": elem.tool,
            "arguments": elem.input,
        });

        match client.call("tools/call", params).await {
            Ok(result) => {
                let formatted = format_mcp_result(&result, &self.format_config);
                format!(
                    "<hick:tool-result server=\"{}\" name=\"{}\" status=\"ok\">\
                     <![CDATA[{formatted}]]>\
                     </hick:tool-result>",
                    elem.server, elem.tool
                )
            }
            Err(e) => format!(
                "<hick:tool-result server=\"{}\" name=\"{}\" status=\"error\">\
                 <![CDATA[{e}]]>\
                 </hick:tool-result>",
                elem.server, elem.tool
            ),
        }
    }

    /// System prompt fragment documenting the `<hick:tool>` element schema.
    pub fn system_prompt_fragment(&self, tools_by_server: &[(String, Vec<McpToolDef>)]) -> String {
        let mut lines = vec![
            "## MCP tool calls via <hick:tool> XML elements".to_string(),
            String::new(),
            "To call an MCP tool, include a `<hick:tool>` element in your response:".to_string(),
            String::new(),
            "```xml".to_string(),
            "<hick:tool server=\"SERVER\" name=\"TOOL\">".to_string(),
            "  <hick:arg name=\"param1\">value1</hick:arg>".to_string(),
            "  <hick:arg name=\"param2\">value2</hick:arg>".to_string(),
            "</hick:tool>".to_string(),
            "```".to_string(),
            String::new(),
            "For complex JSON inputs, use CDATA:".to_string(),
            String::new(),
            "```xml".to_string(),
            "<hick:tool server=\"SERVER\" name=\"TOOL\">".to_string(),
            "  <hick:input><![CDATA[{\"key\": \"value\"}]]></hick:input>".to_string(),
            "</hick:tool>".to_string(),
            "```".to_string(),
            String::new(),
            "## Available tools".to_string(),
            String::new(),
        ];

        for (server, tools) in tools_by_server {
            for tool in tools {
                lines.push(format!("- `{server}/{}`", tool.name));
                if !tool.description.is_empty() {
                    lines.push(format!("  {}", tool.description));
                }
                // List argument names.
                if let Some(props) = tool.input_schema.get("properties").and_then(|v| v.as_object()) {
                    let args: Vec<&str> = props.keys().map(|k| k.as_str()).collect();
                    if !args.is_empty() {
                        lines.push(format!("  args: {}", args.join(", ")));
                    }
                }
            }
        }

        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── XML scanning ──────────────────────────────────────────────────────────

    #[test]
    fn scan_hick_arg_children() {
        let text = r#"Some text before.
<hick:tool server="github" name="create_issue">
  <hick:arg name="owner">myorg</hick:arg>
  <hick:arg name="repo">myrepo</hick:arg>
  <hick:arg name="title">Fix bug</hick:arg>
</hick:tool>
Some text after."#;

        let elements = scan_tool_elements(text);
        assert_eq!(elements.len(), 1);
        let elem = &elements[0];
        assert_eq!(elem.server, "github");
        assert_eq!(elem.tool, "create_issue");
        assert_eq!(elem.input["owner"], "myorg");
        assert_eq!(elem.input["repo"], "myrepo");
        assert_eq!(elem.input["title"], "Fix bug");
    }

    #[test]
    fn scan_hick_input_cdata() {
        let text = r#"<hick:tool server="filesystem" name="write_file">
  <hick:input><![CDATA[{"path": "/workspace/out.txt", "content": "hello"}]]></hick:input>
</hick:tool>"#;

        let elements = scan_tool_elements(text);
        assert_eq!(elements.len(), 1);
        let elem = &elements[0];
        assert_eq!(elem.server, "filesystem");
        assert_eq!(elem.input["path"], "/workspace/out.txt");
        assert_eq!(elem.input["content"], "hello");
    }

    #[test]
    fn scan_skips_tool_result_elements() {
        let text = r#"<hick:tool-result server="github" name="create_issue" status="ok">
  <![CDATA[{"number": 42}]]>
</hick:tool-result>
<hick:tool server="github" name="list_repos">
  <hick:arg name="org">myorg</hick:arg>
</hick:tool>"#;

        let elements = scan_tool_elements(text);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].tool, "list_repos");
    }

    #[test]
    fn scan_multiple_elements() {
        let text = r#"
<hick:tool server="github" name="create_issue">
  <hick:arg name="title">Issue 1</hick:arg>
</hick:tool>
<hick:tool server="github" name="create_issue">
  <hick:arg name="title">Issue 2</hick:arg>
</hick:tool>"#;

        let elements = scan_tool_elements(text);
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].input["title"], "Issue 1");
        assert_eq!(elements[1].input["title"], "Issue 2");
    }

    #[test]
    fn scan_empty_when_no_elements() {
        assert!(scan_tool_elements("No tools here.").is_empty());
        assert!(scan_tool_elements("").is_empty());
    }

    // ── ToolGate integration ──────────────────────────────────────────────────

    #[tokio::test]
    async fn process_response_blocked_on_secret_classification() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig {
            rejection_classifications: vec![Classification::new(vec![
                json!({"sensitivity": "secret"}),
            ])],
        };
        let gate = Arc::new(ToolGate::new(config, HashMap::new(), ToolTrust::Trusted, tx));
        let ext = McpXmlExtension::new(gate, HashMap::new(), ShapeFormatConfig::default());

        let response = r#"<hick:tool server="github" name="create_issue">
  <hick:arg name="title">test</hick:arg>
</hick:tool>"#;

        let secret_scope = vec![Classification::new(vec![json!({"sensitivity": "secret"})])];
        let obs = ext.process_response(response, &secret_scope).await;
        assert!(obs.is_some());
        let obs = obs.unwrap();
        assert!(obs.contains("status=\"blocked\""), "got: {obs}");
    }

    #[tokio::test]
    async fn process_response_returns_none_when_no_tool_elements() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let ext = McpXmlExtension::new(gate, HashMap::new(), ShapeFormatConfig::default());
        let obs = ext.process_response("No tools here.", &[]).await;
        assert!(obs.is_none());
    }

    // ── System prompt ─────────────────────────────────────────────────────────

    #[test]
    fn system_prompt_lists_tools() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let ext = McpXmlExtension::new(gate, HashMap::new(), ShapeFormatConfig::default());

        let tools = vec![(
            "github".to_string(),
            vec![McpToolDef {
                name: "create_issue".to_string(),
                description: "Create a GitHub issue".to_string(),
                input_schema: json!({"properties": {"title": {}, "body": {}}}),
                output_schema: None,
            }],
        )];

        let fragment = ext.system_prompt_fragment(&tools);
        assert!(fragment.contains("github/create_issue"));
        assert!(fragment.contains("hick:tool"));
    }

    // ── TASK-34.5: ToolGate verification for McpXmlExtension ─────────────────

    /// Verify: a Block verdict from ToolGate prevents McpClient::call from
    /// being reached in the McpXmlExtension (direct McpClient::call) mode.
    #[tokio::test]
    async fn xml_extension_block_verdict_prevents_mcp_call() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig {
            rejection_classifications: vec![hick_classify::Classification::new(vec![
                json!({"sensitivity": "secret"}),
            ])],
        };
        let gate = Arc::new(hick_agent_sdk::ToolGate::new(config, HashMap::new(), ToolTrust::Trusted, tx));

        // Spy McpClient: records whether call() was reached.
        struct SpyClient {
            called: std::sync::Arc<std::sync::Mutex<bool>>,
        }
        #[async_trait::async_trait]
        impl McpClient for SpyClient {
            async fn call(&self, _: &str, _: serde_json::Value) -> anyhow::Result<serde_json::Value> {
                *self.called.lock().unwrap() = true;
                Ok(serde_json::Value::Null)
            }
            async fn list_tools(&self) -> anyhow::Result<Vec<McpToolDef>> { Ok(vec![]) }
            async fn refresh_tools(&self) -> anyhow::Result<Vec<McpToolDef>> { Ok(vec![]) }
        }

        let call_reached = std::sync::Arc::new(std::sync::Mutex::new(false));
        let spy: Arc<dyn McpClient> = Arc::new(SpyClient { called: call_reached.clone() });

        let mut clients = HashMap::new();
        clients.insert("github".to_string(), spy);

        let ext = McpXmlExtension::new(gate, clients, ShapeFormatConfig::default());

        let secret_scope = vec![hick_classify::Classification::new(vec![
            json!({"sensitivity": "secret"}),
        ])];

        let response = r#"<hick:tool server="github" name="create_issue">
  <hick:arg name="title">Test</hick:arg>
</hick:tool>"#;

        let obs = ext.process_response(response, &secret_scope).await;
        // ToolGate should block — tool-result should show "blocked".
        assert!(obs.is_some());
        let obs_str = obs.unwrap();
        assert!(obs_str.contains("blocked"), "result must be blocked: {obs_str}");
        assert!(
            !*call_reached.lock().unwrap(),
            "McpClient::call must NOT be reached when ToolGate blocks"
        );
    }
}
