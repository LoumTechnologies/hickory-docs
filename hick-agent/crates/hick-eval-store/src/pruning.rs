//! Pruning analytics: config ranking, Wilson CI, is_pruned predicate.
//!
//! None of these functions write to the store — they are read-only queries
//! over a slice of [`EvalRecord`]s loaded by the caller.

use std::collections::HashMap;

use crate::record::EvalRecord;

/// Per-config aggregated stats used by the pruning view.
#[derive(Debug, Clone)]
pub struct ConfigStats {
    pub config_name: String,
    /// Weighted mean score across all scored evaluations (1–10 scale).
    pub combined_score: f64,
    /// Number of scored evaluations (judge or human).
    pub sample_count: usize,
    /// Proportion of evaluations that scored above 5.0 (binary win).
    pub win_rate: f64,
    /// Wilson score CI lower bound on `win_rate` (95%).
    pub ci_low: f64,
    /// Wilson score CI upper bound on `win_rate` (95%).
    pub ci_high: f64,
    /// Derived from replaying prune/unprune log entries.
    pub is_pruned: bool,
    /// `true` when the config is a candidate for pruning:
    /// not already pruned, enough samples, and CI upper bound is
    /// below the best config's win rate.
    pub is_candidate: bool,
}

// Z-value for a one-sided 95% confidence interval.
const Z_95: f64 = 1.645;

/// Wilson score confidence interval for a proportion.
///
/// Returns `(lower, upper)` in [0, 1]. Handles n = 0 by returning (0, 1).
pub fn wilson_ci(wins: usize, n: usize) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let p = wins as f64 / n as f64;
    let z = Z_95;
    let z2 = z * z;
    let nf = n as f64;
    let center = (p + z2 / (2.0 * nf)) / (1.0 + z2 / nf);
    let margin = z * (p * (1.0 - p) / nf + z2 / (4.0 * nf * nf)).sqrt() / (1.0 + z2 / nf);
    ((center - margin).max(0.0), (center + margin).min(1.0))
}

fn mean_of_map(scores: &HashMap<String, f64>) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    scores.values().sum::<f64>() / scores.len() as f64
}

