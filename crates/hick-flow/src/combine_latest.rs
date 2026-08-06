//! `DynamicCombineLatest` — a reactive stream combinator that dynamically
//! manages multiple input sources and emits combined results when all sources
//! have produced at least one value.

use std::collections::{BTreeMap, HashSet};
use std::pin::Pin;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::node::BoxStream;

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

enum Message<T> {
    AddSource { id: usize, source: BoxStream<T> },
    RemoveSource { id: usize },
    Data { id: usize, value: T },
    Complete { id: usize },
}

// ---------------------------------------------------------------------------
// DynamicCombineLatest
// ---------------------------------------------------------------------------

/// A reactive combinator that tracks multiple async stream sources, collecting
/// latest values and emitting combined results whenever all active sources
/// have produced at least one value.
pub struct DynamicCombineLatest<TSource, TResult>
where
    TSource: Clone + Send + Sync + 'static,
    TResult: Send + Sync + 'static,
{
    combiner: std::sync::Arc<dyn Fn(Vec<TSource>) -> TResult + Send + Sync>,
    tx: mpsc::UnboundedSender<Message<TSource>>,
    rx: Mutex<Option<mpsc::UnboundedReceiver<Message<TSource>>>>,
    global_token: CancellationToken,
    id_counter: AtomicUsize,
    complete_on_quiescence: bool,
}

impl<TSource, TResult> DynamicCombineLatest<TSource, TResult>
where
    TSource: Clone + Send + Sync + 'static,
    TResult: Send + Sync + 'static,
{
    pub fn new<F>(combiner: F) -> Self
    where
        F: Fn(Vec<TSource>) -> TResult + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            combiner: std::sync::Arc::new(combiner),
            tx,
            rx: Mutex::new(Some(rx)),
            global_token: CancellationToken::new(),
            id_counter: AtomicUsize::new(0),
            complete_on_quiescence: true,
        }
    }

    /// Constructor allowing configuration of the completion policy.
    pub fn new_with_config<F>(combiner: F, complete_on_quiescence: bool) -> Self
    where
        F: Fn(Vec<TSource>) -> TResult + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            combiner: std::sync::Arc::new(combiner),
            tx,
            rx: Mutex::new(Some(rx)),
            global_token: CancellationToken::new(),
            id_counter: AtomicUsize::new(0),
            complete_on_quiescence,
        }
    }

    pub fn add_source<S>(&self, source: S) -> usize
    where
        S: Stream<Item = TSource> + Send + 'static,
    {
        let id = self.id_counter.fetch_add(1, Ordering::SeqCst) + 1;
        let boxed_source: BoxStream<TSource> = Box::pin(source);
        let _ = self.tx.send(Message::AddSource {
            id,
            source: boxed_source,
        });
        id
    }

    pub fn remove_source(&self, id: usize) {
        let _ = self.tx.send(Message::RemoveSource { id });
    }

    /// Single-reader: panics if called twice.
    pub fn stream(&self) -> impl Stream<Item = TResult> {
        let rx = self
            .rx
            .lock()
            .unwrap()
            .take()
            .expect("DynamicCombineLatest only supports one active enumerator at a time.");

        let global_token = self.global_token.clone();
        let tx = self.tx.clone();
        let combiner = self.combiner.clone();
        let complete_on_quiescence = self.complete_on_quiescence;

        DynamicCombineLatestStream {
            rx,
            tx,
            combiner,
            handlers: HashSet::new(),
            latest_values: BTreeMap::new(),
            unready_ids: HashSet::new(),
            token: global_token,
            complete_on_quiescence,
            ever_had_source: false,
        }
    }

    pub fn shutdown(&self) {
        self.global_token.cancel();
    }
}

// ---------------------------------------------------------------------------
// Stream implementation
// ---------------------------------------------------------------------------

struct DynamicCombineLatestStream<TSource, TResult> {
    rx: mpsc::UnboundedReceiver<Message<TSource>>,
    tx: mpsc::UnboundedSender<Message<TSource>>,
    combiner: std::sync::Arc<dyn Fn(Vec<TSource>) -> TResult + Send + Sync>,
    token: CancellationToken,
    complete_on_quiescence: bool,
    ever_had_source: bool,

    handlers: HashSet<usize>,
    latest_values: BTreeMap<usize, TSource>,
    unready_ids: HashSet<usize>,
}

impl<TSource, TResult> Stream for DynamicCombineLatestStream<TSource, TResult>
where
    TSource: Clone + Send + Sync + 'static,
    TResult: Send + Sync + 'static,
{
    type Item = TResult;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            if self.token.is_cancelled() {
                return std::task::Poll::Ready(None);
            }

            if self.complete_on_quiescence && self.ever_had_source && self.handlers.is_empty() {
                return std::task::Poll::Ready(None);
            }

            match self.rx.poll_recv(cx) {
                std::task::Poll::Ready(Some(msg)) => match msg {
                    Message::AddSource { id, source } => {
                        if !self.handlers.contains(&id) {
                            self.handlers.insert(id);
                            self.unready_ids.insert(id);
                            self.ever_had_source = true;

                            let tx = self.tx.clone();
                            let token = self.token.clone();
                            let task_id = id;
                            let mut stream = source;

                            tokio::spawn(async move {
                                loop {
                                    tokio::select! {
                                        _ = token.cancelled() => break,
                                        next = stream.next() => {
                                            match next {
                                                Some(item) => {
                                                    if tx.send(Message::Data { id: task_id, value: item }).is_err() {
                                                        break;
                                                    }
                                                }
                                                None => {
                                                    let _ = tx.send(Message::Complete { id: task_id });
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                    Message::RemoveSource { id } => {
                        self.handlers.remove(&id);
                        self.latest_values.remove(&id);
                        self.unready_ids.remove(&id);
                    }
                    Message::Data { id, value } => {
                        if self.handlers.contains(&id) {
                            self.latest_values.insert(id, value);
                            self.unready_ids.remove(&id);

                            if self.unready_ids.is_empty() && !self.handlers.is_empty() {
                                let snapshot: Vec<TSource> =
                                    self.latest_values.values().cloned().collect();
                                let result = (self.combiner)(snapshot);
                                return std::task::Poll::Ready(Some(result));
                            }
                        }
                    }
                    Message::Complete { id } => {
                        self.handlers.remove(&id);
                        // Keep last value in latest_values; keep stream alive.
                    }
                },
                std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }
    }
}
