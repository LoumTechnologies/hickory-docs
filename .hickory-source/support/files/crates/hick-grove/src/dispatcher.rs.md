<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-grove/src/dispatcher.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0442f07381aa8224d5d7d5c3e3eebbe2a2e2a1e5ad82690e6b5c7714bcb31cd2`.

<book:entry path="crates/hick-grove/src/dispatcher.rs">
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
</book:entry>

</book:doc>
