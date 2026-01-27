use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, DuplexStream, ReadBuf};
use wasmtime_wasi::cli::{IsTerminal, StdinStream, StdoutStream};

/// A stdin stream backed by a [`DuplexStream`], allowing the host to write
/// bytes that the WASM module reads as its standard input.
///
/// Cloning produces a handle to the same underlying stream, satisfying the
/// WASI requirement that multiple stdin handles share read progress.
#[derive(Clone)]
pub struct ChannelStdin(Arc<std::sync::Mutex<DuplexStream>>);

impl ChannelStdin {
    pub fn new(stream: DuplexStream) -> Self {
        Self(Arc::new(std::sync::Mutex::new(stream)))
    }
}

impl AsyncRead for ChannelStdin {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut guard = self.0.lock().unwrap();
        Pin::new(&mut *guard).poll_read(cx, buf)
    }
}

impl IsTerminal for ChannelStdin {
    fn is_terminal(&self) -> bool {
        false
    }
}

impl StdinStream for ChannelStdin {
    fn async_stream(&self) -> Box<dyn AsyncRead + Send + Sync> {
        Box::new(self.clone())
    }
}

/// A stdout/stderr stream backed by a [`DuplexStream`], allowing the WASM
/// module to write output that the host reads.
///
/// Cloning produces a handle to the same underlying stream, satisfying the
/// WASI requirement that multiple output handles share write state.
#[derive(Clone)]
pub struct ChannelStdout(Arc<std::sync::Mutex<DuplexStream>>);

impl ChannelStdout {
    pub fn new(stream: DuplexStream) -> Self {
        Self(Arc::new(std::sync::Mutex::new(stream)))
    }
}

impl AsyncWrite for ChannelStdout {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut guard = self.0.lock().unwrap();
        Pin::new(&mut *guard).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut guard = self.0.lock().unwrap();
        Pin::new(&mut *guard).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut guard = self.0.lock().unwrap();
        Pin::new(&mut *guard).poll_shutdown(cx)
    }
}

impl IsTerminal for ChannelStdout {
    fn is_terminal(&self) -> bool {
        false
    }
}

impl StdoutStream for ChannelStdout {
    fn async_stream(&self) -> Box<dyn AsyncWrite + Send + Sync> {
        Box::new(self.clone())
    }
}
