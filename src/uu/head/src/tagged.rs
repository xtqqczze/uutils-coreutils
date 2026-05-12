use std::io::{self, Read, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum IoError {
    #[error("read error: {}", _0)]
    Read(io::Error),
    #[error("write error: {}", _0)]
    Write(io::Error),
}

pub(crate) struct Reader<R> {
    inner: R,
}

impl<W> From<W> for Reader<W> {
    fn from(inner: W) -> Self {
        Self { inner }
    }
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner
            .read(buf)
            .map_err(|e| io::Error::new(e.kind(), IoError::Read(e)))
    }
}

pub(crate) struct Writer<W> {
    inner: W,
}

impl<W> From<W> for Writer<W> {
    fn from(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner
            .write(buf)
            .map_err(|e| io::Error::new(e.kind(), IoError::Write(e)))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner
            .flush()
            .map_err(|e| io::Error::new(e.kind(), IoError::Write(e)))
    }
}
