//! Agent configuration — unified `.hick/agent.toml` schema.
//!
//! Supports two config shapes:
//! - **Point config**: no `[search]` section → ready to run immediately.
//! - **Search space config**: has `[search]` + at least one axis → use
//!   `SearchSpaceConfig::expand_trials()` to enumerate concrete configs.
//!
//! Config resolution order:
//! 1. `--config <path>` (explicit override)
//! 2. `.hick/agent.toml` in the project directory
//! 3. `~/.hick/agent.toml` in the user's home directory
//! 4. Built-in defaults (error: model_id is required)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context as _};
use hick_agent_sdk::AgentSecurityConfig;
use serde::Deserialize;

// Known valid keys for `[intent_parsers]` and `[classifiers]`. Any other key
// produces a startup error (AC #2). These lists grow as TASK-33 lands.
const KNOWN_INTENT_PARSERS: &[&str] = &["script_first", "hick_edit"];
const KNOWN_CLASSIFIERS: &[&str] = &["phi", "secrets"];

// ---------------------------------------------------------------------------
// [model] section
// ---------------------------------------------------------------------------

/// LLM provider.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Anthropic,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "xai")]
    XAI,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "deepseek")]
    DeepSeek,
}

/// `[model]` section: which LLM and credentials to use.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelSection {
    /// Provider backend.
    #[serde(default)]
    pub provider: Provider,
    /// Model identifier (e.g. `"claude-opus-4-7-20251101"`).
    pub model_id: String,
    /// Name of the environment variable that holds the API key.
    /// Defaults to the provider's conventional key name.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Context window size hint (informational; not enforced here).
    #[serde(default)]
    pub context_window: Option<u32>,
    /// Maximum output tokens per request.
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

// ---------------------------------------------------------------------------
// [extensions] section
// ---------------------------------------------------------------------------

/// `[extensions]` section: which optional extension crates are active.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExtensionsSection {
    #[serde(default)]
    pub script_first: bool,
    #[serde(default)]
    pub hick_edit: bool,
    #[serde(default)]
    pub mcp_wasm: bool,
    #[serde(default)]
    pub mcp_sdk: bool,
    #[serde(default)]
    pub mcp_xml: bool,
    #[serde(default)]
    pub mcp_graphql: bool,
    /// When true, wraps the LLM client with [`hick_agent_telemetry::TelemetryClient`]
    /// to phone-home token usage to the billing server.
    #[serde(default)]
    pub telemetry: bool,
}

// ---------------------------------------------------------------------------
// [telemetry] section
// ---------------------------------------------------------------------------

/// `[telemetry]` section: billing server config, used when
/// `extensions.telemetry = true`.
#[derive(Debug, Clone, Deserialize)]
pub struct TelemetrySection {
    /// Base URL of the billing server.
    #[serde(default = "default_billing_url")]
    pub billing_url: String,
    /// Path to a file holding the user's billing JWT (from `hick auth`).
    /// Default: `~/.hick/billing_token`.
    #[serde(default)]
    pub token_file: Option<std::path::PathBuf>,
}

fn default_billing_url() -> String {
    "https://billing.hickory.dev".into()
}

impl Default for TelemetrySection {
    fn default() -> Self {
        Self { billing_url: default_billing_url(), token_file: None }
    }
}

// ---------------------------------------------------------------------------
// [mcp] section
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct McpResponseFormatSection {
    #[serde(default = "default_fmt")]
    pub default: String,
    #[serde(default)]
    pub by_shape: ByShapeSection,
}

impl Default for McpResponseFormatSection {
    fn default() -> Self {
        Self { default: default_fmt(), by_shape: ByShapeSection::default() }
    }
}

