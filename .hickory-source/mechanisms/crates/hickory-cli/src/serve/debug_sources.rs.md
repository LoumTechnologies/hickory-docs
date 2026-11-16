<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/debug_sources.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9c1c3fe2ca3c"></a>
## `crates/hickory-cli/src/serve/debug_sources.rs`

<a id="f-9c1c3fe2ca3c-document"></a>
### `document`

<storycode:copy id="f-9c1c3fe2ca3c-document">
use super::*;
use anyhow::Context;
use hick_dap::Breakpoint;
use std::path::Path;

// Validate the client identity before asking the session's exact mapping.
// A captured URI must belong to that session; a workspace path cannot escape
// the open folder through '..' or a symlink. Neither route substitutes bytes.
pub(super) fn document(root: &Path, source: &str) -> anyhow::Result<PathBuf> {
    crate::debug_sessions::source_document(root, source)
}
</storycode:copy>
<a id="debug-breakpoint-source-admission"></a>
### Confirm the source revision before replacing breakpoints

A source hash labels a position intention, not a successful binding. Require
the captured launch identity and current saved bytes before DAP replacement.
Only a successful replacement records the accepted ticket; later promotions
repeat that accepted identity, so delayed replies cannot label a newer draft.

<storycode:copy id="debug-breakpoint-source-admission">
pub(super) fn valid_hash(hash: Option<&str>) -> anyhow::Result<&str> {
    let hash=hash.context("Source revision is missing. Reopen this source with a compatible debugger before setting breakpoints.")?;
    anyhow::ensure!(
        hash.len() == 64 && hash.bytes().all(|c| c.is_ascii_hexdigit()),
        "Source revision is malformed. Reload the source and retry Debug."
    );
    Ok(hash)
}

pub(super) fn validate(
    live: &crate::debug_sessions::Live,
    document: &Path,
    expected: Option<&str>,
    lines: &[u32],
) -> anyhow::Result<String> {
    let expected = valid_hash(expected)?;
    anyhow::ensure!(
        live.session.source_hash(document).as_deref() == Some(expected),
        "Source differs from the captured launch project. Reopen its captured source or stop and launch again before setting breakpoints."
    );
    if !document.to_string_lossy().starts_with("hick-project:///") {
        let bytes = std::fs::read(document)?;
        let line_count = bytes.iter().filter(|byte| **byte == b'\n').count() + 1;
        anyhow::ensure!(
            lines.iter().all(|line| (*line as usize) < line_count),
            "Breakpoint or movement position is outside its captured source. Refresh this source and its controls."
        );
        anyhow::ensure!(
            super::super::source_tests::digest(&bytes) == expected,
            "Saved source changed since launch. Restore its launch text or start a new debugger before setting breakpoints."
        );
    }
    Ok(expected.to_owned())
}

pub(super) fn validate_primary(
    live: &crate::debug_sessions::Live,
    expected: Option<&str>,
    lines: &[u32],
) -> anyhow::Result<String> {
    validate(live, live.session.source_document(), expected, lines)
}

pub(super) fn primary_status(session: &str, live: &crate::debug_sessions::Live) -> Response {
    let intent = live.breakpoint_intents.lock().unwrap().get("").cloned();
    Response::Breakpoints {
        session: session.to_owned(),
        breakpoints: live.session.breakpoint_statuses(),
        request_id: intent.map(|value| value.request_id),
        source_hash: live.session.primary_source_hash(),
    }
}
</storycode:copy>

<a id="f-9c1c3fe2ca3c-initial"></a>
### `initial`

<storycode:copy id="f-9c1c3fe2ca3c-initial">
pub(super) fn initial(
    root: &Path,
    sources: std::collections::HashMap<String, Vec<Breakpoint>>,
    hashes: std::collections::HashMap<String, String>,
) -> anyhow::Result<Vec<hick_dap::SourceBreakpoints>> {
    anyhow::ensure!(
        sources.len() == hashes.len() && sources.keys().all(|key| hashes.contains_key(key)),
        "Breakpoint source revisions do not match the requested source set. Prepare Debug again."
    );
    sources
        .into_iter()
        .map(|(source, breakpoints)| {
            Ok(hick_dap::SourceBreakpoints {
                document: document(root, &source)?,
                source_hash: Some(valid_hash(hashes.get(&source).map(String::as_str))?.to_owned()),
                source,
                breakpoints,
            })
        })
        .collect()
}
</storycode:copy>
<a id="f-9c1c3fe2ca3c-set"></a>
### `set`

