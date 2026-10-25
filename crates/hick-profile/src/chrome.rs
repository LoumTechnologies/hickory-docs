use super::*;
pub(super) fn parse(v: &Value) -> Result<Vec<Thread>> {
    let nodes = v["nodes"].as_array().context("Missing V8 nodes")?;
    ensure!(nodes.len() <= 200_000, "Too many V8 nodes");
    let mut frames = BTreeMap::new();
    let mut parents = BTreeMap::new();
    for node in nodes {
        let id = number(&node["id"])?;
        let call = &node["callFrame"];
        let zero_line = call["lineNumber"].as_i64().unwrap_or(-1);
        let zero_col = call["columnNumber"].as_i64().unwrap_or(-1);
        ensure!(!frames.contains_key(&id), "Duplicate V8 node");
        frames.insert(
            id,
            Frame {
                name: call["functionName"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("(anonymous)")
                    .into(),
                file: call["url"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(str::to_string),
                line: (zero_line >= 0)
                    .then(|| u32::try_from(zero_line + 1).ok())
                    .flatten(),
                column: (zero_col >= 0)
                    .then(|| u32::try_from(zero_col + 1).ok())
                    .flatten(),
                script_id: call["scriptId"].as_str().map(str::to_string),
            },
        );
        if let Some(children) = node["children"].as_array() {
            for child in children {
                let child = number(child)?;
                ensure!(
                    parents.insert(child, id).is_none(),
                    "V8 node has multiple parents"
                );
            }
        }
    }
    let start = finite(&v["startTime"])? * 1e-6;
    let end = finite(&v["endTime"])? * 1e-6;
    ensure!(end >= start, "V8 interval runs backwards");
    let samples = v["samples"]
        .as_array()
        .context("V8 profile has no sample sequence")?;
    let deltas = v["timeDeltas"].as_array();
    if let Some(d) = deltas {
        ensure!(d.len() == samples.len(), "Sample/delta lengths differ");
    }
    let mut rows = BTreeMap::new();
    let mut weight = 0.;
    for (index, sample) in samples.iter().enumerate() {
        let mut id = number(sample)?;
        let mut stack = Vec::new();
        let mut seen = BTreeSet::new();
        loop {
            ensure!(seen.insert(id), "Cyclic V8 sample ancestry");
            stack.push(frames.get(&id).context("Unknown sampled V8 node")?.clone());
            if let Some(parent) = parents.get(&id) {
                id = *parent;
            } else {
                break;
            }
        }
        stack.reverse();
        let delta = match deltas {
            Some(d) => finite(&d[index])? * 1e-6,
            None => 1.,
        };
        weight += delta;
        add(&mut rows, &stack, delta, true);
    }
    Ok(vec![Thread {
        name: "V8 isolate CPU samples".into(),
        unit: if deltas.is_some() {
            "seconds"
        } else {
            "samples"
        }
        .into(),
        aggregation: "sampled stack weights; function locations are anchors, not line timings"
            .into(),
        interval_unit: "seconds".into(),
        clock: "V8 monotonic".into(),
        interval_start: start,
        interval_end: end,
        measured_weight: weight,
        rows: rows.into_values().collect(),
    }])
}