/// Compute per-config stats by replaying the full record log.
///
/// Human ratings override judge scores for the same (config, query) pair.
/// Configs with zero scored evaluations are omitted from the result.
/// Result is sorted by `combined_score` descending (best first).
pub fn compute_config_stats(records: &[EvalRecord]) -> Vec<ConfigStats> {
    // Replay prune/unprune events to derive current pruning state.
    let mut prune_state: HashMap<String, bool> = HashMap::new();

    // Per config → per query: (judge_score?, human_score?).
    let mut per_config: HashMap<String, HashMap<String, (Option<f64>, Option<f64>)>> =
        HashMap::new();

    for record in records {
        match record {
            EvalRecord::JudgeScore(r) => {
                let score = mean_of_map(&r.scores);
                let entry = per_config
                    .entry(r.config_name.clone())
                    .or_default()
                    .entry(r.query.clone())
                    .or_insert((None, None));
                entry.0 = Some(score);
            }
            EvalRecord::HumanRating(r) => {
                let score = mean_of_map(&r.scores);
                let entry = per_config
                    .entry(r.config_name.clone())
                    .or_default()
                    .entry(r.query.clone())
                    .or_insert((None, None));
                entry.1 = Some(score);
            }
            EvalRecord::Prune(r) => {
                if let Some(ref cn) = r.config_name {
                    prune_state.insert(cn.clone(), true);
                }
            }
            EvalRecord::Unprune(r) => {
                prune_state.insert(r.config_name.clone(), false);
            }
            _ => {}
        }
    }

    let mut stats: Vec<ConfigStats> = per_config
        .into_iter()
        .map(|(config_name, query_scores)| {
            // Use human rating where available; fall back to judge score.
            let scores: Vec<f64> = query_scores
                .values()
                .filter_map(|(judge, human)| human.or(*judge))
                .filter(|&s| s > 0.0)
                .collect();
            let n = scores.len();
            let combined_score = if n > 0 {
                scores.iter().sum::<f64>() / n as f64
            } else {
                0.0
            };
            let wins = scores.iter().filter(|&&s| s > 5.0).count();
            let (ci_low, ci_high) = wilson_ci(wins, n);
            let win_rate = if n > 0 { wins as f64 / n as f64 } else { 0.0 };
            let pruned = *prune_state.get(&config_name).unwrap_or(&false);
            ConfigStats {
                config_name,
                combined_score,
                sample_count: n,
                win_rate,
                ci_low,
                ci_high,
                is_pruned: pruned,
                is_candidate: false, // set below
            }
        })
        .collect();

    // Rank by combined score to find the best.
    let best_win_rate = stats
        .iter()
        .filter(|s| !s.is_pruned)
        .map(|s| s.win_rate)
        .fold(0.0_f64, f64::max);

    // Flag candidates: not pruned, at least 3 samples, CI upper bound < best win rate.
    const MIN_SAMPLES: usize = 3;
    for s in &mut stats {
        s.is_candidate =
            !s.is_pruned && s.sample_count >= MIN_SAMPLES && s.ci_high < best_win_rate;
    }

    stats.sort_by(|a, b| {
        b.combined_score
            .partial_cmp(&a.combined_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    stats
}

/// Derive whether `config_name` is currently pruned by replaying the log.
///
/// Call sites that need to check pruning without the full stats computation
/// (e.g., TASK-50 idle eval scheduler) should use this.
pub fn is_pruned(config_name: &str, records: &[EvalRecord]) -> bool {
    let mut pruned = false;
    for record in records {
        match record {
            EvalRecord::Prune(r) if r.config_name.as_deref() == Some(config_name) => {
                pruned = true;
            }
            EvalRecord::Unprune(r) if r.config_name == config_name => {
                pruned = false;
            }
            _ => {}
        }
    }
    pruned
}

/// Return the names of configs that should be auto-pruned at `confidence_threshold`.
///
/// A config is auto-prunable when:
/// - It is not already pruned.
/// - It has at least 3 scored evaluations.
/// - Its Wilson CI upper bound (`ci_high`) at 95% is below
///   `best_win_rate * (1 - (1 - confidence_threshold))`.
///
/// The caller (TASK-50 idle eval) should pass a `confidence_threshold` from
/// `[eval] auto_prune_threshold` in `agent.toml` (e.g. `0.95`).
pub fn auto_prune_candidates(records: &[EvalRecord], confidence_threshold: f64) -> Vec<String> {
    let stats = compute_config_stats(records);
    let best_win_rate = stats
        .iter()
        .filter(|s| !s.is_pruned)
        .map(|s| s.win_rate)
        .fold(0.0_f64, f64::max);

    stats
        .into_iter()
        .filter(|s| {
            !s.is_pruned
                && s.sample_count >= 3
                && best_win_rate > 0.0
                && s.ci_high < best_win_rate * confidence_threshold
        })
        .map(|s| s.config_name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{
        EvalRecord, HumanRatingRecord, JudgeScoreRecord, PruneRecord, UnpruneRecord,
    };
    use chrono::Utc;

    fn judge(config: &str, query: &str, score: f64) -> EvalRecord {
        let mut scores = HashMap::new();
        scores.insert("task_completion".to_string(), score);
        EvalRecord::JudgeScore(JudgeScoreRecord {
            machine_id: "m".to_string(),
            timestamp: Utc::now(),
            config_name: config.to_string(),
            judge_model: "model".to_string(),
            prompt_hash: "hash".to_string(),
            query: query.to_string(),
            scores,
            backtracking_events: vec![],
            token_cost: None,
            wall_secs: None,
            rework_count: None,
        })
    }

    fn human(config: &str, query: &str, score: f64) -> EvalRecord {
        let mut scores = HashMap::new();
        scores.insert("task_completion".to_string(), score);
        EvalRecord::HumanRating(HumanRatingRecord {
            machine_id: "m".to_string(),
            timestamp: Utc::now(),
            config_name: config.to_string(),
            query: query.to_string(),
            scores,
            judge_timestamp: None,
        })
    }

    fn prune(config: &str) -> EvalRecord {
        EvalRecord::Prune(PruneRecord {
            machine_id: "m".to_string(),
            timestamp: Utc::now(),
            config_name: Some(config.to_string()),
            reason: None,
            pruned_before: None,
            scores: None,
            confidence: None,
            trigger: Some("manual".to_string()),
        })
    }

    fn unprune(config: &str) -> EvalRecord {
        EvalRecord::Unprune(UnpruneRecord {
            machine_id: "m".to_string(),
            timestamp: Utc::now(),
            config_name: config.to_string(),
            reason: None,
        })
    }

    #[test]
    fn wilson_ci_all_wins() {
        let (lo, hi) = wilson_ci(10, 10);
        assert!(lo > 0.6, "lower bound should be well above 0 for all wins: {lo}");
        assert!(hi <= 1.0);
    }

    #[test]
    fn wilson_ci_no_data() {
        let (lo, hi) = wilson_ci(0, 0);
        assert_eq!(lo, 0.0);
        assert_eq!(hi, 1.0);
    }

    #[test]
    fn wilson_ci_half_wins() {
        let (lo, hi) = wilson_ci(5, 10);
        assert!(lo < 0.5, "lower bound < 0.5 with 5/10 wins: {lo}");
        assert!(hi > 0.5, "upper bound > 0.5 with 5/10 wins: {hi}");
    }

    #[test]
    fn is_pruned_after_prune_record() {
        let records = vec![
            judge("cfg-a", "q1", 8.0),
            prune("cfg-a"),
        ];
        assert!(is_pruned("cfg-a", &records));
        assert!(!is_pruned("cfg-b", &records));
    }

    #[test]
    fn is_pruned_false_after_unprune() {
        let records = vec![
            judge("cfg-a", "q1", 8.0),
            prune("cfg-a"),
            unprune("cfg-a"),
        ];
        assert!(!is_pruned("cfg-a", &records));
    }

    #[test]
    fn compute_stats_human_overrides_judge() {
        // Judge says 3.0, human says 9.0 — combined should be 9.0.
        let records = vec![
            judge("cfg-a", "q1", 3.0),
            human("cfg-a", "q1", 9.0),
        ];
        let stats = compute_config_stats(&records);
        assert_eq!(stats.len(), 1);
        assert!((stats[0].combined_score - 9.0).abs() < 0.01);
    }

    #[test]
    fn compute_stats_sorts_by_score_descending() {
        let records = vec![
            judge("cfg-a", "q1", 5.0),
            judge("cfg-b", "q1", 9.0),
        ];
        let stats = compute_config_stats(&records);
        assert_eq!(stats[0].config_name, "cfg-b");
        assert_eq!(stats[1].config_name, "cfg-a");
    }

    #[test]
    fn compute_stats_pruned_flag_from_log() {
        let records = vec![
            judge("cfg-a", "q1", 8.0),
            judge("cfg-b", "q1", 5.0),
            prune("cfg-b"),
        ];
        let stats = compute_config_stats(&records);
        let a = stats.iter().find(|s| s.config_name == "cfg-a").unwrap();
        let b = stats.iter().find(|s| s.config_name == "cfg-b").unwrap();
        assert!(!a.is_pruned);
        assert!(b.is_pruned);
    }

    #[test]
    fn compute_stats_candidate_requires_min_samples() {
        // Only 2 samples for cfg-b — should not be a candidate even if score is low.
        let records = vec![
            judge("cfg-a", "q1", 9.0),
            judge("cfg-a", "q2", 9.0),
            judge("cfg-a", "q3", 9.0),
            judge("cfg-b", "q1", 1.0),
            judge("cfg-b", "q2", 1.0),
        ];
        let stats = compute_config_stats(&records);
        let b = stats.iter().find(|s| s.config_name == "cfg-b").unwrap();
        assert!(!b.is_candidate, "need >= 3 samples to be a candidate");
    }

    #[test]
    fn compute_stats_candidate_flagged_when_clearly_worse() {
        // cfg-b scores 1.0 every query; cfg-a scores 9.0. With enough samples cfg-b should
        // be flagged.
        let records = vec![
            judge("cfg-a", "q1", 9.0),
            judge("cfg-a", "q2", 9.0),
            judge("cfg-a", "q3", 9.0),
            judge("cfg-b", "q1", 1.0),
            judge("cfg-b", "q2", 1.0),
            judge("cfg-b", "q3", 1.0),
        ];
        let stats = compute_config_stats(&records);
        let b = stats.iter().find(|s| s.config_name == "cfg-b").unwrap();
        // cfg-b wins=0/3, ci_high should be well below cfg-a win_rate=1.0.
        assert!(b.is_candidate, "cfg-b should be a pruning candidate");
    }

    #[test]
    fn auto_prune_candidates_returns_losers() {
        let records = vec![
            judge("cfg-a", "q1", 9.0),
            judge("cfg-a", "q2", 9.0),
            judge("cfg-a", "q3", 9.0),
            judge("cfg-b", "q1", 1.0),
            judge("cfg-b", "q2", 1.0),
            judge("cfg-b", "q3", 1.0),
        ];
        let candidates = auto_prune_candidates(&records, 0.95);
        assert!(candidates.contains(&"cfg-b".to_string()));
        assert!(!candidates.contains(&"cfg-a".to_string()));
    }

    #[test]
    fn auto_prune_skips_already_pruned() {
        let records = vec![
            judge("cfg-a", "q1", 9.0),
            judge("cfg-a", "q2", 9.0),
            judge("cfg-a", "q3", 9.0),
            judge("cfg-b", "q1", 1.0),
            judge("cfg-b", "q2", 1.0),
            judge("cfg-b", "q3", 1.0),
            prune("cfg-b"),
        ];
        let candidates = auto_prune_candidates(&records, 0.95);
        assert!(candidates.is_empty(), "cfg-b is already pruned, should not reappear");
    }
}
