<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-profile/src/firefox.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c75582396aa342d9943fc95a35816d31da9d7b5a34d9d0781f46f46c7dee08f2`.

<book:entry path="crates/hick-profile/src/firefox.rs">
//! Symbolicated processed Firefox profiles (including exports of samply captures).
//! Without explicit time weights, retain sample weights rather than estimating CPU time.
use super::*;
pub(super) fn parse(v: &Value) -> Result<Vec<Thread>> {
    let mut scopes = Vec::new();
    for thread in v["threads"]
        .as_array()
        .context("Missing processed Firefox threads")?
    {
        let strings = thread["stringArray"].as_array().context(
            "Expected processed Firefox stringArray; raw Gecko profiles require conversion",
        )?;
        let string = |index: &Value| -> Option<String> {
            strings
                .get(index.as_u64()? as usize)?
                .as_str()
                .map(str::to_string)
        };
        let funcs = &thread["funcTable"];
        let mut frames = Vec::new();
        for function in thread["frameTable"]["func"]
            .as_array()
            .context("Missing Firefox frame functions")?
        {
            let id = number(function)?;
            frames.push(Frame {
                name: string(&funcs["name"][id])
                    .unwrap_or_else(|| "unresolved native symbol".into()),
                file: string(&funcs["fileName"][id]),
                line: funcs["lineNumber"][id]
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .filter(|n| *n > 0),
                column: funcs["columnNumber"][id]
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok()),
                script_id: None,
            });
        }
        let samples = thread["samples"]["stack"]
            .as_array()
            .context("Missing Firefox samples")?;
        let weights = thread["samples"]["weight"].as_array();
        if let Some(weights) = weights {
            ensure!(
                weights.len() == samples.len(),
                "Firefox sample/weight lengths differ"
            );
        }
        let unit = match thread["samples"]["weightType"].as_str() {
            Some("tracing-ms") => "seconds",
            Some("bytes") => "bytes",
            Some("samples") | None => "samples",
            Some(other) => bail!("Unsupported Firefox weight type {other}"),
        };
        ensure!(
            unit == "samples" || weights.is_some(),
            "Explicit Firefox time/byte weights required"
        );
        let mut rows = BTreeMap::new();
        let mut measured = 0.;
        let stacks = &thread["stackTable"];
        for (sample_index, sample) in samples.iter().enumerate() {
            if sample.is_null() {
                continue;
            }
            let mut id = number(sample)?;
            let mut seen = BTreeSet::new();
            let mut stack = Vec::new();
            loop {
                ensure!(seen.insert(id), "Cyclic Firefox stack");
                let frame = number(&stacks["frame"][id])?;
                stack.push(frames.get(frame).context("Unknown Firefox frame")?.clone());
                let parent = &stacks["prefix"][id];
                if parent.is_null() {
                    break;
                }
                id = number(parent)?;
            }
            stack.reverse();
            let mut weight = if let Some(weights) = weights {
                finite(&weights[sample_index])?
            } else {
                1.
            };
            if unit == "seconds" {
                weight *= 1e-3;
            }
            measured += weight;
            add(&mut rows, &stack, weight, true);
        }
        let times = thread["samples"]["time"].as_array();
        let start = times
            .and_then(|a| a.first())
            .and_then(Value::as_f64)
            .unwrap_or(0.)
            * 1e-3;
        let end = times
            .and_then(|a| a.last())
            .and_then(Value::as_f64)
            .unwrap_or(0.)
            * 1e-3;
        ensure!(
            start.is_finite() && end.is_finite() && end >= start,
            "Invalid Firefox timestamp interval"
        );
        scopes.push(Thread {name:format!("{} (tid {})",thread["name"].as_str().unwrap_or("thread"),thread["tid"]),unit:unit.into(),aggregation:"processed Firefox stack weights; sample count is supplied stack observations, not function calls; interval uses profile-relative seconds".into(),interval_unit:if times.is_some(){"seconds"}else{"stack observations"}.into(),clock:if times.is_some(){"profile-relative"}else{"sample-sequence-index"}.into(),interval_start:if times.is_some(){start}else{0.},interval_end:if times.is_some(){end}else{samples.len() as f64},measured_weight:measured,rows:rows.into_values().collect()});
    }
    Ok(scopes)
}
</book:entry>

</book:doc>
