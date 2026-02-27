use std::sync::OnceLock;

use tracing::field::{Field, Visit};
use tracing::span;
use tracing::subscriber::Interest;
use tracing::{Event, Metadata, Subscriber};

/// Callback interface for forwarding Rust log events across the FFI boundary.
#[uniffi::export(callback_interface)]
pub trait LogSink: Send + Sync {
    fn log(&self, level: String, message: String, target: String, file: String, line: u32);
}

static SINK: OnceLock<Box<dyn LogSink>> = OnceLock::new();

/// Install a global tracing subscriber that forwards events to the given sink.
///
/// Safe to call more than once — only the first call takes effect.
#[uniffi::export]
pub fn init_logging(sink: Box<dyn LogSink>) {
    if SINK.set(sink).is_ok() {
        let _ = tracing::subscriber::set_global_default(CallbackSubscriber);
    }
}

struct CallbackSubscriber;

impl Subscriber for CallbackSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
        Interest::always()
    }

    fn new_span(&self, _span: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(1)
    }

    fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    fn event(&self, event: &Event<'_>) {
        let Some(sink) = SINK.get() else { return };

        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let meta = event.metadata();

        sink.log(
            meta.level().as_str().to_owned(),
            visitor.finish(),
            meta.target().to_owned(),
            meta.file().unwrap_or("").to_owned(),
            meta.line().unwrap_or(0),
        );
    }

    fn enter(&self, _span: &span::Id) {}
    fn exit(&self, _span: &span::Id) {}
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
    fields: Vec<String>,
}

impl MessageVisitor {
    fn finish(self) -> String {
        if self.fields.is_empty() {
            self.message
        } else {
            format!("{} {}", self.message, self.fields.join(" "))
        }
    }
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields.push(format!("{}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_owned();
        } else {
            self.fields.push(format!("{}=\"{}\"", field.name(), value));
        }
    }
}