fn default_fmt() -> String {
    "json".into()
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ByShapeSection {
    #[serde(default)]
    pub list: Option<String>,
    #[serde(default)]
    pub nested: Option<String>,
    #[serde(default)]
    pub scalar: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct McpSection {
    #[serde(default = "default_servers_file")]
    pub servers_file: String,
    #[serde(default = "default_secrets_file")]
    pub secrets_file: String,
    #[serde(default)]
    pub response_format: McpResponseFormatSection,
}

fn default_servers_file() -> String {
    ".hick/mcp-servers.toml".into()
}
fn default_secrets_file() -> String {
    ".hick/mcp-secrets.toml".into()
}

impl Default for McpSection {
    fn default() -> Self {
        Self {
            servers_file: default_servers_file(),
            secrets_file: default_secrets_file(),
            response_format: McpResponseFormatSection::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// [machine] section
// ---------------------------------------------------------------------------

/// A GGUF model file available on this machine.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelPathEntry {
    /// Short name used in eval config references (e.g. `"phi-3-mini"`).
    pub name: String,
    /// Absolute (or config-file-relative) path to the `.gguf` file.
    pub path: PathBuf,
    /// Approximate model size in gigabytes — compared against `max_model_gb`
    /// to determine feasibility.
    pub size_gb: f64,
}

/// `[machine]` section: per-machine hardware capability declarations.
///
/// The eval coordinator reads this section to skip model/config combinations
/// that exceed the machine's capacity. A machine with no `[machine]` section
/// is treated as unconstrained.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MachineSection {
    /// Inference backends compiled in and available on this machine.
    /// Recognised values: `"Metal"`, `"CUDA"`, `"CPU"`.
    /// An empty list means the machine has no declared preference.
    #[serde(default)]
    pub backends: Vec<String>,
    /// Maximum model size (gigabytes) this machine can load.
    /// Combinations requiring a larger model are skipped by the eval
    /// coordinator. Absent means no limit is declared.
    #[serde(default)]
    pub max_model_gb: Option<f64>,
    /// GGUF model files available on this machine.
    #[serde(default)]
    pub model_paths: Vec<ModelPathEntry>,
    /// Names of eval configs this machine should attempt.
    /// An empty list means "attempt all configs".
    #[serde(default)]
    pub eval_configs: Vec<String>,
}

impl MachineSection {
    /// Returns `true` if this machine can load a model of `size_gb` gigabytes.
    ///
    /// When `max_model_gb` is absent, the machine is assumed capable of
    /// running any model size.
    // Called by the eval coordinator (TASK-50); unused until then.
    #[allow(dead_code)]
    pub fn can_run_model(&self, size_gb: f64) -> bool {
        self.max_model_gb.map_or(true, |max| size_gb <= max)
    }

    /// Validate that every declared `model_paths` entry exists on disk.
    ///
    /// Relative paths are resolved against `base_dir` (typically the
    /// directory containing the config file). Absolute paths are checked
    /// as-is.
    pub fn validate_paths(&self, base_dir: &Path) -> anyhow::Result<()> {
        for entry in &self.model_paths {
            let resolved = if entry.path.is_absolute() {
                entry.path.clone()
            } else {
                base_dir.join(&entry.path)
            };
            if !resolved.exists() {
                bail!(
                    "[machine.model_paths] entry '{}': path does not exist: {}",
                    entry.name,
                    resolved.display()
                );
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// [eval] section (TASK-48 adds judge sub-section)
// ---------------------------------------------------------------------------

/// Default judge prompt template — separates backtracking from unnecessary steps (AC #9).
const DEFAULT_JUDGE_PROMPT: &str = "\
You are evaluating an AI coding agent's response to a user query. Score it on the dimensions below.\n\
\n\
## Query\n\
{query}\n\
\n\
## Agent Response / Actions\n\
{response}\n\
\n\
## Scoring Dimensions\n\
Score each dimension from 1.0 (very poor) to 10.0 (excellent).\n\
\n\
{dimensions}\n\
\n\
### Backtracking Analysis (REQUIRED)\n\
Backtracking means the agent explicitly reversed a prior decision:\n\
- Rewrote a file it had already written\n\
- Undid an edit or reverted a change\n\
- Retried a failed approach with a different strategy\n\
\n\
This is DIFFERENT from 'unnecessary steps' (steps that could be skipped but not reversed).\n\
\n\
For each reversal identified:\n\
1. Describe what was reversed and what triggered it\n\
2. Note whether it was avoidable (bad initial plan) or unavoidable (new information arrived)\n\
\n\
## Response Format\n\
Return ONLY a JSON object in this exact format:\n\
{\n\
  \"task_completion\": <1.0-10.0>,\n\
  \"tool_efficiency\": <1.0-10.0>,\n\
  \"unnecessary_steps\": <1.0-10.0>,\n\
  \"backtracking\": <1.0-10.0>,\n\
  \"output_quality\": <1.0-10.0>,\n\
  \"backtracking_events\": [\n\
    { \"description\": \"<what was reversed and why>\", \"avoidable\": <true|false> }\n\
  ]\n\
}\n\
If no backtracking occurred, use an empty array for \"backtracking_events\".\n\
";

/// A single scoring dimension for the judge (name + weight).
#[derive(Debug, Clone, Deserialize)]
pub struct JudgeDimension {
    pub name: String,
    /// Relative weight for weighted-average scoring (all weights should sum to 1.0).
    pub weight: f64,
}

/// `[eval.judge]` section: LLM-as-judge configuration (TASK-48).
#[derive(Debug, Clone, Deserialize)]
pub struct JudgeSection {
    /// Judge model identifier (e.g. `"claude-opus-4-7-20251101"`).
    pub model: String,
    /// LLM provider for the judge. Defaults to `anthropic`.
    #[serde(default)]
    pub provider: Provider,
    /// Environment variable holding the judge API key.
    /// Defaults to the provider's conventional key name.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Judge prompt template. Use `{query}`, `{response}`, and `{dimensions}` placeholders.
    /// Defaults to the built-in template which includes backtracking analysis (AC #9).
    #[serde(default)]
    pub prompt_template: Option<String>,
    /// Scoring dimensions with weights.
    /// Defaults to task_completion / tool_efficiency / unnecessary_steps / backtracking / output_quality.
    #[serde(default)]
    pub dimensions: Vec<JudgeDimension>,
}

impl JudgeSection {
    /// Effective dimensions — user-configured or the built-in defaults (AC #9).
    pub fn effective_dimensions(&self) -> Vec<JudgeDimension> {
        if self.dimensions.is_empty() {
            vec![
                JudgeDimension { name: "task_completion".into(), weight: 0.3 },
                JudgeDimension { name: "tool_efficiency".into(), weight: 0.2 },
                JudgeDimension { name: "unnecessary_steps".into(), weight: 0.15 },
                JudgeDimension { name: "backtracking".into(), weight: 0.25 },
                JudgeDimension { name: "output_quality".into(), weight: 0.1 },
            ]
        } else {
            self.dimensions.clone()
        }
    }

    /// Effective prompt template — user-configured or the built-in default (AC #6 / #9).
    pub fn effective_prompt_template(&self) -> &str {
        self.prompt_template.as_deref().unwrap_or(DEFAULT_JUDGE_PROMPT)
    }
}

/// `[eval]` section.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EvalSection {
    /// Optional LLM-as-judge configuration (TASK-48).
    #[serde(default)]
    pub judge: Option<JudgeSection>,
    /// Confidence threshold for automatic pruning (TASK-49).
    ///
    /// When set (e.g. `0.95`), a config whose Wilson CI upper bound falls below
    /// `best_win_rate * auto_prune_threshold` is automatically pruned during
    /// idle eval (TASK-50). Absent means auto-prune is disabled.
    #[serde(default)]
    pub auto_prune_threshold: Option<f64>,
}

// ---------------------------------------------------------------------------
// [search] section
// ---------------------------------------------------------------------------

/// Strategy for expanding the search space into concrete trials.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SearchStrategy {
    /// Cartesian product of all axes.
    #[default]
    Grid,
    /// One-factor-at-a-time: baseline + one variation per axis value.
    Ofat,
}

/// A single search axis: a named extension field and the values to try.
#[derive(Debug, Clone)]
pub struct SearchAxis {
    pub name: String,
    pub values: Vec<toml::Value>,
}

/// A constraint that prunes invalid trial combinations.
///
/// Semantics: if all `if_keys` match a trial, then all `then_keys` must also
/// match; otherwise the trial is pruned.
///
/// TOML example:
/// ```toml
/// [[search.constraints]]
/// if_keys  = { hick_edit = true }
/// then_keys = { script_first = true }
/// ```
#[derive(Debug, Clone)]
pub struct SearchConstraint {
    pub if_keys: HashMap<String, toml::Value>,
    pub then_keys: HashMap<String, toml::Value>,
}

/// Parsed `[search]` section.
#[derive(Debug, Clone)]
pub struct SearchSection {
    pub strategy: SearchStrategy,
    /// Search axes (order preserved; used as baseline for OFAT).
    pub axes: Vec<SearchAxis>,
    pub constraints: Vec<SearchConstraint>,
}

// ---------------------------------------------------------------------------
// Point config
// ---------------------------------------------------------------------------

/// A concrete, ready-to-run agent configuration.
#[derive(Debug, Clone)]
pub struct PointConfig {
    pub model: ModelSection,
    pub extensions: ExtensionsSection,
    /// Inline security config (replaces the old `agent-security.toml` pointer).
    pub security: AgentSecurityConfig,
    pub mcp: McpSection,
    pub machine: MachineSection,
    pub eval: EvalSection,
    /// Extra text appended to the agent system prompt. Path relative to
    /// the project directory.
    pub system_prompt_extra: Option<String>,
    pub telemetry: TelemetrySection,
}

// ---------------------------------------------------------------------------
// Search space config
// ---------------------------------------------------------------------------

/// A search space configuration for eval / A-B testing.
///
/// Use [`SearchSpaceConfig::expand_trials`] to enumerate all concrete configs.
#[derive(Debug, Clone)]
pub struct SearchSpaceConfig {
    /// Shared base config; per-trial extensions override this.
    pub base: PointConfig,
    pub search: SearchSection,
}

impl SearchSpaceConfig {
    /// Expand the search space into all concrete trial configs, pruned by
    /// constraints.
    pub fn expand_trials(&self) -> Vec<PointConfig> {
        let combos = match self.search.strategy {
            SearchStrategy::Grid => self.grid_combinations(),
            SearchStrategy::Ofat => self.ofat_combinations(),
        };
        combos
            .into_iter()
            .filter(|c| self.satisfies_constraints(c))
            .map(|c| self.apply_combo(c))
            .collect()
    }

    fn grid_combinations(&self) -> Vec<HashMap<String, toml::Value>> {
        let mut result: Vec<HashMap<String, toml::Value>> = vec![HashMap::new()];
        for axis in &self.search.axes {
            let mut expanded = Vec::new();
            for combo in result {
                for val in &axis.values {
                    let mut c = combo.clone();
                    c.insert(axis.name.clone(), val.clone());
                    expanded.push(c);
                }
            }
            result = expanded;
        }
        result
    }

    fn ofat_combinations(&self) -> Vec<HashMap<String, toml::Value>> {
        // Baseline: first value of every axis.
        let baseline: HashMap<String, toml::Value> = self
            .search
            .axes
            .iter()
            .map(|a| (a.name.clone(), a.values[0].clone()))
            .collect();

        let mut result = vec![baseline.clone()];
        for axis in &self.search.axes {
            for val in axis.values.iter().skip(1) {
                let mut c = baseline.clone();
                c.insert(axis.name.clone(), val.clone());
                result.push(c);
            }
        }
        result
    }

    fn satisfies_constraints(&self, combo: &HashMap<String, toml::Value>) -> bool {
        for constraint in &self.search.constraints {
            let if_matches = constraint
                .if_keys
                .iter()
                .all(|(k, v)| combo.get(k).map_or(false, |cv| cv == v));
            if if_matches {
                let then_holds = constraint
                    .then_keys
                    .iter()
                    .all(|(k, v)| combo.get(k).map_or(false, |cv| cv == v));
                if !then_holds {
                    return false;
                }
            }
        }
        true
    }

    fn apply_combo(&self, combo: HashMap<String, toml::Value>) -> PointConfig {
        let mut cfg = self.base.clone();
        for (key, val) in combo {
            match key.as_str() {
                "script_first" => {
                    if let Some(b) = val.as_bool() {
                        cfg.extensions.script_first = b;
                    }
                }
                "hick_edit" => {
                    if let Some(b) = val.as_bool() {
                        cfg.extensions.hick_edit = b;
                    }
                }
                "mcp_wasm" => {
                    if let Some(b) = val.as_bool() {
                        cfg.extensions.mcp_wasm = b;
                    }
                }
                "mcp_sdk" => {
                    if let Some(b) = val.as_bool() {
                        cfg.extensions.mcp_sdk = b;
                    }
                }
                "mcp_xml" => {
                    if let Some(b) = val.as_bool() {
                        cfg.extensions.mcp_xml = b;
                    }
                }
                "mcp_graphql" => {
                    if let Some(b) = val.as_bool() {
                        cfg.extensions.mcp_graphql = b;
                    }
                }
                _ => {}
            }
        }
        cfg
    }

    /// Format a single trial as a TOML `[extensions]` fragment.
    pub fn format_trial_toml(trial: &PointConfig) -> String {
        let ext = &trial.extensions;
        format!(
            "[extensions]\n\
             script_first = {}\n\
             hick_edit = {}\n\
             mcp_wasm = {}\n\
             mcp_sdk = {}\n\
             mcp_xml = {}\n\
             mcp_graphql = {}\n\
             telemetry = {}\n",
            ext.script_first,
            ext.hick_edit,
            ext.mcp_wasm,
            ext.mcp_sdk,
            ext.mcp_xml,
            ext.mcp_graphql,
            ext.telemetry,
        )
    }
}

// ---------------------------------------------------------------------------
// Discriminated AgentConfig
// ---------------------------------------------------------------------------

/// Discriminated agent configuration.
///
/// A file without `[search]` parses as `Point`; one with `[search]` and at
/// least one axis parses as `Search`.
#[derive(Debug)]
pub enum AgentConfig {
    Point(PointConfig),
    Search(SearchSpaceConfig),
}

impl AgentConfig {
    /// Load config from the first path that exists, in resolution order:
    /// 1. `explicit_path` (from `--config`)
    /// 2. `<project_dir>/.hick/agent.toml`
    /// 3. `~/.hick/agent.toml`
    ///
    /// Returns an error if `explicit_path` is given but missing, or if
    /// neither project nor home config exists.
    pub fn load(
        explicit_path: Option<&Path>,
        project_dir: &Path,
    ) -> anyhow::Result<Self> {
        if let Some(path) = explicit_path {
            return Self::load_from_file(path);
        }

        let project_cfg = project_dir.join(".hick/agent.toml");
        if project_cfg.exists() {
            return Self::load_from_file(&project_cfg);
        }

        if let Some(home_cfg) = home_config_path() {
            if home_cfg.exists() {
                return Self::load_from_file(&home_cfg);
            }
        }

        bail!(
            "no agent config found; create .hick/agent.toml in your project \
             directory or pass --config <path>"
        )
    }

    fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading config: {}", path.display()))?;
        let cfg = Self::parse(&content)
            .with_context(|| format!("parsing config: {}", path.display()))?;

        let base_dir = path.parent().unwrap_or(Path::new("."));
        let machine = match &cfg {
            AgentConfig::Point(pc) => &pc.machine,
            AgentConfig::Search(sc) => &sc.base.machine,
        };
        machine
            .validate_paths(base_dir)
            .with_context(|| format!("validating [machine] paths in: {}", path.display()))?;

        Ok(cfg)
    }

    /// Parse config from a TOML string. Exposed for testing.
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let raw_val: toml::Value =
            toml::from_str(content).context("invalid TOML")?;
        let table = raw_val
            .as_table()
            .ok_or_else(|| anyhow::anyhow!("config must be a TOML table"))?;

        // Validate known keys in [intent_parsers] (AC #2).
        if let Some(ip) = table.get("intent_parsers") {
            if let Some(t) = ip.as_table() {
                for key in t.keys() {
                    if !KNOWN_INTENT_PARSERS.contains(&key.as_str()) {
                        bail!(
                            "unknown key in [intent_parsers]: '{}'; \
                             recognised keys: {}",
                            key,
                            KNOWN_INTENT_PARSERS.join(", ")
                        );
                    }
                }
            }
        }

        // Validate known keys in [classifiers] (AC #2).
        if let Some(cl) = table.get("classifiers") {
            if let Some(t) = cl.as_table() {
                for key in t.keys() {
                    if !KNOWN_CLASSIFIERS.contains(&key.as_str()) {
                        bail!(
                            "unknown key in [classifiers]: '{}'; \
                             recognised keys: {}",
                            key,
                            KNOWN_CLASSIFIERS.join(", ")
                        );
                    }
                }
            }
        }

        // Deserialise the fields we know how to handle (all except [search]).
        let raw: RawConfig =
            toml::from_str(content).context("config schema error")?;

        let base = PointConfig {
            model: raw.model,
            extensions: raw.extensions,
            security: raw.security,
            mcp: raw.mcp,
            machine: raw.machine,
            eval: raw.eval,
            system_prompt_extra: raw.system_prompt_extra,
            telemetry: raw.telemetry,
        };

        // Discriminate on [search] presence (AC #1).
        if let Some(search_val) = table.get("search") {
            let search = parse_search_section(search_val)
                .context("[search] section error")?;
            Ok(AgentConfig::Search(SearchSpaceConfig { base, search }))
        } else {
            Ok(AgentConfig::Point(base))
        }
    }
}

// ---------------------------------------------------------------------------
// Raw TOML deserialization target
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawConfig {
    model: ModelSection,
    #[serde(default)]
    extensions: ExtensionsSection,
    /// Inline security config — fields mirror `AgentSecurityConfig`.
    #[serde(default)]
    security: AgentSecurityConfig,
    #[serde(default)]
    mcp: McpSection,
    #[serde(default)]
    machine: MachineSection,
    #[serde(default)]
    eval: EvalSection,
    #[serde(default)]
    system_prompt_extra: Option<String>,
    #[serde(default)]
    telemetry: TelemetrySection,
    // [search] is parsed separately from the raw toml::Value.
}

// ---------------------------------------------------------------------------
// [search] section parsing
// ---------------------------------------------------------------------------

fn parse_search_section(val: &toml::Value) -> anyhow::Result<SearchSection> {
    let table = val
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("[search] must be a TOML table"))?;

    let strategy = match table.get("strategy").and_then(|v| v.as_str()) {
        Some("grid") | None => SearchStrategy::Grid,
        Some("ofat") => SearchStrategy::Ofat,
        Some(other) => bail!(
            "unknown search strategy '{}'; known: grid, ofat",
            other
        ),
    };

    let constraints = table
        .get("constraints")
        .map(|v| {
            v.as_array()
                .ok_or_else(|| anyhow::anyhow!("[search.constraints] must be an array"))
                .and_then(|arr| arr.iter().map(parse_constraint).collect())
        })
        .transpose()?
        .unwrap_or_default();

    // Every key except "strategy" and "constraints" is a search axis.
    let mut axes: Vec<SearchAxis> = Vec::new();
    for (key, value) in table {
        if key == "strategy" || key == "constraints" {
            continue;
        }
        let values = value
            .as_array()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "[search] axis '{}' must be an array of values",
                    key
                )
            })?
            .clone();
        if values.is_empty() {
            bail!("[search] axis '{}' must have at least one value", key);
        }
        axes.push(SearchAxis { name: key.clone(), values });
    }

    Ok(SearchSection { strategy, axes, constraints })
}

