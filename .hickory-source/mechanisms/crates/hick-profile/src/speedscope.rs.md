<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-profile/src/speedscope.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-223d38487353"></a>
## `crates/hick-profile/src/speedscope.rs`

<a id="f-223d38487353-parse"></a>
### `parse`

<storycode:copy id="f-223d38487353-parse">
use super::*;
pub(super) fn parse(v: &Value) -> Result<Vec<Thread>> {
    let frames: Vec<_> = v["shared"]["frames"]
        .as_array()
        .context("Missing Speedscope frames")?
        .iter()
        .map(frame)
        .collect();
    let mut scopes = Vec::new();
    for p in v["profiles"]
        .as_array()
        .context("Missing Speedscope profiles")?
    {
        let (unit, multiplier) = unit(p["unit"].as_str().context("Missing profile unit")?)?;
        let start = finite(&p["startValue"])?;
        let end = finite(&p["endValue"])?;
        ensure!(end >= start, "Profile interval runs backwards");
        let mut rows = BTreeMap::new();
        let mut measured = 0.;
        match p["type"].as_str() {
            Some("sampled") => {
                let samples = p["samples"].as_array().context("Missing samples")?;
                let weights = p["weights"].as_array();
                if let Some(w) = weights {
                    ensure!(w.len() == samples.len(), "Sample/weight lengths differ");
                } else {
                    ensure!(
                        unit == "samples",
                        "Time/byte profile requires explicit weights"
                    );
                }
                for (index, sample) in samples.iter().enumerate() {
                    let stack: Result<Vec<_>> = sample
                        .as_array()
                        .context("Invalid sampled stack")?
                        .iter()
                        .map(|i| {
                            frames
                                .get(number(i)?)
                                .cloned()
                                .context("Unknown Speedscope frame")
                        })
                        .collect();
                    let weight = if let Some(w) = weights {
                        finite(&w[index])?
                    } else {
                        1.
                    } * multiplier;
                    measured += weight;
                    add(&mut rows, &stack?, weight, true);
                }
            }
            Some("evented") => {
                let mut stack = Vec::<usize>::new();
                let mut previous = start;
                for e in p["events"].as_array().context("Missing profile events")? {
                    let at = finite(&e["at"])?;
                    ensure!(
                        at >= previous && at <= end,
                        "Event outside ordered profile interval"
                    );
                    let weight = (at - previous) * multiplier;
                    let callstack: Vec<_> = stack.iter().map(|i| frames[*i].clone()).collect();
                    if !stack.is_empty() {
                        measured += weight;
                        add(&mut rows, &callstack, weight, false);
                    }
                    let index = number(&e["frame"])?;
                    ensure!(index < frames.len(), "Unknown event frame");
                    match e["type"].as_str() {
                        Some("O") => stack.push(index),
                        Some("C") => ensure!(stack.pop() == Some(index), "Mismatched frame close"),
                        _ => bail!("Unknown Speedscope event"),
                    }
                    previous = at;
                }
                ensure!(stack.is_empty(), "Unclosed evented stack");
            }
            _ => bail!("Unsupported Speedscope profile type"),
        }
        scopes.push(Thread {
            name: p["name"].as_str().unwrap_or("unnamed scope").into(),
            unit: unit.into(),
            aggregation: "per-profile stack weights; inclusive costs overlap".into(),
            interval_unit: unit.into(),
            clock: "provider-relative".into(),
            interval_start: start * multiplier,
            interval_end: end * multiplier,
            measured_weight: measured,
            rows: rows.into_values().collect(),
        });
    }
    Ok(scopes)
}
</storycode:copy>

### Assemble `crates/hick-profile/src/speedscope.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-profile/src/speedscope.rs"><storycode:paste select="#f-223d38487353-parse"/></storycode:file>

</storycode:doc>
