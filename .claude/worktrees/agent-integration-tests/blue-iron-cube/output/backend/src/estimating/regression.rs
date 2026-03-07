use anyhow::Result;
use ndarray::{Array1, Array2};
use rusqlite::Connection;

/// Feature names in the order they appear in the feature matrix.
pub const FEATURE_NAMES: &[&str] = &[
    "linear_feet",
    "trench_depth_inches",
    "soil_clay",
    "soil_rocky",
    "soil_sand",
    "soil_loam",
    "grade_slight",
    "grade_moderate",
    "grade_steep",
    "access_moderate",
    "access_difficult",
    "access_hand_carry",
    "sump_pump",
    "downspout_tie_in",
    "catch_basins",
];

/// One-hot encode a categorical value.
fn one_hot(value: &str, categories: &[&str]) -> Vec<f64> {
    categories.iter().map(|c| if *c == value { 1.0 } else { 0.0 }).collect()
}

/// Build the feature matrix from completed jobs.
pub fn build_training_data(conn: &Connection) -> Result<(Array2<f64>, Array1<f64>, i64)> {
    let mut stmt = conn.prepare(
        "SELECT j.linear_feet_actual, j.trench_depth_inches, j.soil_type, j.yard_grade,
                j.site_access, j.includes_sump_pump, j.includes_downspout_tie_in,
                j.number_of_catch_basins,
                COALESCE((SELECT SUM(total_cost) FROM job_line_items WHERE job_id = j.id AND category != 'overhead'), 0)
         FROM jobs j
         WHERE j.status IN ('completed', 'invoiced')
           AND j.linear_feet_actual IS NOT NULL"
    )?;

    let mut features: Vec<Vec<f64>> = Vec::new();
    let mut targets: Vec<f64> = Vec::new();

    let rows = stmt.query_map([], |row| {
        let linear_feet: f64 = row.get(0)?;
        let depth: f64 = row.get(1)?;
        let soil: String = row.get(2)?;
        let grade: String = row.get(3)?;
        let access: String = row.get(4)?;
        let sump: bool = row.get(5)?;
        let downspout: bool = row.get(6)?;
        let basins: i32 = row.get(7)?;
        let direct_cost: f64 = row.get(8)?;

        let mut feat = vec![linear_feet, depth];
        feat.extend(one_hot(&soil, &["clay", "rocky", "sand", "loam"]));
        feat.extend(one_hot(&grade, &["slight", "moderate", "steep"]));
        feat.extend(one_hot(&access, &["moderate", "difficult", "hand_carry_only"]));
        feat.push(if sump { 1.0 } else { 0.0 });
        feat.push(if downspout { 1.0 } else { 0.0 });
        feat.push(basins as f64);

        Ok((feat, direct_cost))
    })?;

    for row in rows {
        let (feat, target) = row?;
        features.push(feat);
        targets.push(target);
    }

    let n = features.len();
    let p = FEATURE_NAMES.len();
    let sample_size = n as i64;

    if n < 3 {
        anyhow::bail!("Need at least 3 completed jobs to train the model, have {}", n);
    }

    let flat: Vec<f64> = features.into_iter().flatten().collect();
    let x = Array2::from_shape_vec((n, p), flat)?;
    let y = Array1::from_vec(targets);

    Ok((x, y, sample_size))
}

/// Train the model and return weights + intercept.
pub fn train(conn: &Connection) -> Result<(Vec<f64>, f64, f64, i64)> {
    use linfa::prelude::*;
    use linfa_linear::LinearRegression;

    let (x, y, sample_size) = build_training_data(conn)?;

    let dataset = linfa::Dataset::new(x, y);
    let model = LinearRegression::default().fit(&dataset)?;

    let weights: Vec<f64> = model.params().iter().copied().collect();
    let intercept = model.intercept();

    // Compute R-squared
    let predictions = model.predict(dataset.records());
    let targets = dataset.targets();
    let ss_res: f64 = predictions.iter().zip(targets.iter())
        .map(|(p, t)| (p - t).powi(2)).sum();
    let mean: f64 = targets.mean().unwrap_or(0.0);
    let ss_tot: f64 = targets.iter().map(|t| (t - mean).powi(2)).sum();
    let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

    Ok((weights, intercept, r_squared, sample_size))
}

/// Predict direct cost for given features.
pub fn predict(weights: &[f64], intercept: f64, features: &[f64]) -> f64 {
    let dot: f64 = weights.iter().zip(features.iter()).map(|(w, f)| w * f).sum();
    (dot + intercept).max(0.0)
}