fn parse_constraint(val: &toml::Value) -> anyhow::Result<SearchConstraint> {
    let table = val
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("each constraint must be a TOML table"))?;

    let if_keys = table
        .get("if_keys")
        .and_then(|v| v.as_table())
        .map(|t| t.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    let then_keys = table
        .get("then_keys")
        .and_then(|v| v.as_table())
        .map(|t| t.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    Ok(SearchConstraint { if_keys, then_keys })
}

// ---------------------------------------------------------------------------
// Home config path helper
// ---------------------------------------------------------------------------

fn home_config_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".hick/agent.toml"))
}

// ---------------------------------------------------------------------------
// User keys file (~/.config/hick/keys.toml)
// ---------------------------------------------------------------------------

/// API keys loaded from `~/.config/hick/keys.toml`.
///
/// All fields are optional. A key present here is used as a fallback when the
/// corresponding environment variable (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`,
/// `XAI_API_KEY`, `OPENROUTER_API_KEY`, `DEEPSEEK_API_KEY`) is absent or empty.
///
/// Example `~/.config/hick/keys.toml`:
/// ```toml
/// anthropic_api_key  = "sk-ant-..."
/// openai_api_key     = "sk-..."
/// xai_api_key        = "xai-..."
/// openrouter_api_key = "sk-or-..."
/// deepseek_api_key   = "sk-..."
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserKeys {
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub xai_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub deepseek_api_key: Option<String>,
}

