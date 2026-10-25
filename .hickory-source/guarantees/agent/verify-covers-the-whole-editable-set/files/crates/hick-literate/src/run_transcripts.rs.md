<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/src/run_transcripts.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Verify Covers the Whole Editable Set](../../../../contract.md)

Captured SHA-256: `3712a56432e065200f8799fc777650997f35fef8e43d4170af6df3f8ff9f2e69`.

<book:entry path="crates/hick-literate/src/run_transcripts.rs">
//! A reused executor's prior runs are not this pipeline's runtime evidence.
use hickory_executor::{Executor, Transcripts};
use std::collections::HashMap;

pub(crate) struct Scope {
    previous: HashMap<String, usize>,
}
impl Scope {
    pub fn capture(executor: &dyn Executor) -> Self {
        Self {
            previous: executor
                .transcripts()
                .into_iter()
                .map(|(name, entries)| (name, entries.len()))
                .collect(),
        }
    }

    /// Executor transcripts append for the lifetime of a run. Like execution
    /// itself, this requires a single caller per executor; it does not isolate
    /// concurrent pipelines or reset container files/process state.
    pub fn current_lines(
        &self,
        executor: &dyn Executor,
        lines: &HashMap<String, Vec<usize>>,
    ) -> Transcripts {
        let mut transcripts = executor.transcripts();
        for (name, entries) in &mut transcripts {
            let old = self
                .previous
                .get(name)
                .copied()
                .unwrap_or(0)
                .min(entries.len());
            entries.drain(..old);
            if let Some(lines) = lines.get(name) {
                for (entry, line) in entries.iter_mut().zip(lines) {
                    entry.source_line = Some(*line);
                }
            }
        }
        transcripts.retain(|_, entries| !entries.is_empty());
        transcripts
    }
}
</book:entry>

</book:doc>
