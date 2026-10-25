<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/src/prediction/source_question.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `06462887b0739637fdf473b77ff22b82364829f55653c24956c6e1e61b97b81a`.

<book:entry path="crates/hick-literate/src/prediction/source_question.rs">
//! Rebind one host-selected question after a source edit, without executing it.
//! Capture selection stays pinned; this does not establish semantic family identity.
use super::{OutputEncoding, OutputRequest, SealedRequest, combined, debugger, output_target};
use anyhow::{Result, ensure};
use hick_dap::CaptureSpec;
use hick_observe::prediction::{Identity, Target};
use std::path::Path;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceQuestion {
    template: Target,
    capture: Option<CaptureSpec>,
    anchor: Option<String>,
    container: Option<String>,
    encoding: OutputEncoding,
}
impl SourceQuestion {
    pub fn capture(
        request: &SealedRequest,
        source: &str,
        encoding: OutputEncoding,
    ) -> Result<Self> {
        let scratch = tempfile::tempdir()?;
        let capture = Self::validate(request, source, encoding, scratch.path())?;
        let container = if matches!(
            request.target,
            Target::Output { .. } | Target::Combined { .. }
        ) {
            let document = hick_lang::parse(source)?;
            let execs: Vec<_> = document
                .all_tags()
                .into_iter()
                .filter(|tag| tag.name == "exec")
                .collect();
            ensure!(execs.len() == 1);
            Some(
                execs[0]
                    .get_attribute("container")
                    .ok_or_else(|| anyhow::anyhow!("Captured execution container unavailable"))?
                    .to_owned(),
            )
        } else {
            None
        };
        let debug_target = match &request.target {
            Target::Debugger { .. } => Some(&request.target),
            Target::Combined { debugger, .. } => Some(debugger.as_ref()),
            _ => None,
        };
        let anchor = match debug_target {
            Some(Target::Debugger {
                start_byte,
                end_byte,
                ..
            }) => Some(
                source
                    .get(*start_byte as usize..*end_byte as usize)
                    .ok_or_else(|| anyhow::anyhow!("Capture source span unavailable"))?
                    .to_owned(),
            ),
            _ => None,
        };
        Ok(Self {
            template: request.target.clone(),
            capture,
            anchor,
            container,
            encoding,
        })
    }
    fn validate(
        request: &SealedRequest,
        source: &str,
        encoding: OutputEncoding,
        scratch: &Path,
    ) -> Result<Option<CaptureSpec>> {
        Ok(match &request.target {
            Target::Output { .. } => {
                let actual = super::validate(
                    &OutputRequest {
                        seal: request.seal.clone(),
                        request_id: request.request_id.clone(),
                        actor: request.actor.clone(),
                        identity: request.identity.clone(),
                        encoding,
                    },
                    source,
                )?;
                ensure!(
                    actual == request.target,
                    "Output question differs from source"
                );
                None
            }
            Target::Debugger { .. } => {
                Some(debugger::validate_source(request, source, scratch, false)?.0)
            }
            Target::Combined { .. } => {
                ensure!(
                    matches!(encoding, OutputEncoding::Json),
                    "Combined question requires JSON output"
                );
                Some(combined::validate(request, source, scratch)?.0)
            }
        })
    }
    /// The host independently supplies identities; this method binds only the
    /// approved source location. It cannot re-capture inputs or environment.
    pub fn rebind(&self, identity: &Identity, source: &str) -> Result<Target> {
        let scratch = tempfile::tempdir()?;
        let output = || -> Result<Target> {
            let document = hick_lang::parse(source)?;
            let execs: Vec<_> = document
                .all_tags()
                .into_iter()
                .filter(|tag| tag.name == "exec")
                .collect();
            ensure!(
                execs.len() == 1
                    && execs[0].get_attribute("container") == self.container.as_deref(),
                "Execution question changed"
            );
            Ok(output_target(
                self.container.as_deref().unwrap(),
                execs[0].source_line,
                self.encoding,
            ))
        };
        let debugger = |template: &Target| -> Result<Target> {
            let pinned = self
                .capture
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Capture question unavailable"))?;
            let document = hick_lang::parse(source)?;
            let captures = crate::capture::collect(&document.nodes).map_err(anyhow::Error::msg)?;
            ensure!(
                captures.len() == 1
                    && captures[0].specs.len() == 1
                    && serde_json::to_value(&captures[0].specs[0])?
                        == serde_json::to_value(pinned)?,
                "Capture file, location, expression, condition or hit bound changed"
            );
            let mapping =
                hick_dap::Mapping::for_document(Path::new("held-out.md"), source, scratch.path())?;
            let line = mapping
                .to_document(
                    &scratch.path().join(&pinned.file),
                    pinned
                        .line
                        .checked_sub(1)
                        .ok_or_else(|| anyhow::anyhow!("Capture line unavailable"))?,
                )
                .ok_or_else(|| anyhow::anyhow!("Capture mapping unavailable"))?;
            let (start, end) = debugger::source_span(source, line)?;
            ensure!(
                source.get(start as usize..end as usize) == self.anchor.as_deref(),
                "Selected source statement changed; host question review is required"
            );
            let mut target = template.clone();
            let Target::Debugger {
                source_revision,
                start_byte,
                end_byte,
                ..
            } = &mut target
            else {
                anyhow::bail!("Debugger question required")
            };
            *source_revision = identity.source.clone();
            *start_byte = start;
            *end_byte = end;
            Ok(target)
        };
        let target = match &self.template {
            Target::Output { .. } => output()?,
            Target::Debugger { .. } => debugger(&self.template)?,
            Target::Combined { debugger: arm, .. } => Target::Combined {
                output: Box::new(output()?),
                debugger: Box::new(debugger(arm)?),
            },
        };
        let request = SealedRequest {
            seal: String::new(),
            request_id: String::new(),
            actor: hick_observe::prediction::Actor {
                kind: "host".into(),
                id: "source-question-binding".into(),
            },
            identity: identity.clone(),
            target: target.clone(),
        };
        Self::validate(&request, source, self.encoding, scratch.path())?;
        Ok(target)
    }
}

/// Host-reviewed external-stimulus syntax for the currently supported inline
/// fixtures. Preserving these anchors does not prove program semantics.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInputs {
    pub value: serde_json::Value,
    pub anchors: Vec<String>,
}
impl SourceInputs {
    pub fn validate(&self, identity: &Identity, source: &str) -> Result<()> {
        ensure!(
            hick_observe::prediction::digest(&self.value)? == identity.inputs,
            "Source input identity differs from the host capture"
        );
        ensure!(
            !self.anchors.is_empty() && self.anchors.len() <= 32,
            "A bounded host input binding is required"
        );
        for anchor in &self.anchors {
            ensure!(
                !anchor.is_empty() && anchor.len() <= 65536 && source.matches(anchor).count() == 1,
                "Host-selected input syntax changed or is ambiguous"
            );
        }
        Ok(())
    }
}
</book:entry>

</book:doc>