impl UserKeys {
    /// Load from `~/.config/hick/keys.toml`.
    ///
    /// Returns `Ok(None)` when the file does not exist.
    /// Returns an error when the file exists but cannot be parsed.
    pub fn load() -> anyhow::Result<Option<Self>> {
        let Some(path) = home_keys_path() else {
            return Ok(None);
        };
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let keys: UserKeys = toml::from_str(&content)
            .with_context(|| format!("parsing {}", path.display()))?;
        Ok(Some(keys))
    }

    /// Return the non-empty key for `provider`, or `None`.
    pub fn key_for(&self, provider: &Provider) -> Option<&str> {
        let raw = match provider {
            Provider::Anthropic  => self.anthropic_api_key.as_deref(),
            Provider::OpenAI     => self.openai_api_key.as_deref(),
            Provider::XAI        => self.xai_api_key.as_deref(),
            Provider::OpenRouter => self.openrouter_api_key.as_deref(),
            Provider::DeepSeek   => self.deepseek_api_key.as_deref(),
        };
        raw.filter(|k| !k.is_empty())
    }
}

fn home_keys_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".config/hick/keys.toml"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_point_toml(script_first: bool, hick_edit: bool) -> String {
        format!(
            r#"
[model]
model_id = "claude-opus-4-7-20251101"

[extensions]
script_first = {script_first}
hick_edit = {hick_edit}
"#
        )
    }

    // AC #1: no [search] → Point
    #[test]
    fn point_config_parsed_without_search_section() {
        let toml = minimal_point_toml(true, false);
        let cfg = AgentConfig::parse(&toml).unwrap();
        assert!(matches!(cfg, AgentConfig::Point(_)));
    }

    // AC #1: [search] with axes → Search
    #[test]
    fn search_config_parsed_with_search_section() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[search]
