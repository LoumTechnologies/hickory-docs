<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/failed_run.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1fc98c095f08dd780e28cec471f6ae9f791c59a455000c7e625550b00189ea69`.

<book:entry path="crates/hickory-cli/src/failed_run.rs">
//! Keep runtime evidence when execution stops before the final weave.
use super::*;

#[derive(Debug)]
pub(crate) struct FailedRun {
    pub blocks: Vec<Block>,
    pub error: anyhow::Error,
}

impl std::fmt::Display for FailedRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.error, f)
    }
}

impl std::error::Error for FailedRun {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

#[derive(Clone, Default)]
pub(crate) struct Evidence(Arc<std::sync::Mutex<hickory_executor::Transcripts>>);

impl Evidence {
    pub fn hook(&self) -> hick_literate::ExecEventHook {
        let evidence = self.clone();
        Arc::new(move |container, line, entry| {
            let mut entry = entry.clone();
            entry.source_line = Some(line);
            evidence
                .0
                .lock()
                .unwrap()
                .entry(container.to_owned())
                .or_default()
                .push(entry);
        })
    }

    pub fn failure(&self, doc: &hick_lang::HickDocument, error: anyhow::Error) -> anyhow::Error {
        let transcripts = self.0.lock().unwrap();
        if transcripts.is_empty() {
            return error;
        }
        FailedRun {
            blocks: build_block_model(&BlockModelInput {
                doc,
                transcripts: &transcripts,
                expectations: &[],
                files: None,
                never_run: &Default::default(),
                stale: &Default::default(),
            }),
            error,
        }
        .into()
    }
}
</book:entry>

</book:doc>
