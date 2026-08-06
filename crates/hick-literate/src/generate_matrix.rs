//! Generate-matrix subcommand: generate all feature combinations.
//!
//! Creates a matrix of all feature combinations and generates each to a
//! separate output directory. Optionally runs verification commands.

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use log::info;

use crate::config::{HickConfig, find_config};

use crate::expand_path_arg;
use crate::visual_regression::{self, VisualRegressionConfig};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
pub struct GenerateMatrixArgs {
    /// `.hick` files or directory to process.
    pub files: Vec<PathBuf>,

    /// Path to config file (default: auto-discover `_hick.yml`).
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Set a variable: --param key=value
    #[arg(long = "param", value_parser = crate::parse_param)]
    pub params: Vec<(String, String)>,

    /// Output base directory for generated feature combinations.
    /// Each combination gets a subdirectory named after its features.
    #[arg(long, default_value = "matrix")]
    pub output_dir: PathBuf,

    /// Only include these features (comma-separated).
    /// If not specified, uses all leaf features (non-meta features).
    #[arg(long)]
    pub include_features: Option<String>,

    /// Exclude these features from combinations (comma-separated).
    #[arg(long)]
    pub exclude_features: Option<String>,

    /// Commands to run for verification after each combination.
    /// Commands run in the output directory. Use {{features}} for the
    /// comma-separated list of enabled features, {{dir}} for the directory.
    /// Prefix with [feature] to run only when feature is enabled.
    /// Use [f1,f2] for OR (any), [f1+f2] for AND (all).
    #[arg(long)]
    pub verify: Vec<String>,

    /// Timeout for verification commands in seconds.
    #[arg(long, default_value = "300")]
    pub verify_timeout: u64,

    /// Continue on verification failure instead of stopping.
    #[arg(long)]
    pub continue_on_error: bool,

    /// Only generate combinations that include at least one of these features.
    /// Useful to avoid generating the empty combination.
    #[arg(long)]
    pub require_any: Option<String>,

    /// Maximum number of features in a combination (0 = unlimited).
    #[arg(long, default_value = "0")]
    pub max_combination_size: usize,

    /// Generate only single-feature combinations (shortcut for --max-combination-size 1).
    #[arg(long)]
    pub singles_only: bool,

    /// Dry run: show what would be generated without actually generating.
    #[arg(long)]
    pub dry_run: bool,

    /// Enable verbose output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Number of parallel jobs (0 = auto-detect from CPU count).
    #[arg(short = 'j', long, default_value = "0")]
    pub jobs: usize,

    /// Skip confirmation prompt for embedded verify commands.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Screenshot capture command for visual regression (e.g., "devenv-pilot screenshot {{output}}")
    #[arg(long)]
    pub screenshot: Option<String>,

    /// Update baseline screenshots from current run
    #[arg(long)]
    pub update_baselines: bool,

    /// Pixel diff threshold for visual regression (0.0-1.0, default: 0.01)
    #[arg(long, default_value_t = 0.01)]
    pub screenshot_threshold: f64,
}

// ---------------------------------------------------------------------------
// Verify Command Parsing
// ---------------------------------------------------------------------------

/// A verify command with optional feature condition.
struct VerifyCommand {
    command: String,
    condition: VerifyCondition,
}

/// Condition for when a verify command should run.
enum VerifyCondition {
    /// Always run this command.
    Always,
    /// Run if ANY of these features are present (OR logic).
    AnyOf(Vec<String>),
    /// Run if ALL of these features are present (AND logic).
    AllOf(Vec<String>),
}

impl VerifyCommand {
    /// Parse "[condition] command" syntax.
    ///
    /// Examples:
    /// - `"cargo build"` → Always run
    /// - `"[rust-frontend-lib] cargo build"` → Run if rust-frontend-lib enabled
    /// - `"[spa,extension] npm test"` → Run if spa OR extension enabled (comma = OR)
    /// - `"[spa|extension] npm test"` → Run if spa OR extension enabled (pipe = OR)
    /// - `"[spa+backend] npm run e2e"` → Run if spa AND backend enabled
    fn parse(input: &str) -> Self {
        let input = input.trim();
        if input.starts_with('[')
            && let Some(end) = input.find(']')
        {
            let condition_str = &input[1..end];
            let command = input[end + 1..].trim().to_string();

            if condition_str.contains('+') {
                // AND logic
                let features: Vec<String> = condition_str
                    .split('+')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return Self {
                    command,
                    condition: VerifyCondition::AllOf(features),
                };
            } else {
                // OR logic (comma or pipe separated)
                let features: Vec<String> = condition_str
                    .split([',', '|'])
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return Self {
                    command,
                    condition: VerifyCondition::AnyOf(features),
                };
            }
        }
        Self {
            command: input.to_string(),
            condition: VerifyCondition::Always,
        }
    }