strategy = "grid"
script_first = [true, false]
hick_edit = [true, false]
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        assert!(matches!(cfg, AgentConfig::Search(_)));
    }

    // AC #2: unknown key in [intent_parsers] → error
    #[test]
    fn unknown_intent_parser_key_is_an_error() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[intent_parsers]
unknown_parser = true
"#;
        let err = AgentConfig::parse(toml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown_parser"), "error should name the key: {msg}");
        assert!(msg.contains("intent_parsers"), "error should mention the section: {msg}");
    }

    // AC #2: unknown key in [classifiers] → error
    #[test]
    fn unknown_classifier_key_is_an_error() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[classifiers]
weird_label = true
"#;
        let err = AgentConfig::parse(toml).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("weird_label"), "error should name the key: {msg}");
    }

    // AC #3: grid strategy with 2 binary axes → 4 trials
    #[test]
    fn grid_strategy_two_binary_axes_produces_four_trials() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[search]
strategy = "grid"
script_first = [true, false]
hick_edit = [true, false]
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Search(sc) = cfg else { panic!("expected Search") };
        let trials = sc.expand_trials();
        assert_eq!(trials.len(), 4);
    }

    // AC #4: ofat with n axes → 1 + sum(len - 1) trials
    #[test]
    fn ofat_strategy_produces_correct_trial_count() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[search]
