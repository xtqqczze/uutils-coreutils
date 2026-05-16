use std::io::{self, Read, Write};
use uucore::error::strip_errno;

#[derive(Debug, thiserror::Error)]
#[error("{}", strip_errno(_0))]
pub(crate) struct ReadError(io::Error);

#[derive(Debug, thiserror::Error)]
#[error("{}", strip_errno(_0))]
pub(crate) struct WriteError(io::Error);

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
            .map_err(|e| io::Error::new(e.kind(), ReadError(e)))
    }
}

pub(crate) enum Writer<W> {
    Stdout(W),
    File(W),
}

impl<W> From<W> for Writer<W> {
    fn from(value: W) -> Self {
        Writer::File(value)
    }
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::Stdout(inner) | Writer::File(inner) => inner
                .write(buf)
                .map_err(|e| io::Error::new(e.kind(), WriteError(e))),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::Stdout(inner) | Writer::File(inner) => inner
                .flush()
                .map_err(|e| io::Error::new(e.kind(), WriteError(e))),
        }
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
            Err(Error::new(
                ErrorKind::BrokenPipe,
                Error::from_raw_os_error(32),
            ))
        }
    }

    struct FailWriter;

    impl Write for FailWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(Error::new(
                ErrorKind::BrokenPipe,
                Error::from_raw_os_error(32),
            ))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(Error::new(
                ErrorKind::BrokenPipe,
                Error::from_raw_os_error(32),
            ))
        }
    }

    #[test]
    fn test_reader_wraps_read_errors() {
        let mut reader = Reader::from(FailReader);
        let mut buffer = [0u8; 1];
        let err = reader.read(&mut buffer).unwrap_err();

        assert_eq!(err.raw_os_error(), Some(32));
        assert_eq!(err.kind(), ErrorKind::BrokenPipe);
        assert_eq!(err.to_string(), "Broken pipe");
        let inner = err.get_ref().and_then(|e| e.downcast_ref::<ReadError>());
        assert!(matches!(inner, Some(ReadError(_))));
    }

    #[test]
    fn test_writer_wraps_write_errors() {
        let mut writer = Writer::File(FailWriter);
        let err = writer.write(b"hello").unwrap_err();

        assert_eq!(err.kind(), ErrorKind::BrokenPipe);
        assert_eq!(err.to_string(), "Broken pipe");
        let inner = err.get_ref().and_then(|e| e.downcast_ref::<WriteError>());
        assert!(matches!(inner, Some(WriteError(_))));
    }

    #[test]
    fn test_writer_wraps_flush_errors() {
        let mut writer = Writer::File(FailWriter);
        let err = writer.flush().unwrap_err();

        assert_eq!(err.kind(), ErrorKind::BrokenPipe);
        assert_eq!(err.to_string(), "Broken pipe");
        let inner = err.get_ref().and_then(|e| e.downcast_ref::<WriteError>());
        assert!(matches!(inner, Some(WriteError(_))));
    }
}
