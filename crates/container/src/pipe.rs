use std::any::Any;
use std::io::{IoSlice, IoSliceMut, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, RwLock};

use wasi_common::Error;
use wasi_common::file::{FdFlags, FileType, WasiFile};

/// WASI stdin backed by a non-blocking [`UnixStream`].
///
/// Implements [`WasiFile`] with synchronous `read_vectored`. The stream is
/// non-blocking; when no data is available, returns EAGAIN so the kernel
/// can proceed with other operations without blocking.
#[derive(Clone)]
pub struct PipeReader {
    inner: Arc<RwLock<UnixStream>>,
}

impl PipeReader {
    fn new(stream: UnixStream) -> Self {
        Self {
            inner: Arc::new(RwLock::new(stream)),
        }
    }

    /// Create from a raw UnixStream (for testing).
    pub fn from_unix_stream(stream: UnixStream) -> Self {
        Self::new(stream)
    }
}

#[async_trait::async_trait]
impl WasiFile for PipeReader {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(unix)]
    fn pollable(&self) -> Option<rustix::fd::BorrowedFd<'_>> {
        // Expose the underlying UnixStream fd for poll_oneoff.
        // SAFETY: The fd is valid as long as `self` lives because the UnixStream
        // is owned by the Arc<RwLock<UnixStream>> which won't be dropped while
        // `self` exists.
        let guard = self.inner.read().unwrap();
        Some(unsafe {
            rustix::fd::BorrowedFd::borrow_raw(std::os::fd::AsRawFd::as_raw_fd(&*guard))
        })
    }

    async fn get_filetype(&self) -> Result<FileType, Error> {
        Ok(FileType::Pipe)
    }

    async fn readable(&self) -> Result<(), Error> {
        // For a non-blocking pipe, return immediately. The kernel can then
        // call fd_read, which will return EAGAIN if no data is available.
        // This allows poll_oneoff to complete so the kernel can proceed
        // with other operations (like writing to stdout).
        Ok(())
    }

    async fn read_vectored<'a>(&self, bufs: &mut [IoSliceMut<'a>]) -> Result<u64, Error> {
        // Non-blocking read: return EAGAIN if no data, not 0 (which means EOF).
        // The WouldBlock error is converted to EAGAIN by wasi-common.
        match self.inner.write().unwrap().read_vectored(bufs) {
            Ok(n) => Ok(n as u64),
            Err(e) => Err(e.into()), // WouldBlock becomes EAGAIN
        }
    }
}

/// WASI stdout/stderr backed by a [`UnixStream`].
///
/// Implements [`WasiFile`] with synchronous `write_vectored`.
#[derive(Clone)]
pub struct PipeWriter {
    inner: Arc<RwLock<UnixStream>>,
}

impl PipeWriter {
    fn new(stream: UnixStream) -> Self {
        Self {
            inner: Arc::new(RwLock::new(stream)),
        }
    }

    /// Create from a raw UnixStream (for testing).
    pub fn from_unix_stream(stream: UnixStream) -> Self {
        Self::new(stream)
    }
}

#[async_trait::async_trait]
impl WasiFile for PipeWriter {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(unix)]
    fn pollable(&self) -> Option<rustix::fd::BorrowedFd<'_>> {
        let guard = self.inner.read().unwrap();
        Some(unsafe {
            rustix::fd::BorrowedFd::borrow_raw(std::os::fd::AsRawFd::as_raw_fd(&*guard))
        })
    }

    async fn get_filetype(&self) -> Result<FileType, Error> {
        Ok(FileType::Pipe)
    }

    async fn get_fdflags(&self) -> Result<FdFlags, Error> {
        Ok(FdFlags::APPEND)
    }

    async fn writable(&self) -> Result<(), Error> {
        // Pipe is always writable from the WASM side's perspective.
        // poll_oneoff needs this to complete so fd_write can proceed.
        // If the pipe buffer is full, the synchronous write will block,
        // but that's acceptable since the host is always reading.
        Ok(())
    }

    async fn write_vectored<'a>(&self, bufs: &[IoSlice<'a>]) -> Result<u64, Error> {
        let n = self.inner.write().unwrap().write_vectored(bufs)?;
        Ok(n as u64)
    }
}

/// Create a stdin pipe pair.
///
/// Returns `(host_write, wasi_read)` where the host writes bytes into
/// `host_write` (a tokio `UnixStream`) and the WASM module reads them
/// via the `PipeReader` WasiFile.
pub fn stdin_pipe() -> anyhow::Result<(tokio::net::UnixStream, PipeReader)> {
    let (host, wasi) = UnixStream::pair()?;
    host.set_nonblocking(true)?;
    // WASI side is non-blocking: reads return EAGAIN when no data is available,
    // allowing the kernel to proceed with other operations (like stdout writes)
    // instead of blocking on terminal escape sequence responses.
    wasi.set_nonblocking(true)?;
    let host = tokio::net::UnixStream::from_std(host)?;
    let wasi = PipeReader::new(wasi);
    Ok((host, wasi))
}

/// Create a stdout/stderr pipe pair.
///
/// Returns `(host_read, wasi_write)` where the WASM module writes bytes
/// into the `PipeWriter` WasiFile and the host reads them from `host_read`
/// (a tokio `UnixStream`).
pub fn stdout_pipe() -> anyhow::Result<(tokio::net::UnixStream, PipeWriter)> {
    let (host, wasi) = UnixStream::pair()?;
    host.set_nonblocking(true)?;
    let host = tokio::net::UnixStream::from_std(host)?;
    let wasi = PipeWriter::new(wasi);
    Ok((host, wasi))
}

/// Create a stderr pipe pair.
pub fn stderr_pipe() -> anyhow::Result<(tokio::net::UnixStream, PipeWriter)> {
    let (host, wasi) = UnixStream::pair()?;
    host.set_nonblocking(true)?;
    let host = tokio::net::UnixStream::from_std(host)?;
    let wasi = PipeWriter::new(wasi);
    Ok((host, wasi))
}
