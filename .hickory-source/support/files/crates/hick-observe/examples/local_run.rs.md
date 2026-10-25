<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-observe/examples/local_run.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Runtime observations retain their producing revision and occurrence](../../../../../guarantees/execution/revision-bound-runtime-observations/contract.md)

Captured SHA-256: `387359baf695edd868aab356aab55e45f4f0c652666a6401cc5a325b7e8ca1eb`.

<book:entry path="crates/hick-observe/examples/local_run.rs">
//! cargo run -p hick-observe --example local_run
//! A deterministic provider harness; no language instrumentation is implied.
use anyhow::Result;
use hick_observe::*;
struct Fixture(std::collections::VecDeque<Event>);
impl ObservationProvider for Fixture {
    fn id(&self) -> &str {
        "fixture"
    }
    fn poll(&mut self) -> Result<Option<Event>> {
        Ok(self.0.pop_front())
    }
}
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
</book:entry>

</book:doc>
