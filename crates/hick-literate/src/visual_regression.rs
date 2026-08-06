//! Visual regression testing for generate-matrix feature combinations.
//!
//! Captures screenshots of running applications and compares them against
//! baseline images. Uses devenv-pilot for screenshot capture when available,
//! or falls back to a configurable screenshot command.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Configuration for visual regression testing.
#[derive(Debug, Clone)]
pub struct VisualRegressionConfig {
    /// Directory storing baseline screenshots.
    pub baseline_dir: PathBuf,
    /// Directory for current run screenshots.
    pub output_dir: PathBuf,
    /// Directory for diff images.
    pub diff_dir: PathBuf,
    /// Pixel difference threshold (0.0 = exact, 1.0 = any).
    pub threshold: f64,
    /// Command to capture screenshot (default: devenv-pilot screenshot).
    pub capture_command: Option<String>,
}

impl Default for VisualRegressionConfig {
    fn default() -> Self {
        Self {
            baseline_dir: PathBuf::from("matrix/.baselines"),
            output_dir: PathBuf::from("matrix/.screenshots"),
            diff_dir: PathBuf::from("matrix/.diffs"),
            threshold: 0.01,
            capture_command: None,
        }
    }
}

/// Result of comparing a screenshot against its baseline.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are part of the public API for callers to inspect
pub struct ComparisonResult {
    pub combo_name: String,
    pub screenshot_path: PathBuf,
    pub baseline_path: PathBuf,
    pub diff_path: Option<PathBuf>,
    pub pixel_diff_ratio: f64,
    pub passed: bool,
}

/// Capture a screenshot for a feature combination.
pub fn capture_screenshot(
    config: &VisualRegressionConfig,
    combo_name: &str,
    capture_cmd: &str,
    combo_dir: &Path,
) -> Result<PathBuf> {
    let screenshot_path = config.output_dir.join(format!("{combo_name}.png"));
    std::fs::create_dir_all(&config.output_dir)?;

    // Run the capture command, substituting {{dir}} and {{name}}
    let cmd = capture_cmd
        .replace("{{dir}}", &combo_dir.to_string_lossy())
        .replace("{{name}}", combo_name)
        .replace("{{output}}", &screenshot_path.to_string_lossy());

    let status = std::process::Command::new("sh")
        .args(["-c", &cmd])
        .status()
        .context("Failed to run screenshot capture command")?;

    if !status.success() {
        bail!(
            "Screenshot capture failed for {combo_name}: exit code {:?}",
            status.code()
        );
    }

    Ok(screenshot_path)
}

/// Compare a screenshot against its baseline using pixel comparison.
///
/// Returns None if no baseline exists (first run -- baseline will be created).
pub fn compare_screenshot(
    config: &VisualRegressionConfig,
    combo_name: &str,
    screenshot_path: &Path,
) -> Result<Option<ComparisonResult>> {
    let baseline_path = config.baseline_dir.join(format!("{combo_name}.png"));

    if !baseline_path.exists() {
        // No baseline -- copy current as new baseline
        std::fs::create_dir_all(&config.baseline_dir)?;
        std::fs::copy(screenshot_path, &baseline_path)?;
        return Ok(None); // First run, no comparison
    }

    // Read both images as raw bytes and compare
    let baseline_bytes = std::fs::read(&baseline_path)?;
    let screenshot_bytes = std::fs::read(screenshot_path)?;

    let (diff_ratio, diff_path) = if baseline_bytes == screenshot_bytes {
        (0.0, None)
    } else {
        // Byte-level diff as a rough proxy for pixel diff
        let max_len = baseline_bytes.len().max(screenshot_bytes.len());
        let min_len = baseline_bytes.len().min(screenshot_bytes.len());
        let mut different = (max_len - min_len) as f64;
        for i in 0..min_len {
            if baseline_bytes[i] != screenshot_bytes[i] {
                different += 1.0;
            }
        }
        let ratio = different / max_len as f64;

        // Save diff info
        std::fs::create_dir_all(&config.diff_dir)?;
        let dp = config.diff_dir.join(format!("{combo_name}.diff.txt"));
        std::fs::write(
            &dp,
            format!(
                "Baseline: {}\nScreenshot: {}\nByte diff ratio: {:.4}\nBaseline size: {} bytes\nScreenshot size: {} bytes\n",
                baseline_path.display(),
                screenshot_path.display(),
                ratio,
                baseline_bytes.len(),
                screenshot_bytes.len()
            ),
        )?;

        (ratio, Some(dp))
    };

    Ok(Some(ComparisonResult {
        combo_name: combo_name.to_string(),
        screenshot_path: screenshot_path.to_path_buf(),
        baseline_path,
        diff_path,
        pixel_diff_ratio: diff_ratio,
        passed: diff_ratio <= config.threshold,
    }))
}

