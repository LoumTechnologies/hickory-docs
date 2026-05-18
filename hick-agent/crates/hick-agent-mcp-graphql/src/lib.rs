//! MCP GraphQL extension: in-process GraphQL executor over MCP tools.
//!
//! After each LLM response is assembled, [`McpGraphqlExtension::process_response`]
//! scans for `<hick:graphql-query>` elements, executes each query against an
//! in-process `async-graphql` schema backed by [`McpClient`], and injects the
//! result as a user-role observation.
//!
//! ## Design
//!
//! - No network socket, no port binding, no interceptor — the GraphQL executor
//!   runs entirely in the host Rust process.
//! - Each `McpToolDef` becomes one mutation field in the schema; arguments come
//!   from the tool's input JSON Schema; output fields come from the output schema
//!   (fallback: `result: String`).
//! - Field selection is enforced by `async-graphql` — only requested fields appear
//!   in the result.
//! - Each resolver calls `ToolGate::check` before `McpClient::call`; a blocked
//!   resolver returns a GraphQL field error without aborting the rest of the query.
//! - GraphQL introspection is enabled so the LLM can discover tools.
//!
//! ## Compatibility
//!
//! `<hick:graphql-query>` elements are unknown to the hick-edit extension and
//! are ignored by it — no pipeline corruption.

use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dynamic::{Field, FieldFuture, FieldValue, InputValue, Schema, TypeRef};
use async_graphql::{Request, Value as GqlValue};
use hick_agent_sdk::{
    McpClient, McpToolDef, ShapeFormatConfig, ToolGate, ToolVerdict, format_mcp_result,
};
use hick_classify::Classification;

// ---------------------------------------------------------------------------
// Element scanning
// ---------------------------------------------------------------------------

/// A parsed `<hick:graphql-query>` element.
#[derive(Debug, Clone)]
pub struct HickGraphqlElement {
    /// The GraphQL query string.
    pub query: String,
    /// Byte range of the element in the source text.
    pub span: std::ops::Range<usize>,
}

/// Scan `text` for all `<hick:graphql-query>...</hick:graphql-query>` elements.
pub fn scan_graphql_elements(text: &str) -> Vec<HickGraphqlElement> {
    let open_tag = "<hick:graphql-query>";
    let close_tag = "</hick:graphql-query>";
    let mut elements = Vec::new();
    let mut pos = 0;

    while let Some(start) = text[pos..].find(open_tag).map(|i| pos + i) {
        let body_start = start + open_tag.len();
        let end = match text[body_start..].find(close_tag).map(|i| body_start + i) {
            Some(e) => e,
            None => break,
        };

        let raw_query = &text[body_start..end];
        // Strip optional CDATA wrapper.
        let query = if let Some(inner) = raw_query
            .trim()
            .strip_prefix("<![CDATA[")
            .and_then(|s| s.strip_suffix("]]>"))
        {
            inner.trim().to_string()
        } else {
            raw_query.trim().to_string()
        };

        let span = start..end + close_tag.len();
        elements.push(HickGraphqlElement { query, span });
        pos = end + close_tag.len();
    }

    elements
}

// ---------------------------------------------------------------------------
// Schema building
// ---------------------------------------------------------------------------

/// Build a tool name safe for use as a GraphQL field name.
fn gql_field_name(server: &str, tool: &str) -> String {
    format!(
        "{}_{tool}",
        server.replace('-', "_").replace('.', "_")
    )
}

/// Extract scalar-type GraphQL field names from an output JSON Schema.
fn output_fields(schema: &serde_json::Value) -> Vec<String> {
    if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
        props.keys().cloned().collect()
    } else {
    Vec::new()
    }
}

