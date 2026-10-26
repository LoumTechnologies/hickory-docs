<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-observe/examples/local_run.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-490351fb838b"></a>
## `crates/hick-observe/examples/local_run.rs`

Related contracts: [Runtime observations retain their producing revision and occurrence](../../../../guarantees/execution/revision-bound-runtime-observations/contract.md).

<a id="f-490351fb838b-Fixture"></a>
### `Fixture`

cargo run -p hick-observe --example local_run
A deterministic provider harness; no language instrumentation is implied.

<storycode:copy id="f-490351fb838b-Fixture">
//! cargo run -p hick-observe --example local_run
//! A deterministic provider harness; no language instrumentation is implied.
use anyhow::Result;
use hick_observe::*;
struct Fixture(std::collections::VecDeque<Event>);
</storycode:copy>
<a id="f-490351fb838b-impl-ObservationProvider-for-Fixture"></a>
### `impl-ObservationProvider for Fixture`

<storycode:copy id="f-490351fb838b-impl-ObservationProvider-for-Fixture">
impl ObservationProvider for Fixture {
    fn id(&self) -> &str {
        "fixture"
    }
    fn poll(&mut self) -> Result<Option<Event>> {
        Ok(self.0.pop_front())
    }
}
</storycode:copy>
<a id="f-490351fb838b-main"></a>
### `main`

<storycode:copy id="f-490351fb838b-main">
fn main() -> Result<()> {
    let project = tempfile::tempdir()?;
    let store = LocalStore::open(project.path())?;
    let mut run = Run::new(
        vec![Source::capture(
            "source".into(),
            "demo.md".into(),
            b"print('hello')".to_vec(),
        )],
        0,
    );
    store.save(&run)?;
    let mut provider = Fixture(
        [Event {
            id: "write-1".into(),
            provider: "fixture".into(),
            block: Some("cell-1".into()),
            invocation: None,
            parent_invocation: None,
            thread: None,
            task: None,
            time: None,
            emission: Some(Location {
                source: "source".into(),
                block: Some("cell-1".into()),
                bytes: None,
                precision: Precision::BlockOnly,
            }),
            callers: vec![],
            observation: Observation::Output {
                stream: "stdout".into(),
                category: "console".into(),
                bytes: b"hello\n".to_vec(),
            },
        }]
        .into(),
    );
    assert_eq!(run.collect(&mut provider, 1024 * 1024)?, 1);
    run.finish(Status::Completed)?;
    store.save(&run)?;
    let Retrieved::Present(restored) = store.get(&run.id)? else {
        anyhow::bail!("missing run")
    };
    assert_eq!(
        output_lines(&restored, "stdout", "console")[0].bytes,
        b"hello\n"
    );
    assert_eq!(
        restored.events[0].emission.as_ref().unwrap().precision,
        Precision::BlockOnly
    );
    println!("provider -> durable run -> retrieved line: hello (block-only)");
    Ok(())
}
</storycode:copy>

### Assemble `crates/hick-observe/examples/local_run.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-observe/examples/local_run.rs"><storycode:paste select="#f-490351fb838b-Fixture"/><storycode:paste select="#f-490351fb838b-impl-ObservationProvider-for-Fixture"/><storycode:paste select="#f-490351fb838b-main"/></storycode:file>

</storycode:doc>
