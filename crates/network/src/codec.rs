//! QEMU `-netdev socket` stream protocol codec.
//!
//! Each frame on the wire is: `[4-byte big-endian length][payload]`.
//! The payload is a raw Ethernet frame.

use log::{trace, warn};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Read a single length-prefixed Ethernet frame.
///
/// Returns `Ok(None)` on clean EOF (reader closed).
pub async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> anyhow::Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {
            let len = u32::from_be_bytes(len_buf) as usize;
            trace!("[codec] Read frame header, len={len}");
            let mut frame = vec![0u8; len];
            if len > 0 {
                reader.read_exact(&mut frame).await?;
            }
            Ok(Some(frame))
        }
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            trace!("[codec] EOF on frame read");
            Ok(None)
        }
        Err(e) => {
            warn!("[codec] Error reading frame: {e}");
            Err(e.into())
        }
    }
}

/// Write a single length-prefixed Ethernet frame.
pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    frame: &[u8],
) -> anyhow::Result<()> {
    let len = frame.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(frame).await?;
    Ok(())
}

/// Bidirectional wrapper around an `AsyncRead + AsyncWrite` stream that can be
/// split into independent reader/writer halves for concurrent use.
pub struct FrameIo<S> {
    stream: S,
}

impl<S> FrameIo<S> {
    pub fn new(stream: S) -> Self {
        Self { stream }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> FrameIo<S> {
    /// Read the next frame. Returns `Ok(None)` on EOF.
    pub async fn read(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        read_frame(&mut self.stream).await
    }

    /// Write a frame.
    pub async fn write(&mut self, frame: &[u8]) -> anyhow::Result<()> {
        write_frame(&mut self.stream, frame).await
    }
}

impl<R, W> FrameIo<CombinedStream<R, W>>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    pub fn from_parts(reader: R, writer: W) -> Self {
        Self {
            stream: CombinedStream { reader, writer },
        }
    }
}

/// Split a `FrameIo` over a tokio `DuplexStream` (or any splittable stream)
/// into independent reader and writer halves.
pub fn split<R, W>(reader: R, writer: W) -> (FrameReader<R>, FrameWriter<W>)
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    (FrameReader { reader }, FrameWriter { writer })
}

/// Read-half of a split `FrameIo`.
pub struct FrameReader<R> {
    reader: R,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    pub async fn read(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        read_frame(&mut self.reader).await
    }
}

/// Write-half of a split `FrameIo`.
pub struct FrameWriter<W> {
    writer: W,
}

impl<W: AsyncWrite + Unpin> FrameWriter<W> {
    pub async fn write(&mut self, frame: &[u8]) -> anyhow::Result<()> {
        write_frame(&mut self.writer, frame).await
    }
}

/// A combined stream pairing separate reader and writer halves.
pub struct CombinedStream<R, W> {
    reader: R,
    writer: W,
}

impl<R: AsyncRead + Unpin, W: Unpin> AsyncRead for CombinedStream<R, W> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.reader).poll_read(cx, buf)
    }
}

impl<R: Unpin, W: AsyncWrite + Unpin> AsyncWrite for CombinedStream<R, W> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.writer).poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.writer).poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.writer).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn test_roundtrip_single_frame() {
        let (mut client, mut server) = duplex(1024);

        let payload = b"hello ethernet frame";
        write_frame(&mut client, payload).await.unwrap();

        let received = read_frame(&mut server).await.unwrap().unwrap();
        assert_eq!(received, payload);

        // Also verify server→client direction
        let payload2 = b"response frame";
        write_frame(&mut server, payload2).await.unwrap();

        let received2 = read_frame(&mut client).await.unwrap().unwrap();
        assert_eq!(received2, payload2);
    }

    #[tokio::test]
    async fn test_roundtrip_multiple_frames() {
        let (mut client, mut server) = duplex(65536);

        let frames: Vec<Vec<u8>> = vec![
            b"frame one".to_vec(),
            b"frame two with more data".to_vec(),
            vec![0xDE, 0xAD, 0xBE, 0xEF],
            vec![0u8; 1500], // MTU-sized frame
        ];

        for f in &frames {
            write_frame(&mut client, f).await.unwrap();
        }

        for expected in &frames {
            let received = read_frame(&mut server).await.unwrap().unwrap();
            assert_eq!(&received, expected);
        }
    }

    #[tokio::test]
    async fn test_read_frame_eof() {
        let (client, mut server) = duplex(1024);

        // Drop client entirely to trigger EOF on server read
        drop(client);

        let result = read_frame(&mut server).await.unwrap();
        assert!(result.is_none(), "Expected None on EOF, got {result:?}");
    }

    #[tokio::test]
    async fn test_write_zero_length_frame() {
        let (mut client, mut server) = duplex(1024);

        write_frame(&mut client, b"").await.unwrap();

        let received = read_frame(&mut server).await.unwrap().unwrap();
        assert!(received.is_empty());
    }

    #[tokio::test]
    async fn test_frame_io_split() {
        let (client, server) = duplex(4096);
        let (cr, cw) = tokio::io::split(client);
        let (sr, sw) = tokio::io::split(server);

        let (mut client_reader, mut client_writer) = split(cr, cw);
        let (mut server_reader, mut server_writer) = split(sr, sw);

        let send_task = tokio::spawn(async move {
            client_writer.write(b"ping").await.unwrap();
        });

        let recv_task = tokio::spawn(async move {
            let frame = server_reader.read().await.unwrap().unwrap();
            assert_eq!(frame, b"ping");
            // Send reply
            server_writer.write(b"pong").await.unwrap();
        });

        send_task.await.unwrap();
        recv_task.await.unwrap();

        let reply = client_reader.read().await.unwrap().unwrap();
        assert_eq!(reply, b"pong");
    }
}
