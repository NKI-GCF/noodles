mod builder;
pub(crate) mod deflate;
mod deflater;

pub use self::builder::Builder;

use std::{
    cmp,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Buf, Bytes, BytesMut};
use flate2::Compression;
use futures::{ready, sink::Buffer, Sink};
use pin_project_lite::pin_project;
use tokio::io::{self, AsyncWrite};

use crate::block;

use self::{deflate::Deflate, deflater::Deflater};

pin_project! {
    /// An async BGZF writer.
    pub struct Writer<W> {
        #[pin]
        sink: Buffer<Deflater<W>, Deflate>,
        buf: BytesMut,
        #[pin]
        eof_buf: Bytes,
        compression_level: Compression,
    }
}

impl<W> Writer<W>
where
    W: AsyncWrite + Unpin,
{
    /// Creates an async BGZF writer builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// let builder = bgzf::AsyncWriter::builder(Vec::new());
    /// let writer = builder.build();
    /// ```
    pub fn builder(inner: W) -> Builder<W> {
        Builder::new(inner)
    }

    /// Creates an async BGZF writer with a default compression level.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// let writer = bgzf::AsyncWriter::new(Vec::new());
    /// ```
    pub fn new(inner: W) -> Self {
        Self::builder(inner).build()
    }

    /// Returns the underlying writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// let writer = bgzf::AsyncWriter::new(Vec::new());
    /// assert!(writer.into_inner().is_empty());
    /// ```
    pub fn into_inner(self) -> W {
        self.sink.into_inner().into_inner()
    }
}

impl<W> AsyncWrite for Writer<W>
where
    W: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        if self.buf.len() >= block::MAX_UNCOMPRESSED_DATA_LENGTH {
            if let Err(e) = ready!(self.as_mut().poll_flush(cx)) {
                return Poll::Ready(Err(e));
            }
        }

        let n = cmp::min(
            block::MAX_UNCOMPRESSED_DATA_LENGTH - self.buf.len(),
            buf.len(),
        );

        self.as_mut().buf.extend_from_slice(&buf[..n]);

        Poll::Ready(Ok(n))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let mut this = self.project();

        if this.buf.is_empty() {
            return Poll::Ready(Ok(()));
        }

        ready!(this.sink.as_mut().poll_ready(cx))?;

        let buf = this.buf.split();
        this.sink
            .as_mut()
            .start_send(Deflate::new(buf, *this.compression_level))?;

        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        ready!(self.as_mut().poll_flush(cx))?;

        let mut this = self.project();
        let mut sink = this.sink.as_mut();

        ready!(sink.as_mut().poll_close(cx))?;

        let mut inner = sink.get_mut().get_mut().get_mut();

        while this.eof_buf.has_remaining() {
            let bytes_written = ready!(Pin::new(&mut inner).poll_write(cx, this.eof_buf.chunk()))?;

            this.eof_buf.advance(bytes_written);

            if bytes_written == 0 {
                return Poll::Ready(Err(io::Error::from(io::ErrorKind::WriteZero)));
            }
        }

        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::writer::BGZF_EOF;

    #[tokio::test]
    async fn test_write() -> io::Result<()> {
        let mut writer = Writer::new(Vec::new());
        writer.write_all(b"noodles").await?;
        writer.shutdown().await?;

        let actual = writer.into_inner();

        let mut expected = vec![
            0x1f, 0x8b, 0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x06, 0x00, 0x42, 0x43,
            0x02, 0x00, 0x22, 0x00, 0xcb, 0xcb, 0xcf, 0x4f, 0xc9, 0x49, 0x2d, 0x06, 0x00, 0xa1,
            0x58, 0x2a, 0x80, 0x07, 0x00, 0x00, 0x00,
        ];
        expected.extend_from_slice(BGZF_EOF);

        assert_eq!(actual, expected);

        Ok(())
    }
}
