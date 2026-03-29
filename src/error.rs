use std::fmt::{Display, Formatter};

/// The category of a [`WatchDirError`].
#[derive(Debug)]
pub enum ErrorKind {
    /// An error from the underlying `notify` filesystem watcher.
    Notify(notify::Error),
    /// An I/O error encountered while reading a file or scanning a directory.
    Io(std::io::Error),
}

/// Error type returned by this crate.
///
/// Use [`WatchDirError::kind`] to inspect the underlying cause.
#[derive(Debug)]
pub struct WatchDirError {
    kind: ErrorKind,
}

impl WatchDirError {
    /// Returns the underlying [`ErrorKind`].
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl Display for WatchDirError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Notify(e) => write!(f, "notify error: {}", e),
            ErrorKind::Io(e) => write!(f, "io error: {}", e),
        }
    }
}

impl From<notify::Error> for WatchDirError {
    fn from(e: notify::Error) -> Self {
        Self {
            kind: ErrorKind::Notify(e),
        }
    }
}

impl From<std::io::Error> for WatchDirError {
    fn from(e: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(e),
        }
    }
}