strategy = "ofat"
script_first = [true, false]
hick_edit = [true, false]
mcp_xml = [true, false, true]
"#;
        // n=3 axes, lengths [2, 2, 3]
        // expected = 1 + (2-1) + (2-1) + (3-1) = 1 + 1 + 1 + 2 = 5
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Search(sc) = cfg else { panic!("expected Search") };
        let trials = sc.expand_trials();
        assert_eq!(trials.len(), 5);
    }

    // AC #5: constraint prunes invalid combo
    #[test]
    fn constraint_prunes_hick_edit_without_script_first() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[search]
strategy = "grid"
script_first = [true, false]
hick_edit = [true, false]

[[search.constraints]]
if_keys  = { hick_edit = true }
then_keys = { script_first = true }
"#;
        // 4 combos; {hick_edit=true, script_first=false} pruned → 3 trials
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Search(sc) = cfg else { panic!("expected Search") };
        let trials = sc.expand_trials();
        assert_eq!(trials.len(), 3);
        // Confirm the pruned combo is absent
        let pruned = trials.iter().any(|t| t.extensions.hick_edit && !t.extensions.script_first);
        assert!(!pruned);
    }

    // AC #8: adding a new axis is one line
    #[test]
    fn adding_new_axis_requires_one_line() {
        // Demonstrates that `new_ext = [true, false]` inside [search] is all
        // that is needed — no other file changes. This test parses such a config.
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[search]
strategy = "grid"
script_first = [true, false]
new_ext = [true, false]
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Search(sc) = cfg else { panic!("expected Search") };
        // new_ext axis is captured even though it maps to no known extension
        let axis_names: Vec<&str> = sc.search.axes.iter().map(|a| a.name.as_str()).collect();
        assert!(axis_names.contains(&"new_ext"), "new_ext axis should be captured");
        // 2×2 = 4 trials (axis values still expand even if the extension is unknown)
        assert_eq!(sc.expand_trials().len(), 4);
    }

    // Verify security defaults are applied when [security] is absent.
    #[test]
    fn security_defaults_when_section_absent() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Point(pc) = cfg else { panic!("expected Point") };
        assert!(pc.security.deny_all_network);
    }

    // Verify [security] section overrides default.
    #[test]
    fn security_section_overrides_default() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[security]