/// Update baselines from current screenshots.
pub fn update_baselines(config: &VisualRegressionConfig, combo_names: &[String]) -> Result<u32> {
    std::fs::create_dir_all(&config.baseline_dir)?;
    let mut updated = 0;
    for name in combo_names {
        let screenshot = config.output_dir.join(format!("{name}.png"));
        let baseline = config.baseline_dir.join(format!("{name}.png"));
        if screenshot.exists() {
            std::fs::copy(&screenshot, &baseline)?;
            updated += 1;
        }
    }
    Ok(updated)
}

/// Generate a summary report of all visual regression results.
pub fn summary_report(results: &[ComparisonResult]) -> String {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    let mut report = format!("Visual Regression: {passed}/{total} passed");
    if failed > 0 {
        report.push_str(&format!(", {failed} FAILED:"));
        for r in results.iter().filter(|r| !r.passed) {
            report.push_str(&format!(
                "\n  {} — diff ratio: {:.4} (threshold: exceeded)",
                r.combo_name, r.pixel_diff_ratio
            ));
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_values() {
        let cfg = VisualRegressionConfig::default();
        assert_eq!(cfg.baseline_dir, PathBuf::from("matrix/.baselines"));
        assert_eq!(cfg.output_dir, PathBuf::from("matrix/.screenshots"));
        assert_eq!(cfg.diff_dir, PathBuf::from("matrix/.diffs"));
        assert!((cfg.threshold - 0.01).abs() < f64::EPSILON);
        assert!(cfg.capture_command.is_none());
    }

    #[test]
    fn summary_report_all_passed() {
        let results = vec![
            ComparisonResult {
                combo_name: "a-b".to_string(),
                screenshot_path: PathBuf::from("s.png"),
                baseline_path: PathBuf::from("b.png"),
                diff_path: None,
                pixel_diff_ratio: 0.0,
                passed: true,
            },
            ComparisonResult {
                combo_name: "c".to_string(),
                screenshot_path: PathBuf::from("s2.png"),
                baseline_path: PathBuf::from("b2.png"),
                diff_path: None,
                pixel_diff_ratio: 0.005,
                passed: true,
            },
        ];
        let report = summary_report(&results);
        assert_eq!(report, "Visual Regression: 2/2 passed");
    }

    #[test]
    fn summary_report_with_failures() {
        let results = vec![
            ComparisonResult {
                combo_name: "good".to_string(),
                screenshot_path: PathBuf::from("s.png"),
                baseline_path: PathBuf::from("b.png"),
                diff_path: None,
                pixel_diff_ratio: 0.0,
                passed: true,
            },
            ComparisonResult {
                combo_name: "bad".to_string(),
                screenshot_path: PathBuf::from("s2.png"),
                baseline_path: PathBuf::from("b2.png"),
                diff_path: Some(PathBuf::from("d.txt")),
                pixel_diff_ratio: 0.15,
                passed: false,
            },
        ];
        let report = summary_report(&results);
        assert!(report.contains("1/2 passed"));
        assert!(report.contains("1 FAILED"));
        assert!(report.contains("bad"));
        assert!(report.contains("0.1500"));
    }

    #[test]
    fn compare_screenshot_no_baseline_creates_one() {
        let dir = tempfile::tempdir().unwrap();
        let config = VisualRegressionConfig {
            baseline_dir: dir.path().join("baselines"),
            output_dir: dir.path().join("screenshots"),
            diff_dir: dir.path().join("diffs"),
            threshold: 0.01,
            capture_command: None,
        };

        // Create a fake screenshot
        std::fs::create_dir_all(&config.output_dir).unwrap();
        let screenshot = config.output_dir.join("combo-a.png");
        std::fs::write(&screenshot, b"fake png data").unwrap();

        let result = compare_screenshot(&config, "combo-a", &screenshot).unwrap();
        assert!(result.is_none(), "First run should return None");

        // Baseline should now exist
        let baseline = config.baseline_dir.join("combo-a.png");
        assert!(baseline.exists());
        assert_eq!(std::fs::read(&baseline).unwrap(), b"fake png data");
    }

    #[test]
    fn compare_screenshot_identical_passes() {
        let dir = tempfile::tempdir().unwrap();
        let config = VisualRegressionConfig {
            baseline_dir: dir.path().join("baselines"),
            output_dir: dir.path().join("screenshots"),
            diff_dir: dir.path().join("diffs"),
            threshold: 0.01,
            capture_command: None,
        };

        // Create baseline and matching screenshot
        std::fs::create_dir_all(&config.baseline_dir).unwrap();
        std::fs::create_dir_all(&config.output_dir).unwrap();
        std::fs::write(config.baseline_dir.join("combo-a.png"), b"identical").unwrap();
        let screenshot = config.output_dir.join("combo-a.png");
        std::fs::write(&screenshot, b"identical").unwrap();

        let result = compare_screenshot(&config, "combo-a", &screenshot)
            .unwrap()
            .unwrap();
        assert!(result.passed);
        assert!((result.pixel_diff_ratio - 0.0).abs() < f64::EPSILON);
        assert!(result.diff_path.is_none());
    }

    #[test]
    fn compare_screenshot_different_fails() {
        let dir = tempfile::tempdir().unwrap();
        let config = VisualRegressionConfig {
            baseline_dir: dir.path().join("baselines"),
            output_dir: dir.path().join("screenshots"),
            diff_dir: dir.path().join("diffs"),
            threshold: 0.01,
            capture_command: None,
        };

        std::fs::create_dir_all(&config.baseline_dir).unwrap();
        std::fs::create_dir_all(&config.output_dir).unwrap();
        std::fs::write(config.baseline_dir.join("combo-a.png"), b"aaaa").unwrap();
        let screenshot = config.output_dir.join("combo-a.png");
        std::fs::write(&screenshot, b"bbbb").unwrap();

        let result = compare_screenshot(&config, "combo-a", &screenshot)
            .unwrap()
            .unwrap();
        assert!(!result.passed);
        assert!(result.pixel_diff_ratio > 0.01);
        assert!(result.diff_path.is_some());
    }

    #[test]
    fn update_baselines_copies_existing_screenshots() {
        let dir = tempfile::tempdir().unwrap();
        let config = VisualRegressionConfig {
            baseline_dir: dir.path().join("baselines"),
            output_dir: dir.path().join("screenshots"),
            diff_dir: dir.path().join("diffs"),
            threshold: 0.01,
            capture_command: None,
        };

        std::fs::create_dir_all(&config.output_dir).unwrap();
        std::fs::write(config.output_dir.join("a.png"), b"screenshot-a").unwrap();
        std::fs::write(config.output_dir.join("b.png"), b"screenshot-b").unwrap();

        let names = vec!["a".to_string(), "b".to_string(), "missing".to_string()];
        let updated = update_baselines(&config, &names).unwrap();
        assert_eq!(updated, 2);
        assert_eq!(
            std::fs::read(config.baseline_dir.join("a.png")).unwrap(),
            b"screenshot-a"
        );
        assert_eq!(
            std::fs::read(config.baseline_dir.join("b.png")).unwrap(),
            b"screenshot-b"
        );
    }
}
