<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/dispatcher.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c38022cb5034"></a>
## `crates/hick-grove/src/dispatcher.rs`

<a id="f-c38022cb5034-Dispatcher"></a>
### `Dispatcher`

<storycode:copy id="f-c38022cb5034-Dispatcher">
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::namespace::DocId;
use crate::plugin::{HandlerContext, NamespaceRegistry, SideEffect};
use crate::yrs_bridge::YrsChangeEvent;

/// Dispatches Yrs change events to registered namespace handlers.
pub struct Dispatcher {
    registry: Arc<NamespaceRegistry>,
    rx: mpsc::UnboundedReceiver<YrsChangeEvent>,
    doc_id: DocId,
}

</storycode:copy>
<a id="f-c38022cb5034-impl-Dispatcher"></a>
### `impl-Dispatcher`

<storycode:copy id="f-c38022cb5034-impl-Dispatcher">
impl Dispatcher {
    pub fn new(
        registry: Arc<NamespaceRegistry>,
        rx: mpsc::UnboundedReceiver<YrsChangeEvent>,
        doc_id: DocId,
    ) -> Self {
        Self {
            registry,
            rx,
            doc_id,
        }
    }

    /// Run the dispatch loop, consuming events until the channel closes.
    /// Returns all side effects produced.
    pub async fn run(mut self) -> Vec<SideEffect> {
        let mut all_effects = Vec::new();

        while let Some(change) = self.rx.recv().await {
            let ctx = HandlerContext {
                doc_id: self.doc_id.clone(),
            };

            let handlers = self.registry.handlers_for(&change.namespace_uri);
            for handler in handlers {
                match handler.handle_event(&change.event, &ctx) {
                    Ok(effects) => all_effects.extend(effects),
                    Err(e) => {
                        log::error!(
                            "Handler '{}' error for {:?}: {}",
                            handler.name(),
                            change.namespace_uri,
                            e
                        );
                    }
                }
            }
        }

        all_effects
    }

    /// Process a single batch of events (non-blocking, drains what's available).
    pub fn drain_pending(&mut self) -> Vec<SideEffect> {
        let mut all_effects = Vec::new();

        while let Ok(change) = self.rx.try_recv() {
            let ctx = HandlerContext {
                doc_id: self.doc_id.clone(),
            };

            let handlers = self.registry.handlers_for(&change.namespace_uri);
            for handler in handlers {
                match handler.handle_event(&change.event, &ctx) {
                    Ok(effects) => all_effects.extend(effects),
                    Err(e) => {
                        log::error!(
                            "Handler '{}' error for {:?}: {}",
                            handler.name(),
                            change.namespace_uri,
                            e
                        );
                    }
                }
            }
        }

        all_effects
    }
}
</storycode:copy>

### Assemble `crates/hick-grove/src/dispatcher.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/dispatcher.rs"><storycode:paste select="#f-c38022cb5034-Dispatcher"/><storycode:paste select="#f-c38022cb5034-impl-Dispatcher"/></storycode:file>

</storycode:doc>
