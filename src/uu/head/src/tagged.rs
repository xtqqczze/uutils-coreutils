use std::io::{self, Read, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum IoError {
    #[error("read error: {0}")]
    Read(io::Error),
    #[error("write error: {0}")]
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

pub fn copy<R, W>(reader: &mut R, writer: &mut W) -> io::Result<u64>
where
    R: Read + ?Sized,
    W: Write + ?Sized,
{
    io::copy(&mut Reader::from(reader), &mut Writer::from(writer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind, Read, Write};

    struct FailReader;

    impl Read for FailReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(Error::new(ErrorKind::BrokenPipe, "broken pipe"))
        }
    }

    struct FailWriter;

    impl Write for FailWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(Error::new(ErrorKind::BrokenPipe, "broken pipe"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(Error::new(ErrorKind::BrokenPipe, "broken pipe"))
        }
    }

    #[test]
    fn test_reader_wraps_read_errors() {
        let mut reader = Reader::from(FailReader);
        let mut buffer = [0u8; 1];
        let err = reader.read(&mut buffer).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::BrokenPipe);
        assert!(err.to_string().contains("read error: broken pipe"));
        let inner = err.get_ref().and_then(|e| e.downcast_ref::<IoError>());
        assert!(matches!(inner, Some(IoError::Read(_))));
    }

    #[test]
    fn test_writer_wraps_write_errors() {
        let mut writer = Writer::from(FailWriter);
        let err = writer.write(b"hello").unwrap_err();

        assert_eq!(err.kind(), ErrorKind::BrokenPipe);
        assert!(err.to_string().contains("write error: broken pipe"));
        let inner = err.get_ref().and_then(|e| e.downcast_ref::<IoError>());
        assert!(matches!(inner, Some(IoError::Write(_))));
    }

    #[test]
    fn test_writer_wraps_flush_errors() {
        let mut writer = Writer::from(FailWriter);
        let err = writer.flush().unwrap_err();

        assert_eq!(err.kind(), ErrorKind::BrokenPipe);
        assert!(err.to_string().contains("write error: broken pipe"));
        let inner = err.get_ref().and_then(|e| e.downcast_ref::<IoError>());
        assert!(matches!(inner, Some(IoError::Write(_))));
    }
}
