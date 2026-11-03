//! Newline-delimited JSON framing used on the coordinator's stdin/stdout.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::io::{self, BufRead, Write};

#[derive(Debug)]
pub enum WireError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WireError::Io(e) => write!(f, "i/o error: {e}"),
            WireError::Json(e) => write!(f, "invalid message: {e}"),
        }
    }
}

impl std::error::Error for WireError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WireError::Io(e) => Some(e),
            WireError::Json(e) => Some(e),
        }
    }
}

impl From<io::Error> for WireError {
    fn from(e: io::Error) -> Self {
        WireError::Io(e)
    }
}

impl From<serde_json::Error> for WireError {
    fn from(e: serde_json::Error) -> Self {
        WireError::Json(e)
    }
}

/// Write one message as a single line and flush.
///
/// Serializes to a buffer first, so a failing writer (e.g. a closed pipe) is
/// reported as [`WireError::Io`], never as a JSON error, and a line is written whole.
pub fn send<W: Write, T: Serialize>(w: &mut W, msg: &T) -> Result<(), WireError> {
    let mut line = serde_json::to_vec(msg)?;
    line.push(b'\n');
    w.write_all(&line)?;
    w.flush()?;
    Ok(())
}

/// Read the next message. Returns `Ok(None)` at end of stream; blank lines are skipped.
pub fn recv<R: BufRead, T: DeserializeOwned>(r: &mut R) -> Result<Option<T>, WireError> {
    let mut line = String::new();
    loop {
        line.clear();
        if r.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if !line.trim().is_empty() {
            return Ok(Some(serde_json::from_str(&line)?));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{CoordinatorToSim, Decision};

    #[test]
    fn round_trips_over_a_buffer() {
        let mut buf = Vec::new();
        let msg = CoordinatorToSim::Decision(Decision::empty(3));
        send(&mut buf, &msg).unwrap();
        send(&mut buf, &msg).unwrap();
        assert_eq!(buf.iter().filter(|&&b| b == b'\n').count(), 2);

        let mut r = io::Cursor::new(buf);
        let a: CoordinatorToSim = recv(&mut r).unwrap().unwrap();
        let b: CoordinatorToSim = recv(&mut r).unwrap().unwrap();
        assert_eq!(a, msg);
        assert_eq!(b, msg);
        assert!(recv::<_, CoordinatorToSim>(&mut r).unwrap().is_none());
    }

    #[test]
    fn writer_failure_is_an_io_error() {
        struct Closed;
        impl Write for Closed {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let err = send(&mut Closed, &Decision::empty(0)).unwrap_err();
        assert!(
            matches!(&err, WireError::Io(e) if e.kind() == io::ErrorKind::BrokenPipe),
            "{err:?}"
        );
    }
}