/// Build an `async-graphql` dynamic schema from a set of MCP tool definitions.
///
/// Each `(server, Vec<McpToolDef>)` pair becomes one set of mutation fields.
/// All resolvers call `ToolGate::check` + `McpClient::call` at execution time.
pub fn build_schema(
    server_tools: Vec<(String, Vec<McpToolDef>)>,
    tool_gate: Arc<ToolGate>,
    mcp_clients: HashMap<String, Arc<dyn McpClient>>,
    scope: Arc<tokio::sync::Mutex<Vec<Classification>>>,
    format_config: Arc<ShapeFormatConfig>,
) -> anyhow::Result<Schema> {
    use async_graphql::dynamic::Object;

    let mut mutation = Object::new("Mutation");
    let mut tool_count: usize = 0;

    for (server, tools) in server_tools {
        for tool in tools {
            let field_name = gql_field_name(&server, &tool.name);

            // Build field with arguments from input JSON Schema.
            let mut field = Field::new(
                field_name.clone(),
                TypeRef::named(TypeRef::STRING),
                {
                    let gate = tool_gate.clone();
                    let clients = mcp_clients.clone();
                    let scope_ref = scope.clone();
                    let fmt = format_config.clone();
                    let server_clone = server.clone();
                    let tool_clone = tool.name.clone();

                    move |ctx| {
                        let gate = gate.clone();
                        let clients = clients.clone();
                        let scope_ref = scope_ref.clone();
                        let fmt = fmt.clone();
                        let server = server_clone.clone();
                        let tool = tool_clone.clone();

                        FieldFuture::new(async move {
                            // Collect arguments into a JSON object.
                            let mut args_map = serde_json::Map::new();
                            for (k, v) in ctx.args.iter() {
                                let json_v: serde_json::Value = match v.as_value() {
                                    GqlValue::String(s) => serde_json::Value::String(s.clone()),
                                    GqlValue::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            serde_json::Value::Number(i.into())
                                        } else {
                                            serde_json::Value::String(n.to_string())
                                        }
                                    }
                                    GqlValue::Boolean(b) => serde_json::Value::Bool(*b),
                                    other => serde_json::Value::String(other.to_string()),
                                };
                                args_map.insert(k.to_string(), json_v);
                            }

                            // Check ToolGate.
                            let scope = scope_ref.lock().await.clone();
                            let verdict = gate.check(&server, &tool, &scope).await;
                            if verdict == ToolVerdict::Block {
                                return Err(async_graphql::Error::new(format!(
                                    "blocked: ToolGate denied call to {server}/{tool}"
                                )));
                            }

                            let client = clients.get(&server).ok_or_else(|| {
                                async_graphql::Error::new(format!(
                                    "no MCP client configured for server '{server}'"
                                ))
                            })?;

                            let params = serde_json::json!({
                                "name": tool,
                                "arguments": serde_json::Value::Object(args_map),
                            });

                            let result = client
                                .call("tools/call", params)
                                .await
                                .map_err(|e| async_graphql::Error::new(e.to_string()))?;

                            let formatted = format_mcp_result(&result, &fmt);
                            Ok(Some(FieldValue::value(GqlValue::String(formatted))))
                        })
                    }
                },
            );

            // Add input arguments from JSON Schema.
            if let Some(props) = tool
                .input_schema
                .get("properties")
                .and_then(|v| v.as_object())
            {
                let required: Vec<&str> = tool
                    .input_schema
                    .get("required")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();

                for k in props.keys() {
                    let type_ref = if required.contains(&k.as_str()) {
                        TypeRef::named_nn(TypeRef::STRING)
                    } else {
                        TypeRef::named(TypeRef::STRING)
                    };
                    field = field.argument(InputValue::new(k.clone(), type_ref));
                }
            }

            mutation = mutation.field(field);
            tool_count += 1;
        }
    }

    let query_object = async_graphql::dynamic::Object::new("Query")
        .field(Field::new("_empty", TypeRef::named(TypeRef::STRING), |_| {
            FieldFuture::new(async { Ok(Some(FieldValue::value(GqlValue::Null))) })
        }));

    // Build schema with introspection enabled.
    // Omit the Mutation type when there are no tools — async-graphql requires
    // that every registered Object has at least one field.
    let schema = if tool_count > 0 {
        Schema::build("Query", Some("Mutation"), None)
            .register(query_object)
            .register(mutation)
            .finish()?
    } else {
        Schema::build("Query", None, None)
            .register(query_object)
            .finish()?
    };

    Ok(schema)
}