<storycode:copy id="f-9c1c3fe2ca3c-set">
pub(super) async fn set(
    registry: &Arc<Registry>,
    root: &Path,
    session: String,
    source: String,
    breakpoints: Vec<Breakpoint>,
    source_hash: Option<String>,
    request_id: Option<String>,
) -> Response {
    let result = async {
        let ticket = intent_ticket(request_id.as_deref())?.to_owned();
        let live = registry.get(&session).await?;
        let document = document(root, &source)?;
        let lines = breakpoints
            .iter()
            .map(|value| value.line)
            .collect::<Vec<_>>();
        let hash = validate(&live, &document, source_hash.as_deref(), &lines)?;
        let update = IntentUpdate::begin(&live.breakpoint_intents);
        let statuses = live
            .session
            .set_source_breakpoints(&source, &document, &breakpoints)
            .await?;
        update.accept(
            source.clone(),
            crate::debug_sessions::BreakpointIntent {
                request_id: ticket,
                source_hash: hash.clone(),
            },
        );
        Ok::<_, anyhow::Error>((statuses, hash))
    }
    .await;
    match result {
        Ok((breakpoints, hash)) => Response::SourceBreakpoints {
            session,
            source,
            breakpoints,
            source_hash: Some(hash),
            request_id,
        },
        Err(error) => Response::Failed {
            scope_retired: false,
            request_id,
            session: Some(session),
            doc: Some(source),
            message: format!("{error:#}"),
            about: Some("source_breakpoints".into()),
            lines: breakpoints.iter().map(|bp| bp.line).collect(),
            offer_install: None,
        },
    }
}
</storycode:copy>
<a id="f-9c1c3fe2ca3c-statuses"></a>
### `statuses`

<storycode:copy id="f-9c1c3fe2ca3c-statuses">
pub(super) fn statuses(session: &str, live: &Arc<crate::debug_sessions::Live>) -> Vec<Response> {
    let hashes = live.session.source_breakpoint_hashes();
    let intents = live.breakpoint_intents.lock().unwrap();
    live.session
        .source_breakpoint_statuses()
        .into_iter()
        .map(|(source, breakpoints)| {
            let intent = intents.get(&source);
            Response::SourceBreakpoints {
                session: session.to_owned(),
                source_hash: hashes.get(&source).cloned(),
                request_id: intent.map(|value| value.request_id.clone()),
                source,
                breakpoints,
            }
        })
        .collect()
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/debug_sources.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.


## Retire receipts while a replacement can partially affect the adapter

A DAP error does not prove that every source set remained unchanged. Suspend all
accepted intent tickets before delivery. Restore unchanged intents only after
the entire replacement succeeds, alongside the newly accepted intent. Failure
or cancellation leaves no ticket that a later stop could attach to changed
adapter state. The host retains wishes and history but retires bound markers.
These focused transition checks cover suspended, canceled and confirmed updates;
actual adapter partial-failure injection remains separate acceptance.

<storycode:copy id="debug-breakpoint-intent-update">
/// A DAP replacement may partially apply before failing. Never stamp its later
/// statuses with a receipt from the preceding accepted desired set.
pub(super) struct IntentUpdate<'a> {
    authority: &'a std::sync::Mutex<
        std::collections::HashMap<String, crate::debug_sessions::BreakpointIntent>,
    >,
    previous: std::collections::HashMap<String, crate::debug_sessions::BreakpointIntent>,
}
impl<'a> IntentUpdate<'a> {
    pub(super) fn begin(
        authority: &'a std::sync::Mutex<
            std::collections::HashMap<String, crate::debug_sessions::BreakpointIntent>,
        >,
    ) -> Self {
        Self {
            previous: std::mem::take(&mut *authority.lock().unwrap()),
            authority,
        }
    }
    pub(super) fn accept(
        mut self,
        source: String,
        intent: crate::debug_sessions::BreakpointIntent,
    ) {
        self.previous.insert(source, intent);
        *self.authority.lock().unwrap() = self.previous;
    }
}
#[cfg(test)]
mod intent_update_tests {
    use super::*;
    use crate::debug_sessions::BreakpointIntent;
    fn intent(ticket: &str) -> BreakpointIntent {
        BreakpointIntent {
            request_id: ticket.into(),
            source_hash: "a".repeat(64),
        }
    }
    #[test]
    fn failure_or_cancellation_cannot_reuse_any_previous_receipt() {
        let authority = std::sync::Mutex::new(std::collections::HashMap::from([
            ("".into(), intent("old-root")),
            ("other".into(), intent("old-other")),
        ]));
        let attempted = IntentUpdate::begin(&authority);
        assert!(
            authority.lock().unwrap().is_empty(),
            "in-flight partial effects have no accepted receipt"
        );
        drop(attempted); // An adapter error or canceled task never calls accept.
        assert!(
            authority.lock().unwrap().is_empty(),
            "a later stop must not stamp changed desired sets with old tickets"
        );
    }
    #[test]
    fn confirmed_replacement_preserves_unchanged_source_intents() {
        let authority = std::sync::Mutex::new(std::collections::HashMap::from([
            ("".into(), intent("old-root")),
            ("other".into(), intent("old-other")),
        ]));
        IntentUpdate::begin(&authority).accept("".into(), intent("new-root"));
        let accepted = authority.lock().unwrap();
        assert_eq!(accepted[""].request_id, "new-root");
        assert_eq!(accepted["other"].request_id, "old-other");
    }
}
</storycode:copy>
<storycode:file path="crates/hickory-cli/src/serve/debug_sources.rs"><storycode:paste select="#f-9c1c3fe2ca3c-document"/><storycode:paste select="#debug-breakpoint-source-admission"/><storycode:paste select="#f-9c1c3fe2ca3c-initial"/><storycode:paste select="#f-9c1c3fe2ca3c-set"/><storycode:paste select="#f-9c1c3fe2ca3c-statuses"/><storycode:paste select="#debug-breakpoint-intent-update"/></storycode:file>

</storycode:doc>
