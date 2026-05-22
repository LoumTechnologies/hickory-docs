//! MCP server implementation with tools for lens (read/write) and exec operations.

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use hick_lens::annotate;
use hick_lens::edit::{self, EditConfidence};
use hick_lens::selector::Selector;

use crate::state::ServerState;

// ---------------------------------------------------------------------------
// Tool parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
    #[schemars(
        description = "Selector to query (e.g. 'fn main', 'struct Config', '#id', '.class')"
    )]
    pub selector: String,
    #[schemars(
        description = "Specific output file to search in (optional, searches all if omitted)"
    )]
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StructureParams {
    #[schemars(description = "Output file path to get structure for")]
    pub file: String,
    #[schemars(description = "Maximum nesting depth (optional)")]
    pub depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProvenanceParams {
    #[schemars(description = "Output file path")]
    pub file: String,
    #[schemars(description = "Start line (1-based)")]
    pub start_line: usize,
    #[schemars(description = "End line (1-based, inclusive)")]
    pub end_line: usize,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReplaceParams {
    #[schemars(description = "Selector for element to replace (e.g. 'fn greet')")]
    pub selector: String,
    #[schemars(description = "Output file to search in")]
    pub file: String,
    #[schemars(description = "New content to replace the matched element with")]
    pub new_content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunParams {
    #[schemars(description = "Source .hick files as name:content pairs")]
    pub sources: Vec<SourceFile>,
    #[schemars(description = "Pipeline parameters as key=value pairs")]
    pub params: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SourceFile {
    #[schemars(description = "File name/path")]
    pub name: String,
    #[schemars(description = "File content")]
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreviewParams {
    #[schemars(description = "Output file path to preview")]
    pub file: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExecParams {
    #[schemars(description = "Shell command to run in the container")]
    pub command: String,
    #[schemars(description = "Container image (default: server's configured default)")]
    pub image: Option<String>,
    #[schemars(description = "Container name for session persistence (default: 'workspace')")]
    pub container: Option<String>,
    #[schemars(description = "Timeout in seconds (default: 120)")]
    pub timeout: Option<u64>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn text_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text)])
}

fn err_invalid(msg: impl Into<std::borrow::Cow<'static, str>>) -> ErrorData {
    ErrorData::invalid_params(msg, None)
}

fn err_internal(msg: impl Into<std::borrow::Cow<'static, str>>) -> ErrorData {
    ErrorData::internal_error(msg, None)
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

/// The hick MCP server.
#[derive(Clone)]
pub struct HickMcpServer {
    state: ServerState,
    tool_router: ToolRouter<Self>,
}

impl Default for HickMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl HickMcpServer {
    pub fn new() -> Self {
        Self {
            state: ServerState::new(),
            tool_router: Self::tool_router(),
        }
    }

    pub fn with_state(state: ServerState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }

    // -----------------------------------------------------------------------
    // Lens tools (read)
    // -----------------------------------------------------------------------

    #[tool(
        description = "Query output files for structural elements matching a selector. Supports: 'fn name', 'struct Name', 'class Name', 'enum Name', 'trait Name', '#id', '.class', or bare 'name'."
    )]
    async fn hick_query(
        &self,
        Parameters(params): Parameters<QueryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let selector = Selector::parse(&params.selector)
            .map_err(|e| err_invalid(format!("Invalid selector: {e}")))?;

        let mappings = self.state.get_all_ast_mappings().await;
        let mut results = Vec::new();

        for (path, mapping) in &mappings {
            if let Some(ref file_filter) = params.file
                && path != file_filter
            {
                continue;
            }

            let matches = selector.query(mapping);
            for m in matches {
                let origins = mapping.origins_for_element(m.element);
                let origin_info: Vec<String> = origins.iter().map(|o| format!("{:?}", o)).collect();

                results.push(format!(
                    "=== {:?} '{}' in {} [L{}-L{}] ===\n{}\n--- Origins: {}",
                    m.element.kind,
                    m.element.name,
                    path,
                    m.element.start_line + 1,
                    m.element.end_line + 1,
                    m.output_text,
                    origin_info.join(", "),
                ));
            }
        }

        if results.is_empty() {
            Ok(text_result(format!(
                "No matches for selector '{}'",
                params.selector
            )))
        } else {
            Ok(text_result(results.join("\n\n")))
        }
    }

    #[tool(
        description = "Get the structural summary of an output file: functions, structs, classes, etc. with line ranges."
    )]
    async fn hick_structure(
        &self,
        Parameters(params): Parameters<StructureParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mapping = self
            .state
            .get_ast_mapping(&params.file)
            .await
            .ok_or_else(|| {
                err_invalid(format!(
                    "No AST mapping for '{}'. Run hick_run first.",
                    params.file
                ))
            })?;

        let summary = annotate::structure_summary(&mapping, params.depth);
        Ok(text_result(summary))
    }

    #[tool(
        description = "Get provenance information for a line range in an output file. Shows which .hick source positions produced each region."
    )]
    async fn hick_provenance(
        &self,
        Parameters(params): Parameters<ProvenanceParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let content = self
            .state
            .get_file_content(&params.file)
            .await
            .ok_or_else(|| err_invalid(format!("File '{}' not found", params.file)))?;

        let prov = self
            .state
            .get_provenance_map(&params.file)
            .await
            .unwrap_or_default();

        // Convert line numbers to byte offsets
        let mut line_starts: Vec<usize> = vec![0];
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }

        let start_line = params.start_line.saturating_sub(1);
        let end_byte = line_starts
            .get(params.end_line)
            .copied()
            .unwrap_or(content.len());
        let start_byte = line_starts.get(start_line).copied().unwrap_or(0);

        let spans = prov.spans_in_range(start_byte, end_byte);

        if spans.is_empty() {
            Ok(text_result(format!(
                "No provenance data for lines {}-{} in '{}'",
                params.start_line, params.end_line, params.file
            )))
        } else {
            let mut lines = Vec::new();
            for span in &spans {
                let text_slice = &content
                    [span.output_start.min(content.len())..span.output_end.min(content.len())];
                let preview = if text_slice.len() > 80 {
                    format!("{}...", &text_slice[..77])
                } else {
                    text_slice.to_string()
                };
                lines.push(format!(
                    "[{}-{}] {:?}\n  Text: {:?}",
                    span.output_start, span.output_end, span.origin, preview
                ));
            }
            Ok(text_result(lines.join("\n")))
        }
    }

    // -----------------------------------------------------------------------
    // Lens tools (write)
    // -----------------------------------------------------------------------

    #[tool(
        description = "Compute source edits to replace an element in the .hick source. Returns the edits that would be applied (does not modify files)."
    )]
    async fn hick_replace(
        &self,
        Parameters(params): Parameters<ReplaceParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mapping = self
            .state
            .get_ast_mapping(&params.file)
            .await
            .ok_or_else(|| err_invalid(format!("No AST mapping for '{}'", params.file)))?;

        let selector = Selector::parse(&params.selector)
            .map_err(|e| err_invalid(format!("Invalid selector: {e}")))?;

        let matches = selector.query(&mapping);
        if matches.is_empty() {
            return Ok(text_result(format!(
                "No matches for '{}' in '{}'",
                params.selector, params.file
            )));
        }

        let mut edit_results = Vec::new();
        for m in &matches {
            let edits = edit::compute_edits(m.element, &params.new_content, mapping.provenance());
            for e in &edits {
                let confidence_str = match e.confidence {
                    EditConfidence::Exact => "EXACT",
                    EditConfidence::Approximate => "APPROXIMATE",
                    EditConfidence::Unmapped => "UNMAPPED",
                };
                edit_results.push(format!(
                    "Edit [{}]: {} bytes [{}-{}]\n  Replace with: {}",
                    confidence_str,
                    e.file,
                    e.source_start,
                    e.source_end,
                    if e.new_content.len() > 100 {
                        format!("{}...", &e.new_content[..97])
                    } else {
                        e.new_content.clone()
                    }
                ));
            }
        }

        Ok(text_result(edit_results.join("\n\n")))
    }

    // -----------------------------------------------------------------------
    // Exec tools
    // -----------------------------------------------------------------------

    #[tool(
        description = "Run the hick pipeline on source files and cache results. Returns a summary of output files with provenance info."
    )]
    async fn hick_run(
        &self,
        Parameters(params): Parameters<RunParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let sources: Vec<(String, String)> = params
            .sources
            .into_iter()
            .map(|s| (s.name, s.content))
            .collect();

        self.state.load_sources(sources).await;

        let pipeline_params: Vec<(String, String)> = params
            .params
            .unwrap_or_default()
            .iter()
            .filter_map(|p| {
                let mut parts = p.splitn(2, '=');
                let key = parts.next()?.to_string();
                let value = parts.next().unwrap_or("").to_string();
                Some((key, value))
            })
            .collect();

        let run_summary = self
            .state
            .run_pipeline(&pipeline_params)
            .await
            .map_err(|e| err_internal(format!("Pipeline error: {e}")))?;

        let mut summary = Vec::new();
        summary.push(format!("{} file(s) generated:", run_summary.file_count));
        for (path, size, prov_count) in &run_summary.file_details {
            summary.push(format!(
                "  {} ({}, {} provenance spans)",
                path, size, prov_count
            ));
        }

        Ok(text_result(summary.join("\n")))
    }

    #[tool(description = "Preview the rendered content of a specific output file.")]
    async fn hick_preview(
        &self,
        Parameters(params): Parameters<PreviewParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let content = self
            .state
            .get_file_content(&params.file)
            .await
            .ok_or_else(|| err_invalid(format!("File '{}' not found", params.file)))?;

        Ok(text_result(content))
    }

    // -----------------------------------------------------------------------
    // Container exec tools
    // -----------------------------------------------------------------------

    #[tool(
        description = "Run a shell command in a sandboxed WASM Linux container. The host working directory is mounted at /workspace. Containers persist across calls, preserving installed packages and environment."
    )]
    async fn hick_exec(
        &self,
        Parameters(params): Parameters<ExecParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if !self.state.exec_enabled().await {
            return Err(err_internal(
                "Container execution not configured. Start hick-mcp with --images-dir to enable hick_exec.",
            ));
        }

        let container = params.container.as_deref().unwrap_or("workspace");
        let default_image = self.state.default_image().await;
        let image = params.image.as_deref().unwrap_or(&default_image);

        let output = self
            .state
            .exec_command(container, &params.command, image, params.timeout)
            .await
            .map_err(|e| err_internal(format!("Exec error: {e}")))?;

        Ok(text_result(output))
    }
}

#[tool_handler]
impl ServerHandler for HickMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            instructions: Some(
                "Hick MCP server: query generated output structure, map back to .hick sources, \
                 compute source edits, run pipelines, and execute commands in sandboxed WASM \
                 containers. Use hick_run to load .hick sources, hick_query/hick_structure/\
                 hick_provenance to explore output, and hick_exec to run shell commands in \
                 isolated Linux containers with the working directory mounted at /workspace."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_can_be_constructed() {
        let _server = HickMcpServer::new();
    }
}