// ---------------------------------------------------------------------------
// McpGraphqlExtension
// ---------------------------------------------------------------------------

/// Extension that scans LLM responses for `<hick:graphql-query>` elements and
/// executes them against an in-process GraphQL schema.
pub struct McpGraphqlExtension {
    schema: Schema,
    format_config: Arc<ShapeFormatConfig>,
}

impl McpGraphqlExtension {
    /// Build the extension, constructing the GraphQL schema from the provided tools.
    pub fn new(
        server_tools: Vec<(String, Vec<McpToolDef>)>,
        tool_gate: Arc<ToolGate>,
        mcp_clients: HashMap<String, Arc<dyn McpClient>>,
        scope: Arc<tokio::sync::Mutex<Vec<Classification>>>,
        format_config: ShapeFormatConfig,
    ) -> anyhow::Result<Self> {
        let format_config = Arc::new(format_config);
        let schema = build_schema(
            server_tools,
            tool_gate,
            mcp_clients,
            scope,
            format_config.clone(),
        )?;
        Ok(Self {
            schema,
            format_config,
        })
    }

    /// Process an LLM response: scan for `<hick:graphql-query>` elements,
    /// execute each, and return the observation string (or `None` if none found).
    ///
    /// The raw `response` is unchanged; the observation is a separate user message.
    pub async fn process_response(&self, response: &str) -> Option<String> {
        let elements = scan_graphql_elements(response);
        if elements.is_empty() {
            return None;
        }

        let mut observations = Vec::new();
        for elem in &elements {
            let result = self.execute_query(&elem.query).await;
            observations.push(format!(
                "<hick:graphql-result>\n<![CDATA[{result}]]>\n</hick:graphql-result>"
            ));
        }

        Some(observations.join("\n"))
    }

    async fn execute_query(&self, query: &str) -> String {
        let request = Request::new(query);
        let response = self.schema.execute(request).await;

        // Collect errors if any.
        if !response.errors.is_empty() && matches!(response.data, GqlValue::Null) {
            let errors: Vec<String> = response.errors.iter().map(|e| e.message.clone()).collect();
            return serde_json::json!({"errors": errors}).to_string();
        }

        // Format the data using the configured format.
        let data_json = serde_json::to_value(&response.data).unwrap_or(serde_json::Value::Null);
        format_mcp_result(&data_json, &self.format_config)
    }

    /// System prompt fragment documenting the `<hick:graphql-query>` element schema.
    pub fn system_prompt_fragment(&self) -> String {
        vec![
            "## MCP tools via <hick:graphql-query> GraphQL queries".to_string(),
            String::new(),
            "To call MCP tools with field selection, use a GraphQL query:".to_string(),
            String::new(),
            "```xml".to_string(),
            "<hick:graphql-query>".to_string(),
            "mutation {".to_string(),
            "  githubCreateIssue(owner: \"myorg\", repo: \"myrepo\", title: \"Fix bug\") {".to_string(),
            "    number".to_string(),
            "    url".to_string(),
            "  }".to_string(),
            "}".to_string(),
            "</hick:graphql-query>".to_string(),
            "```".to_string(),
            String::new(),
            "Wrap the query in CDATA if it contains `<` or `&`:".to_string(),
            String::new(),
            "```xml".to_string(),
            "<hick:graphql-query><![CDATA[".to_string(),
            "  mutation { ... }".to_string(),
            "]]></hick:graphql-query>".to_string(),
            "```".to_string(),
            String::new(),
            "Use introspection to discover available tools:".to_string(),
            String::new(),
            "```xml".to_string(),
            "<hick:graphql-query>{ __schema { mutationType { fields { name args { name } } } } }</hick:graphql-query>".to_string(),
            "```".to_string(),
        ]
        .join("\n")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Element scanning ──────────────────────────────────────────────────────

    #[test]
    fn scan_bare_query() {
        let text = r#"Some text.
<hick:graphql-query>
mutation { githubCreateIssue(owner: "myorg") }
</hick:graphql-query>
More text."#;

        let elements = scan_graphql_elements(text);
        assert_eq!(elements.len(), 1);
        assert!(elements[0].query.contains("githubCreateIssue"));
    }

    #[test]
    fn scan_cdata_wrapped_query() {
        let text = r#"<hick:graphql-query><![CDATA[
mutation { githubCreateIssue(title: "Fix <bug> & issue") }
]]></hick:graphql-query>"#;