    /// Check if this command should run for the given feature combination.
    fn should_run(&self, enabled_features: &[String]) -> bool {
        match &self.condition {
            VerifyCondition::Always => true,
            VerifyCondition::AnyOf(required) => {
                required.iter().any(|f| enabled_features.contains(f))
            }
            VerifyCondition::AllOf(required) => {
                required.iter().all(|f| enabled_features.contains(f))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Timing analysis
// ---------------------------------------------------------------------------

/// Data for tracking timing of each combination (for linear regression).
#[derive(Clone)]
struct ComboTiming {
    features: Vec<String>,
    duration_secs: f64,
}

/// Simple linear regression to identify which features contribute most to runtime.
/// Uses ordinary least squares with feature indicator variables.
#[allow(clippy::needless_range_loop)]
fn analyze_feature_timing(all_features: &[String], timings: &[ComboTiming]) -> Vec<(String, f64)> {
    if timings.is_empty() || all_features.is_empty() {
        return vec![];
    }

    let n = timings.len();
    let m = all_features.len();

    // Build design matrix X (n x m+1) with intercept column
    // X[i][0] = 1 (intercept)
    // X[i][j+1] = 1 if feature j is present in combo i, else 0
    let mut x: Vec<Vec<f64>> = vec![vec![0.0; m + 1]; n];
    let mut y: Vec<f64> = vec![0.0; n];

    for (i, timing) in timings.iter().enumerate() {
        x[i][0] = 1.0; // intercept
        for (j, feat) in all_features.iter().enumerate() {
            if timing.features.contains(feat) {
                x[i][j + 1] = 1.0;
            }
        }
        y[i] = timing.duration_secs;
    }

    // Compute X^T * X
    let mut xtx: Vec<Vec<f64>> = vec![vec![0.0; m + 1]; m + 1];
    for i in 0..=m {
        for j in 0..=m {
            for xk in x.iter().take(n) {
                xtx[i][j] += xk[i] * xk[j];
            }
        }
    }

    // Compute X^T * y
    let mut xty: Vec<f64> = vec![0.0; m + 1];
    for i in 0..=m {
        for k in 0..n {
            xty[i] += x[k][i] * y[k];
        }
    }

    // Solve (X^T * X) * beta = X^T * y using Gaussian elimination
    // Augment matrix [xtx | xty]
    let mut aug: Vec<Vec<f64>> = vec![vec![0.0; m + 2]; m + 1];
    for i in 0..=m {
        for j in 0..=m {
            aug[i][j] = xtx[i][j];
        }
        aug[i][m + 1] = xty[i];
    }

    // Forward elimination with partial pivoting
    for col in 0..=m {
        // Find pivot
        let mut max_row = col;
        for row in (col + 1)..=m {
            if aug[row][col].abs() > aug[max_row][col].abs() {
                max_row = row;
            }
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        if pivot.abs() < 1e-10 {
            continue; // Skip singular column
        }

        // Eliminate below
        for row in (col + 1)..=m {
            let factor = aug[row][col] / pivot;
            for j in col..=(m + 1) {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Back substitution
    let mut beta: Vec<f64> = vec![0.0; m + 1];
    for i in (0..=m).rev() {
        let mut sum = aug[i][m + 1];
        for j in (i + 1)..=m {
            sum -= aug[i][j] * beta[j];
        }
        if aug[i][i].abs() > 1e-10 {
            beta[i] = sum / aug[i][i];
        }
    }

    // Return feature coefficients (skip intercept at index 0)
    let mut results: Vec<(String, f64)> = all_features
        .iter()
        .enumerate()
        .map(|(i, feat)| (feat.clone(), beta[i + 1]))
        .collect();

    // Sort by coefficient (descending) to show slowest features first
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

// ---------------------------------------------------------------------------
// Progress display
// ---------------------------------------------------------------------------

/// Shared state for real-time progress display.
struct MatrixProgress {
    completed: std::sync::atomic::AtomicUsize,
    total: usize,
    failed: std::sync::atomic::AtomicUsize,
    cumulative_ms: std::sync::atomic::AtomicU64,
    timings: tokio::sync::Mutex<Vec<ComboTiming>>,
    all_features: Vec<String>,
    current_combo: tokio::sync::Mutex<String>,
}

impl MatrixProgress {
    fn new(total: usize, all_features: Vec<String>) -> Self {
        Self {
            completed: std::sync::atomic::AtomicUsize::new(0),
            total,
            failed: std::sync::atomic::AtomicUsize::new(0),
            cumulative_ms: std::sync::atomic::AtomicU64::new(0),
            timings: tokio::sync::Mutex::new(Vec::new()),
            all_features,
            current_combo: tokio::sync::Mutex::new(String::new()),
        }
    }

    async fn record_success(&self, combo: &[String], duration: std::time::Duration) {
        use std::sync::atomic::Ordering;
        self.completed.fetch_add(1, Ordering::Relaxed);
        self.cumulative_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        self.timings.lock().await.push(ComboTiming {
            features: combo.to_vec(),
            duration_secs: duration.as_secs_f64(),
        });
    }

    fn record_failure(&self) {
        use std::sync::atomic::Ordering;
        self.completed.fetch_add(1, Ordering::Relaxed);
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    async fn set_current(&self, name: &str) {
        *self.current_combo.lock().await = name.to_string();
    }

    async fn get_display_state(&self) -> (usize, usize, usize, u64, String, Vec<ComboTiming>) {
        use std::sync::atomic::Ordering;
        let completed = self.completed.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let cumulative_ms = self.cumulative_ms.load(Ordering::Relaxed);
        let current = self.current_combo.lock().await.clone();
        let timings = self.timings.lock().await.clone();
        (
            completed,
            self.total,
            failed,
            cumulative_ms,
            current,
            timings,
        )
    }
}

/// Format bytes as human-readable size.
fn fmt_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Render the real-time progress display.
#[allow(clippy::too_many_arguments)]
fn render_progress(
    completed: usize,
    total: usize,
    failed: usize,
    cumulative_ms: u64,
    current: &str,
    timings: &[ComboTiming],
    all_features: &[String],
    cpu_usage: f32,
    mem_bytes: u64,
    is_tty: bool,
) {
    use std::io::Write;
    use std::time::Duration;

    if total == 0 {
        return;
    }

    let pct = (completed as f64 / total as f64) * 100.0;
    let avg_ms = if completed > 0 {
        cumulative_ms / completed as u64
    } else {
        0
    };
    let remaining = total.saturating_sub(completed);
    let eta_ms = avg_ms * remaining as u64;
    let eta = crate::fmt_duration(Duration::from_millis(eta_ms));

    // Progress bar (20 chars wide)
    let bar_width = 20;
    let filled = ((pct / 100.0) * bar_width as f64) as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

    // Build the status line
    let status = if failed > 0 {
        format!(
            "\x1b[2K\rMatrix: {:>5.1}% {} {}/{} ({} failed)  cpu:{:>3.0}%  mem:{}  eta:{}  {}",
            pct,
            bar,
            completed,
            total,
            failed,
            cpu_usage,
            fmt_bytes(mem_bytes),
            eta,
            if current.len() > 20 {
                format!("{}...", &current[..17])
            } else {
                current.to_string()
            }
        )
    } else {
        format!(
            "\x1b[2K\rMatrix: {:>5.1}% {} {}/{}  cpu:{:>3.0}%  mem:{}  eta:{}  {}",
            pct,
            bar,
            completed,
            total,
            cpu_usage,
            fmt_bytes(mem_bytes),
            eta,
            if current.len() > 20 {
                format!("{}...", &current[..17])
            } else {
                current.to_string()
            }
        )
    };

    // Feature impact line (only if we have enough data)
    let impact_line = if timings.len() >= all_features.len().max(3) && !all_features.is_empty() {
        let impacts = analyze_feature_timing(all_features, timings);
        let top_impacts: Vec<String> = impacts
            .iter()
            .take(3)
            .filter(|(_, coef)| *coef > 0.01) // Only show significant impacts
            .map(|(feat, coef)| {
                format!(
                    "{} +{}",
                    if feat.len() > 12 {
                        format!("{}…", &feat[..11])
                    } else {
                        feat.clone()
                    },
                    crate::fmt_duration(Duration::from_secs_f64(*coef))
                )
            })
            .collect();
        if !top_impacts.is_empty() {
            format!("\n\x1b[2K  slowest: {}", top_impacts.join(", "))
        } else {
            String::new()
        }
    } else if !timings.is_empty() && timings.len() < all_features.len().max(3) {
        format!(
            "\n\x1b[2K  gathering data... ({}/{})",
            timings.len(),
            all_features.len().max(3)
        )
    } else {
        String::new()
    };

    if is_tty {
        // Move cursor up if we have a second line
        let line_count = if impact_line.is_empty() { 1 } else { 2 };
        if line_count == 2 {
            print!("\x1b[2A"); // Move up 2 lines
        } else {
            print!("\x1b[1A"); // Move up 1 line
        }
        print!("{}{}", status, impact_line);
        if line_count == 2 {
            println!(); // Ensure we're on the line after the second line
        }
        println!();
        let _ = std::io::stdout().flush();
    } else {
        // Non-TTY: just print completion updates
        if completed == total || completed.is_multiple_of(10) {
            println!("Matrix: {:.1}% {}/{}  eta:{}", pct, completed, total, eta);
        }
    }
}

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

/// Run the generate-matrix command: generate all feature combinations.
pub async fn run(cli: GenerateMatrixArgs) -> Result<()> {
    use futures::stream::{self, StreamExt};
    use std::collections::HashSet;
    use std::io::IsTerminal;
    use std::process::Command as ProcessCommand;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use sysinfo::System;

    let start_time = Instant::now();
    let is_tty = std::io::stdout().is_terminal();

    // Suppress INFO logs during matrix generation unless --verbose
    // This prevents noisy DAG validation messages from cluttering the UI
    if !cli.verbose {
        log::set_max_level(log::LevelFilter::Warn);
    }

    // Expand CLI file arguments
    let (expanded_files, dir_config) = if !cli.files.is_empty() {
        let mut all_files = Vec::new();
        let mut found_config = None;
        for path in &cli.files {
            let (files, cfg) = expand_path_arg(path)?;
            all_files.extend(files);
            if cfg.is_some() {
                found_config = cfg;
            }
        }
        (all_files, found_config)
    } else {
        (Vec::new(), None)
    };

    // Discover config file
    let config_path = cli
        .config
        .clone()
        .or(dir_config)
        .or_else(|| find_config(&std::env::current_dir().unwrap_or_default()));

    let config = if let Some(ref path) = config_path {
        info!("Using config: {}", path.display());
        HickConfig::load(path)?
    } else {
        HickConfig::default()
    };

    let config_dir = config_path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(Path::new("."));

    // Resolve files
    let files = if !expanded_files.is_empty() {
        expanded_files
    } else {
        config.resolve_files(config_dir)?
    };

    if files.is_empty() {
        bail!("No .hick files specified");
    }

    // Read all input files
    let mut file_contents = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        file_contents.push((path.display().to_string(), source));
    }

    // Build sources slice for feature extraction
    let sources: Vec<(&str, &str)> = file_contents
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();

    // Extract feature definitions from the hick files
    let feature_defs = crate::extract_feature_definitions(&sources)?;

    // Extract embedded verify commands from the hick files
    let embedded_verify = crate::extract_verify_commands(&sources)?;

    // Identify leaf features (those that are not meta-features)
    // A leaf feature is one that doesn't have any `requires` attributes
    let leaf_features: Vec<String> = feature_defs
        .iter()
        .filter(|(_, def)| def.requires.is_empty())
        .map(|(name, _)| name.clone())
        .collect();

    let meta_features: Vec<String> = feature_defs
        .iter()
        .filter(|(_, def)| !def.requires.is_empty())
        .map(|(name, _)| name.clone())
        .collect();

    info!(
        "Found {} features: {} leaf, {} meta",
        feature_defs.len(),
        leaf_features.len(),
        meta_features.len()
    );

    // Determine which features to use for combinations
    let mut features_to_combine: Vec<String> = if let Some(ref include) = cli.include_features {
        include
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        leaf_features.clone()
    };

    // Apply exclusions
    if let Some(ref exclude) = cli.exclude_features {
        let exclusions: HashSet<String> =
            exclude.split(',').map(|s| s.trim().to_string()).collect();
        features_to_combine.retain(|f| !exclusions.contains(f));
    }

    // Sort for consistent ordering
    features_to_combine.sort();

    info!(
        "Generating combinations for features: {:?}",
        features_to_combine
    );

    // Generate power set of feature combinations
    let max_size = if cli.singles_only {
        1
    } else if cli.max_combination_size > 0 {
        cli.max_combination_size
    } else {
        features_to_combine.len()
    };

    let mut combinations: Vec<Vec<String>> = Vec::new();
    generate_combinations(&features_to_combine, max_size, &mut combinations);

    // Filter out empty combination if require_any is specified
    if let Some(ref require_any) = cli.require_any {
        let required: HashSet<String> = require_any
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        combinations.retain(|combo| combo.iter().any(|f| required.contains(f)));
    } else {
        // By default, skip the empty combination
        combinations.retain(|combo| !combo.is_empty());
    }

    // Filter out combinations with conflicting features
    let pre_filter_count = combinations.len();
    combinations.retain(|combo| {
        // Check for pairwise conflicts
        for (i, a) in combo.iter().enumerate() {
            for b in combo.iter().skip(i + 1) {
                // Check explicit conflicts_with (either direction)
                if let Some(def_a) = feature_defs.get(a)
                    && def_a.conflicts_with.iter().any(|c| c == b)
                {
                    return false;
                }
                if let Some(def_b) = feature_defs.get(b)
                    && def_b.conflicts_with.iter().any(|c| c == a)
                {
                    return false;
                }

                // Check exclusive groups
                if let (Some(def_a), Some(def_b)) = (feature_defs.get(a), feature_defs.get(b))
                    && let (Some(group_a), Some(group_b)) =
                        (&def_a.exclusive_group, &def_b.exclusive_group)
                    && group_a == group_b
                {
                    return false;
                }
            }
        }
        true
    });

    let filtered_count = pre_filter_count - combinations.len();
    if filtered_count > 0 {
        info!(
            "Filtered out {} combinations due to feature conflicts",
            filtered_count
        );
    }

    info!(
        "Generated {} feature combinations (max size: {})",
        combinations.len(),
        max_size
    );

    // Confirm embedded verify commands (security: these come from files, not CLI)
    if !embedded_verify.is_empty() && !cli.dry_run {
        println!("Embedded verify commands found in hick files:\n");
        for (i, cmd) in embedded_verify.iter().enumerate() {
            let desc = cmd.description.as_deref().unwrap_or("");
            if desc.is_empty() {
                println!("  {}. {}", i + 1, cmd.command);
            } else {
                println!("  {}. {} — {}", i + 1, cmd.command, desc);
            }
            println!("     (from {}:{})", cmd.source_file, cmd.source_line);
        }
        println!();

        if !cli.yes {
            if !is_tty {
                anyhow::bail!("embedded verify commands require --yes in non-interactive mode");
            }
            eprint!("Run these commands? [y/N] ");
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer)?;
            if !answer.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }
    }

    if cli.dry_run {
        println!("Feature combinations (dry run):\n");
        for combo in &combinations {
            let name = if combo.is_empty() {
                "none".to_string()
            } else {
                combo.join("-")
            };
            println!("  {}/", name);
        }
        println!("\nTotal: {} combinations", combinations.len());

        if !embedded_verify.is_empty() || !cli.verify.is_empty() {
            println!("\nVerify commands:");
            for cmd in &embedded_verify {
                let desc = cmd.description.as_deref().unwrap_or("");
                if desc.is_empty() {
                    println!("  [embedded] {}", cmd.command);
                } else {
                    println!("  [embedded] {} — {}", cmd.command, desc);
                }
            }
            for v in &cli.verify {
                println!("  [cli]      {}", v);
            }
        }

        return Ok(());
    }

    // Create output directory
    std::fs::create_dir_all(&cli.output_dir).with_context(|| {
        format!(
            "failed to create output directory: {}",
            cli.output_dir.display()
        )
    })?;

    // Merge config vars with CLI params
    let mut base_params: Vec<(String, String)> = config
        .vars
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (k, v) in &cli.params {
        base_params.retain(|(pk, _)| pk != k);
        base_params.push((k.clone(), v.clone()));
    }

    // Determine parallelism
    let jobs = if cli.jobs == 0 {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    } else {
        cli.jobs
    };

    info!("Running with {} parallel job(s)", jobs);

    // Parse verify commands once before the loop: embedded first, then CLI
    let mut verify_cmds: Vec<VerifyCommand> = embedded_verify
        .iter()
        .map(|ev| VerifyCommand::parse(&ev.command))
        .collect();
    verify_cmds.extend(cli.verify.iter().map(|s| VerifyCommand::parse(s)));

    // Set up visual regression config if --screenshot is provided
    let vr_config = cli
        .screenshot
        .as_ref()
        .map(|capture_cmd| VisualRegressionConfig {
            baseline_dir: cli.output_dir.join(".baselines"),
            output_dir: cli.output_dir.join(".screenshots"),
            diff_dir: cli.output_dir.join(".diffs"),
            threshold: cli.screenshot_threshold,
            capture_command: Some(capture_cmd.clone()),
        });

    // Wrap shared data in Arc for parallel access
    let file_contents = Arc::new(file_contents);
    let base_params = Arc::new(base_params);
    let output_dir = Arc::new(cli.output_dir.clone());
    let verify_cmds = Arc::new(verify_cmds);
    let vr_config = Arc::new(vr_config);
    let continue_on_error = cli.continue_on_error;
    let total_combinations = combinations.len();

    // Flag to signal cancellation on first failure (when !continue_on_error)
    let cancelled = Arc::new(AtomicBool::new(false));

    // Collect failures for final report
    let failures = Arc::new(tokio::sync::Mutex::new(Vec::<(String, String)>::new()));

    // Collect visual regression results
    let vr_results = Arc::new(tokio::sync::Mutex::new(Vec::<
        visual_regression::ComparisonResult,
    >::new()));

    // Clone features list for regression analysis
    let all_features_for_analysis: Vec<String> = features_to_combine.clone();

    // Create shared progress state
    let progress = Arc::new(MatrixProgress::new(
        total_combinations,
        all_features_for_analysis.clone(),
    ));

    // Spawn background task to update progress display with CPU/RAM info
    let progress_for_display = progress.clone();
    let display_cancelled = cancelled.clone();
    let display_handle = tokio::spawn(async move {
        let mut sys = System::new();
        let pid = sysinfo::get_current_pid().ok();

        // Print initial blank lines for progress display
        if is_tty {
            println!();
            println!();
        }

        loop {
            // Refresh system info
            sys.refresh_all();

            // Get process-specific CPU and memory
            let (cpu_usage, mem_bytes) = if let Some(pid) = pid {
                if let Some(proc) = sys.process(pid) {
                    (proc.cpu_usage(), proc.memory())
                } else {
                    (0.0, 0)
                }
            } else {
                (0.0, 0)
            };

            let (completed, total, failed, cumulative_ms, current, timings) =
                progress_for_display.get_display_state().await;

            render_progress(
                completed,
                total,
                failed,
                cumulative_ms,
                &current,
                &timings,
                &progress_for_display.all_features,
                cpu_usage,
                mem_bytes,
                is_tty,
            );

            if completed >= total || display_cancelled.load(Ordering::Relaxed) {
                break;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    // Process combinations in parallel
    let results: Vec<Result<String, ()>> = stream::iter(combinations.into_iter().enumerate())
        .map(|(_i, combo)| {
            let file_contents = file_contents.clone();
            let base_params = base_params.clone();
            let output_dir = output_dir.clone();
            let verify_cmds = verify_cmds.clone();
            let vr_config = vr_config.clone();
            let vr_results = vr_results.clone();
            let cancelled = cancelled.clone();
            let failures = failures.clone();
            let progress = progress.clone();

            async move {
                // Check if we should skip due to earlier failure
                if cancelled.load(Ordering::Relaxed) {
                    return Err(());
                }

                // Start timing this combination
                let combo_start = Instant::now();

                let combo_name = if combo.is_empty() {
                    "none".to_string()
                } else {
                    combo.join("-")
                };

                // Update current combo for display
                progress.set_current(&combo_name).await;

                let combo_dir = output_dir.join(&combo_name);
                let features_str = combo.join(",");

                // Build sources from file_contents
                let sources: Vec<(&str, &str)> = file_contents
                    .iter()
                    .map(|(name, content)| (name.as_str(), content.as_str()))
                    .collect();

                // Build params for this combination
                let mut params: Vec<(String, String)> = base_params
                    .iter()
                    .filter(|(pk, _)| pk != "features")
                    .cloned()
                    .collect();
                if !combo.is_empty() {
                    params.push(("features".to_string(), features_str.clone()));
                }

                // Run the pipeline for this combination
                let result = crate::run_pipeline(&sources, &params).await;
                let combo_elapsed = combo_start.elapsed();

                match result {
                    Ok(pipeline_result) => {
                        // Write outputs
                        if let Err(e) = std::fs::create_dir_all(&combo_dir) {
                            let error_msg = format!("failed to create directory: {}", e);
                            progress.record_failure();
                            failures.lock().await.push((combo_name.clone(), error_msg));
                            if !continue_on_error {
                                cancelled.store(true, Ordering::Relaxed);
                            }
                            return Err(());
                        }

                        for (path, content) in &pipeline_result.files {
                            let full_path = combo_dir.join(path);
                            if let Some(parent) = full_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            let write_result = match content {
                                hick_exec::node::FileContent::Text(s) => {
                                    std::fs::write(&full_path, s)
                                }
                                hick_exec::node::FileContent::Binary(d) => {
                                    std::fs::write(&full_path, d.to_bytes().unwrap_or_default())
                                }
                            };
                            if let Err(e) = write_result {
                                let error_msg = format!("failed to write {}: {}", path, e);
                                progress.record_failure();
                                failures.lock().await.push((combo_name.clone(), error_msg));
                                if !continue_on_error {
                                    cancelled.store(true, Ordering::Relaxed);
                                }
                                return Err(());
                            }
                        }

                        // Run verification commands
                        for verify in verify_cmds.iter() {
                            // Skip commands whose conditions don't match this combo
                            if !verify.should_run(&combo) {
                                continue;
                            }

                            let cmd_with_features =
                                verify.command.replace("{{features}}", &features_str);
                            let cmd_with_dir =
                                cmd_with_features.replace("{{dir}}", &combo_dir.to_string_lossy());

                            let combo_dir_for_verify = combo_dir.clone();
                            let verify_result = tokio::task::spawn_blocking(move || {
                                ProcessCommand::new("sh")
                                    .arg("-c")
                                    .arg(&cmd_with_dir)
                                    .current_dir(&combo_dir_for_verify)
                                    .output()
                            })
                            .await;

                            match verify_result {
                                Ok(Ok(output)) => {
                                    if !output.status.success() {
                                        let stderr = String::from_utf8_lossy(&output.stderr);
                                        let stdout = String::from_utf8_lossy(&output.stdout);
                                        let error_msg = format!(
                                            "exit code: {:?}\nstdout: {}\nstderr: {}",
                                            output.status.code(),
                                            stdout,
                                            stderr
                                        );
                                        progress.record_failure();
                                        failures.lock().await.push((combo_name.clone(), error_msg));
                                        if !continue_on_error {
                                            cancelled.store(true, Ordering::Relaxed);
                                        }
                                        return Err(());
                                    }
                                }
                                Ok(Err(e)) => {
                                    let error_msg = format!("failed to run command: {}", e);
                                    progress.record_failure();
                                    failures.lock().await.push((combo_name.clone(), error_msg));
                                    if !continue_on_error {
                                        cancelled.store(true, Ordering::Relaxed);
                                    }
                                    return Err(());
                                }
                                Err(e) => {
                                    let error_msg = format!("task join error: {}", e);
                                    progress.record_failure();
                                    failures.lock().await.push((combo_name.clone(), error_msg));
                                    if !continue_on_error {
                                        cancelled.store(true, Ordering::Relaxed);
                                    }
                                    return Err(());
                                }
                            }
                        }

                        // Run visual regression if configured
                        if let Some(ref vr_cfg) = *vr_config {
                            let capture_cmd = vr_cfg
                                .capture_command
                                .as_deref()
                                .unwrap_or("echo 'no capture command'");
                            match visual_regression::capture_screenshot(
                                vr_cfg,
                                &combo_name,
                                capture_cmd,
                                &combo_dir,
                            ) {
                                Ok(screenshot_path) => {
                                    match visual_regression::compare_screenshot(
                                        vr_cfg,
                                        &combo_name,
                                        &screenshot_path,
                                    ) {
                                        Ok(Some(comparison)) => {
                                            if !comparison.passed {
                                                let error_msg = format!(
                                                    "visual regression failed: diff ratio {:.4} exceeds threshold {:.4}",
                                                    comparison.pixel_diff_ratio, vr_cfg.threshold
                                                );
                                                if !continue_on_error {
                                                    progress.record_failure();
                                                    failures
                                                        .lock()
                                                        .await
                                                        .push((combo_name.clone(), error_msg));
                                                    cancelled.store(true, Ordering::Relaxed);
                                                    vr_results.lock().await.push(comparison);
                                                    return Err(());
                                                }
                                            }
                                            vr_results.lock().await.push(comparison);
                                        }
                                        Ok(None) => {
                                            // First run, baseline created
                                            info!(
                                                "Created baseline screenshot for {}",
                                                combo_name
                                            );
                                        }
                                        Err(e) => {
                                            let error_msg =
                                                format!("screenshot comparison failed: {}", e);
                                            progress.record_failure();
                                            failures
                                                .lock()
                                                .await
                                                .push((combo_name.clone(), error_msg));
                                            if !continue_on_error {
                                                cancelled.store(true, Ordering::Relaxed);
                                            }
                                            return Err(());
                                        }
                                    }
                                }
                                Err(e) => {
                                    let error_msg =
                                        format!("screenshot capture failed: {}", e);
                                    progress.record_failure();
                                    failures
                                        .lock()
                                        .await
                                        .push((combo_name.clone(), error_msg));
                                    if !continue_on_error {
                                        cancelled.store(true, Ordering::Relaxed);
                                    }
                                    return Err(());
                                }
                            }
                        }

                        // Record success with timing
                        progress.record_success(&combo, combo_elapsed).await;
                        Ok(combo_name)
                    }
                    Err(e) => {
                        let error_msg = format!("{}", e);
                        progress.record_failure();
                        failures.lock().await.push((combo_name, error_msg));
                        if !continue_on_error {
                            cancelled.store(true, Ordering::Relaxed);
                        }
                        Err(())
                    }
                }
            }
        })
        .buffer_unordered(jobs)
        .collect()
        .await;

    // Wait for display task to finish
    let _ = display_handle.await;

    // Count results
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.iter().filter(|r| r.is_err()).count();
    let failures = match Arc::try_unwrap(failures) {
        Ok(mutex) => mutex.into_inner(),
        Err(arc) => arc.lock().await.clone(),
    };

    // Extract timings for analysis from progress state
    let timings_data = match Arc::try_unwrap(progress) {
        Ok(p) => p.timings.into_inner(),
        Err(arc) => arc.timings.lock().await.clone(),
    };

    // Clear progress lines and print summary
    let elapsed = start_time.elapsed();
    if is_tty {
        print!("\x1b[2K\r"); // Clear the progress line
    }
    println!("\n--- Summary ---");
    println!("Total combinations: {}", total_combinations);
    println!("Parallel jobs: {}", jobs);
    println!("Successful: {}", successful);
    println!("Failed: {}", failed);
    println!("Total time: {}", crate::fmt_duration(elapsed));

    // Calculate timing statistics
    if !timings_data.is_empty() {
        let total_combo_time: f64 = timings_data.iter().map(|t| t.duration_secs).sum();
        let avg_time = total_combo_time / timings_data.len() as f64;
        let min_time = timings_data
            .iter()
            .map(|t| t.duration_secs)
            .fold(f64::INFINITY, f64::min);
        let max_time = timings_data
            .iter()
            .map(|t| t.duration_secs)
            .fold(f64::NEG_INFINITY, f64::max);

        println!("\n--- Timing Statistics ---");
        println!(
            "Average time per combination: {}",
            crate::fmt_duration(Duration::from_secs_f64(avg_time))
        );
        println!(
            "Min time: {}",
            crate::fmt_duration(Duration::from_secs_f64(min_time))
        );
        println!(
            "Max time: {}",
            crate::fmt_duration(Duration::from_secs_f64(max_time))
        );
        println!(
            "Sum of all iteration times: {}",
            crate::fmt_duration(Duration::from_secs_f64(total_combo_time))
        );

        // Run linear regression to identify slow features
        if !all_features_for_analysis.is_empty() && timings_data.len() >= 2 {
            let feature_impacts = analyze_feature_timing(&all_features_for_analysis, &timings_data);

            if !feature_impacts.is_empty() {
                println!("\n--- Feature Time Impact (Linear Regression) ---");
                println!("Estimated additional time per feature (sorted by impact):");
                for (feat, coef) in &feature_impacts {
                    let sign = if *coef >= 0.0 { "+" } else { "" };
                    println!(
                        "  {:20} {}{}",
                        feat,
                        sign,
                        crate::fmt_duration(Duration::from_secs_f64(coef.abs()))
                    );
                }
            }
        }
    }

    if !failures.is_empty() {
        println!("\nFailures:");
        for (name, error) in &failures {
            println!(
                "  {} - {}",
                name,
                error.lines().next().unwrap_or("unknown error")
            );
        }
    }

    // Visual regression summary
    let vr_results_data = match Arc::try_unwrap(vr_results) {
        Ok(mutex) => mutex.into_inner(),
        Err(arc) => arc.lock().await.clone(),
    };
    if !vr_results_data.is_empty() {
        println!("\n--- Visual Regression ---");
        println!("{}", visual_regression::summary_report(&vr_results_data));
    }

    // Handle --update-baselines
    if cli.update_baselines {
        if let Some(ref vr_cfg) = *vr_config {
            let combo_names: Vec<String> = results
                .iter()
                .filter_map(|r| r.as_ref().ok().cloned())
                .collect();
            match visual_regression::update_baselines(vr_cfg, &combo_names) {
                Ok(n) => println!("\nUpdated {} baseline screenshot(s)", n),
                Err(e) => eprintln!("\nWarning: failed to update baselines: {}", e),
            }
        } else {
            eprintln!("\nWarning: --update-baselines requires --screenshot");
        }
    }

    if failed > 0 {
        bail!("{} combination(s) failed", failed);
    }

    Ok(())
}

/// Generate all combinations of features up to max_size.
fn generate_combinations(features: &[String], max_size: usize, result: &mut Vec<Vec<String>>) {
    let n = features.len();
    // Power set: iterate through all 2^n combinations
    for mask in 0..(1usize << n) {
        let mut combo: Vec<String> = Vec::new();
        for (i, feature) in features.iter().enumerate() {
            if mask & (1 << i) != 0 {
                combo.push(feature.clone());
            }
        }
        if combo.len() <= max_size {
            result.push(combo);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_command_parse_unconditional() {
        let cmd = VerifyCommand::parse("cargo build");
        assert_eq!(cmd.command, "cargo build");
        assert!(matches!(cmd.condition, VerifyCondition::Always));
        assert!(cmd.should_run(&[]));
        assert!(cmd.should_run(&["spa".to_string()]));
    }

    #[test]
    fn verify_command_parse_single_feature() {
        let cmd = VerifyCommand::parse("[rust-frontend-lib] cargo check");
        assert_eq!(cmd.command, "cargo check");
        assert!(matches!(cmd.condition, VerifyCondition::AnyOf(_)));

        // Should run when feature is enabled
        assert!(cmd.should_run(&["rust-frontend-lib".to_string()]));
        assert!(cmd.should_run(&["other".to_string(), "rust-frontend-lib".to_string()]));

        // Should not run when feature is disabled
        assert!(!cmd.should_run(&[]));
        assert!(!cmd.should_run(&["spa".to_string()]));
    }

    #[test]
    fn verify_command_parse_or_comma() {
        let cmd = VerifyCommand::parse("[spa,extension] npm test");
        assert_eq!(cmd.command, "npm test");
        if let VerifyCondition::AnyOf(features) = &cmd.condition {
            assert_eq!(features, &["spa", "extension"]);
        } else {
            panic!("Expected AnyOf condition");
        }

        // Should run if ANY of the features is enabled
        assert!(cmd.should_run(&["spa".to_string()]));
        assert!(cmd.should_run(&["extension".to_string()]));
        assert!(cmd.should_run(&["spa".to_string(), "extension".to_string()]));

        // Should not run if none are enabled
        assert!(!cmd.should_run(&[]));
        assert!(!cmd.should_run(&["backend".to_string()]));
    }

    #[test]
    fn verify_command_parse_or_pipe() {
        let cmd = VerifyCommand::parse("[spa|extension] npm test");
        assert_eq!(cmd.command, "npm test");
        if let VerifyCondition::AnyOf(features) = &cmd.condition {
            assert_eq!(features, &["spa", "extension"]);
        } else {
            panic!("Expected AnyOf condition");
        }
    }

    #[test]
    fn verify_command_parse_and() {
        let cmd = VerifyCommand::parse("[spa+backend] npm run e2e");
        assert_eq!(cmd.command, "npm run e2e");
        if let VerifyCondition::AllOf(features) = &cmd.condition {
            assert_eq!(features, &["spa", "backend"]);
        } else {
            panic!("Expected AllOf condition");
        }

        // Should run only if ALL features are enabled
        assert!(cmd.should_run(&["spa".to_string(), "backend".to_string()]));
        assert!(cmd.should_run(&[
            "spa".to_string(),
            "backend".to_string(),
            "other".to_string()
        ]));

        // Should not run if any are missing
        assert!(!cmd.should_run(&["spa".to_string()]));
        assert!(!cmd.should_run(&["backend".to_string()]));
        assert!(!cmd.should_run(&[]));
    }

    #[test]
    fn verify_command_parse_whitespace_handling() {
        // Spaces around features
        let cmd = VerifyCommand::parse("[ spa , extension ] npm test");
        assert_eq!(cmd.command, "npm test");
        if let VerifyCondition::AnyOf(features) = &cmd.condition {
            assert_eq!(features, &["spa", "extension"]);
        } else {
            panic!("Expected AnyOf condition");
        }

        // Leading/trailing whitespace
        let cmd2 = VerifyCommand::parse("  cargo build  ");
        assert_eq!(cmd2.command, "cargo build");
    }

    #[test]
    fn verify_command_parse_empty_brackets() {
        // Edge case: empty brackets should result in empty AnyOf
        let cmd = VerifyCommand::parse("[] cargo build");
        assert_eq!(cmd.command, "cargo build");
        // Empty AnyOf should never match
        assert!(!cmd.should_run(&["spa".to_string()]));
    }

    // Embedded verify command round-trip tests

    #[test]
    fn embedded_verify_roundtrips_through_parse() {
        // Simulate what happens: extract command string from hick file, then parse it
        let embedded_cmds = [
            "[backend] cargo test",
            "[spa+backend] npm run e2e",
            "cargo check",
        ];

        let parsed: Vec<VerifyCommand> = embedded_cmds
            .iter()
            .map(|s| VerifyCommand::parse(s))
            .collect();

        // Conditional command
        assert_eq!(parsed[0].command, "cargo test");
        assert!(parsed[0].should_run(&["backend".to_string()]));
        assert!(!parsed[0].should_run(&["spa".to_string()]));

        // AND condition
        assert_eq!(parsed[1].command, "npm run e2e");
        assert!(parsed[1].should_run(&["spa".to_string(), "backend".to_string()]));
        assert!(!parsed[1].should_run(&["spa".to_string()]));

        // Unconditional
        assert_eq!(parsed[2].command, "cargo check");
        assert!(parsed[2].should_run(&[]));
    }
}