deny_all_network = false
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Point(pc) = cfg else { panic!("expected Point") };
        assert!(!pc.security.deny_all_network);
    }

    // ---------------------------------------------------------------------------
    // [machine] section tests
    // ---------------------------------------------------------------------------

    // AC #1: [machine] parses all declared fields correctly.
    #[test]
    fn machine_section_fields_parse() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[machine]
backends     = ["Metal", "CPU"]
max_model_gb = 8.0
eval_configs = ["standard", "hick-edit"]

[[machine.model_paths]]
name    = "phi-3-mini"
path    = "/tmp/phi-3-mini.gguf"
size_gb = 2.3
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Point(pc) = cfg else { panic!("expected Point") };
        let m = &pc.machine;
        assert_eq!(m.backends, vec!["Metal", "CPU"]);
        assert_eq!(m.max_model_gb, Some(8.0));
        assert_eq!(m.eval_configs, vec!["standard", "hick-edit"]);
        assert_eq!(m.model_paths.len(), 1);
        assert_eq!(m.model_paths[0].name, "phi-3-mini");
        assert_eq!(m.model_paths[0].size_gb, 2.3);
    }

    // AC #1: empty [machine] section uses safe defaults (backwards compat).
    #[test]
    fn machine_section_empty_is_ok() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[machine]
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Point(pc) = cfg else { panic!("expected Point") };
        assert!(pc.machine.backends.is_empty());
        assert!(pc.machine.max_model_gb.is_none());
        assert!(pc.machine.model_paths.is_empty());
        assert!(pc.machine.eval_configs.is_empty());
    }

    // AC #1 / #2: absent [machine] section defaults correctly.
    #[test]
    fn machine_section_absent_defaults_ok() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Point(pc) = cfg else { panic!("expected Point") };
        assert!(pc.machine.can_run_model(999.0), "no limit declared → always feasible");
    }

    // AC #2: feasibility predicate — model fits.
    #[test]
    fn can_run_model_true_when_within_limit() {
        let m = MachineSection {
            max_model_gb: Some(8.0),
            ..Default::default()
        };
        assert!(m.can_run_model(7.9));
        assert!(m.can_run_model(8.0));
    }

    // AC #2: feasibility predicate — model too large.
    #[test]
    fn can_run_model_false_when_exceeds_limit() {
        let m = MachineSection {
            max_model_gb: Some(8.0),
            ..Default::default()
        };
        assert!(!m.can_run_model(8.1));
        assert!(!m.can_run_model(70.0));
    }

    // AC #2: no limit declared → always feasible.
    #[test]
    fn can_run_model_true_when_no_limit() {
        let m = MachineSection::default();
        assert!(m.can_run_model(999.0));
    }

    // AC #3: validate_paths returns error for non-existent path.
    #[test]
    fn validate_paths_errors_on_missing_file() {
        let m = MachineSection {
            model_paths: vec![ModelPathEntry {
                name: "missing".into(),
                path: PathBuf::from("/nonexistent/model.gguf"),
                size_gb: 4.0,
            }],
            ..Default::default()
        };
        let err = m.validate_paths(Path::new("/")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing"), "error should name the entry: {msg}");
        assert!(msg.contains("does not exist"), "error should state reason: {msg}");
    }

    // AC #3: validate_paths is a no-op when model_paths is empty.
    #[test]
    fn validate_paths_ok_when_no_model_paths() {
        let m = MachineSection::default();
        m.validate_paths(Path::new("/")).unwrap();
    }

    // UserKeys: key_for returns None for an empty string value.
    #[test]
    fn user_keys_key_for_filters_empty_string() {
        let keys = UserKeys {
            anthropic_api_key: Some(String::new()),
            openai_api_key: Some("sk-real".into()),
            xai_api_key: None,
            openrouter_api_key: None,
            deepseek_api_key: None,
        };
        assert!(keys.key_for(&Provider::Anthropic).is_none());
        assert_eq!(keys.key_for(&Provider::OpenAI), Some("sk-real"));
        assert!(keys.key_for(&Provider::XAI).is_none());
    }

    // UserKeys: missing file → Ok(None).
    #[test]
    fn user_keys_load_returns_none_when_file_absent() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        // Safety: single-threaded test environment.
        unsafe { std::env::set_var("HOME", dir.path()) };
        let result = UserKeys::load().unwrap();
        assert!(result.is_none());
    }

    // UserKeys: valid file → parses correctly.
    #[test]
    fn user_keys_load_parses_valid_file() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let hick_dir = dir.path().join(".config/hick");
        std::fs::create_dir_all(&hick_dir).unwrap();
        std::fs::write(hick_dir.join("keys.toml"), r#"anthropic_api_key = "sk-ant-test""#).unwrap();
        // Safety: single-threaded test environment.
        unsafe { std::env::set_var("HOME", dir.path()) };
        let keys = UserKeys::load().unwrap().unwrap();
        assert_eq!(keys.anthropic_api_key.as_deref(), Some("sk-ant-test"));
        assert!(keys.openai_api_key.is_none());
    }

    // AC #3: validate_paths succeeds for an existing file.
    #[test]
    fn validate_paths_ok_for_existing_file() {
        use tempfile::NamedTempFile;
        let f = NamedTempFile::new().unwrap();
        let m = MachineSection {
            model_paths: vec![ModelPathEntry {
                name: "real-model".into(),
                path: f.path().to_path_buf(),
                size_gb: 2.0,
            }],
            ..Default::default()
        };
        m.validate_paths(Path::new("/")).unwrap();
    }

    // AC #3: relative path is resolved against base_dir.
    #[test]
    fn validate_paths_resolves_relative_paths() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        // Create a dummy file inside the temp dir
        let model_file = dir.path().join("model.gguf");
        std::fs::write(&model_file, b"dummy").unwrap();

        let m = MachineSection {
            model_paths: vec![ModelPathEntry {
                name: "local-model".into(),
                path: PathBuf::from("model.gguf"),
                size_gb: 1.0,
            }],
            ..Default::default()
        };
        m.validate_paths(dir.path()).unwrap();
    }

    // ---------------------------------------------------------------------------
    // [eval] section tests
    // ---------------------------------------------------------------------------

    #[test]
    fn eval_section_auto_prune_threshold_parses() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"

[eval]
auto_prune_threshold = 0.95
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Point(pc) = cfg else { panic!("expected Point") };
        assert_eq!(pc.eval.auto_prune_threshold, Some(0.95));
    }

    #[test]
    fn eval_section_auto_prune_threshold_absent_is_none() {
        let toml = r#"
[model]
model_id = "claude-opus-4-7-20251101"
"#;
        let cfg = AgentConfig::parse(toml).unwrap();
        let AgentConfig::Point(pc) = cfg else { panic!("expected Point") };
        assert!(pc.eval.auto_prune_threshold.is_none());
    }
}
