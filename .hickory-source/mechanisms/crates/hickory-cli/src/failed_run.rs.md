<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/failed_run.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3cf003894222"></a>
## `crates/hickory-cli/src/failed_run.rs`

<a id="f-3cf003894222-FailedRun"></a>
### `FailedRun`

Keep runtime evidence when execution stops before the final weave.

<storycode:copy id="f-3cf003894222-FailedRun">
//! Keep runtime evidence when execution stops before the final weave.
use super::*;

#[derive(Debug)]
pub(crate) struct FailedRun {
    pub blocks: Vec<Block>,
    pub error: anyhow::Error,
}

</storycode:copy>
<a id="f-3cf003894222-impl-std-fmt-Display-for-FailedRun"></a>
### `impl-std::fmt::Display for FailedRun`

<storycode:copy id="f-3cf003894222-impl-std-fmt-Display-for-FailedRun">
impl std::fmt::Display for FailedRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.error, f)
    }
}

</storycode:copy>
<a id="f-3cf003894222-impl-std-error-Error-for-FailedRun"></a>
### `impl-std::error::Error for FailedRun`

<storycode:copy id="f-3cf003894222-impl-std-error-Error-for-FailedRun">
impl std::error::Error for FailedRun {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

</storycode:copy>
<a id="f-3cf003894222-Evidence"></a>
### `Evidence`

<storycode:copy id="f-3cf003894222-Evidence">
#[derive(Clone, Default)]
pub(crate) struct Evidence(Arc<std::sync::Mutex<hickory_executor::Transcripts>>);

</storycode:copy>
<a id="f-3cf003894222-impl-Evidence"></a>
### `impl-Evidence`

<storycode:copy id="f-3cf003894222-impl-Evidence">
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/failed_run.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/failed_run.rs"><storycode:paste select="#f-3cf003894222-FailedRun"/><storycode:paste select="#f-3cf003894222-impl-std-fmt-Display-for-FailedRun"/><storycode:paste select="#f-3cf003894222-impl-std-error-Error-for-FailedRun"/><storycode:paste select="#f-3cf003894222-Evidence"/><storycode:paste select="#f-3cf003894222-impl-Evidence"/></storycode:file>

</storycode:doc>