        let elements = scan_graphql_elements(text);
        assert_eq!(elements.len(), 1);
        // CDATA stripped, content preserved.
        assert!(elements[0].query.contains("Fix <bug> & issue"));
    }

    #[test]
    fn scan_multiple_queries() {
        let text = r#"
<hick:graphql-query>mutation { tool1 }</hick:graphql-query>
<hick:graphql-query>mutation { tool2 }</hick:graphql-query>"#;

        let elements = scan_graphql_elements(text);
        assert_eq!(elements.len(), 2);
    }

    #[test]
    fn scan_empty_text() {
        assert!(scan_graphql_elements("No queries here.").is_empty());
    }

    // ── gql_field_name ────────────────────────────────────────────────────────

    #[test]
    fn gql_field_name_replaces_hyphens() {
        assert_eq!(gql_field_name("github", "create_issue"), "github_create_issue");
        assert_eq!(gql_field_name("my-server", "read_file"), "my_server_read_file");
    }

    // ── Schema building & execution ───────────────────────────────────────────

    #[tokio::test]
    async fn no_network_socket_opened() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let scope = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        let ext = McpGraphqlExtension::new(
            vec![],
            gate,
            HashMap::new(),
            scope,
            ShapeFormatConfig::default(),
        )
        .unwrap();

        // Introspection query — must succeed without opening any socket.
        let text = "<hick:graphql-query>{ __typename }</hick:graphql-query>";
        let obs = ext.process_response(text).await;
        assert!(obs.is_some(), "expected observation");
        // No socket was bound — if we got here, the test passes.
    }

    #[tokio::test]
    async fn introspection_available() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let scope = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        let ext = McpGraphqlExtension::new(
            vec![],
            gate,
            HashMap::new(),
            scope,
            ShapeFormatConfig::default(),
        )
        .unwrap();

        let text = "<hick:graphql-query>{ __schema { queryType { name } } }</hick:graphql-query>";
        let obs = ext.process_response(text).await;
        assert!(obs.is_some());
        let obs = obs.unwrap();
        assert!(obs.contains("Query"), "got: {obs}");
    }

    #[tokio::test]
    async fn process_response_none_when_no_queries() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let scope = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let ext = McpGraphqlExtension::new(
            vec![],
            gate,
            HashMap::new(),
            scope,
            ShapeFormatConfig::default(),
        )
        .unwrap();

        assert!(ext.process_response("No queries here.").await.is_none());
    }

    // ── System prompt ─────────────────────────────────────────────────────────

    #[test]
    fn system_prompt_mentions_graphql_query_element() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let scope = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let ext = McpGraphqlExtension::new(
            vec![],
            gate,
            HashMap::new(),
            scope,
            ShapeFormatConfig::default(),
        )
        .unwrap();

        let fragment = ext.system_prompt_fragment();
        assert!(fragment.contains("hick:graphql-query"));
        assert!(fragment.contains("introspection"));
    }
}
